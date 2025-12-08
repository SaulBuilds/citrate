//! AI model tools - real implementations
//!
//! These tools provide model listing, inference, and deployment using ModelManager.

use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use super::super::dispatcher::{DispatchError, ToolHandler, ToolOutput};
use super::super::intent::IntentParams;
use crate::models::{ModelManager, ModelType};
use crate::wallet::WalletManager;

/// List available models tool
pub struct ListModelsTool {
    model_manager: Arc<ModelManager>,
}

impl ListModelsTool {
    pub fn new(model_manager: Arc<ModelManager>) -> Self {
        Self { model_manager }
    }
}

impl ToolHandler for ListModelsTool {
    fn name(&self) -> &str {
        "list_models"
    }

    fn description(&self) -> &str {
        "List available AI models (local GGUF files and on-chain registered models)"
    }

    fn execute(
        &self,
        _params: &IntentParams,
    ) -> Pin<Box<dyn Future<Output = Result<ToolOutput, DispatchError>> + Send + '_>> {
        let model_manager = self.model_manager.clone();
        Box::pin(async move {
            match model_manager.get_models().await {
                Ok(models) => {
                    let model_list: Vec<serde_json::Value> = models
                        .iter()
                        .map(|m| {
                            serde_json::json!({
                                "id": m.id,
                                "name": m.name,
                                "type": format!("{:?}", m.model_type),
                                "description": m.description,
                                "size_mb": m.size_mb,
                                "parameters": m.parameters,
                                "architecture": m.architecture,
                                "owner": m.owner,
                            })
                        })
                        .collect();

                    let count = model_list.len();

                    // Categorize by type
                    let language = models
                        .iter()
                        .filter(|m| matches!(m.model_type, ModelType::Language))
                        .count();
                    let image = models
                        .iter()
                        .filter(|m| matches!(m.model_type, ModelType::Image))
                        .count();
                    let other = count - language - image;

                    Ok(ToolOutput {
                        tool: "list_models".to_string(),
                        success: true,
                        message: format!(
                            "Found {} models: {} language, {} image, {} other",
                            count, language, image, other
                        ),
                        data: Some(serde_json::json!({
                            "count": count,
                            "models": model_list,
                            "by_type": {
                                "language": language,
                                "image": image,
                                "other": other
                            }
                        })),
                    })
                }
                Err(e) => Ok(ToolOutput {
                    tool: "list_models".to_string(),
                    success: false,
                    message: format!("Failed to list models: {}", e),
                    data: None,
                }),
            }
        })
    }
}

/// Run inference tool
pub struct RunInferenceTool {
    model_manager: Arc<ModelManager>,
}

impl RunInferenceTool {
    pub fn new(model_manager: Arc<ModelManager>) -> Self {
        Self { model_manager }
    }
}

impl ToolHandler for RunInferenceTool {
    fn name(&self) -> &str {
        "run_inference"
    }

    fn description(&self) -> &str {
        "Run inference on an AI model. Provide model name and prompt."
    }

    fn execute(
        &self,
        params: &IntentParams,
    ) -> Pin<Box<dyn Future<Output = Result<ToolOutput, DispatchError>> + Send + '_>> {
        let model_manager = self.model_manager.clone();
        let model_name = params.model_name.clone();
        let prompt = params.prompt.clone();
        Box::pin(async move {
            let model = model_name.unwrap_or_else(|| "gpt-citrate".to_string());
            let input = prompt.ok_or_else(|| {
                DispatchError::InvalidParams("Prompt required for inference".to_string())
            })?;

            // Create inference request
            let request = crate::models::InferenceRequest {
                model_id: model.clone(),
                input: input.clone(),
                parameters: HashMap::new(),
            };

            match model_manager.request_inference(request).await {
                Ok(response) => Ok(ToolOutput {
                    tool: "run_inference".to_string(),
                    success: true,
                    message: format!(
                        "Model '{}' inference completed in {}ms. Confidence: {:.2}%",
                        model, response.latency_ms, response.confidence * 100.0
                    ),
                    data: Some(serde_json::json!({
                        "model": model,
                        "input": input,
                        "output": response.result,
                        "latency_ms": response.latency_ms,
                        "confidence": response.confidence,
                        "cost": response.cost,
                        "request_id": response.request_id,
                    })),
                }),
                Err(e) => Ok(ToolOutput {
                    tool: "run_inference".to_string(),
                    success: false,
                    message: format!("Inference failed: {}", e),
                    data: Some(serde_json::json!({
                        "model": model,
                        "input": input,
                        "error": e.to_string()
                    })),
                }),
            }
        })
    }
}

/// Deploy model tool - register model on-chain
pub struct DeployModelTool {
    model_manager: Arc<ModelManager>,
    wallet_manager: Arc<WalletManager>,
}

impl DeployModelTool {
    pub fn new(model_manager: Arc<ModelManager>, wallet_manager: Arc<WalletManager>) -> Self {
        Self {
            model_manager,
            wallet_manager,
        }
    }
}

impl ToolHandler for DeployModelTool {
    fn name(&self) -> &str {
        "deploy_model"
    }

    fn description(&self) -> &str {
        "Deploy/register an AI model to the blockchain registry. Requires confirmation."
    }

    fn execute(
        &self,
        params: &IntentParams,
    ) -> Pin<Box<dyn Future<Output = Result<ToolOutput, DispatchError>> + Send + '_>> {
        let model_manager = self.model_manager.clone();
        let wallet_manager = self.wallet_manager.clone();
        let model_name = params.model_name.clone();
        Box::pin(async move {
            let name = model_name.ok_or_else(|| {
                DispatchError::InvalidParams("Model name required".to_string())
            })?;

            // Get owner address
            let accounts = wallet_manager.get_accounts().await;
            if accounts.is_empty() {
                return Ok(ToolOutput {
                    tool: "deploy_model".to_string(),
                    success: false,
                    message: "No wallet accounts found. Create a wallet first.".to_string(),
                    data: None,
                });
            }
            let owner = accounts[0].address.clone();

            // Create deployment request
            let deployment = crate::models::ModelDeployment {
                id: format!("deploy_{}", chrono::Utc::now().timestamp()),
                model_id: name.clone(),
                endpoint: format!("http://localhost:8081/models/{}", name),
                status: crate::models::DeploymentStatus::Pending,
                replicas: 1,
                memory_mb: 4096,
                cpu_cores: 2,
                gpu_count: 0,
                created_at: chrono::Utc::now().timestamp() as u64,
            };

            match model_manager.deploy_model(deployment).await {
                Ok(deployment_id) => Ok(ToolOutput {
                    tool: "deploy_model".to_string(),
                    success: true,
                    message: format!(
                        "Model '{}' deployment initiated. ID: {}",
                        name, deployment_id
                    ),
                    data: Some(serde_json::json!({
                        "model_name": name,
                        "deployment_id": deployment_id,
                        "owner": owner,
                        "status": "deploying"
                    })),
                }),
                Err(e) => Ok(ToolOutput {
                    tool: "deploy_model".to_string(),
                    success: false,
                    message: format!("Model deployment failed: {}", e),
                    data: None,
                }),
            }
        })
    }

    fn requires_confirmation(&self) -> bool {
        true
    }
}

/// Get model info tool
pub struct GetModelInfoTool {
    model_manager: Arc<ModelManager>,
}

impl GetModelInfoTool {
    pub fn new(model_manager: Arc<ModelManager>) -> Self {
        Self { model_manager }
    }
}

impl ToolHandler for GetModelInfoTool {
    fn name(&self) -> &str {
        "get_model_info"
    }

    fn description(&self) -> &str {
        "Get detailed information about a specific AI model"
    }

    fn execute(
        &self,
        params: &IntentParams,
    ) -> Pin<Box<dyn Future<Output = Result<ToolOutput, DispatchError>> + Send + '_>> {
        let model_manager = self.model_manager.clone();
        let model_name = params.model_name.clone();
        Box::pin(async move {
            let name = model_name.ok_or_else(|| {
                DispatchError::InvalidParams("Model name or ID required".to_string())
            })?;

            match model_manager.get_model(&name).await {
                Ok(Some(model)) => Ok(ToolOutput {
                    tool: "get_model_info".to_string(),
                    success: true,
                    message: format!(
                        "Model '{}': {:?} ({} MB, {} parameters)",
                        model.name, model.model_type, model.size_mb, model.parameters
                    ),
                    data: Some(serde_json::json!({
                        "id": model.id,
                        "name": model.name,
                        "type": format!("{:?}", model.model_type),
                        "description": model.description,
                        "version": model.version,
                        "size_mb": model.size_mb,
                        "parameters": model.parameters,
                        "architecture": model.architecture,
                        "owner": model.owner,
                        "created_at": model.created_at,
                        "hash": model.hash,
                    })),
                }),
                Ok(None) => Ok(ToolOutput {
                    tool: "get_model_info".to_string(),
                    success: false,
                    message: format!("Model '{}' not found", name),
                    data: None,
                }),
                Err(e) => Ok(ToolOutput {
                    tool: "get_model_info".to_string(),
                    success: false,
                    message: format!("Failed to get model info: {}", e),
                    data: None,
                }),
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_models_tool_name() {
        let model_manager = Arc::new(ModelManager::new());
        let tool = ListModelsTool::new(model_manager);
        assert_eq!(tool.name(), "list_models");
    }

    #[test]
    fn test_list_models_tool_description() {
        let model_manager = Arc::new(ModelManager::new());
        let tool = ListModelsTool::new(model_manager);
        assert!(tool.description().contains("List"));
    }

    #[test]
    fn test_run_inference_tool_name() {
        let model_manager = Arc::new(ModelManager::new());
        let tool = RunInferenceTool::new(model_manager);
        assert_eq!(tool.name(), "run_inference");
    }

    #[test]
    fn test_run_inference_tool_description() {
        let model_manager = Arc::new(ModelManager::new());
        let tool = RunInferenceTool::new(model_manager);
        assert!(tool.description().contains("inference"));
    }

    #[test]
    fn test_deploy_model_tool_name() {
        let model_manager = Arc::new(ModelManager::new());
        let wallet_manager = match crate::wallet::WalletManager::new() {
            Ok(wm) => Arc::new(wm),
            Err(_) => return, // Skip test in CI
        };
        let tool = DeployModelTool::new(model_manager, wallet_manager);
        assert_eq!(tool.name(), "deploy_model");
    }

    #[test]
    fn test_deploy_model_requires_confirmation() {
        let model_manager = Arc::new(ModelManager::new());
        let wallet_manager = match crate::wallet::WalletManager::new() {
            Ok(wm) => Arc::new(wm),
            Err(_) => return,
        };
        let tool = DeployModelTool::new(model_manager, wallet_manager);
        assert!(tool.requires_confirmation());
    }

    #[test]
    fn test_get_model_info_tool_name() {
        let model_manager = Arc::new(ModelManager::new());
        let tool = GetModelInfoTool::new(model_manager);
        assert_eq!(tool.name(), "get_model_info");
    }

    #[test]
    fn test_get_model_info_tool_description() {
        let model_manager = Arc::new(ModelManager::new());
        let tool = GetModelInfoTool::new(model_manager);
        assert!(tool.description().contains("detailed"));
    }

    #[tokio::test]
    async fn test_list_models_execution() {
        let model_manager = Arc::new(ModelManager::new());
        let tool = ListModelsTool::new(model_manager);

        let params = IntentParams::default();
        let result = tool.execute(&params).await;

        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.success);
        assert!(output.data.is_some());

        // Should have at least sample models
        let data = output.data.unwrap();
        let count = data.get("count").and_then(|v| v.as_u64()).unwrap_or(0);
        assert!(count >= 2); // gpt-citrate and stable-citrate
    }

    #[tokio::test]
    async fn test_get_model_info_for_sample_model() {
        let model_manager = Arc::new(ModelManager::new());
        let tool = GetModelInfoTool::new(model_manager);

        let mut params = IntentParams::default();
        params.model_name = Some("gpt-citrate".to_string());

        let result = tool.execute(&params).await;

        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.success);
        assert!(output.data.is_some());

        let data = output.data.unwrap();
        assert_eq!(data.get("id").and_then(|v| v.as_str()), Some("gpt-citrate"));
    }

    #[tokio::test]
    async fn test_get_model_info_not_found() {
        let model_manager = Arc::new(ModelManager::new());
        let tool = GetModelInfoTool::new(model_manager);

        let mut params = IntentParams::default();
        params.model_name = Some("nonexistent-model".to_string());

        let result = tool.execute(&params).await;

        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(!output.success);
        assert!(output.message.contains("not found"));
    }

    #[tokio::test]
    async fn test_get_model_info_missing_param() {
        let model_manager = Arc::new(ModelManager::new());
        let tool = GetModelInfoTool::new(model_manager);

        let params = IntentParams::default(); // No model name

        let result = tool.execute(&params).await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        match err {
            DispatchError::InvalidParams(msg) => {
                assert!(msg.contains("required"));
            }
            _ => panic!("Expected InvalidParams error"),
        }
    }
}
