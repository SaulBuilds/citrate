pub mod registry;
pub mod provider;
pub mod execution;
pub mod verification;
pub mod cache;
pub mod types;

use crate::types::{ModelId, ModelMetadata};
use lattice_execution::Address;
use std::sync::Arc;
use tracing::info;

/// MCP Service coordinator
pub struct MCPService {
    pub model_registry: Arc<registry::ModelRegistry>,
    pub provider_registry: Arc<provider::ProviderRegistry>,
    pub executor: Arc<execution::ModelExecutor>,
    pub verifier: Arc<verification::ExecutionVerifier>,
}

impl MCPService {
    pub fn new(
        storage: Arc<lattice_storage::StorageManager>,
        vm: Arc<lattice_execution::vm::VM>,
    ) -> Self {
        let model_registry = Arc::new(registry::ModelRegistry::new(storage.clone()));
        let provider_registry = Arc::new(provider::ProviderRegistry::new());
        let cache = Arc::new(cache::ModelCache::new(1024 * 1024 * 1024)); // 1GB cache
        let verifier = Arc::new(verification::ExecutionVerifier::new());
        let executor = Arc::new(execution::ModelExecutor::new(
            vm,
            cache,
            verifier.clone(),
        ));
        
        info!("MCP Service initialized");
        
        Self {
            model_registry,
            provider_registry,
            executor,
            verifier,
        }
    }
    
    /// Register a new AI model
    pub async fn register_model(
        &self,
        metadata: ModelMetadata,
        providers: Vec<Address>,
    ) -> anyhow::Result<ModelId> {
        self.model_registry.register(metadata, providers).await
    }
    
    /// Execute model inference
    pub async fn execute_inference(
        &self,
        model_id: ModelId,
        input: Vec<u8>,
        provider: Address,
    ) -> anyhow::Result<execution::InferenceResult> {
        self.executor.execute_inference(model_id, input, provider).await
    }
}
