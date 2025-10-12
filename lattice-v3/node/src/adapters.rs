use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use lattice_execution::executor::{AIModelStorage, ModelRegistryAdapter};
use lattice_execution::{ModelId, ModelState};
use lattice_mcp::{
    types::{ComputeRequirements, Currency, ModelMetadata, PricingModel},
    MCPService,
};
use lattice_storage::StorageManager;

/// Bridge that persists model metadata & artifacts via `StorageManager`.
pub struct StorageAdapter {
    storage: Arc<StorageManager>,
}

impl StorageAdapter {
    pub fn new(storage: Arc<StorageManager>) -> Self {
        Self { storage }
    }
}

impl AIModelStorage for StorageAdapter {
    fn register_model(
        &self,
        model_id: ModelId,
        model_state: &ModelState,
        weight_cid: &str,
    ) -> Result<()> {
        self.storage
            .register_model(model_id, model_state.clone(), weight_cid.to_string())
    }

    fn update_model_weights(
        &self,
        model_id: ModelId,
        weight_cid: &str,
        new_version: u32,
    ) -> Result<()> {
        self.storage
            .update_model_weights(model_id, weight_cid.to_string(), new_version)
    }
}

/// Bridge that keeps the MCP model registry in sync with execution-layer events.
pub struct MCPRegistryBridge {
    mcp: Arc<MCPService>,
}

impl MCPRegistryBridge {
    pub fn new(mcp: Arc<MCPService>) -> Self {
        Self { mcp }
    }

    fn to_mcp_metadata(&self, _model_id: ModelId, model_state: &ModelState) -> ModelMetadata {
        ModelMetadata {
            id: lattice_mcp::types::ModelId::from_hash(&model_state.model_hash),
            owner: model_state.owner,
            name: model_state.metadata.name.clone(),
            version: model_state.metadata.version.clone(),
            hash: model_state.model_hash,
            size: model_state.metadata.size_bytes,
            compute_requirements: ComputeRequirements {
                min_memory: model_state.metadata.size_bytes.max(1),
                min_compute: 1,
                gpu_required: false,
                supported_hardware: vec![],
            },
            pricing: PricingModel {
                base_price: Default::default(),
                per_token_price: Default::default(),
                per_second_price: Default::default(),
                currency: Currency::LAT,
            },
        }
    }
}

#[async_trait]
impl ModelRegistryAdapter for MCPRegistryBridge {
    async fn register_model(
        &self,
        model_id: ModelId,
        model_state: &ModelState,
        artifact_cid: Option<&str>,
    ) -> Result<()> {
        let metadata = self.to_mcp_metadata(model_id, model_state);
        let providers = vec![model_state.owner];
        self.mcp
            .register_model(metadata, providers, artifact_cid.map(|s| s.to_string()))
            .await?;
        Ok(())
    }

    async fn update_model(
        &self,
        model_id: ModelId,
        _model_state: &ModelState,
        artifact_cid: Option<&str>,
    ) -> Result<()> {
        if let Some(cid) = artifact_cid {
            self.mcp
                .update_model_weight(model_id, cid.to_string())
                .await?;
        }
        Ok(())
    }
}
