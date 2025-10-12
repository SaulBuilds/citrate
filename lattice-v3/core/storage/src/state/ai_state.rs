// lattice-v3/core/storage/src/state/ai_state.rs

use anyhow::Result;
use lattice_consensus::types::Hash;
use lattice_execution::{Address, JobId, ModelId, ModelState, TrainingJob};
use serde::{Deserialize, Serialize};
use sha3::{Digest, Sha3_256};
use std::collections::HashMap;

/// AI-specific state tree for managing model and training state
#[derive(Debug, Clone)]
pub struct AIStateTree {
    /// Registered models indexed by ID
    pub models: HashMap<ModelId, ModelState>,

    /// Active training jobs
    pub training_jobs: HashMap<JobId, TrainingJob>,

    /// Model weight CIDs for IPFS storage
    pub model_weights: HashMap<ModelId, String>,

    /// Inference cache for recent results
    pub inference_cache: HashMap<Hash, InferenceResult>,

    /// LoRA adapters indexed by base model
    pub lora_adapters: HashMap<ModelId, Vec<LoRAAdapter>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceResult {
    pub model_id: ModelId,
    pub input_hash: Hash,
    pub output: Vec<u8>,
    pub gas_used: u64,
    pub timestamp: u64,
    pub proof: Option<Vec<u8>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoRAAdapter {
    pub adapter_id: Hash,
    pub base_model: ModelId,
    pub owner: Address,
    pub weight_cid: String,
    pub rank: u32,
    pub alpha: f32,
    pub created_at: u64,
}

impl AIStateTree {
    pub fn new() -> Self {
        Self {
            models: HashMap::new(),
            training_jobs: HashMap::new(),
            model_weights: HashMap::new(),
            inference_cache: HashMap::new(),
            lora_adapters: HashMap::new(),
        }
    }

    /// Calculate the root hash of all AI state
    pub fn calculate_root(&self) -> Result<Hash> {
        let model_root = self.calculate_model_root()?;
        let training_root = self.calculate_training_root()?;
        let inference_root = self.calculate_inference_root()?;
        let lora_root = self.calculate_lora_root()?;

        // Combine all roots into a single AI state root
        let mut hasher = Sha3_256::new();
        hasher.update(model_root.as_bytes());
        hasher.update(training_root.as_bytes());
        hasher.update(inference_root.as_bytes());
        hasher.update(lora_root.as_bytes());

        let hash_bytes = hasher.finalize();
        let mut hash_array = [0u8; 32];
        hash_array.copy_from_slice(&hash_bytes[..32]);
        Ok(Hash::new(hash_array))
    }

    /// Calculate model registry root
    pub fn calculate_model_root(&self) -> Result<Hash> {
        if self.models.is_empty() {
            return Ok(Hash::default());
        }

        // Sort models by ID for deterministic ordering
        let mut model_entries: Vec<_> = self.models.iter().collect();
        model_entries.sort_by(|a, b| a.0 .0.as_bytes().cmp(b.0 .0.as_bytes()));

        let mut hasher = Sha3_256::new();
        for (model_id, model_state) in model_entries {
            // Hash model ID
            hasher.update(model_id.0.as_bytes());

            // Hash model state fields
            hasher.update(model_state.owner.0);
            hasher.update(model_state.model_hash.as_bytes());
            hasher.update(model_state.version.to_le_bytes());

            // Hash metadata
            hasher.update(model_state.metadata.name.as_bytes());
            hasher.update(model_state.metadata.version.as_bytes());
            hasher.update(model_state.metadata.size_bytes.to_le_bytes());

            // Hash usage stats
            hasher.update(model_state.usage_stats.total_inferences.to_le_bytes());
            hasher.update(model_state.usage_stats.total_gas_used.to_le_bytes());
        }

        let hash_bytes = hasher.finalize();
        let mut hash_array = [0u8; 32];
        hash_array.copy_from_slice(&hash_bytes[..32]);
        Ok(Hash::new(hash_array))
    }

    /// Calculate training jobs root
    pub fn calculate_training_root(&self) -> Result<Hash> {
        if self.training_jobs.is_empty() {
            return Ok(Hash::default());
        }

        let mut job_entries: Vec<_> = self.training_jobs.iter().collect();
        job_entries.sort_by(|a, b| a.0 .0.as_bytes().cmp(b.0 .0.as_bytes()));

        let mut hasher = Sha3_256::new();
        for (job_id, job) in job_entries {
            hasher.update(job_id.0.as_bytes());
            hasher.update(job.model_id.0.as_bytes());
            hasher.update(job.owner.0);
            hasher.update(job.dataset_hash.as_bytes());
            hasher.update(job.gradients_submitted.to_le_bytes());
            hasher.update(job.gradients_required.to_le_bytes());

            // Hash participants
            for participant in &job.participants {
                hasher.update(participant.0);
            }

            // Hash status
            let status_byte = match job.status {
                lattice_execution::JobStatus::Pending => 0u8,
                lattice_execution::JobStatus::Active => 1u8,
                lattice_execution::JobStatus::Completed => 2u8,
                lattice_execution::JobStatus::Failed => 3u8,
                lattice_execution::JobStatus::Cancelled => 4u8,
            };
            hasher.update([status_byte]);
        }

        let hash_bytes = hasher.finalize();
        let mut hash_array = [0u8; 32];
        hash_array.copy_from_slice(&hash_bytes[..32]);
        Ok(Hash::new(hash_array))
    }

    /// Calculate inference cache root
    pub fn calculate_inference_root(&self) -> Result<Hash> {
        if self.inference_cache.is_empty() {
            return Ok(Hash::default());
        }

        let mut cache_entries: Vec<_> = self.inference_cache.iter().collect();
        cache_entries.sort_by(|a, b| a.0.as_bytes().cmp(b.0.as_bytes()));

        let mut hasher = Sha3_256::new();
        for (cache_key, result) in cache_entries {
            hasher.update(cache_key.as_bytes());
            hasher.update(result.model_id.0.as_bytes());
            hasher.update(result.input_hash.as_bytes());
            hasher.update(&result.output);
            hasher.update(result.gas_used.to_le_bytes());
            hasher.update(result.timestamp.to_le_bytes());

            if let Some(proof) = &result.proof {
                hasher.update(proof);
            }
        }

        let hash_bytes = hasher.finalize();
        let mut hash_array = [0u8; 32];
        hash_array.copy_from_slice(&hash_bytes[..32]);
        Ok(Hash::new(hash_array))
    }

    /// Calculate LoRA adapters root
    pub fn calculate_lora_root(&self) -> Result<Hash> {
        if self.lora_adapters.is_empty() {
            return Ok(Hash::default());
        }

        let mut lora_entries: Vec<_> = self.lora_adapters.iter().collect();
        lora_entries.sort_by(|a, b| a.0 .0.as_bytes().cmp(b.0 .0.as_bytes()));

        let mut hasher = Sha3_256::new();
        for (base_model, adapters) in lora_entries {
            hasher.update(base_model.0.as_bytes());

            for adapter in adapters {
                hasher.update(adapter.adapter_id.as_bytes());
                hasher.update(adapter.owner.0);
                hasher.update(adapter.weight_cid.as_bytes());
                hasher.update(adapter.rank.to_le_bytes());
                hasher.update(adapter.alpha.to_le_bytes());
                hasher.update(adapter.created_at.to_le_bytes());
            }
        }

        let hash_bytes = hasher.finalize();
        let mut hash_array = [0u8; 32];
        hash_array.copy_from_slice(&hash_bytes[..32]);
        Ok(Hash::new(hash_array))
    }

    /// Register a new model
    pub fn register_model(
        &mut self,
        model_id: ModelId,
        model_state: ModelState,
        weight_cid: String,
    ) {
        self.models.insert(model_id, model_state);
        self.model_weights.insert(model_id, weight_cid);
    }

    /// Update model weights
    pub fn update_model_weights(&mut self, model_id: ModelId, new_cid: String, new_version: u32) {
        if let Some(model) = self.models.get_mut(&model_id) {
            model.version = new_version;
            self.model_weights.insert(model_id, new_cid);
        }
    }

    /// Add inference result to cache
    pub fn cache_inference(&mut self, result: InferenceResult) {
        let cache_key = self.compute_inference_key(&result.model_id, &result.input_hash);
        self.inference_cache.insert(cache_key, result);
    }

    /// Add LoRA adapter
    pub fn add_lora_adapter(&mut self, adapter: LoRAAdapter) {
        let base_model = adapter.base_model;
        self.lora_adapters
            .entry(base_model)
            .or_default()
            .push(adapter);
    }

    /// Compute cache key for inference results
    fn compute_inference_key(&self, model_id: &ModelId, input_hash: &Hash) -> Hash {
        let mut hasher = Sha3_256::new();
        hasher.update(model_id.0.as_bytes());
        hasher.update(input_hash.as_bytes());

        let hash_bytes = hasher.finalize();
        let mut hash_array = [0u8; 32];
        hash_array.copy_from_slice(&hash_bytes[..32]);
        Hash::new(hash_array)
    }

    /// Prune old inference cache entries
    pub fn prune_inference_cache(&mut self, max_age: u64, current_time: u64) {
        self.inference_cache
            .retain(|_, result| current_time - result.timestamp < max_age);
    }
}

impl Default for AIStateTree {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lattice_execution::{AccessPolicy, ModelMetadata, UsageStats};

    #[test]
    fn test_ai_state_root_calculation() {
        let mut ai_state = AIStateTree::new();

        // Add a test model
        let model_id = ModelId(Hash::new([1; 32]));
        let model_state = ModelState {
            owner: Address([2; 20]),
            model_hash: Hash::new([3; 32]),
            version: 1,
            metadata: ModelMetadata {
                name: "TestModel".to_string(),
                version: "1.0".to_string(),
                description: "Test model".to_string(),
                framework: "PyTorch".to_string(),
                input_shape: vec![1, 224, 224, 3],
                output_shape: vec![1, 1000],
                size_bytes: 1_000_000,
                created_at: 12345,
            },
            access_policy: AccessPolicy::Public,
            usage_stats: UsageStats::default(),
        };

        ai_state.register_model(model_id, model_state, "QmTest123".to_string());

        // Calculate root
        let root = ai_state.calculate_root().unwrap();
        assert_ne!(root, Hash::default());

        // Root should be deterministic
        let root2 = ai_state.calculate_root().unwrap();
        assert_eq!(root, root2);
    }

    #[test]
    fn test_lora_adapter_management() {
        let mut ai_state = AIStateTree::new();

        let base_model = ModelId(Hash::new([1; 32]));
        let adapter = LoRAAdapter {
            adapter_id: Hash::new([2; 32]),
            base_model,
            owner: Address([3; 20]),
            weight_cid: "QmLoRA456".to_string(),
            rank: 16,
            alpha: 32.0,
            created_at: 12345,
        };

        ai_state.add_lora_adapter(adapter.clone());

        assert!(ai_state.lora_adapters.contains_key(&base_model));
        assert_eq!(ai_state.lora_adapters[&base_model].len(), 1);

        // Calculate LoRA root
        let root = ai_state.calculate_lora_root().unwrap();
        assert_ne!(root, Hash::default());
    }
}
