// citrate/core/consensus/src/vrf.rs

use crate::types::{Hash, PublicKey, VrfProof};
use sha3::{Digest, Sha3_256};
use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::RwLock;
use tracing::info;

#[derive(Error, Debug)]
pub enum VrfError {
    #[error("Invalid VRF proof")]
    InvalidProof,

    #[error("Validator not found")]
    ValidatorNotFound,

    #[error("Threshold not met")]
    ThresholdNotMet,

    #[error("Cryptographic error: {0}")]
    CryptoError(String),
}

/// Validator information for VRF
#[derive(Debug, Clone)]
pub struct Validator {
    pub pubkey: PublicKey,
    pub stake: u128,
    pub is_active: bool,
}

/// VRF-based proposer selection
pub struct VrfProposerSelector {
    validators: Arc<RwLock<HashMap<PublicKey, Validator>>>,
    total_stake: Arc<RwLock<u128>>,
    difficulty_adjustment: f64,
}

impl VrfProposerSelector {
    pub fn new() -> Self {
        Self {
            validators: Arc::new(RwLock::new(HashMap::new())),
            total_stake: Arc::new(RwLock::new(0)),
            difficulty_adjustment: 1.0,
        }
    }

    /// Register a validator
    pub async fn register_validator(&self, validator: Validator) {
        let mut validators = self.validators.write().await;
        let mut total_stake = self.total_stake.write().await;

        if validator.is_active {
            *total_stake += validator.stake;
        }

        validators.insert(validator.pubkey, validator.clone());
        info!("Registered validator with stake {}", validator.stake);
    }

    /// Remove a validator
    pub async fn remove_validator(&self, pubkey: &PublicKey) -> Result<(), VrfError> {
        let mut validators = self.validators.write().await;
        let mut total_stake = self.total_stake.write().await;

        if let Some(validator) = validators.remove(pubkey) {
            if validator.is_active {
                *total_stake = total_stake.saturating_sub(validator.stake);
            }
            Ok(())
        } else {
            Err(VrfError::ValidatorNotFound)
        }
    }

    /// Generate VRF proof for proposer eligibility
    pub fn generate_vrf_proof(
        &self,
        secret_key: &[u8; 32],
        previous_vrf: &Hash,
        slot: u64,
    ) -> Result<VrfProof, VrfError> {
        // Create input for VRF
        let mut hasher = Sha3_256::new();
        hasher.update(previous_vrf.as_bytes());
        hasher.update(slot.to_le_bytes());
        let input = hasher.finalize();

        // Generate VRF proof (simplified - in production use proper VRF like ECVRF)
        let mut proof_hasher = Sha3_256::new();
        proof_hasher.update(secret_key);
        proof_hasher.update(input);
        let proof_bytes = proof_hasher.finalize();

        // Generate VRF output
        let mut output_hasher = Sha3_256::new();
        output_hasher.update(proof_bytes);
        let output_bytes = output_hasher.finalize();

        Ok(VrfProof {
            proof: proof_bytes.to_vec(),
            output: Hash::from_bytes(&output_bytes),
        })
    }

    /// Verify VRF proof
    pub fn verify_vrf_proof(
        &self,
        _pubkey: &PublicKey,
        proof: &VrfProof,
        previous_vrf: &Hash,
        slot: u64,
    ) -> Result<bool, VrfError> {
        // Create expected input
        let mut hasher = Sha3_256::new();
        hasher.update(previous_vrf.as_bytes());
        hasher.update(slot.to_le_bytes());
        let _input = hasher.finalize();

        // Verify proof matches expected format
        // In production, use proper VRF verification
        if proof.proof.len() != 32 {
            return Ok(false);
        }

        // Verify output matches proof
        let mut output_hasher = Sha3_256::new();
        output_hasher.update(&proof.proof);
        let expected_output = Hash::from_bytes(&output_hasher.finalize());

        Ok(proof.output == expected_output)
    }

    /// Check if a validator is eligible to propose for a slot
    pub async fn is_eligible_proposer(
        &self,
        pubkey: &PublicKey,
        vrf_output: &Hash,
        slot: u64,
    ) -> Result<bool, VrfError> {
        let validators = self.validators.read().await;
        let total_stake = self.total_stake.read().await;

        let validator = validators.get(pubkey).ok_or(VrfError::ValidatorNotFound)?;

        if !validator.is_active {
            return Ok(false);
        }

        // Calculate threshold based on stake
        let stake_ratio = validator.stake as f64 / *total_stake as f64;
        let threshold = self.calculate_threshold(stake_ratio, slot);

        // Convert VRF output to a number between 0 and 1
        let vrf_value = self.vrf_output_to_float(vrf_output);

        Ok(vrf_value < threshold)
    }

    /// Calculate threshold for proposer eligibility
    fn calculate_threshold(&self, stake_ratio: f64, slot: u64) -> f64 {
        // Base threshold proportional to stake
        let base_threshold = stake_ratio * self.difficulty_adjustment;

        // Add time-based variation to prevent predictability
        let time_factor = ((slot % 100) as f64 / 100.0) * 0.1;

        (base_threshold + time_factor).min(1.0)
    }

    /// Convert VRF output to a float between 0 and 1
    fn vrf_output_to_float(&self, output: &Hash) -> f64 {
        let bytes = output.as_bytes();
        let mut value = 0u64;

        // Use first 8 bytes for the value
        for &b in bytes.iter().take(8) {
            value = (value << 8) | b as u64;
        }

        value as f64 / u64::MAX as f64
    }

    /// Select proposer for a slot
    pub async fn select_proposer(
        &self,
        slot: u64,
        previous_vrf: &Hash,
    ) -> Result<Option<PublicKey>, VrfError> {
        let validators = self.validators.read().await;

        let mut best_vrf_value = f64::MAX;
        let mut selected_proposer = None;

        // Each validator computes their VRF and the lowest wins
        for (pubkey, validator) in validators.iter() {
            if !validator.is_active {
                continue;
            }

            // Simulate VRF output for this validator
            let mut hasher = Sha3_256::new();
            hasher.update(pubkey.0);
            hasher.update(previous_vrf.as_bytes());
            hasher.update(slot.to_le_bytes());
            let vrf_output = Hash::from_bytes(&hasher.finalize());

            let vrf_value = self.vrf_output_to_float(&vrf_output);

            // Weight by stake
            let weighted_value = vrf_value / (validator.stake as f64).sqrt();

            if weighted_value < best_vrf_value {
                best_vrf_value = weighted_value;
                selected_proposer = Some(*pubkey);
            }
        }

        Ok(selected_proposer)
    }

    /// Update validator stake
    pub async fn update_stake(&self, pubkey: &PublicKey, new_stake: u128) -> Result<(), VrfError> {
        let mut validators = self.validators.write().await;
        let mut total_stake = self.total_stake.write().await;

        if let Some(validator) = validators.get_mut(pubkey) {
            if validator.is_active {
                *total_stake = total_stake.saturating_sub(validator.stake);
                *total_stake += new_stake;
            }
            validator.stake = new_stake;
            Ok(())
        } else {
            Err(VrfError::ValidatorNotFound)
        }
    }

    /// Get active validator count
    pub async fn active_validator_count(&self) -> usize {
        self.validators
            .read()
            .await
            .values()
            .filter(|v| v.is_active)
            .count()
    }

    /// Get total stake
    pub async fn total_stake(&self) -> u128 {
        *self.total_stake.read().await
    }
}

impl Default for VrfProposerSelector {
    fn default() -> Self {
        Self::new()
    }
}

/// Leader election using VRF
pub struct LeaderElection {
    vrf_selector: Arc<VrfProposerSelector>,
    _epoch_length: u64,
    slots_per_epoch: u64,
}

impl LeaderElection {
    pub fn new(vrf_selector: Arc<VrfProposerSelector>, epoch_length: u64) -> Self {
        Self {
            vrf_selector,
            _epoch_length: epoch_length,
            slots_per_epoch: epoch_length,
        }
    }

    /// Get current epoch from slot
    pub fn get_epoch(&self, slot: u64) -> u64 {
        slot / self.slots_per_epoch
    }

    /// Get slot within epoch
    pub fn get_slot_in_epoch(&self, slot: u64) -> u64 {
        slot % self.slots_per_epoch
    }

    /// Elect leader for a slot
    pub async fn elect_leader(
        &self,
        slot: u64,
        previous_vrf: &Hash,
    ) -> Result<Option<PublicKey>, VrfError> {
        self.vrf_selector.select_proposer(slot, previous_vrf).await
    }

    /// Verify leader eligibility
    pub async fn verify_leader(
        &self,
        pubkey: &PublicKey,
        proof: &VrfProof,
        slot: u64,
        previous_vrf: &Hash,
    ) -> Result<bool, VrfError> {
        // Verify VRF proof
        if !self
            .vrf_selector
            .verify_vrf_proof(pubkey, proof, previous_vrf, slot)?
        {
            return Ok(false);
        }

        // Check eligibility
        self.vrf_selector
            .is_eligible_proposer(pubkey, &proof.output, slot)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_validator_registration() {
        let selector = VrfProposerSelector::new();

        let validator = Validator {
            pubkey: PublicKey::new([1; 32]),
            stake: 1000,
            is_active: true,
        };

        selector.register_validator(validator).await;

        assert_eq!(selector.active_validator_count().await, 1);
        assert_eq!(selector.total_stake().await, 1000);
    }

    #[tokio::test]
    async fn test_vrf_proof_generation() {
        let selector = VrfProposerSelector::new();
        let secret_key = [42; 32];
        let previous_vrf = Hash::new([1; 32]);
        let slot = 100;

        let proof = selector
            .generate_vrf_proof(&secret_key, &previous_vrf, slot)
            .unwrap();

        assert_eq!(proof.proof.len(), 32);
        assert_ne!(proof.output, Hash::default());
    }

    #[tokio::test]
    async fn test_proposer_selection() {
        let selector = Arc::new(VrfProposerSelector::new());

        // Register multiple validators
        for i in 0..5 {
            let validator = Validator {
                pubkey: PublicKey::new([i as u8; 32]),
                stake: 1000 * (i as u128 + 1),
                is_active: true,
            };
            selector.register_validator(validator).await;
        }

        let previous_vrf = Hash::new([0; 32]);
        let proposer = selector.select_proposer(1, &previous_vrf).await.unwrap();

        assert!(proposer.is_some());
    }

    #[tokio::test]
    async fn test_leader_election() {
        let vrf_selector = Arc::new(VrfProposerSelector::new());
        let leader_election = LeaderElection::new(vrf_selector.clone(), 100);

        // Register validators
        for i in 0..3 {
            let validator = Validator {
                pubkey: PublicKey::new([i as u8; 32]),
                stake: 1000,
                is_active: true,
            };
            vrf_selector.register_validator(validator).await;
        }

        let previous_vrf = Hash::new([0; 32]);
        let leader = leader_election
            .elect_leader(50, &previous_vrf)
            .await
            .unwrap();

        assert!(leader.is_some());
        assert_eq!(leader_election.get_epoch(50), 0);
        assert_eq!(leader_election.get_slot_in_epoch(50), 50);
    }
}
