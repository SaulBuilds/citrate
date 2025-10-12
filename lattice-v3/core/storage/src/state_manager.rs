// lattice-v3/core/storage/src/state_manager.rs

use crate::db::RocksDB;
use crate::state::{AIStateTree, StateStore};
use anyhow::Result;
use lattice_consensus::types::Hash;
use lattice_execution::{JobId, ModelId, ModelState, TrainingJob};
use sha3::{Digest, Sha3_256};
use std::sync::Arc;
use tracing::{debug, info};

/// Unified state manager combining account state and AI state
pub struct StateManager {
    /// Traditional account state store
    pub state_store: Arc<StateStore>,

    /// AI-specific state tree
    pub ai_state: Arc<parking_lot::RwLock<AIStateTree>>,

    /// Database reference
    db: Arc<RocksDB>,
}

impl StateManager {
    pub fn new(db: Arc<RocksDB>) -> Self {
        Self {
            state_store: Arc::new(StateStore::new(db.clone())),
            ai_state: Arc::new(parking_lot::RwLock::new(AIStateTree::new())),
            db,
        }
    }

    /// Calculate unified state root including AI state
    pub async fn calculate_state_root(&self) -> Result<Hash> {
        // Calculate account state root
        let account_root = self.calculate_account_root().await?;

        // Calculate contract storage root
        let storage_root = self.calculate_storage_root().await?;

        // Calculate AI state root
        let ai_root = self.ai_state.read().calculate_root()?;

        // Combine into unified state root
        let mut hasher = Sha3_256::new();
        hasher.update(account_root.as_bytes());
        hasher.update(storage_root.as_bytes());
        hasher.update(ai_root.as_bytes());

        let hash_bytes = hasher.finalize();
        let mut hash_array = [0u8; 32];
        hash_array.copy_from_slice(&hash_bytes[..32]);

        let unified_root = Hash::new(hash_array);

        debug!("Calculated unified state root: {:?}", unified_root);
        debug!("  Account root: {:?}", account_root);
        debug!("  Storage root: {:?}", storage_root);
        debug!("  AI root: {:?}", ai_root);

        Ok(unified_root)
    }

    /// Calculate account state root
    async fn calculate_account_root(&self) -> Result<Hash> {
        let mut hasher = Sha3_256::new();

        // Get all accounts from state store
        let accounts = self.state_store.get_all_accounts()?;

        // Sort accounts by address for deterministic ordering
        let mut sorted_accounts: Vec<_> = accounts.into_iter().collect();
        sorted_accounts.sort_by(|a, b| a.0 .0.cmp(&b.0 .0));

        // Hash each account's state
        for (address, account) in sorted_accounts {
            hasher.update(address.0);
            // Convert U256 to bytes
            let mut balance_bytes = [0u8; 32];
            account.balance.to_little_endian(&mut balance_bytes);
            hasher.update(balance_bytes);
            hasher.update(account.nonce.to_le_bytes());
            hasher.update(account.code_hash.as_bytes());
        }

        let hash_bytes = hasher.finalize();
        let mut hash_array = [0u8; 32];
        hash_array.copy_from_slice(&hash_bytes[..32]);
        Ok(Hash::new(hash_array))
    }

    /// Calculate storage root
    async fn calculate_storage_root(&self) -> Result<Hash> {
        let mut hasher = Sha3_256::new();

        // Get all contract storage data
        let storage_data = self.state_store.get_all_storage()?;

        // Sort by contract address and storage key for deterministic ordering
        let mut sorted_storage: Vec<_> = storage_data.into_iter().collect();
        sorted_storage.sort_by(|a, b| {
            // Compare addresses first, then storage keys
            a.0 .0
                 .0
                .cmp(&b.0 .0 .0)
                .then_with(|| a.0 .1.as_bytes().cmp(b.0 .1.as_bytes()))
        });

        // Hash each storage entry
        for ((address, key), value) in sorted_storage {
            hasher.update(address.0);
            hasher.update(key.as_bytes());
            hasher.update(value.as_bytes());
        }

        let hash_bytes = hasher.finalize();
        let mut hash_array = [0u8; 32];
        hash_array.copy_from_slice(&hash_bytes[..32]);
        Ok(Hash::new(hash_array))
    }

    /// Register a new AI model
    pub fn register_model(
        &self,
        model_id: ModelId,
        model_state: ModelState,
        weight_cid: String,
    ) -> Result<()> {
        // Persist to database first (before move)
        let key = format!("model:{}", hex::encode(model_id.0.as_bytes()));
        let data = bincode::serialize(&model_state)?;
        self.db.put_cf("state", key.as_bytes(), &data)?;

        // Then register in AI state (moves model_state)
        self.ai_state
            .write()
            .register_model(model_id, model_state, weight_cid);

        info!("Registered model {:?}", model_id);
        Ok(())
    }

    /// Update model weights
    pub fn update_model_weights(
        &self,
        model_id: ModelId,
        new_cid: String,
        new_version: u32,
    ) -> Result<()> {
        self.ai_state
            .write()
            .update_model_weights(model_id, new_cid.clone(), new_version);

        // Persist weight CID
        let key = format!("weights:{}", hex::encode(model_id.0.as_bytes()));
        self.db
            .put_cf("state", key.as_bytes(), new_cid.as_bytes())?;

        info!(
            "Updated model {:?} weights to version {}",
            model_id, new_version
        );
        Ok(())
    }

    /// Add training job
    pub fn add_training_job(&self, job_id: JobId, job: TrainingJob) -> Result<()> {
        self.ai_state
            .write()
            .training_jobs
            .insert(job_id, job.clone());

        // Persist to database
        let key = format!("job:{}", hex::encode(job_id.0.as_bytes()));
        let data = bincode::serialize(&job)?;
        self.db.put_cf("state", key.as_bytes(), &data)?;

        info!("Added training job {:?}", job_id);
        Ok(())
    }

    /// Cache inference result
    pub fn cache_inference_result(&self, result: crate::state::InferenceResult) -> Result<()> {
        self.ai_state.write().cache_inference(result.clone());

        // Persist to database with TTL (could use RocksDB TTL feature)
        let key = format!(
            "inference:{}:{}",
            hex::encode(result.model_id.0.as_bytes()),
            hex::encode(result.input_hash.as_bytes())
        );
        let data = bincode::serialize(&result)?;
        self.db.put_cf("cache", key.as_bytes(), &data)?;

        debug!("Cached inference result for model {:?}", result.model_id);
        Ok(())
    }

    /// Add LoRA adapter
    pub fn add_lora_adapter(&self, adapter: crate::state::LoRAAdapter) -> Result<()> {
        self.ai_state.write().add_lora_adapter(adapter.clone());

        // Persist to database
        let key = format!(
            "lora:{}:{}",
            hex::encode(adapter.base_model.0.as_bytes()),
            hex::encode(adapter.adapter_id.as_bytes())
        );
        let data = bincode::serialize(&adapter)?;
        self.db.put_cf("state", key.as_bytes(), &data)?;

        info!(
            "Added LoRA adapter {:?} for model {:?}",
            adapter.adapter_id, adapter.base_model
        );
        Ok(())
    }

    /// Get model state
    pub fn get_model(&self, model_id: &ModelId) -> Option<ModelState> {
        self.ai_state.read().models.get(model_id).cloned()
    }

    /// Get training job
    pub fn get_training_job(&self, job_id: &JobId) -> Option<TrainingJob> {
        self.ai_state.read().training_jobs.get(job_id).cloned()
    }

    /// Prune old inference cache entries
    pub fn prune_inference_cache(&self, max_age: u64) {
        let current_time = chrono::Utc::now().timestamp() as u64;
        self.ai_state
            .write()
            .prune_inference_cache(max_age, current_time);
        info!(
            "Pruned inference cache entries older than {} seconds",
            max_age
        );
    }

    /// Get AI state statistics
    pub fn get_ai_stats(&self) -> AIStateStats {
        let ai_state = self.ai_state.read();
        AIStateStats {
            total_models: ai_state.models.len(),
            active_training_jobs: ai_state.training_jobs.len(),
            cached_inferences: ai_state.inference_cache.len(),
            total_lora_adapters: ai_state.lora_adapters.values().map(|v| v.len()).sum(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct AIStateStats {
    pub total_models: usize,
    pub active_training_jobs: usize,
    pub cached_inferences: usize,
    pub total_lora_adapters: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use lattice_execution::types::Address;
    use lattice_execution::{AccessPolicy, ModelMetadata, UsageStats};
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_unified_state_root() {
        let temp_dir = TempDir::new().unwrap();
        let db = Arc::new(RocksDB::open(temp_dir.path()).unwrap());
        let state_manager = StateManager::new(db);

        // Calculate initial root
        let root1 = state_manager.calculate_state_root().await.unwrap();

        // Add a model
        let model_id = ModelId(Hash::new([1; 32]));
        let model_state = ModelState {
            owner: Address([2; 20]),
            model_hash: Hash::new([3; 32]),
            version: 1,
            metadata: ModelMetadata {
                name: "TestModel".to_string(),
                version: "1.0".to_string(),
                description: "Test".to_string(),
                framework: "PyTorch".to_string(),
                input_shape: vec![1, 224, 224, 3],
                output_shape: vec![1, 1000],
                size_bytes: 1_000_000,
                created_at: 12345,
            },
            access_policy: AccessPolicy::Public,
            usage_stats: UsageStats::default(),
        };

        state_manager
            .register_model(model_id, model_state, "QmTest".to_string())
            .unwrap();

        // Calculate new root - should be different
        let root2 = state_manager.calculate_state_root().await.unwrap();
        assert_ne!(root1, root2);

        // Root should be deterministic
        let root3 = state_manager.calculate_state_root().await.unwrap();
        assert_eq!(root2, root3);
    }
}
