//! Tool result formatting for AI agent responses
//!
//! Provides consistent formatting for tool outputs to be displayed
//! in the chat interface and processed by the AI model.

use serde::{Deserialize, Serialize};
use super::dispatcher::ToolOutput;

/// Formatted result ready for display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormattedResult {
    /// Original tool output
    pub output: ToolOutput,
    /// Human-readable formatted text
    pub formatted_text: String,
    /// Markdown formatted version
    pub markdown: String,
    /// Whether to show as expandable details
    pub expandable: bool,
    /// Category for grouping
    pub category: ResultCategory,
}

/// Result category for visual grouping
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ResultCategory {
    /// Blockchain query results (balances, blocks, etc.)
    Query,
    /// Transaction results (send, deploy, etc.)
    Transaction,
    /// Contract interaction results
    Contract,
    /// Model/AI results
    Model,
    /// Node/network status
    Status,
    /// Error results
    Error,
    /// General/other results
    General,
}

impl FormattedResult {
    /// Format a tool output for display
    pub fn from_output(output: ToolOutput) -> Self {
        let category = categorize_tool(&output.tool);
        let formatted_text = format_text(&output, category);
        let markdown = format_markdown(&output, category);
        let expandable = should_expand(&output);

        Self {
            output,
            formatted_text,
            markdown,
            expandable,
            category,
        }
    }

    /// Get a summary line for the result
    pub fn summary(&self) -> String {
        let icon = match self.category {
            ResultCategory::Query => "🔍",
            ResultCategory::Transaction => "📤",
            ResultCategory::Contract => "📜",
            ResultCategory::Model => "🤖",
            ResultCategory::Status => "📊",
            ResultCategory::Error => "❌",
            ResultCategory::General => "ℹ️",
        };

        let status = if self.output.success { "✓" } else { "✗" };

        format!("{} {} {}", icon, status, self.output.message)
    }

    /// Get compact one-liner for status bar
    pub fn compact(&self) -> String {
        let max_len = 60;
        let msg = &self.output.message;
        if msg.len() > max_len {
            format!("{}...", &msg[..max_len - 3])
        } else {
            msg.clone()
        }
    }
}

/// Determine the category of a tool result
fn categorize_tool(tool_name: &str) -> ResultCategory {
    match tool_name {
        "node_status" | "dag_status" => ResultCategory::Status,
        "query_balance" | "account_info" | "block_info" | "transaction_info" | "transaction_history" => {
            ResultCategory::Query
        }
        "send_transaction" => ResultCategory::Transaction,
        "deploy_contract" | "call_contract" | "write_contract" => ResultCategory::Contract,
        "list_models" | "run_inference" | "deploy_model" | "get_model_info" => ResultCategory::Model,
        _ => ResultCategory::General,
    }
}

/// Format output as plain text
fn format_text(output: &ToolOutput, category: ResultCategory) -> String {
    let mut text = String::new();

    // Header
    let header = match category {
        ResultCategory::Query => "Query Result",
        ResultCategory::Transaction => "Transaction",
        ResultCategory::Contract => "Contract",
        ResultCategory::Model => "AI Model",
        ResultCategory::Status => "Status",
        ResultCategory::Error => "Error",
        ResultCategory::General => "Result",
    };

    text.push_str(&format!("=== {} ===\n", header));
    text.push_str(&format!("Tool: {}\n", output.tool));
    text.push_str(&format!("Status: {}\n", if output.success { "Success" } else { "Failed" }));
    text.push_str(&format!("\n{}\n", output.message));

    // Add data details if present
    if let Some(data) = &output.data {
        text.push_str("\nDetails:\n");
        text.push_str(&format_data_text(data, 0));
    }

    text
}

/// Format output as Markdown
fn format_markdown(output: &ToolOutput, category: ResultCategory) -> String {
    let mut md = String::new();

    // Status badge
    let status_badge = if output.success {
        "**✓ Success**"
    } else {
        "**✗ Failed**"
    };

    md.push_str(&format!("### {} | {}\n\n", output.tool, status_badge));
    md.push_str(&format!("{}\n\n", output.message));

    // Add formatted data
    if let Some(data) = &output.data {
        match category {
            ResultCategory::Query => md.push_str(&format_query_md(data)),
            ResultCategory::Transaction => md.push_str(&format_transaction_md(data)),
            ResultCategory::Contract => md.push_str(&format_contract_md(data)),
            ResultCategory::Model => md.push_str(&format_model_md(data)),
            ResultCategory::Status => md.push_str(&format_status_md(data)),
            _ => md.push_str(&format_generic_md(data)),
        }
    }

    md
}

/// Format data as indented text
fn format_data_text(data: &serde_json::Value, indent: usize) -> String {
    let prefix = "  ".repeat(indent);
    let mut text = String::new();

    match data {
        serde_json::Value::Object(map) => {
            for (k, v) in map {
                match v {
                    serde_json::Value::Object(_) | serde_json::Value::Array(_) => {
                        text.push_str(&format!("{}{}:\n", prefix, k));
                        text.push_str(&format_data_text(v, indent + 1));
                    }
                    _ => {
                        text.push_str(&format!("{}{}: {}\n", prefix, k, format_value(v)));
                    }
                }
            }
        }
        serde_json::Value::Array(arr) => {
            for (i, v) in arr.iter().enumerate() {
                text.push_str(&format!("{}[{}]:\n", prefix, i));
                text.push_str(&format_data_text(v, indent + 1));
            }
        }
        _ => {
            text.push_str(&format!("{}{}\n", prefix, format_value(data)));
        }
    }

    text
}

/// Format a JSON value for display
fn format_value(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::Bool(b) => b.to_string(),
        serde_json::Value::Null => "null".to_string(),
        _ => value.to_string(),
    }
}

/// Format query results as Markdown table
fn format_query_md(data: &serde_json::Value) -> String {
    let mut md = String::new();

    if let Some(obj) = data.as_object() {
        md.push_str("| Field | Value |\n|-------|-------|\n");
        for (k, v) in obj {
            if !v.is_object() && !v.is_array() {
                md.push_str(&format!("| {} | {} |\n", k, format_value(v)));
            }
        }
    }

    md
}

/// Format transaction results as Markdown
fn format_transaction_md(data: &serde_json::Value) -> String {
    let mut md = String::new();

    if let Some(obj) = data.as_object() {
        if let Some(hash) = obj.get("tx_hash") {
            md.push_str(&format!("**Transaction Hash:** `{}`\n\n", format_value(hash)));
        }

        if let Some(status) = obj.get("status") {
            let emoji = if format_value(status) == "pending" { "⏳" } else { "✅" };
            md.push_str(&format!("**Status:** {} {}\n\n", emoji, format_value(status)));
        }

        // From/To
        if let (Some(from), Some(to)) = (obj.get("from"), obj.get("to")) {
            md.push_str(&format!("**From:** `{}`\n", format_value(from)));
            md.push_str(&format!("**To:** `{}`\n\n", format_value(to)));
        }

        // Value
        if let Some(value_ctr) = obj.get("value_ctr") {
            md.push_str(&format!("**Value:** {} CTR\n", format_value(value_ctr)));
        }
    }

    md
}

/// Format contract results as Markdown
fn format_contract_md(data: &serde_json::Value) -> String {
    let mut md = String::new();

    if let Some(obj) = data.as_object() {
        if let Some(addr) = obj.get("contract_address") {
            md.push_str(&format!("**Contract Address:** `{}`\n\n", format_value(addr)));
        }

        if let Some(func) = obj.get("function") {
            md.push_str(&format!("**Function:** `{}`\n", format_value(func)));
        }

        if let Some(calldata) = obj.get("calldata") {
            md.push_str(&format!("\n```\nCalldata: {}\n```\n", format_value(calldata)));
        }
    }

    md
}

/// Format model results as Markdown
fn format_model_md(data: &serde_json::Value) -> String {
    let mut md = String::new();

    if let Some(obj) = data.as_object() {
        // Inference output
        if let Some(output) = obj.get("output") {
            md.push_str("**Output:**\n```\n");
            md.push_str(&format_value(output));
            md.push_str("\n```\n\n");
        }

        // Model list
        if let Some(models) = obj.get("models").and_then(|m| m.as_array()) {
            md.push_str("| Model | Type | Size |\n|-------|------|------|\n");
            for model in models {
                if let Some(m) = model.as_object() {
                    let name = m.get("name").map(format_value).unwrap_or_default();
                    let typ = m.get("type").map(format_value).unwrap_or_default();
                    let size = m.get("size_mb").map(format_value).unwrap_or_default();
                    md.push_str(&format!("| {} | {} | {} MB |\n", name, typ, size));
                }
            }
        }

        // Performance metrics
        if let Some(latency) = obj.get("latency_ms") {
            md.push_str(&format!("\n**Latency:** {}ms\n", format_value(latency)));
        }
        if let Some(confidence) = obj.get("confidence") {
            let conf: f64 = confidence.as_f64().unwrap_or(0.0);
            md.push_str(&format!("**Confidence:** {:.1}%\n", conf * 100.0));
        }
    }

    md
}

/// Format status results as Markdown
fn format_status_md(data: &serde_json::Value) -> String {
    let mut md = String::new();

    if let Some(obj) = data.as_object() {
        // Running status indicator
        if let Some(running) = obj.get("running").and_then(|r| r.as_bool()) {
            let indicator = if running { "🟢 Online" } else { "🔴 Offline" };
            md.push_str(&format!("**Node Status:** {}\n\n", indicator));
        }

        // Key metrics
        let metrics = ["block_height", "peers", "dag_tips", "blue_score", "syncing"];
        let mut has_metrics = false;

        for metric in metrics {
            if let Some(val) = obj.get(metric) {
                if !has_metrics {
                    md.push_str("| Metric | Value |\n|--------|-------|\n");
                    has_metrics = true;
                }
                md.push_str(&format!("| {} | {} |\n", metric, format_value(val)));
            }
        }
    }

    md
}

/// Format generic data as Markdown
fn format_generic_md(data: &serde_json::Value) -> String {
    let mut md = String::new();

    md.push_str("```json\n");
    md.push_str(&serde_json::to_string_pretty(data).unwrap_or_else(|_| data.to_string()));
    md.push_str("\n```\n");

    md
}

/// Determine if result should be expandable
fn should_expand(output: &ToolOutput) -> bool {
    if let Some(data) = &output.data {
        // Expand if data is complex
        let json_str = data.to_string();
        json_str.len() > 200 || data.is_array()
    } else {
        false
    }
}

/// Format a batch of results
pub fn format_batch(outputs: Vec<ToolOutput>) -> Vec<FormattedResult> {
    outputs.into_iter().map(FormattedResult::from_output).collect()
}

/// Generate a summary for multiple results
pub fn batch_summary(results: &[FormattedResult]) -> String {
    let success_count = results.iter().filter(|r| r.output.success).count();
    let fail_count = results.len() - success_count;

    let mut summary = format!(
        "Executed {} tool{}: {} succeeded",
        results.len(),
        if results.len() == 1 { "" } else { "s" },
        success_count
    );

    if fail_count > 0 {
        summary.push_str(&format!(", {} failed", fail_count));
    }

    summary
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_categorize_tools() {
        assert_eq!(categorize_tool("node_status"), ResultCategory::Status);
        assert_eq!(categorize_tool("query_balance"), ResultCategory::Query);
        assert_eq!(categorize_tool("send_transaction"), ResultCategory::Transaction);
        assert_eq!(categorize_tool("deploy_contract"), ResultCategory::Contract);
        assert_eq!(categorize_tool("run_inference"), ResultCategory::Model);
        assert_eq!(categorize_tool("unknown"), ResultCategory::General);
    }

    #[test]
    fn test_format_result() {
        let output = ToolOutput {
            tool: "query_balance".to_string(),
            success: true,
            message: "Balance: 100 CTR".to_string(),
            data: Some(serde_json::json!({
                "address": "0x1234",
                "balance_ctr": 100.0
            })),
        };

        let formatted = FormattedResult::from_output(output);
        assert_eq!(formatted.category, ResultCategory::Query);
        assert!(formatted.formatted_text.contains("Query Result"));
        assert!(formatted.markdown.contains("query_balance"));
    }

    #[test]
    fn test_batch_summary() {
        let results = vec![
            FormattedResult::from_output(ToolOutput {
                tool: "test".to_string(),
                success: true,
                message: "OK".to_string(),
                data: None,
            }),
            FormattedResult::from_output(ToolOutput {
                tool: "test2".to_string(),
                success: false,
                message: "Failed".to_string(),
                data: None,
            }),
        ];

        let summary = batch_summary(&results);
        assert!(summary.contains("2 tools"));
        assert!(summary.contains("1 succeeded"));
        assert!(summary.contains("1 failed"));
    }
}
