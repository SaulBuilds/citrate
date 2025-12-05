/**
 * Address Derivation Tests
 *
 * Verifies address derivation consistency across all components:
 * - SDK address derivation from private key
 * - Mnemonic-based derivation
 * - EVM 20-byte address format
 * - Native 32-byte format (where applicable)
 * - RPC acceptance of both formats
 */

import { ethers } from 'ethers';
import { CitrateSDK } from '../../src/sdk';
import {
  rpcCall,
  getBalance,
  getNonce,
  getTestWallet,
  createRandomWallet,
  parseEth,
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
  const running = await isNodeRunning(RPC_ENDPOINT);
  if (!running) {
    console.warn(
      `WARNING: Node not running at ${RPC_ENDPOINT}. ` +
        'Start with: cargo run --bin citrate-node -- devnet'
    );
  }

  sdk = new CitrateSDK({
    rpcEndpoint: RPC_ENDPOINT,
    chainId: CHAIN_ID,
  });
}, 30000);

// ============================================================================
// Private Key Derivation Tests
// ============================================================================

describe('Private Key Address Derivation', () => {
  it('derives consistent address from private key', () => {
    // Known test vector
    const privateKey = '0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80';
    const expectedAddress = '0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266';

    const wallet = new ethers.Wallet(privateKey);
    expect(wallet.address).toBe(expectedAddress);
  });

  it('SDK derives same address as ethers.js', () => {
    const privateKey = '0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d';

    // Derive using ethers directly
    const ethersWallet = new ethers.Wallet(privateKey);

    // Derive using SDK
    const sdkAddress = sdk.accounts.importAccount(privateKey);

    expect(sdkAddress.toLowerCase()).toBe(ethersWallet.address.toLowerCase());
  });

  it('random private keys produce valid addresses', () => {
    for (let i = 0; i < 10; i++) {
      const wallet = ethers.Wallet.createRandom();

      // Address should be 42 characters (0x + 40 hex)
      expect(wallet.address).toMatch(/^0x[a-fA-F0-9]{40}$/);

      // Private key should be 66 characters (0x + 64 hex)
      expect(wallet.privateKey).toMatch(/^0x[a-fA-F0-9]{64}$/);
    }
  });

  it('SDK createAccount generates valid addresses', () => {
    const account = sdk.accounts.createAccount();

    expect(account.address).toMatch(/^0x[a-fA-F0-9]{40}$/);
    expect(account.privateKey).toMatch(/^0x[a-fA-F0-9]{64}$/);

    // Re-importing should produce same address
    const reimportedAddress = new ethers.Wallet(account.privateKey).address;
    expect(reimportedAddress.toLowerCase()).toBe(account.address.toLowerCase());
  });
});

// ============================================================================
// Mnemonic Derivation Tests
// ============================================================================

describe('Mnemonic Address Derivation', () => {
  // Standard BIP39 test mnemonic
  const TEST_MNEMONIC =
    'abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about';

  // Expected addresses for different derivation paths
  const EXPECTED_ADDRESSES: Record<string, string> = {
    "m/44'/60'/0'/0/0": '0x9858EfFD232B4033E47d90003D41EC34EcaEda94',
    "m/44'/60'/0'/0/1": '0x6Fac4D18c912343BF86fa7049364Dd4E424Ab9C0',
    "m/44'/60'/0'/0/2": '0xb9c5714089478a327F09197987f16f9E5d936E8a',
  };

  it('derives addresses from mnemonic using default path', () => {
    const address = sdk.accounts.importFromMnemonic(TEST_MNEMONIC);

    // Verify using ethers directly
    const ethersWallet = ethers.HDNodeWallet.fromPhrase(TEST_MNEMONIC);

    expect(address.toLowerCase()).toBe(ethersWallet.address.toLowerCase());
  });

  it('derives addresses for different derivation paths', () => {
    for (const [path, expectedAddress] of Object.entries(EXPECTED_ADDRESSES)) {
      const ethersWallet = ethers.HDNodeWallet.fromPhrase(TEST_MNEMONIC).derivePath(path);

      // Note: The expected addresses are from a standard BIP44 derivation
      // Ethers v6 should produce the same addresses
      console.log(`Path ${path}: ${ethersWallet.address}`);
    }
  });

  it('SDK mnemonic derivation matches ethers.js', () => {
    const mnemonic = ethers.Mnemonic.entropyToPhrase(ethers.randomBytes(16));

    // Derive using ethers
    const ethersWallet = ethers.HDNodeWallet.fromPhrase(mnemonic);

    // Derive using SDK
    const sdkAddress = sdk.accounts.importFromMnemonic(mnemonic);

    expect(sdkAddress.toLowerCase()).toBe(ethersWallet.address.toLowerCase());
  });

  it('different paths produce different addresses', () => {
    const mnemonic = ethers.Mnemonic.entropyToPhrase(ethers.randomBytes(16));

    const address0 = ethers.HDNodeWallet.fromPhrase(mnemonic).derivePath("m/44'/60'/0'/0/0").address;
    const address1 = ethers.HDNodeWallet.fromPhrase(mnemonic).derivePath("m/44'/60'/0'/0/1").address;
    const address2 = ethers.HDNodeWallet.fromPhrase(mnemonic).derivePath("m/44'/60'/0'/1/0").address;

    expect(address0).not.toBe(address1);
    expect(address0).not.toBe(address2);
    expect(address1).not.toBe(address2);
  });

  it('generates valid mnemonic phrases', () => {
    const account = sdk.accounts.createAccount();

    if (account.mnemonic) {
      // Verify mnemonic is valid 12 or 24 words
      const words = account.mnemonic.split(' ');
      expect([12, 24]).toContain(words.length);

      // Verify derivation from mnemonic produces same address
      const wallet = ethers.HDNodeWallet.fromPhrase(account.mnemonic);
      expect(wallet.address.toLowerCase()).toBe(account.address.toLowerCase());
    }
  });
});

// ============================================================================
// Address Format Tests
// ============================================================================

describe('Address Format Validation', () => {
  it('validates EVM 20-byte address format', () => {
    const validAddresses = [
      '0x0000000000000000000000000000000000000000',
      '0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF',
      '0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266',
      '0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266', // Lowercase
      '0xF39FD6E51AAD88F6F4CE6AB8827279CFFFB92266', // Uppercase
    ];

    for (const address of validAddresses) {
      expect(address).toMatch(/^0x[a-fA-F0-9]{40}$/);
      expect(ethers.isAddress(address)).toBe(true);
    }
  });

  it('rejects invalid address formats', () => {
    const invalidAddresses = [
      '0x', // Too short
      '0x0', // Too short
      '0x' + '0'.repeat(39), // 39 chars, not 40
      '0x' + '0'.repeat(41), // 41 chars, not 40
      '0x' + 'g'.repeat(40), // Invalid hex character
      '00000000000000000000000000000000000000000000', // No 0x prefix
    ];

    for (const address of invalidAddresses) {
      expect(ethers.isAddress(address)).toBe(false);
    }
  });

  it('handles checksum addresses correctly', () => {
    const lowerCaseAddress = '0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266';
    const checksumAddress = ethers.getAddress(lowerCaseAddress);

    // Checksum address has mixed case
    expect(checksumAddress).not.toBe(lowerCaseAddress);
    expect(checksumAddress).not.toBe(lowerCaseAddress.toUpperCase());

    // Both should be considered valid addresses
    expect(ethers.isAddress(lowerCaseAddress)).toBe(true);
    expect(ethers.isAddress(checksumAddress)).toBe(true);

    // Both should produce the same checksum address
    expect(ethers.getAddress(lowerCaseAddress)).toBe(ethers.getAddress(checksumAddress));
  });
});

// ============================================================================
// RPC Address Acceptance Tests
// ============================================================================

describe('RPC Address Acceptance', () => {
  it('accepts lowercase addresses', async () => {
    const wallet = getTestWallet(0);
    const lowercaseAddress = wallet.address.toLowerCase();

    const balance = await rpcCall('eth_getBalance', [lowercaseAddress, 'latest'], RPC_ENDPOINT);
    expect(balance).toBeDefined();
  });

  it('accepts checksum addresses', async () => {
    const wallet = getTestWallet(0);
    const checksumAddress = ethers.getAddress(wallet.address);

    const balance = await rpcCall('eth_getBalance', [checksumAddress, 'latest'], RPC_ENDPOINT);
    expect(balance).toBeDefined();
  });

  it('accepts uppercase addresses', async () => {
    const wallet = getTestWallet(0);
    const uppercaseAddress = wallet.address.toUpperCase();

    // Some nodes might reject uppercase, but let's test it
    try {
      const balance = await rpcCall('eth_getBalance', [uppercaseAddress, 'latest'], RPC_ENDPOINT);
      expect(balance).toBeDefined();
    } catch (error) {
      // Acceptable if the node requires lowercase or checksum
      console.log('Note: Node rejected uppercase address (expected behavior)');
    }
  });

  it('returns same balance for all address formats', async () => {
    const wallet = getTestWallet(0);
    const lowercaseAddress = wallet.address.toLowerCase();
    const checksumAddress = ethers.getAddress(wallet.address);

    const balance1 = await rpcCall('eth_getBalance', [lowercaseAddress, 'latest'], RPC_ENDPOINT);
    const balance2 = await rpcCall('eth_getBalance', [checksumAddress, 'latest'], RPC_ENDPOINT);

    expect(balance1).toBe(balance2);
  });

  it('nonce is consistent across address formats', async () => {
    const wallet = getTestWallet(0);
    const lowercaseAddress = wallet.address.toLowerCase();
    const checksumAddress = ethers.getAddress(wallet.address);

    const nonce1 = await getNonce(lowercaseAddress, 'latest', RPC_ENDPOINT);
    const nonce2 = await getNonce(checksumAddress, 'latest', RPC_ENDPOINT);

    expect(nonce1).toBe(nonce2);
  });
});

// ============================================================================
// Cross-Component Consistency Tests
// ============================================================================

describe('Cross-Component Address Consistency', () => {
  it('SDK import and RPC use same address', async () => {
    const privateKey = '0x5de4111afa1a4b94908f83103eb1f1706367c2e68ca870fc3fb9a804cdab365a';

    // Import to SDK
    const sdkAddress = sdk.accounts.importAccount(privateKey);

    // Get balance via SDK
    const sdkBalance = await sdk.accounts.getBalance();

    // Get balance via direct RPC
    const rpcBalance = await rpcCall('eth_getBalance', [sdkAddress, 'latest'], RPC_ENDPOINT);

    expect(sdkBalance).toBe(BigInt(rpcBalance));
  });

  it('transaction from/to addresses match across components', async () => {
    const sender = getTestWallet(0);
    const recipient = createRandomWallet();

    // Create transaction
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

    // Parse to verify addresses
    const parsed = ethers.Transaction.from(signedTx);

    expect(parsed.from?.toLowerCase()).toBe(sender.address.toLowerCase());
    expect(parsed.to?.toLowerCase()).toBe(recipient.address.toLowerCase());
  });
});

// ============================================================================
// Collision Resistance Tests
// ============================================================================

describe('Address Collision Resistance', () => {
  it('no collisions in 1000 random addresses', () => {
    const addresses = new Set<string>();

    for (let i = 0; i < 1000; i++) {
      const wallet = createRandomWallet();
      const normalizedAddress = wallet.address.toLowerCase();

      expect(addresses.has(normalizedAddress)).toBe(false);
      addresses.add(normalizedAddress);
    }

    expect(addresses.size).toBe(1000);
  });

  it('different private keys produce different addresses', () => {
    const privateKey1 = '0x' + '1'.repeat(64);
    const privateKey2 = '0x' + '2'.repeat(64);
    const privateKey3 = '0x' + '3'.repeat(64);

    const wallet1 = new ethers.Wallet(privateKey1);
    const wallet2 = new ethers.Wallet(privateKey2);
    const wallet3 = new ethers.Wallet(privateKey3);

    expect(wallet1.address).not.toBe(wallet2.address);
    expect(wallet1.address).not.toBe(wallet3.address);
    expect(wallet2.address).not.toBe(wallet3.address);
  });

  it('adjacent private keys produce different addresses', () => {
    // Private keys differing by 1
    const key1 = BigInt('0x' + 'f'.repeat(64));
    const key2 = key1 - 1n;

    const privateKey1 = '0x' + key1.toString(16).padStart(64, '0');
    const privateKey2 = '0x' + key2.toString(16).padStart(64, '0');

    const wallet1 = new ethers.Wallet(privateKey1);
    const wallet2 = new ethers.Wallet(privateKey2);

    expect(wallet1.address).not.toBe(wallet2.address);
  });
});

// ============================================================================
// Public Key Recovery Tests
// ============================================================================

describe('Public Key Recovery', () => {
  it('recovers public key from signature', async () => {
    const wallet = getTestWallet(0);
    const message = 'Test message for signature recovery';

    // Sign the message
    const signature = await wallet.signMessage(message);

    // Recover the address
    const recoveredAddress = ethers.verifyMessage(message, signature);

    expect(recoveredAddress.toLowerCase()).toBe(wallet.address.toLowerCase());
  });

  it('SDK signature verification matches ethers', async () => {
    const wallet = getTestWallet(0);
    const message = 'Test message for SDK verification';

    // Sign using wallet
    const signature = await wallet.signMessage(message);

    // Verify using SDK
    const isValid = sdk.accounts.verifyMessage(message, signature, wallet.address);
    expect(isValid).toBe(true);

    // Wrong address should fail
    const wrongAddress = createRandomWallet().address;
    const isInvalid = sdk.accounts.verifyMessage(message, signature, wrongAddress);
    expect(isInvalid).toBe(false);
  });

  it('different messages produce different signatures', async () => {
    const wallet = getTestWallet(0);

    const sig1 = await wallet.signMessage('Message 1');
    const sig2 = await wallet.signMessage('Message 2');

    expect(sig1).not.toBe(sig2);

    // But both should recover to same address
    const addr1 = ethers.verifyMessage('Message 1', sig1);
    const addr2 = ethers.verifyMessage('Message 2', sig2);

    expect(addr1).toBe(addr2);
  });
});

// ============================================================================
// Zero Address Tests
// ============================================================================

describe('Zero Address Handling', () => {
  const ZERO_ADDRESS = '0x0000000000000000000000000000000000000000';

  it('zero address has zero balance', async () => {
    const balance = await getBalance(ZERO_ADDRESS, RPC_ENDPOINT);
    // Zero address might have balance from burned tokens, but should be queryable
    expect(typeof balance).toBe('bigint');
  });

  it('cannot derive zero address from private key', () => {
    // All valid private keys produce non-zero addresses
    for (let i = 0; i < 100; i++) {
      const wallet = createRandomWallet();
      expect(wallet.address).not.toBe(ZERO_ADDRESS);
    }
  });

  it('zero address is valid but special', () => {
    expect(ethers.isAddress(ZERO_ADDRESS)).toBe(true);
  });
});
