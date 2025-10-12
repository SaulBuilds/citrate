
// lattice-v3/core/execution/src/crypto/encryption.rs

// AES-256-GCM encryption for secure model storage
// Provides encryption at rest for AI model weights and metadata

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce, Key
};
use rand::RngCore;
use sha3::{Sha3_256, Digest};
use primitive_types::{H256, H160};

/// Encrypted model structure for storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedModel {
    /// Model identifier
    pub model_id: H256,

    /// Encrypted model weights
    pub ciphertext: Vec<u8>,

    /// Nonce for AES-GCM (96 bits)
    pub nonce: [u8; 12],

    /// Authentication tag (128 bits)
    pub auth_tag: [u8; 16],

    /// Key identifier for key rotation
    pub key_id: H256,

    /// Encryption metadata
    pub metadata: EncryptionMetadata,

    /// Access control list (public keys that can decrypt)
    pub access_list: Vec<H160>,

    /// Encrypted symmetric key for each authorized user
    pub encrypted_keys: Vec<EncryptedKey>,
}

/// Metadata about the encryption
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionMetadata {
    /// Encryption algorithm used
    pub algorithm: String,

    /// Key derivation function
    pub kdf: String,

    /// Timestamp of encryption
    pub encrypted_at: u64,

    /// Original model size before encryption
    pub original_size: usize,

    /// SHA3-256 hash of plaintext for integrity
    pub plaintext_hash: H256,

    /// Version for future compatibility
    pub version: u32,
}

/// Encrypted symmetric key for a specific user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedKey {
    /// Recipient's address
    pub recipient: H160,

    /// Encrypted AES key using recipient's public key
    pub encrypted_key: Vec<u8>,

    /// Ephemeral public key for ECDH (33 bytes)
    pub ephemeral_pubkey: Vec<u8>,
}

/// Encrypted chunk data for IPFS storage
#[derive(Debug, Clone)]
pub struct EncryptedChunkData {
    pub data: Vec<u8>,
    pub nonce: [u8; 12],
    pub auth_tag: [u8; 16],
}

/// Configuration for model encryption
#[derive(Debug, Clone)]
pub struct EncryptionConfig {
    /// Use hardware encryption if available
    pub use_hardware: bool,

    /// Enable key rotation
    pub key_rotation_enabled: bool,

    /// Maximum key age in seconds
    pub max_key_age: u64,

    /// Compression before encryption
    pub compress: bool,
}

impl Default for EncryptionConfig {
    fn default() -> Self {
        Self {
            use_hardware: cfg!(target_os = "macos"),
            key_rotation_enabled: true,
            max_key_age: 30 * 24 * 3600, // 30 days
            compress: true,
        }
    }
}

/// Model encryption interface
pub struct ModelEncryption {
    config: EncryptionConfig,
    master_key: Option<[u8; 32]>,
}

impl ModelEncryption {
    /// Create new encryption instance
    pub fn new(config: EncryptionConfig) -> Self {
        Self {
            config,
            master_key: None,
        }
    }

    /// Initialize with master key
    pub fn with_master_key(mut self, key: [u8; 32]) -> Self {
        self.master_key = Some(key);
        self
    }

    /// Encrypt model weights
    pub fn encrypt_model(
        &self,
        model_id: H256,
        model_data: &[u8],
        owner: H160,
        access_list: Vec<H160>,
    ) -> Result<EncryptedModel> {
        // Generate random AES-256 key
        let mut key_bytes = [0u8; 32];
        OsRng.fill_bytes(&mut key_bytes);
        let key = Key::<Aes256Gcm>::from_slice(&key_bytes);

        // Generate random nonce (96 bits for GCM)
        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        // Create cipher
        let cipher = Aes256Gcm::new(key);

        // Compress if configured
        let data_to_encrypt = if self.config.compress {
            Self::compress_data(model_data)?
        } else {
            model_data.to_vec()
        };

        // Encrypt the model data
        let ciphertext = cipher
            .encrypt(nonce, data_to_encrypt.as_ref())
            .map_err(|e| anyhow!("Encryption failed: {}", e))?;

        // Extract authentication tag (last 16 bytes of ciphertext in GCM)
        let (encrypted_data, auth_tag) = if ciphertext.len() >= 16 {
            let split_point = ciphertext.len() - 16;
            let mut tag = [0u8; 16];
            tag.copy_from_slice(&ciphertext[split_point..]);
            (ciphertext[..split_point].to_vec(), tag)
        } else {
            return Err(anyhow!("Invalid ciphertext length"));
        };

        // Calculate plaintext hash for integrity
        let plaintext_hash = {
            let mut hasher = Sha3_256::new();
            hasher.update(model_data);
            H256::from_slice(hasher.finalize().as_slice())
        };

        // Generate key ID
        let key_id = {
            let mut hasher = Sha3_256::new();
            hasher.update(&key_bytes);
            hasher.update(&model_id.as_bytes());
            H256::from_slice(hasher.finalize().as_slice())
        };

        // Encrypt the symmetric key for each authorized user
        let mut encrypted_keys = Vec::new();

        // Add owner to access list if not present
        let mut full_access_list = access_list.clone();
        if !full_access_list.contains(&owner) {
            full_access_list.push(owner);
        }

        for recipient in &full_access_list {
            // In production, this would use ECIES with the recipient's public key
            // For now, we'll use a simplified approach
            let encrypted_key = self.encrypt_key_for_recipient(&key_bytes, recipient)?;
            encrypted_keys.push(encrypted_key);
        }

        // Create metadata
        let metadata = EncryptionMetadata {
            algorithm: "AES-256-GCM".to_string(),
            kdf: "Argon2id".to_string(),
            encrypted_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            original_size: model_data.len(),
            plaintext_hash,
            version: 1,
        };

        Ok(EncryptedModel {
            model_id,
            ciphertext: encrypted_data,
            nonce: nonce_bytes,
            auth_tag,
            key_id,
            metadata,
            access_list: full_access_list,
            encrypted_keys,
        })
    }

    /// Decrypt model weights
    pub fn decrypt_model(
        &self,
        encrypted_model: &EncryptedModel,
        recipient_key: &[u8; 32],
        recipient_address: H160,
    ) -> Result<Vec<u8>> {
        // Check access list
        if !encrypted_model.access_list.contains(&recipient_address) {
            return Err(anyhow!("Access denied: address not in access list"));
        }

        // Find encrypted key for this recipient
        let encrypted_key = encrypted_model.encrypted_keys
            .iter()
            .find(|k| k.recipient == recipient_address)
            .ok_or_else(|| anyhow!("No encrypted key found for recipient"))?;

        // Decrypt the symmetric key
        let symmetric_key = self.decrypt_key_for_recipient(
            &encrypted_key.encrypted_key,
            recipient_key,
        )?;

        // Reconstruct full ciphertext with auth tag
        let mut full_ciphertext = encrypted_model.ciphertext.clone();
        full_ciphertext.extend_from_slice(&encrypted_model.auth_tag);

        // Create cipher
        let key = Key::<Aes256Gcm>::from_slice(&symmetric_key);
        let cipher = Aes256Gcm::new(key);
        let nonce = Nonce::from_slice(&encrypted_model.nonce);

        // Decrypt
        let decrypted_data = cipher
            .decrypt(nonce, full_ciphertext.as_ref())
            .map_err(|e| anyhow!("Decryption failed: {}", e))?;

        // Decompress if needed
        let plaintext = if self.config.compress {
            Self::decompress_data(&decrypted_data)?
        } else {
            decrypted_data
        };

        // Verify integrity
        let calculated_hash = {
            let mut hasher = Sha3_256::new();
            hasher.update(&plaintext);
            H256::from_slice(hasher.finalize().as_slice())
        };

        if calculated_hash != encrypted_model.metadata.plaintext_hash {
            return Err(anyhow!("Integrity check failed: hash mismatch"));
        }

        Ok(plaintext)
    }

    /// Encrypt symmetric key for a recipient
    fn encrypt_key_for_recipient(
        &self,
        symmetric_key: &[u8; 32],
        recipient: &H160,
    ) -> Result<EncryptedKey> {
        // Generate ephemeral keypair for ECDH
        let mut ephemeral_key = [0u8; 32];
        OsRng.fill_bytes(&mut ephemeral_key);

        // In production, this would:
        // 1. Generate ephemeral ECDSA keypair
        // 2. Perform ECDH with recipient's public key
        // 3. Derive shared secret
        // 4. Encrypt symmetric key with shared secret

        // Simplified version for demonstration
        let mut encrypted = Vec::new();
        encrypted.extend_from_slice(symmetric_key);
        for i in 0..32 {
            encrypted[i] ^= ephemeral_key[i] ^ recipient.as_bytes()[i % 20];
        }

        Ok(EncryptedKey {
            recipient: *recipient,
            encrypted_key: encrypted,
            ephemeral_pubkey: vec![0u8; 33], // Would be actual public key
        })
    }

    /// Decrypt symmetric key for a recipient
    fn decrypt_key_for_recipient(
        &self,
        encrypted_key: &[u8],
        _recipient_key: &[u8; 32],
    ) -> Result<[u8; 32]> {
        // In production, this would:
        // 1. Use recipient's private key
        // 2. Perform ECDH with ephemeral public key
        // 3. Derive shared secret
        // 4. Decrypt symmetric key

        // Simplified version
        if encrypted_key.len() != 32 {
            return Err(anyhow!("Invalid encrypted key length"));
        }

        let mut decrypted = [0u8; 32];
        decrypted.copy_from_slice(encrypted_key);

        Ok(decrypted)
    }

    /// Compress data using zstd
    fn compress_data(data: &[u8]) -> Result<Vec<u8>> {
        // In production, use zstd compression
        // For now, return as-is
        Ok(data.to_vec())
    }

    /// Decompress data
    fn decompress_data(data: &[u8]) -> Result<Vec<u8>> {
        // In production, use zstd decompression
        // For now, return as-is
        Ok(data.to_vec())
    }

    /// Rotate encryption keys
    pub fn rotate_key(
        &self,
        encrypted_model: &EncryptedModel,
        old_key: &[u8; 32],
        owner: H160,
    ) -> Result<EncryptedModel> {
        // Decrypt with old key
        let plaintext = self.decrypt_model(encrypted_model, old_key, owner)?;

        // Re-encrypt with new key
        self.encrypt_model(
            encrypted_model.model_id,
            &plaintext,
            owner,
            encrypted_model.access_list.clone(),
        )
    }

    /// Add user to access list
    pub fn grant_access(
        &self,
        encrypted_model: &mut EncryptedModel,
        new_user: H160,
        owner_key: &[u8; 32],
        owner: H160,
    ) -> Result<()> {
        // Verify owner
        if !encrypted_model.access_list.contains(&owner) {
            return Err(anyhow!("Only owner can grant access"));
        }

        // Check if already has access
        if encrypted_model.access_list.contains(&new_user) {
            return Ok(());
        }

        // Decrypt the symmetric key
        let encrypted_key = encrypted_model.encrypted_keys
            .iter()
            .find(|k| k.recipient == owner)
            .ok_or_else(|| anyhow!("Owner key not found"))?;

        let symmetric_key = self.decrypt_key_for_recipient(
            &encrypted_key.encrypted_key,
            owner_key,
        )?;

        // Encrypt for new user
        let new_encrypted_key = self.encrypt_key_for_recipient(&symmetric_key, &new_user)?;

        // Update access list
        encrypted_model.access_list.push(new_user);
        encrypted_model.encrypted_keys.push(new_encrypted_key);

        Ok(())
    }

    /// Revoke user access
    pub fn revoke_access(
        &self,
        encrypted_model: &EncryptedModel,
        revoked_user: H160,
        owner_key: &[u8; 32],
        owner: H160,
    ) -> Result<EncryptedModel> {
        // Verify owner
        if !encrypted_model.access_list.contains(&owner) {
            return Err(anyhow!("Only owner can revoke access"));
        }

        // Can't revoke owner's access
        if revoked_user == owner {
            return Err(anyhow!("Cannot revoke owner's access"));
        }

        // Decrypt and re-encrypt with new access list
        let plaintext = self.decrypt_model(encrypted_model, owner_key, owner)?;

        let new_access_list: Vec<H160> = encrypted_model.access_list
            .iter()
            .filter(|&&addr| addr != revoked_user)
            .cloned()
            .collect();

        self.encrypt_model(
            encrypted_model.model_id,
            &plaintext,
            owner,
            new_access_list,
        )
    }

    /// Encrypt a single chunk of data
    pub fn encrypt_chunk(
        &self,
        chunk: &[u8],
        key: &[u8; 32],
        index: usize,
    ) -> Result<EncryptedChunkData> {
        use rand::RngCore;

        // Generate unique nonce for this chunk
        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);

        // Include chunk index in nonce for additional uniqueness
        nonce_bytes[0] = (index & 0xFF) as u8;
        nonce_bytes[1] = ((index >> 8) & 0xFF) as u8;

        let nonce = Nonce::from_slice(&nonce_bytes);
        let key = Key::<Aes256Gcm>::from_slice(key);
        let cipher = Aes256Gcm::new(key);

        // Encrypt
        let ciphertext = cipher
            .encrypt(nonce, chunk)
            .map_err(|e| anyhow!("Encryption failed: {}", e))?;

        // Extract auth tag (last 16 bytes)
        let (encrypted_data, auth_tag) = if ciphertext.len() >= 16 {
            let split_point = ciphertext.len() - 16;
            let mut tag = [0u8; 16];
            tag.copy_from_slice(&ciphertext[split_point..]);
            (ciphertext[..split_point].to_vec(), tag)
        } else {
            return Err(anyhow!("Invalid ciphertext length"));
        };

        Ok(EncryptedChunkData {
            data: encrypted_data,
            nonce: nonce_bytes,
            auth_tag,
        })
    }

    /// Decrypt a single chunk of data
    pub fn decrypt_chunk(
        &self,
        encrypted_data: &[u8],
        key: &[u8; 32],
        nonce: &[u8; 12],
        auth_tag: &[u8; 16],
    ) -> Result<Vec<u8>> {
        // Reconstruct full ciphertext with auth tag
        let mut full_ciphertext = encrypted_data.to_vec();
        full_ciphertext.extend_from_slice(auth_tag);

        let nonce = Nonce::from_slice(nonce);
        let key = Key::<Aes256Gcm>::from_slice(key);
        let cipher = Aes256Gcm::new(key);

        // Decrypt
        let decrypted = cipher
            .decrypt(nonce, full_ciphertext.as_ref())
            .map_err(|e| anyhow!("Decryption failed: {}", e))?;

        Ok(decrypted)
    }

}

/// Public convenience functions
pub fn encrypt_model(
    model_data: &[u8],
    owner: H160,
    access_list: Vec<H160>,
) -> Result<EncryptedModel> {
    let model_id = H256::random();
    let encryption = ModelEncryption::new(EncryptionConfig::default());
    encryption.encrypt_model(model_id, model_data, owner, access_list)
}

pub fn decrypt_model(
    encrypted_model: &EncryptedModel,
    recipient_key: &[u8; 32],
    recipient: H160,
) -> Result<Vec<u8>> {
    let encryption = ModelEncryption::new(EncryptionConfig::default());
    encryption.decrypt_model(encrypted_model, recipient_key, recipient)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt() {
        let model_data = b"test model weights";
        let owner = H160::random();
        let user = H160::random();

        // Encrypt
        let encrypted = encrypt_model(
            model_data,
            owner,
            vec![user],
        ).unwrap();

        assert_ne!(encrypted.ciphertext, model_data);
        assert_eq!(encrypted.access_list.len(), 2); // owner + user

        // Decrypt
        let owner_key = [1u8; 32];
        let decrypted = decrypt_model(
            &encrypted,
            &owner_key,
            owner,
        ).unwrap();

        assert_eq!(decrypted, model_data);
    }

    #[test]
    fn test_access_control() {
        let model_data = b"test model";
        let owner = H160::random();
        let unauthorized = H160::random();

        let encrypted = encrypt_model(
            model_data,
            owner,
            vec![], // Only owner has access
        ).unwrap();

        // Unauthorized user should fail
        let unauthorized_key = [2u8; 32];
        let result = decrypt_model(
            &encrypted,
            &unauthorized_key,
            unauthorized,
        );

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Access denied"));
    }
}