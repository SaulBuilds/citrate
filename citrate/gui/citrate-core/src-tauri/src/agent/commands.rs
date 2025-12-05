//! Tauri commands for agent functionality
//!
//! Exposes the agent module to the React frontend.

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::{Emitter, State};
use tokio::sync::RwLock;

use super::config::AgentConfig;
use super::intent::{Intent, IntentMatch};
use super::llm::local::{scan_for_models, GGUFModelInfo};
use super::orchestrator::{AgentOrchestrator, OrchestratorError, ProcessingResult};
use super::session::{AgentSession, Message, PendingToolCall, SessionId, SessionState};
use super::streaming::StreamStatus;
use super::AgentManager;

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
