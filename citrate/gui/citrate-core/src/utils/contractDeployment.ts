/**
 * Contract Deployment Utility
 *
 * Handles contract deployment transactions via the Tauri backend.
 * Encodes constructor parameters and sends deployment transaction.
 */

import { invoke } from '@tauri-apps/api/core';
import { encodeConstructorParams } from './abiEncoder';

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

  console.log(
    '[ContractDeployment] Contract address retrieval not yet implemented.',
    'Waiting for receipt:', txHash
  );

  // In production, poll for receipt:
  // const receipt = await invoke('get_transaction_receipt', { txHash });
  // return receipt.contractAddress;

  return undefined;
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

  while (Date.now() - startTime < timeout) {
    try {
      // TODO: Implement receipt polling via RPC
      // const receipt = await invoke('get_transaction_receipt', { txHash });
      // if (receipt && receipt.contractAddress) {
      //   return receipt.contractAddress;
      // }

      console.log(`[ContractDeployment] Waiting for deployment confirmation... (${txHash.slice(0, 10)}...)`);
      await new Promise(resolve => setTimeout(resolve, pollInterval));
    } catch (error) {
      console.error('[ContractDeployment] Error polling receipt:', error);
    }
  }

  throw new Error(`Deployment confirmation timeout after ${timeout}ms`);
}

/**
 * Estimate gas for contract deployment
 *
 * @param bytecode - Contract bytecode with constructor args
 * @returns Estimated gas
 */
export async function estimateDeploymentGas(
  bytecode: string
): Promise<number> {
  // TODO: Implement gas estimation via RPC
  // For now, use a conservative estimate based on bytecode size

  const bytecodeLength = (bytecode.length - 2) / 2; // Remove '0x' and convert hex to bytes
  const baseGas = 32000; // Transaction base cost
  const creationGas = 32000; // Contract creation cost
  const codeDepositGas = bytecodeLength * 200; // Code storage cost
  const calldataGas = bytecodeLength * 68; // Calldata cost

  const estimate = baseGas + creationGas + codeDepositGas + calldataGas;

  console.log('[ContractDeployment] Gas estimate:', estimate);
  return estimate;
}
