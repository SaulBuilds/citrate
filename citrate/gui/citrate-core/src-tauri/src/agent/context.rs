//! Context management for agent conversations
//!
//! Handles conversation history, context windows, and token management
//! for maintaining coherent multi-turn conversations.

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

use super::config::ContextConfig;
use super::session::Message;

/// A window of conversation context for LLM calls
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextWindow {
    /// System prompt/instructions
    pub system_prompt: String,
    /// System context (node status, wallet info, etc.)
    pub system_context: Option<SystemContext>,
    /// Messages included in this context window
    pub messages: Vec<ContextMessage>,
    /// Estimated token count
    pub estimated_tokens: usize,
    /// Whether context was truncated
    pub was_truncated: bool,
}

/// System context injected into conversations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemContext {
    /// Current wallet address
    pub wallet_address: Option<String>,
    /// Current wallet balance
    pub wallet_balance: Option<String>,
    /// Node connection status
    pub node_connected: bool,
    /// Current block height
    pub block_height: Option<u64>,
    /// Network name
    pub network: String,
    /// Available models
    pub available_models: Vec<String>,
}

impl SystemContext {
    /// Format system context as a string for injection
    pub fn to_context_string(&self) -> String {
        let mut parts = Vec::new();

        parts.push(format!("Network: {}", self.network));
        parts.push(format!(
            "Node Status: {}",
            if self.node_connected {
                "Connected"
            } else {
                "Disconnected"
            }
        ));

        if let Some(height) = self.block_height {
            parts.push(format!("Block Height: {}", height));
        }

        if let Some(addr) = &self.wallet_address {
            parts.push(format!("Wallet: {}", addr));
        }

        if let Some(balance) = &self.wallet_balance {
            parts.push(format!("Balance: {}", balance));
        }

        if !self.available_models.is_empty() {
            parts.push(format!("Available Models: {}", self.available_models.join(", ")));
        }

        parts.join("\n")
    }
}

/// A message formatted for context inclusion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextMessage {
    pub role: String,
    pub content: String,
    pub name: Option<String>,
    pub tool_call_id: Option<String>,
}

impl From<&Message> for ContextMessage {
    fn from(msg: &Message) -> Self {
        Self {
            role: msg.role.to_string(),
            content: msg.content.clone(),
            name: None,
            tool_call_id: msg.tool_call_id.clone(),
        }
    }
}

/// Manages conversation history with smart truncation
#[derive(Debug)]
pub struct ConversationHistory {
    /// All messages in the conversation
    messages: VecDeque<Message>,
    /// Maximum messages to retain
    max_messages: usize,
}

impl ConversationHistory {
    /// Create a new conversation history
    pub fn new() -> Self {
        Self {
            messages: VecDeque::new(),
            max_messages: 1000,
        }
    }

    /// Create with custom max messages
    pub fn with_max_messages(max_messages: usize) -> Self {
        Self {
            messages: VecDeque::new(),
            max_messages,
        }
    }

    /// Add a message to history
    pub fn add_message(&mut self, message: Message) {
        self.messages.push_back(message);

        // Trim if we exceed max messages
        while self.messages.len() > self.max_messages {
            self.messages.pop_front();
        }
    }

    /// Get all messages (cloned)
    pub fn messages(&self) -> Vec<Message> {
        self.messages.iter().cloned().collect()
    }

    /// Get all messages as refs
    pub fn messages_ref(&self) -> Vec<&Message> {
        self.messages.iter().collect()
    }

    /// Get recent messages (returns cloned)
    pub fn recent(&self, count: usize) -> Vec<Message> {
        self.messages.iter().rev().take(count).rev().cloned().collect()
    }

    /// Get recent messages (returns refs)
    pub fn recent_refs(&self, count: usize) -> Vec<&Message> {
        self.messages.iter().rev().take(count).rev().collect()
    }

    /// Clear history
    pub fn clear(&mut self) {
        self.messages.clear();
    }

    /// Get message count
    pub fn len(&self) -> usize {
        self.messages.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }

    /// Build a context window for LLM calls
    pub fn build_context_window(
        &self,
        system_prompt: &str,
        system_context: Option<SystemContext>,
        max_tokens: usize,
        min_recent: usize,
    ) -> ContextWindow {
        let mut context_messages = Vec::new();
        let mut estimated_tokens = estimate_tokens(system_prompt);
        let mut was_truncated = false;

        // Add system context tokens
        if let Some(ref ctx) = system_context {
            estimated_tokens += estimate_tokens(&ctx.to_context_string());
        }

        // Always include minimum recent messages
        let min_messages: Vec<_> = self
            .messages
            .iter()
            .rev()
            .take(min_recent)
            .rev()
            .collect();

        // Add remaining messages until we hit token limit
        let remaining_budget = max_tokens.saturating_sub(estimated_tokens);
        let mut remaining_tokens = remaining_budget;

        for msg in min_messages {
            let msg_tokens = estimate_tokens(&msg.content);
            context_messages.push(ContextMessage::from(msg));
            remaining_tokens = remaining_tokens.saturating_sub(msg_tokens);
            estimated_tokens += msg_tokens;
        }

        // Try to add older messages if we have budget
        let older_messages: Vec<_> = self
            .messages
            .iter()
            .rev()
            .skip(min_recent)
            .collect();

        for msg in older_messages.into_iter().rev() {
            let msg_tokens = estimate_tokens(&msg.content);
            if remaining_tokens >= msg_tokens {
                context_messages.insert(0, ContextMessage::from(msg));
                remaining_tokens = remaining_tokens.saturating_sub(msg_tokens);
                estimated_tokens += msg_tokens;
            } else {
                was_truncated = true;
                break;
            }
        }

        ContextWindow {
            system_prompt: system_prompt.to_string(),
            system_context,
            messages: context_messages,
            estimated_tokens,
            was_truncated,
        }
    }
}

impl Default for ConversationHistory {
    fn default() -> Self {
        Self::new()
    }
}

/// Context manager for handling multiple sessions
pub struct ContextManager {
    /// Default config
    config: ContextConfig,
}

impl ContextManager {
    pub fn new(config: ContextConfig) -> Self {
        Self { config }
    }

    pub fn config(&self) -> &ContextConfig {
        &self.config
    }
}

impl Default for ContextManager {
    fn default() -> Self {
        Self::new(ContextConfig::default())
    }
}

/// Simple token estimation (roughly 4 chars per token)
fn estimate_tokens(text: &str) -> usize {
    (text.len() + 3) / 4
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::session::MessageRole;

    fn create_test_message(role: MessageRole, content: &str) -> Message {
        match role {
            MessageRole::User => Message::user(content.to_string()),
            MessageRole::Assistant => Message::assistant(content.to_string()),
            MessageRole::System => Message::system(content.to_string()),
            _ => Message::user(content.to_string()),
        }
    }

    #[test]
    fn test_conversation_history() {
        let mut history = ConversationHistory::new();

        history.add_message(create_test_message(MessageRole::User, "Hello"));
        history.add_message(create_test_message(MessageRole::Assistant, "Hi there!"));

        assert_eq!(history.len(), 2);
        assert!(!history.is_empty());
    }

    #[test]
    fn test_context_window_building() {
        let mut history = ConversationHistory::new();

        history.add_message(create_test_message(MessageRole::User, "First message"));
        history.add_message(create_test_message(MessageRole::Assistant, "First reply"));
        history.add_message(create_test_message(MessageRole::User, "Second message"));
        history.add_message(create_test_message(MessageRole::Assistant, "Second reply"));

        let window = history.build_context_window("You are a helpful assistant.", None, 1000, 2);

        assert!(!window.messages.is_empty());
        assert!(window.estimated_tokens > 0);
    }

    #[test]
    fn test_system_context_formatting() {
        let ctx = SystemContext {
            wallet_address: Some("0x1234...5678".to_string()),
            wallet_balance: Some("100 CTR".to_string()),
            node_connected: true,
            block_height: Some(12345),
            network: "devnet".to_string(),
            available_models: vec!["llama-7b".to_string()],
        };

        let formatted = ctx.to_context_string();
        assert!(formatted.contains("devnet"));
        assert!(formatted.contains("Connected"));
        assert!(formatted.contains("12345"));
    }
}
