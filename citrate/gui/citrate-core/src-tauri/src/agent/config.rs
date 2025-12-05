// citrate-core/src-tauri/src/agent/config.rs
//
// Agent configuration

use serde::{Deserialize, Serialize};

/// LLM backend type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum LLMBackendType {
    /// Use OpenAI API
    OpenAI,
    /// Use Anthropic API (Claude)
    Anthropic,
    /// Use local GGUF model
    LocalGGUF,
    /// Auto-select: prefer local, fallback to API
    Auto,
}

impl Default for LLMBackendType {
    fn default() -> Self {
        Self::Auto
    }
}

/// LLM model configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMConfig {
    /// Backend type
    pub backend: LLMBackendType,
    /// Model identifier (e.g., "gpt-4", "claude-3", "mistral-7b-instruct.gguf")
    pub model_id: String,
    /// API key (for cloud backends)
    #[serde(skip_serializing)]
    pub api_key: Option<String>,
    /// API base URL (for custom endpoints)
    pub api_base_url: Option<String>,
    /// Temperature for generation
    pub temperature: f32,
    /// Maximum tokens to generate
    pub max_tokens: u32,
    /// Top-p sampling
    pub top_p: f32,
    /// Context window size
    pub context_size: u32,
}

impl Default for LLMConfig {
    fn default() -> Self {
        Self {
            backend: LLMBackendType::Auto,
            model_id: "auto".to_string(),
            api_key: None,
            api_base_url: None,
            temperature: 0.7,
            max_tokens: 2048,
            top_p: 0.9,
            context_size: 8192,
        }
    }
}

/// Intent classification configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassifierConfig {
    /// Confidence threshold for fast pattern matching
    /// If below this, fall back to LLM
    pub pattern_confidence_threshold: f32,
    /// Whether to use LLM fallback for low-confidence matches
    pub use_llm_fallback: bool,
    /// Cache size for classification results
    pub cache_size: usize,
}

impl Default for ClassifierConfig {
    fn default() -> Self {
        Self {
            pattern_confidence_threshold: 0.8,
            use_llm_fallback: true,
            cache_size: 1000,
        }
    }
}

/// Tool execution configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolConfig {
    /// Whether transactions require explicit user approval
    pub require_transaction_approval: bool,
    /// Whether contract deployments require explicit user approval
    pub require_deployment_approval: bool,
    /// Maximum gas limit for automatic transactions
    pub auto_gas_limit: u64,
    /// Timeout for tool execution in seconds
    pub execution_timeout_secs: u64,
}

impl Default for ToolConfig {
    fn default() -> Self {
        Self {
            require_transaction_approval: true,
            require_deployment_approval: true,
            auto_gas_limit: 100_000,
            execution_timeout_secs: 30,
        }
    }
}

/// Streaming configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamingConfig {
    /// Whether to stream tokens
    pub enabled: bool,
    /// Minimum delay between tokens in ms (for rate limiting)
    pub min_token_delay_ms: u64,
    /// Buffer size for tokens before flushing
    pub buffer_size: usize,
}

impl Default for StreamingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            min_token_delay_ms: 0,
            buffer_size: 1,
        }
    }
}

/// Context management configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextConfig {
    /// Maximum messages to keep in context
    pub max_messages: usize,
    /// Maximum tokens in context window
    pub max_context_tokens: u32,
    /// Whether to persist conversations
    pub persist_conversations: bool,
    /// Directory for conversation storage
    pub storage_dir: Option<String>,
}

impl Default for ContextConfig {
    fn default() -> Self {
        Self {
            max_messages: 50,
            max_context_tokens: 4096,
            persist_conversations: true,
            storage_dir: None,
        }
    }
}

/// Main agent configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AgentConfig {
    /// LLM configuration
    pub llm: LLMConfig,
    /// Intent classifier configuration
    pub classifier: ClassifierConfig,
    /// Tool execution configuration
    pub tools: ToolConfig,
    /// Streaming configuration
    pub streaming: StreamingConfig,
    /// Context management configuration
    pub context: ContextConfig,
    /// System prompt to prepend to conversations
    pub system_prompt: Option<String>,
    /// Whether agent is enabled
    pub enabled: bool,
}

impl AgentConfig {
    /// Create a new config with default system prompt
    pub fn with_defaults() -> Self {
        let mut config = Self::default();
        config.enabled = true;
        config.system_prompt = Some(Self::default_system_prompt());
        config
    }

    /// Get the default system prompt
    pub fn default_system_prompt() -> String {
        r#"You are Citrate AI, an intelligent assistant for the Citrate blockchain. You help users:

- Query their wallet balance and transaction history
- Send transactions and interact with smart contracts
- Deploy and interact with AI models on-chain
- Explore the DAG structure and block information
- Understand blockchain concepts and Citrate-specific features

You have access to tools that can:
- Query blockchain state (balances, blocks, transactions)
- Send transactions (with user approval)
- Deploy and call smart contracts
- Run AI model inference
- Explore the DAG visualization

Always be helpful, accurate, and security-conscious. For transactions involving value transfer, always confirm with the user before executing.

Current context:
- Network: Citrate Testnet
- Consensus: GhostDAG
- Block time: ~2 seconds
- Finality: ~12 seconds"#.to_string()
    }

    /// Load config from file
    pub fn load(path: &str) -> Result<Self, anyhow::Error> {
        let content = std::fs::read_to_string(path)?;
        let config: AgentConfig = serde_json::from_str(&content)?;
        Ok(config)
    }

    /// Save config to file
    pub fn save(&self, path: &str) -> Result<(), anyhow::Error> {
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }
}
