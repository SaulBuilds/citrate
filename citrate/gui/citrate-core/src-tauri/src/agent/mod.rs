// citrate-core/src-tauri/src/agent/mod.rs
//
// Agent Module - AI-first conversational interface for Citrate
//
// This module provides the core agent infrastructure that powers
// the AI-first GUI transformation. It includes:
// - Intent classification (fast patterns + LLM fallback)
// - Tool dispatch with MCP bindings
// - Streaming response infrastructure
// - Conversation context management
// - Hybrid LLM support (API + local GGUF)

pub mod classifier;
pub mod commands;
pub mod config;
pub mod context;
pub mod dispatcher;
pub mod formatting;
pub mod intent;
pub mod llm;
pub mod onboarding;
pub mod orchestrator;
pub mod session;
pub mod streaming;
pub mod tools;

// Re-exports for convenient access
pub use classifier::IntentClassifier;
pub use commands::AgentState;
pub use config::{
    AgentConfig, AIProvider, AIProvidersConfig, ProviderSettings,
    LLMBackendType, LLMConfig, ClassifierConfig, ToolConfig,
    StreamingConfig, ContextConfig,
};
pub use context::{ContextManager, ContextWindow, ConversationHistory};
pub use dispatcher::ToolDispatcher;
pub use formatting::{FormattedResult, ResultCategory};
pub use intent::{Intent, IntentMatch, IntentParams};
pub use onboarding::{OnboardingManager, SkillLevel, UserAssessment, AssessmentResponse};
pub use orchestrator::AgentOrchestrator;
pub use session::{AgentSession, SessionId};
pub use streaming::{StreamToken, StreamingResponse};

use std::sync::Arc;
use tokio::sync::RwLock;

use crate::node::NodeManager;
use crate::wallet::WalletManager;
use crate::models::ModelManager;
use crate::dag::DAGManager;

/// Agent manager that coordinates all agent components
pub struct AgentManager {
    /// The agent orchestrator
    orchestrator: Arc<RwLock<AgentOrchestrator>>,
    /// Agent configuration
    config: Arc<RwLock<AgentConfig>>,
}

impl AgentManager {
    /// Create a new agent manager with access to blockchain components
    pub fn new(
        node_manager: Arc<NodeManager>,
        wallet_manager: Arc<WalletManager>,
        model_manager: Arc<ModelManager>,
        dag_manager: Arc<RwLock<Option<Arc<DAGManager>>>>,
    ) -> Self {
        let config = AgentConfig::default();
        let orchestrator = AgentOrchestrator::new(
            config.clone(),
            node_manager,
            wallet_manager,
            model_manager,
            dag_manager,
        );

        Self {
            orchestrator: Arc::new(RwLock::new(orchestrator)),
            config: Arc::new(RwLock::new(config)),
        }
    }

    /// Get a reference to the orchestrator
    pub fn orchestrator(&self) -> Arc<RwLock<AgentOrchestrator>> {
        self.orchestrator.clone()
    }

    /// Get a reference to the config
    pub fn config(&self) -> Arc<RwLock<AgentConfig>> {
        self.config.clone()
    }

    /// Update agent configuration
    pub async fn update_config(&self, config: AgentConfig) {
        *self.config.write().await = config.clone();
        self.orchestrator.write().await.update_config(config);
    }

    /// Configure a local model path and update the LLM backend
    /// Returns true if the model was configured successfully
    pub async fn configure_local_model(&self, model_path: String) -> bool {
        let mut config = self.config.write().await;
        config.providers.local_model_path = Some(model_path.clone());

        // Update the orchestrator to recreate the LLM backend
        self.orchestrator.write().await.update_config(config.clone());

        tracing::info!("Local model configured: {}", model_path);
        true
    }

    /// Check if the agent has a working LLM backend configured
    pub async fn has_llm_backend(&self) -> bool {
        let config = self.config.read().await;
        config.providers.local_model_path.is_some()
            || config.providers.openai.is_ready()
            || config.providers.anthropic.is_ready()
            || config.providers.gemini.is_ready()
            || config.providers.xai.is_ready()
    }
}
