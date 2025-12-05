// citrate-core/src-tauri/src/agent/session.rs
//
// Agent session management

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;

use super::context::ConversationHistory;
use super::intent::IntentMatch;

/// Unique session identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SessionId(pub String);

impl SessionId {
    /// Create a new random session ID
    pub fn new() -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();
        let random: u32 = rand::random();
        Self(format!("session_{:x}_{:08x}", timestamp, random))
    }

    /// Create from string
    pub fn from_string(s: String) -> Self {
        Self(s)
    }
}

impl Default for SessionId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for SessionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Message role in conversation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    /// User message
    User,
    /// Assistant (agent) message
    Assistant,
    /// System message
    System,
    /// Tool invocation
    Tool,
    /// Tool result
    ToolResult,
}

impl std::fmt::Display for MessageRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MessageRole::User => write!(f, "user"),
            MessageRole::Assistant => write!(f, "assistant"),
            MessageRole::System => write!(f, "system"),
            MessageRole::Tool => write!(f, "tool"),
            MessageRole::ToolResult => write!(f, "tool_result"),
        }
    }
}

/// A message in the conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Unique message ID
    pub id: String,
    /// Message role
    pub role: MessageRole,
    /// Message content
    pub content: String,
    /// Timestamp (Unix ms)
    pub timestamp: u64,
    /// Intent classification (for user messages)
    pub intent: Option<IntentMatch>,
    /// Tool call ID (for tool messages)
    pub tool_call_id: Option<String>,
    /// Tool name (for tool messages)
    pub tool_name: Option<String>,
    /// Token usage
    pub tokens: Option<TokenUsage>,
    /// Whether message is still streaming
    pub is_streaming: bool,
    /// Metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

impl Message {
    /// Create a new user message
    pub fn user(content: String) -> Self {
        Self {
            id: Self::generate_id(),
            role: MessageRole::User,
            content,
            timestamp: Self::now(),
            intent: None,
            tool_call_id: None,
            tool_name: None,
            tokens: None,
            is_streaming: false,
            metadata: HashMap::new(),
        }
    }

    /// Create a new assistant message
    pub fn assistant(content: String) -> Self {
        Self {
            id: Self::generate_id(),
            role: MessageRole::Assistant,
            content,
            timestamp: Self::now(),
            intent: None,
            tool_call_id: None,
            tool_name: None,
            tokens: None,
            is_streaming: false,
            metadata: HashMap::new(),
        }
    }

    /// Create a new system message
    pub fn system(content: String) -> Self {
        Self {
            id: Self::generate_id(),
            role: MessageRole::System,
            content,
            timestamp: Self::now(),
            intent: None,
            tool_call_id: None,
            tool_name: None,
            tokens: None,
            is_streaming: false,
            metadata: HashMap::new(),
        }
    }

    /// Create a new tool call message
    pub fn tool_call(tool_name: String, tool_call_id: String, content: String) -> Self {
        Self {
            id: Self::generate_id(),
            role: MessageRole::Tool,
            content,
            timestamp: Self::now(),
            intent: None,
            tool_call_id: Some(tool_call_id),
            tool_name: Some(tool_name),
            tokens: None,
            is_streaming: false,
            metadata: HashMap::new(),
        }
    }

    /// Create a new tool result message
    pub fn tool_result(tool_call_id: String, content: String) -> Self {
        Self {
            id: Self::generate_id(),
            role: MessageRole::ToolResult,
            content,
            timestamp: Self::now(),
            intent: None,
            tool_call_id: Some(tool_call_id),
            tool_name: None,
            tokens: None,
            is_streaming: false,
            metadata: HashMap::new(),
        }
    }

    /// Mark as streaming
    pub fn streaming(mut self) -> Self {
        self.is_streaming = true;
        self
    }

    /// Set intent
    pub fn with_intent(mut self, intent: IntentMatch) -> Self {
        self.intent = Some(intent);
        self
    }

    /// Set tokens
    pub fn with_tokens(mut self, tokens: TokenUsage) -> Self {
        self.tokens = Some(tokens);
        self
    }

    fn generate_id() -> String {
        let random: u64 = rand::random();
        format!("msg_{:016x}", random)
    }

    fn now() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64
    }
}

/// Token usage statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TokenUsage {
    /// Prompt tokens
    pub prompt_tokens: u32,
    /// Completion tokens
    pub completion_tokens: u32,
    /// Total tokens
    pub total_tokens: u32,
}

/// Session state
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionState {
    /// Session is active and ready
    Active,
    /// Session is processing a request
    Processing,
    /// Session is waiting for user input (e.g., tool approval)
    WaitingForInput,
    /// Session is paused
    Paused,
    /// Session is closed
    Closed,
}

/// Pending tool call awaiting approval
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingToolCall {
    /// Tool call ID
    pub id: String,
    /// Tool name
    pub tool_name: String,
    /// Tool parameters
    pub params: serde_json::Value,
    /// Description for user
    pub description: String,
    /// Whether this is a high-risk operation
    pub high_risk: bool,
    /// Timestamp when created
    pub created_at: u64,
}

/// An agent conversation session
pub struct AgentSession {
    /// Session ID
    pub id: SessionId,
    /// Session state
    state: RwLock<SessionState>,
    /// Conversation history
    history: RwLock<ConversationHistory>,
    /// Pending tool calls awaiting approval
    pending_tools: RwLock<Vec<PendingToolCall>>,
    /// Session metadata
    metadata: RwLock<HashMap<String, serde_json::Value>>,
    /// Created timestamp
    created_at: u64,
    /// Last activity timestamp
    last_activity: RwLock<u64>,
}

impl AgentSession {
    /// Create a new session
    pub fn new() -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        Self {
            id: SessionId::new(),
            state: RwLock::new(SessionState::Active),
            history: RwLock::new(ConversationHistory::new()),
            pending_tools: RwLock::new(Vec::new()),
            metadata: RwLock::new(HashMap::new()),
            created_at: now,
            last_activity: RwLock::new(now),
        }
    }

    /// Create a session with a specific ID
    pub fn with_id(id: SessionId) -> Self {
        let mut session = Self::new();
        session.id = id;
        session
    }

    /// Get the session ID
    pub fn id(&self) -> &SessionId {
        &self.id
    }

    /// Get the current state
    pub async fn state(&self) -> SessionState {
        self.state.read().await.clone()
    }

    /// Set the session state
    pub async fn set_state(&self, state: SessionState) {
        *self.state.write().await = state;
        self.touch().await;
    }

    /// Add a message to the history
    pub async fn add_message(&self, message: Message) {
        self.history.write().await.add_message(message);
        self.touch().await;
    }

    /// Get all messages
    pub async fn messages(&self) -> Vec<Message> {
        self.history.read().await.messages().clone()
    }

    /// Get recent messages (for context window)
    pub async fn recent_messages(&self, count: usize) -> Vec<Message> {
        self.history.read().await.recent(count)
    }

    /// Clear the conversation history
    pub async fn clear_history(&self) {
        self.history.write().await.clear();
        self.touch().await;
    }

    /// Add a pending tool call
    pub async fn add_pending_tool(&self, tool_call: PendingToolCall) {
        self.pending_tools.write().await.push(tool_call);
        *self.state.write().await = SessionState::WaitingForInput;
        self.touch().await;
    }

    /// Get pending tool calls
    pub async fn pending_tools(&self) -> Vec<PendingToolCall> {
        self.pending_tools.read().await.clone()
    }

    /// Approve a pending tool call
    pub async fn approve_tool(&self, tool_id: &str) -> Option<PendingToolCall> {
        let mut pending = self.pending_tools.write().await;
        if let Some(pos) = pending.iter().position(|t| t.id == tool_id) {
            let tool = pending.remove(pos);
            if pending.is_empty() {
                *self.state.write().await = SessionState::Active;
            }
            self.touch().await;
            Some(tool)
        } else {
            None
        }
    }

    /// Reject a pending tool call
    pub async fn reject_tool(&self, tool_id: &str) -> bool {
        let mut pending = self.pending_tools.write().await;
        if let Some(pos) = pending.iter().position(|t| t.id == tool_id) {
            pending.remove(pos);
            if pending.is_empty() {
                *self.state.write().await = SessionState::Active;
            }
            self.touch().await;
            true
        } else {
            false
        }
    }

    /// Set metadata
    pub async fn set_metadata(&self, key: String, value: serde_json::Value) {
        self.metadata.write().await.insert(key, value);
    }

    /// Get metadata
    pub async fn get_metadata(&self, key: &str) -> Option<serde_json::Value> {
        self.metadata.read().await.get(key).cloned()
    }

    /// Get creation timestamp
    pub fn created_at(&self) -> u64 {
        self.created_at
    }

    /// Get last activity timestamp
    pub async fn last_activity(&self) -> u64 {
        *self.last_activity.read().await
    }

    /// Update last activity timestamp
    async fn touch(&self) {
        *self.last_activity.write().await = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
    }

    /// Check if session is active
    pub async fn is_active(&self) -> bool {
        matches!(*self.state.read().await, SessionState::Active)
    }

    /// Check if session is processing
    pub async fn is_processing(&self) -> bool {
        matches!(*self.state.read().await, SessionState::Processing)
    }

    /// Check if session is waiting for input
    pub async fn is_waiting_for_input(&self) -> bool {
        matches!(*self.state.read().await, SessionState::WaitingForInput)
    }
}

impl Default for AgentSession {
    fn default() -> Self {
        Self::new()
    }
}
