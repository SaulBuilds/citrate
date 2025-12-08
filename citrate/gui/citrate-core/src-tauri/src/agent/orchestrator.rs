//! Agent orchestrator - central coordination for AI conversations
//!
//! The orchestrator manages the complete lifecycle of user interactions:
//! 1. Receives user message
//! 2. Classifies intent
//! 3. Dispatches to appropriate tools or LLM
//! 4. Streams response back to user
//! 5. Manages conversation context

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use super::classifier::IntentClassifier;
use super::config::{AgentConfig, ClassifierConfig};
use super::context::{ContextWindow, ConversationHistory, SystemContext};
use super::dispatcher::ToolDispatcher;
use super::intent::{Intent, IntentMatch};
use super::llm::{LLMBackend, LLMConfig, LLMFactory};
use super::session::{AgentSession, Message, SessionId};
use super::streaming::StreamManager;
use super::tools::register_all_tools;

use crate::dag::DAGManager;
use crate::models::ModelManager;
use crate::node::NodeManager;
use crate::wallet::WalletManager;

/// Result type for orchestrator operations
pub type OrchestratorResult<T> = Result<T, OrchestratorError>;

/// Errors that can occur during orchestration
#[derive(Debug, Clone)]
pub enum OrchestratorError {
    /// Session not found
    SessionNotFound(String),
    /// Classification failed
    ClassificationFailed(String),
    /// Tool execution failed
    ToolExecutionFailed(String),
    /// LLM call failed
    LLMError(String),
    /// Streaming error
    StreamError(String),
    /// Configuration error
    ConfigError(String),
    /// Internal error
    Internal(String),
}

impl std::fmt::Display for OrchestratorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SessionNotFound(id) => write!(f, "Session not found: {}", id),
            Self::ClassificationFailed(e) => write!(f, "Classification failed: {}", e),
            Self::ToolExecutionFailed(e) => write!(f, "Tool execution failed: {}", e),
            Self::LLMError(e) => write!(f, "LLM error: {}", e),
            Self::StreamError(e) => write!(f, "Stream error: {}", e),
            Self::ConfigError(e) => write!(f, "Config error: {}", e),
            Self::Internal(e) => write!(f, "Internal error: {}", e),
        }
    }
}

impl std::error::Error for OrchestratorError {}

/// Response from processing a message
#[derive(Debug, Clone)]
pub struct ProcessingResult {
    /// The assistant's response message
    pub response: Message,
    /// The detected intent
    pub intent: IntentMatch,
    /// Whether a tool was invoked
    pub tool_invoked: bool,
    /// Tool result if applicable
    pub tool_result: Option<ToolResult>,
    /// Whether response was streamed
    pub was_streamed: bool,
}

/// Result from a tool invocation
#[derive(Debug, Clone)]
pub struct ToolResult {
    /// Tool name
    pub tool_name: String,
    /// Whether tool succeeded
    pub success: bool,
    /// Tool output
    pub output: String,
    /// Structured data if applicable
    pub data: Option<serde_json::Value>,
}

/// The main agent orchestrator
pub struct AgentOrchestrator {
    /// Configuration
    config: AgentConfig,
    /// Active sessions
    sessions: RwLock<HashMap<String, Arc<AgentSession>>>,
    /// Intent classifier
    classifier: IntentClassifier,
    /// Tool dispatcher
    dispatcher: ToolDispatcher,
    /// LLM backend
    llm: Box<dyn LLMBackend + Send + Sync>,
    /// Stream manager
    stream_manager: Arc<StreamManager>,
    /// Node manager reference
    node_manager: Arc<NodeManager>,
    /// Wallet manager reference
    wallet_manager: Arc<WalletManager>,
    /// Model manager reference
    model_manager: Arc<ModelManager>,
    /// DAG manager reference
    dag_manager: Arc<RwLock<Option<Arc<DAGManager>>>>,
}

impl AgentOrchestrator {
    /// Create a new orchestrator with manager references
    pub fn new(
        config: AgentConfig,
        node_manager: Arc<NodeManager>,
        wallet_manager: Arc<WalletManager>,
        model_manager: Arc<ModelManager>,
        dag_manager: Arc<RwLock<Option<Arc<DAGManager>>>>,
    ) -> Self {
        let classifier = IntentClassifier::new(config.classifier.clone());

        // Create dispatcher and register all tools with real manager implementations
        let mut dispatcher = ToolDispatcher::new();
        register_all_tools(
            &mut dispatcher,
            node_manager.clone(),
            wallet_manager.clone(),
            model_manager.clone(),
            dag_manager.clone(),
        );

        // Use the config to create the appropriate LLM backend
        // This will check for local models, API keys, and fall back to UnconfiguredLLMBackend
        let llm = Self::create_llm_from_config(&config);

        Self {
            config,
            sessions: RwLock::new(HashMap::new()),
            classifier,
            dispatcher,
            llm,
            stream_manager: Arc::new(StreamManager::new()),
            node_manager,
            wallet_manager,
            model_manager,
            dag_manager,
        }
    }

    /// Create a new session
    pub async fn create_session(&self) -> Arc<AgentSession> {
        let session = Arc::new(AgentSession::new());
        let session_id = session.id().0.clone();

        self.sessions
            .write()
            .await
            .insert(session_id, session.clone());

        session
    }

    /// Get a session by ID
    pub async fn get_session(&self, session_id: &str) -> Option<Arc<AgentSession>> {
        self.sessions.read().await.get(session_id).cloned()
    }

    /// Delete a session
    pub async fn delete_session(&self, session_id: &str) -> bool {
        self.sessions.write().await.remove(session_id).is_some()
    }

    /// List all session IDs
    pub async fn list_sessions(&self) -> Vec<String> {
        self.sessions.read().await.keys().cloned().collect()
    }

    /// Process a user message
    pub async fn process_message(
        &self,
        session_id: &str,
        user_message: &str,
    ) -> OrchestratorResult<ProcessingResult> {
        // Get or create session
        let session = self
            .get_session(session_id)
            .await
            .ok_or_else(|| OrchestratorError::SessionNotFound(session_id.to_string()))?;

        // Create user message
        let user_msg = Message::user(user_message.to_string());
        session.add_message(user_msg.clone()).await;

        // Classify intent
        let intent_match = self.classify_intent(user_message).await?;

        // Process based on intent
        let (response, tool_invoked, tool_result) = match &intent_match.intent {
            // Direct tool intents
            Intent::QueryBalance
            | Intent::SendTransaction
            | Intent::GetTransactionHistory
            | Intent::DeployContract
            | Intent::CallContract
            | Intent::WriteContract
            | Intent::GetBlockInfo
            | Intent::GetDAGStatus
            | Intent::GetNodeStatus
            | Intent::ListModels
            | Intent::RunInference
            | Intent::DeployModel => {
                self.handle_tool_intent(&session, &intent_match, user_message)
                    .await?
            }

            // Conversational intents - use LLM
            Intent::GeneralChat | Intent::Help | Intent::Unknown => {
                self.handle_chat_intent(&session, &intent_match, user_message)
                    .await?
            }

            // Other intents
            _ => {
                self.handle_chat_intent(&session, &intent_match, user_message)
                    .await?
            }
        };

        // Add response to session
        session.add_message(response.clone()).await;

        Ok(ProcessingResult {
            response,
            intent: intent_match,
            tool_invoked,
            tool_result,
            was_streamed: false,
        })
    }

    /// Classify user intent
    async fn classify_intent(&self, message: &str) -> OrchestratorResult<IntentMatch> {
        self.classifier
            .classify(message)
            .await
            .map_err(|e| OrchestratorError::ClassificationFailed(e.to_string()))
    }

    /// Handle a tool-based intent
    async fn handle_tool_intent(
        &self,
        _session: &Arc<AgentSession>,
        intent: &IntentMatch,
        user_message: &str,
    ) -> OrchestratorResult<(Message, bool, Option<ToolResult>)> {
        // Get the tool for this intent
        let tool_name = intent.intent.tool_name().unwrap_or("unknown");

        // Execute the tool
        match self.dispatcher.dispatch(tool_name, &intent.params).await {
            Ok(output) => {
                let tool_result = ToolResult {
                    tool_name: tool_name.to_string(),
                    success: true,
                    output: output.clone(),
                    data: None,
                };

                // Format response with LLM if configured
                let response_text = if self.config.llm.max_tokens > 0 {
                    self.format_tool_result_with_llm(user_message, &output)
                        .await
                        .unwrap_or(output)
                } else {
                    output
                };

                Ok((
                    Message::assistant(response_text),
                    true,
                    Some(tool_result),
                ))
            }
            Err(e) => {
                let error_msg = format!("Tool execution failed: {}", e);
                Ok((Message::assistant(error_msg), true, None))
            }
        }
    }

    /// Handle a chat/conversational intent
    async fn handle_chat_intent(
        &self,
        session: &Arc<AgentSession>,
        _intent: &IntentMatch,
        _user_message: &str,
    ) -> OrchestratorResult<(Message, bool, Option<ToolResult>)> {
        tracing::debug!("handle_chat_intent starting");

        // Build context window
        let system_prompt = self
            .config
            .system_prompt
            .clone()
            .unwrap_or_else(|| AgentConfig::default_system_prompt());

        let system_context = self.get_system_context().await;

        // Get recent messages for context
        let recent = session.recent_messages(10).await;
        tracing::debug!("Got {} recent messages for context", recent.len());

        let mut history = ConversationHistory::new();
        for msg in recent {
            history.add_message(msg);
        }

        let context_window = history.build_context_window(
            &system_prompt,
            Some(system_context),
            self.config.context.max_context_tokens as usize,
            10,
        );

        tracing::debug!("Calling LLM backend: {}", self.llm.name());

        // Call LLM
        let response = self
            .llm
            .complete(&context_window)
            .await
            .map_err(|e| {
                tracing::error!("LLM error: {}", e);
                OrchestratorError::LLMError(e.to_string())
            })?;

        tracing::debug!("Got response: {} chars", response.len());
        Ok((Message::assistant(response), false, None))
    }

    /// Format tool result with LLM for natural language
    async fn format_tool_result_with_llm(
        &self,
        user_message: &str,
        tool_output: &str,
    ) -> Result<String, OrchestratorError> {
        let prompt = format!(
            "The user asked: \"{}\"\n\nThe tool returned:\n{}\n\nProvide a natural, helpful response:",
            user_message, tool_output
        );

        // Create a simple context window for formatting
        let mut history = ConversationHistory::new();
        history.add_message(Message::user(prompt));

        let context = history.build_context_window(
            "You are a helpful assistant that formats tool outputs into natural responses.",
            None,
            2048,
            1,
        );

        self.llm
            .complete(&context)
            .await
            .map_err(|e| OrchestratorError::LLMError(e.to_string()))
    }

    /// Get current system context
    async fn get_system_context(&self) -> SystemContext {
        // Get wallet info - use first account if available
        let accounts = self.wallet_manager.get_accounts().await;
        let first_account = accounts.first();
        let wallet_address = first_account.map(|a| a.address.clone());
        let wallet_balance = first_account.map(|a| format!("{} CTR", a.balance));

        // Get node status
        let node_status = self.node_manager.get_status().await.ok();
        let node_connected = node_status.as_ref().map(|s| s.running).unwrap_or(false);
        let block_height = node_status.as_ref().map(|s| s.block_height);

        // Get network from config
        let config = self.node_manager.get_config().await;
        let network = config.network.clone();

        // Get available models
        let models = self.model_manager.get_models().await.unwrap_or_default();
        let available_models: Vec<String> = models.iter().map(|m| m.name.clone()).collect();

        SystemContext {
            wallet_address,
            wallet_balance,
            node_connected,
            block_height,
            network,
            available_models,
        }
    }

    /// Get stream manager
    pub fn stream_manager(&self) -> Arc<StreamManager> {
        self.stream_manager.clone()
    }

    /// Update configuration and recreate LLM backend if needed
    pub fn update_config(&mut self, config: AgentConfig) {
        // Check if we need to recreate the LLM backend
        let old_model_path = self.config.providers.local_model_path.clone();
        let new_model_path = config.providers.local_model_path.clone();

        let old_openai_ready = self.config.providers.openai.is_ready();
        let new_openai_ready = config.providers.openai.is_ready();

        let old_anthropic_ready = self.config.providers.anthropic.is_ready();
        let new_anthropic_ready = config.providers.anthropic.is_ready();

        self.config = config.clone();

        // Recreate LLM backend if provider configuration changed
        if old_model_path != new_model_path
            || old_openai_ready != new_openai_ready
            || old_anthropic_ready != new_anthropic_ready
        {
            tracing::info!("Provider configuration changed, recreating LLM backend");
            self.llm = Self::create_llm_from_config(&config);
        }
    }

    /// Create LLM backend from AgentConfig
    fn create_llm_from_config(config: &AgentConfig) -> Box<dyn LLMBackend + Send + Sync> {
        use super::llm::{LLMBackendType as LLMType, LLMConfig as LLMCfg};

        // Priority order based on providers.preferred_order:
        // 1. Local GGUF model (if available and configured) - preferred for privacy
        // 2. Cloud APIs (OpenAI, Anthropic) as fallback

        // First, try to find a local model if local is in preferred order
        let local_model_path = Self::find_local_model(&config.providers.local_model_path);

        // Use the active provider from the config (respects preferred_order)
        if let Some(provider) = config.providers.get_active_provider() {
            match provider {
                super::config::AIProvider::Local => {
                    if let Some(ref path) = local_model_path {
                        tracing::info!("Creating Local GGUF backend with model: {}", path);
                        let llm_config = LLMCfg {
                            backend: LLMType::LocalGGUF,
                            api_key: None,
                            model: "local".to_string(),
                            max_tokens: config.llm.max_tokens as usize,
                            temperature: config.llm.temperature,
                            top_p: config.llm.top_p,
                            stream: config.streaming.enabled,
                            format_tool_results: true,
                            local_model_path: Some(path.clone()),
                            api_base_url: None,
                            context_size: Some(config.llm.context_size as usize),
                        };
                        return LLMFactory::create(llm_config);
                    }
                }
                super::config::AIProvider::OpenAI => {
                    if config.providers.openai.is_ready() {
                        tracing::info!("Creating OpenAI backend");
                        let llm_config = LLMCfg {
                            backend: LLMType::OpenAI,
                            api_key: config.providers.openai.api_key.clone(),
                            model: config.providers.openai.model_id.clone(),
                            max_tokens: config.llm.max_tokens as usize,
                            temperature: config.llm.temperature,
                            top_p: config.llm.top_p,
                            stream: config.streaming.enabled,
                            format_tool_results: true,
                            local_model_path: None,
                            api_base_url: config.providers.openai.base_url.clone(),
                            context_size: Some(config.llm.context_size as usize),
                        };
                        return LLMFactory::create(llm_config);
                    }
                }
                super::config::AIProvider::Anthropic => {
                    if config.providers.anthropic.is_ready() {
                        tracing::info!("Creating Anthropic backend");
                        let llm_config = LLMCfg {
                            backend: LLMType::Anthropic,
                            api_key: config.providers.anthropic.api_key.clone(),
                            model: config.providers.anthropic.model_id.clone(),
                            max_tokens: config.llm.max_tokens as usize,
                            temperature: config.llm.temperature,
                            top_p: config.llm.top_p,
                            stream: config.streaming.enabled,
                            format_tool_results: true,
                            local_model_path: None,
                            api_base_url: config.providers.anthropic.base_url.clone(),
                            context_size: Some(config.llm.context_size as usize),
                        };
                        return LLMFactory::create(llm_config);
                    }
                }
                _ => {}
            }
        }

        // Fallback: try local model even if not in preferred order
        if let Some(ref path) = local_model_path {
            tracing::info!("Using local GGUF model: {}", path);
            let llm_config = LLMCfg {
                backend: LLMType::LocalGGUF,
                api_key: None,
                model: "local".to_string(),
                max_tokens: config.llm.max_tokens as usize,
                temperature: config.llm.temperature,
                top_p: config.llm.top_p,
                stream: config.streaming.enabled,
                format_tool_results: true,
                local_model_path: Some(path.clone()),
                api_base_url: None,
                context_size: Some(config.llm.context_size as usize),
            };
            return LLMFactory::create(llm_config);
        }

        tracing::info!("No LLM provider configured, using UnconfiguredLLMBackend");
        LLMFactory::create(LLMCfg::default())
    }

    /// Find a local model path, checking config first then default locations
    fn find_local_model(config_path: &Option<String>) -> Option<String> {
        tracing::info!("Searching for local GGUF model...");

        // 1. Check if config has a path and it exists
        if let Some(ref path) = config_path {
            tracing::info!("Checking configured path: {}", path);
            if std::path::Path::new(path).exists() {
                tracing::info!("Using configured local model: {}", path);
                return Some(path.clone());
            } else {
                tracing::warn!("Configured path does not exist: {}", path);
            }
        } else {
            tracing::info!("No local model path configured, searching default locations...");
        }

        // 2. Check resources directory (bundled with app) - both relative and absolute
        let mut resource_paths = vec![
            "resources/models/qwen2-0_5b-instruct-q4_k_m.gguf".to_string(),
            "../resources/models/qwen2-0_5b-instruct-q4_k_m.gguf".to_string(),
            "src-tauri/resources/models/qwen2-0_5b-instruct-q4_k_m.gguf".to_string(),
        ];

        // Add absolute paths based on executable location
        if let Ok(exe_path) = std::env::current_exe() {
            if let Some(exe_dir) = exe_path.parent() {
                // For debug builds, go up to find resources
                let debug_path = exe_dir
                    .join("../../../gui/citrate-core/src-tauri/resources/models/qwen2-0_5b-instruct-q4_k_m.gguf");
                resource_paths.push(debug_path.to_string_lossy().to_string());

                // For release builds
                let release_path = exe_dir.join("resources/models/qwen2-0_5b-instruct-q4_k_m.gguf");
                resource_paths.push(release_path.to_string_lossy().to_string());

                // macOS app bundle path
                let bundle_path = exe_dir
                    .join("../Resources/models/qwen2-0_5b-instruct-q4_k_m.gguf");
                resource_paths.push(bundle_path.to_string_lossy().to_string());
            }
        }

        // Also check project models directory (for development)
        let project_model_paths = [
            "/Users/soleilklosowski/Downloads/citrate/citrate/gui/citrate-core/src-tauri/resources/models/qwen2-0_5b-instruct-q4_k_m.gguf",
            "/Users/soleilklosowski/Downloads/citrate/citrate/target/debug/models/qwen2-0_5b-instruct-q4_k_m.gguf",
            "/Users/soleilklosowski/Downloads/citrate/citrate/target/release/models/qwen2-0_5b-instruct-q4_k_m.gguf",
            "/Users/soleilklosowski/Downloads/citrate/citrate/models/Mistral-7B-Instruct-v0.3-Q4_K_M.gguf",
        ];
        resource_paths.extend(project_model_paths.iter().map(|s| s.to_string()));

        for rel_path in &resource_paths {
            let path = std::path::Path::new(rel_path);
            if path.exists() {
                // Canonicalize to get absolute path
                if let Ok(abs_path) = path.canonicalize() {
                    let path_str = abs_path.to_string_lossy().to_string();
                    tracing::info!("Found bundled model at: {}", path_str);
                    return Some(path_str);
                } else {
                    tracing::info!("Found bundled model at: {}", rel_path);
                    return Some(rel_path.clone());
                }
            }
        }

        // 3. Check default data directory
        if let Some(data_dir) = dirs::data_local_dir() {
            let model_dir = data_dir.join("citrate").join("models");
            if model_dir.exists() {
                // Scan for any .gguf file, prefer instruction-tuned models
                if let Ok(entries) = std::fs::read_dir(&model_dir) {
                    let mut models: Vec<_> = entries
                        .filter_map(|e| e.ok())
                        .filter(|e| e.path().extension().map_or(false, |ext| ext == "gguf"))
                        .map(|e| e.path())
                        .collect();

                    // Sort by preference: instruct models first, then by size
                    models.sort_by(|a, b| {
                        let a_name = a.file_name().unwrap_or_default().to_string_lossy().to_lowercase();
                        let b_name = b.file_name().unwrap_or_default().to_string_lossy().to_lowercase();
                        let a_instruct = a_name.contains("instruct");
                        let b_instruct = b_name.contains("instruct");

                        if a_instruct && !b_instruct {
                            std::cmp::Ordering::Less
                        } else if !a_instruct && b_instruct {
                            std::cmp::Ordering::Greater
                        } else {
                            // Compare by file size (larger = better quality usually)
                            let a_size = std::fs::metadata(a).map(|m| m.len()).unwrap_or(0);
                            let b_size = std::fs::metadata(b).map(|m| m.len()).unwrap_or(0);
                            b_size.cmp(&a_size)
                        }
                    });

                    if let Some(model_path) = models.first() {
                        let path_str = model_path.to_string_lossy().to_string();
                        tracing::info!("Found local model in data dir: {}", path_str);
                        return Some(path_str);
                    }
                }
            }
        }

        // 4. Check executable directory
        if let Ok(exe_path) = std::env::current_exe() {
            if let Some(exe_dir) = exe_path.parent() {
                let models_dir = exe_dir.join("models");
                if models_dir.exists() {
                    if let Ok(entries) = std::fs::read_dir(&models_dir) {
                        for entry in entries.flatten() {
                            let path = entry.path();
                            if path.extension().map_or(false, |ext| ext == "gguf") {
                                let path_str = path.to_string_lossy().to_string();
                                tracing::info!("Found model near executable: {}", path_str);
                                return Some(path_str);
                            }
                        }
                    }
                }
            }
        }

        tracing::warn!("No local GGUF model found");
        None
    }

    /// Get current configuration
    pub fn get_config(&self) -> &AgentConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: Full tests require mock managers
    // This is a placeholder for the test structure
    #[test]
    fn test_orchestrator_error_display() {
        let err = OrchestratorError::SessionNotFound("test".to_string());
        assert!(err.to_string().contains("test"));
    }
}
