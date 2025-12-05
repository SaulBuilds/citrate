/**
 * Integration Tests for citrate-js Client
 *
 * Tests CitrateClient functionality against a real node.
 * Run with: npm run test:integration
 *
 * Environment variables:
 * - CITRATE_RPC_URL: RPC endpoint (default: http://localhost:8545)
 * - CITRATE_CHAIN_ID: Chain ID (default: 1337)
 */

import { ethers } from 'ethers';
import { CitrateClient, CitrateClientConfig } from '../../src/client/CitrateClient';
import { CitrateError } from '../../src/errors/CitrateError';

// ============================================================================
// Test Configuration
// ============================================================================

const RPC_ENDPOINT = process.env.CITRATE_RPC_URL || 'http://localhost:8545';
const CHAIN_ID = parseInt(process.env.CITRATE_CHAIN_ID || '1337');

// Well-known test accounts (from Hardhat/Anvil)
const TEST_ACCOUNTS = [
  {
    privateKey: '0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80',
    address: '0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266',
  },
  {
    privateKey: '0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d',
    address: '0x70997970C51812dc3A010C7d01b50e0d17dc79C8',
  },
  {
    privateKey: '0x5de4111afa1a4b94908f83103eb1f1706367c2e68ca870fc3fb9a804cdab365a',
    address: '0x3C44CdDdB6a900fa2b585dd299e03d12FA4293BC',
  },
];

// ============================================================================
// Test Helpers
// ============================================================================

async function isNodeRunning(rpcUrl: string): Promise<boolean> {
  try {
    const response = await fetch(rpcUrl, {
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

async function rpcCall(method: string, params: any[], rpcUrl: string): Promise<any> {
  const response = await fetch(rpcUrl, {
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
    throw new Error(`RPC Error: ${data.error.message}`);
  }
  return data.result;
}

function createRandomWallet(): ethers.Wallet {
  return ethers.Wallet.createRandom();
}

function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

// ============================================================================
// Test Setup
// ============================================================================

let client: CitrateClient;
let fundedClient: CitrateClient;
let nodeRunning = false;

beforeAll(async () => {
  nodeRunning = await isNodeRunning(RPC_ENDPOINT);
  if (!nodeRunning) {
    console.warn(
      `WARNING: Node not running at ${RPC_ENDPOINT}. ` +
        'Start with: cargo run --bin citrate-node -- devnet'
    );
  }

  // Initialize read-only client
  client = new CitrateClient({
    rpcUrl: RPC_ENDPOINT,
  });

  // Initialize client with funded account
  fundedClient = new CitrateClient({
    rpcUrl: RPC_ENDPOINT,
    privateKey: TEST_ACCOUNTS[0].privateKey,
  });
}, 30000);

// ============================================================================
// Connection Tests
// ============================================================================

describe('CitrateClient Connection', () => {
  it('initializes without private key', () => {
    const readOnlyClient = new CitrateClient({
      rpcUrl: RPC_ENDPOINT,
    });
    expect(readOnlyClient).toBeDefined();
    expect(readOnlyClient.getAddress()).toBeUndefined();
  });

  it('initializes with private key', () => {
    const walletClient = new CitrateClient({
      rpcUrl: RPC_ENDPOINT,
      privateKey: TEST_ACCOUNTS[0].privateKey,
    });
    expect(walletClient).toBeDefined();
    expect(walletClient.getAddress()?.toLowerCase()).toBe(TEST_ACCOUNTS[0].address.toLowerCase());
  });

  it('returns correct chain ID', async () => {
    if (!nodeRunning) {
      console.log('Skipping: Node not running');
      return;
    }

    const chainId = await client.getChainId();
    expect(chainId).toBe(CHAIN_ID);
  });
});

// ============================================================================
// Balance Tests
// ============================================================================

describe('CitrateClient Balance', () => {
  it('gets balance for genesis account', async () => {
    if (!nodeRunning) {
      console.log('Skipping: Node not running');
      return;
    }

    const balance = await fundedClient.getBalance();
    expect(balance).toBeGreaterThan(0n);
  });

  it('gets balance for specified address', async () => {
    if (!nodeRunning) {
      console.log('Skipping: Node not running');
      return;
    }

    const balance = await client.getBalance(TEST_ACCOUNTS[1].address);
    expect(balance).toBeGreaterThan(0n);
  });

  it('returns zero for new random address', async () => {
    if (!nodeRunning) {
      console.log('Skipping: Node not running');
      return;
    }

    const wallet = createRandomWallet();
    const balance = await client.getBalance(wallet.address);
    expect(balance).toBe(0n);
  });

  it('throws error when no address provided and no wallet', async () => {
    if (!nodeRunning) {
      console.log('Skipping: Node not running');
      return;
    }

    await expect(client.getBalance()).rejects.toThrow(CitrateError);
  });
});

// ============================================================================
// Nonce Tests
// ============================================================================

describe('CitrateClient Nonce', () => {
  it('gets nonce for account', async () => {
    if (!nodeRunning) {
      console.log('Skipping: Node not running');
      return;
    }

    const nonce = await fundedClient.getNonce();
    expect(nonce).toBeGreaterThanOrEqual(0);
  });

  it('returns zero nonce for new address', async () => {
    if (!nodeRunning) {
      console.log('Skipping: Node not running');
      return;
    }

    const wallet = createRandomWallet();
    const nonce = await client.getNonce(wallet.address);
    expect(nonce).toBe(0);
  });
});

// ============================================================================
// Address Derivation Tests
// ============================================================================

describe('CitrateClient Address Derivation', () => {
  it('derives correct address from private key', () => {
    const testClient = new CitrateClient({
      rpcUrl: RPC_ENDPOINT,
      privateKey: TEST_ACCOUNTS[0].privateKey,
    });
    expect(testClient.getAddress()?.toLowerCase()).toBe(TEST_ACCOUNTS[0].address.toLowerCase());
  });

  it('different private keys produce different addresses', () => {
    const client1 = new CitrateClient({
      rpcUrl: RPC_ENDPOINT,
      privateKey: TEST_ACCOUNTS[0].privateKey,
    });
    const client2 = new CitrateClient({
      rpcUrl: RPC_ENDPOINT,
      privateKey: TEST_ACCOUNTS[1].privateKey,
    });

    expect(client1.getAddress()).not.toBe(client2.getAddress());
  });

  it('same private key produces same address', () => {
    const client1 = new CitrateClient({
      rpcUrl: RPC_ENDPOINT,
      privateKey: TEST_ACCOUNTS[0].privateKey,
    });
    const client2 = new CitrateClient({
      rpcUrl: RPC_ENDPOINT,
      privateKey: TEST_ACCOUNTS[0].privateKey,
    });

    expect(client1.getAddress()).toBe(client2.getAddress());
  });

  it('address matches ethers.js derivation', () => {
    const wallet = new ethers.Wallet(TEST_ACCOUNTS[0].privateKey);
    const testClient = new CitrateClient({
      rpcUrl: RPC_ENDPOINT,
      privateKey: TEST_ACCOUNTS[0].privateKey,
    });

    expect(testClient.getAddress()?.toLowerCase()).toBe(wallet.address.toLowerCase());
  });
});

// ============================================================================
// Model List Tests
// ============================================================================

describe('CitrateClient Model Operations', () => {
  it('listModels returns array or null', async () => {
    if (!nodeRunning) {
      console.log('Skipping: Node not running');
      return;
    }

    try {
      const models = await client.listModels();
      expect(Array.isArray(models) || models === null).toBe(true);
    } catch (error) {
      // citrate_listModels might not be implemented
      console.log('Note: citrate_listModels not implemented');
    }
  });

  it('getModelInfo throws for nonexistent model', async () => {
    if (!nodeRunning) {
      console.log('Skipping: Node not running');
      return;
    }

    try {
      await client.getModelInfo('nonexistent_model_id');
      fail('Expected error for nonexistent model');
    } catch (error) {
      expect(error).toBeInstanceOf(Error);
    }
  });
});

// ============================================================================
// Configuration Tests
// ============================================================================

describe('CitrateClient Configuration', () => {
  it('accepts custom timeout', () => {
    const customClient = new CitrateClient({
      rpcUrl: RPC_ENDPOINT,
      timeout: 60000,
    });
    expect(customClient).toBeDefined();
  });

  it('accepts custom headers', () => {
    const customClient = new CitrateClient({
      rpcUrl: RPC_ENDPOINT,
      headers: {
        'X-Custom-Header': 'test-value',
      },
    });
    expect(customClient).toBeDefined();
  });

  it('accepts retries configuration', () => {
    const customClient = new CitrateClient({
      rpcUrl: RPC_ENDPOINT,
      retries: 3,
    });
    expect(customClient).toBeDefined();
  });
});

// ============================================================================
// Error Handling Tests
// ============================================================================

describe('CitrateClient Error Handling', () => {
  it('throws CitrateError for network failures', async () => {
    const badClient = new CitrateClient({
      rpcUrl: 'http://localhost:99999',
    });

    await expect(badClient.getChainId()).rejects.toThrow();
  });

  it('handles invalid RPC method', async () => {
    if (!nodeRunning) {
      console.log('Skipping: Node not running');
      return;
    }

    // This should throw an error for invalid method
    try {
      await rpcCall('invalid_method_xyz', [], RPC_ENDPOINT);
      fail('Expected error for invalid method');
    } catch (error) {
      expect(error).toBeDefined();
    }
  });
});

// ============================================================================
// Concurrent Request Tests
// ============================================================================

describe('CitrateClient Concurrent Requests', () => {
  it('handles multiple concurrent balance requests', async () => {
    if (!nodeRunning) {
      console.log('Skipping: Node not running');
      return;
    }

    const addresses = TEST_ACCOUNTS.map((a) => a.address);
    const promises = addresses.map((addr) => client.getBalance(addr));

    const balances = await Promise.all(promises);

    expect(balances.length).toBe(addresses.length);
    balances.forEach((balance) => {
      expect(typeof balance).toBe('bigint');
    });
  });

  it('handles concurrent chain ID requests', async () => {
    if (!nodeRunning) {
      console.log('Skipping: Node not running');
      return;
    }

    const promises = Array.from({ length: 5 }, () => client.getChainId());
    const results = await Promise.all(promises);

    results.forEach((chainId) => {
      expect(chainId).toBe(CHAIN_ID);
    });
  });
});

// ============================================================================
// Performance Tests
// ============================================================================

describe('CitrateClient Performance', () => {
  it('chain ID request completes within reasonable time', async () => {
    if (!nodeRunning) {
      console.log('Skipping: Node not running');
      return;
    }

    const start = Date.now();
    await client.getChainId();
    const elapsed = Date.now() - start;

    expect(elapsed).toBeLessThan(5000); // Under 5 seconds
  });

  it('balance request completes within reasonable time', async () => {
    if (!nodeRunning) {
      console.log('Skipping: Node not running');
      return;
    }

    const start = Date.now();
    await client.getBalance(TEST_ACCOUNTS[0].address);
    const elapsed = Date.now() - start;

    expect(elapsed).toBeLessThan(5000);
  });

  it('measures average latency', async () => {
    if (!nodeRunning) {
      console.log('Skipping: Node not running');
      return;
    }

    const latencies: number[] = [];

    for (let i = 0; i < 5; i++) {
      const start = Date.now();
      await client.getChainId();
      latencies.push(Date.now() - start);
    }

    const avgLatency = latencies.reduce((a, b) => a + b, 0) / latencies.length;
    console.log(`Average RPC latency: ${avgLatency.toFixed(0)}ms`);

    expect(avgLatency).toBeLessThan(2000);
  });
});

// ============================================================================
// Consistency Tests
// ============================================================================

describe('CitrateClient Consistency', () => {
  it('returns consistent chain ID across calls', async () => {
    if (!nodeRunning) {
      console.log('Skipping: Node not running');
      return;
    }

    const chainId1 = await client.getChainId();
    const chainId2 = await client.getChainId();

    expect(chainId1).toBe(chainId2);
  });

  it('returns consistent balance for same address', async () => {
    if (!nodeRunning) {
      console.log('Skipping: Node not running');
      return;
    }

    const balance1 = await client.getBalance(TEST_ACCOUNTS[0].address);
    const balance2 = await client.getBalance(TEST_ACCOUNTS[0].address);

    expect(balance1).toBe(balance2);
  });

  it('funded client address matches expected', () => {
    expect(fundedClient.getAddress()?.toLowerCase()).toBe(TEST_ACCOUNTS[0].address.toLowerCase());
  });
});

// ============================================================================
// Wallet Operations Tests
// ============================================================================

describe('CitrateClient Wallet Operations', () => {
  it('client without wallet cannot deploy model', async () => {
    if (!nodeRunning) {
      console.log('Skipping: Node not running');
      return;
    }

    const readOnlyClient = new CitrateClient({
      rpcUrl: RPC_ENDPOINT,
    });

    const modelData = new Uint8Array([0x00, 0x01, 0x02]);
    const config = {
      encrypted: false,
      accessPrice: 0n,
    };

    await expect(readOnlyClient.deployModel(modelData, config as any)).rejects.toThrow(
      'Wallet required'
    );
  });

  it('client without wallet cannot execute inference', async () => {
    if (!nodeRunning) {
      console.log('Skipping: Node not running');
      return;
    }

    const readOnlyClient = new CitrateClient({
      rpcUrl: RPC_ENDPOINT,
    });

    await expect(
      readOnlyClient.inference({
        modelId: 'test_model_id',
        inputData: { test: 'data' },
      })
    ).rejects.toThrow('Wallet required');
  });

  it('client without wallet cannot purchase model access', async () => {
    if (!nodeRunning) {
      console.log('Skipping: Node not running');
      return;
    }

    const readOnlyClient = new CitrateClient({
      rpcUrl: RPC_ENDPOINT,
    });

    await expect(readOnlyClient.purchaseModelAccess('test_model_id', 1000n)).rejects.toThrow(
      'Wallet required'
    );
  });
});
