/**
 * Cryptography-related type definitions
 */

export interface KeyPair {
  privateKey: string;
  publicKey: string;
  address: string;
}

export interface EncryptedData {
  ciphertext: Uint8Array;
  nonce: Uint8Array;
  authTag: Uint8Array;
}

export interface KeyShareData {
  x: string;
  y: string;
  threshold: string;
}

export interface HDKeyOptions {
  path: string;
  purpose: 'signing' | 'encryption' | 'model';
  chainCode?: string;
}

export interface SecureEnclaveConfig {
  enabled: boolean;
  attestation: boolean;
  sealedStorage: boolean;
}

export interface ZKProofConfig {
  circuit: string;
  inputs: Record<string, any>;
  publicInputs: string[];
  enableOptimizations: boolean;
}