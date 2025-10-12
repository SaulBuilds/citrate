// lattice-v3/core/execution/src/crypto/key_manager.rs

// Hierarchical Deterministic (HD) Key Management System
// Manages encryption keys for models with support for key derivation and rotation

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use sha3::{Sha3_256, Sha3_512, Digest};
use primitive_types::{H256, H160};
use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use hmac::{Hmac, Mac};

type HmacSha512 = Hmac<Sha3_512>;

/// Derived key for a specific model
#[derive(Debug, Clone)]
pub struct DerivedKey {
    /// Key identifier
    pub key_id: H256,

    /// The derived key material
    pub key: [u8; 32],

    /// Derivation path
    pub path: String,

    /// Creation timestamp
    pub created_at: u64,

    /// Expiry timestamp (0 = no expiry)
    pub expires_at: u64,

    /// Key purpose
    pub purpose: KeyPurpose,
}

/// Purpose of a derived key
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum KeyPurpose {
    /// Encryption/decryption of model weights
    ModelEncryption,

    /// Signing model metadata
    ModelSigning,

    /// Inference proof generation
    ProofGeneration,

    /// Access token generation
    AccessToken,
}

/// Threshold key for multi-party models
#[derive(Debug, Clone)]
pub struct ThresholdKey {
    /// Key identifier
    pub key_id: H256,

    /// Threshold (k of n)
    pub threshold: u32,

    /// Total shares
    pub total_shares: u32,

    /// Share holders
    pub share_holders: Vec<H160>,

    /// Encrypted shares
    pub encrypted_shares: HashMap<H160, Vec<u8>>,
}

/// Access policy for models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessPolicy {
    /// Owner of the model
    pub owner: H160,

    /// Addresses with full access
    pub full_access: Vec<H160>,

    /// Addresses with inference-only access
    pub inference_only: Vec<H160>,

    /// Addresses with time-limited access
    pub time_limited: HashMap<H160, TimeLimitedAccess>,

    /// Whether model requires payment
    pub requires_payment: bool,

    /// Minimum stake required for access
    pub min_stake: Option<u64>,
}

/// Time-limited access configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeLimitedAccess {
    /// Start timestamp
    pub valid_from: u64,

    /// End timestamp
    pub valid_until: u64,

    /// Maximum number of uses
    pub max_uses: Option<u32>,

    /// Current uses
    pub current_uses: u32,
}

/// Key management system
pub struct KeyManager {
    /// Master key (never stored, only in memory)
    master_key: Option<[u8; 32]>,

    /// Chain code for HD derivation
    chain_code: Option<[u8; 32]>,

    /// Derived keys cache
    derived_keys: Arc<RwLock<HashMap<H256, DerivedKey>>>,

    /// Threshold keys
    threshold_keys: Arc<RwLock<HashMap<H256, ThresholdKey>>>,

    /// Access policies
    access_policies: Arc<RwLock<HashMap<H256, AccessPolicy>>>,

    /// Key rotation schedule
    rotation_schedule: Arc<RwLock<HashMap<H256, u64>>>,
}

impl KeyManager {
    /// Create new key manager
    pub fn new() -> Self {
        Self {
            master_key: None,
            chain_code: None,
            derived_keys: Arc::new(RwLock::new(HashMap::new())),
            threshold_keys: Arc::new(RwLock::new(HashMap::new())),
            access_policies: Arc::new(RwLock::new(HashMap::new())),
            rotation_schedule: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Initialize from seed
    pub fn from_seed(seed: &[u8]) -> Result<Self> {
        if seed.len() < 32 {
            return Err(anyhow!("Seed must be at least 32 bytes"));
        }

        // Derive master key and chain code using HMAC-SHA512
        let mut mac = HmacSha512::new_from_slice(b"Lattice seed")?;
        mac.update(seed);
        let result = mac.finalize();
        let bytes = result.into_bytes();

        let mut master_key = [0u8; 32];
        let mut chain_code = [0u8; 32];

        master_key.copy_from_slice(&bytes[..32]);
        chain_code.copy_from_slice(&bytes[32..64]);

        let mut manager = Self::new();
        manager.master_key = Some(master_key);
        manager.chain_code = Some(chain_code);

        Ok(manager)
    }

    /// Derive key for a specific path
    pub fn derive_key(
        &self,
        path: &str,
        purpose: KeyPurpose,
    ) -> Result<DerivedKey> {
        let master_key = self.master_key
            .ok_or_else(|| anyhow!("Master key not initialized"))?;
        let chain_code = self.chain_code
            .ok_or_else(|| anyhow!("Chain code not initialized"))?;

        // Parse path (e.g., "m/44'/60'/0'/0/0")
        let components: Vec<&str> = path.split('/').collect();
        if components.is_empty() || components[0] != "m" {
            return Err(anyhow!("Invalid derivation path"));
        }

        let mut current_key = master_key;
        let mut current_chain = chain_code;

        // Derive for each path component
        for component in components.iter().skip(1) {
            let (index, hardened) = if component.ends_with('\'') {
                let idx_str = &component[..component.len() - 1];
                let idx = idx_str.parse::<u32>()
                    .map_err(|_| anyhow!("Invalid path component"))?;
                (idx + 0x80000000, true) // Hardened derivation
            } else {
                let idx = component.parse::<u32>()
                    .map_err(|_| anyhow!("Invalid path component"))?;
                (idx, false)
            };

            // Perform child key derivation
            let (child_key, child_chain) = self.derive_child_key(
                &current_key,
                &current_chain,
                index,
            )?;

            current_key = child_key;
            current_chain = child_chain;
        }

        // Generate key ID
        let key_id = {
            let mut hasher = Sha3_256::new();
            hasher.update(&current_key);
            hasher.update(path.as_bytes());
            H256::from_slice(hasher.finalize().as_slice())
        };

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let derived_key = DerivedKey {
            key_id,
            key: current_key,
            path: path.to_string(),
            created_at: now,
            expires_at: now + 30 * 24 * 3600, // 30 days default
            purpose,
        };

        // Cache the derived key
        self.derived_keys.write().insert(key_id, derived_key.clone());

        Ok(derived_key)
    }

    /// Derive child key using BIP32-like derivation
    fn derive_child_key(
        &self,
        parent_key: &[u8; 32],
        parent_chain: &[u8; 32],
        index: u32,
    ) -> Result<([u8; 32], [u8; 32])> {
        let mut mac = HmacSha512::new_from_slice(parent_chain)?;

        if index >= 0x80000000 {
            // Hardened derivation
            mac.update(&[0x00]);
            mac.update(parent_key);
        } else {
            // Non-hardened derivation (would use public key in real implementation)
            mac.update(parent_key);
        }

        mac.update(&index.to_be_bytes());

        let result = mac.finalize();
        let bytes = result.into_bytes();

        let mut child_key = [0u8; 32];
        let mut child_chain = [0u8; 32];

        child_key.copy_from_slice(&bytes[..32]);
        child_chain.copy_from_slice(&bytes[32..64]);

        Ok((child_key, child_chain))
    }

    /// Create threshold key for multi-party control
    pub fn create_threshold_key(
        &self,
        model_id: H256,
        threshold: u32,
        share_holders: Vec<H160>,
    ) -> Result<ThresholdKey> {
        let total_shares = share_holders.len() as u32;

        if threshold > total_shares {
            return Err(anyhow!("Threshold cannot exceed total shares"));
        }

        if threshold == 0 {
            return Err(anyhow!("Threshold must be at least 1"));
        }

        // Generate secret to share
        let secret = self.derive_key(
            &format!("m/threshold/{}", hex::encode(model_id)),
            KeyPurpose::ModelEncryption,
        )?;

        // Use Shamir's Secret Sharing to split the secret
        let shares = self.split_secret(&secret.key, threshold, total_shares)?;

        // Encrypt each share for its holder
        let mut encrypted_shares = HashMap::new();
        for (i, holder) in share_holders.iter().enumerate() {
            // In production, encrypt with holder's public key
            let encrypted_share = self.encrypt_share(&shares[i], holder)?;
            encrypted_shares.insert(*holder, encrypted_share);
        }

        let threshold_key = ThresholdKey {
            key_id: model_id,
            threshold,
            total_shares,
            share_holders,
            encrypted_shares,
        };

        self.threshold_keys.write().insert(model_id, threshold_key.clone());

        Ok(threshold_key)
    }

    /// Split secret using Shamir's Secret Sharing (simplified)
    fn split_secret(
        &self,
        secret: &[u8; 32],
        threshold: u32,
        total: u32,
    ) -> Result<Vec<Vec<u8>>> {
        // Simplified implementation
        // In production, use proper Shamir's Secret Sharing

        let mut shares = Vec::new();
        for i in 0..total {
            let mut share = Vec::new();
            share.push(i as u8);
            share.push(threshold as u8);
            share.extend_from_slice(secret);

            // XOR with index for differentiation (not secure, just demo)
            for byte in share.iter_mut().skip(2) {
                *byte ^= i as u8;
            }

            shares.push(share);
        }

        Ok(shares)
    }

    /// Encrypt share for a holder
    fn encrypt_share(&self, share: &[u8], holder: &H160) -> Result<Vec<u8>> {
        // In production, use holder's public key for encryption
        // Simplified version for demonstration
        let mut encrypted = share.to_vec();
        for (i, byte) in encrypted.iter_mut().enumerate() {
            *byte ^= holder.as_bytes()[i % 20];
        }
        Ok(encrypted)
    }

    /// Reconstruct secret from threshold shares
    pub fn reconstruct_secret(
        &self,
        shares: Vec<(H160, Vec<u8>)>,
        threshold_key: &ThresholdKey,
    ) -> Result<[u8; 32]> {
        if shares.len() < threshold_key.threshold as usize {
            return Err(anyhow!("Insufficient shares for reconstruction"));
        }

        // Verify all shares are from valid holders
        for (holder, _) in &shares {
            if !threshold_key.share_holders.contains(holder) {
                return Err(anyhow!("Invalid share holder"));
            }
        }

        // Decrypt shares
        let mut decrypted_shares = Vec::new();
        for (holder, encrypted_share) in shares {
            let share = self.decrypt_share(&encrypted_share, &holder)?;
            decrypted_shares.push(share);
        }

        // Reconstruct secret (simplified)
        // In production, use proper Shamir's reconstruction
        let mut secret = [0u8; 32];
        if !decrypted_shares.is_empty() {
            let first_share = &decrypted_shares[0];
            if first_share.len() >= 34 {
                secret.copy_from_slice(&first_share[2..34]);
                // XOR back to get original
                let index = first_share[0];
                for byte in secret.iter_mut() {
                    *byte ^= index;
                }
            }
        }

        Ok(secret)
    }

    /// Decrypt share
    fn decrypt_share(&self, encrypted: &[u8], holder: &H160) -> Result<Vec<u8>> {
        // Simplified decryption
        let mut decrypted = encrypted.to_vec();
        for (i, byte) in decrypted.iter_mut().enumerate() {
            *byte ^= holder.as_bytes()[i % 20];
        }
        Ok(decrypted)
    }

    /// Set access policy for a model
    pub fn set_access_policy(
        &self,
        model_id: H256,
        policy: AccessPolicy,
    ) -> Result<()> {
        self.access_policies.write().insert(model_id, policy);
        Ok(())
    }

    /// Check if address has access
    pub fn check_access(
        &self,
        model_id: H256,
        address: H160,
        access_type: AccessType,
    ) -> Result<bool> {
        let policies = self.access_policies.read();
        let policy = policies.get(&model_id)
            .ok_or_else(|| anyhow!("No policy found for model"))?;

        // Owner always has access
        if address == policy.owner {
            return Ok(true);
        }

        // Check full access list
        if policy.full_access.contains(&address) {
            return Ok(true);
        }

        // Check inference-only for inference requests
        if access_type == AccessType::Inference && policy.inference_only.contains(&address) {
            return Ok(true);
        }

        // Check time-limited access
        if let Some(time_limited) = policy.time_limited.get(&address) {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();

            if now >= time_limited.valid_from && now <= time_limited.valid_until {
                if let Some(max_uses) = time_limited.max_uses {
                    if time_limited.current_uses < max_uses {
                        return Ok(true);
                    }
                }
            }
        }

        Ok(false)
    }

    /// Schedule key rotation
    pub fn schedule_rotation(&self, key_id: H256, rotation_time: u64) -> Result<()> {
        self.rotation_schedule.write().insert(key_id, rotation_time);
        Ok(())
    }

    /// Get expired keys that need rotation
    pub fn get_expired_keys(&self) -> Vec<H256> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let keys = self.derived_keys.read();
        keys.iter()
            .filter(|(_, key)| key.expires_at > 0 && key.expires_at <= now)
            .map(|(id, _)| *id)
            .collect()
    }

    /// Clear expired keys from cache
    pub fn cleanup_expired(&self) {
        let expired = self.get_expired_keys();
        let mut keys = self.derived_keys.write();

        for key_id in expired {
            keys.remove(&key_id);
        }
    }
}

/// Type of access being requested
#[derive(Debug, Clone, PartialEq)]
pub enum AccessType {
    /// Full model access (download, modify)
    Full,

    /// Inference only
    Inference,

    /// Metadata only
    Metadata,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_derivation() {
        let seed = [0u8; 64];
        let manager = KeyManager::from_seed(&seed).unwrap();

        let key1 = manager.derive_key("m/44'/60'/0'/0/0", KeyPurpose::ModelEncryption).unwrap();
        let key2 = manager.derive_key("m/44'/60'/0'/0/1", KeyPurpose::ModelEncryption).unwrap();

        assert_ne!(key1.key, key2.key);
        assert_ne!(key1.key_id, key2.key_id);
    }

    #[test]
    fn test_threshold_keys() {
        let seed = [1u8; 64];
        let manager = KeyManager::from_seed(&seed).unwrap();

        let model_id = H256::random();
        let holders = vec![H160::random(), H160::random(), H160::random()];

        let threshold_key = manager.create_threshold_key(
            model_id,
            2, // 2 of 3
            holders.clone(),
        ).unwrap();

        assert_eq!(threshold_key.threshold, 2);
        assert_eq!(threshold_key.total_shares, 3);
        assert_eq!(threshold_key.share_holders.len(), 3);
    }

    #[test]
    fn test_access_policy() {
        let manager = KeyManager::new();
        let model_id = H256::random();
        let owner = H160::random();
        let user = H160::random();

        let policy = AccessPolicy {
            owner,
            full_access: vec![],
            inference_only: vec![user],
            time_limited: HashMap::new(),
            requires_payment: false,
            min_stake: None,
        };

        manager.set_access_policy(model_id, policy).unwrap();

        // Owner should have full access
        assert!(manager.check_access(model_id, owner, AccessType::Full).unwrap());

        // User should have inference access only
        assert!(manager.check_access(model_id, user, AccessType::Inference).unwrap());
        assert!(!manager.check_access(model_id, user, AccessType::Full).unwrap());
    }
}