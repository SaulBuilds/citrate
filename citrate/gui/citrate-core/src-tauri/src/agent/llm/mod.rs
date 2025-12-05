//! LLM backend abstraction layer
//!
//! Supports multiple backends:
//! - OpenAI API
//! - Anthropic API
//! - Local GGUF models

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use super::context::ContextWindow;

pub mod api;
pub mod local;

/// LLM configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMConfig {
    /// Backend type
    pub backend: LLMBackendType,
    /// API key (for OpenAI/Anthropic)
    pub api_key: Option<String>,
    /// Model name
    pub model: String,
    /// Max tokens to generate
    pub max_tokens: usize,
    /// Temperature (0.0 - 2.0)
    pub temperature: f32,
    /// Top-p sampling
    pub top_p: f32,
    /// Whether to stream responses
    pub stream: bool,
    /// Whether to format tool results with LLM
    pub format_tool_results: bool,
    /// Local model path (for GGUF)
    pub local_model_path: Option<String>,
    /// API base URL override
    pub api_base_url: Option<String>,
    /// Context window size
    pub context_size: Option<usize>,
}

impl Default for LLMConfig {
    fn default() -> Self {
        Self {
            backend: LLMBackendType::Auto,
            api_key: None,
            model: "gpt-4".to_string(),
            max_tokens: 2048,
            temperature: 0.7,
            top_p: 1.0,
            stream: true,
            format_tool_results: true,
            local_model_path: None,
            api_base_url: None,
            context_size: Some(8192),
        }
    }
}

/// Type of LLM backend
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum LLMBackendType {
    /// OpenAI API
    OpenAI,
    /// Anthropic API
    Anthropic,
    /// Local GGUF model
    LocalGGUF,
    /// Auto-select based on availability
    Auto,
}

/// Error from LLM operations
#[derive(Debug, Clone)]
pub struct LLMError(pub String);

impl std::fmt::Display for LLMError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "LLM error: {}", self.0)
    }
}

impl std::error::Error for LLMError {}

/// Token usage information
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TokenUsage {
    pub prompt_tokens: usize,
    pub completion_tokens: usize,
    pub total_tokens: usize,
}

/// Completion response
#[derive(Debug, Clone)]
pub struct CompletionResponse {
    /// Generated text
    pub text: String,
    /// Token usage
    pub usage: TokenUsage,
    /// Finish reason
    pub finish_reason: String,
    /// Model used
    pub model: String,
}

/// LLM backend trait
#[async_trait]
pub trait LLMBackend: Send + Sync {
    /// Get backend name
    fn name(&self) -> &str;

    /// Complete a prompt
    async fn complete(&self, context: &ContextWindow) -> Result<String, LLMError>;

    /// Complete with full response details
    async fn complete_with_details(
        &self,
        context: &ContextWindow,
    ) -> Result<CompletionResponse, LLMError> {
        let text = self.complete(context).await?;
        Ok(CompletionResponse {
            text,
            usage: TokenUsage::default(),
            finish_reason: "stop".to_string(),
            model: self.name().to_string(),
        })
    }

    /// Check if backend is available
    fn is_available(&self) -> bool {
        true
    }

    /// Get configuration
    fn config(&self) -> &LLMConfig;
}

/// Unconfigured LLM backend - fails with helpful error message
/// This replaces the mock backend to ensure users configure a real LLM
pub struct UnconfiguredLLMBackend {
    config: LLMConfig,
}

impl UnconfiguredLLMBackend {
    pub fn new() -> Self {
        Self {
            config: LLMConfig::default(),
        }
    }
}

impl Default for UnconfiguredLLMBackend {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl LLMBackend for UnconfiguredLLMBackend {
    fn name(&self) -> &str {
        "unconfigured"
    }

    async fn complete(&self, _context: &ContextWindow) -> Result<String, LLMError> {
        Err(LLMError(
            "No LLM backend configured. Please configure one of the following:\n\
             1. Set OPENAI_API_KEY environment variable for OpenAI\n\
             2. Set ANTHROPIC_API_KEY environment variable for Anthropic\n\
             3. Provide a local_model_path for GGUF models\n\
             \n\
             Configure the LLM in Settings > AI Configuration.".to_string()
        ))
    }

    fn is_available(&self) -> bool {
        false
    }

    fn config(&self) -> &LLMConfig {
        &self.config
    }
}

/// Mock LLM backend for testing ONLY
/// Not used in production - use UnconfiguredLLMBackend as fallback
#[cfg(test)]
pub struct MockLLMBackend {
    config: LLMConfig,
    response: String,
}

#[cfg(test)]
impl MockLLMBackend {
    pub fn new() -> Self {
        Self {
            config: LLMConfig::default(),
            response: "This is a mock response from the AI assistant.".to_string(),
        }
    }

    pub fn with_response(response: String) -> Self {
        Self {
            config: LLMConfig::default(),
            response,
        }
    }
}

#[cfg(test)]
impl Default for MockLLMBackend {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[async_trait]
impl LLMBackend for MockLLMBackend {
    fn name(&self) -> &str {
        "mock"
    }

    async fn complete(&self, _context: &ContextWindow) -> Result<String, LLMError> {
        Ok(self.response.clone())
    }

    fn config(&self) -> &LLMConfig {
        &self.config
    }
}

/// Factory for creating LLM backends
pub struct LLMFactory;

impl LLMFactory {
    /// Create an LLM backend from config
    pub fn create(config: LLMConfig) -> Box<dyn LLMBackend + Send + Sync> {
        match config.backend {
            LLMBackendType::OpenAI => {
                Box::new(api::OpenAIBackend::new(config))
            }
            LLMBackendType::Anthropic => {
                Box::new(api::AnthropicBackend::new(config))
            }
            LLMBackendType::LocalGGUF => {
                Box::new(local::GGUFBackend::new(config))
            }
            LLMBackendType::Auto => {
                // Try local first, then API - fail with helpful error if none configured
                if config.local_model_path.is_some() {
                    Box::new(local::GGUFBackend::new(config))
                } else if config.api_key.is_some() {
                    Box::new(api::OpenAIBackend::new(config))
                } else {
                    // No configuration - return unconfigured backend that fails with helpful error
                    Box::new(UnconfiguredLLMBackend::new())
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_backend() {
        let backend = MockLLMBackend::new();

        let context = ContextWindow {
            system_prompt: "You are helpful.".to_string(),
            system_context: None,
            messages: vec![],
            estimated_tokens: 0,
            was_truncated: false,
        };

        let response = backend.complete(&context).await.unwrap();
        assert!(!response.is_empty());
    }

    #[tokio::test]
    async fn test_unconfigured_backend_fails() {
        let backend = UnconfiguredLLMBackend::new();

        let context = ContextWindow {
            system_prompt: "You are helpful.".to_string(),
            system_context: None,
            messages: vec![],
            estimated_tokens: 0,
            was_truncated: false,
        };

        let result = backend.complete(&context).await;
        assert!(result.is_err());
        assert!(!backend.is_available());
    }

    #[test]
    fn test_factory_auto_selection() {
        let config = LLMConfig {
            backend: LLMBackendType::Auto,
            ..Default::default()
        };

        let backend = LLMFactory::create(config);
        // Falls back to unconfigured when no key/path
        assert_eq!(backend.name(), "unconfigured");
        assert!(!backend.is_available());
    }
}
