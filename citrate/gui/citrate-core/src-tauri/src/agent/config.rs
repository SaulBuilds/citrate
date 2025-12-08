// citrate-core/src-tauri/src/agent/config.rs
//
// Agent configuration with multi-provider API support
//
// Supports:
// - OpenAI (GPT-4, GPT-3.5)
// - Anthropic (Claude 3)
// - Google Gemini
// - xAI (Grok)
// - Local GGUF models

use serde::{Deserialize, Serialize};

/// AI Provider enumeration
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum AIProvider {
    /// OpenAI (GPT-4, GPT-3.5-turbo)
    OpenAI,
    /// Anthropic (Claude 3 Opus, Sonnet, Haiku)
    Anthropic,
    /// Google Gemini (Gemini Pro, Gemini Ultra)
    Gemini,
    /// xAI (Grok)
    XAI,
    /// Local GGUF model
    Local,
}

impl Default for AIProvider {
    fn default() -> Self {
        Self::Local
    }
}

impl std::fmt::Display for AIProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AIProvider::OpenAI => write!(f, "OpenAI"),
            AIProvider::Anthropic => write!(f, "Anthropic"),
            AIProvider::Gemini => write!(f, "Google Gemini"),
            AIProvider::XAI => write!(f, "xAI"),
            AIProvider::Local => write!(f, "Local Model"),
        }
    }
}

/// Provider-specific settings
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProviderSettings {
    /// API key for this provider (stored securely, not serialized)
    #[serde(skip_serializing)]
    pub api_key: Option<String>,
    /// Whether this provider is enabled
    pub enabled: bool,
    /// Default model ID for this provider
    pub model_id: String,
    /// Custom API base URL (for proxies or self-hosted)
    pub base_url: Option<String>,
    /// Whether connection has been verified
    #[serde(skip)]
    pub verified: bool,
}

impl ProviderSettings {
    /// Create new provider settings
    pub fn new(model_id: &str) -> Self {
        Self {
            api_key: None,
            enabled: false,
            model_id: model_id.to_string(),
            base_url: None,
            verified: false,
        }
    }

    /// Check if this provider is configured and ready to use
    pub fn is_ready(&self) -> bool {
        self.enabled && self.api_key.is_some()
    }
}

/// Multi-provider AI configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIProvidersConfig {
    /// OpenAI settings
    pub openai: ProviderSettings,
    /// Anthropic settings
    pub anthropic: ProviderSettings,
    /// Google Gemini settings
    pub gemini: ProviderSettings,
    /// xAI settings
    pub xai: ProviderSettings,
    /// Preferred provider order (first available will be used)
    pub preferred_order: Vec<AIProvider>,
    /// Always fallback to local model if cloud providers fail
    pub local_fallback: bool,
    /// Local model path (GGUF file)
    pub local_model_path: Option<String>,
    /// Local model IPFS CID (for re-download)
    pub local_model_cid: Option<String>,
}

impl Default for AIProvidersConfig {
    fn default() -> Self {
        Self {
            openai: ProviderSettings::new("gpt-4o-mini"),
            anthropic: ProviderSettings::new("claude-3-haiku-20240307"),
            gemini: ProviderSettings::new("gemini-1.5-flash"),
            xai: ProviderSettings::new("grok-beta"),
            preferred_order: vec![AIProvider::Local, AIProvider::OpenAI, AIProvider::Anthropic],
            local_fallback: true,
            local_model_path: None,
            local_model_cid: None,
        }
    }
}

impl AIProvidersConfig {
    /// Get the first available provider based on preference order
    pub fn get_active_provider(&self) -> Option<AIProvider> {
        for provider in &self.preferred_order {
            match provider {
                AIProvider::OpenAI if self.openai.is_ready() => return Some(AIProvider::OpenAI),
                AIProvider::Anthropic if self.anthropic.is_ready() => return Some(AIProvider::Anthropic),
                AIProvider::Gemini if self.gemini.is_ready() => return Some(AIProvider::Gemini),
                AIProvider::XAI if self.xai.is_ready() => return Some(AIProvider::XAI),
                AIProvider::Local if self.local_model_path.is_some() => return Some(AIProvider::Local),
                _ => continue,
            }
        }

        // Fallback to local if enabled
        if self.local_fallback && self.local_model_path.is_some() {
            return Some(AIProvider::Local);
        }

        None
    }

    /// Get settings for a specific provider
    pub fn get_provider_settings(&self, provider: AIProvider) -> Option<&ProviderSettings> {
        match provider {
            AIProvider::OpenAI => Some(&self.openai),
            AIProvider::Anthropic => Some(&self.anthropic),
            AIProvider::Gemini => Some(&self.gemini),
            AIProvider::XAI => Some(&self.xai),
            AIProvider::Local => None, // Local doesn't use ProviderSettings
        }
    }

    /// Get mutable settings for a specific provider
    pub fn get_provider_settings_mut(&mut self, provider: AIProvider) -> Option<&mut ProviderSettings> {
        match provider {
            AIProvider::OpenAI => Some(&mut self.openai),
            AIProvider::Anthropic => Some(&mut self.anthropic),
            AIProvider::Gemini => Some(&mut self.gemini),
            AIProvider::XAI => Some(&mut self.xai),
            AIProvider::Local => None,
        }
    }

    /// Set API key for a provider
    pub fn set_api_key(&mut self, provider: AIProvider, key: String) {
        if let Some(settings) = self.get_provider_settings_mut(provider) {
            settings.api_key = Some(key);
            settings.enabled = true;
        }
    }

    /// Get API base URL for a provider
    pub fn get_base_url(&self, provider: AIProvider) -> &str {
        match provider {
            AIProvider::OpenAI => self.openai.base_url.as_deref().unwrap_or("https://api.openai.com/v1"),
            AIProvider::Anthropic => self.anthropic.base_url.as_deref().unwrap_or("https://api.anthropic.com"),
            AIProvider::Gemini => self.gemini.base_url.as_deref().unwrap_or("https://generativelanguage.googleapis.com/v1beta"),
            AIProvider::XAI => self.xai.base_url.as_deref().unwrap_or("https://api.x.ai/v1"),
            AIProvider::Local => "",
        }
    }

    /// Get model ID for a provider
    pub fn get_model_id(&self, provider: AIProvider) -> &str {
        match provider {
            AIProvider::OpenAI => &self.openai.model_id,
            AIProvider::Anthropic => &self.anthropic.model_id,
            AIProvider::Gemini => &self.gemini.model_id,
            AIProvider::XAI => &self.xai.model_id,
            AIProvider::Local => self.local_model_path.as_deref().unwrap_or(""),
        }
    }
}

/// LLM backend type (legacy, kept for compatibility)
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
    /// LLM configuration (legacy, prefer using providers)
    pub llm: LLMConfig,
    /// Multi-provider AI configuration
    #[serde(default)]
    pub providers: AIProvidersConfig,
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
    /// Whether onboarding has been completed
    #[serde(default)]
    pub onboarding_completed: bool,
    /// First run flag
    #[serde(default)]
    pub first_run: bool,
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
