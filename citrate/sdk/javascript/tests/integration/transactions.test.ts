/**
 * Integration Tests for Transaction Types
 *
 * Tests legacy, EIP-2930, and EIP-1559 transaction types.
 * Verifies RLP encoding, signature recovery, and gas pricing.
 */

import { ethers } from 'ethers';
import {
  rpcCall,
  getBalance,
  getNonce,
  getTestWallet,
  createRandomWallet,
  createLegacyTransaction,
  createEIP1559Transaction,
  createEIP2930Transaction,
  sendAndWait,
  waitForTransaction,
  parseEth,
  formatEth,
  isNodeRunning,
  DEFAULT_TEST_CONFIG,
  deployContract,
  SIMPLE_STORAGE_BYTECODE,
} from './testHarness';

// ============================================================================
// Test Setup
// ============================================================================

const RPC_ENDPOINT = process.env.CITRATE_RPC_URL || 'http://localhost:8545';
const CHAIN_ID = parseInt(process.env.CITRATE_CHAIN_ID || '1337');

beforeAll(async () => {
  const running = await isNodeRunning(RPC_ENDPOINT);
  if (!running) {
    console.warn(
      `WARNING: Node not running at ${RPC_ENDPOINT}. ` +
        'Start with: cargo run --bin citrate-node -- devnet'
    );
  }
}, 30000);

// ============================================================================
// Legacy Transaction Tests (Type 0)
// ============================================================================

describe('Legacy Transactions (Type 0)', () => {
  it('creates valid legacy transaction', async () => {
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

    // Verify it starts with 0x (not 0x01 or 0x02)
    expect(signedTx.startsWith('0x')).toBe(true);

    // Legacy transactions start with RLP-encoded values, not type prefix
    // The first byte should be >= 0xc0 (RLP list prefix)
    const firstByte = parseInt(signedTx.slice(2, 4), 16);
    expect(firstByte).toBeGreaterThanOrEqual(0xc0);
  });

  it('sends and confirms legacy transaction', async () => {
    const sender = getTestWallet(0);
    const recipient = createRandomWallet();
    const value = parseEth('0.005');

    const signedTx = await createLegacyTransaction(
      sender,
      recipient.address,
      value,
      '0x',
      21000,
      RPC_ENDPOINT
    );

    const txHash = await rpcCall('eth_sendRawTransaction', [signedTx], RPC_ENDPOINT);
    const receipt = await waitForTransaction(txHash, 30000, RPC_ENDPOINT);

    expect(receipt.status).toBe('0x1');

    // Verify balance transferred
    const balance = await getBalance(recipient.address, RPC_ENDPOINT);
    expect(balance).toBe(value);
  }, 60000);

  it('legacy transaction has gasPrice field in receipt', async () => {
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
    const tx = await rpcCall('eth_getTransactionByHash', [txHash], RPC_ENDPOINT);

    expect(tx.gasPrice).toBeDefined();
    expect(tx.type).toBe('0x0');
  }, 60000);

  it('signature recovery returns correct sender', async () => {
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

    // Parse the transaction to verify signature
    const parsed = ethers.Transaction.from(signedTx);
    expect(parsed.from?.toLowerCase()).toBe(sender.address.toLowerCase());
  });
});

// ============================================================================
// EIP-2930 Transaction Tests (Type 1)
// ============================================================================

describe('EIP-2930 Transactions (Type 1)', () => {
  it('creates valid EIP-2930 transaction with access list', async () => {
    const sender = getTestWallet(0);
    const recipient = createRandomWallet();

    const accessList = [
      {
        address: recipient.address,
        storageKeys: ['0x' + '0'.repeat(64)],
      },
    ];

    const signedTx = await createEIP2930Transaction(
      sender,
      recipient.address,
      parseEth('0.01'),
      accessList,
      '0x',
      30000, // Slightly higher gas for access list
      RPC_ENDPOINT
    );

    // EIP-2930 transactions start with 0x01
    expect(signedTx.startsWith('0x01')).toBe(true);
  });

  it('sends EIP-2930 transaction with empty access list', async () => {
    const sender = getTestWallet(0);
    const recipient = createRandomWallet();
    const value = parseEth('0.003');

    const signedTx = await createEIP2930Transaction(
      sender,
      recipient.address,
      value,
      [], // Empty access list
      '0x',
      21000,
      RPC_ENDPOINT
    );

    const txHash = await rpcCall('eth_sendRawTransaction', [signedTx], RPC_ENDPOINT);
    const receipt = await waitForTransaction(txHash, 30000, RPC_ENDPOINT);

    expect(receipt.status).toBe('0x1');

    const tx = await rpcCall('eth_getTransactionByHash', [txHash], RPC_ENDPOINT);
    expect(tx.type).toBe('0x1');
    expect(tx.accessList).toBeDefined();
    expect(Array.isArray(tx.accessList)).toBe(true);
  }, 60000);

  it('sends EIP-2930 transaction with access list entries', async () => {
    const sender = getTestWallet(0);
    const recipient = createRandomWallet();

    // Create access list with storage slots
    const accessList = [
      {
        address: '0x' + '1'.repeat(40),
        storageKeys: [
          '0x' + '0'.repeat(64),
          '0x' + '1'.padStart(64, '0'),
        ],
      },
      {
        address: '0x' + '2'.repeat(40),
        storageKeys: [],
      },
    ];

    const signedTx = await createEIP2930Transaction(
      sender,
      recipient.address,
      parseEth('0.002'),
      accessList,
      '0x',
      50000, // Higher gas for access list
      RPC_ENDPOINT
    );

    const txHash = await rpcCall('eth_sendRawTransaction', [signedTx], RPC_ENDPOINT);
    const receipt = await waitForTransaction(txHash, 30000, RPC_ENDPOINT);

    expect(receipt.status).toBe('0x1');
  }, 60000);

  it('EIP-2930 signature recovery returns correct sender', async () => {
    const sender = getTestWallet(0);
    const recipient = createRandomWallet();

    const signedTx = await createEIP2930Transaction(
      sender,
      recipient.address,
      parseEth('0.001'),
      [],
      '0x',
      21000,
      RPC_ENDPOINT
    );

    const parsed = ethers.Transaction.from(signedTx);
    expect(parsed.from?.toLowerCase()).toBe(sender.address.toLowerCase());
  });
});

// ============================================================================
// EIP-1559 Transaction Tests (Type 2)
// ============================================================================

describe('EIP-1559 Transactions (Type 2)', () => {
  it('creates valid EIP-1559 transaction', async () => {
    const sender = getTestWallet(0);
    const recipient = createRandomWallet();

    const signedTx = await createEIP1559Transaction(
      sender,
      recipient.address,
      parseEth('0.01'),
      '0x',
      21000,
      RPC_ENDPOINT
    );

    // EIP-1559 transactions start with 0x02
    expect(signedTx.startsWith('0x02')).toBe(true);
  });

  it('sends and confirms EIP-1559 transaction', async () => {
    const sender = getTestWallet(0);
    const recipient = createRandomWallet();
    const value = parseEth('0.004');

    const signedTx = await createEIP1559Transaction(
      sender,
      recipient.address,
      value,
      '0x',
      21000,
      RPC_ENDPOINT
    );

    const txHash = await rpcCall('eth_sendRawTransaction', [signedTx], RPC_ENDPOINT);
    const receipt = await waitForTransaction(txHash, 30000, RPC_ENDPOINT);

    expect(receipt.status).toBe('0x1');

    // Verify balance
    const balance = await getBalance(recipient.address, RPC_ENDPOINT);
    expect(balance).toBe(value);
  }, 60000);

  it('EIP-1559 transaction has maxFeePerGas and maxPriorityFeePerGas', async () => {
    const sender = getTestWallet(0);
    const recipient = createRandomWallet();

    const signedTx = await createEIP1559Transaction(
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

    expect(tx.type).toBe('0x2');
    expect(tx.maxFeePerGas).toBeDefined();
    expect(tx.maxPriorityFeePerGas).toBeDefined();
  }, 60000);

  it('EIP-1559 signature recovery returns correct sender', async () => {
    const sender = getTestWallet(0);
    const recipient = createRandomWallet();

    const signedTx = await createEIP1559Transaction(
      sender,
      recipient.address,
      parseEth('0.001'),
      '0x',
      21000,
      RPC_ENDPOINT
    );

    const parsed = ethers.Transaction.from(signedTx);
    expect(parsed.from?.toLowerCase()).toBe(sender.address.toLowerCase());
  });

  it('respects maxFeePerGas limit', async () => {
    const sender = getTestWallet(0);
    const recipient = createRandomWallet();

    // Get current gas price
    const gasPrice = await rpcCall('eth_gasPrice', [], RPC_ENDPOINT);
    const gasPriceBigInt = BigInt(gasPrice);

    // Create transaction with specific fee parameters
    const nonce = await getNonce(sender.address, 'pending', RPC_ENDPOINT);

    const tx = {
      type: 2,
      to: recipient.address,
      value: parseEth('0.001'),
      data: '0x',
      gasLimit: 21000,
      maxFeePerGas: gasPriceBigInt * 2n,
      maxPriorityFeePerGas: gasPriceBigInt / 10n,
      nonce,
      chainId: CHAIN_ID,
    };

    const signedTx = await sender.signTransaction(tx);
    const txHash = await rpcCall('eth_sendRawTransaction', [signedTx], RPC_ENDPOINT);
    const receipt = await waitForTransaction(txHash, 30000, RPC_ENDPOINT);

    expect(receipt.status).toBe('0x1');

    // Verify effective gas price is within bounds
    const receiptTx = await rpcCall('eth_getTransactionByHash', [txHash], RPC_ENDPOINT);
    const effectiveGasPrice = BigInt(receipt.effectiveGasPrice || receiptTx.gasPrice);
    expect(effectiveGasPrice).toBeLessThanOrEqual(gasPriceBigInt * 2n);
  }, 60000);
});

// ============================================================================
// Gas Pricing Tests
// ============================================================================

describe('Gas Pricing', () => {
  it('legacy transaction uses gasPrice', async () => {
    const sender = getTestWallet(0);
    const recipient = createRandomWallet();

    const gasPrice = await rpcCall('eth_gasPrice', [], RPC_ENDPOINT);

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
    expect(BigInt(tx.gasPrice)).toBeGreaterThan(0n);
  }, 60000);

  it('EIP-1559 effective gas price respects base fee', async () => {
    // Get fee history to find base fee
    const feeHistory = await rpcCall('eth_feeHistory', ['0x1', 'latest', []], RPC_ENDPOINT);

    if (feeHistory.baseFeePerGas && feeHistory.baseFeePerGas.length > 0) {
      const baseFee = BigInt(feeHistory.baseFeePerGas[0]);
      console.log('Current base fee:', formatEth(baseFee * 1000000000n), 'gwei');

      const sender = getTestWallet(0);
      const recipient = createRandomWallet();

      const signedTx = await createEIP1559Transaction(
        sender,
        recipient.address,
        parseEth('0.001'),
        '0x',
        21000,
        RPC_ENDPOINT
      );

      const txHash = await rpcCall('eth_sendRawTransaction', [signedTx], RPC_ENDPOINT);
      const receipt = await waitForTransaction(txHash, 30000, RPC_ENDPOINT);

      // Effective gas price should be at least base fee
      if (receipt.effectiveGasPrice) {
        const effectiveGasPrice = BigInt(receipt.effectiveGasPrice);
        expect(effectiveGasPrice).toBeGreaterThanOrEqual(baseFee);
      }
    }
  }, 60000);
});

// ============================================================================
// RLP Encoding Validation
// ============================================================================

describe('RLP Encoding', () => {
  it('legacy transaction RLP matches ethers.js encoding', async () => {
    const sender = getTestWallet(0);
    const recipient = createRandomWallet();

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
      chainId: CHAIN_ID,
    };

    const signedTx = await sender.signTransaction(tx);

    // Verify the signed transaction can be parsed back
    const parsed = ethers.Transaction.from(signedTx);

    expect(parsed.to?.toLowerCase()).toBe(recipient.address.toLowerCase());
    expect(parsed.value).toBe(parseEth('0.001'));
    expect(parsed.nonce).toBe(nonce);
    expect(parsed.chainId).toBe(BigInt(CHAIN_ID));
  });

  it('EIP-2930 transaction RLP includes access list', async () => {
    const sender = getTestWallet(0);
    const recipient = createRandomWallet();

    const accessList = [
      {
        address: '0x' + 'a'.repeat(40),
        storageKeys: ['0x' + '1'.padStart(64, '0')],
      },
    ];

    const signedTx = await createEIP2930Transaction(
      sender,
      recipient.address,
      parseEth('0.001'),
      accessList,
      '0x',
      30000,
      RPC_ENDPOINT
    );

    const parsed = ethers.Transaction.from(signedTx);

    expect(parsed.accessList).toBeDefined();
    expect(parsed.accessList?.length).toBe(1);
    expect(parsed.accessList?.[0].address.toLowerCase()).toBe('0x' + 'a'.repeat(40));
  });

  it('EIP-1559 transaction RLP includes fee fields', async () => {
    const sender = getTestWallet(0);
    const recipient = createRandomWallet();

    const signedTx = await createEIP1559Transaction(
      sender,
      recipient.address,
      parseEth('0.001'),
      '0x',
      21000,
      RPC_ENDPOINT
    );

    const parsed = ethers.Transaction.from(signedTx);

    expect(parsed.maxFeePerGas).toBeDefined();
    expect(parsed.maxPriorityFeePerGas).toBeDefined();
    expect(parsed.maxFeePerGas).toBeGreaterThan(0n);
    expect(parsed.maxPriorityFeePerGas).toBeGreaterThan(0n);
  });
});

// ============================================================================
// Negative Tests
// ============================================================================

describe('Negative Tests', () => {
  it('rejects transaction with nonce too low', async () => {
    const sender = getTestWallet(0);
    const recipient = createRandomWallet();

    const currentNonce = await getNonce(sender.address, 'latest', RPC_ENDPOINT);

    // If nonce is 0, can't test "too low"
    if (currentNonce > 0) {
      const gasPrice = await rpcCall('eth_gasPrice', [], RPC_ENDPOINT);

      const tx = {
        type: 0,
        to: recipient.address,
        value: parseEth('0.001'),
        data: '0x',
        gasLimit: 21000,
        gasPrice: BigInt(gasPrice),
        nonce: 0, // Too low
        chainId: CHAIN_ID,
      };

      const signedTx = await sender.signTransaction(tx);

      await expect(
        rpcCall('eth_sendRawTransaction', [signedTx], RPC_ENDPOINT)
      ).rejects.toThrow();
    }
  });

  it('rejects transaction with insufficient balance', async () => {
    // Use a new wallet with no balance
    const poorWallet = createRandomWallet();
    const recipient = createRandomWallet();

    const gasPrice = await rpcCall('eth_gasPrice', [], RPC_ENDPOINT);

    const tx = {
      type: 0,
      to: recipient.address,
      value: parseEth('1000'), // Way more than balance
      data: '0x',
      gasLimit: 21000,
      gasPrice: BigInt(gasPrice),
      nonce: 0,
      chainId: CHAIN_ID,
    };

    const signedTx = await poorWallet.signTransaction(tx);

    await expect(
      rpcCall('eth_sendRawTransaction', [signedTx], RPC_ENDPOINT)
    ).rejects.toThrow();
  });

  it('rejects malformed RLP', async () => {
    const malformedTx = '0x' + 'ff'.repeat(100);

    await expect(
      rpcCall('eth_sendRawTransaction', [malformedTx], RPC_ENDPOINT)
    ).rejects.toThrow();
  });
});
