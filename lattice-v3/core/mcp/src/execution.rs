use crate::cache::ModelCache;
use crate::registry::ModelRegistry;
use crate::types::{ExecutionProof, ModelId, ModelMetadata as RegistryModelMetadata};
use crate::verification::ExecutionVerifier;
use anyhow::{anyhow, Result};
use hex;
use lattice_execution::inference::{
    MetalModel, MetalModelFormat, MetalRuntime, ModelConfig, QuantizationType,
};
use lattice_execution::vm::VM;
use lattice_execution::{Address, Hash};
use lattice_storage::ipfs::{chunking, Cid, IPFSService};
use serde_json;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs;
use tokio::sync::Mutex;
use tracing::{debug, info, warn};

/// Result of model inference
#[derive(Debug, Clone)]
pub struct InferenceResult {
    pub output: Vec<u8>,
    pub proof: ExecutionProof,
    pub gas_used: u64,
    pub latency_ms: u64,
    pub provider: Address,
}

/// Model executor for running AI models
pub struct ModelExecutor {
    #[allow(dead_code)]
    vm: Arc<VM>,
    cache: Arc<ModelCache>,
    verifier: Arc<ExecutionVerifier>,
    registry: Arc<ModelRegistry>,
    ipfs: Mutex<IPFSService>,
    runtime: Mutex<MetalRuntime>,
    model_dir: PathBuf,
}

impl ModelExecutor {
    pub fn new(
        vm: Arc<VM>,
        cache: Arc<ModelCache>,
        verifier: Arc<ExecutionVerifier>,
        registry: Arc<ModelRegistry>,
        ipfs: IPFSService,
    ) -> Self {
        let runtime = MetalRuntime::new().unwrap_or_else(|err| {
            warn!(
                "Metal runtime initialization failed: {}. Falling back to CPU mode.",
                err
            );
            MetalRuntime::cpu_only()
        });
        let model_dir = std::env::temp_dir().join("lattice-model-cache");
        if let Err(err) = std::fs::create_dir_all(&model_dir) {
            warn!(
                "Failed to create local model cache directory {:?}: {}",
                model_dir, err
            );
        }

        Self {
            vm,
            cache,
            verifier,
            registry,
            ipfs: Mutex::new(ipfs),
            runtime: Mutex::new(runtime),
            model_dir,
        }
    }

    /// Execute model inference
    pub async fn execute_inference(
        &self,
        model_id: ModelId,
        input: Vec<u8>,
        provider: Address,
    ) -> Result<InferenceResult> {
        let start_time = std::time::Instant::now();

        // 1. Load model from cache or storage
        let model = self.load_model(model_id).await?;

        // 2. Verify model integrity
        self.verifier.verify_model(&model)?;

        // 3. Prepare execution context
        let context = self.prepare_context(&model, &input)?;

        // 4. Execute inference via runtime backend
        let (output, gas_used) = self.execute_in_runtime(&model, &context).await?;

        // 5. Generate execution proof
        let proof = self.generate_proof(&model, &input, &output, provider)?;

        let latency_ms = start_time.elapsed().as_millis() as u64;

        info!(
            "Inference completed for model {:?} in {}ms using {} gas",
            hex::encode(&model_id.0[..8]),
            latency_ms,
            gas_used
        );

        Ok(InferenceResult {
            output,
            proof,
            gas_used,
            latency_ms,
            provider,
        })
    }

    /// Execute training step
    pub async fn execute_training(
        &self,
        model_id: ModelId,
        training_data: Vec<u8>,
        current_weights: Vec<u8>,
        provider: Address,
    ) -> Result<TrainingResult> {
        let start_time = std::time::Instant::now();

        // 1. Load model architecture
        let model = self.load_model(model_id).await?;

        // 2. Prepare training context
        let context = self.prepare_training_context(&model, &training_data, &current_weights)?;

        // 3. Execute training step (CPU fallback)
        let (updated_weights, metrics, gas_used) =
            self.execute_training_step(&model, &context).await?;

        // 4. Generate training proof
        let proof = self.generate_training_proof(
            &model,
            &training_data,
            &current_weights,
            &updated_weights,
            provider,
        )?;

        let latency_ms = start_time.elapsed().as_millis() as u64;

        info!(
            "Training step completed for model {:?} in {}ms",
            hex::encode(&model_id.0[..8]),
            latency_ms
        );

        Ok(TrainingResult {
            updated_weights,
            metrics,
            proof,
            gas_used,
            latency_ms,
            provider,
        })
    }

    /// Load model from cache or storage
    async fn load_model(&self, model_id: ModelId) -> Result<Model> {
        if let Some(model) = self.cache.get(&model_id).await {
            debug!(
                "Model loaded from cache: {:?}",
                hex::encode(&model_id.0[..8])
            );
            return Ok(model);
        }

        let record = self.registry.get_record(&model_id).await?;
        let weight_cid = record.weight_cid.clone().ok_or_else(|| {
            anyhow!(
                "Model {:?} missing weight CID",
                hex::encode(&model_id.0[..8])
            )
        })?;

        let weights = {
            let ipfs = self.ipfs.lock().await;
            let cid = Cid(weight_cid.clone());
            let raw = ipfs.retrieve_model(&cid).await?;
            if let Ok(manifest) = serde_json::from_slice::<chunking::ChunkManifest>(&raw) {
                let mut assembled = Vec::with_capacity(manifest.total_size as usize);
                for chunk_cid in manifest.chunks {
                    match ipfs.fetch_raw(&chunk_cid).await {
                        Ok(bytes) => assembled.extend(bytes),
                        Err(err) => {
                            warn!("Failed to fetch chunk {}: {}", chunk_cid.0, err);
                            return Err(err);
                        }
                    }
                }
                assembled
            } else {
                raw
            }
        };

        let architecture_bytes = serde_json::to_vec(&record.metadata)?;
        let model = Model {
            id: model_id,
            architecture: architecture_bytes,
            weights,
            metadata: record.metadata.clone(),
        };

        self.cache.put(model_id, model.clone()).await?;

        Ok(model)
    }

    /// Prepare execution context
    fn prepare_context(&self, model: &Model, input: &[u8]) -> Result<ExecutionContext> {
        Ok(ExecutionContext {
            model_id: model.id,
            input: input.to_vec(),
            memory_limit: 1024 * 1024 * 100, // 100MB
            gas_limit: 10_000_000,
            execution_mode: ExecutionMode::Inference,
        })
    }

    /// Prepare training context
    fn prepare_training_context(
        &self,
        model: &Model,
        training_data: &[u8],
        weights: &[u8],
    ) -> Result<ExecutionContext> {
        Ok(ExecutionContext {
            model_id: model.id,
            input: training_data.to_vec(),
            memory_limit: 1024 * 1024 * 500, // 500MB for training
            gas_limit: 50_000_000,
            execution_mode: ExecutionMode::Training {
                current_weights: weights.to_vec(),
            },
        })
    }

    /// Execute inference using the configured runtime/backend.
    async fn execute_in_runtime(
        &self,
        model: &Model,
        context: &ExecutionContext,
    ) -> Result<(Vec<u8>, u64)> {
        self.run_inference_backend(model, &context.input).await
    }

    /// Execute a simplified training step (hash-based deterministic update for now).
    async fn execute_training_step(
        &self,
        model: &Model,
        context: &ExecutionContext,
    ) -> Result<(Vec<u8>, TrainingMetrics, u64)> {
        let updated_weights = Self::derive_updated_weights(model, &context.input);
        let weight_count = updated_weights.len().max(1);
        let gas_used = self.compute_gas_usage(
            context.input.len(),
            weight_count / std::mem::size_of::<f32>(),
            model.weights.len(),
        );
        let metrics = TrainingMetrics {
            loss: 1.0 / (weight_count as f64),
            accuracy: 0.5,
            epoch: 1,
        };
        Ok((updated_weights, metrics, gas_used))
    }

    async fn run_inference_backend(&self, model: &Model, input: &[u8]) -> Result<(Vec<u8>, u64)> {
        let weights_path = self.ensure_weights_file(model).await?;
        let float_input = Self::decode_input_to_f32(input);
        let input_elements = float_input.len().max(1);
        let output_elements = Self::estimate_output_len(model, input_elements);

        let runtime_model = MetalModel {
            id: hex::encode(model.id.0),
            name: model.metadata.name.clone(),
            format: MetalModelFormat::CoreML,
            weights: model.weights.clone(),
            weights_path,
            config: ModelConfig {
                input_shape: vec![input_elements],
                output_shape: vec![output_elements],
                batch_size: 1,
                max_sequence_length: None,
                quantization: QuantizationType::Float32,
                memory_required_mb: ((model.metadata.size.max(1) + 1_048_575) / 1_048_576) as u32,
            },
            metal_optimized: false,
            uses_neural_engine: false,
        };

        let mut runtime = self.runtime.lock().await;
        if !runtime.is_model_loaded(&runtime_model.id) {
            runtime.load_model(runtime_model.clone()).await?;
        }
        let outputs = runtime.infer(&runtime_model.id, &float_input).await?;
        drop(runtime);

        let mut output_bytes = Vec::with_capacity(outputs.len() * std::mem::size_of::<f32>());
        for value in &outputs {
            output_bytes.extend_from_slice(&value.to_le_bytes());
        }

        let gas_used =
            self.compute_gas_usage(input.len(), outputs.len().max(1), model.weights.len());

        Ok((output_bytes, gas_used))
    }

    async fn ensure_weights_file(&self, model: &Model) -> Result<PathBuf> {
        let filename = format!("{}.bin", hex::encode(model.id.0));
        let filepath = self.model_dir.join(filename);

        let should_write = match fs::metadata(&filepath).await {
            Ok(meta) => meta.len() != model.weights.len() as u64,
            Err(_) => true,
        };

        if should_write {
            fs::write(&filepath, &model.weights).await?;
        }

        Ok(filepath)
    }

    fn decode_input_to_f32(input: &[u8]) -> Vec<f32> {
        if input.is_empty() {
            return vec![0.0];
        }

        if let Ok(json_values) = serde_json::from_slice::<Vec<f32>>(input) {
            if !json_values.is_empty() {
                return json_values;
            }
        }

        let mut floats = Vec::with_capacity((input.len() + 3) / 4);
        for chunk in input.chunks(4) {
            let mut bytes = [0u8; 4];
            for (idx, byte) in chunk.iter().enumerate() {
                bytes[idx] = *byte;
            }
            floats.push(f32::from_le_bytes(bytes));
        }

        if floats.is_empty() {
            floats.push(0.0);
        }

        floats
    }

    fn estimate_output_len(model: &Model, input_elements: usize) -> usize {
        let weight_factor = (model.weights.len() / 64).max(1);
        let input_factor = (input_elements / 8).max(1);
        let combined = weight_factor.saturating_add(input_factor);
        combined.clamp(1, 256)
    }

    fn compute_gas_usage(
        &self,
        input_bytes: usize,
        output_elements: usize,
        weight_bytes: usize,
    ) -> u64 {
        let base = 500_000u64;
        let input_component = ((input_bytes as u64 + 255) / 256) * 2_000;
        let output_component = (output_elements as u64) * 150;
        let weight_component = ((weight_bytes as u64 + 1_023) / 1_024) * 5_000;
        base + input_component + output_component + weight_component
    }

    fn derive_updated_weights(model: &Model, training_input: &[u8]) -> Vec<u8> {
        use sha3::{Digest, Sha3_256};

        let mut hasher = Sha3_256::new();
        hasher.update(&model.weights);
        hasher.update(training_input);
        hasher.update(model.metadata.name.as_bytes());
        let digest = hasher.finalize();

        // Repeat digest to craft new weight blob
        let mut updated = Vec::with_capacity(model.weights.len().max(digest.len()));
        while updated.len() < model.weights.len() {
            updated.extend_from_slice(digest.as_slice());
        }
        updated.truncate(model.weights.len());
        if updated.is_empty() {
            updated.extend_from_slice(digest.as_slice());
        }

        updated
    }

    /// Generate execution proof
    fn generate_proof(
        &self,
        model: &Model,
        input: &[u8],
        output: &[u8],
        provider: Address,
    ) -> Result<ExecutionProof> {
        use sha3::{Digest, Sha3_256};

        // Hash model
        let model_hash = {
            let mut hasher = Sha3_256::new();
            hasher.update(&model.architecture);
            hasher.update(&model.weights);
            Hash::new(hasher.finalize().into())
        };

        // Hash input/output
        let input_hash = {
            let mut hasher = Sha3_256::new();
            hasher.update(input);
            Hash::new(hasher.finalize().into())
        };

        let output_hash = {
            let mut hasher = Sha3_256::new();
            hasher.update(output);
            Hash::new(hasher.finalize().into())
        };

        // Create IO commitment
        let io_commitment = {
            let mut hasher = Sha3_256::new();
            hasher.update(input_hash.as_bytes());
            hasher.update(output_hash.as_bytes());
            Hash::new(hasher.finalize().into())
        };

        Ok(ExecutionProof {
            model_hash,
            input_hash,
            output_hash,
            io_commitment,
            statement: vec![],  // Placeholder for ZK statement
            proof_data: vec![], // Placeholder for ZK proof
            timestamp: chrono::Utc::now().timestamp() as u64,
            provider,
        })
    }

    /// Generate training proof
    fn generate_training_proof(
        &self,
        model: &Model,
        training_data: &[u8],
        _current_weights: &[u8],
        updated_weights: &[u8],
        provider: Address,
    ) -> Result<ExecutionProof> {
        // Similar to generate_proof but for training
        self.generate_proof(model, training_data, updated_weights, provider)
    }
}

/// Model representation
#[derive(Debug, Clone)]
pub struct Model {
    pub id: ModelId,
    pub architecture: Vec<u8>,
    pub weights: Vec<u8>,
    pub metadata: RegistryModelMetadata,
}

/// Execution context
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct ExecutionContext {
    model_id: ModelId,
    input: Vec<u8>,
    memory_limit: u64,
    gas_limit: u64,
    execution_mode: ExecutionMode,
}

/// Execution mode
#[derive(Debug, Clone)]
#[allow(dead_code)]
enum ExecutionMode {
    Inference,
    Training { current_weights: Vec<u8> },
}

/// Training result
#[derive(Debug, Clone)]
pub struct TrainingResult {
    pub updated_weights: Vec<u8>,
    pub metrics: TrainingMetrics,
    pub proof: ExecutionProof,
    pub gas_used: u64,
    pub latency_ms: u64,
    pub provider: Address,
}

/// Training metrics
#[derive(Debug, Clone)]
pub struct TrainingMetrics {
    pub loss: f64,
    pub accuracy: f64,
    pub epoch: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ComputeRequirements, Currency, PricingModel};
    use lattice_execution::Hash;
    use lattice_execution::vm::VM;
    use lattice_storage::ipfs::IPFSService;
    use lattice_storage::pruning::PruningConfig;
    use lattice_storage::StorageManager;
    use primitive_types::U256;
    use std::sync::Arc;
    use tempfile::TempDir;

    fn sample_metadata() -> RegistryModelMetadata {
        RegistryModelMetadata {
            id: ModelId([0u8; 32]),
            owner: Address::zero(),
            name: "demo-model".to_string(),
            version: "1.0.0".to_string(),
            hash: Hash::default(),
            size: 4_096,
            compute_requirements: ComputeRequirements {
                min_memory: 1_048_576,
                min_compute: 1,
                gpu_required: false,
                supported_hardware: vec![],
            },
            pricing: PricingModel {
                base_price: U256::zero(),
                per_token_price: U256::zero(),
                per_second_price: U256::zero(),
                currency: Currency::LAT,
            },
        }
    }

    fn sample_model(weight_len: usize) -> Model {
        let mut weights = vec![0u8; weight_len];
        if weight_len == 0 {
            weights.push(1);
        }
        Model {
            id: ModelId([0u8; 32]),
            architecture: vec![1, 2, 3],
            weights,
            metadata: sample_metadata(),
        }
    }

    #[test]
    fn decode_input_from_json() {
        let payload = serde_json::to_vec(&vec![1.0_f32, 2.5, 3.75]).unwrap();
        let floats = ModelExecutor::decode_input_to_f32(&payload);
        assert_eq!(floats.len(), 3);
        assert!((floats[1] - 2.5).abs() < f32::EPSILON);
    }

    #[test]
    fn decode_input_from_bytes() {
        let bytes = vec![0u8, 0, 128, 63]; // 1.0f32
        let floats = ModelExecutor::decode_input_to_f32(&bytes);
        assert_eq!(floats.len(), 1);
        assert!((floats[0] - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn estimate_output_len_scales_with_weights() {
        let small_model = sample_model(64);
        let large_model = sample_model(2048);

        let small_len = ModelExecutor::estimate_output_len(&small_model, 8);
        let large_len = ModelExecutor::estimate_output_len(&large_model, 8);

        assert!(large_len >= small_len);
        assert!(large_len <= 256);
    }

    #[test]
    fn derive_updated_weights_changes_payload() {
        let model = sample_model(128);
        let original = model.weights.clone();
        let updated = ModelExecutor::derive_updated_weights(&model, b"training-data");
        assert_eq!(updated.len(), original.len());
        assert_ne!(updated, original);
    }

    #[tokio::test]
    async fn execute_inference_uses_runtime_pipeline() {
        let tmp = TempDir::new().unwrap();
        let storage =
            Arc::new(StorageManager::new(tmp.path(), PruningConfig::default()).unwrap());
        let registry = Arc::new(ModelRegistry::new(storage));
        let cache = Arc::new(ModelCache::new(1 << 20));
        let vm = Arc::new(VM::new(10_000_000));
        let verifier = Arc::new(ExecutionVerifier::new());
        let ipfs = IPFSService::new("http://127.0.0.1:5001".to_string());
        let executor = ModelExecutor::new(vm, cache.clone(), verifier, registry, ipfs);

        let mut model = sample_model(128);
        model.id = ModelId([0x42; 32]);
        model.metadata.id = model.id;
        model.metadata.hash = Hash::new([0x24; 32]);

        cache.put(model.id, model.clone()).await.unwrap();

        let provider = Address([0xAA; 20]);
        let input = vec![1u8, 2, 3, 4, 5, 6, 7, 8];
        let result = executor
            .execute_inference(model.id, input.clone(), provider)
            .await
            .unwrap();

        assert_eq!(result.provider, provider);
        assert!(!result.output.is_empty());
        assert!(result.gas_used > 0);
        assert_eq!(result.output.len() % 4, 0); // Should be f32-aligned bytes
    }
}
