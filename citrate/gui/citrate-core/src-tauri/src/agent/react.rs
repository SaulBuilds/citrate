//! ReAct (Reasoning + Acting) pattern implementation
//!
//! This module implements the ReAct pattern for tool orchestration,
//! allowing the LLM to reason about actions, execute tools, and
//! iterate until task completion.

use std::collections::HashMap;
use std::sync::Arc;

use super::context::{ContextMessage, ContextWindow, ConversationHistory, SystemContext};
use super::dispatcher::{DispatchError, ToolDefinition, ToolDispatcher, ToolOutput};
use super::intent::IntentParams;
use super::llm::LLMBackend;

/// Maximum number of ReAct iterations to prevent infinite loops
const MAX_ITERATIONS: usize = 5;

/// ReAct step types
#[derive(Debug, Clone)]
pub enum ReActStep {
    /// Thinking/reasoning step
    Thought(String),
    /// Action to take (tool call)
    Action { tool: String, params: IntentParams },
    /// Observation from tool execution
    Observation(String),
    /// Final answer to the user
    Answer(String),
}

/// Result of a ReAct execution
#[derive(Debug, Clone)]
pub struct ReActResult {
    /// The final response to the user
    pub response: String,
    /// Steps taken during execution
    pub steps: Vec<ReActStep>,
    /// Tools that were called
    pub tools_used: Vec<String>,
    /// Whether the execution completed successfully
    pub success: bool,
    /// Number of iterations
    pub iterations: usize,
}

/// ReAct executor that implements the reasoning-acting loop
pub struct ReActExecutor {
    /// Maximum iterations
    max_iterations: usize,
}

impl ReActExecutor {
    /// Create a new ReAct executor
    pub fn new() -> Self {
        Self {
            max_iterations: MAX_ITERATIONS,
        }
    }

    /// Create with custom max iterations
    pub fn with_max_iterations(max_iterations: usize) -> Self {
        Self { max_iterations }
    }

    /// Execute a ReAct loop for a user query
    pub async fn execute(
        &self,
        user_message: &str,
        llm: &dyn LLMBackend,
        dispatcher: &ToolDispatcher,
        system_context: Option<SystemContext>,
        conversation_history: &[ContextMessage],
    ) -> ReActResult {
        let mut steps: Vec<ReActStep> = Vec::new();
        let mut tools_used: Vec<String> = Vec::new();
        let mut iterations = 0;

        // Build the system prompt with tool definitions
        let system_prompt = self.build_react_system_prompt(dispatcher);

        // Start the ReAct loop
        loop {
            iterations += 1;

            if iterations > self.max_iterations {
                tracing::warn!("ReAct: Max iterations ({}) reached", self.max_iterations);
                // Generate a final response based on what we have
                let final_response = self
                    .generate_final_response(user_message, &steps, llm, &system_prompt)
                    .await;
                return ReActResult {
                    response: final_response,
                    steps,
                    tools_used,
                    success: false,
                    iterations,
                };
            }

            // Build context for this iteration
            let context = self.build_iteration_context(
                &system_prompt,
                system_context.clone(),
                conversation_history,
                user_message,
                &steps,
            );

            tracing::debug!(
                "ReAct iteration {}: calling LLM with {} messages",
                iterations,
                context.messages.len()
            );

            // Get LLM response
            let llm_response = match llm.complete(&context).await {
                Ok(response) => response,
                Err(e) => {
                    tracing::error!("ReAct: LLM error: {}", e);
                    return ReActResult {
                        response: format!("I encountered an error while processing: {}", e),
                        steps,
                        tools_used,
                        success: false,
                        iterations,
                    };
                }
            };

            tracing::debug!("ReAct iteration {}: LLM response: {}", iterations, &llm_response[..llm_response.len().min(200)]);

            // Parse the LLM response to determine next action
            match self.parse_react_response(&llm_response) {
                ParsedResponse::Thought(thought) => {
                    tracing::debug!("ReAct: Thought: {}", thought);
                    steps.push(ReActStep::Thought(thought));
                    // Continue to next iteration to get action
                }
                ParsedResponse::Action { tool, params } => {
                    tracing::info!("ReAct: Action: {} with params {:?}", tool, params);
                    steps.push(ReActStep::Action {
                        tool: tool.clone(),
                        params: params.clone(),
                    });
                    tools_used.push(tool.clone());

                    // Execute the tool
                    let observation = match dispatcher.dispatch_confirmed(&tool, &params).await {
                        Ok(output) => output.message,
                        Err(e) => format!("Tool error: {}", e),
                    };

                    tracing::debug!("ReAct: Observation: {}", observation);
                    steps.push(ReActStep::Observation(observation));
                    // Continue loop to process observation
                }
                ParsedResponse::Answer(answer) => {
                    tracing::info!("ReAct: Final answer after {} iterations", iterations);
                    steps.push(ReActStep::Answer(answer.clone()));
                    return ReActResult {
                        response: answer,
                        steps,
                        tools_used,
                        success: true,
                        iterations,
                    };
                }
                ParsedResponse::DirectResponse(response) => {
                    // LLM didn't use ReAct format, treat as direct answer
                    tracing::debug!("ReAct: Direct response (no tool use)");
                    return ReActResult {
                        response,
                        steps,
                        tools_used,
                        success: true,
                        iterations,
                    };
                }
            }
        }
    }

    /// Build the system prompt with tool definitions
    fn build_react_system_prompt(&self, dispatcher: &ToolDispatcher) -> String {
        let tools = dispatcher.list_tools();
        let tool_descriptions = self.format_tool_descriptions(tools);

        format!(
            r#"You are a helpful AI assistant for the Citrate blockchain platform. You can help users with wallet operations, blockchain queries, smart contracts, and AI model management.

You have access to the following tools:

{tool_descriptions}

When you need to use a tool to complete a task, respond in this format:
Thought: <your reasoning about what to do>
Action: <tool_name>
Action Input: <JSON object with parameters>

After receiving a tool result, you can:
1. Use another tool if needed
2. Provide a final answer

When you have enough information to answer the user's question, respond with:
Answer: <your final response to the user>

If the user's request doesn't require any tools, respond directly without the Thought/Action format.

Important guidelines:
- Always explain what you're doing in the Thought section
- Use the most appropriate tool for each task
- If a tool fails, explain what happened and try an alternative if available
- Be concise but informative in your final answers
- Format numbers and addresses nicely for readability"#,
            tool_descriptions = tool_descriptions
        )
    }

    /// Format tool descriptions for the system prompt
    fn format_tool_descriptions(&self, tools: &[ToolDefinition]) -> String {
        tools
            .iter()
            .map(|t| format!("- {}: {}", t.name, t.description))
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Build context for a ReAct iteration
    fn build_iteration_context(
        &self,
        system_prompt: &str,
        system_context: Option<SystemContext>,
        conversation_history: &[ContextMessage],
        user_message: &str,
        steps: &[ReActStep],
    ) -> ContextWindow {
        let mut messages: Vec<ContextMessage> = Vec::new();

        // Add conversation history
        for msg in conversation_history {
            messages.push(msg.clone());
        }

        // Add the current user message
        messages.push(ContextMessage {
            role: "user".to_string(),
            content: user_message.to_string(),
            name: None,
            tool_call_id: None,
        });

        // Add ReAct steps as assistant/tool messages
        for step in steps {
            match step {
                ReActStep::Thought(thought) => {
                    messages.push(ContextMessage {
                        role: "assistant".to_string(),
                        content: format!("Thought: {}", thought),
                        name: None,
                        tool_call_id: None,
                    });
                }
                ReActStep::Action { tool, params } => {
                    let params_json = serde_json::to_string(&params)
                        .unwrap_or_else(|_| "{}".to_string());
                    messages.push(ContextMessage {
                        role: "assistant".to_string(),
                        content: format!("Action: {}\nAction Input: {}", tool, params_json),
                        name: None,
                        tool_call_id: None,
                    });
                }
                ReActStep::Observation(obs) => {
                    messages.push(ContextMessage {
                        role: "user".to_string(),
                        content: format!("Observation: {}", obs),
                        name: Some("tool_result".to_string()),
                        tool_call_id: None,
                    });
                }
                ReActStep::Answer(_) => {
                    // Don't add final answer to context
                }
            }
        }

        ContextWindow {
            system_prompt: system_prompt.to_string(),
            system_context,
            messages,
            estimated_tokens: 0,
            was_truncated: false,
        }
    }

    /// Parse an LLM response to determine the next action
    fn parse_react_response(&self, response: &str) -> ParsedResponse {
        let response = response.trim();

        // Check for Answer: prefix (final answer)
        if let Some(answer) = self.extract_after_prefix(response, "Answer:") {
            return ParsedResponse::Answer(answer.trim().to_string());
        }

        // Check for Thought: and Action: pattern
        if let Some(thought) = self.extract_after_prefix(response, "Thought:") {
            // Look for Action after Thought
            if let Some(action_start) = response.find("Action:") {
                let after_thought = &response[action_start..];
                if let Some(action) = self.extract_after_prefix(after_thought, "Action:") {
                    let tool_name = action.lines().next().unwrap_or("").trim().to_string();

                    // Look for Action Input
                    let params = if let Some(input_start) = after_thought.find("Action Input:") {
                        let input_str = &after_thought[input_start + "Action Input:".len()..];
                        self.parse_action_input(input_str.trim())
                    } else {
                        IntentParams::default()
                    };

                    return ParsedResponse::Action {
                        tool: tool_name,
                        params,
                    };
                }
            }

            // Just a thought without action, return it
            return ParsedResponse::Thought(thought.lines().next().unwrap_or(&thought).trim().to_string());
        }

        // Check for just Action: (without Thought:)
        if let Some(action) = self.extract_after_prefix(response, "Action:") {
            let tool_name = action.lines().next().unwrap_or("").trim().to_string();

            let params = if let Some(input_start) = response.find("Action Input:") {
                let input_str = &response[input_start + "Action Input:".len()..];
                self.parse_action_input(input_str.trim())
            } else {
                IntentParams::default()
            };

            return ParsedResponse::Action {
                tool: tool_name,
                params,
            };
        }

        // No ReAct format detected, treat as direct response
        ParsedResponse::DirectResponse(response.to_string())
    }

    /// Extract text after a prefix
    fn extract_after_prefix<'a>(&self, text: &'a str, prefix: &str) -> Option<&'a str> {
        if let Some(pos) = text.find(prefix) {
            Some(&text[pos + prefix.len()..])
        } else {
            None
        }
    }

    /// Parse action input JSON into IntentParams
    fn parse_action_input(&self, input: &str) -> IntentParams {
        // Try to find JSON object
        let input = input.trim();

        // Handle case where input continues to next line
        let json_str = if input.starts_with('{') {
            // Find matching closing brace
            let mut depth = 0;
            let mut end_idx = 0;
            for (i, c) in input.chars().enumerate() {
                match c {
                    '{' => depth += 1,
                    '}' => {
                        depth -= 1;
                        if depth == 0 {
                            end_idx = i + 1;
                            break;
                        }
                    }
                    _ => {}
                }
            }
            if end_idx > 0 {
                &input[..end_idx]
            } else {
                input.lines().next().unwrap_or(input)
            }
        } else {
            input.lines().next().unwrap_or(input)
        };

        match serde_json::from_str::<HashMap<String, serde_json::Value>>(json_str) {
            Ok(map) => {
                let mut params = IntentParams::default();
                // Map known fields
                if let Some(v) = map.get("address") {
                    params.address = v.as_str().map(String::from);
                }
                if let Some(v) = map.get("amount") {
                    params.amount = v.as_str().map(String::from);
                }
                if let Some(v) = map.get("tx_hash") {
                    params.tx_hash = v.as_str().map(String::from);
                }
                if let Some(v) = map.get("contract_address") {
                    params.contract_address = v.as_str().map(String::from);
                }
                if let Some(v) = map.get("prompt") {
                    params.prompt = v.as_str().map(String::from);
                }
                if let Some(v) = map.get("search_query") {
                    params.search_query = v.as_str().map(String::from);
                }
                // Store all values as strings in extra for tools that need raw access
                for (k, v) in map {
                    params.extra.insert(k, v.to_string().trim_matches('"').to_string());
                }
                params
            }
            Err(e) => {
                tracing::debug!("Failed to parse action input as JSON: {} - input was: {}", e, json_str);
                IntentParams::default()
            }
        }
    }

    /// Generate a final response when max iterations reached
    async fn generate_final_response(
        &self,
        user_message: &str,
        steps: &[ReActStep],
        llm: &dyn LLMBackend,
        system_prompt: &str,
    ) -> String {
        // Build a summary of what was accomplished
        let steps_summary: Vec<String> = steps
            .iter()
            .filter_map(|s| match s {
                ReActStep::Observation(obs) => Some(format!("- {}", obs)),
                _ => None,
            })
            .collect();

        let prompt = format!(
            "The user asked: \"{}\"\n\nBased on the following information gathered:\n{}\n\nProvide a helpful response:",
            user_message,
            steps_summary.join("\n")
        );

        let context = ContextWindow {
            system_prompt: system_prompt.to_string(),
            system_context: None,
            messages: vec![ContextMessage {
                role: "user".to_string(),
                content: prompt,
                name: None,
                tool_call_id: None,
            }],
            estimated_tokens: 0,
            was_truncated: false,
        };

        llm.complete(&context)
            .await
            .unwrap_or_else(|e| format!("I gathered some information but couldn't complete the analysis: {}", e))
    }
}

impl Default for ReActExecutor {
    fn default() -> Self {
        Self::new()
    }
}

/// Parsed response from the LLM
#[derive(Debug)]
enum ParsedResponse {
    Thought(String),
    Action { tool: String, params: IntentParams },
    Answer(String),
    DirectResponse(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_answer() {
        let executor = ReActExecutor::new();
        let response = "Answer: The current balance is 100 SALT.";

        match executor.parse_react_response(response) {
            ParsedResponse::Answer(answer) => {
                assert!(answer.contains("100 SALT"));
            }
            _ => panic!("Expected Answer"),
        }
    }

    #[test]
    fn test_parse_thought_and_action() {
        let executor = ReActExecutor::new();
        let response = r#"Thought: I need to check the user's balance
Action: get_balance
Action Input: {"address": "0x123"}"#;

        match executor.parse_react_response(response) {
            ParsedResponse::Action { tool, params } => {
                assert_eq!(tool, "get_balance");
                assert!(params.address.is_some() || params.extra.contains_key("address"));
            }
            _ => panic!("Expected Action"),
        }
    }

    #[test]
    fn test_parse_direct_response() {
        let executor = ReActExecutor::new();
        let response = "Hello! How can I help you today?";

        match executor.parse_react_response(response) {
            ParsedResponse::DirectResponse(text) => {
                assert!(text.contains("Hello"));
            }
            _ => panic!("Expected DirectResponse"),
        }
    }
}
