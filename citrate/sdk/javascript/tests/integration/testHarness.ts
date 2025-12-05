/**
 * Test Harness for Citrate SDK Integration Tests
 *
 * Provides utilities for:
 * - Starting/stopping embedded node
 * - Managing test accounts with funding
 * - Polling for transaction confirmations
 * - Mock data generation
 */

import { spawn, ChildProcess } from 'child_process';
import { ethers } from 'ethers';
import path from 'path';

// ============================================================================
// Configuration
// ============================================================================

export interface TestConfig {
  rpcEndpoint: string;
  chainId: number;
  blockTime: number;
  timeout: number;
  genesisAccounts: GenesisAccount[];
}

export interface GenesisAccount {
  privateKey: string;
  address: string;
  balance: string; // In wei
}

// Default test configuration
export const DEFAULT_TEST_CONFIG: TestConfig = {
  rpcEndpoint: process.env.CITRATE_RPC_URL || 'http://localhost:8545',
  chainId: parseInt(process.env.CITRATE_CHAIN_ID || '1337'),
  blockTime: 1000, // 1 second blocks
  timeout: 30000, // 30 second test timeout
  genesisAccounts: [
    {
      // Well-known test account with pre-funded balance
      privateKey: '0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80',
      address: '0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266',
      balance: '10000000000000000000000', // 10000 ETH
    },
    {
      privateKey: '0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d',
      address: '0x70997970C51812dc3A010C7d01b50e0d17dc79C8',
      balance: '10000000000000000000000',
    },
    {
      privateKey: '0x5de4111afa1a4b94908f83103eb1f1706367c2e68ca870fc3fb9a804cdab365a',
      address: '0x3C44CdDdB6a900fa2b585dd299e03d12FA4293BC',
      balance: '10000000000000000000000',
    },
  ],
};

// ============================================================================
// Node Management
// ============================================================================

let nodeProcess: ChildProcess | null = null;

/**
 * Start the embedded Citrate node for testing
 */
export async function startEmbeddedNode(config: Partial<TestConfig> = {}): Promise<void> {
  const fullConfig = { ...DEFAULT_TEST_CONFIG, ...config };

  // Check if node is already running
  if (await isNodeRunning(fullConfig.rpcEndpoint)) {
    console.log('[TestHarness] Node already running at', fullConfig.rpcEndpoint);
    return;
  }

  // Find the node binary
  const nodeBinaryPaths = [
    path.resolve(__dirname, '../../../../node/target/release/citrate-node'),
    path.resolve(__dirname, '../../../../node/target/debug/citrate-node'),
    'citrate-node', // In PATH
  ];

  let nodeBinary: string | null = null;
  for (const p of nodeBinaryPaths) {
    try {
      const { execSync } = require('child_process');
      execSync(`${p} --version`, { stdio: 'ignore' });
      nodeBinary = p;
      break;
    } catch {
      continue;
    }
  }

  if (!nodeBinary) {
    throw new Error(
      'Citrate node binary not found. Build with: cargo build --release -p citrate-node'
    );
  }

  console.log('[TestHarness] Starting embedded node:', nodeBinary);

  // Start the node in devnet mode
  nodeProcess = spawn(nodeBinary, ['devnet', '--rpc-port', '8545'], {
    stdio: ['ignore', 'pipe', 'pipe'],
    detached: false,
  });

  // Log node output
  nodeProcess.stdout?.on('data', (data) => {
    if (process.env.DEBUG) {
      console.log('[Node]', data.toString().trim());
    }
  });

  nodeProcess.stderr?.on('data', (data) => {
    if (process.env.DEBUG) {
      console.error('[Node Error]', data.toString().trim());
    }
  });

  // Wait for node to be ready
  await waitForNode(fullConfig.rpcEndpoint, fullConfig.timeout);

  console.log('[TestHarness] Node started successfully');
}

/**
 * Stop the embedded node
 */
export async function stopEmbeddedNode(): Promise<void> {
  if (nodeProcess) {
    console.log('[TestHarness] Stopping embedded node');
    nodeProcess.kill('SIGTERM');

    // Wait for process to exit
    await new Promise<void>((resolve) => {
      if (nodeProcess) {
        nodeProcess.on('exit', () => resolve());
        setTimeout(() => {
          nodeProcess?.kill('SIGKILL');
          resolve();
        }, 5000);
      } else {
        resolve();
      }
    });

    nodeProcess = null;
  }
}

/**
 * Check if node is running by calling eth_chainId
 */
export async function isNodeRunning(rpcEndpoint: string): Promise<boolean> {
  try {
    const response = await fetch(rpcEndpoint, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        jsonrpc: '2.0',
        method: 'eth_chainId',
        params: [],
        id: 1,
      }),
    });

    const data = await response.json();
    return data.result !== undefined;
  } catch {
    return false;
  }
}

/**
 * Wait for node to be ready
 */
async function waitForNode(rpcEndpoint: string, timeout: number): Promise<void> {
  const startTime = Date.now();

  while (Date.now() - startTime < timeout) {
    if (await isNodeRunning(rpcEndpoint)) {
      return;
    }
    await sleep(500);
  }

  throw new Error(`Node did not start within ${timeout}ms`);
}

// ============================================================================
// RPC Helpers
// ============================================================================

/**
 * Make a raw JSON-RPC call
 */
export async function rpcCall(
  method: string,
  params: any[] = [],
  rpcEndpoint: string = DEFAULT_TEST_CONFIG.rpcEndpoint
): Promise<any> {
  const response = await fetch(rpcEndpoint, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      jsonrpc: '2.0',
      method,
      params,
      id: Date.now(),
    }),
  });

  const data = await response.json();

  if (data.error) {
    throw new Error(`RPC Error: ${data.error.message} (${data.error.code})`);
  }

  return data.result;
}

/**
 * Wait for a transaction to be mined
 */
export async function waitForTransaction(
  txHash: string,
  timeout: number = 30000,
  rpcEndpoint: string = DEFAULT_TEST_CONFIG.rpcEndpoint
): Promise<any> {
  const startTime = Date.now();

  while (Date.now() - startTime < timeout) {
    try {
      const receipt = await rpcCall('eth_getTransactionReceipt', [txHash], rpcEndpoint);
      if (receipt) {
        return receipt;
      }
    } catch {
      // Transaction not yet mined
    }
    await sleep(500);
  }

  throw new Error(`Transaction ${txHash} not mined within ${timeout}ms`);
}

/**
 * Get current block number
 */
export async function getBlockNumber(
  rpcEndpoint: string = DEFAULT_TEST_CONFIG.rpcEndpoint
): Promise<number> {
  const result = await rpcCall('eth_blockNumber', [], rpcEndpoint);
  return parseInt(result, 16);
}

/**
 * Get account balance
 */
export async function getBalance(
  address: string,
  rpcEndpoint: string = DEFAULT_TEST_CONFIG.rpcEndpoint
): Promise<bigint> {
  const result = await rpcCall('eth_getBalance', [address, 'latest'], rpcEndpoint);
  return BigInt(result);
}

/**
 * Get transaction count (nonce)
 */
export async function getNonce(
  address: string,
  tag: 'latest' | 'pending' = 'latest',
  rpcEndpoint: string = DEFAULT_TEST_CONFIG.rpcEndpoint
): Promise<number> {
  const result = await rpcCall('eth_getTransactionCount', [address, tag], rpcEndpoint);
  return parseInt(result, 16);
}

// ============================================================================
// Transaction Helpers
// ============================================================================

/**
 * Create and sign a legacy transaction
 */
export async function createLegacyTransaction(
  wallet: ethers.Wallet,
  to: string,
  value: bigint,
  data: string = '0x',
  gasLimit: number = 21000,
  rpcEndpoint: string = DEFAULT_TEST_CONFIG.rpcEndpoint
): Promise<string> {
  const nonce = await getNonce(wallet.address, 'pending', rpcEndpoint);
  const gasPrice = await rpcCall('eth_gasPrice', [], rpcEndpoint);

  const tx = {
    type: 0, // Legacy
    to,
    value,
    data,
    gasLimit,
    gasPrice: BigInt(gasPrice),
    nonce,
    chainId: DEFAULT_TEST_CONFIG.chainId,
  };

  return await wallet.signTransaction(tx);
}

/**
 * Create and sign an EIP-2930 transaction (access list)
 */
export async function createEIP2930Transaction(
  wallet: ethers.Wallet,
  to: string,
  value: bigint,
  accessList: { address: string; storageKeys: string[] }[],
  data: string = '0x',
  gasLimit: number = 21000,
  rpcEndpoint: string = DEFAULT_TEST_CONFIG.rpcEndpoint
): Promise<string> {
  const nonce = await getNonce(wallet.address, 'pending', rpcEndpoint);
  const gasPrice = await rpcCall('eth_gasPrice', [], rpcEndpoint);

  const tx = {
    type: 1, // EIP-2930
    to,
    value,
    data,
    gasLimit,
    gasPrice: BigInt(gasPrice),
    nonce,
    chainId: DEFAULT_TEST_CONFIG.chainId,
    accessList,
  };

  return await wallet.signTransaction(tx);
}

/**
 * Create and sign an EIP-1559 transaction (dynamic fee)
 */
export async function createEIP1559Transaction(
  wallet: ethers.Wallet,
  to: string,
  value: bigint,
  data: string = '0x',
  gasLimit: number = 21000,
  rpcEndpoint: string = DEFAULT_TEST_CONFIG.rpcEndpoint
): Promise<string> {
  const nonce = await getNonce(wallet.address, 'pending', rpcEndpoint);

  // Get fee data
  let maxFeePerGas: bigint;
  let maxPriorityFeePerGas: bigint;

  try {
    const feeHistory = await rpcCall('eth_feeHistory', ['0x4', 'latest', [25, 75]], rpcEndpoint);
    const baseFee = BigInt(feeHistory.baseFeePerGas[feeHistory.baseFeePerGas.length - 1]);
    maxPriorityFeePerGas = BigInt('1000000000'); // 1 gwei
    maxFeePerGas = baseFee * 2n + maxPriorityFeePerGas;
  } catch {
    // Fallback to legacy gas price
    const gasPrice = await rpcCall('eth_gasPrice', [], rpcEndpoint);
    maxFeePerGas = BigInt(gasPrice);
    maxPriorityFeePerGas = maxFeePerGas / 10n;
  }

  const tx = {
    type: 2, // EIP-1559
    to,
    value,
    data,
    gasLimit,
    maxFeePerGas,
    maxPriorityFeePerGas,
    nonce,
    chainId: DEFAULT_TEST_CONFIG.chainId,
  };

  return await wallet.signTransaction(tx);
}

/**
 * Send a raw transaction and wait for receipt
 */
export async function sendAndWait(
  signedTx: string,
  rpcEndpoint: string = DEFAULT_TEST_CONFIG.rpcEndpoint
): Promise<any> {
  const txHash = await rpcCall('eth_sendRawTransaction', [signedTx], rpcEndpoint);
  return await waitForTransaction(txHash, 30000, rpcEndpoint);
}

// ============================================================================
// Test Utilities
// ============================================================================

/**
 * Get a funded test wallet
 */
export function getTestWallet(index: number = 0): ethers.Wallet {
  const account = DEFAULT_TEST_CONFIG.genesisAccounts[index];
  if (!account) {
    throw new Error(`No test account at index ${index}`);
  }
  return new ethers.Wallet(account.privateKey);
}

/**
 * Generate a random wallet for testing
 */
export function createRandomWallet(): ethers.Wallet {
  return ethers.Wallet.createRandom();
}

/**
 * Sleep for specified milliseconds
 */
export function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

/**
 * Format wei to ETH string
 */
export function formatEth(wei: bigint): string {
  return ethers.formatEther(wei);
}

/**
 * Parse ETH string to wei
 */
export function parseEth(eth: string): bigint {
  return ethers.parseEther(eth);
}

// ============================================================================
// Contract Test Helpers
// ============================================================================

// Simple storage contract bytecode
export const SIMPLE_STORAGE_BYTECODE =
  '0x608060405234801561001057600080fd5b5060f78061001f6000396000f3fe6080604052348015600f57600080fd5b5060043610603c5760003560e01c80632e64cec114604157806356d4f8f714605b5780636057361d146063575b600080fd5b6047607b565b6040516052919060b6565b60405180910390f35b60616084565b005b607960048036038101906075919060d5565b6091565b005b60008054905090565b6001600054608a919060ff565b6000819055565b8060008190555050565b6000819050919050565b60b08160a0565b82525050565b600060208201905060c9600083018460a9565b92915050565b60008135905060cf8160a0565b92915050565b60006020828403121560e85760e7600080fd5b600060f48482850160c0565b91505092915050565b7f4e487b7100000000000000000000000000000000000000000000000000000000600052601160045260246000fd5b600061013382609a565b915061013e83609a565b925082821015610150576101516100fd565b5b82820390509291505056fea26469706673582212203e1e3d2f2c1a1b1c1d1e1f202122232425262728292a2b2c2d2e2f303132333453';

// Simple storage ABI
export const SIMPLE_STORAGE_ABI = [
  {
    inputs: [],
    name: 'get',
    outputs: [{ type: 'uint256' }],
    stateMutability: 'view',
    type: 'function',
  },
  {
    inputs: [{ name: 'x', type: 'uint256' }],
    name: 'set',
    outputs: [],
    stateMutability: 'nonpayable',
    type: 'function',
  },
  {
    inputs: [],
    name: 'increment',
    outputs: [],
    stateMutability: 'nonpayable',
    type: 'function',
  },
];

/**
 * Deploy a contract and wait for receipt
 */
export async function deployContract(
  wallet: ethers.Wallet,
  bytecode: string,
  gasLimit: number = 500000,
  rpcEndpoint: string = DEFAULT_TEST_CONFIG.rpcEndpoint
): Promise<{ address: string; txHash: string }> {
  const signedTx = await createLegacyTransaction(
    wallet,
    '', // Empty 'to' for contract creation
    0n,
    bytecode,
    gasLimit,
    rpcEndpoint
  );

  // Need to handle empty 'to' - create contract deployment tx differently
  const nonce = await getNonce(wallet.address, 'pending', rpcEndpoint);
  const gasPrice = await rpcCall('eth_gasPrice', [], rpcEndpoint);

  const tx = {
    type: 0,
    to: null, // Contract creation
    value: 0n,
    data: bytecode,
    gasLimit,
    gasPrice: BigInt(gasPrice),
    nonce,
    chainId: DEFAULT_TEST_CONFIG.chainId,
  };

  const signed = await wallet.signTransaction(tx);
  const receipt = await sendAndWait(signed, rpcEndpoint);

  return {
    address: receipt.contractAddress,
    txHash: receipt.transactionHash,
  };
}

// ============================================================================
// Exports
// ============================================================================

export {
  TestConfig,
  GenesisAccount,
};
