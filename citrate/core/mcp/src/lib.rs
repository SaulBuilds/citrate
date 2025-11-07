// citrate/core/mcp/src/lib.rs

// MCP Service coordinator
pub mod cache;
pub mod execution;
pub mod gguf_engine;
pub mod provider;
pub mod registry;
pub mod types;
pub mod verification;

use crate::types::{ModelId, ModelMetadata};
use citrate_execution::Address;
use citrate_storage::ipfs::IPFSService;
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
        storage: Arc<citrate_storage::StorageManager>,
        vm: Arc<citrate_execution::vm::VM>,
    ) -> Self {
        let model_registry = Arc::new(registry::ModelRegistry::new(storage.clone()));
        let provider_registry = Arc::new(provider::ProviderRegistry::new());
        let cache = Arc::new(cache::ModelCache::new(1024 * 1024 * 1024)); // 1GB cache
        let verifier = Arc::new(verification::ExecutionVerifier::new());
        let ipfs_endpoint = std::env::var("CITRATE_IPFS_API")
            .unwrap_or_else(|_| "http://127.0.0.1:5001".to_string());
        let ipfs_service = IPFSService::new(ipfs_endpoint);
        let executor = Arc::new(execution::ModelExecutor::new(
            vm,
            cache,
            verifier.clone(),
            model_registry.clone(),
            ipfs_service,
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
        weight_cid: Option<String>,
    ) -> anyhow::Result<ModelId> {
        self.model_registry
            .register(metadata, providers, weight_cid)
            .await
    }

    pub async fn update_model_weight(
        &self,
        model_id: ModelId,
        weight_cid: String,
    ) -> anyhow::Result<()> {
        self.model_registry
            .update_weight(&model_id, weight_cid)
            .await
    }

    /// Execute model inference
    pub async fn execute_inference(
        &self,
        model_id: ModelId,
        input: Vec<u8>,
        provider: Address,
    ) -> anyhow::Result<execution::InferenceResult> {
        self.executor
            .execute_inference(model_id, input, provider)
            .await
    }
}
