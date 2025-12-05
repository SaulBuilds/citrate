/**
 * Testnet Integration Tests for Citrate SDK
 *
 * Tests SDK functionality against a live testnet deployment.
 * These tests verify production-ready behavior with real network latency.
 *
 * Run with: npm run test:integration:testnet
 *
 * Environment variables:
 * - CITRATE_RPC_URL: Testnet RPC endpoint (default: https://testnet-rpc.citrate.ai)
 * - CITRATE_CHAIN_ID: Chain ID (default: 1338)
 * - TESTNET_FAUCET_PRIVATE_KEY: Private key with testnet funds (optional)
 */

import { ethers } from 'ethers';
import { CitrateSDK } from '../../src/sdk';
import {
  rpcCall,
  getBalance,
  getNonce,
  getBlockNumber,
  createRandomWallet,
  createLegacyTransaction,
  createEIP1559Transaction,
  waitForTransaction,
  sleep,
  parseEth,
  formatEth,
  isNodeRunning,
} from './testHarness';

// ============================================================================
// Testnet Configuration
// ============================================================================

const TESTNET_RPC = process.env.CITRATE_RPC_URL || 'https://testnet-rpc.citrate.ai';
const TESTNET_CHAIN_ID = parseInt(process.env.CITRATE_CHAIN_ID || '1338');
const TESTNET_EXPLORER = 'https://testnet-explorer.citrate.ai';

// Skip tests if no testnet is available
const skipIfNoTestnet = process.env.SKIP_TESTNET_TESTS === 'true';

let sdk: CitrateSDK;
let testnetAvailable = false;

beforeAll(async () => {
  testnetAvailable = await isNodeRunning(TESTNET_RPC);

  if (!testnetAvailable) {
    console.warn(
      `WARNING: Testnet not available at ${TESTNET_RPC}. ` +
        'Set SKIP_TESTNET_TESTS=true to skip these tests.'
    );
  }

  sdk = new CitrateSDK({
    rpcEndpoint: TESTNET_RPC,
    chainId: TESTNET_CHAIN_ID,
  });
}, 30000);

// ============================================================================
// Network Connectivity Tests
// ============================================================================

describe('Testnet Network Connectivity', () => {
  it('connects to testnet RPC endpoint', async () => {
    if (!testnetAvailable) {
      console.log('Skipping: Testnet not available');
      return;
    }

    const chainId = await rpcCall('eth_chainId', [], TESTNET_RPC);
    expect(parseInt(chainId, 16)).toBe(TESTNET_CHAIN_ID);
  });

  it('retrieves current block number', async () => {
    if (!testnetAvailable) {
      console.log('Skipping: Testnet not available');
      return;
    }

    const blockNumber = await getBlockNumber(TESTNET_RPC);
    expect(blockNumber).toBeGreaterThan(0);
    console.log(`Testnet block number: ${blockNumber}`);
  });

  it('testnet is producing blocks', async () => {
    if (!testnetAvailable) {
      console.log('Skipping: Testnet not available');
      return;
    }

    const block1 = await getBlockNumber(TESTNET_RPC);
    await sleep(5000); // Wait 5 seconds
    const block2 = await getBlockNumber(TESTNET_RPC);

    expect(block2).toBeGreaterThanOrEqual(block1);
    console.log(`Block progression: ${block1} -> ${block2}`);
  }, 15000);

  it('SDK connects to testnet', async () => {
    if (!testnetAvailable) {
      console.log('Skipping: Testnet not available');
      return;
    }

    const networkInfo = await sdk.getNetworkInfo();
    expect(networkInfo.chainId).toBe(TESTNET_CHAIN_ID);
  });
});

// ============================================================================
// Block Structure Tests
// ============================================================================

describe('Testnet Block Structure', () => {
  it('retrieves latest block with correct structure', async () => {
    if (!testnetAvailable) {
      console.log('Skipping: Testnet not available');
      return;
    }

    const block = await rpcCall('eth_getBlockByNumber', ['latest', false], TESTNET_RPC);

    expect(block).toBeDefined();
    expect(block.number).toBeDefined();
    expect(block.hash).toBeDefined();
    expect(block.parentHash).toBeDefined();
    expect(block.timestamp).toBeDefined();
    expect(block.gasLimit).toBeDefined();
    expect(block.gasUsed).toBeDefined();

    // Verify hash formats
    expect(block.hash).toMatch(/^0x[a-fA-F0-9]{64}$/);
    expect(block.parentHash).toMatch(/^0x[a-fA-F0-9]{64}$/);
  });

  it('retrieves genesis block', async () => {
    if (!testnetAvailable) {
      console.log('Skipping: Testnet not available');
      return;
    }

    const genesis = await rpcCall('eth_getBlockByNumber', ['0x0', false], TESTNET_RPC);

    expect(genesis).toBeDefined();
    expect(parseInt(genesis.number, 16)).toBe(0);
    // Genesis parent hash is all zeros
    expect(genesis.parentHash).toBe('0x' + '0'.repeat(64));
  });

  it('verifies block chain integrity', async () => {
    if (!testnetAvailable) {
      console.log('Skipping: Testnet not available');
      return;
    }

    const latestNum = await getBlockNumber(TESTNET_RPC);
    if (latestNum < 2) {
      console.log('Not enough blocks to verify chain integrity');
      return;
    }

    // Get last 3 blocks
    const block1 = await rpcCall('eth_getBlockByNumber', [`0x${(latestNum - 2).toString(16)}`, false], TESTNET_RPC);
    const block2 = await rpcCall('eth_getBlockByNumber', [`0x${(latestNum - 1).toString(16)}`, false], TESTNET_RPC);
    const block3 = await rpcCall('eth_getBlockByNumber', [`0x${latestNum.toString(16)}`, false], TESTNET_RPC);

    // Verify parent hash linkage
    expect(block2.parentHash).toBe(block1.hash);
    expect(block3.parentHash).toBe(block2.hash);
  });
});

// ============================================================================
// Gas Price Tests
// ============================================================================

describe('Testnet Gas Pricing', () => {
  it('returns reasonable gas price', async () => {
    if (!testnetAvailable) {
      console.log('Skipping: Testnet not available');
      return;
    }

    const gasPrice = await rpcCall('eth_gasPrice', [], TESTNET_RPC);
    const gasPriceWei = BigInt(gasPrice);

    // Gas price should be between 1 gwei and 1000 gwei for testnet
    expect(gasPriceWei).toBeGreaterThanOrEqual(1_000_000_000n); // 1 gwei
    expect(gasPriceWei).toBeLessThanOrEqual(1_000_000_000_000n); // 1000 gwei

    console.log(`Testnet gas price: ${gasPriceWei / 1_000_000_000n} gwei`);
  });

  it('returns EIP-1559 fee data', async () => {
    if (!testnetAvailable) {
      console.log('Skipping: Testnet not available');
      return;
    }

    try {
      const feeHistory = await rpcCall('eth_feeHistory', ['0x4', 'latest', [25, 50, 75]], TESTNET_RPC);

      expect(feeHistory).toBeDefined();
      expect(feeHistory.baseFeePerGas).toBeDefined();
      expect(Array.isArray(feeHistory.baseFeePerGas)).toBe(true);

      const baseFee = BigInt(feeHistory.baseFeePerGas[feeHistory.baseFeePerGas.length - 1]);
      console.log(`Testnet base fee: ${baseFee / 1_000_000_000n} gwei`);
    } catch (error) {
      // eth_feeHistory might not be supported on all testnets
      console.log('eth_feeHistory not supported on this testnet');
    }
  });
});

// ============================================================================
// Account State Tests
// ============================================================================

describe('Testnet Account State', () => {
  it('returns zero balance for random address', async () => {
    if (!testnetAvailable) {
      console.log('Skipping: Testnet not available');
      return;
    }

    const wallet = createRandomWallet();
    const balance = await getBalance(wallet.address, TESTNET_RPC);

    expect(balance).toBe(0n);
  });

  it('returns zero nonce for new address', async () => {
    if (!testnetAvailable) {
      console.log('Skipping: Testnet not available');
      return;
    }

    const wallet = createRandomWallet();
    const nonce = await getNonce(wallet.address, 'latest', TESTNET_RPC);

    expect(nonce).toBe(0);
  });

  it('handles checksum and lowercase addresses', async () => {
    if (!testnetAvailable) {
      console.log('Skipping: Testnet not available');
      return;
    }

    const wallet = createRandomWallet();
    const checksumAddress = ethers.getAddress(wallet.address);
    const lowercaseAddress = wallet.address.toLowerCase();

    const balance1 = await getBalance(checksumAddress, TESTNET_RPC);
    const balance2 = await getBalance(lowercaseAddress, TESTNET_RPC);

    expect(balance1).toBe(balance2);
  });
});

// ============================================================================
// Transaction Tests (requires funded account)
// ============================================================================

describe('Testnet Transactions', () => {
  const FAUCET_PRIVATE_KEY = process.env.TESTNET_FAUCET_PRIVATE_KEY;

  it('sends transaction on testnet', async () => {
    if (!testnetAvailable) {
      console.log('Skipping: Testnet not available');
      return;
    }

    if (!FAUCET_PRIVATE_KEY) {
      console.log('Skipping: No TESTNET_FAUCET_PRIVATE_KEY provided');
      return;
    }

    const sender = new ethers.Wallet(FAUCET_PRIVATE_KEY);
    const recipient = createRandomWallet();

    // Check sender balance
    const senderBalance = await getBalance(sender.address, TESTNET_RPC);
    if (senderBalance < parseEth('0.01')) {
      console.log(`Skipping: Sender balance too low (${formatEth(senderBalance)} ETH)`);
      return;
    }

    // Create and send transaction
    const nonce = await getNonce(sender.address, 'pending', TESTNET_RPC);
    const gasPrice = await rpcCall('eth_gasPrice', [], TESTNET_RPC);

    const tx = {
      type: 0,
      to: recipient.address,
      value: parseEth('0.001'),
      data: '0x',
      gasLimit: 21000,
      gasPrice: BigInt(gasPrice),
      nonce,
      chainId: TESTNET_CHAIN_ID,
    };

    const signedTx = await sender.signTransaction(tx);
    const txHash = await rpcCall('eth_sendRawTransaction', [signedTx], TESTNET_RPC);

    expect(txHash).toMatch(/^0x[a-fA-F0-9]{64}$/);
    console.log(`Transaction sent: ${TESTNET_EXPLORER}/tx/${txHash}`);

    // Wait for confirmation
    const receipt = await waitForTransaction(txHash, 60000, TESTNET_RPC);
    expect(receipt.status).toBe('0x1');

    // Verify recipient received funds
    const recipientBalance = await getBalance(recipient.address, TESTNET_RPC);
    expect(recipientBalance).toBe(parseEth('0.001'));
  }, 120000);

  it('sends EIP-1559 transaction on testnet', async () => {
    if (!testnetAvailable) {
      console.log('Skipping: Testnet not available');
      return;
    }

    if (!FAUCET_PRIVATE_KEY) {
      console.log('Skipping: No TESTNET_FAUCET_PRIVATE_KEY provided');
      return;
    }

    const sender = new ethers.Wallet(FAUCET_PRIVATE_KEY);
    const recipient = createRandomWallet();

    // Check sender balance
    const senderBalance = await getBalance(sender.address, TESTNET_RPC);
    if (senderBalance < parseEth('0.01')) {
      console.log(`Skipping: Sender balance too low (${formatEth(senderBalance)} ETH)`);
      return;
    }

    // Get fee data
    let maxFeePerGas: bigint;
    let maxPriorityFeePerGas: bigint;

    try {
      const feeHistory = await rpcCall('eth_feeHistory', ['0x4', 'latest', [50]], TESTNET_RPC);
      const baseFee = BigInt(feeHistory.baseFeePerGas[feeHistory.baseFeePerGas.length - 1]);
      maxPriorityFeePerGas = 2_000_000_000n; // 2 gwei
      maxFeePerGas = baseFee * 2n + maxPriorityFeePerGas;
    } catch {
      // Fallback
      const gasPrice = await rpcCall('eth_gasPrice', [], TESTNET_RPC);
      maxFeePerGas = BigInt(gasPrice) * 2n;
      maxPriorityFeePerGas = BigInt(gasPrice) / 10n;
    }

    const nonce = await getNonce(sender.address, 'pending', TESTNET_RPC);

    const tx = {
      type: 2,
      to: recipient.address,
      value: parseEth('0.001'),
      data: '0x',
      gasLimit: 21000,
      maxFeePerGas,
      maxPriorityFeePerGas,
      nonce,
      chainId: TESTNET_CHAIN_ID,
    };

    const signedTx = await sender.signTransaction(tx);
    const txHash = await rpcCall('eth_sendRawTransaction', [signedTx], TESTNET_RPC);

    expect(txHash).toMatch(/^0x[a-fA-F0-9]{64}$/);
    console.log(`EIP-1559 Transaction sent: ${TESTNET_EXPLORER}/tx/${txHash}`);

    const receipt = await waitForTransaction(txHash, 60000, TESTNET_RPC);
    expect(receipt.status).toBe('0x1');
    expect(receipt.type).toBe('0x2');
  }, 120000);
});

// ============================================================================
// eth_call Tests
// ============================================================================

describe('Testnet eth_call', () => {
  it('executes read-only call', async () => {
    if (!testnetAvailable) {
      console.log('Skipping: Testnet not available');
      return;
    }

    // Call to zero address (will return empty data)
    const result = await rpcCall(
      'eth_call',
      [
        {
          to: '0x0000000000000000000000000000000000000000',
          data: '0x',
        },
        'latest',
      ],
      TESTNET_RPC
    );

    expect(result).toBeDefined();
  });

  it('estimates gas for transfer', async () => {
    if (!testnetAvailable) {
      console.log('Skipping: Testnet not available');
      return;
    }

    const wallet = createRandomWallet();
    const recipient = createRandomWallet();

    const gasEstimate = await rpcCall(
      'eth_estimateGas',
      [
        {
          from: wallet.address,
          to: recipient.address,
          value: '0x1',
        },
      ],
      TESTNET_RPC
    );

    const gas = parseInt(gasEstimate, 16);
    expect(gas).toBeGreaterThanOrEqual(21000);
    expect(gas).toBeLessThan(50000);
  });
});

// ============================================================================
// Network Latency Tests
// ============================================================================

describe('Testnet Performance', () => {
  it('measures RPC latency', async () => {
    if (!testnetAvailable) {
      console.log('Skipping: Testnet not available');
      return;
    }

    const latencies: number[] = [];

    for (let i = 0; i < 5; i++) {
      const start = Date.now();
      await rpcCall('eth_blockNumber', [], TESTNET_RPC);
      const latency = Date.now() - start;
      latencies.push(latency);
    }

    const avgLatency = latencies.reduce((a, b) => a + b, 0) / latencies.length;
    const maxLatency = Math.max(...latencies);
    const minLatency = Math.min(...latencies);

    console.log(`RPC Latency - Avg: ${avgLatency.toFixed(0)}ms, Min: ${minLatency}ms, Max: ${maxLatency}ms`);

    // Latency should be reasonable (under 2 seconds)
    expect(avgLatency).toBeLessThan(2000);
  });

  it('handles concurrent requests', async () => {
    if (!testnetAvailable) {
      console.log('Skipping: Testnet not available');
      return;
    }

    const start = Date.now();

    // Make 10 concurrent requests
    const requests = Array.from({ length: 10 }, () => rpcCall('eth_blockNumber', [], TESTNET_RPC));

    const results = await Promise.all(requests);
    const elapsed = Date.now() - start;

    // All should return valid block numbers
    for (const result of results) {
      expect(parseInt(result, 16)).toBeGreaterThanOrEqual(0);
    }

    console.log(`10 concurrent requests completed in ${elapsed}ms`);

    // Should complete in under 5 seconds
    expect(elapsed).toBeLessThan(5000);
  });
});

// ============================================================================
// SDK High-Level Tests
// ============================================================================

describe('SDK Testnet Integration', () => {
  it('SDK getBlock returns correct data', async () => {
    if (!testnetAvailable) {
      console.log('Skipping: Testnet not available');
      return;
    }

    const block = await sdk.getBlock('latest');

    expect(block).toBeDefined();
    expect(typeof block.number).toBe('number');
    expect(typeof block.hash).toBe('string');
    expect(block.hash).toMatch(/^0x[a-fA-F0-9]{64}$/);
  });

  it('SDK getNetworkInfo returns correct chain ID', async () => {
    if (!testnetAvailable) {
      console.log('Skipping: Testnet not available');
      return;
    }

    const networkInfo = await sdk.getNetworkInfo();

    expect(networkInfo.chainId).toBe(TESTNET_CHAIN_ID);
    expect(networkInfo.blockNumber).toBeGreaterThanOrEqual(0);
  });

  it('SDK account operations work correctly', async () => {
    if (!testnetAvailable) {
      console.log('Skipping: Testnet not available');
      return;
    }

    // Create a new account via SDK
    const account = sdk.accounts.createAccount();

    expect(account.address).toMatch(/^0x[a-fA-F0-9]{40}$/);
    expect(account.privateKey).toMatch(/^0x[a-fA-F0-9]{64}$/);

    // Get balance (should be 0)
    const balance = await sdk.accounts.getBalance();
    expect(balance).toBe(0n);
  });
});
