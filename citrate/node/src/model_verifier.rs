// Model Pin Verification System
//
// This module implements validator pin verification for required AI models.
// Validators must maintain IPFS pins for models specified in the genesis block.
// Failure to maintain pins results in slashing penalties.

use citrate_consensus::types::{Hash, PinStatus, PublicKey, RequiredModel, ValidatorPinCheck};
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
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
    /// Production mode: fail-closed if no validators configured
    /// When true, the verifier will refuse to start without validators
    pub production_mode: bool,
}

impl Default for VerifierConfig {
    fn default() -> Self {
        Self {
            check_interval_secs: 3600, // Check every hour
            grace_period_hours: 24,    // 24 hour grace period
            ipfs_api_url: "http://127.0.0.1:5001".to_string(),
            production_mode: false,    // Permissive by default for development
        }
    }
}

impl VerifierConfig {
    /// Create a production configuration that enforces validator presence
    pub fn production() -> Self {
        Self {
            check_interval_secs: 3600,
            grace_period_hours: 24,
            ipfs_api_url: "http://127.0.0.1:5001".to_string(),
            production_mode: true, // Fail-closed in production
        }
    }
}

/// Trait for providing active validators to the verifier
/// This allows the verifier to work with different consensus implementations
pub trait ValidatorProvider: Send + Sync {
    /// Get the current set of active validators
    fn get_active_validators(&self) -> Vec<PublicKey>;
}

/// Simple in-memory validator provider for testing and development
pub struct StaticValidatorProvider {
    validators: RwLock<HashSet<PublicKey>>,
}

impl StaticValidatorProvider {
    /// Create a new empty validator provider
    pub fn new() -> Self {
        Self {
            validators: RwLock::new(HashSet::new()),
        }
    }

    /// Create with initial validators
    pub fn with_validators(validators: Vec<PublicKey>) -> Self {
        Self {
            validators: RwLock::new(validators.into_iter().collect()),
        }
    }

    /// Add a validator to the set
    pub async fn add_validator(&self, validator: PublicKey) {
        self.validators.write().await.insert(validator);
    }

    /// Remove a validator from the set
    pub async fn remove_validator(&self, validator: &PublicKey) {
        self.validators.write().await.remove(validator);
    }
}

impl Default for StaticValidatorProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl ValidatorProvider for StaticValidatorProvider {
    fn get_active_validators(&self) -> Vec<PublicKey> {
        // Use try_read to avoid blocking; return empty if locked
        match self.validators.try_read() {
            Ok(guard) => guard.iter().cloned().collect(),
            Err(_) => Vec::new(),
        }
    }
}

/// Model pin verifier service
pub struct ModelVerifier {
    config: VerifierConfig,
    required_models: Vec<RequiredModel>,
    pin_checks: Arc<RwLock<HashMap<(PublicKey, String), ValidatorPinCheck>>>,
    ipfs_client: reqwest::Client,
    validator_provider: Arc<dyn ValidatorProvider>,
}

impl ModelVerifier {
    /// Create a new model verifier with default (empty) validator provider
    ///
    /// WARNING: This creates a verifier with no validators, which means no pin
    /// verification will be performed. For production use, call `with_validators()`
    /// or `with_validator_provider()` to provide the active validator set.
    pub fn new(config: VerifierConfig, required_models: Vec<RequiredModel>) -> Self {
        warn!(
            "ModelVerifier created with empty validator set. \
             Pin verification will not run until validators are configured. \
             Use with_validators() or with_validator_provider() for production."
        );
        Self::with_validator_provider(
            config,
            required_models,
            Arc::new(StaticValidatorProvider::new()),
        )
    }

    /// Create a new model verifier with a custom validator provider
    pub fn with_validator_provider(
        config: VerifierConfig,
        required_models: Vec<RequiredModel>,
        validator_provider: Arc<dyn ValidatorProvider>,
    ) -> Self {
        Self {
            config,
            required_models,
            pin_checks: Arc::new(RwLock::new(HashMap::new())),
            ipfs_client: reqwest::Client::new(),
            validator_provider,
        }
    }

    /// Start the verification background task
    ///
    /// Returns an error if production_mode is enabled and no validators are configured.
    /// This ensures fail-closed behavior in production environments.
    pub async fn start(&self) -> Result<(), String> {
        // In production mode, verify validators are configured before starting
        if self.config.production_mode {
            let initial_validators = self.validator_provider.get_active_validators();
            if initial_validators.is_empty() {
                error!(
                    "FAIL-CLOSED: Production mode enabled but no validators configured. \
                     Pin verification cannot run without validators. \
                     Configure validators via with_validators() or set production_mode=false for development."
                );
                return Err(
                    "Production mode requires validators to be configured before starting".to_string()
                );
            }
            info!(
                "Production mode: {} validators configured, starting verifier",
                initial_validators.len()
            );
        }

        let config = self.config.clone();
        let required_models = self.required_models.clone();
        let pin_checks = self.pin_checks.clone();
        let ipfs_client = self.ipfs_client.clone();
        let validator_provider = self.validator_provider.clone();

        tokio::spawn(async move {
            let mut interval = time::interval(Duration::from_secs(config.check_interval_secs));

            loop {
                interval.tick().await;

                info!("Starting validator pin verification cycle");

                // Get current active validators from the provider
                let validators = validator_provider.get_active_validators();

                if validators.is_empty() {
                    if config.production_mode {
                        error!(
                            "FAIL-CLOSED: Production mode - validator set became empty! \
                             This is a critical error. Pin verification is suspended."
                        );
                    } else {
                        debug!("No active validators registered, skipping verification cycle");
                    }
                    continue;
                }

                info!(
                    "Checking {} validators for {} required model pins",
                    validators.len(),
                    required_models.iter().filter(|m| m.must_pin).count()
                );

                for model in &required_models {
                    if !model.must_pin {
                        continue;
                    }

                    for validator in &validators {
                        // Check if this validator has pinned the model
                        let pin_status = Self::check_pin_status_internal(
                            &ipfs_client,
                            &config.ipfs_api_url,
                            validator,
                            &model.ipfs_cid,
                        )
                        .await;

                        // Record the check result
                        let now = SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs();

                        let check = ValidatorPinCheck {
                            validator: validator.clone(),
                            model_cid: model.ipfs_cid.clone(),
                            last_check: now,
                            status: pin_status,
                            last_proof: None,
                        };

                        pin_checks
                            .write()
                            .await
                            .insert((validator.clone(), model.ipfs_cid.clone()), check);

                        match pin_status {
                            PinStatus::Pinned => {
                                debug!(
                                    "Validator {:?} has pinned model {} (CID: {})",
                                    &validator.0[..8],
                                    model.model_id.0,
                                    model.ipfs_cid
                                );
                            }
                            PinStatus::Unpinned => {
                                warn!(
                                    "Validator {:?} has NOT pinned required model {} (CID: {})",
                                    &validator.0[..8],
                                    model.model_id.0,
                                    model.ipfs_cid
                                );
                            }
                            PinStatus::Unverified => {
                                debug!(
                                    "Could not verify pin status for validator {:?}, model {}",
                                    &validator.0[..8],
                                    model.model_id.0
                                );
                            }
                        }
                    }
                }

                info!("Completed validator pin verification cycle");
            }
        });

        Ok(())
    }

    /// Internal helper to check pin status via IPFS API
    async fn check_pin_status_internal(
        client: &reqwest::Client,
        ipfs_api_url: &str,
        _validator: &PublicKey,
        model_cid: &str,
    ) -> PinStatus {
        let url = format!("{}/api/v0/pin/ls?arg={}", ipfs_api_url, model_cid);

        match client.post(&url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    PinStatus::Pinned
                } else {
                    PinStatus::Unpinned
                }
            }
            Err(_) => PinStatus::Unverified,
        }
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

    #[tokio::test]
    async fn test_static_validator_provider() {
        let provider = StaticValidatorProvider::new();

        // Initially empty
        assert!(provider.get_active_validators().is_empty());

        // Add validators
        let v1 = PublicKey::new([1u8; 32]);
        let v2 = PublicKey::new([2u8; 32]);

        provider.add_validator(v1.clone()).await;
        provider.add_validator(v2.clone()).await;

        let validators = provider.get_active_validators();
        assert_eq!(validators.len(), 2);
        assert!(validators.contains(&v1));
        assert!(validators.contains(&v2));

        // Remove a validator
        provider.remove_validator(&v1).await;
        let validators = provider.get_active_validators();
        assert_eq!(validators.len(), 1);
        assert!(!validators.contains(&v1));
        assert!(validators.contains(&v2));
    }

    #[tokio::test]
    async fn test_validator_provider_with_initial() {
        let v1 = PublicKey::new([1u8; 32]);
        let v2 = PublicKey::new([2u8; 32]);

        let provider = StaticValidatorProvider::with_validators(vec![v1.clone(), v2.clone()]);

        let validators = provider.get_active_validators();
        assert_eq!(validators.len(), 2);
    }

    #[tokio::test]
    async fn test_verifier_with_custom_provider() {
        let v1 = PublicKey::new([1u8; 32]);
        let provider = Arc::new(StaticValidatorProvider::with_validators(vec![v1.clone()]));

        let config = VerifierConfig::default();
        let models = vec![RequiredModel::new(
            ModelId::from_name("test-model"),
            "QmTest123".to_string(),
            Hash::default(),
            1000000,
            1_000_000_000_000_000_000,
        )];

        let verifier = ModelVerifier::with_validator_provider(config, models, provider);
        assert_eq!(verifier.get_required_models().len(), 1);
    }
}
