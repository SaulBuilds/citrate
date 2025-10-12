// lattice-v3/core/storage/src/ipfs/encrypted_store.rs

//! Encrypted IPFS storage for secure model distribution
//!
//! This module integrates AES-256-GCM encryption with IPFS storage,
//! ensuring models are encrypted before being stored on the distributed network.

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;

use lattice_execution::crypto::encryption::{
    ModelEncryption, EncryptionConfig,
    EncryptionMetadata, EncryptedKey,
};
use lattice_execution::crypto::key_manager::{
    KeyManager, KeyPurpose,
};
use primitive_types::{H256, H160};

use super::{Cid, ModelMetadata, IPFSService};

/// Encrypted model manifest stored on IPFS
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedManifest {
    /// Model identifier
    pub model_id: H256,

    /// List of encrypted chunk CIDs
    pub encrypted_chunks: Vec<EncryptedChunk>,

    /// Encryption metadata
    pub encryption_metadata: EncryptionMetadata,

    /// Access control list
    pub access_list: Vec<H160>,

    /// Encrypted symmetric keys for authorized users
    pub encrypted_keys: Vec<EncryptedKey>,

    /// Original model metadata (public)
    pub model_metadata: ModelMetadata,

    /// Manifest version
    pub version: u32,
}

/// Encrypted chunk information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedChunk {
    /// IPFS CID of the encrypted chunk
    pub cid: Cid,

    /// Chunk index for ordering
    pub index: usize,

    /// Size of encrypted chunk
    pub size_bytes: u64,

    /// Nonce for this chunk (unique per chunk)
    pub nonce: [u8; 12],

    /// Authentication tag
    pub auth_tag: [u8; 16],
}

/// Configuration for encrypted IPFS storage
#[derive(Debug, Clone)]
pub struct EncryptedStorageConfig {
    /// Maximum chunk size (default: 256MB for Metal-optimized processing)
    pub chunk_size: usize,

    /// Enable compression before encryption
    pub compress: bool,

    /// Replication factor for important models
    pub replication_factor: u32,

    /// Auto-rotate keys after this many days
    pub key_rotation_days: u32,
}

impl Default for EncryptedStorageConfig {
    fn default() -> Self {
        Self {
            chunk_size: 256 * 1024 * 1024, // 256MB chunks
            compress: true,
            replication_factor: 3,
            key_rotation_days: 30,
        }
    }
}

/// Encrypted IPFS storage service
pub struct EncryptedIPFSStore {
    /// Underlying IPFS service
    ipfs: Arc<IPFSService>,

    /// Encryption module
    encryption: ModelEncryption,

    /// Key manager
    key_manager: Arc<KeyManager>,

    /// Configuration
    config: EncryptedStorageConfig,

    /// Cache of encrypted manifests
    manifest_cache: Arc<RwLock<HashMap<H256, EncryptedManifest>>>,
}

impl EncryptedIPFSStore {
    /// Create new encrypted IPFS store
    pub fn new(
        ipfs: Arc<IPFSService>,
        key_manager: Arc<KeyManager>,
        config: EncryptedStorageConfig,
    ) -> Self {
        let encryption_config = EncryptionConfig {
            use_hardware: cfg!(target_os = "macos"),
            key_rotation_enabled: true,
            max_key_age: config.key_rotation_days as u64 * 24 * 3600,
            compress: config.compress,
        };

        Self {
            ipfs,
            encryption: ModelEncryption::new(encryption_config),
            key_manager,
            config,
            manifest_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Store encrypted model on IPFS
    pub async fn store_encrypted_model(
        &self,
        model_id: H256,
        model_data: &[u8],
        metadata: ModelMetadata,
        owner: H160,
        access_list: Vec<H160>,
    ) -> Result<Cid> {
        // Derive encryption key for this model
        let key_path = format!("m/model/{}", hex::encode(model_id));
        let derived_key = self.key_manager.derive_key(
            &key_path,
            KeyPurpose::ModelEncryption,
        )?;

        // Chunk the model data
        let chunks = self.chunk_model_data(model_data)?;
        let mut encrypted_chunks = Vec::new();

        // Encrypt each chunk and upload to IPFS
        for (index, chunk) in chunks.iter().enumerate() {
            let encrypted_chunk = self.encrypt_chunk(
                chunk,
                &derived_key.key,
                index,
            )?;

            // Store size before moving data
            let data_len = encrypted_chunk.data.len() as u64;

            // Upload encrypted chunk to IPFS
            let cid = self.ipfs.add(encrypted_chunk.data).await?;

            encrypted_chunks.push(EncryptedChunk {
                cid: Cid(cid),
                index,
                size_bytes: data_len,
                nonce: encrypted_chunk.nonce,
                auth_tag: encrypted_chunk.auth_tag,
            });
        }

        // Create encrypted manifest
        let manifest = EncryptedManifest {
            model_id,
            encrypted_chunks,
            encryption_metadata: EncryptionMetadata {
                algorithm: "AES-256-GCM".to_string(),
                kdf: "PBKDF2".to_string(),
                encrypted_at: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                original_size: model_data.len(),
                plaintext_hash: self.calculate_hash(model_data),
                version: 1,
            },
            access_list: access_list.clone(),
            encrypted_keys: self.create_encrypted_keys(
                &derived_key.key,
                &access_list,
                owner,
            )?,
            model_metadata: metadata,
            version: 1,
        };

        // Store manifest on IPFS
        let manifest_data = serde_json::to_vec(&manifest)?;
        let manifest_cid = self.ipfs.add(manifest_data).await?;

        // Pin manifest and chunks for persistence
        self.ipfs.pin(&manifest_cid).await?;
        for chunk in &manifest.encrypted_chunks {
            self.ipfs.pin(&chunk.cid.0).await?;
        }

        // Cache manifest
        self.manifest_cache.write().insert(model_id, manifest);

        Ok(Cid(manifest_cid))
    }

    /// Retrieve and decrypt model from IPFS
    pub async fn retrieve_encrypted_model(
        &self,
        manifest_cid: &Cid,
        recipient: H160,
        recipient_key: &[u8; 32],
    ) -> Result<Vec<u8>> {
        // Fetch manifest from IPFS
        let manifest_data = self.ipfs.cat(&manifest_cid.0).await?;
        let manifest: EncryptedManifest = serde_json::from_slice(&manifest_data)?;

        // Check access permissions
        if !manifest.access_list.contains(&recipient) {
            return Err(anyhow!("Access denied: recipient not in access list"));
        }

        // Find encrypted key for recipient
        let encrypted_key = manifest.encrypted_keys
            .iter()
            .find(|k| k.recipient == recipient)
            .ok_or_else(|| anyhow!("No encrypted key found for recipient"))?;

        // Decrypt the symmetric key
        let symmetric_key = self.decrypt_key_for_recipient(
            &encrypted_key.encrypted_key,
            recipient_key,
        )?;

        // Download and decrypt all chunks
        let mut decrypted_data = Vec::new();
        for chunk_info in &manifest.encrypted_chunks {
            // Fetch encrypted chunk from IPFS
            let encrypted_chunk = self.ipfs.cat(&chunk_info.cid.0).await?;

            // Decrypt chunk
            let decrypted_chunk = self.decrypt_chunk(
                &encrypted_chunk,
                &symmetric_key,
                &chunk_info.nonce,
                &chunk_info.auth_tag,
            )?;

            decrypted_data.extend(decrypted_chunk);
        }

        // Verify integrity
        let calculated_hash = self.calculate_hash(&decrypted_data);
        if calculated_hash != manifest.encryption_metadata.plaintext_hash {
            return Err(anyhow!("Integrity check failed: hash mismatch"));
        }

        Ok(decrypted_data)
    }

    /// Grant access to additional user
    pub async fn grant_access(
        &self,
        manifest_cid: &Cid,
        new_user: H160,
        owner: H160,
        owner_key: &[u8; 32],
    ) -> Result<Cid> {
        // Fetch current manifest
        let manifest_data = self.ipfs.cat(&manifest_cid.0).await?;
        let mut manifest: EncryptedManifest = serde_json::from_slice(&manifest_data)?;

        // Verify owner
        if !manifest.access_list.contains(&owner) {
            return Err(anyhow!("Only authorized users can grant access"));
        }

        // Check if already has access
        if manifest.access_list.contains(&new_user) {
            return Ok(manifest_cid.clone());
        }

        // Decrypt the symmetric key using owner's key
        let owner_encrypted_key = manifest.encrypted_keys
            .iter()
            .find(|k| k.recipient == owner)
            .ok_or_else(|| anyhow!("Owner key not found"))?;

        let symmetric_key = self.decrypt_key_for_recipient(
            &owner_encrypted_key.encrypted_key,
            owner_key,
        )?;

        // Create encrypted key for new user
        let new_encrypted_key = self.encrypt_key_for_recipient(
            &symmetric_key,
            &new_user,
        )?;

        // Update manifest
        manifest.access_list.push(new_user);
        manifest.encrypted_keys.push(new_encrypted_key);

        // Upload updated manifest
        let updated_manifest = serde_json::to_vec(&manifest)?;
        let new_manifest_cid = self.ipfs.add(updated_manifest).await?;

        // Pin new manifest
        self.ipfs.pin(&new_manifest_cid).await?;

        // Update cache
        self.manifest_cache.write().insert(manifest.model_id, manifest);

        Ok(Cid(new_manifest_cid))
    }

    /// Chunk model data for efficient storage
    fn chunk_model_data(&self, data: &[u8]) -> Result<Vec<Vec<u8>>> {
        let chunk_size = self.config.chunk_size;
        let mut chunks = Vec::new();

        for chunk in data.chunks(chunk_size) {
            chunks.push(chunk.to_vec());
        }

        Ok(chunks)
    }

    /// Encrypt a single chunk
    fn encrypt_chunk(
        &self,
        chunk: &[u8],
        key: &[u8; 32],
        index: usize,
    ) -> Result<EncryptedChunkData> {
        // Compress if configured
        let data_to_encrypt = if self.config.compress {
            self.compress_data(chunk)?
        } else {
            chunk.to_vec()
        };

        // Use the encryption module for actual encryption
        let encrypted_result = self.encryption.encrypt_chunk(&data_to_encrypt, key, index)?;

        Ok(EncryptedChunkData {
            data: encrypted_result.data,
            nonce: encrypted_result.nonce,
            auth_tag: encrypted_result.auth_tag,
        })
    }

    /// Decrypt a chunk
    fn decrypt_chunk(
        &self,
        encrypted_data: &[u8],
        key: &[u8; 32],
        nonce: &[u8; 12],
        auth_tag: &[u8; 16],
    ) -> Result<Vec<u8>> {
        // Use the encryption module for actual decryption
        let decrypted = self.encryption.decrypt_chunk(encrypted_data, key, nonce, auth_tag)?;

        // Decompress if needed
        if self.config.compress {
            self.decompress_data(&decrypted)
        } else {
            Ok(decrypted)
        }
    }

    /// Create encrypted keys for all recipients
    fn create_encrypted_keys(
        &self,
        symmetric_key: &[u8; 32],
        access_list: &[H160],
        owner: H160,
    ) -> Result<Vec<EncryptedKey>> {
        let mut encrypted_keys = Vec::new();

        // Add owner if not in list
        let mut full_list = access_list.to_vec();
        if !full_list.contains(&owner) {
            full_list.push(owner);
        }

        for recipient in &full_list {
            let encrypted_key = self.encrypt_key_for_recipient(
                symmetric_key,
                recipient,
            )?;
            encrypted_keys.push(encrypted_key);
        }

        Ok(encrypted_keys)
    }

    /// Encrypt symmetric key for recipient
    fn encrypt_key_for_recipient(
        &self,
        symmetric_key: &[u8; 32],
        recipient: &H160,
    ) -> Result<EncryptedKey> {
        // In production, use recipient's public key with ECIES
        // For now, simplified encryption
        use rand::RngCore;
        use aes_gcm::aead::OsRng;

        let mut ephemeral_key = [0u8; 32];
        OsRng.fill_bytes(&mut ephemeral_key);

        let mut encrypted = symmetric_key.to_vec();
        for i in 0..32 {
            encrypted[i] ^= ephemeral_key[i] ^ recipient.as_bytes()[i % 20];
        }

        Ok(EncryptedKey {
            recipient: *recipient,
            encrypted_key: encrypted,
            ephemeral_pubkey: vec![0u8; 33], // Would be actual public key
        })
    }

    /// Decrypt symmetric key for recipient
    fn decrypt_key_for_recipient(
        &self,
        encrypted_key: &[u8],
        _recipient_key: &[u8; 32],
    ) -> Result<[u8; 32]> {
        // Simplified decryption (production would use ECIES)
        if encrypted_key.len() != 32 {
            return Err(anyhow!("Invalid encrypted key length"));
        }

        let mut decrypted = [0u8; 32];
        decrypted.copy_from_slice(encrypted_key);

        // In production, perform proper ECDH and derive shared secret
        Ok(decrypted)
    }

    /// Calculate hash for integrity verification
    fn calculate_hash(&self, data: &[u8]) -> H256 {
        use sha3::{Sha3_256, Digest};
        let mut hasher = Sha3_256::new();
        hasher.update(data);
        H256::from_slice(hasher.finalize().as_slice())
    }

    /// Compress data using zstd
    fn compress_data(&self, data: &[u8]) -> Result<Vec<u8>> {
        // In production, use zstd::encode_all
        // For now, return as-is
        Ok(data.to_vec())
    }

    /// Decompress data
    fn decompress_data(&self, data: &[u8]) -> Result<Vec<u8>> {
        // In production, use zstd::decode_all
        Ok(data.to_vec())
    }
}

/// Encrypted chunk data structure
struct EncryptedChunkData {
    data: Vec<u8>,
    nonce: [u8; 12],
    auth_tag: [u8; 16],
}

/// Async trait implementation for IPFS operations
#[async_trait]
impl IPFSOperations for IPFSService {
    async fn add(&self, data: Vec<u8>) -> Result<String> {
        // Implementation would call IPFS HTTP API
        // POST /api/v0/add
        Ok(format!("Qm{}", hex::encode(&data[..16])))
    }

    async fn cat(&self, _cid: &str) -> Result<Vec<u8>> {
        // Implementation would call IPFS HTTP API
        // GET /api/v0/cat?arg={cid}
        Ok(Vec::new())
    }

    async fn pin(&self, _cid: &str) -> Result<()> {
        // Implementation would call IPFS HTTP API
        // POST /api/v0/pin/add?arg={cid}
        Ok(())
    }
}

#[async_trait]
trait IPFSOperations {
    async fn add(&self, data: Vec<u8>) -> Result<String>;
    async fn cat(&self, cid: &str) -> Result<Vec<u8>>;
    async fn pin(&self, cid: &str) -> Result<()>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_encrypted_storage() {
        // Create services
        let ipfs = Arc::new(IPFSService::new("http://localhost:5001".to_string()));
        let key_manager = Arc::new(KeyManager::from_seed(&[0u8; 64]).unwrap());
        let config = EncryptedStorageConfig::default();

        let store = EncryptedIPFSStore::new(ipfs, key_manager, config);

        // Test data
        let model_data = b"test model weights";
        let model_id = H256::random();
        let owner = H160::random();
        let user = H160::random();

        let metadata = ModelMetadata {
            name: "test_model".to_string(),
            version: "1.0".to_string(),
            framework: ModelFramework::PyTorch,
            model_type: ModelType::Language,
            size_bytes: model_data.len() as u64,
            input_shape: vec![1, 512],
            output_shape: vec![1, 1000],
            description: "Test model".to_string(),
            author: "test".to_string(),
            license: "MIT".to_string(),
            created_at: 0,
        };

        // Store encrypted model
        let manifest_cid = store.store_encrypted_model(
            model_id,
            model_data,
            metadata,
            owner,
            vec![user],
        ).await.unwrap();

        assert!(!manifest_cid.0.is_empty());
    }

    #[test]
    fn test_chunking() {
        let key_manager = Arc::new(KeyManager::from_seed(&[0u8; 64]).unwrap());
        let config = EncryptedStorageConfig {
            chunk_size: 10, // Small chunks for testing
            ..Default::default()
        };

        let ipfs = Arc::new(IPFSService::new("http://localhost:5001".to_string()));
        let store = EncryptedIPFSStore::new(ipfs, key_manager, config);

        let data = b"This is a test of chunking functionality";
        let chunks = store.chunk_model_data(data).unwrap();

        assert_eq!(chunks.len(), 5); // 41 bytes / 10 = 5 chunks
        assert_eq!(chunks[0].len(), 10);
        assert_eq!(chunks[4].len(), 1); // Last chunk has 1 byte
    }
}