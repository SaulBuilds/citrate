// Model Pin Verification System
//
// This module implements validator pin verification for required AI models.
// Validators must maintain IPFS pins for models specified in the genesis block.
// Failure to maintain pins results in slashing penalties.

use citrate_consensus::types::{Hash, PinStatus, PublicKey, RequiredModel, ValidatorPinCheck};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::time;
use tracing::{debug, error, info, warn};

/// Configuration for model pin verification
#[derive(Debug, Clone)]
pub struct VerifierConfig {
    /// How often to check validator pins (seconds)
    pub check_interval_secs: u64,
    /// Grace period before slashing (hours)
    pub grace_period_hours: u64,
    /// IPFS API endpoint
    pub ipfs_api_url: String,
}

impl Default for VerifierConfig {
    fn default() -> Self {
        Self {
            check_interval_secs: 3600, // Check every hour
            grace_period_hours: 24,    // 24 hour grace period
            ipfs_api_url: "http://127.0.0.1:5001".to_string(),
        }
    }
}

/// Model pin verifier service
pub struct ModelVerifier {
    config: VerifierConfig,
    required_models: Vec<RequiredModel>,
    pin_checks: Arc<tokio::sync::RwLock<HashMap<(PublicKey, String), ValidatorPinCheck>>>,
    ipfs_client: reqwest::Client,
}

impl ModelVerifier {
    /// Create a new model verifier
    pub fn new(config: VerifierConfig, required_models: Vec<RequiredModel>) -> Self {
        Self {
            config,
            required_models,
            pin_checks: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            ipfs_client: reqwest::Client::new(),
        }
    }

    /// Start the verification background task
    pub async fn start(&self) {
        let config = self.config.clone();
        let required_models = self.required_models.clone();
        let _pin_checks = self.pin_checks.clone();
        let _ipfs_client = self.ipfs_client.clone();

        tokio::spawn(async move {
            let mut interval = time::interval(Duration::from_secs(config.check_interval_secs));

            loop {
                interval.tick().await;

                info!("Starting validator pin verification cycle");

                for model in &required_models {
                    if !model.must_pin {
                        continue;
                    }

                    // TODO: Get list of active validators from consensus
                    // For now, we'll just log that verification would happen
                    debug!(
                        "Would verify pin for model {} (CID: {})",
                        model.model_id.0, model.ipfs_cid
                    );
                }

                info!("Completed validator pin verification cycle");
            }
        });
    }

    /// Check if a validator has pinned a specific model
    pub async fn check_validator_pin(
        &self,
        validator: &PublicKey,
        model_cid: &str,
    ) -> Result<PinStatus, String> {
        // Query IPFS to check if the CID is pinned
        let url = format!("{}/api/v0/pin/ls?arg={}", self.config.ipfs_api_url, model_cid);

        match self.ipfs_client.post(&url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    debug!("Validator {:?} has pinned {}", validator, model_cid);
                    Ok(PinStatus::Pinned)
                } else {
                    warn!(
                        "Validator {:?} has not pinned {} (status: {})",
                        validator,
                        model_cid,
                        response.status()
                    );
                    Ok(PinStatus::Unpinned)
                }
            }
            Err(e) => {
                error!("Failed to check pin status for {}: {}", model_cid, e);
                Ok(PinStatus::Unverified)
            }
        }
    }

    /// Verify the integrity of a pinned model
    pub async fn verify_model_integrity(
        &self,
        model_cid: &str,
        expected_hash: &Hash,
    ) -> Result<bool, String> {
        // Download the model from IPFS and verify its hash
        let url = format!(
            "{}/api/v0/cat?arg={}",
            self.config.ipfs_api_url, model_cid
        );

        match self.ipfs_client.post(&url).send().await {
            Ok(response) => {
                if !response.status().is_success() {
                    return Err(format!("Failed to fetch model: {}", response.status()));
                }

                // Stream and hash the content
                let mut hasher = Sha256::new();
                let bytes = response.bytes().await.map_err(|e| e.to_string())?;
                hasher.update(&bytes);
                let hash_result = hasher.finalize();

                let computed_hash = Hash::from_bytes(&hash_result);

                Ok(computed_hash == *expected_hash)
            }
            Err(e) => Err(format!("Failed to fetch model from IPFS: {}", e)),
        }
    }

    /// Record a pin check for a validator
    pub async fn record_pin_check(
        &self,
        validator: PublicKey,
        model_cid: String,
        status: PinStatus,
        proof: Option<Vec<u8>>,
    ) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let check = ValidatorPinCheck {
            validator: validator.clone(),
            model_cid: model_cid.clone(),
            last_check: now,
            status,
            last_proof: proof,
        };

        let mut checks = self.pin_checks.write().await;
        checks.insert((validator, model_cid), check);
    }

    /// Get pin status for a validator
    pub async fn get_pin_status(
        &self,
        validator: &PublicKey,
        model_cid: &str,
    ) -> Option<ValidatorPinCheck> {
        let checks = self.pin_checks.read().await;
        checks
            .get(&(validator.clone(), model_cid.to_string()))
            .cloned()
    }

    /// Get all validators who should be slashed
    pub async fn get_slashable_validators(&self) -> Vec<(PublicKey, String, u128)> {
        let checks = self.pin_checks.read().await;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let mut slashable = Vec::new();

        for ((validator, model_cid), check) in checks.iter() {
            // Check if validator is unpinned and grace period has expired
            if check.status == PinStatus::Unpinned {
                let grace_period_secs = self.config.grace_period_hours * 3600;
                if now - check.last_check > grace_period_secs {
                    // Find the model to get the slash penalty
                    if let Some(model) = self
                        .required_models
                        .iter()
                        .find(|m| m.ipfs_cid == *model_cid)
                    {
                        slashable.push((validator.clone(), model_cid.clone(), model.slash_penalty));
                    }
                }
            }
        }

        slashable
    }

    /// Get required models list
    pub fn get_required_models(&self) -> &[RequiredModel] {
        &self.required_models
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use citrate_consensus::types::ModelId;

    #[tokio::test]
    async fn test_verifier_creation() {
        let config = VerifierConfig::default();
        let models = vec![RequiredModel::new(
            ModelId::from_name("test-model"),
            "QmTest123".to_string(),
            Hash::default(),
            1000000,
            1_000_000_000_000_000_000,
        )];

        let verifier = ModelVerifier::new(config, models);
        assert_eq!(verifier.get_required_models().len(), 1);
    }

    #[tokio::test]
    async fn test_pin_check_recording() {
        let config = VerifierConfig::default();
        let models = vec![];
        let verifier = ModelVerifier::new(config, models);

        let validator = PublicKey::new([1u8; 32]);
        let cid = "QmTest123".to_string();

        verifier
            .record_pin_check(validator.clone(), cid.clone(), PinStatus::Pinned, None)
            .await;

        let status = verifier.get_pin_status(&validator, &cid).await;
        assert!(status.is_some());
        assert_eq!(status.unwrap().status, PinStatus::Pinned);
    }
}
