use async_trait::async_trait;
use citrate_execution::{executor::InferenceService, Address, ModelId};
use citrate_mcp::MCPService;
use primitive_types::U256;
use std::sync::Arc;

/// Simple MCP-backed inference service that executes locally via MCP's ModelExecutor
pub struct NodeInferenceService {
    mcp: Arc<MCPService>,
    provider: Address,
    provider_fee_wei: U256,
}

impl NodeInferenceService {
    pub fn new(mcp: Arc<MCPService>, provider: Address, provider_fee_wei: U256) -> Self {
        Self {
            mcp,
            provider,
            provider_fee_wei,
        }
    }
}

#[async_trait]
impl InferenceService for NodeInferenceService {
    async fn run_inference(
        &self,
        model_id: ModelId,
        input: Vec<u8>,
        _max_gas: u64,
    ) -> Result<(Vec<u8>, u64, Address, U256, Option<Vec<u8>>), citrate_execution::ExecutionError>
    {
        // Convert execution ModelId(Hash) to MCP ModelId([u8;32])
        let mcp_model_id = citrate_mcp::types::ModelId::from_hash(&model_id.0);
        let result = self
            .mcp
            .executor
            .execute_inference(mcp_model_id, input, self.provider)
            .await
            .map_err(|e| citrate_execution::ExecutionError::Reverted(e.to_string()))?;

        let proof_bytes = serde_json::to_vec(&serde_json::json!({
            "model_hash": hex::encode(result.proof.model_hash.as_bytes()),
            "input_hash": hex::encode(result.proof.input_hash.as_bytes()),
            "output_hash": hex::encode(result.proof.output_hash.as_bytes()),
            "io_commitment": hex::encode(result.proof.io_commitment.as_bytes()),
            "provider": hex::encode(self.provider.0),
            "timestamp": result.proof.timestamp,
        }))
        .ok();

        Ok((
            result.output,
            result.gas_used,
            self.provider,
            self.provider_fee_wei,
            proof_bytes,
        ))
    }
}
