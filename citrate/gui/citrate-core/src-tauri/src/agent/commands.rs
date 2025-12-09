//! Tauri commands for agent functionality
//!
//! Exposes the agent module to the React frontend.

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::{Emitter, State};
use tokio::sync::RwLock;

use super::config::{
    AgentConfig, AIProvider, ApiKeyManager, ApiKeyValidationResult,
    SecureApiKeyStore
};
use super::intent::{Intent, IntentMatch};
use super::llm::local::{scan_for_models, GGUFModelInfo};
use super::orchestrator::{AgentOrchestrator, OrchestratorError, ProcessingResult};
use super::session::{AgentSession, Message, PendingToolCall, SessionId, SessionState};
use super::streaming::StreamStatus;
use super::AgentManager;

use once_cell::sync::Lazy;

// Global secure API key manager instance
static API_KEY_MANAGER: Lazy<ApiKeyManager> = Lazy::new(ApiKeyManager::new);

/// Agent state accessible from Tauri commands
pub struct AgentState {
    pub manager: Arc<RwLock<Option<AgentManager>>>,
}

impl AgentState {
    pub fn new() -> Self {
        Self {
            manager: Arc::new(RwLock::new(None)),
        }
    }

    pub async fn is_initialized(&self) -> bool {
        self.manager.read().await.is_some()
    }
}

impl Default for AgentState {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// Response Types for Frontend
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSessionInfo {
    pub id: String,
    pub state: String,
    pub message_count: usize,
    pub created_at: u64,
    pub last_activity: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMessageResponse {
    pub session_id: String,
    pub message_id: String,
    pub content: String,
    pub role: String,
    pub intent: Option<String>,
    pub intent_confidence: Option<f32>,
    pub tool_invoked: bool,
    pub tool_name: Option<String>,
    pub pending_approval: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfigResponse {
    pub enabled: bool,
    pub llm_backend: String,
    pub model: String,
    pub streaming_enabled: bool,
    pub local_model_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalModelInfo {
    pub name: String,
    pub size: String,
    pub quantization: String,
    pub path: String,
}

// =============================================================================
// Session Management Commands
// =============================================================================

/// Create a new agent conversation session
#[tauri::command]
pub async fn agent_create_session(
    state: State<'_, AgentState>,
) -> Result<AgentSessionInfo, String> {
    let manager_guard = state.manager.read().await;
    let manager = manager_guard
        .as_ref()
        .ok_or("Agent not initialized")?;

    let orchestrator = manager.orchestrator();
    let session = orchestrator.read().await.create_session().await;

    Ok(AgentSessionInfo {
        id: session.id().0.clone(),
        state: "active".to_string(),
        message_count: 0,
        created_at: session.created_at(),
        last_activity: session.last_activity().await,
    })
}

/// Get info about a session
#[tauri::command]
pub async fn agent_get_session(
    state: State<'_, AgentState>,
    session_id: String,
) -> Result<Option<AgentSessionInfo>, String> {
    let manager_guard = state.manager.read().await;
    let manager = manager_guard
        .as_ref()
        .ok_or("Agent not initialized")?;

    let orchestrator = manager.orchestrator();
    let session = orchestrator.read().await.get_session(&session_id).await;

    match session {
        Some(s) => {
            let messages = s.messages().await;
            let state_str = match s.state().await {
                SessionState::Active => "active",
                SessionState::Processing => "processing",
                SessionState::WaitingForInput => "waiting_for_input",
                SessionState::Paused => "paused",
                SessionState::Closed => "closed",
            };

            Ok(Some(AgentSessionInfo {
                id: s.id().0.clone(),
                state: state_str.to_string(),
                message_count: messages.len(),
                created_at: s.created_at(),
                last_activity: s.last_activity().await,
            }))
        }
        None => Ok(None),
    }
}

/// List all active sessions
#[tauri::command]
pub async fn agent_list_sessions(
    state: State<'_, AgentState>,
) -> Result<Vec<String>, String> {
    let manager_guard = state.manager.read().await;
    let manager = manager_guard
        .as_ref()
        .ok_or("Agent not initialized")?;

    let orchestrator = manager.orchestrator();
    let sessions = orchestrator.read().await.list_sessions().await;
    Ok(sessions)
}

/// Delete a session
#[tauri::command]
pub async fn agent_delete_session(
    state: State<'_, AgentState>,
    session_id: String,
) -> Result<bool, String> {
    let manager_guard = state.manager.read().await;
    let manager = manager_guard
        .as_ref()
        .ok_or("Agent not initialized")?;

    let orchestrator = manager.orchestrator();
    let result = orchestrator.read().await.delete_session(&session_id).await;
    Ok(result)
}

// =============================================================================
// Message Processing Commands
// =============================================================================

/// Send a message to the agent and get a response
#[tauri::command]
pub async fn agent_send_message(
    state: State<'_, AgentState>,
    session_id: String,
    message: String,
) -> Result<AgentMessageResponse, String> {
    let manager_guard = state.manager.read().await;
    let manager = manager_guard
        .as_ref()
        .ok_or("Agent not initialized")?;

    let orchestrator = manager.orchestrator();
    let result = orchestrator
        .read()
        .await
        .process_message(&session_id, &message)
        .await
        .map_err(|e| e.to_string())?;

    // Check for pending tool approvals
    let session = orchestrator
        .read()
        .await
        .get_session(&session_id)
        .await
        .ok_or("Session not found")?;

    let pending = session.pending_tools().await;

    Ok(AgentMessageResponse {
        session_id,
        message_id: result.response.id.clone(),
        content: result.response.content.clone(),
        role: result.response.role.to_string(),
        intent: Some(format!("{:?}", result.intent.intent)),
        intent_confidence: Some(result.intent.confidence),
        tool_invoked: result.tool_invoked,
        tool_name: result.tool_result.map(|t| t.tool_name),
        pending_approval: !pending.is_empty(),
    })
}

/// Get message history for a session
#[tauri::command]
pub async fn agent_get_messages(
    state: State<'_, AgentState>,
    session_id: String,
) -> Result<Vec<serde_json::Value>, String> {
    let manager_guard = state.manager.read().await;
    let manager = manager_guard
        .as_ref()
        .ok_or("Agent not initialized")?;

    let orchestrator = manager.orchestrator();
    let session = orchestrator
        .read()
        .await
        .get_session(&session_id)
        .await
        .ok_or("Session not found")?;

    let messages = session.messages().await;

    Ok(messages
        .iter()
        .map(|m| {
            serde_json::json!({
                "id": m.id,
                "role": m.role.to_string(),
                "content": m.content,
                "timestamp": m.timestamp,
                "is_streaming": m.is_streaming,
                "intent": m.intent.as_ref().map(|i| format!("{:?}", i.intent)),
            })
        })
        .collect())
}

/// Clear session history
#[tauri::command]
pub async fn agent_clear_history(
    state: State<'_, AgentState>,
    session_id: String,
) -> Result<(), String> {
    let manager_guard = state.manager.read().await;
    let manager = manager_guard
        .as_ref()
        .ok_or("Agent not initialized")?;

    let orchestrator = manager.orchestrator();
    let session = orchestrator
        .read()
        .await
        .get_session(&session_id)
        .await
        .ok_or("Session not found")?;

    session.clear_history().await;
    Ok(())
}

// =============================================================================
// Tool Approval Commands
// =============================================================================

/// Get pending tool calls for a session
#[tauri::command]
pub async fn agent_get_pending_tools(
    state: State<'_, AgentState>,
    session_id: String,
) -> Result<Vec<serde_json::Value>, String> {
    let manager_guard = state.manager.read().await;
    let manager = manager_guard
        .as_ref()
        .ok_or("Agent not initialized")?;

    let orchestrator = manager.orchestrator();
    let session = orchestrator
        .read()
        .await
        .get_session(&session_id)
        .await
        .ok_or("Session not found")?;

    let pending = session.pending_tools().await;

    Ok(pending
        .iter()
        .map(|t| {
            serde_json::json!({
                "id": t.id,
                "tool_name": t.tool_name,
                "description": t.description,
                "high_risk": t.high_risk,
                "params": t.params,
            })
        })
        .collect())
}

/// Approve a pending tool call
#[tauri::command]
pub async fn agent_approve_tool(
    state: State<'_, AgentState>,
    session_id: String,
    tool_id: String,
) -> Result<bool, String> {
    let manager_guard = state.manager.read().await;
    let manager = manager_guard
        .as_ref()
        .ok_or("Agent not initialized")?;

    let orchestrator = manager.orchestrator();
    let session = orchestrator
        .read()
        .await
        .get_session(&session_id)
        .await
        .ok_or("Session not found")?;

    let tool = session.approve_tool(&tool_id).await;
    Ok(tool.is_some())
}

/// Reject a pending tool call
#[tauri::command]
pub async fn agent_reject_tool(
    state: State<'_, AgentState>,
    session_id: String,
    tool_id: String,
) -> Result<bool, String> {
    let manager_guard = state.manager.read().await;
    let manager = manager_guard
        .as_ref()
        .ok_or("Agent not initialized")?;

    let orchestrator = manager.orchestrator();
    let session = orchestrator
        .read()
        .await
        .get_session(&session_id)
        .await
        .ok_or("Session not found")?;

    Ok(session.reject_tool(&tool_id).await)
}

// =============================================================================
// Configuration Commands
// =============================================================================

/// Get current agent configuration
#[tauri::command]
pub async fn agent_get_config(
    state: State<'_, AgentState>,
) -> Result<AgentConfigResponse, String> {
    let manager_guard = state.manager.read().await;
    let manager = manager_guard
        .as_ref()
        .ok_or("Agent not initialized")?;

    let config = manager.config();
    let cfg = config.read().await;

    Ok(AgentConfigResponse {
        enabled: cfg.enabled,
        llm_backend: format!("{:?}", cfg.llm.backend),
        model: cfg.llm.model_id.clone(),
        streaming_enabled: cfg.streaming.enabled,
        local_model_path: cfg.llm.model_id.clone().into(),
    })
}

/// Update agent configuration
#[tauri::command]
pub async fn agent_update_config(
    state: State<'_, AgentState>,
    enabled: Option<bool>,
    api_key: Option<String>,
    model: Option<String>,
    streaming_enabled: Option<bool>,
) -> Result<(), String> {
    let manager_guard = state.manager.read().await;
    let manager = manager_guard
        .as_ref()
        .ok_or("Agent not initialized")?;

    let config = manager.config();
    let mut cfg = config.write().await;

    if let Some(e) = enabled {
        cfg.enabled = e;
    }
    if let Some(key) = api_key {
        cfg.llm.api_key = Some(key);
    }
    if let Some(m) = model {
        cfg.llm.model_id = m;
    }
    if let Some(s) = streaming_enabled {
        cfg.streaming.enabled = s;
    }

    Ok(())
}

// =============================================================================
// Local Model Commands
// =============================================================================

/// Scan for available local GGUF models
#[tauri::command]
pub async fn agent_scan_local_models(
    directory: Option<String>,
) -> Result<Vec<LocalModelInfo>, String> {
    let dir = directory.unwrap_or_else(|| {
        super::llm::local::get_default_models_dir()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| ".".to_string())
    });

    let models = scan_for_models(&dir);

    Ok(models
        .into_iter()
        .map(|m| {
            let size = m.size_human();
            LocalModelInfo {
                name: m.name,
                size,
                quantization: m.quantization,
                path: m.path,
            }
        })
        .collect())
}

/// Get the default models directory
#[tauri::command]
pub async fn agent_get_models_dir() -> Result<Option<String>, String> {
    Ok(super::llm::local::get_default_models_dir()
        .map(|p| p.to_string_lossy().to_string()))
}

/// Load a local GGUF model as the active LLM backend
#[tauri::command]
pub async fn agent_load_local_model(
    state: State<'_, AgentState>,
    model_path: String,
) -> Result<String, String> {
    let manager_guard = state.manager.read().await;
    let manager = manager_guard
        .as_ref()
        .ok_or("Agent not initialized")?;

    // Update config to use local backend
    let config = manager.config();
    {
        let mut cfg = config.write().await;
        cfg.llm.backend = super::config::LLMBackendType::LocalGGUF;
        cfg.llm.model_id = model_path.clone();
    }

    // Verify the model file exists
    let path = std::path::Path::new(&model_path);
    if !path.exists() {
        return Err(format!("Model file not found: {}", model_path));
    }

    if path.extension().map_or(true, |ext| ext != "gguf") {
        return Err("Model must be a .gguf file".to_string());
    }

    let model_name = path
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    tracing::info!("Loaded local model: {}", model_name);
    Ok(format!("Model loaded: {}", model_name))
}

/// Get information about the currently loaded model
#[tauri::command]
pub async fn agent_get_active_model(
    state: State<'_, AgentState>,
) -> Result<serde_json::Value, String> {
    let manager_guard = state.manager.read().await;
    let manager = manager_guard
        .as_ref()
        .ok_or("Agent not initialized")?;

    let config = manager.config();
    let cfg = config.read().await;

    Ok(serde_json::json!({
        "backend": format!("{:?}", cfg.llm.backend),
        "model_id": cfg.llm.model_id,
        "temperature": cfg.llm.temperature,
        "max_tokens": cfg.llm.max_tokens,
        "context_size": cfg.llm.context_size,
        "has_api_key": cfg.llm.api_key.is_some(),
    }))
}

/// Set API key for cloud LLM providers
#[tauri::command]
pub async fn agent_set_api_key(
    state: State<'_, AgentState>,
    provider: String,
    api_key: String,
) -> Result<(), String> {
    let manager_guard = state.manager.read().await;
    let manager = manager_guard
        .as_ref()
        .ok_or("Agent not initialized")?;

    let config = manager.config();
    let mut cfg = config.write().await;

    // Set the backend type based on provider
    match provider.to_lowercase().as_str() {
        "openai" => {
            cfg.llm.backend = super::config::LLMBackendType::OpenAI;
            cfg.llm.api_key = Some(api_key);
            if cfg.llm.model_id == "auto" || cfg.llm.model_id.contains(".gguf") {
                cfg.llm.model_id = "gpt-4o".to_string();
            }
        }
        "anthropic" => {
            cfg.llm.backend = super::config::LLMBackendType::Anthropic;
            cfg.llm.api_key = Some(api_key);
            if cfg.llm.model_id == "auto" || cfg.llm.model_id.contains(".gguf") {
                cfg.llm.model_id = "claude-3-sonnet-20240229".to_string();
            }
        }
        _ => {
            return Err(format!("Unknown provider: {}. Use 'openai' or 'anthropic'.", provider));
        }
    }

    Ok(())
}

/// Switch to auto mode (prefer local, fallback to API)
#[tauri::command]
pub async fn agent_set_auto_mode(
    state: State<'_, AgentState>,
) -> Result<(), String> {
    let manager_guard = state.manager.read().await;
    let manager = manager_guard
        .as_ref()
        .ok_or("Agent not initialized")?;

    let config = manager.config();
    let mut cfg = config.write().await;
    cfg.llm.backend = super::config::LLMBackendType::Auto;

    Ok(())
}

// =============================================================================
// Status Commands
// =============================================================================

/// Check if agent is initialized and ready
#[tauri::command]
pub async fn agent_is_ready(
    state: State<'_, AgentState>,
) -> Result<bool, String> {
    Ok(state.is_initialized().await)
}

/// Get agent status summary
#[tauri::command]
pub async fn agent_get_status(
    state: State<'_, AgentState>,
) -> Result<serde_json::Value, String> {
    let manager_guard = state.manager.read().await;

    if let Some(manager) = manager_guard.as_ref() {
        let config = manager.config();
        let cfg = config.read().await;

        let orchestrator = manager.orchestrator();
        let sessions = orchestrator.read().await.list_sessions().await;

        Ok(serde_json::json!({
            "initialized": true,
            "enabled": cfg.enabled,
            "llm_backend": format!("{:?}", cfg.llm.backend),
            "active_sessions": sessions.len(),
            "streaming_enabled": cfg.streaming.enabled,
        }))
    } else {
        Ok(serde_json::json!({
            "initialized": false,
            "enabled": false,
        }))
    }
}

// =============================================================================
// Multi-Provider AI Configuration Commands
// =============================================================================

/// Response type for AI providers configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIProvidersConfigResponse {
    pub openai: ProviderSettingsResponse,
    pub anthropic: ProviderSettingsResponse,
    pub gemini: ProviderSettingsResponse,
    pub xai: ProviderSettingsResponse,
    pub preferred_order: Vec<String>,
    pub local_fallback: bool,
    pub local_model_path: Option<String>,
    pub local_model_cid: Option<String>,
    pub active_provider: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderSettingsResponse {
    pub enabled: bool,
    pub model_id: String,
    pub base_url: Option<String>,
    pub has_api_key: bool,
    pub verified: bool,
}

/// Get the current AI providers configuration
#[tauri::command]
pub async fn get_ai_providers_config(
    state: State<'_, AgentState>,
) -> Result<AIProvidersConfigResponse, String> {
    let manager_guard = state.manager.read().await;
    let manager = manager_guard
        .as_ref()
        .ok_or("Agent not initialized")?;

    let config = manager.config();
    let cfg = config.read().await;
    let providers = &cfg.providers;

    let active = providers.get_active_provider();

    Ok(AIProvidersConfigResponse {
        openai: ProviderSettingsResponse {
            enabled: providers.openai.enabled,
            model_id: providers.openai.model_id.clone(),
            base_url: providers.openai.base_url.clone(),
            has_api_key: providers.openai.api_key.is_some(),
            verified: providers.openai.verified,
        },
        anthropic: ProviderSettingsResponse {
            enabled: providers.anthropic.enabled,
            model_id: providers.anthropic.model_id.clone(),
            base_url: providers.anthropic.base_url.clone(),
            has_api_key: providers.anthropic.api_key.is_some(),
            verified: providers.anthropic.verified,
        },
        gemini: ProviderSettingsResponse {
            enabled: providers.gemini.enabled,
            model_id: providers.gemini.model_id.clone(),
            base_url: providers.gemini.base_url.clone(),
            has_api_key: providers.gemini.api_key.is_some(),
            verified: providers.gemini.verified,
        },
        xai: ProviderSettingsResponse {
            enabled: providers.xai.enabled,
            model_id: providers.xai.model_id.clone(),
            base_url: providers.xai.base_url.clone(),
            has_api_key: providers.xai.api_key.is_some(),
            verified: providers.xai.verified,
        },
        preferred_order: providers.preferred_order.iter().map(|p| format!("{:?}", p).to_lowercase()).collect(),
        local_fallback: providers.local_fallback,
        local_model_path: providers.local_model_path.clone(),
        local_model_cid: providers.local_model_cid.clone(),
        active_provider: active.map(|p| format!("{:?}", p).to_lowercase()),
    })
}

/// Get API keys for all providers (returns masked keys for security)
#[tauri::command]
pub async fn get_ai_provider_keys(
    state: State<'_, AgentState>,
) -> Result<serde_json::Value, String> {
    let manager_guard = state.manager.read().await;
    let manager = manager_guard
        .as_ref()
        .ok_or("Agent not initialized")?;

    let config = manager.config();
    let cfg = config.read().await;
    let providers = &cfg.providers;

    // Return masked keys (only show last 4 characters)
    let mask_key = |key: &Option<String>| -> Option<String> {
        key.as_ref().map(|k| {
            if k.len() > 8 {
                format!("{}...{}", &k[..4], &k[k.len()-4..])
            } else {
                "****".to_string()
            }
        })
    };

    Ok(serde_json::json!({
        "openai": mask_key(&providers.openai.api_key),
        "anthropic": mask_key(&providers.anthropic.api_key),
        "gemini": mask_key(&providers.gemini.api_key),
        "xai": mask_key(&providers.xai.api_key),
    }))
}

/// Update AI providers configuration for a single provider
#[tauri::command]
pub async fn update_ai_providers_config(
    state: State<'_, AgentState>,
    provider: String,
    api_key: Option<String>,
    enabled: Option<bool>,
    model_id: Option<String>,
    base_url: Option<String>,
) -> Result<(), String> {
    let manager_guard = state.manager.read().await;
    let manager = manager_guard
        .as_ref()
        .ok_or("Agent not initialized")?;

    let config = manager.config();
    let mut cfg = config.write().await;

    match provider.to_lowercase().as_str() {
        "openai" | "open_ai" => {
            if let Some(key) = api_key {
                cfg.providers.openai.api_key = Some(key);
            }
            if let Some(e) = enabled {
                cfg.providers.openai.enabled = e;
            }
            if let Some(m) = model_id {
                cfg.providers.openai.model_id = m;
            }
            if let Some(url) = base_url {
                cfg.providers.openai.base_url = Some(url);
            }
        }
        "anthropic" => {
            if let Some(key) = api_key {
                cfg.providers.anthropic.api_key = Some(key);
            }
            if let Some(e) = enabled {
                cfg.providers.anthropic.enabled = e;
            }
            if let Some(m) = model_id {
                cfg.providers.anthropic.model_id = m;
            }
            if let Some(url) = base_url {
                cfg.providers.anthropic.base_url = Some(url);
            }
        }
        "gemini" => {
            if let Some(key) = api_key {
                cfg.providers.gemini.api_key = Some(key);
            }
            if let Some(e) = enabled {
                cfg.providers.gemini.enabled = e;
            }
            if let Some(m) = model_id {
                cfg.providers.gemini.model_id = m;
            }
            if let Some(url) = base_url {
                cfg.providers.gemini.base_url = Some(url);
            }
        }
        "xai" | "x_ai" => {
            if let Some(key) = api_key {
                cfg.providers.xai.api_key = Some(key);
            }
            if let Some(e) = enabled {
                cfg.providers.xai.enabled = e;
            }
            if let Some(m) = model_id {
                cfg.providers.xai.model_id = m;
            }
            if let Some(url) = base_url {
                cfg.providers.xai.base_url = Some(url);
            }
        }
        "local" => {
            if let Some(path) = model_id {
                cfg.providers.local_model_path = Some(path);
            }
        }
        _ => {
            return Err(format!("Unknown provider: {}", provider));
        }
    }

    // Drop the config lock before updating orchestrator
    let updated_config = cfg.clone();
    drop(cfg);

    // Update the orchestrator to recreate LLM backend with new config
    manager.orchestrator().write().await.update_config(updated_config);

    tracing::info!("Updated AI provider configuration for: {} and recreated LLM backend", provider);
    Ok(())
}

/// Input types for batch config update from frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchProviderSettingsInput {
    pub enabled: bool,
    pub model_id: String,
    #[serde(default)]
    pub base_url: Option<String>,
    #[serde(default)]
    pub verified: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchAIProvidersConfigInput {
    pub openai: BatchProviderSettingsInput,
    pub anthropic: BatchProviderSettingsInput,
    pub gemini: BatchProviderSettingsInput,
    pub xai: BatchProviderSettingsInput,
    pub preferred_order: Vec<String>,
    pub local_fallback: bool,
    #[serde(default)]
    pub local_model_path: Option<String>,
    #[serde(default)]
    pub local_model_cid: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchApiKeysInput {
    #[serde(default)]
    pub open_ai: Option<String>,
    #[serde(default)]
    pub anthropic: Option<String>,
    #[serde(default)]
    pub gemini: Option<String>,
    #[serde(default)]
    pub x_ai: Option<String>,
    #[serde(default)]
    pub local: Option<String>,
}

/// Batch update AI providers configuration (matches frontend saveConfig call)
/// This command accepts the full config and apiKeys objects from the frontend
#[tauri::command]
pub async fn save_ai_providers_config(
    state: State<'_, AgentState>,
    config: BatchAIProvidersConfigInput,
    api_keys: BatchApiKeysInput,
) -> Result<(), String> {
    let manager_guard = state.manager.read().await;
    let manager = manager_guard
        .as_ref()
        .ok_or("Agent not initialized")?;

    let agent_config = manager.config();
    let mut cfg = agent_config.write().await;

    // Update OpenAI settings
    cfg.providers.openai.enabled = config.openai.enabled;
    cfg.providers.openai.model_id = config.openai.model_id;
    cfg.providers.openai.base_url = config.openai.base_url;
    if let Some(key) = api_keys.open_ai {
        if !key.is_empty() && !key.contains("...") {
            cfg.providers.openai.api_key = Some(key);
        }
    }

    // Update Anthropic settings
    cfg.providers.anthropic.enabled = config.anthropic.enabled;
    cfg.providers.anthropic.model_id = config.anthropic.model_id;
    cfg.providers.anthropic.base_url = config.anthropic.base_url;
    if let Some(key) = api_keys.anthropic {
        if !key.is_empty() && !key.contains("...") {
            cfg.providers.anthropic.api_key = Some(key);
        }
    }

    // Update Gemini settings
    cfg.providers.gemini.enabled = config.gemini.enabled;
    cfg.providers.gemini.model_id = config.gemini.model_id;
    cfg.providers.gemini.base_url = config.gemini.base_url;
    if let Some(key) = api_keys.gemini {
        if !key.is_empty() && !key.contains("...") {
            cfg.providers.gemini.api_key = Some(key);
        }
    }

    // Update xAI settings
    cfg.providers.xai.enabled = config.xai.enabled;
    cfg.providers.xai.model_id = config.xai.model_id;
    cfg.providers.xai.base_url = config.xai.base_url;
    if let Some(key) = api_keys.x_ai {
        if !key.is_empty() && !key.contains("...") {
            cfg.providers.xai.api_key = Some(key);
        }
    }

    // Update local model settings
    cfg.providers.local_fallback = config.local_fallback;
    if let Some(path) = config.local_model_path {
        cfg.providers.local_model_path = Some(path);
    }
    if let Some(cid) = config.local_model_cid {
        cfg.providers.local_model_cid = Some(cid);
    }

    // Update preferred order
    cfg.providers.preferred_order = config.preferred_order
        .iter()
        .filter_map(|s| match s.to_lowercase().as_str() {
            "openai" | "open_ai" => Some(super::config::AIProvider::OpenAI),
            "anthropic" => Some(super::config::AIProvider::Anthropic),
            "gemini" => Some(super::config::AIProvider::Gemini),
            "xai" | "x_ai" => Some(super::config::AIProvider::XAI),
            "local" => Some(super::config::AIProvider::Local),
            _ => None,
        })
        .collect();

    // Clone config before dropping lock
    let updated_config = cfg.clone();
    drop(cfg);

    // Update the orchestrator to recreate LLM backend with new config
    manager.orchestrator().write().await.update_config(updated_config);

    tracing::info!("Batch updated AI providers configuration and recreated LLM backend");
    Ok(())
}

/// Test connection to an AI provider
#[tauri::command]
pub async fn test_ai_provider_connection(
    state: State<'_, AgentState>,
    provider: String,
) -> Result<serde_json::Value, String> {
    let manager_guard = state.manager.read().await;
    let manager = manager_guard
        .as_ref()
        .ok_or("Agent not initialized")?;

    let config = manager.config();
    let mut cfg = config.write().await;

    let result = match provider.to_lowercase().as_str() {
        "openai" => {
            if let Some(ref key) = cfg.providers.openai.api_key {
                // Test OpenAI API connection
                match test_openai_connection(key, &cfg.providers.openai.model_id, cfg.providers.openai.base_url.as_deref()).await {
                    Ok(model_info) => {
                        cfg.providers.openai.verified = true;
                        serde_json::json!({
                            "success": true,
                            "provider": "openai",
                            "model": model_info,
                            "message": "Successfully connected to OpenAI API"
                        })
                    }
                    Err(e) => {
                        cfg.providers.openai.verified = false;
                        serde_json::json!({
                            "success": false,
                            "provider": "openai",
                            "error": e
                        })
                    }
                }
            } else {
                serde_json::json!({
                    "success": false,
                    "provider": "openai",
                    "error": "No API key configured"
                })
            }
        }
        "anthropic" => {
            if let Some(ref key) = cfg.providers.anthropic.api_key {
                match test_anthropic_connection(key, &cfg.providers.anthropic.model_id, cfg.providers.anthropic.base_url.as_deref()).await {
                    Ok(model_info) => {
                        cfg.providers.anthropic.verified = true;
                        serde_json::json!({
                            "success": true,
                            "provider": "anthropic",
                            "model": model_info,
                            "message": "Successfully connected to Anthropic API"
                        })
                    }
                    Err(e) => {
                        cfg.providers.anthropic.verified = false;
                        serde_json::json!({
                            "success": false,
                            "provider": "anthropic",
                            "error": e
                        })
                    }
                }
            } else {
                serde_json::json!({
                    "success": false,
                    "provider": "anthropic",
                    "error": "No API key configured"
                })
            }
        }
        "gemini" => {
            if let Some(ref key) = cfg.providers.gemini.api_key {
                match test_gemini_connection(key, &cfg.providers.gemini.model_id, cfg.providers.gemini.base_url.as_deref()).await {
                    Ok(model_info) => {
                        cfg.providers.gemini.verified = true;
                        serde_json::json!({
                            "success": true,
                            "provider": "gemini",
                            "model": model_info,
                            "message": "Successfully connected to Google Gemini API"
                        })
                    }
                    Err(e) => {
                        cfg.providers.gemini.verified = false;
                        serde_json::json!({
                            "success": false,
                            "provider": "gemini",
                            "error": e
                        })
                    }
                }
            } else {
                serde_json::json!({
                    "success": false,
                    "provider": "gemini",
                    "error": "No API key configured"
                })
            }
        }
        "xai" => {
            if let Some(ref key) = cfg.providers.xai.api_key {
                match test_xai_connection(key, &cfg.providers.xai.model_id, cfg.providers.xai.base_url.as_deref()).await {
                    Ok(model_info) => {
                        cfg.providers.xai.verified = true;
                        serde_json::json!({
                            "success": true,
                            "provider": "xai",
                            "model": model_info,
                            "message": "Successfully connected to xAI API"
                        })
                    }
                    Err(e) => {
                        cfg.providers.xai.verified = false;
                        serde_json::json!({
                            "success": false,
                            "provider": "xai",
                            "error": e
                        })
                    }
                }
            } else {
                serde_json::json!({
                    "success": false,
                    "provider": "xai",
                    "error": "No API key configured"
                })
            }
        }
        "local" => {
            if let Some(ref path) = cfg.providers.local_model_path {
                if std::path::Path::new(path).exists() {
                    serde_json::json!({
                        "success": true,
                        "provider": "local",
                        "model": path,
                        "message": "Local model file found"
                    })
                } else {
                    serde_json::json!({
                        "success": false,
                        "provider": "local",
                        "error": "Model file not found"
                    })
                }
            } else {
                serde_json::json!({
                    "success": false,
                    "provider": "local",
                    "error": "No local model configured"
                })
            }
        }
        _ => {
            return Err(format!("Unknown provider: {}", provider));
        }
    };

    Ok(result)
}

// Helper functions for testing provider connections
async fn test_openai_connection(api_key: &str, model: &str, base_url: Option<&str>) -> Result<String, String> {
    let url = format!("{}/models", base_url.unwrap_or("https://api.openai.com/v1"));

    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", api_key))
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| format!("Connection failed: {}", e))?;

    if response.status().is_success() {
        Ok(model.to_string())
    } else {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        Err(format!("API error ({}): {}", status, body))
    }
}

async fn test_anthropic_connection(api_key: &str, model: &str, base_url: Option<&str>) -> Result<String, String> {
    let url = format!("{}/v1/messages", base_url.unwrap_or("https://api.anthropic.com"));

    let client = reqwest::Client::new();
    let response = client
        .post(&url)
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .header("content-type", "application/json")
        .json(&serde_json::json!({
            "model": model,
            "max_tokens": 1,
            "messages": [{"role": "user", "content": "test"}]
        }))
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| format!("Connection failed: {}", e))?;

    if response.status().is_success() {
        Ok(model.to_string())
    } else {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        // Check if it's just a model error but connection worked
        if status.as_u16() == 400 && body.contains("model") {
            Ok(model.to_string()) // Connection works, model might need adjustment
        } else {
            Err(format!("API error ({}): {}", status, body))
        }
    }
}

async fn test_gemini_connection(api_key: &str, model: &str, base_url: Option<&str>) -> Result<String, String> {
    let url = format!(
        "{}/models/{}?key={}",
        base_url.unwrap_or("https://generativelanguage.googleapis.com/v1beta"),
        model,
        api_key
    );

    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| format!("Connection failed: {}", e))?;

    if response.status().is_success() {
        Ok(model.to_string())
    } else {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        Err(format!("API error ({}): {}", status, body))
    }
}

async fn test_xai_connection(api_key: &str, model: &str, base_url: Option<&str>) -> Result<String, String> {
    let url = format!("{}/models", base_url.unwrap_or("https://api.x.ai/v1"));

    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", api_key))
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| format!("Connection failed: {}", e))?;

    if response.status().is_success() {
        Ok(model.to_string())
    } else {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        Err(format!("API error ({}): {}", status, body))
    }
}

// =============================================================================
// IPFS Model Management Commands
// =============================================================================

/// Pin the local model to IPFS
#[tauri::command]
pub async fn pin_local_model_to_ipfs(
    state: State<'_, AgentState>,
    ipfs_manager: State<'_, std::sync::Arc<crate::ipfs::IpfsManager>>,
) -> Result<serde_json::Value, String> {
    let manager_guard = state.manager.read().await;
    let manager = manager_guard
        .as_ref()
        .ok_or("Agent not initialized")?;

    let config = manager.config();
    let mut cfg = config.write().await;

    let model_path = cfg.providers.local_model_path.clone()
        .ok_or("No local model configured")?;

    // Check if IPFS is running
    if !ipfs_manager.is_running().await {
        return Err("IPFS daemon is not running. Please start IPFS first.".to_string());
    }

    // Pin the model file
    tracing::info!("Pinning model to IPFS: {}", model_path);
    let result = ipfs_manager.add_file(&std::path::PathBuf::from(&model_path)).await
        .map_err(|e| format!("Failed to pin to IPFS: {}", e))?;

    // Store the CID in config
    cfg.providers.local_model_cid = Some(result.cid.clone());

    tracing::info!("Model pinned with CID: {}", result.cid);
    Ok(serde_json::json!({
        "success": true,
        "cid": result.cid,
        "model_path": model_path,
        "size": result.size,
        "gateway_url": result.gateway_url
    }))
}

/// Delete the local model file (after confirming it's pinned to IPFS)
#[tauri::command]
pub async fn delete_local_model(
    state: State<'_, AgentState>,
    force: bool,
) -> Result<serde_json::Value, String> {
    let manager_guard = state.manager.read().await;
    let manager = manager_guard
        .as_ref()
        .ok_or("Agent not initialized")?;

    let config = manager.config();
    let mut cfg = config.write().await;

    let model_path = cfg.providers.local_model_path.clone()
        .ok_or("No local model configured")?;

    // Check if model is pinned to IPFS (unless force is true)
    if !force && cfg.providers.local_model_cid.is_none() {
        return Err("Model is not pinned to IPFS. Pin it first or use force=true to delete anyway.".to_string());
    }

    // Delete the file
    let path = std::path::Path::new(&model_path);
    if path.exists() {
        std::fs::remove_file(path)
            .map_err(|e| format!("Failed to delete model file: {}", e))?;

        tracing::info!("Deleted local model: {}", model_path);

        // Clear the local path but keep the CID
        cfg.providers.local_model_path = None;

        Ok(serde_json::json!({
            "success": true,
            "deleted_path": model_path,
            "cid": cfg.providers.local_model_cid
        }))
    } else {
        // File doesn't exist, just clear the path
        cfg.providers.local_model_path = None;

        Ok(serde_json::json!({
            "success": true,
            "message": "Model file was already deleted",
            "cid": cfg.providers.local_model_cid
        }))
    }
}

/// Download model from IPFS
#[tauri::command]
pub async fn download_model_from_ipfs(
    state: State<'_, AgentState>,
    ipfs_manager: State<'_, std::sync::Arc<crate::ipfs::IpfsManager>>,
    cid: Option<String>,
) -> Result<serde_json::Value, String> {
    let manager_guard = state.manager.read().await;
    let manager = manager_guard
        .as_ref()
        .ok_or("Agent not initialized")?;

    let config = manager.config();
    let mut cfg = config.write().await;

    // Use provided CID or the one stored in config
    let model_cid = cid.or_else(|| cfg.providers.local_model_cid.clone())
        .ok_or("No IPFS CID provided or stored")?;

    // Get the models directory
    let models_dir = super::llm::local::get_default_models_dir()
        .ok_or("Could not determine models directory")?;

    // Ensure directory exists
    std::fs::create_dir_all(&models_dir)
        .map_err(|e| format!("Failed to create models directory: {}", e))?;

    // Destination path
    let dest_path = models_dir.join(format!("{}.gguf", &model_cid[..8.min(model_cid.len())]));

    // Check if IPFS is running
    if !ipfs_manager.is_running().await {
        return Err("IPFS daemon is not running. Please start IPFS first.".to_string());
    }

    // Download from IPFS using the get command
    tracing::info!("Downloading model from IPFS: {}", model_cid);
    let content = ipfs_manager.get(&model_cid).await
        .map_err(|e| format!("Failed to download from IPFS: {}", e))?;

    // Write content to file
    if let Some(data) = content.data {
        std::fs::write(&dest_path, &data)
            .map_err(|e| format!("Failed to write model file: {}", e))?;
    } else {
        return Err("IPFS returned no data for the given CID".to_string());
    }

    // Update config with new path
    let dest_str = dest_path.to_string_lossy().to_string();
    cfg.providers.local_model_path = Some(dest_str.clone());
    cfg.providers.local_model_cid = Some(model_cid.clone());

    tracing::info!("Model downloaded to: {}", dest_str);
    Ok(serde_json::json!({
        "success": true,
        "cid": model_cid,
        "path": dest_str
    }))
}

/// Set the preferred provider order
#[tauri::command]
pub async fn set_preferred_provider_order(
    state: State<'_, AgentState>,
    order: Vec<String>,
) -> Result<(), String> {
    let manager_guard = state.manager.read().await;
    let manager = manager_guard
        .as_ref()
        .ok_or("Agent not initialized")?;

    let config = manager.config();
    let mut cfg = config.write().await;

    // Convert strings to AIProvider enum
    let providers: Vec<super::config::AIProvider> = order
        .iter()
        .filter_map(|s| match s.to_lowercase().as_str() {
            "openai" => Some(super::config::AIProvider::OpenAI),
            "anthropic" => Some(super::config::AIProvider::Anthropic),
            "gemini" => Some(super::config::AIProvider::Gemini),
            "xai" => Some(super::config::AIProvider::XAI),
            "local" => Some(super::config::AIProvider::Local),
            _ => None,
        })
        .collect();

    cfg.providers.preferred_order = providers;
    tracing::info!("Updated preferred provider order");
    Ok(())
}

/// Set local fallback preference
#[tauri::command]
pub async fn set_local_fallback(
    state: State<'_, AgentState>,
    enabled: bool,
) -> Result<(), String> {
    let manager_guard = state.manager.read().await;
    let manager = manager_guard
        .as_ref()
        .ok_or("Agent not initialized")?;

    let config = manager.config();
    let mut cfg = config.write().await;
    cfg.providers.local_fallback = enabled;

    tracing::info!("Local fallback set to: {}", enabled);
    Ok(())
}

/// Check if onboarding has been completed
#[tauri::command]
pub async fn check_onboarding_status(
    state: State<'_, AgentState>,
) -> Result<serde_json::Value, String> {
    let manager_guard = state.manager.read().await;
    let manager = manager_guard
        .as_ref()
        .ok_or("Agent not initialized")?;

    let config = manager.config();
    let cfg = config.read().await;

    Ok(serde_json::json!({
        "onboarding_completed": cfg.onboarding_completed,
        "first_run": cfg.first_run,
        "has_any_provider": cfg.providers.get_active_provider().is_some(),
    }))
}

/// Mark onboarding as completed
#[tauri::command]
pub async fn complete_onboarding(
    state: State<'_, AgentState>,
) -> Result<(), String> {
    let manager_guard = state.manager.read().await;
    let manager = manager_guard
        .as_ref()
        .ok_or("Agent not initialized")?;

    let config = manager.config();
    let mut cfg = config.write().await;
    cfg.onboarding_completed = true;
    cfg.first_run = false;

    tracing::info!("Onboarding marked as completed");
    Ok(())
}

// =============================================================================
// First-Run and Bundled Model Commands
// =============================================================================

/// Check if this is the first run of the application
#[tauri::command]
pub async fn check_first_run(
    state: State<'_, AgentState>,
) -> Result<serde_json::Value, String> {
    let manager_guard = state.manager.read().await;
    let manager = manager_guard
        .as_ref()
        .ok_or("Agent not initialized")?;

    let config = manager.config();
    let cfg = config.read().await;

    // Check if we have any configured providers or if onboarding is complete
    let has_any_provider = cfg.providers.get_active_provider().is_some();
    let is_first_run = !cfg.onboarding_completed && !has_any_provider;

    Ok(serde_json::json!({
        "is_first_run": is_first_run,
        "onboarding_completed": cfg.onboarding_completed,
        "has_any_provider": has_any_provider,
        "has_local_model": cfg.providers.local_model_path.is_some(),
        "has_cloud_provider": cfg.providers.openai.is_ready() ||
                              cfg.providers.anthropic.is_ready() ||
                              cfg.providers.gemini.is_ready() ||
                              cfg.providers.xai.is_ready()
    }))
}

/// Check for bundled model in app resources and configure if found
#[tauri::command]
pub async fn setup_bundled_model(
    state: State<'_, AgentState>,
    app_handle: tauri::AppHandle,
) -> Result<serde_json::Value, String> {
    use tauri::Manager;

    let manager_guard = state.manager.read().await;
    let manager = manager_guard
        .as_ref()
        .ok_or("Agent not initialized")?;

    let config = manager.config();
    let mut cfg = config.write().await;

    // Try to find bundled model in app resources
    let resource_path = app_handle
        .path()
        .resource_dir()
        .map_err(|e| format!("Failed to get resource directory: {}", e))?;

    let bundled_model_path = resource_path.join("models").join("qwen2-0_5b-instruct-q4_k_m.gguf");

    tracing::info!("Looking for bundled model at: {:?}", bundled_model_path);

    if bundled_model_path.exists() {
        // Get file size
        let metadata = std::fs::metadata(&bundled_model_path)
            .map_err(|e| format!("Failed to get model metadata: {}", e))?;
        let size_mb = metadata.len() / (1024 * 1024);

        // Copy to user models directory for persistent access
        let models_dir = super::llm::local::get_default_models_dir()
            .ok_or("Could not determine models directory")?;

        std::fs::create_dir_all(&models_dir)
            .map_err(|e| format!("Failed to create models directory: {}", e))?;

        let dest_path = models_dir.join("qwen2-0_5b-instruct-q4_k_m.gguf");

        // Only copy if not already exists
        if !dest_path.exists() {
            tracing::info!("Copying bundled model to: {:?}", dest_path);
            std::fs::copy(&bundled_model_path, &dest_path)
                .map_err(|e| format!("Failed to copy bundled model: {}", e))?;
        }

        // Configure the local model
        let dest_str = dest_path.to_string_lossy().to_string();
        cfg.providers.local_model_path = Some(dest_str.clone());

        tracing::info!("Bundled model configured: {}", dest_str);

        Ok(serde_json::json!({
            "found": true,
            "path": dest_str,
            "size_mb": size_mb,
            "model_name": "Qwen2 0.5B Instruct",
            "quantization": "Q4_K_M"
        }))
    } else {
        // No bundled model found - this is okay for dev builds
        tracing::info!("No bundled model found at {:?}", bundled_model_path);

        Ok(serde_json::json!({
            "found": false,
            "expected_path": bundled_model_path.to_string_lossy().to_string()
        }))
    }
}

/// Get onboarding questions for the frontend
#[tauri::command]
pub async fn get_onboarding_questions() -> Result<serde_json::Value, String> {
    let manager = super::onboarding::OnboardingManager::new();
    let questions = manager.get_questions();

    let questions_json: Vec<serde_json::Value> = questions.iter().map(|q| {
        serde_json::json!({
            "id": q.id,
            "question": q.question,
            "category": format!("{:?}", q.category),
            "options": q.options.iter().map(|o| {
                serde_json::json!({
                    "text": o.text,
                    "skill_points": o.skill_points,
                    "follow_up": o.follow_up
                })
            }).collect::<Vec<_>>()
        })
    }).collect();

    Ok(serde_json::json!({
        "questions": questions_json,
        "total": questions.len(),
        "welcome_message": super::onboarding::OnboardingManager::get_welcome_message(),
        "assessment_intro": super::onboarding::OnboardingManager::get_assessment_intro()
    }))
}

/// Process an onboarding assessment answer
#[tauri::command]
pub async fn process_onboarding_answer(
    state: State<'_, AgentState>,
    answers: Vec<u8>,
) -> Result<serde_json::Value, String> {
    let manager_guard = state.manager.read().await;
    let manager = manager_guard
        .as_ref()
        .ok_or("Agent not initialized")?;

    let config = manager.config();
    let mut cfg = config.write().await;

    // Create assessment and process answers
    let mut assessment = super::onboarding::UserAssessment::new();
    let onboarding = super::onboarding::OnboardingManager::new();
    let questions = onboarding.get_questions();

    for (i, &points) in answers.iter().enumerate() {
        if let Some(q) = questions.get(i) {
            assessment.record_answer(&q.id, points);
        }
    }

    assessment.finalize();

    // Get the result message and path
    let result_message = super::onboarding::OnboardingManager::get_result_message(assessment.skill_level);
    let path = onboarding.get_path(assessment.skill_level);

    // Mark onboarding as completed
    cfg.onboarding_completed = true;
    cfg.first_run = false;

    tracing::info!("Onboarding completed with skill level: {:?}", assessment.skill_level);

    Ok(serde_json::json!({
        "skill_level": format!("{:?}", assessment.skill_level).to_lowercase(),
        "total_score": assessment.total_score,
        "result_message": result_message,
        "path": path.map(|p| serde_json::json!({
            "name": p.name,
            "description": p.description,
            "steps": p.steps.iter().map(|s| serde_json::json!({
                "id": s.id,
                "title": s.title,
                "content": s.content,
                "optional": s.optional
            })).collect::<Vec<_>>()
        }))
    }))
}

/// Skip onboarding (user can set up later)
#[tauri::command]
pub async fn skip_onboarding(
    state: State<'_, AgentState>,
) -> Result<(), String> {
    let manager_guard = state.manager.read().await;
    let manager = manager_guard
        .as_ref()
        .ok_or("Agent not initialized")?;

    let config = manager.config();
    let mut cfg = config.write().await;

    // Mark as completed but with default skill level
    cfg.onboarding_completed = true;
    cfg.first_run = false;

    tracing::info!("Onboarding skipped by user");
    Ok(())
}

// =============================================================================
// Secure API Key Management Commands
// =============================================================================

/// Helper to convert string to AIProvider
fn parse_provider(provider: &str) -> Result<AIProvider, String> {
    match provider.to_lowercase().as_str() {
        "openai" | "open_ai" => Ok(AIProvider::OpenAI),
        "anthropic" => Ok(AIProvider::Anthropic),
        "gemini" | "google" | "google_gemini" => Ok(AIProvider::Gemini),
        "xai" | "x_ai" | "grok" => Ok(AIProvider::XAI),
        "local" => Ok(AIProvider::Local),
        _ => Err(format!("Unknown provider: {}. Valid options: openai, anthropic, gemini, xai, local", provider))
    }
}

/// Store an API key securely with optional validation
/// This uses OS keychain when available, with encrypted file fallback
#[tauri::command]
pub async fn secure_store_api_key(
    state: State<'_, AgentState>,
    provider: String,
    api_key: String,
    validate: bool,
) -> Result<serde_json::Value, String> {
    let ai_provider = parse_provider(&provider)?;

    // Validate and store the key
    let result = API_KEY_MANAGER
        .set_key(ai_provider, &api_key, validate, None)
        .await
        .map_err(|e| e.to_string())?;

    // Also update the in-memory config if agent is initialized
    if let Some(manager) = state.manager.read().await.as_ref() {
        let config = manager.config();
        let mut cfg = config.write().await;
        cfg.providers.set_api_key(ai_provider, api_key);
        if let Some(settings) = cfg.providers.get_provider_settings_mut(ai_provider) {
            settings.verified = result.valid;
        }
    }

    tracing::info!("Securely stored {} API key (validated: {})", provider, validate);

    Ok(serde_json::json!({
        "success": true,
        "provider": provider,
        "validated": result.valid,
        "model_access": result.model_access,
        "rate_limit_remaining": result.rate_limit_remaining
    }))
}

/// Validate an API key format without storing it
#[tauri::command]
pub async fn validate_api_key_format(
    provider: String,
    api_key: String,
) -> Result<serde_json::Value, String> {
    let ai_provider = parse_provider(&provider)?;

    match SecureApiKeyStore::validate_key_format(ai_provider, &api_key) {
        Ok(()) => Ok(serde_json::json!({
            "valid": true,
            "provider": provider
        })),
        Err(e) => Ok(serde_json::json!({
            "valid": false,
            "provider": provider,
            "error": e.to_string()
        }))
    }
}

/// Validate a stored API key by making a test API call
#[tauri::command]
pub async fn validate_stored_api_key(
    provider: String,
) -> Result<serde_json::Value, String> {
    let ai_provider = parse_provider(&provider)?;

    let result = API_KEY_MANAGER
        .validate_stored_key(ai_provider, None)
        .await;

    Ok(serde_json::json!({
        "valid": result.valid,
        "provider": provider,
        "error_message": result.error_message,
        "model_access": result.model_access,
        "rate_limit_remaining": result.rate_limit_remaining
    }))
}

/// Check if a secure API key exists for a provider
#[tauri::command]
pub async fn has_secure_api_key(
    provider: String,
) -> Result<bool, String> {
    let ai_provider = parse_provider(&provider)?;
    Ok(API_KEY_MANAGER.has_key(ai_provider))
}

/// Delete a securely stored API key
#[tauri::command]
pub async fn delete_secure_api_key(
    state: State<'_, AgentState>,
    provider: String,
) -> Result<(), String> {
    let ai_provider = parse_provider(&provider)?;

    API_KEY_MANAGER
        .delete_key(ai_provider)
        .map_err(|e| e.to_string())?;

    // Also clear from in-memory config
    if let Some(manager) = state.manager.read().await.as_ref() {
        let config = manager.config();
        let mut cfg = config.write().await;
        if let Some(settings) = cfg.providers.get_provider_settings_mut(ai_provider) {
            settings.api_key = None;
            settings.enabled = false;
            settings.verified = false;
        }
    }

    tracing::info!("Deleted {} API key from secure storage", provider);
    Ok(())
}

/// Load all stored API keys into the agent config
/// Call this during agent initialization
#[tauri::command]
pub async fn load_secure_api_keys(
    state: State<'_, AgentState>,
) -> Result<serde_json::Value, String> {
    let manager_guard = state.manager.read().await;
    let manager = manager_guard
        .as_ref()
        .ok_or("Agent not initialized")?;

    let config = manager.config();
    let mut cfg = config.write().await;

    // Load keys from secure storage
    cfg.load_api_keys(API_KEY_MANAGER.store());

    // Count loaded keys
    let mut loaded = Vec::new();
    for provider in [AIProvider::OpenAI, AIProvider::Anthropic, AIProvider::Gemini, AIProvider::XAI] {
        if API_KEY_MANAGER.has_key(provider) {
            loaded.push(format!("{:?}", provider).to_lowercase());
        }
    }

    tracing::info!("Loaded {} API keys from secure storage", loaded.len());

    Ok(serde_json::json!({
        "loaded_count": loaded.len(),
        "providers": loaded
    }))
}

/// Get status of all securely stored API keys
#[tauri::command]
pub async fn get_secure_api_key_status() -> Result<serde_json::Value, String> {
    let providers = [
        ("openai", AIProvider::OpenAI),
        ("anthropic", AIProvider::Anthropic),
        ("gemini", AIProvider::Gemini),
        ("xai", AIProvider::XAI),
    ];

    let mut status = serde_json::Map::new();

    for (name, provider) in providers {
        let has_key = API_KEY_MANAGER.has_key(provider);
        status.insert(name.to_string(), serde_json::json!({
            "has_key": has_key,
            "provider": name
        }));
    }

    Ok(serde_json::Value::Object(status))
}

// =============================================================================
// Enhanced Model Download Commands
// =============================================================================

/// Download the enhanced 7B model from HuggingFace with progress reporting
/// This is called automatically during onboarding to give users the best experience
#[tauri::command]
pub async fn download_enhanced_model(
    state: State<'_, AgentState>,
    app_handle: tauri::AppHandle,
) -> Result<serde_json::Value, String> {
    use std::io::Write;

    let model_url = "https://huggingface.co/Qwen/Qwen2.5-Coder-7B-Instruct-GGUF/resolve/main/qwen2.5-coder-7b-instruct-q4_k_m.gguf";
    let model_filename = "qwen2.5-coder-7b-instruct-q4_k_m.gguf";
    let expected_size: u64 = 4_700_000_000; // ~4.7GB

    // Get the models directory
    let models_dir = super::llm::local::get_default_models_dir()
        .ok_or("Could not determine models directory")?;

    // Ensure directory exists
    std::fs::create_dir_all(&models_dir)
        .map_err(|e| format!("Failed to create models directory: {}", e))?;

    let dest_path = models_dir.join(model_filename);

    // Check if already downloaded
    if dest_path.exists() {
        let metadata = std::fs::metadata(&dest_path)
            .map_err(|e| format!("Failed to get file metadata: {}", e))?;

        // If file is close to expected size, consider it complete
        if metadata.len() > expected_size - 100_000_000 {
            tracing::info!("Enhanced model already downloaded: {:?}", dest_path);

            // Configure as active model
            let manager_guard = state.manager.read().await;
            if let Some(manager) = manager_guard.as_ref() {
                let config = manager.config();
                let mut cfg = config.write().await;
                cfg.providers.local_model_path = Some(dest_path.to_string_lossy().to_string());
            }

            return Ok(serde_json::json!({
                "success": true,
                "already_exists": true,
                "path": dest_path.to_string_lossy().to_string(),
                "size_mb": metadata.len() / (1024 * 1024),
                "model_name": "Qwen2.5-Coder-7B-Instruct"
            }));
        }
    }

    tracing::info!("Starting enhanced model download from HuggingFace");

    // Emit initial progress event
    let _ = app_handle.emit("model-download-progress", serde_json::json!({
        "status": "starting",
        "progress": 0,
        "message": "Connecting to HuggingFace..."
    }));

    // Start the download
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(3600)) // 1 hour timeout for large file
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    let response = client
        .get(model_url)
        .send()
        .await
        .map_err(|e| format!("Failed to connect to HuggingFace: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("HuggingFace returned error: {}", response.status()));
    }

    let total_size = response.content_length().unwrap_or(expected_size);

    // Emit download started event
    let _ = app_handle.emit("model-download-progress", serde_json::json!({
        "status": "downloading",
        "progress": 0,
        "total_size_mb": total_size / (1024 * 1024),
        "message": "Downloading Qwen2.5-Coder-7B model..."
    }));

    // Create temporary file for download
    let temp_path = dest_path.with_extension("gguf.download");
    let mut file = std::fs::File::create(&temp_path)
        .map_err(|e| format!("Failed to create temp file: {}", e))?;

    // Download in chunks manually using tokio::io::copy with progress
    let bytes = response.bytes().await
        .map_err(|e| format!("Failed to download: {}", e))?;

    let downloaded = bytes.len() as u64;
    let progress = (downloaded as f64 / total_size as f64 * 100.0).min(100.0) as u8;
    let downloaded_mb = downloaded / (1024 * 1024);
    let total_mb = total_size / (1024 * 1024);

    // Emit progress updates
    let _ = app_handle.emit("model-download-progress", serde_json::json!({
        "status": "downloading",
        "progress": progress,
        "downloaded_mb": downloaded_mb,
        "total_size_mb": total_mb,
        "message": format!("Downloading: {} MB / {} MB ({}%)", downloaded_mb, total_mb, progress)
    }));

    // Write the bytes to file
    file.write_all(&bytes)
        .map_err(|e| format!("Failed to write file: {}", e))?;

    // Flush and rename temp file to final destination
    file.flush().map_err(|e| format!("Failed to flush file: {}", e))?;
    drop(file);

    std::fs::rename(&temp_path, &dest_path)
        .map_err(|e| format!("Failed to finalize download: {}", e))?;

    let final_size = std::fs::metadata(&dest_path)
        .map(|m| m.len())
        .unwrap_or(downloaded);

    // Configure as active model
    let manager_guard = state.manager.read().await;
    if let Some(manager) = manager_guard.as_ref() {
        let config = manager.config();
        let mut cfg = config.write().await;
        cfg.providers.local_model_path = Some(dest_path.to_string_lossy().to_string());
    }

    // Emit completion event
    let _ = app_handle.emit("model-download-progress", serde_json::json!({
        "status": "complete",
        "progress": 100,
        "path": dest_path.to_string_lossy().to_string(),
        "size_mb": final_size / (1024 * 1024),
        "message": "Download complete! Enhanced AI model is ready."
    }));

    tracing::info!("Enhanced model download complete: {:?}", dest_path);

    Ok(serde_json::json!({
        "success": true,
        "already_exists": false,
        "path": dest_path.to_string_lossy().to_string(),
        "size_mb": final_size / (1024 * 1024),
        "model_name": "Qwen2.5-Coder-7B-Instruct"
    }))
}

/// Check if the enhanced 7B model is already downloaded
#[tauri::command]
pub async fn check_enhanced_model_status() -> Result<serde_json::Value, String> {
    let model_filename = "qwen2.5-coder-7b-instruct-q4_k_m.gguf";
    let expected_size: u64 = 4_700_000_000; // ~4.7GB

    let models_dir = super::llm::local::get_default_models_dir()
        .ok_or("Could not determine models directory")?;

    let model_path = models_dir.join(model_filename);

    if model_path.exists() {
        let metadata = std::fs::metadata(&model_path)
            .map_err(|e| format!("Failed to get file metadata: {}", e))?;

        let is_complete = metadata.len() > expected_size - 100_000_000;

        Ok(serde_json::json!({
            "exists": true,
            "complete": is_complete,
            "path": model_path.to_string_lossy().to_string(),
            "size_mb": metadata.len() / (1024 * 1024),
            "model_name": "Qwen2.5-Coder-7B-Instruct"
        }))
    } else {
        Ok(serde_json::json!({
            "exists": false,
            "complete": false,
            "expected_path": model_path.to_string_lossy().to_string()
        }))
    }
}

/// Cancel an in-progress model download
#[tauri::command]
pub async fn cancel_model_download() -> Result<(), String> {
    let model_filename = "qwen2.5-coder-7b-instruct-q4_k_m.gguf";

    let models_dir = super::llm::local::get_default_models_dir()
        .ok_or("Could not determine models directory")?;

    // Remove the temp download file if it exists
    let temp_path = models_dir.join(format!("{}.download", model_filename));
    if temp_path.exists() {
        std::fs::remove_file(&temp_path)
            .map_err(|e| format!("Failed to remove temp file: {}", e))?;
    }

    tracing::info!("Model download cancelled");
    Ok(())
}
