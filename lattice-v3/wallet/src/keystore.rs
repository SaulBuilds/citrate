use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use argon2::{
    password_hash::{PasswordHasher, SaltString},
    Argon2,
};
use ed25519_dalek::{SigningKey, VerifyingKey};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::errors::WalletError;

/// Encrypted key storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedKey {
    /// Encrypted private key
    pub ciphertext: Vec<u8>,
    /// Salt for key derivation
    pub salt: String,
    /// Nonce for AES-GCM
    pub nonce: Vec<u8>,
    /// Public key (not encrypted)
    pub public_key: Vec<u8>,
    /// Optional key alias
    pub alias: Option<String>,
}

/// Key store for managing encrypted keys
pub struct KeyStore {
    /// Path to keystore file
    path: PathBuf,
    /// Encrypted keys
    keys: Vec<EncryptedKey>,
    /// Decrypted keys (in memory when unlocked)
    unlocked: Vec<SigningKey>,
    /// Whether keystore is locked
    locked: bool,
}

impl KeyStore {
    /// Create new keystore at path
    pub fn new(path: impl AsRef<Path>) -> Result<Self, WalletError> {
        let path = path.as_ref().to_path_buf();
        
        // Load existing keys if file exists
        let keys = if path.exists() {
            let data = std::fs::read(&path)?;
            serde_json::from_slice(&data)?
        } else {
            Vec::new()
        };
        
        Ok(Self {
            path,
            keys,
            unlocked: Vec::new(),
            locked: true,
        })
    }
    
    /// Generate new key pair
    pub fn generate_key(&mut self, password: &str, alias: Option<String>) -> Result<VerifyingKey, WalletError> {
        // Generate new signing key
        let mut secret_bytes = [0u8; 32];
        OsRng.fill_bytes(&mut secret_bytes);
        let signing_key = SigningKey::from_bytes(&secret_bytes);
        let verifying_key = signing_key.verifying_key();
        
        // Encrypt and store
        let encrypted = self.encrypt_key(&signing_key, password)?;
        
        let mut encrypted_key = encrypted;
        encrypted_key.alias = alias;
        encrypted_key.public_key = verifying_key.to_bytes().to_vec();
        
        self.keys.push(encrypted_key);
        
        // Save to disk
        self.save()?;
        
        // Add to unlocked if keystore is unlocked
        if !self.locked {
            self.unlocked.push(signing_key);
        }
        
        Ok(verifying_key)
    }
    
    /// Import existing key
    pub fn import_key(&mut self, private_key_hex: &str, password: &str, alias: Option<String>) -> Result<VerifyingKey, WalletError> {
        let private_bytes = hex::decode(private_key_hex)?;
        
        if private_bytes.len() != 32 {
            return Err(WalletError::Other("Invalid private key length".to_string()));
        }
        
        let mut key_bytes = [0u8; 32];
        key_bytes.copy_from_slice(&private_bytes);
        
        let signing_key = SigningKey::from_bytes(&key_bytes);
        let verifying_key = signing_key.verifying_key();
        
        // Encrypt and store
        let mut encrypted = self.encrypt_key(&signing_key, password)?;
        encrypted.alias = alias;
        encrypted.public_key = verifying_key.to_bytes().to_vec();
        
        self.keys.push(encrypted);
        
        // Save to disk
        self.save()?;
        
        // Add to unlocked if keystore is unlocked
        if !self.locked {
            self.unlocked.push(signing_key);
        }
        
        Ok(verifying_key)
    }
    
    /// Unlock keystore with password
    pub fn unlock(&mut self, password: &str) -> Result<(), WalletError> {
        // Try to decrypt all keys
        let mut unlocked = Vec::new();
        
        for encrypted_key in &self.keys {
            let signing_key = self.decrypt_key(encrypted_key, password)?;
            unlocked.push(signing_key);
        }
        
        self.unlocked = unlocked;
        self.locked = false;
        
        Ok(())
    }
    
    /// Lock keystore
    pub fn lock(&mut self) {
        self.unlocked.clear();
        self.locked = true;
    }
    
    /// Get signing key by index
    pub fn get_signing_key(&self, index: usize) -> Result<&SigningKey, WalletError> {
        if self.locked {
            return Err(WalletError::WalletLocked);
        }
        
        self.unlocked.get(index)
            .ok_or_else(|| WalletError::AccountNotFound(format!("Index {}", index)))
    }
    
    /// Get signing key by public key
    pub fn get_signing_key_by_public(&self, public_key: &[u8]) -> Result<&SigningKey, WalletError> {
        if self.locked {
            return Err(WalletError::WalletLocked);
        }
        
        for (i, encrypted) in self.keys.iter().enumerate() {
            if encrypted.public_key == public_key {
                return self.get_signing_key(i);
            }
        }
        
        Err(WalletError::AccountNotFound("Public key not found".to_string()))
    }
    
    /// List all accounts
    pub fn list_accounts(&self) -> Vec<(usize, Vec<u8>, Option<String>)> {
        self.keys
            .iter()
            .enumerate()
            .map(|(i, k)| (i, k.public_key.clone(), k.alias.clone()))
            .collect()
    }
    
    /// Encrypt a signing key
    fn encrypt_key(&self, signing_key: &SigningKey, password: &str) -> Result<EncryptedKey, WalletError> {
        // Generate salt
        let salt = SaltString::generate(&mut OsRng);
        
        // Derive key from password
        let argon2 = Argon2::default();
        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| WalletError::Encryption(e.to_string()))?;
        
        // Get the hash bytes for AES key
        let hash_bytes = password_hash.hash.unwrap();
        let key_bytes = hash_bytes.as_bytes();
        
        // Ensure we have exactly 32 bytes for AES-256
        let mut aes_key = [0u8; 32];
        aes_key.copy_from_slice(&key_bytes[..32]);
        
        // Create cipher
        let key = Key::<Aes256Gcm>::from_slice(&aes_key);
        let cipher = Aes256Gcm::new(key);
        
        // Generate nonce
        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);
        
        // Encrypt private key
        let plaintext = signing_key.to_bytes();
        let ciphertext = cipher
            .encrypt(nonce, plaintext.as_ref())
            .map_err(|e| WalletError::Encryption(e.to_string()))?;
        
        Ok(EncryptedKey {
            ciphertext,
            salt: salt.to_string(),
            nonce: nonce_bytes.to_vec(),
            public_key: signing_key.verifying_key().to_bytes().to_vec(),
            alias: None,
        })
    }
    
    /// Decrypt an encrypted key
    fn decrypt_key(&self, encrypted: &EncryptedKey, password: &str) -> Result<SigningKey, WalletError> {
        // Parse salt
        let salt = SaltString::from_b64(&encrypted.salt)
            .map_err(|e| WalletError::Decryption(e.to_string()))?;
        
        // Derive key from password
        let argon2 = Argon2::default();
        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| WalletError::Decryption(e.to_string()))?;
        
        // Get the hash bytes for AES key
        let hash_bytes = password_hash.hash.unwrap();
        let key_bytes = hash_bytes.as_bytes();
        
        // Ensure we have exactly 32 bytes for AES-256
        let mut aes_key = [0u8; 32];
        aes_key.copy_from_slice(&key_bytes[..32]);
        
        // Create cipher
        let key = Key::<Aes256Gcm>::from_slice(&aes_key);
        let cipher = Aes256Gcm::new(key);
        
        // Decrypt
        let nonce = Nonce::from_slice(&encrypted.nonce);
        let plaintext = cipher
            .decrypt(nonce, encrypted.ciphertext.as_ref())
            .map_err(|_| WalletError::InvalidPassword)?;
        
        // Convert to signing key
        let mut key_bytes = [0u8; 32];
        key_bytes.copy_from_slice(&plaintext);
        
        Ok(SigningKey::from_bytes(&key_bytes))
    }
    
    /// Save keystore to disk
    fn save(&self) -> Result<(), WalletError> {
        let data = serde_json::to_vec_pretty(&self.keys)?;
        std::fs::write(&self.path, data)?;
        Ok(())
    }
    
    /// Export private key (requires unlock)
    pub fn export_private_key(&self, index: usize) -> Result<String, WalletError> {
        if self.locked {
            return Err(WalletError::WalletLocked);
        }
        
        let signing_key = self.get_signing_key(index)?;
        Ok(hex::encode(signing_key.to_bytes()))
    }
}
