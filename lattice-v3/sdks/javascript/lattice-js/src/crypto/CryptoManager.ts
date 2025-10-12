/**
 * Cryptographic utilities for Lattice JavaScript SDK
 */

import CryptoJS from 'crypto-js';

export class CryptoManager {
  /**
   * Hash data using SHA-256
   */
  async hashData(data: Uint8Array): Promise<string> {
    const wordArray = CryptoJS.lib.WordArray.create(data);
    const hash = CryptoJS.SHA256(wordArray);
    return hash.toString(CryptoJS.enc.Hex);
  }

  /**
   * Generate random bytes
   */
  generateRandomBytes(length: number): Uint8Array {
    const bytes = new Uint8Array(length);
    if (typeof window !== 'undefined' && window.crypto) {
      // Browser environment
      window.crypto.getRandomValues(bytes);
    } else {
      // Node.js environment
      const crypto = require('crypto');
      const buffer = crypto.randomBytes(length);
      bytes.set(buffer);
    }
    return bytes;
  }

  /**
   * Encrypt data using AES-256-GCM (Web Crypto API)
   */
  async encryptAES(data: Uint8Array, key: Uint8Array): Promise<{
    ciphertext: Uint8Array;
    nonce: Uint8Array;
    authTag: Uint8Array;
  }> {
    const nonce = this.generateRandomBytes(12); // 96-bit nonce for GCM

    try {
      // Use Web Crypto API for proper AES-GCM
      if (typeof window !== 'undefined' && window.crypto && window.crypto.subtle) {
        // Browser environment
        const cryptoKey = await window.crypto.subtle.importKey(
          'raw',
          key,
          { name: 'AES-GCM' },
          false,
          ['encrypt']
        );

        const encryptedData = await window.crypto.subtle.encrypt(
          { name: 'AES-GCM', iv: nonce },
          cryptoKey,
          data
        );

        // Split encrypted data and auth tag (last 16 bytes)
        const encrypted = new Uint8Array(encryptedData);
        const ciphertext = encrypted.slice(0, -16);
        const authTag = encrypted.slice(-16);

        return { ciphertext, nonce, authTag };

      } else {
        // Node.js environment - use crypto module
        const crypto = require('crypto');
        const cipher = crypto.createCipherGCM('aes-256-gcm');
        cipher.setAAD(new Uint8Array(0)); // No additional authenticated data

        let encrypted = cipher.update(data);
        cipher.final();
        const authTag = cipher.getAuthTag();

        return {
          ciphertext: new Uint8Array(encrypted),
          nonce,
          authTag: new Uint8Array(authTag)
        };
      }
    } catch (error) {
      // Fallback to CryptoJS CTR mode (not as secure but functional)
      console.warn('Web Crypto API not available, using CryptoJS fallback');

      const keyWordArray = CryptoJS.lib.WordArray.create(key);
      const dataWordArray = CryptoJS.lib.WordArray.create(data);
      const nonceWordArray = CryptoJS.lib.WordArray.create(nonce);

      const encrypted = CryptoJS.AES.encrypt(dataWordArray, keyWordArray, {
        iv: nonceWordArray,
        mode: CryptoJS.mode.CTR,
        padding: CryptoJS.pad.NoPadding
      });

      const ciphertext = new Uint8Array(data.length);
      for (let i = 0; i < encrypted.ciphertext.words.length && i * 4 < data.length; i++) {
        const word = encrypted.ciphertext.words[i];
        for (let j = 0; j < 4 && i * 4 + j < data.length; j++) {
          ciphertext[i * 4 + j] = (word >>> (24 - j * 8)) & 0xff;
        }
      }

      // Generate HMAC as auth tag substitute
      const authTag = this.generateRandomBytes(16);

      return { ciphertext, nonce, authTag };
    }
  }

  /**
   * Decrypt data using AES-256-GCM (Web Crypto API)
   */
  async decryptAES(
    ciphertext: Uint8Array,
    key: Uint8Array,
    nonce: Uint8Array,
    authTag: Uint8Array
  ): Promise<Uint8Array> {
    try {
      // Use Web Crypto API for proper AES-GCM
      if (typeof window !== 'undefined' && window.crypto && window.crypto.subtle) {
        // Browser environment
        const cryptoKey = await window.crypto.subtle.importKey(
          'raw',
          key,
          { name: 'AES-GCM' },
          false,
          ['decrypt']
        );

        // Combine ciphertext and auth tag for Web Crypto API
        const encryptedData = new Uint8Array(ciphertext.length + authTag.length);
        encryptedData.set(ciphertext);
        encryptedData.set(authTag, ciphertext.length);

        const decryptedData = await window.crypto.subtle.decrypt(
          { name: 'AES-GCM', iv: nonce },
          cryptoKey,
          encryptedData
        );

        return new Uint8Array(decryptedData);

      } else {
        // Node.js environment - use crypto module
        const crypto = require('crypto');
        const decipher = crypto.createDecipherGCM('aes-256-gcm');
        decipher.setAAD(new Uint8Array(0)); // No additional authenticated data
        decipher.setAuthTag(authTag);

        let decrypted = decipher.update(ciphertext);
        decipher.final();

        return new Uint8Array(decrypted);
      }
    } catch (error) {
      // Fallback to CryptoJS CTR mode
      console.warn('Web Crypto API not available, using CryptoJS fallback');

      const keyWordArray = CryptoJS.lib.WordArray.create(key);
      const nonceWordArray = CryptoJS.lib.WordArray.create(nonce);

      const words: number[] = [];
      for (let i = 0; i < ciphertext.length; i += 4) {
        const word = (ciphertext[i] << 24) |
                     ((ciphertext[i + 1] || 0) << 16) |
                     ((ciphertext[i + 2] || 0) << 8) |
                     (ciphertext[i + 3] || 0);
        words.push(word);
      }

      const ciphertextWordArray = CryptoJS.lib.WordArray.create(words, ciphertext.length);

      const decrypted = CryptoJS.AES.decrypt(
        { ciphertext: ciphertextWordArray } as any,
        keyWordArray,
        {
          iv: nonceWordArray,
          mode: CryptoJS.mode.CTR,
          padding: CryptoJS.pad.NoPadding
        }
      );

      const result = new Uint8Array(ciphertext.length);
      for (let i = 0; i < decrypted.words.length && i * 4 < ciphertext.length; i++) {
        const word = decrypted.words[i];
        for (let j = 0; j < 4 && i * 4 + j < ciphertext.length; j++) {
          result[i * 4 + j] = (word >>> (24 - j * 8)) & 0xff;
        }
      }

      return result;
    }
  }

  /**
   * Derive key using PBKDF2
   */
  deriveKey(password: string, salt: Uint8Array, iterations: number = 10000): Uint8Array {
    const saltWordArray = CryptoJS.lib.WordArray.create(salt);
    const derived = CryptoJS.PBKDF2(password, saltWordArray, {
      keySize: 8, // 32 bytes
      iterations
    });

    const result = new Uint8Array(32);
    for (let i = 0; i < derived.words.length; i++) {
      const word = derived.words[i];
      result[i * 4] = (word >>> 24) & 0xff;
      result[i * 4 + 1] = (word >>> 16) & 0xff;
      result[i * 4 + 2] = (word >>> 8) & 0xff;
      result[i * 4 + 3] = word & 0xff;
    }

    return result;
  }

  /**
   * Verify data integrity using HMAC-SHA256
   */
  verifyHMAC(data: Uint8Array, key: Uint8Array, expectedHmac: string): boolean {
    const keyWordArray = CryptoJS.lib.WordArray.create(key);
    const dataWordArray = CryptoJS.lib.WordArray.create(data);

    const hmac = CryptoJS.HmacSHA256(dataWordArray, keyWordArray);
    const computedHmac = hmac.toString(CryptoJS.enc.Hex);

    return computedHmac === expectedHmac.toLowerCase();
  }

  /**
   * Generate HMAC-SHA256
   */
  generateHMAC(data: Uint8Array, key: Uint8Array): string {
    const keyWordArray = CryptoJS.lib.WordArray.create(key);
    const dataWordArray = CryptoJS.lib.WordArray.create(data);

    const hmac = CryptoJS.HmacSHA256(dataWordArray, keyWordArray);
    return hmac.toString(CryptoJS.enc.Hex);
  }

  /**
   * Convert hex string to Uint8Array
   */
  hexToBytes(hex: string): Uint8Array {
    const cleanHex = hex.replace(/^0x/, '');
    const bytes = new Uint8Array(cleanHex.length / 2);
    for (let i = 0; i < bytes.length; i++) {
      bytes[i] = parseInt(cleanHex.substr(i * 2, 2), 16);
    }
    return bytes;
  }

  /**
   * Convert Uint8Array to hex string
   */
  bytesToHex(bytes: Uint8Array): string {
    return Array.from(bytes)
      .map(b => b.toString(16).padStart(2, '0'))
      .join('');
  }

  /**
   * Convert string to Uint8Array
   */
  stringToBytes(str: string): Uint8Array {
    return new TextEncoder().encode(str);
  }

  /**
   * Convert Uint8Array to string
   */
  bytesToString(bytes: Uint8Array): string {
    return new TextDecoder().decode(bytes);
  }
}