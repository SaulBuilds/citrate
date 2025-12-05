//! API-based LLM backends (OpenAI, Anthropic)

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use super::{CompletionResponse, LLMBackend, LLMConfig, LLMError, TokenUsage};
use crate::agent::context::ContextWindow;

/// OpenAI API backend
pub struct OpenAIBackend {
    config: LLMConfig,
    client: reqwest::Client,
}

impl OpenAIBackend {
    pub fn new(config: LLMConfig) -> Self {
        Self {
            config,
            client: reqwest::Client::new(),
        }
    }

    fn base_url(&self) -> &str {
        self.config
            .api_base_url
            .as_deref()
            .unwrap_or("https://api.openai.com/v1")
    }
}

#[async_trait]
impl LLMBackend for OpenAIBackend {
    fn name(&self) -> &str {
        "openai"
    }

    async fn complete(&self, context: &ContextWindow) -> Result<String, LLMError> {
        let api_key = self
            .config
            .api_key
            .as_ref()
            .ok_or_else(|| LLMError("No API key configured".to_string()))?;

        // Build messages
        let mut messages = vec![serde_json::json!({
            "role": "system",
            "content": &context.system_prompt
        })];

        // Add system context if present
        if let Some(ref ctx) = context.system_context {
            messages.push(serde_json::json!({
                "role": "system",
                "content": format!("Current context:\n{}", ctx.to_context_string())
            }));
        }

        // Add conversation messages
        for msg in &context.messages {
            messages.push(serde_json::json!({
                "role": &msg.role,
                "content": &msg.content
            }));
        }

        let request_body = serde_json::json!({
            "model": &self.config.model,
            "messages": messages,
            "max_tokens": self.config.max_tokens,
            "temperature": self.config.temperature,
            "top_p": self.config.top_p
        });

        let response = self
            .client
            .post(format!("{}/chat/completions", self.base_url()))
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await
            .map_err(|e| LLMError(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(LLMError(format!("API error: {}", error_text)));
        }

        let response_json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| LLMError(format!("Failed to parse response: {}", e)))?;

        let content = response_json["choices"][0]["message"]["content"]
            .as_str()
            .ok_or_else(|| LLMError("No content in response".to_string()))?;

        Ok(content.to_string())
    }

    async fn complete_with_details(
        &self,
        context: &ContextWindow,
    ) -> Result<CompletionResponse, LLMError> {
        // For now, just wrap complete
        let text = self.complete(context).await?;
        Ok(CompletionResponse {
            text,
            usage: TokenUsage::default(),
            finish_reason: "stop".to_string(),
            model: self.config.model.clone(),
        })
    }

    fn is_available(&self) -> bool {
        self.config.api_key.is_some()
    }

    fn config(&self) -> &LLMConfig {
        &self.config
    }
}

/// Anthropic API backend
pub struct AnthropicBackend {
    config: LLMConfig,
    client: reqwest::Client,
}

impl AnthropicBackend {
    pub fn new(config: LLMConfig) -> Self {
        Self {
            config,
            client: reqwest::Client::new(),
        }
    }

    fn base_url(&self) -> &str {
        self.config
            .api_base_url
            .as_deref()
            .unwrap_or("https://api.anthropic.com/v1")
    }
}

#[async_trait]
impl LLMBackend for AnthropicBackend {
    fn name(&self) -> &str {
        "anthropic"
    }

    async fn complete(&self, context: &ContextWindow) -> Result<String, LLMError> {
        let api_key = self
            .config
            .api_key
            .as_ref()
            .ok_or_else(|| LLMError("No API key configured".to_string()))?;

        // Build system prompt with context
        let mut system = context.system_prompt.clone();
        if let Some(ref ctx) = context.system_context {
            system.push_str(&format!("\n\nCurrent context:\n{}", ctx.to_context_string()));
        }

        // Build messages (Anthropic format)
        let messages: Vec<serde_json::Value> = context
            .messages
            .iter()
            .map(|msg| {
                serde_json::json!({
                    "role": if msg.role == "assistant" { "assistant" } else { "user" },
                    "content": &msg.content
                })
            })
            .collect();

        let request_body = serde_json::json!({
            "model": &self.config.model,
            "system": system,
            "messages": messages,
            "max_tokens": self.config.max_tokens,
            "temperature": self.config.temperature
        });

        let response = self
            .client
            .post(format!("{}/messages", self.base_url()))
            .header("x-api-key", api_key)
            .header("anthropic-version", "2023-06-01")
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await
            .map_err(|e| LLMError(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(LLMError(format!("API error: {}", error_text)));
        }

        let response_json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| LLMError(format!("Failed to parse response: {}", e)))?;

        let content = response_json["content"][0]["text"]
            .as_str()
            .ok_or_else(|| LLMError("No content in response".to_string()))?;

        Ok(content.to_string())
    }

    fn is_available(&self) -> bool {
        self.config.api_key.is_some()
    }

    fn config(&self) -> &LLMConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_openai_availability() {
        let config = LLMConfig {
            api_key: None,
            ..Default::default()
        };
        let backend = OpenAIBackend::new(config);
        assert!(!backend.is_available());

        let config = LLMConfig {
            api_key: Some("test-key".to_string()),
            ..Default::default()
        };
        let backend = OpenAIBackend::new(config);
        assert!(backend.is_available());
    }

    #[test]
    fn test_anthropic_availability() {
        let config = LLMConfig {
            api_key: None,
            ..Default::default()
        };
        let backend = AnthropicBackend::new(config);
        assert!(!backend.is_available());
    }
}
