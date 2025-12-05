//! Tool dispatcher for executing blockchain and AI operations
//!
//! Dispatches tool calls to appropriate handlers based on the
//! Model Context Protocol (MCP) standard.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

use super::intent::IntentParams;

/// Configuration for tool dispatch
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolConfig {
    /// Whether to require user confirmation for sensitive operations
    pub require_confirmation: bool,
    /// Operations that require confirmation
    pub confirmation_required_tools: Vec<String>,
    /// Timeout for tool execution in milliseconds
    pub execution_timeout_ms: u64,
    /// Whether to enable parallel tool execution
    pub parallel_execution: bool,
}

impl Default for ToolConfig {
    fn default() -> Self {
        Self {
            require_confirmation: true,
            confirmation_required_tools: vec![
                "send_transaction".to_string(),
                "deploy_contract".to_string(),
                "write_contract".to_string(),
                "deploy_model".to_string(),
            ],
            execution_timeout_ms: 30000,
            parallel_execution: false,
        }
    }
}

/// Error during tool dispatch
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DispatchError {
    /// Tool not found
    ToolNotFound(String),
    /// Invalid parameters
    InvalidParams(String),
    /// Execution failed
    ExecutionFailed(String),
    /// Requires confirmation
    RequiresConfirmation(String),
    /// Timeout
    Timeout,
    /// Internal error
    Internal(String),
}

impl std::fmt::Display for DispatchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ToolNotFound(name) => write!(f, "Tool not found: {}", name),
            Self::InvalidParams(e) => write!(f, "Invalid parameters: {}", e),
            Self::ExecutionFailed(e) => write!(f, "Execution failed: {}", e),
            Self::RequiresConfirmation(tool) => {
                write!(f, "Tool '{}' requires user confirmation", tool)
            }
            Self::Timeout => write!(f, "Tool execution timed out"),
            Self::Internal(e) => write!(f, "Internal error: {}", e),
        }
    }
}

impl std::error::Error for DispatchError {}

/// Result of tool execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolOutput {
    /// Tool name
    pub tool: String,
    /// Success status
    pub success: bool,
    /// Output message
    pub message: String,
    /// Structured data
    pub data: Option<serde_json::Value>,
}

/// A registered tool handler
pub trait ToolHandler: Send + Sync {
    /// Tool name
    fn name(&self) -> &str;

    /// Tool description
    fn description(&self) -> &str;

    /// Execute the tool
    fn execute(
        &self,
        params: &IntentParams,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<ToolOutput, DispatchError>> + Send + '_>,
    >;

    /// Whether this tool requires confirmation
    fn requires_confirmation(&self) -> bool {
        false
    }
}

/// MCP-compatible tool definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    /// Tool name
    pub name: String,
    /// Description
    pub description: String,
    /// Input schema
    pub input_schema: serde_json::Value,
    /// Whether it requires confirmation
    pub requires_confirmation: bool,
}

/// Main tool dispatcher
pub struct ToolDispatcher {
    config: ToolConfig,
    handlers: HashMap<String, Arc<dyn ToolHandler>>,
    definitions: Vec<ToolDefinition>,
}

impl ToolDispatcher {
    /// Create a new dispatcher
    pub fn new() -> Self {
        Self {
            config: ToolConfig::default(),
            handlers: HashMap::new(),
            definitions: Vec::new(),
        }
    }

    /// Create with configuration
    pub fn with_config(config: ToolConfig) -> Self {
        Self {
            config,
            handlers: HashMap::new(),
            definitions: Vec::new(),
        }
    }

    /// Register a tool handler
    pub fn register<T: ToolHandler + 'static>(&mut self, handler: T) {
        let name = handler.name().to_string();
        let description = handler.description().to_string();
        let requires_confirmation = handler.requires_confirmation();

        self.definitions.push(ToolDefinition {
            name: name.clone(),
            description,
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {}
            }),
            requires_confirmation,
        });

        self.handlers.insert(name, Arc::new(handler));
    }

    /// Dispatch a tool call
    pub async fn dispatch(
        &self,
        tool_name: &str,
        params: &IntentParams,
    ) -> Result<String, DispatchError> {
        // Check if tool exists
        let handler = self
            .handlers
            .get(tool_name)
            .ok_or_else(|| DispatchError::ToolNotFound(tool_name.to_string()))?;

        // Check if confirmation required
        if self.requires_confirmation(tool_name) {
            return Err(DispatchError::RequiresConfirmation(tool_name.to_string()));
        }

        // Execute with timeout
        let result = tokio::time::timeout(
            std::time::Duration::from_millis(self.config.execution_timeout_ms),
            handler.execute(params),
        )
        .await
        .map_err(|_| DispatchError::Timeout)?;

        match result {
            Ok(output) => Ok(output.message),
            Err(e) => Err(e),
        }
    }

    /// Execute a tool with confirmation bypass (for approved actions)
    pub async fn dispatch_confirmed(
        &self,
        tool_name: &str,
        params: &IntentParams,
    ) -> Result<ToolOutput, DispatchError> {
        let handler = self
            .handlers
            .get(tool_name)
            .ok_or_else(|| DispatchError::ToolNotFound(tool_name.to_string()))?;

        tokio::time::timeout(
            std::time::Duration::from_millis(self.config.execution_timeout_ms),
            handler.execute(params),
        )
        .await
        .map_err(|_| DispatchError::Timeout)?
    }

    /// Check if a tool requires confirmation
    fn requires_confirmation(&self, tool_name: &str) -> bool {
        if !self.config.require_confirmation {
            return false;
        }

        self.config
            .confirmation_required_tools
            .contains(&tool_name.to_string())
    }

    /// List available tools
    pub fn list_tools(&self) -> &[ToolDefinition] {
        &self.definitions
    }

    /// Get tool definition by name
    pub fn get_tool(&self, name: &str) -> Option<&ToolDefinition> {
        self.definitions.iter().find(|t| t.name == name)
    }

    /// Get tools as MCP-compatible JSON
    pub fn tools_as_mcp_json(&self) -> serde_json::Value {
        serde_json::json!({
            "tools": self.definitions.iter().map(|t| {
                serde_json::json!({
                    "name": t.name,
                    "description": t.description,
                    "inputSchema": t.input_schema
                })
            }).collect::<Vec<_>>()
        })
    }
}

impl Default for ToolDispatcher {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// Test Tool Handlers (for unit tests only)
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// Simple mock tool for testing
    struct MockTool {
        name: &'static str,
    }

    impl MockTool {
        fn new(name: &'static str) -> Self {
            Self { name }
        }
    }

    impl ToolHandler for MockTool {
        fn name(&self) -> &str {
            self.name
        }

        fn description(&self) -> &str {
            "Mock tool for testing"
        }

        fn execute(
            &self,
            _params: &IntentParams,
        ) -> std::pin::Pin<
            Box<dyn std::future::Future<Output = Result<ToolOutput, DispatchError>> + Send + '_>,
        > {
            let tool_name = self.name.to_string();
            Box::pin(async move {
                Ok(ToolOutput {
                    tool: tool_name.clone(),
                    success: true,
                    message: format!("Mock result from {}", tool_name),
                    data: None,
                })
            })
        }
    }

    #[tokio::test]
    async fn test_register_and_dispatch() {
        let mut dispatcher = ToolDispatcher::new();
        dispatcher.register(MockTool::new("test_tool"));

        assert_eq!(dispatcher.list_tools().len(), 1);

        let params = IntentParams::default();

        let result = dispatcher.dispatch("test_tool", &params).await;
        assert!(result.is_ok());
        assert!(result.unwrap().contains("Mock result"));
    }

    #[tokio::test]
    async fn test_tool_not_found() {
        let dispatcher = ToolDispatcher::new();
        let params = IntentParams::default();

        let result = dispatcher.dispatch("nonexistent_tool", &params).await;
        assert!(matches!(result, Err(DispatchError::ToolNotFound(_))));
    }

    #[tokio::test]
    async fn test_mcp_json_format() {
        let mut dispatcher = ToolDispatcher::new();
        dispatcher.register(MockTool::new("tool_a"));
        dispatcher.register(MockTool::new("tool_b"));

        let json = dispatcher.tools_as_mcp_json();
        let tools = json.get("tools").unwrap().as_array().unwrap();

        assert_eq!(tools.len(), 2);
    }
}
