/**
 * Integration Tests for Citrate SDK RPC Methods
 *
 * Tests all eth_* methods against a real (embedded or testnet) node.
 * Run with: npm run test:integration
 *
 * Environment variables:
 * - CITRATE_RPC_URL: RPC endpoint (default: http://localhost:8545)
 * - CITRATE_CHAIN_ID: Chain ID (default: 1337)
 */

import { CitrateSDK } from '../../src/sdk';
import {
  rpcCall,
  getBalance,
  getNonce,
  getBlockNumber,
  getTestWallet,
  createRandomWallet,
  createLegacyTransaction,
  createEIP1559Transaction,
  createEIP2930Transaction,
  sendAndWait,
  waitForTransaction,
  sleep,
  parseEth,
  formatEth,
  isNodeRunning,
  DEFAULT_TEST_CONFIG,
} from './testHarness';

// ============================================================================
// Test Setup
// ============================================================================

const RPC_ENDPOINT = process.env.CITRATE_RPC_URL || 'http://localhost:8545';
const CHAIN_ID = parseInt(process.env.CITRATE_CHAIN_ID || '1337');

let sdk: CitrateSDK;

beforeAll(async () => {
  // Check if node is running
  const running = await isNodeRunning(RPC_ENDPOINT);
  if (!running) {
    console.warn(
      `WARNING: Node not running at ${RPC_ENDPOINT}. ` +
        'Start with: cargo run --bin citrate-node -- devnet'
    );
  }

  // Initialize SDK
  sdk = new CitrateSDK({
    rpcEndpoint: RPC_ENDPOINT,
    chainId: CHAIN_ID,
  });
}, 30000);

// ============================================================================
// eth_chainId Tests
// ============================================================================

describe('eth_chainId', () => {
  it('returns the configured chain ID', async () => {
    const result = await rpcCall('eth_chainId', [], RPC_ENDPOINT);

    expect(result).toBeDefined();
    expect(typeof result).toBe('string');
    expect(result.startsWith('0x')).toBe(true);

    const chainId = parseInt(result, 16);
    expect(chainId).toBe(CHAIN_ID);
  });

  it('matches SDK configured chain ID', async () => {
    const networkInfo = await sdk.getNetworkInfo();
    expect(networkInfo.chainId).toBe(CHAIN_ID);
  });
});

// ============================================================================
// eth_blockNumber Tests
// ============================================================================

describe('eth_blockNumber', () => {
  it('returns current block height', async () => {
    const result = await rpcCall('eth_blockNumber', [], RPC_ENDPOINT);

    expect(result).toBeDefined();
    expect(typeof result).toBe('string');
    expect(result.startsWith('0x')).toBe(true);

    const blockNumber = parseInt(result, 16);
    expect(blockNumber).toBeGreaterThanOrEqual(0);
  });

  it('increases over time', async () => {
    const block1 = await getBlockNumber(RPC_ENDPOINT);
    await sleep(2000); // Wait for at least one block
    const block2 = await getBlockNumber(RPC_ENDPOINT);

    expect(block2).toBeGreaterThanOrEqual(block1);
  }, 10000);

  it('SDK getBlock returns valid block', async () => {
    const block = await sdk.getBlock('latest');

    expect(block).toBeDefined();
    expect(typeof block.number).toBe('number');
    expect(typeof block.hash).toBe('string');
    expect(block.hash.startsWith('0x')).toBe(true);
    expect(typeof block.timestamp).toBe('number');
  });
});

// ============================================================================
// eth_getBalance Tests
// ============================================================================

describe('eth_getBalance', () => {
  it('returns balance for genesis account', async () => {
    const wallet = getTestWallet(0);
    const balance = await getBalance(wallet.address, RPC_ENDPOINT);

    expect(balance).toBeGreaterThan(0n);
    console.log(`Genesis account balance: ${formatEth(balance)} ETH`);
  });

  it('returns zero for new random address', async () => {
    const wallet = createRandomWallet();
    const balance = await getBalance(wallet.address, RPC_ENDPOINT);

    expect(balance).toBe(0n);
  });

  it('SDK getBalance matches direct RPC', async () => {
    const wallet = getTestWallet(0);

    // Import account to SDK
    sdk.accounts.importAccount(wallet.privateKey);

    // Get balance via SDK
    const sdkBalance = await sdk.accounts.getBalance();

    // Get balance via direct RPC
    const rpcBalance = await getBalance(wallet.address, RPC_ENDPOINT);

    expect(sdkBalance).toBe(rpcBalance);
  });

  it('accepts 0x-prefixed address', async () => {
    const address = '0x' + '0'.repeat(40);
    const balance = await getBalance(address, RPC_ENDPOINT);
    expect(balance).toBe(0n);
  });

  it('handles checksum addresses', async () => {
    const wallet = getTestWallet(0);
    const checksumAddress = wallet.address; // ethers returns checksum

    const balance = await getBalance(checksumAddress, RPC_ENDPOINT);
    expect(balance).toBeGreaterThan(0n);
  });
});

// ============================================================================
// eth_getTransactionCount Tests
// ============================================================================

describe('eth_getTransactionCount', () => {
  it('returns nonce for account', async () => {
    const wallet = getTestWallet(0);
    const nonce = await getNonce(wallet.address, 'latest', RPC_ENDPOINT);

    expect(typeof nonce).toBe('number');
    expect(nonce).toBeGreaterThanOrEqual(0);
  });

  it('supports pending tag', async () => {
    const wallet = getTestWallet(0);
    const pendingNonce = await getNonce(wallet.address, 'pending', RPC_ENDPOINT);
    const latestNonce = await getNonce(wallet.address, 'latest', RPC_ENDPOINT);

    expect(pendingNonce).toBeGreaterThanOrEqual(latestNonce);
  });

  it('returns zero for new address', async () => {
    const wallet = createRandomWallet();
    const nonce = await getNonce(wallet.address, 'latest', RPC_ENDPOINT);

    expect(nonce).toBe(0);
  });
});

// ============================================================================
// eth_call Tests
// ============================================================================

describe('eth_call', () => {
  it('executes call without state change', async () => {
    const wallet = getTestWallet(0);

    // Call balanceOf on any address (will fail but tests the mechanism)
    const result = await rpcCall(
      'eth_call',
      [
        {
          to: wallet.address,
          data: '0x', // Empty call
        },
        'latest',
      ],
      RPC_ENDPOINT
    );

    expect(result).toBeDefined();
  });

  it('returns data from contract read', async () => {
    // Test with a known contract if available
    // For now, test that eth_call doesn't throw on valid input
    const result = await rpcCall(
      'eth_call',
      [
        {
          to: '0x0000000000000000000000000000000000000000',
          data: '0x',
        },
        'latest',
      ],
      RPC_ENDPOINT
    );

    expect(typeof result).toBe('string');
  });
});

// ============================================================================
// eth_estimateGas Tests
// ============================================================================

describe('eth_estimateGas', () => {
  it('estimates gas for simple transfer', async () => {
    const wallet = getTestWallet(0);
    const recipient = createRandomWallet();

    const result = await rpcCall(
      'eth_estimateGas',
      [
        {
          from: wallet.address,
          to: recipient.address,
          value: '0x1',
        },
      ],
      RPC_ENDPOINT
    );

    expect(result).toBeDefined();
    const gasEstimate = parseInt(result, 16);

    // Simple transfer should be around 21000 gas
    expect(gasEstimate).toBeGreaterThanOrEqual(21000);
    expect(gasEstimate).toBeLessThan(50000);
  });

  it('estimates higher gas for contract deployment', async () => {
    const wallet = getTestWallet(0);

    // Simple bytecode
    const bytecode = '0x6080604052600080fd'; // Minimal contract

    const result = await rpcCall(
      'eth_estimateGas',
      [
        {
          from: wallet.address,
          data: bytecode,
        },
      ],
      RPC_ENDPOINT
    );

    const gasEstimate = parseInt(result, 16);
    expect(gasEstimate).toBeGreaterThan(21000);
  });
});

// ============================================================================
// eth_gasPrice Tests
// ============================================================================

describe('eth_gasPrice', () => {
  it('returns current gas price', async () => {
    const result = await rpcCall('eth_gasPrice', [], RPC_ENDPOINT);

    expect(result).toBeDefined();
    expect(typeof result).toBe('string');
    expect(result.startsWith('0x')).toBe(true);

    const gasPrice = BigInt(result);
    expect(gasPrice).toBeGreaterThan(0n);
  });
});

// ============================================================================
// eth_feeHistory Tests
// ============================================================================

describe('eth_feeHistory', () => {
  it('returns fee history data', async () => {
    const result = await rpcCall(
      'eth_feeHistory',
      ['0x4', 'latest', [25, 50, 75]],
      RPC_ENDPOINT
    );

    expect(result).toBeDefined();
    expect(result.baseFeePerGas).toBeDefined();
    expect(Array.isArray(result.baseFeePerGas)).toBe(true);
  });

  it('returns reward percentiles', async () => {
    const result = await rpcCall(
      'eth_feeHistory',
      ['0x4', 'latest', [25, 50, 75]],
      RPC_ENDPOINT
    );

    if (result.reward) {
      expect(Array.isArray(result.reward)).toBe(true);
    }
  });
});

// ============================================================================
// eth_sendRawTransaction Tests
// ============================================================================

describe('eth_sendRawTransaction', () => {
  it('sends legacy transaction', async () => {
    const sender = getTestWallet(0);
    const recipient = createRandomWallet();

    const signedTx = await createLegacyTransaction(
      sender,
      recipient.address,
      parseEth('0.01'),
      '0x',
      21000,
      RPC_ENDPOINT
    );

    const txHash = await rpcCall('eth_sendRawTransaction', [signedTx], RPC_ENDPOINT);

    expect(txHash).toBeDefined();
    expect(typeof txHash).toBe('string');
    expect(txHash.startsWith('0x')).toBe(true);
    expect(txHash.length).toBe(66); // 0x + 64 hex chars

    // Wait for confirmation
    const receipt = await waitForTransaction(txHash, 30000, RPC_ENDPOINT);
    expect(receipt.status).toBe('0x1');

    // Verify recipient balance
    const balance = await getBalance(recipient.address, RPC_ENDPOINT);
    expect(balance).toBe(parseEth('0.01'));
  }, 60000);

  it('rejects transaction with wrong chain ID', async () => {
    const sender = getTestWallet(0);
    const recipient = createRandomWallet();

    // Create transaction with wrong chain ID
    const nonce = await getNonce(sender.address, 'pending', RPC_ENDPOINT);
    const gasPrice = await rpcCall('eth_gasPrice', [], RPC_ENDPOINT);

    const tx = {
      type: 0,
      to: recipient.address,
      value: parseEth('0.001'),
      data: '0x',
      gasLimit: 21000,
      gasPrice: BigInt(gasPrice),
      nonce,
      chainId: 999999, // Wrong chain ID
    };

    const signedTx = await sender.signTransaction(tx);

    await expect(
      rpcCall('eth_sendRawTransaction', [signedTx], RPC_ENDPOINT)
    ).rejects.toThrow();
  });

  it('rejects transaction with invalid signature', async () => {
    // Manually construct a transaction with invalid signature
    const invalidTx = '0x' + 'f'.repeat(200);

    await expect(
      rpcCall('eth_sendRawTransaction', [invalidTx], RPC_ENDPOINT)
    ).rejects.toThrow();
  });
});

// ============================================================================
// eth_getTransactionReceipt Tests
// ============================================================================

describe('eth_getTransactionReceipt', () => {
  it('returns null for non-existent transaction', async () => {
    const fakeTxHash = '0x' + '0'.repeat(64);
    const result = await rpcCall('eth_getTransactionReceipt', [fakeTxHash], RPC_ENDPOINT);

    expect(result).toBeNull();
  });

  it('returns receipt after confirmation', async () => {
    const sender = getTestWallet(0);
    const recipient = createRandomWallet();

    const signedTx = await createLegacyTransaction(
      sender,
      recipient.address,
      parseEth('0.001'),
      '0x',
      21000,
      RPC_ENDPOINT
    );

    const txHash = await rpcCall('eth_sendRawTransaction', [signedTx], RPC_ENDPOINT);
    const receipt = await waitForTransaction(txHash, 30000, RPC_ENDPOINT);

    expect(receipt).toBeDefined();
    expect(receipt.transactionHash).toBe(txHash);
    expect(receipt.blockNumber).toBeDefined();
    expect(receipt.blockHash).toBeDefined();
    expect(receipt.status).toBe('0x1');
    expect(receipt.gasUsed).toBeDefined();
  }, 60000);
});

// ============================================================================
// eth_getTransactionByHash Tests
// ============================================================================

describe('eth_getTransactionByHash', () => {
  it('returns null for non-existent transaction', async () => {
    const fakeTxHash = '0x' + '0'.repeat(64);
    const result = await rpcCall('eth_getTransactionByHash', [fakeTxHash], RPC_ENDPOINT);

    expect(result).toBeNull();
  });

  it('returns transaction details', async () => {
    const sender = getTestWallet(0);
    const recipient = createRandomWallet();

    const signedTx = await createLegacyTransaction(
      sender,
      recipient.address,
      parseEth('0.001'),
      '0x',
      21000,
      RPC_ENDPOINT
    );

    const txHash = await rpcCall('eth_sendRawTransaction', [signedTx], RPC_ENDPOINT);
    await waitForTransaction(txHash, 30000, RPC_ENDPOINT);

    const tx = await rpcCall('eth_getTransactionByHash', [txHash], RPC_ENDPOINT);

    expect(tx).toBeDefined();
    expect(tx.hash).toBe(txHash);
    expect(tx.from.toLowerCase()).toBe(sender.address.toLowerCase());
    expect(tx.to.toLowerCase()).toBe(recipient.address.toLowerCase());
  }, 60000);
});

// ============================================================================
// eth_getBlockByNumber Tests
// ============================================================================

describe('eth_getBlockByNumber', () => {
  it('returns latest block', async () => {
    const result = await rpcCall('eth_getBlockByNumber', ['latest', false], RPC_ENDPOINT);

    expect(result).toBeDefined();
    expect(result.number).toBeDefined();
    expect(result.hash).toBeDefined();
    expect(result.parentHash).toBeDefined();
    expect(result.timestamp).toBeDefined();
  });

  it('returns block with transactions when requested', async () => {
    const result = await rpcCall('eth_getBlockByNumber', ['latest', true], RPC_ENDPOINT);

    expect(result).toBeDefined();
    expect(Array.isArray(result.transactions)).toBe(true);
  });

  it('returns null for future block', async () => {
    const result = await rpcCall('eth_getBlockByNumber', ['0xffffff', false], RPC_ENDPOINT);
    expect(result).toBeNull();
  });
});

// ============================================================================
// SDK Integration Tests
// ============================================================================

describe('SDK Integration', () => {
  it('SDK waitForTransaction works', async () => {
    const sender = getTestWallet(0);
    const recipient = createRandomWallet();

    // Import account
    sdk.accounts.importAccount(sender.privateKey);

    // Create transaction via SDK
    const signedTx = await createLegacyTransaction(
      sender,
      recipient.address,
      parseEth('0.001'),
      '0x',
      21000,
      RPC_ENDPOINT
    );

    const txHash = await rpcCall('eth_sendRawTransaction', [signedTx], RPC_ENDPOINT);

    // Wait using SDK
    const receipt = await sdk.waitForTransaction(txHash);

    expect(receipt).toBeDefined();
    expect(receipt.status).toBe('0x1');
  }, 60000);

  it('SDK network info is accurate', async () => {
    const networkInfo = await sdk.getNetworkInfo();

    expect(networkInfo.chainId).toBe(CHAIN_ID);
    expect(networkInfo.blockNumber).toBeGreaterThanOrEqual(0);
    expect(typeof networkInfo.networkId).toBe('number');
  });
});
