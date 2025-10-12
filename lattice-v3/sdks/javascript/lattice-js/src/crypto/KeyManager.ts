/**
 * Key management for Lattice JavaScript SDK
 */

import { ethers } from 'ethers';
import { CryptoManager } from './CryptoManager';
import { EncryptionConfig } from '../types/Model';

export interface EncryptedModelResult {
  encryptedData: Uint8Array;
  metadata: {
    algorithm: string;
    nonce: string;
    keyDerivation: string;
    encryptedKey: string;
    accessControl: boolean;
    keyShares?: Array<{ x: string; y: string; threshold: string }>;
  };
}

export class KeyManager {
  private wallet: ethers.Wallet;
  private cryptoManager: CryptoManager;

  constructor(privateKey?: string) {
    if (privateKey) {
      this.wallet = new ethers.Wallet(privateKey);
    } else {
      this.wallet = ethers.Wallet.createRandom();
    }
    this.cryptoManager = new CryptoManager();
  }

  /**
   * Get Ethereum address
   */
  getAddress(): string {
    return this.wallet.address;
  }

  /**
   * Get private key
   */
  getPrivateKey(): string {
    return this.wallet.privateKey;
  }

  /**
   * Get public key for ECDH
   */
  getPublicKey(): string {
    // Derive public key from private key
    const privateKeyBytes = this.cryptoManager.hexToBytes(this.wallet.privateKey.slice(2));
    // In production, use proper secp256k1 point multiplication
    // For now, use a simplified approach
    const hash = ethers.keccak256(privateKeyBytes);
    return hash.slice(2, 66); // 32 bytes as hex
  }

  /**
   * Sign transaction
   */
  async signTransaction(transaction: ethers.TransactionRequest): Promise<string> {
    const signedTx = await this.wallet.signTransaction(transaction);
    return signedTx;
  }

  /**
   * Encrypt model data
   */
  async encryptModel(
    modelData: Uint8Array,
    config?: EncryptionConfig
  ): Promise<EncryptedModelResult> {
    const algorithm = config?.algorithm || 'AES-256-GCM';
    const keyDerivation = config?.keyDerivation || 'HKDF-SHA256';

    // Generate random encryption key
    const encryptionKey = this.cryptoManager.generateRandomBytes(32);

    // Encrypt model data
    const encrypted = await this.cryptoManager.encryptAES(modelData, encryptionKey);

    // Encrypt the encryption key for owner
    const encryptedKey = await this.encryptKeyForOwner(encryptionKey);

    // Create metadata
    const metadata = {
      algorithm,
      nonce: this.cryptoManager.bytesToHex(encrypted.nonce),
      keyDerivation,
      encryptedKey,
      accessControl: config?.accessControl || true
    };

    // Add threshold sharing if enabled
    if (config?.thresholdShares && config.thresholdShares > 0) {
      const keyShares = this.createKeyShares(
        encryptionKey,
        config.thresholdShares,
        config.totalShares
      );
      metadata.keyShares = keyShares;
    }

    // Combine ciphertext and auth tag
    const encryptedData = new Uint8Array(encrypted.ciphertext.length + encrypted.authTag.length);
    encryptedData.set(encrypted.ciphertext);
    encryptedData.set(encrypted.authTag, encrypted.ciphertext.length);

    return {
      encryptedData,
      metadata
    };
  }

  /**
   * Decrypt model data
   */
  async decryptModel(
    encryptedData: Uint8Array,
    metadata: any
  ): Promise<Uint8Array> {
    // Extract ciphertext and auth tag
    const authTagLength = 16;
    const ciphertext = encryptedData.slice(0, -authTagLength);
    const authTag = encryptedData.slice(-authTagLength);
    const nonce = this.cryptoManager.hexToBytes(metadata.nonce);

    // Decrypt the encryption key
    const encryptionKey = await this.decryptKeyFromOwner(metadata.encryptedKey);

    // Decrypt model data
    const decrypted = await this.cryptoManager.decryptAES(
      ciphertext,
      encryptionKey,
      nonce,
      authTag
    );

    return decrypted;
  }

  /**
   * Encrypt arbitrary data
   */
  async encryptData(data: string): Promise<string> {
    const dataBytes = this.cryptoManager.stringToBytes(data);
    const key = this.cryptoManager.generateRandomBytes(32);

    const encrypted = await this.cryptoManager.encryptAES(dataBytes, key);

    const package_ = {
      ciphertext: this.cryptoManager.bytesToHex(encrypted.ciphertext),
      nonce: this.cryptoManager.bytesToHex(encrypted.nonce),
      authTag: this.cryptoManager.bytesToHex(encrypted.authTag),
      key: this.cryptoManager.bytesToHex(key)
    };

    return JSON.stringify(package_);
  }

  /**
   * Decrypt arbitrary data
   */
  async decryptData(encryptedPackage: string): Promise<string> {
    const package_ = JSON.parse(encryptedPackage);

    const ciphertext = this.cryptoManager.hexToBytes(package_.ciphertext);
    const nonce = this.cryptoManager.hexToBytes(package_.nonce);
    const authTag = this.cryptoManager.hexToBytes(package_.authTag);
    const key = this.cryptoManager.hexToBytes(package_.key);

    const decrypted = await this.cryptoManager.decryptAES(ciphertext, key, nonce, authTag);
    return this.cryptoManager.bytesToString(decrypted);
  }

  /**
   * Derive shared key using simplified ECDH
   */
  async deriveSharedKey(peerPublicKey: string): Promise<Uint8Array> {
    // Simplified ECDH implementation
    const privateKeyBytes = this.cryptoManager.hexToBytes(this.wallet.privateKey.slice(2));
    const peerKeyBytes = this.cryptoManager.hexToBytes(peerPublicKey);

    // Combine keys and hash
    const combined = new Uint8Array(privateKeyBytes.length + peerKeyBytes.length);
    combined.set(privateKeyBytes);
    combined.set(peerKeyBytes, privateKeyBytes.length);

    const sharedSecret = await this.cryptoManager.hashData(combined);
    return this.cryptoManager.hexToBytes(sharedSecret);
  }

  /**
   * Encrypt key for model owner
   */
  private async encryptKeyForOwner(key: Uint8Array): Promise<string> {
    const ownerKey = await this.cryptoManager.hashData(
      this.cryptoManager.hexToBytes(this.wallet.privateKey.slice(2))
    );
    const ownerKeyBytes = this.cryptoManager.hexToBytes(ownerKey);

    const encrypted = await this.cryptoManager.encryptAES(key, ownerKeyBytes);

    return JSON.stringify({
      encryptedKey: this.cryptoManager.bytesToHex(encrypted.ciphertext),
      nonce: this.cryptoManager.bytesToHex(encrypted.nonce),
      authTag: this.cryptoManager.bytesToHex(encrypted.authTag)
    });
  }

  /**
   * Decrypt key for model owner
   */
  private async decryptKeyFromOwner(encryptedKeyPackage: string): Promise<Uint8Array> {
    const package_ = JSON.parse(encryptedKeyPackage);

    const ownerKey = await this.cryptoManager.hashData(
      this.cryptoManager.hexToBytes(this.wallet.privateKey.slice(2))
    );
    const ownerKeyBytes = this.cryptoManager.hexToBytes(ownerKey);

    const encryptedKey = this.cryptoManager.hexToBytes(package_.encryptedKey);
    const nonce = this.cryptoManager.hexToBytes(package_.nonce);
    const authTag = this.cryptoManager.hexToBytes(package_.authTag);

    return await this.cryptoManager.decryptAES(encryptedKey, ownerKeyBytes, nonce, authTag);
  }

  /**
   * Create Shamir's secret shares for key (simplified)
   */
  private createKeyShares(
    key: Uint8Array,
    threshold: number,
    total: number
  ): Array<{ x: string; y: string; threshold: string }> {
    const shares: Array<{ x: string; y: string; threshold: string }> = [];

    for (let i = 1; i <= total; i++) {
      // Simplified share generation - in production use proper finite field arithmetic
      const combined = new Uint8Array(key.length + 4);
      combined.set(key);
      combined.set(new Uint8Array([i, 0, 0, 0]), key.length);

      const shareValue = this.cryptoManager.bytesToHex(combined.slice(0, 32));

      shares.push({
        x: i.toString(),
        y: shareValue,
        threshold: threshold.toString()
      });
    }

    return shares;
  }

  /**
   * Reconstruct key from Shamir's shares (simplified)
   */
  reconstructKeyFromShares(shares: Array<{ x: string; y: string; threshold: string }>): Uint8Array {
    const threshold = parseInt(shares[0].threshold);

    if (shares.length < threshold) {
      throw new Error('Insufficient shares for key reconstruction');
    }

    // Simplified reconstruction - in production use proper Lagrange interpolation
    const activeShares = shares.slice(0, threshold);
    let combined = new Uint8Array(32);

    for (const share of activeShares) {
      const shareBytes = this.cryptoManager.hexToBytes(share.y);
      for (let i = 0; i < combined.length; i++) {
        combined[i] ^= shareBytes[i];
      }
    }

    return combined;
  }
}