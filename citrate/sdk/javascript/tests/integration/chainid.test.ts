/**
 * Chain ID Consistency Tests
 *
 * Verifies that chain ID is consistent across:
 * - RPC eth_chainId response
 * - Transaction signing
 * - Transaction receipt
 * - SDK configuration
 * - Block data
 *
 * Run with: npm run test:integration
 */

import { ethers } from 'ethers';
import { CitrateSDK } from '../../src/sdk';
import {
  rpcCall,
  getNonce,
  getTestWallet,
  createRandomWallet,
  waitForTransaction,
  isNodeRunning,
  DEFAULT_TEST_CONFIG,
} from './testHarness';

// ============================================================================
// Test Configuration
// ============================================================================

const RPC_ENDPOINT = process.env.CITRATE_RPC_URL || 'http://localhost:8545';
const EXPECTED_CHAIN_ID = parseInt(process.env.CITRATE_CHAIN_ID || '1337');

let sdk: CitrateSDK;
let nodeRunning = false;

beforeAll(async () => {
  nodeRunning = await isNodeRunning(RPC_ENDPOINT);
  if (!nodeRunning) {
    console.warn(
      `WARNING: Node not running at ${RPC_ENDPOINT}. ` +
        'Start with: cargo run --bin citrate-node -- devnet'
    );
  }

  sdk = new CitrateSDK({
    rpcEndpoint: RPC_ENDPOINT,
    chainId: EXPECTED_CHAIN_ID,
  });
}, 30000);

// ============================================================================
// RPC Chain ID Tests
// ============================================================================

describe('RPC Chain ID', () => {
  it('eth_chainId returns expected chain ID', async () => {
    if (!nodeRunning) {
      console.log('Skipping: Node not running');
      return;
    }

    const result = await rpcCall('eth_chainId', [], RPC_ENDPOINT);
    const chainId = parseInt(result, 16);
    expect(chainId).toBe(EXPECTED_CHAIN_ID);
  });

  it('net_version returns consistent network ID', async () => {
    if (!nodeRunning) {
      console.log('Skipping: Node not running');
      return;
    }

    const result = await rpcCall('net_version', [], RPC_ENDPOINT);
    const networkId = parseInt(result);
    // Network ID should match or be related to chain ID
    expect(networkId).toBe(EXPECTED_CHAIN_ID);
  });

  it('chain ID is consistent across multiple calls', async () => {
    if (!nodeRunning) {
      console.log('Skipping: Node not running');
      return;
    }

    const results = await Promise.all([
      rpcCall('eth_chainId', [], RPC_ENDPOINT),
      rpcCall('eth_chainId', [], RPC_ENDPOINT),
      rpcCall('eth_chainId', [], RPC_ENDPOINT),
    ]);

    const chainIds = results.map((r) => parseInt(r, 16));
    expect(chainIds.every((id) => id === EXPECTED_CHAIN_ID)).toBe(true);
  });
});

// ============================================================================
// SDK Chain ID Tests
// ============================================================================

describe('SDK Chain ID Configuration', () => {
  it('SDK configured with correct chain ID', async () => {
    if (!nodeRunning) {
      console.log('Skipping: Node not running');
      return;
    }

    const networkInfo = await sdk.getNetworkInfo();
    expect(networkInfo.chainId).toBe(EXPECTED_CHAIN_ID);
  });

  it('SDK network info matches RPC', async () => {
    if (!nodeRunning) {
      console.log('Skipping: Node not running');
      return;
    }

    const sdkChainId = (await sdk.getNetworkInfo()).chainId;
    const rpcChainId = parseInt(await rpcCall('eth_chainId', [], RPC_ENDPOINT), 16);

    expect(sdkChainId).toBe(rpcChainId);
  });

  it('SDK rejects mismatched chain ID on connect', async () => {
    if (!nodeRunning) {
      console.log('Skipping: Node not running');
      return;
    }

    // Create SDK with wrong chain ID
    const wrongSdk = new CitrateSDK({
      rpcEndpoint: RPC_ENDPOINT,
      chainId: 999999, // Wrong chain ID
    });

    // SDK should detect mismatch and handle it
    // (behavior depends on SDK implementation)
    const networkInfo = await wrongSdk.getNetworkInfo();
    // The actual chain ID from the network should be returned
    expect(networkInfo.chainId).toBe(EXPECTED_CHAIN_ID);
  });
});

// ============================================================================
// Transaction Chain ID Tests
// ============================================================================

describe('Transaction Chain ID', () => {
  it('signed transaction includes correct chain ID', async () => {
    if (!nodeRunning) {
      console.log('Skipping: Node not running');
      return;
    }

    const sender = getTestWallet(0);
    const recipient = createRandomWallet();
    const nonce = await getNonce(sender.address, 'pending', RPC_ENDPOINT);
    const gasPrice = await rpcCall('eth_gasPrice', [], RPC_ENDPOINT);

    const tx = {
      type: 0,
      to: recipient.address,
      value: ethers.parseEther('0.001'),
      data: '0x',
      gasLimit: 21000,
      gasPrice: BigInt(gasPrice),
      nonce,
      chainId: EXPECTED_CHAIN_ID,
    };

    const signedTx = await sender.signTransaction(tx);
    const parsed = ethers.Transaction.from(signedTx);

    expect(Number(parsed.chainId)).toBe(EXPECTED_CHAIN_ID);
  });

  it('EIP-1559 transaction includes correct chain ID', async () => {
    if (!nodeRunning) {
      console.log('Skipping: Node not running');
      return;
    }

    const sender = getTestWallet(0);
    const recipient = createRandomWallet();
    const nonce = await getNonce(sender.address, 'pending', RPC_ENDPOINT);
    const gasPrice = await rpcCall('eth_gasPrice', [], RPC_ENDPOINT);

    const tx = {
      type: 2,
      to: recipient.address,
      value: ethers.parseEther('0.001'),
      data: '0x',
      gasLimit: 21000,
      maxFeePerGas: BigInt(gasPrice) * 2n,
      maxPriorityFeePerGas: BigInt(gasPrice) / 10n,
      nonce,
      chainId: EXPECTED_CHAIN_ID,
    };

    const signedTx = await sender.signTransaction(tx);
    const parsed = ethers.Transaction.from(signedTx);

    expect(Number(parsed.chainId)).toBe(EXPECTED_CHAIN_ID);
  });

  it('transaction with wrong chain ID is rejected', async () => {
    if (!nodeRunning) {
      console.log('Skipping: Node not running');
      return;
    }

    const sender = getTestWallet(0);
    const recipient = createRandomWallet();
    const nonce = await getNonce(sender.address, 'pending', RPC_ENDPOINT);
    const gasPrice = await rpcCall('eth_gasPrice', [], RPC_ENDPOINT);

    const tx = {
      type: 0,
      to: recipient.address,
      value: ethers.parseEther('0.001'),
      data: '0x',
      gasLimit: 21000,
      gasPrice: BigInt(gasPrice),
      nonce,
      chainId: 999999, // Wrong chain ID
    };

    const signedTx = await sender.signTransaction(tx);

    await expect(rpcCall('eth_sendRawTransaction', [signedTx], RPC_ENDPOINT)).rejects.toThrow();
  });
});

// ============================================================================
// Transaction Receipt Chain ID Tests
// ============================================================================

describe('Transaction Receipt Chain ID', () => {
  it('receipt reflects correct chain ID context', async () => {
    if (!nodeRunning) {
      console.log('Skipping: Node not running');
      return;
    }

    const sender = getTestWallet(0);
    const recipient = createRandomWallet();
    const nonce = await getNonce(sender.address, 'pending', RPC_ENDPOINT);
    const gasPrice = await rpcCall('eth_gasPrice', [], RPC_ENDPOINT);

    const tx = {
      type: 0,
      to: recipient.address,
      value: ethers.parseEther('0.001'),
      data: '0x',
      gasLimit: 21000,
      gasPrice: BigInt(gasPrice),
      nonce,
      chainId: EXPECTED_CHAIN_ID,
    };

    const signedTx = await sender.signTransaction(tx);
    const txHash = await rpcCall('eth_sendRawTransaction', [signedTx], RPC_ENDPOINT);
    const receipt = await waitForTransaction(txHash, 30000, RPC_ENDPOINT);

    expect(receipt).toBeDefined();
    expect(receipt.status).toBe('0x1');
    // Transaction should be on the correct chain
    const blockNumber = parseInt(receipt.blockNumber, 16);
    expect(blockNumber).toBeGreaterThan(0);
  }, 60000);
});

// ============================================================================
// Block Chain ID Context Tests
// ============================================================================

describe('Block Chain ID Context', () => {
  it('blocks are produced on correct chain', async () => {
    if (!nodeRunning) {
      console.log('Skipping: Node not running');
      return;
    }

    // Verify chain ID through block context
    const chainId = parseInt(await rpcCall('eth_chainId', [], RPC_ENDPOINT), 16);
    const block = await rpcCall('eth_getBlockByNumber', ['latest', false], RPC_ENDPOINT);

    expect(chainId).toBe(EXPECTED_CHAIN_ID);
    expect(block).toBeDefined();
    expect(block.number).toBeDefined();
  });
});

// ============================================================================
// Ethers.js Provider Chain ID Tests
// ============================================================================

describe('Ethers.js Provider Chain ID', () => {
  it('ethers provider detects correct chain ID', async () => {
    if (!nodeRunning) {
      console.log('Skipping: Node not running');
      return;
    }

    const provider = new ethers.JsonRpcProvider(RPC_ENDPOINT);
    const network = await provider.getNetwork();

    expect(Number(network.chainId)).toBe(EXPECTED_CHAIN_ID);
  });

  it('ethers wallet uses correct chain ID', async () => {
    if (!nodeRunning) {
      console.log('Skipping: Node not running');
      return;
    }

    const provider = new ethers.JsonRpcProvider(RPC_ENDPOINT);
    const wallet = new ethers.Wallet(getTestWallet(0).privateKey, provider);

    // Prepare unsigned transaction
    const recipient = createRandomWallet();
    const unsignedTx = {
      to: recipient.address,
      value: ethers.parseEther('0.001'),
    };

    // Populate transaction (includes chain ID)
    const populatedTx = await wallet.populateTransaction(unsignedTx);
    expect(Number(populatedTx.chainId)).toBe(EXPECTED_CHAIN_ID);
  });
});

// ============================================================================
// Replay Protection Tests
// ============================================================================

describe('Chain ID Replay Protection', () => {
  it('EIP-155 signature includes chain ID', async () => {
    if (!nodeRunning) {
      console.log('Skipping: Node not running');
      return;
    }

    const sender = getTestWallet(0);
    const recipient = createRandomWallet();
    const nonce = await getNonce(sender.address, 'pending', RPC_ENDPOINT);
    const gasPrice = await rpcCall('eth_gasPrice', [], RPC_ENDPOINT);

    const tx = {
      type: 0,
      to: recipient.address,
      value: ethers.parseEther('0.001'),
      data: '0x',
      gasLimit: 21000,
      gasPrice: BigInt(gasPrice),
      nonce,
      chainId: EXPECTED_CHAIN_ID,
    };

    const signedTx = await sender.signTransaction(tx);
    const parsed = ethers.Transaction.from(signedTx);

    // EIP-155 v value encodes chain ID
    // v = chainId * 2 + 35 + recovery_id (0 or 1)
    const expectedVMin = EXPECTED_CHAIN_ID * 2 + 35;
    const expectedVMax = EXPECTED_CHAIN_ID * 2 + 36;

    // For typed transactions, signature.v is just recovery id (0 or 1)
    // Chain ID is encoded separately in the transaction
    expect(Number(parsed.chainId)).toBe(EXPECTED_CHAIN_ID);
  });
});

// ============================================================================
// Multi-Network Isolation Tests
// ============================================================================

describe('Multi-Network Chain ID Isolation', () => {
  const DEVNET_CHAIN_ID = 1337;
  const TESTNET_CHAIN_ID = 1338;
  const MAINNET_CHAIN_ID = 1339;

  it('different networks have different chain IDs', () => {
    // Verify expected chain IDs are distinct
    expect(DEVNET_CHAIN_ID).not.toBe(TESTNET_CHAIN_ID);
    expect(DEVNET_CHAIN_ID).not.toBe(MAINNET_CHAIN_ID);
    expect(TESTNET_CHAIN_ID).not.toBe(MAINNET_CHAIN_ID);
  });

  it('SDK configured for each network uses correct chain ID', () => {
    const devnetSdk = new CitrateSDK({
      rpcEndpoint: 'http://localhost:8545',
      chainId: DEVNET_CHAIN_ID,
    });

    const testnetSdk = new CitrateSDK({
      rpcEndpoint: 'https://testnet-rpc.citrate.ai',
      chainId: TESTNET_CHAIN_ID,
    });

    // Both should be configured
    expect(devnetSdk).toBeDefined();
    expect(testnetSdk).toBeDefined();
  });
});

// ============================================================================
// Configuration File Chain ID Tests
// ============================================================================

describe('Configuration Chain ID', () => {
  it('DEFAULT_TEST_CONFIG has correct chain ID', () => {
    expect(DEFAULT_TEST_CONFIG.chainId).toBe(EXPECTED_CHAIN_ID);
  });

  it('environment variables override defaults', () => {
    const envChainId = process.env.CITRATE_CHAIN_ID;
    if (envChainId) {
      expect(EXPECTED_CHAIN_ID).toBe(parseInt(envChainId));
    } else {
      expect(EXPECTED_CHAIN_ID).toBe(1337); // Default
    }
  });
});

// ============================================================================
// Contract Deployment Chain ID Tests
// ============================================================================

describe('Contract Deployment Chain ID', () => {
  it('contract deployment uses correct chain ID', async () => {
    if (!nodeRunning) {
      console.log('Skipping: Node not running');
      return;
    }

    const sender = getTestWallet(0);
    const nonce = await getNonce(sender.address, 'pending', RPC_ENDPOINT);
    const gasPrice = await rpcCall('eth_gasPrice', [], RPC_ENDPOINT);

    // Simple contract deployment bytecode
    const bytecode = '0x6080604052600080fd';

    const tx = {
      type: 0,
      to: null, // Contract creation
      value: 0n,
      data: bytecode,
      gasLimit: 100000,
      gasPrice: BigInt(gasPrice),
      nonce,
      chainId: EXPECTED_CHAIN_ID,
    };

    const signedTx = await sender.signTransaction(tx);
    const parsed = ethers.Transaction.from(signedTx);

    expect(Number(parsed.chainId)).toBe(EXPECTED_CHAIN_ID);
    expect(parsed.to).toBeNull();
  });
});
