/**
 * Contract Deployment Utility
 *
 * Handles contract deployment transactions via the Tauri backend.
 * Encodes constructor parameters and sends deployment transaction.
 */

import { invoke } from '@tauri-apps/api/core';
import { encodeConstructorParams } from './abiEncoder';
import { rpcClient } from '../services/rpc-client';

export interface DeploymentRequest {
  from: string;
  bytecode: string;
  abi: any[];
  constructorArgs?: any[];
  gasLimit?: number;
  gasPrice?: string;
  password: string;
}

export interface DeploymentResult {
  txHash: string;
  contractAddress?: string; // Will be derived from txHash and nonce
}

/**
 * Deploy a smart contract to Citrate
 *
 * @param request - Deployment parameters
 * @returns Transaction hash and estimated contract address
 *
 * @example
 * ```typescript
 * const result = await deployContract({
 *   from: '0xabc...',
 *   bytecode: compilationResult.bytecode,
 *   abi: compilationResult.abi,
 *   constructorArgs: ['MyToken', 'MTK', 1000000],
 *   password: 'user-password',
 * });
 * console.log('Deployed at:', result.contractAddress);
 * ```
 */
export async function deployContract(
  request: DeploymentRequest
): Promise<DeploymentResult> {
  console.log('[ContractDeployment] Deploying contract from:', request.from);

  // Encode constructor arguments if present
  let deploymentData = request.bytecode;

  if (request.constructorArgs && request.constructorArgs.length > 0) {
    // Find constructor in ABI
    const constructor = request.abi.find((item: any) => item.type === 'constructor');

    if (constructor && constructor.inputs) {
      console.log('[ContractDeployment] Encoding constructor args:', request.constructorArgs);

      // Use ABI encoder to encode constructor parameters
      const types = constructor.inputs.map((input: any) => input.type);
      const encodedArgs = encodeConstructorParams(types, request.constructorArgs);

      deploymentData = deploymentData + encodedArgs;
    }
  }

  // Prepare transaction request
  const txRequest = {
    from: request.from,
    to: null, // null = contract creation
    value: '0', // No ETH sent unless payable constructor
    gasLimit: request.gasLimit || 5000000, // 5M gas default
    gasPrice: request.gasPrice || '1000000000', // 1 Gwei default
    data: deploymentData,
  };

  console.log('[ContractDeployment] Sending deployment transaction:', {
    from: txRequest.from,
    gasLimit: txRequest.gasLimit,
    dataLength: deploymentData.length,
  });

  // Send transaction via Tauri
  try {
    const txHash = await invoke<string>('send_transaction', {
      request: txRequest,
      password: request.password,
    });

    console.log('[ContractDeployment] Deployment transaction sent:', txHash);

    // Calculate contract address (CREATE opcode formula)
    // address = keccak256(rlp([sender, nonce]))[12:]
    // For now, we'll need to fetch the receipt to get the actual address
    const contractAddress = await getContractAddressFromReceipt(txHash);

    return {
      txHash,
      contractAddress,
    };
  } catch (error: any) {
    console.error('[ContractDeployment] Deployment failed:', error);
    throw new Error(`Contract deployment failed: ${error.message || error}`);
  }
}

/**
 * Get the deployed contract address from a transaction receipt
 *
 * @param txHash - Transaction hash
 * @returns Contract address or undefined if not yet mined
 */
async function getContractAddressFromReceipt(
  txHash: string
): Promise<string | undefined> {
  // TODO: Implement receipt fetching via RPC
  // For now, return undefined (address will be shown after receipt)

  try {
    const receipt = await rpcClient.getTransactionReceipt(txHash);
    if (receipt && receipt.contractAddress) {
      console.log('[ContractDeployment] Contract address found:', receipt.contractAddress);
      return receipt.contractAddress;
    }
    console.log('[ContractDeployment] Receipt found but no contract address (transaction may not be mined yet)');
    return undefined;
  } catch (error) {
    console.log('[ContractDeployment] Could not get receipt (transaction may not be mined yet):', error);
    return undefined;
  }
}

/**
 * Wait for a deployment transaction to be confirmed
 *
 * @param txHash - Transaction hash
 * @param timeout - Maximum wait time in milliseconds
 * @returns Contract address
 */
export async function waitForDeployment(
  txHash: string,
  timeout: number = 60000
): Promise<string> {
  const startTime = Date.now();
  const pollInterval = 2000; // 2 seconds

  console.log(`[ContractDeployment] Waiting for deployment confirmation (${txHash.slice(0, 10)}...)`);

  while (Date.now() - startTime < timeout) {
    try {
      const receipt = await rpcClient.getTransactionReceipt(txHash);
      if (receipt) {
        // Check if transaction was successful
        if (receipt.status === false || receipt.status === '0x0') {
          throw new Error('Contract deployment transaction failed');
        }

        if (receipt.contractAddress) {
          console.log('[ContractDeployment] Deployment confirmed! Contract address:', receipt.contractAddress);
          return receipt.contractAddress;
        }

        // Receipt exists but no contract address - might be a regular tx not a deployment
        console.warn('[ContractDeployment] Receipt found but no contractAddress - may not be a deployment tx');
        throw new Error('Transaction confirmed but no contract address - verify this is a deployment transaction');
      }

      // Receipt not yet available - transaction still pending
      const elapsed = Math.round((Date.now() - startTime) / 1000);
      console.log(`[ContractDeployment] Still waiting... (${elapsed}s elapsed)`);
      await new Promise(resolve => setTimeout(resolve, pollInterval));
    } catch (error) {
      // If error is from our own validation, re-throw it
      if (error instanceof Error && error.message.includes('failed')) {
        throw error;
      }
      // Otherwise it's likely RPC connection issue - log and continue polling
      console.debug('[ContractDeployment] Polling attempt failed:', error);
      await new Promise(resolve => setTimeout(resolve, pollInterval));
    }
  }

  throw new Error(`Deployment confirmation timeout after ${timeout}ms. Transaction may still be pending.`);
}

/**
 * Estimate gas for contract deployment
 *
 * @param from - Sender address
 * @param bytecode - Contract bytecode with constructor args
 * @returns Estimated gas
 */
export async function estimateDeploymentGas(
  from: string,
  bytecode: string
): Promise<number> {
  try {
    // Use RPC to get accurate gas estimation
    const estimateStr = await rpcClient.estimateGas({
      from,
      data: bytecode.startsWith('0x') ? bytecode : `0x${bytecode}`,
    });

    const estimate = parseInt(estimateStr, 10);
    console.log('[ContractDeployment] Gas estimate from RPC:', estimate);

    // Add 20% buffer for safety
    const bufferedEstimate = Math.ceil(estimate * 1.2);
    console.log('[ContractDeployment] Buffered gas estimate:', bufferedEstimate);

    return bufferedEstimate;
  } catch (error) {
    console.warn('[ContractDeployment] RPC gas estimation failed, using fallback calculation:', error);

    // Fallback: use a conservative estimate based on bytecode size
    const bytecodeLength = (bytecode.length - 2) / 2; // Remove '0x' and convert hex to bytes
    const baseGas = 32000; // Transaction base cost
    const creationGas = 32000; // Contract creation cost
    const codeDepositGas = bytecodeLength * 200; // Code storage cost
    const calldataGas = bytecodeLength * 68; // Calldata cost

    const fallbackEstimate = baseGas + creationGas + codeDepositGas + calldataGas;
    console.log('[ContractDeployment] Fallback gas estimate:', fallbackEstimate);

    return fallbackEstimate;
  }
}
