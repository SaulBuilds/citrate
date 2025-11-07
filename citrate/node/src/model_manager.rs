// Automatic Model Manager
//
// This module handles automatic downloading, pinning, and management of AI models
// required by the Citrate network. It makes model management seamless for validators
// and users with minimal IPFS experience.

use citrate_consensus::types::{Hash, RequiredModel};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::fs;
use tokio::io::AsyncWriteExt;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// Status of a model download/pin operation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModelStatus {
    NotPinned,
    Downloading { progress_bytes: u64, total_bytes: u64 },
    Verifying,
    Pinned { last_verified: u64 },
    Failed { error: String },
}

/// Metadata about a pinned model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PinnedModelMetadata {
    pub cid: String,
    pub model_id: String,
    pub file_path: PathBuf,
    pub size_bytes: u64,
    pub sha256_hash: String,
    pub pinned_at: u64,
    pub last_verified: u64,
    pub status: ModelStatus,
}

/// Configuration for model manager
#[derive(Debug, Clone)]
pub struct ModelManagerConfig {
    /// IPFS API endpoint
    pub ipfs_api_url: String,
    /// Directory to store downloaded models
    pub models_dir: PathBuf,
    /// Whether to auto-pin required models on startup
    pub auto_pin: bool,
    /// Timeout for IPFS downloads (seconds)
    pub download_timeout_secs: u64,
    /// Maximum retry attempts
    pub max_retries: u32,
}

impl Default for ModelManagerConfig {
    fn default() -> Self {
        Self {
            ipfs_api_url: "http://127.0.0.1:5001".to_string(),
            models_dir: PathBuf::from(".citrate/models"),
            auto_pin: true,
            download_timeout_secs: 3600, // 1 hour for large models
            max_retries: 3,
        }
    }
}

/// Model manager handles automatic downloading and pinning
pub struct ModelManager {
    config: ModelManagerConfig,
    ipfs_client: reqwest::Client,
    pinned_models: Arc<RwLock<HashMap<String, PinnedModelMetadata>>>,
    metadata_file: PathBuf,
}

impl ModelManager {
    /// Create a new model manager
    pub async fn new(config: ModelManagerConfig) -> Result<Self, String> {
        // Create models directory if it doesn't exist
        fs::create_dir_all(&config.models_dir)
            .await
            .map_err(|e| format!("Failed to create models directory: {}", e))?;

        let metadata_file = config.models_dir.join("pinned_models.json");

        let mut manager = Self {
            config,
            ipfs_client: reqwest::Client::builder()
                .timeout(Duration::from_secs(3600))
                .build()
                .map_err(|e| format!("Failed to create HTTP client: {}", e))?,
            pinned_models: Arc::new(RwLock::new(HashMap::new())),
            metadata_file,
        };

        // Load existing metadata
        manager.load_metadata().await?;

        Ok(manager)
    }

    /// Check if IPFS daemon is running
    pub async fn check_ipfs_daemon(&self) -> Result<(), String> {
        let url = format!("{}/api/v0/version", self.config.ipfs_api_url);

        match self.ipfs_client.post(&url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    info!("IPFS daemon is running");
                    Ok(())
                } else {
                    Err(format!("IPFS daemon returned status: {}", response.status()))
                }
            }
            Err(e) => Err(format!(
                "IPFS daemon not reachable at {}: {}. Please start IPFS with 'ipfs daemon'",
                self.config.ipfs_api_url, e
            )),
        }
    }

    /// Auto-pin all required models from genesis block
    pub async fn auto_pin_required_models(&self, required_models: &[RequiredModel]) -> Result<(), String> {
        if !self.config.auto_pin {
            info!("Auto-pinning disabled, skipping");
            return Ok(());
        }

        // Check IPFS daemon first
        if let Err(e) = self.check_ipfs_daemon().await {
            error!("Cannot auto-pin models: {}", e);
            return Err(e);
        }

        info!("Starting automatic pinning of {} required models", required_models.len());

        for model in required_models {
            if !model.must_pin {
                debug!("Model {} is not required to be pinned, skipping", model.model_id.0);
                continue;
            }

            // Check if already pinned
            if self.is_model_pinned(&model.ipfs_cid).await {
                info!("Model {} ({}) already pinned", model.model_id.0, model.ipfs_cid);
                continue;
            }

            info!(
                "Auto-pinning model {} (CID: {}, Size: {} MB)",
                model.model_id.0,
                model.ipfs_cid,
                model.size_bytes / 1_000_000
            );

            // Download and pin with retries
            match self.download_and_pin_model(model).await {
                Ok(_) => {
                    info!("Successfully pinned model {}", model.model_id.0);
                }
                Err(e) => {
                    error!("Failed to pin model {}: {}", model.model_id.0, e);
                    // Continue with other models even if one fails
                }
            }
        }

        info!("Completed automatic pinning cycle");
        Ok(())
    }

    /// Download and pin a model from IPFS
    pub async fn download_and_pin_model(&self, model: &RequiredModel) -> Result<(), String> {
        let file_path = self
            .config
            .models_dir
            .join(format!("{}.gguf", model.model_id.0));

        // Update status to downloading
        self.update_model_status(
            &model.ipfs_cid,
            ModelStatus::Downloading {
                progress_bytes: 0,
                total_bytes: model.size_bytes,
            },
        )
        .await;

        // Download from IPFS with timeout
        info!("Downloading model from IPFS: {}", model.ipfs_cid);
        let url = format!("{}/api/v0/cat?arg={}", self.config.ipfs_api_url, model.ipfs_cid);

        let response = self
            .ipfs_client
            .post(&url)
            .timeout(Duration::from_secs(self.config.download_timeout_secs))
            .send()
            .await
            .map_err(|e| format!("Failed to fetch from IPFS: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("IPFS returned status: {}", response.status()));
        }

        // Stream to file and calculate hash
        let mut file = fs::File::create(&file_path)
            .await
            .map_err(|e| format!("Failed to create file: {}", e))?;

        let mut hasher = Sha256::new();

        let bytes = response
            .bytes()
            .await
            .map_err(|e| format!("Failed to read response: {}", e))?;

        file.write_all(&bytes)
            .await
            .map_err(|e| format!("Failed to write file: {}", e))?;

        hasher.update(&bytes);
        let downloaded = bytes.len() as u64;

        info!(
            "Downloaded {} MB / {} MB",
            downloaded / 1_000_000,
            model.size_bytes / 1_000_000
        );

        // Verify integrity
        self.update_model_status(&model.ipfs_cid, ModelStatus::Verifying)
            .await;

        let computed_hash = hasher.finalize();
        let computed_hash_bytes: [u8; 32] = computed_hash.into();

        if Hash::new(computed_hash_bytes) != model.sha256_hash {
            fs::remove_file(&file_path).await.ok();
            return Err("SHA256 hash mismatch - file corrupted".to_string());
        }

        info!("Model integrity verified successfully");

        // Pin in IPFS
        info!("Pinning model in IPFS: {}", model.ipfs_cid);
        let pin_url = format!("{}/api/v0/pin/add?arg={}", self.config.ipfs_api_url, model.ipfs_cid);

        let pin_response = self
            .ipfs_client
            .post(&pin_url)
            .send()
            .await
            .map_err(|e| format!("Failed to pin in IPFS: {}", e))?;

        if !pin_response.status().is_success() {
            return Err(format!("IPFS pin failed: {}", pin_response.status()));
        }

        // Save metadata
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let metadata = PinnedModelMetadata {
            cid: model.ipfs_cid.clone(),
            model_id: model.model_id.0.clone(),
            file_path: file_path.clone(),
            size_bytes: model.size_bytes,
            sha256_hash: hex::encode(computed_hash_bytes),
            pinned_at: now,
            last_verified: now,
            status: ModelStatus::Pinned { last_verified: now },
        };

        self.save_model_metadata(&model.ipfs_cid, metadata).await?;

        info!(
            "Successfully downloaded and pinned model {} at {}",
            model.model_id.0,
            file_path.display()
        );

        Ok(())
    }

    /// Check if a model is already pinned
    pub async fn is_model_pinned(&self, cid: &str) -> bool {
        let models = self.pinned_models.read().await;
        if let Some(metadata) = models.get(cid) {
            matches!(metadata.status, ModelStatus::Pinned { .. })
        } else {
            false
        }
    }

    /// Get status of a model
    pub async fn get_model_status(&self, cid: &str) -> Option<ModelStatus> {
        let models = self.pinned_models.read().await;
        models.get(cid).map(|m| m.status.clone())
    }

    /// Get path to a pinned model file
    pub async fn get_model_path(&self, cid: &str) -> Option<PathBuf> {
        let models = self.pinned_models.read().await;
        models.get(cid).map(|m| m.file_path.clone())
    }

    /// List all pinned models
    pub async fn list_pinned_models(&self) -> Vec<PinnedModelMetadata> {
        let models = self.pinned_models.read().await;
        models.values().cloned().collect()
    }

    /// Update model status
    async fn update_model_status(&self, cid: &str, status: ModelStatus) {
        let mut models = self.pinned_models.write().await;
        if let Some(metadata) = models.get_mut(cid) {
            metadata.status = status;
        }
    }

    /// Save model metadata to disk
    async fn save_model_metadata(&self, cid: &str, metadata: PinnedModelMetadata) -> Result<(), String> {
        let mut models = self.pinned_models.write().await;
        models.insert(cid.to_string(), metadata);

        // Write to disk
        let json = serde_json::to_string_pretty(&*models)
            .map_err(|e| format!("Failed to serialize metadata: {}", e))?;

        fs::write(&self.metadata_file, json)
            .await
            .map_err(|e| format!("Failed to write metadata file: {}", e))?;

        Ok(())
    }

    /// Load model metadata from disk
    async fn load_metadata(&mut self) -> Result<(), String> {
        if !self.metadata_file.exists() {
            debug!("No existing metadata file found");
            return Ok(());
        }

        let json = fs::read_to_string(&self.metadata_file)
            .await
            .map_err(|e| format!("Failed to read metadata file: {}", e))?;

        let models: HashMap<String, PinnedModelMetadata> = serde_json::from_str(&json)
            .map_err(|e| format!("Failed to parse metadata: {}", e))?;

        *self.pinned_models.write().await = models;

        info!(
            "Loaded metadata for {} pinned models",
            self.pinned_models.read().await.len()
        );

        Ok(())
    }

    /// Unpin a model (remove from IPFS and delete file)
    pub async fn unpin_model(&self, cid: &str) -> Result<(), String> {
        // Remove from IPFS
        let url = format!("{}/api/v0/pin/rm?arg={}", self.config.ipfs_api_url, cid);

        let response = self
            .ipfs_client
            .post(&url)
            .send()
            .await
            .map_err(|e| format!("Failed to unpin from IPFS: {}", e))?;

        if !response.status().is_success() {
            warn!("IPFS unpin returned status: {}", response.status());
        }

        // Delete local file
        let models = self.pinned_models.read().await;
        if let Some(metadata) = models.get(cid) {
            if metadata.file_path.exists() {
                fs::remove_file(&metadata.file_path)
                    .await
                    .map_err(|e| format!("Failed to delete file: {}", e))?;
            }
        }
        drop(models);

        // Remove from metadata
        let mut models = self.pinned_models.write().await;
        models.remove(cid);

        // Save updated metadata
        let json = serde_json::to_string_pretty(&*models)
            .map_err(|e| format!("Failed to serialize metadata: {}", e))?;

        fs::write(&self.metadata_file, json)
            .await
            .map_err(|e| format!("Failed to write metadata file: {}", e))?;

        info!("Successfully unpinned model {}", cid);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_manager_creation() {
        let config = ModelManagerConfig {
            models_dir: PathBuf::from("/tmp/citrate-test-models"),
            ..Default::default()
        };

        let manager = ModelManager::new(config).await;
        assert!(manager.is_ok());
    }

    #[tokio::test]
    async fn test_model_status_tracking() {
        let config = ModelManagerConfig {
            models_dir: PathBuf::from("/tmp/citrate-test-models"),
            ..Default::default()
        };

        let manager = ModelManager::new(config).await.unwrap();

        // Initially not pinned
        assert!(!manager.is_model_pinned("QmTest123").await);

        // Status should be None
        assert!(manager.get_model_status("QmTest123").await.is_none());
    }
}
