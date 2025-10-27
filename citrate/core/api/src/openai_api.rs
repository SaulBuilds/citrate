// citrate/core/api/src/openai_api.rs

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::Serialize;
use std::collections::HashMap;
use std::sync::Arc;
use tower::ServiceBuilder;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing::{error, info};

use crate::methods::ai::{
    AiApi, ChatCompletionRequest, ChatCompletionResponse, CreateLoRARequest,
    CreateTrainingJobRequest, DeployModelRequest, EmbeddingsRequest, EmbeddingsResponse,
    InferenceRequest,
};
use citrate_execution::executor::Executor;
use citrate_execution::types::Address;
use citrate_sequencer::mempool::Mempool;
use citrate_storage::StorageManager;

/// OpenAI/Anthropic compatible REST API server
pub struct OpenAiRestServer {
    storage: Arc<StorageManager>,
    mempool: Arc<Mempool>,
    executor: Arc<Executor>,
}

/// Server state for Axum handlers
#[derive(Clone)]
pub struct AppState {
    ai_api: AiApi,
}

/// Error response format
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: ErrorDetail,
}

#[derive(Debug, Serialize)]
pub struct ErrorDetail {
    pub message: String,
    pub r#type: String,
    pub code: Option<String>,
}

/// Model list response (OpenAI compatible)
#[derive(Debug, Serialize)]
pub struct ModelListResponse {
    pub object: String,
    pub data: Vec<ModelInfo>,
}

/// Model info for list response
#[derive(Debug, Serialize)]
pub struct ModelInfo {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub owned_by: String,
}

impl OpenAiRestServer {
    /// Create a new OpenAI REST API server
    pub fn new(
        storage: Arc<StorageManager>,
        mempool: Arc<Mempool>,
        executor: Arc<Executor>,
    ) -> Self {
        Self {
            storage,
            mempool,
            executor,
        }
    }

    /// Create the Axum router with all API endpoints
    pub fn router(&self) -> Router {
        let ai_api = AiApi::new(
            self.storage.clone(),
            self.mempool.clone(),
            self.executor.clone(),
        );
        let state = AppState { ai_api };

        Router::new()
            // OpenAI-compatible endpoints
            .route("/v1/models", get(list_models))
            .route("/v1/chat/completions", post(chat_completions))
            .route("/v1/completions", post(completions))
            .route("/v1/embeddings", post(embeddings))
            // Anthropic-compatible endpoints
            .route("/v1/messages", post(messages))
            // Citrate-specific AI endpoints
            .route("/v1/lattice/models", get(lattice_list_models))
            .route("/v1/lattice/models", post(lattice_deploy_model))
            .route("/v1/lattice/models/:model_id", get(lattice_get_model))
            .route(
                "/v1/lattice/models/:model_id/stats",
                get(lattice_model_stats),
            )
            .route("/v1/lattice/inference", post(lattice_request_inference))
            .route(
                "/v1/lattice/inference/:request_id",
                get(lattice_get_inference),
            )
            .route("/v1/lattice/training", post(lattice_create_training_job))
            .route(
                "/v1/lattice/training/:job_id",
                get(lattice_get_training_job),
            )
            .route("/v1/lattice/lora", post(lattice_create_lora))
            .route("/v1/lattice/lora/:adapter_id", get(lattice_get_lora))
            // Health check
            .route("/health", get(health_check))
            .route("/", get(root))
            .layer(
                ServiceBuilder::new()
                    .layer(TraceLayer::new_for_http())
                    .layer(
                        CorsLayer::new()
                            .allow_origin(Any)
                            .allow_methods(Any)
                            .allow_headers(Any),
                    ),
            )
            .with_state(state)
    }

    /// Start the REST API server
    pub async fn start(&self, addr: std::net::SocketAddr) -> anyhow::Result<()> {
        let app = self.router();

        info!("Starting OpenAI-compatible REST API server on {}", addr);

        let listener = tokio::net::TcpListener::bind(addr).await?;
        axum::serve(listener, app).await?;

        Ok(())
    }
}

// ========== OpenAI-Compatible Handlers ==========

/// GET /v1/models - List available models
async fn list_models(State(state): State<AppState>) -> Result<Json<ModelListResponse>, StatusCode> {
    match state.ai_api.list_models(None, None).await {
        Ok(model_ids) => {
            let models: Vec<ModelInfo> = model_ids
                .iter()
                .map(|id| ModelInfo {
                    id: hex::encode(id.0.as_bytes()),
                    object: "model".to_string(),
                    created: chrono::Utc::now().timestamp() as u64,
                    owned_by: "citrate".to_string(),
                })
                .collect();

            Ok(Json(ModelListResponse {
                object: "list".to_string(),
                data: models,
            }))
        }
        Err(e) => {
            error!("Failed to list models: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// POST /v1/chat/completions - OpenAI chat completions
async fn chat_completions(
    State(state): State<AppState>,
    Json(request): Json<ChatCompletionRequest>,
) -> Result<Json<ChatCompletionResponse>, StatusCode> {
    match state.ai_api.chat_completions(request, None).await {
        Ok(response) => Ok(Json(response)),
        Err(e) => {
            error!("Chat completion failed: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// POST /v1/completions - OpenAI text completions (legacy)
async fn completions(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Convert text completion to chat completion
    if let Some(prompt) = payload.get("prompt").and_then(|p| p.as_str()) {
        let chat_request = ChatCompletionRequest {
            model: payload
                .get("model")
                .and_then(|m| m.as_str())
                .unwrap_or("gpt-3.5-turbo")
                .to_string(),
            messages: vec![crate::methods::ai::ChatMessage {
                role: "user".to_string(),
                content: prompt.to_string(),
            }],
            max_tokens: payload
                .get("max_tokens")
                .and_then(|t| t.as_u64())
                .map(|t| t as u32),
            temperature: payload
                .get("temperature")
                .and_then(|t| t.as_f64())
                .map(|t| t as f32),
            top_p: payload
                .get("top_p")
                .and_then(|t| t.as_f64())
                .map(|t| t as f32),
            n: payload.get("n").and_then(|n| n.as_u64()).map(|n| n as u32),
            stop: payload.get("stop").and_then(|s| {
                if s.is_array() {
                    s.as_array().map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_str().map(|s| s.to_string()))
                            .collect()
                    })
                } else {
                    s.as_str().map(|s| vec![s.to_string()])
                }
            }),
            stream: payload.get("stream").and_then(|s| s.as_bool()),
        };

        match state.ai_api.chat_completions(chat_request, None).await {
            Ok(chat_response) => {
                // Convert back to completions format
                let completions_response = serde_json::json!({
                    "id": chat_response.id,
                    "object": "text_completion",
                    "created": chat_response.created,
                    "model": chat_response.model,
                    "choices": chat_response.choices.into_iter().map(|choice| {
                        serde_json::json!({
                            "text": choice.message.content,
                            "index": choice.index,
                            "finish_reason": choice.finish_reason
                        })
                    }).collect::<Vec<_>>(),
                    "usage": chat_response.usage
                });
                Ok(Json(completions_response))
            }
            Err(e) => {
                error!("Completion failed: {}", e);
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    } else {
        Err(StatusCode::BAD_REQUEST)
    }
}

/// POST /v1/embeddings - OpenAI embeddings
async fn embeddings(
    State(state): State<AppState>,
    Json(request): Json<EmbeddingsRequest>,
) -> Result<Json<EmbeddingsResponse>, StatusCode> {
    match state.ai_api.embeddings(request, None).await {
        Ok(response) => Ok(Json(response)),
        Err(e) => {
            error!("Embeddings failed: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// ========== Anthropic-Compatible Handlers ==========

/// POST /v1/messages - Anthropic messages API
async fn messages(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Convert Anthropic messages format to OpenAI chat format
    if let Some(messages) = payload.get("messages").and_then(|m| m.as_array()) {
        let chat_messages: Vec<crate::methods::ai::ChatMessage> = messages
            .iter()
            .filter_map(|msg| {
                let role = msg.get("role")?.as_str()?;
                let content = msg.get("content")?.as_str()?;
                Some(crate::methods::ai::ChatMessage {
                    role: role.to_string(),
                    content: content.to_string(),
                })
            })
            .collect();

        let chat_request = ChatCompletionRequest {
            model: payload
                .get("model")
                .and_then(|m| m.as_str())
                .unwrap_or("claude-3-sonnet")
                .to_string(),
            messages: chat_messages,
            max_tokens: payload
                .get("max_tokens")
                .and_then(|t| t.as_u64())
                .map(|t| t as u32),
            temperature: payload
                .get("temperature")
                .and_then(|t| t.as_f64())
                .map(|t| t as f32),
            top_p: payload
                .get("top_p")
                .and_then(|t| t.as_f64())
                .map(|t| t as f32),
            n: Some(1),
            stop: None,
            stream: payload.get("stream").and_then(|s| s.as_bool()),
        };

        match state.ai_api.chat_completions(chat_request, None).await {
            Ok(chat_response) => {
                // Convert to Anthropic format
                let anthropic_response = serde_json::json!({
                    "id": chat_response.id,
                    "type": "message",
                    "role": "assistant",
                    "content": chat_response.choices.first()
                        .map(|c| c.message.content.clone())
                        .unwrap_or_default(),
                    "model": chat_response.model,
                    "usage": {
                        "input_tokens": chat_response.usage.prompt_tokens,
                        "output_tokens": chat_response.usage.completion_tokens
                    }
                });
                Ok(Json(anthropic_response))
            }
            Err(e) => {
                error!("Anthropic message failed: {}", e);
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    } else {
        Err(StatusCode::BAD_REQUEST)
    }
}

// ========== Citrate-Specific Handlers ==========

/// GET /v1/lattice/models - List Citrate models with detailed info
async fn lattice_list_models(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let owner = params.get("owner").and_then(|addr_str| {
        hex::decode(addr_str.trim_start_matches("0x"))
            .ok()
            .and_then(|bytes| {
                if bytes.len() == 20 {
                    let mut addr_array = [0u8; 20];
                    addr_array.copy_from_slice(&bytes);
                    Some(Address(addr_array))
                } else {
                    None
                }
            })
    });

    let limit = params.get("limit").and_then(|l| l.parse().ok());

    match state.ai_api.list_models(owner, limit).await {
        Ok(model_ids) => {
            let models_detailed: Result<Vec<_>, _> =
                futures::future::join_all(model_ids.iter().map(|id| state.ai_api.get_model(*id)))
                    .await
                    .into_iter()
                    .collect();

            match models_detailed {
                Ok(models) => Ok(Json(serde_json::json!({
                    "models": models,
                    "count": models.len()
                }))),
                Err(e) => {
                    error!("Failed to get model details: {}", e);
                    Err(StatusCode::INTERNAL_SERVER_ERROR)
                }
            }
        }
        Err(e) => {
            error!("Failed to list models: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// POST /v1/lattice/models - Deploy a new model
async fn lattice_deploy_model(
    State(state): State<AppState>,
    Json(request): Json<DeployModelRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // This would need proper authentication and parameter parsing
    let from = Address::zero(); // Placeholder
    let gas_limit = 1_000_000;
    let gas_price = 10000;

    match state
        .ai_api
        .deploy_model(request, from, gas_limit, gas_price)
        .await
    {
        Ok(tx_hash) => Ok(Json(serde_json::json!({
            "transaction_hash": hex::encode(tx_hash.as_bytes()),
            "status": "pending"
        }))),
        Err(e) => {
            error!("Model deployment failed: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// GET /v1/lattice/models/:model_id - Get model details
async fn lattice_get_model(
    State(state): State<AppState>,
    Path(model_id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match hex::decode(&model_id) {
        Ok(model_id_bytes) if model_id_bytes.len() == 32 => {
            let mut model_id_array = [0u8; 32];
            model_id_array.copy_from_slice(&model_id_bytes);
            let model_id = citrate_execution::types::ModelId(citrate_consensus::types::Hash::new(
                model_id_array,
            ));

            match state.ai_api.get_model(model_id).await {
                Ok(model) => Ok(Json(serde_json::to_value(model).unwrap())),
                Err(e) => {
                    error!("Failed to get model: {}", e);
                    Err(StatusCode::NOT_FOUND)
                }
            }
        }
        _ => Err(StatusCode::BAD_REQUEST),
    }
}

/// GET /v1/lattice/models/:model_id/stats - Get model statistics
async fn lattice_model_stats(
    State(state): State<AppState>,
    Path(model_id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match hex::decode(&model_id) {
        Ok(model_id_bytes) if model_id_bytes.len() == 32 => {
            let mut model_id_array = [0u8; 32];
            model_id_array.copy_from_slice(&model_id_bytes);
            let model_id = citrate_execution::types::ModelId(citrate_consensus::types::Hash::new(
                model_id_array,
            ));

            match state.ai_api.get_model_stats(model_id).await {
                Ok(stats) => Ok(Json(serde_json::to_value(stats).unwrap())),
                Err(e) => {
                    error!("Failed to get model stats: {}", e);
                    Err(StatusCode::NOT_FOUND)
                }
            }
        }
        _ => Err(StatusCode::BAD_REQUEST),
    }
}

/// POST /v1/lattice/inference - Request inference
async fn lattice_request_inference(
    State(state): State<AppState>,
    Json(request): Json<InferenceRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let from = Address::zero(); // Placeholder - would get from auth
    let gas_price = 10000;

    match state
        .ai_api
        .request_inference(request, from, gas_price)
        .await
    {
        Ok(request_hash) => Ok(Json(serde_json::json!({
            "request_id": hex::encode(request_hash.as_bytes()),
            "status": "submitted"
        }))),
        Err(e) => {
            error!("Inference request failed: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// GET /v1/lattice/inference/:request_id - Get inference result
async fn lattice_get_inference(
    State(state): State<AppState>,
    Path(request_id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match hex::decode(&request_id) {
        Ok(hash_bytes) if hash_bytes.len() == 32 => {
            let mut hash_array = [0u8; 32];
            hash_array.copy_from_slice(&hash_bytes);
            let request_hash = citrate_consensus::types::Hash::new(hash_array);

            match state.ai_api.get_inference_result(request_hash).await {
                Ok(result) => Ok(Json(serde_json::to_value(result).unwrap())),
                Err(e) => {
                    error!("Failed to get inference result: {}", e);
                    Err(StatusCode::NOT_FOUND)
                }
            }
        }
        _ => Err(StatusCode::BAD_REQUEST),
    }
}

/// POST /v1/lattice/training - Create training job
async fn lattice_create_training_job(
    State(state): State<AppState>,
    Json(request): Json<CreateTrainingJobRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let from = Address::zero(); // Placeholder
    let gas_limit = 1_000_000;
    let gas_price = 10000;

    match state
        .ai_api
        .create_training_job(request, from, gas_limit, gas_price)
        .await
    {
        Ok(job_hash) => Ok(Json(serde_json::json!({
            "job_id": hex::encode(job_hash.as_bytes()),
            "status": "created"
        }))),
        Err(e) => {
            error!("Training job creation failed: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// GET /v1/lattice/training/:job_id - Get training job
async fn lattice_get_training_job(
    State(state): State<AppState>,
    Path(job_id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match hex::decode(&job_id) {
        Ok(job_id_bytes) if job_id_bytes.len() == 32 => {
            let mut job_id_array = [0u8; 32];
            job_id_array.copy_from_slice(&job_id_bytes);
            let job_id =
                citrate_execution::types::JobId(citrate_consensus::types::Hash::new(job_id_array));

            match state.ai_api.get_training_job(job_id).await {
                Ok(job) => Ok(Json(serde_json::to_value(job).unwrap())),
                Err(e) => {
                    error!("Failed to get training job: {}", e);
                    Err(StatusCode::NOT_FOUND)
                }
            }
        }
        _ => Err(StatusCode::BAD_REQUEST),
    }
}

/// POST /v1/lattice/lora - Create LoRA adapter
async fn lattice_create_lora(
    State(state): State<AppState>,
    Json(request): Json<CreateLoRARequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let from = Address::zero(); // Placeholder

    match state.ai_api.create_lora(request, from).await {
        Ok(adapter_hash) => Ok(Json(serde_json::json!({
            "adapter_id": hex::encode(adapter_hash.as_bytes()),
            "status": "created"
        }))),
        Err(e) => {
            error!("LoRA creation failed: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// GET /v1/lattice/lora/:adapter_id - Get LoRA adapter
async fn lattice_get_lora(
    State(state): State<AppState>,
    Path(adapter_id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match hex::decode(&adapter_id) {
        Ok(adapter_id_bytes) if adapter_id_bytes.len() == 32 => {
            let mut adapter_id_array = [0u8; 32];
            adapter_id_array.copy_from_slice(&adapter_id_bytes);
            let adapter_hash = citrate_consensus::types::Hash::new(adapter_id_array);

            match state.ai_api.get_lora(adapter_hash).await {
                Ok(adapter) => Ok(Json(serde_json::to_value(adapter).unwrap())),
                Err(e) => {
                    error!("Failed to get LoRA adapter: {}", e);
                    Err(StatusCode::NOT_FOUND)
                }
            }
        }
        _ => Err(StatusCode::BAD_REQUEST),
    }
}

// ========== Utility Handlers ==========

/// GET /health - Health check
async fn health_check() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().timestamp(),
        "version": env!("CARGO_PKG_VERSION")
    }))
}

/// GET / - Root endpoint
async fn root() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "name": "Citrate AI API",
        "version": env!("CARGO_PKG_VERSION"),
        "description": "OpenAI/Anthropic compatible API for Citrate blockchain AI models",
        "endpoints": {
            "openai": {
                "models": "/v1/models",
                "chat": "/v1/chat/completions",
                "completions": "/v1/completions",
                "embeddings": "/v1/embeddings"
            },
            "anthropic": {
                "messages": "/v1/messages"
            },
            "citrate": {
                "models": "/v1/lattice/models",
                "inference": "/v1/lattice/inference",
                "training": "/v1/lattice/training",
                "lora": "/v1/lattice/lora"
            }
        }
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_response_format() {
        let error = ErrorResponse {
            error: ErrorDetail {
                message: "Test error".to_string(),
                r#type: "invalid_request_error".to_string(),
                code: Some("invalid_model".to_string()),
            },
        };

        let json = serde_json::to_string(&error).unwrap();
        assert!(json.contains("Test error"));
        assert!(json.contains("invalid_request_error"));
    }
}
