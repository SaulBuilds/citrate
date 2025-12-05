//! Terminal tools - safe command execution
//!
//! These tools provide controlled terminal/command execution capabilities.

use std::future::Future;
use std::path::PathBuf;
use std::pin::Pin;
use std::process::Stdio;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::RwLock;
use tokio::time::timeout;

use super::super::dispatcher::{DispatchError, ToolHandler, ToolOutput};
use super::super::intent::IntentParams;

/// Allowed commands that can be executed
const ALLOWED_COMMANDS: &[&str] = &[
    // Version control
    "git",
    // Package managers
    "npm",
    "npx",
    "yarn",
    "pnpm",
    "cargo",
    "pip",
    "pip3",
    // Build tools
    "make",
    "cmake",
    "forge",
    "foundry",
    // Development
    "node",
    "python",
    "python3",
    "rustc",
    "solc",
    // File operations (safe subset)
    "ls",
    "pwd",
    "cat",
    "head",
    "tail",
    "wc",
    "find",
    "grep",
    "which",
    "echo",
    "mkdir",
    "cp",
    "mv",
    // System info
    "uname",
    "whoami",
    "date",
    "env",
    // Network (read-only)
    "curl",
    "wget",
    "ping",
];

/// Blocked commands/patterns
const BLOCKED_PATTERNS: &[&str] = &[
    "rm -rf",
    "rm -r /",
    "sudo",
    "su ",
    "chmod 777",
    "chown",
    "> /dev",
    "| bash",
    "| sh",
    "; rm",
    "&& rm",
    "mkfs",
    "dd if=",
    ":(){ :|:& };:",
    "eval",
    "exec",
];

/// Execute terminal command tool
pub struct ExecuteCommandTool {
    working_dir: Arc<RwLock<PathBuf>>,
    timeout_secs: u64,
}

impl ExecuteCommandTool {
    pub fn new() -> Self {
        Self {
            working_dir: Arc::new(RwLock::new(std::env::current_dir().unwrap_or_default())),
            timeout_secs: 60,
        }
    }

    /// Check if command is allowed
    fn is_command_allowed(cmd: &str) -> bool {
        let cmd_lower = cmd.to_lowercase();

        // Check for blocked patterns
        for pattern in BLOCKED_PATTERNS {
            if cmd_lower.contains(pattern) {
                return false;
            }
        }

        // Extract the base command
        let base_cmd = cmd.split_whitespace().next().unwrap_or("");

        // Check if base command is in allowlist
        ALLOWED_COMMANDS.iter().any(|&allowed| base_cmd == allowed)
    }
}

impl Default for ExecuteCommandTool {
    fn default() -> Self {
        Self::new()
    }
}

impl ToolHandler for ExecuteCommandTool {
    fn name(&self) -> &str {
        "execute_command"
    }

    fn description(&self) -> &str {
        "Execute a terminal command safely. Only allows specific commands (git, npm, cargo, etc.)"
    }

    fn execute(
        &self,
        params: &IntentParams,
    ) -> Pin<Box<dyn Future<Output = Result<ToolOutput, DispatchError>> + Send + '_>> {
        let working_dir = self.working_dir.clone();
        let timeout_secs = self.timeout_secs;
        let command = params.prompt.clone(); // Use prompt for command
        Box::pin(async move {
            let cmd = command.ok_or_else(|| {
                DispatchError::InvalidParams("Command required".to_string())
            })?;

            // Security check
            if !Self::is_command_allowed(&cmd) {
                return Ok(ToolOutput {
                    tool: "execute_command".to_string(),
                    success: false,
                    message: format!(
                        "Command not allowed: '{}'. Only safe commands are permitted (git, npm, cargo, etc.)",
                        cmd.split_whitespace().next().unwrap_or(&cmd)
                    ),
                    data: Some(serde_json::json!({
                        "blocked": true,
                        "allowed_commands": ALLOWED_COMMANDS
                    })),
                });
            }

            let cwd = working_dir.read().await.clone();

            // Parse command into parts
            let parts: Vec<&str> = cmd.split_whitespace().collect();
            if parts.is_empty() {
                return Err(DispatchError::InvalidParams("Empty command".to_string()));
            }

            let program = parts[0];
            let args = &parts[1..];

            // Execute command with timeout
            let mut child = match Command::new(program)
                .args(args)
                .current_dir(&cwd)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
            {
                Ok(c) => c,
                Err(e) => {
                    return Ok(ToolOutput {
                        tool: "execute_command".to_string(),
                        success: false,
                        message: format!("Failed to execute command: {}", e),
                        data: None,
                    });
                }
            };

            // Capture output with timeout
            let result = timeout(Duration::from_secs(timeout_secs), async {
                let stdout = child.stdout.take();
                let stderr = child.stderr.take();

                let mut stdout_lines = Vec::new();
                let mut stderr_lines = Vec::new();

                if let Some(stdout) = stdout {
                    let reader = BufReader::new(stdout);
                    let mut lines = reader.lines();
                    while let Ok(Some(line)) = lines.next_line().await {
                        stdout_lines.push(line);
                        // Limit output
                        if stdout_lines.len() > 500 {
                            stdout_lines.push("... (output truncated)".to_string());
                            break;
                        }
                    }
                }

                if let Some(stderr) = stderr {
                    let reader = BufReader::new(stderr);
                    let mut lines = reader.lines();
                    while let Ok(Some(line)) = lines.next_line().await {
                        stderr_lines.push(line);
                        if stderr_lines.len() > 100 {
                            stderr_lines.push("... (error output truncated)".to_string());
                            break;
                        }
                    }
                }

                let status = child.wait().await;
                (status, stdout_lines, stderr_lines)
            })
            .await;

            match result {
                Ok((Ok(status), stdout, stderr)) => {
                    let exit_code = status.code().unwrap_or(-1);
                    let success = status.success();

                    let stdout_text = stdout.join("\n");
                    let stderr_text = stderr.join("\n");

                    let output_preview = if stdout_text.len() > 200 {
                        format!("{}...", &stdout_text[..200])
                    } else {
                        stdout_text.clone()
                    };

                    Ok(ToolOutput {
                        tool: "execute_command".to_string(),
                        success,
                        message: if success {
                            format!(
                                "Command completed (exit 0): {}",
                                if output_preview.is_empty() {
                                    "(no output)".to_string()
                                } else {
                                    output_preview
                                }
                            )
                        } else {
                            format!(
                                "Command failed (exit {}): {}",
                                exit_code,
                                if stderr_text.is_empty() {
                                    stdout_text.clone()
                                } else {
                                    stderr_text.clone()
                                }
                            )
                        },
                        data: Some(serde_json::json!({
                            "command": cmd,
                            "working_dir": cwd.to_string_lossy(),
                            "exit_code": exit_code,
                            "stdout": stdout_text,
                            "stderr": stderr_text,
                            "success": success
                        })),
                    })
                }
                Ok((Err(e), _, _)) => Ok(ToolOutput {
                    tool: "execute_command".to_string(),
                    success: false,
                    message: format!("Command process error: {}", e),
                    data: None,
                }),
                Err(_) => {
                    // Timeout - try to kill the process
                    let _ = child.kill().await;
                    Ok(ToolOutput {
                        tool: "execute_command".to_string(),
                        success: false,
                        message: format!(
                            "Command timed out after {} seconds",
                            timeout_secs
                        ),
                        data: Some(serde_json::json!({
                            "command": cmd,
                            "timeout_secs": timeout_secs,
                            "killed": true
                        })),
                    })
                }
            }
        })
    }
}

/// Change working directory tool
pub struct ChangeDirectoryTool {
    working_dir: Arc<RwLock<PathBuf>>,
}

impl ChangeDirectoryTool {
    pub fn new(working_dir: Arc<RwLock<PathBuf>>) -> Self {
        Self { working_dir }
    }
}

impl ToolHandler for ChangeDirectoryTool {
    fn name(&self) -> &str {
        "change_directory"
    }

    fn description(&self) -> &str {
        "Change the working directory for command execution"
    }

    fn execute(
        &self,
        params: &IntentParams,
    ) -> Pin<Box<dyn Future<Output = Result<ToolOutput, DispatchError>> + Send + '_>> {
        let working_dir = self.working_dir.clone();
        let path = params.prompt.clone();
        Box::pin(async move {
            let new_path = path.ok_or_else(|| {
                DispatchError::InvalidParams("Directory path required".to_string())
            })?;

            let current = working_dir.read().await.clone();

            // Resolve relative paths
            let resolved = if new_path.starts_with('/') || new_path.starts_with('~') {
                let expanded = if new_path.starts_with('~') {
                    dirs::home_dir()
                        .map(|h| h.join(&new_path[2..]))
                        .unwrap_or_else(|| PathBuf::from(&new_path))
                } else {
                    PathBuf::from(&new_path)
                };
                expanded
            } else {
                current.join(&new_path)
            };

            // Canonicalize to resolve .. and .
            let canonical = match resolved.canonicalize() {
                Ok(p) => p,
                Err(e) => {
                    return Ok(ToolOutput {
                        tool: "change_directory".to_string(),
                        success: false,
                        message: format!("Invalid path '{}': {}", new_path, e),
                        data: None,
                    });
                }
            };

            // Check if it's a directory
            if !canonical.is_dir() {
                return Ok(ToolOutput {
                    tool: "change_directory".to_string(),
                    success: false,
                    message: format!("'{}' is not a directory", new_path),
                    data: None,
                });
            }

            *working_dir.write().await = canonical.clone();

            Ok(ToolOutput {
                tool: "change_directory".to_string(),
                success: true,
                message: format!("Changed directory to: {}", canonical.display()),
                data: Some(serde_json::json!({
                    "previous": current.to_string_lossy(),
                    "current": canonical.to_string_lossy()
                })),
            })
        })
    }
}

/// Get current working directory tool
pub struct GetWorkingDirectoryTool {
    working_dir: Arc<RwLock<PathBuf>>,
}

impl GetWorkingDirectoryTool {
    pub fn new(working_dir: Arc<RwLock<PathBuf>>) -> Self {
        Self { working_dir }
    }
}

impl ToolHandler for GetWorkingDirectoryTool {
    fn name(&self) -> &str {
        "get_working_directory"
    }

    fn description(&self) -> &str {
        "Get the current working directory"
    }

    fn execute(
        &self,
        _params: &IntentParams,
    ) -> Pin<Box<dyn Future<Output = Result<ToolOutput, DispatchError>> + Send + '_>> {
        let working_dir = self.working_dir.clone();
        Box::pin(async move {
            let cwd = working_dir.read().await.clone();

            Ok(ToolOutput {
                tool: "get_working_directory".to_string(),
                success: true,
                message: format!("Current directory: {}", cwd.display()),
                data: Some(serde_json::json!({
                    "path": cwd.to_string_lossy(),
                    "exists": cwd.exists(),
                    "is_dir": cwd.is_dir()
                })),
            })
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_allowed() {
        assert!(ExecuteCommandTool::is_command_allowed("git status"));
        assert!(ExecuteCommandTool::is_command_allowed("npm install"));
        assert!(ExecuteCommandTool::is_command_allowed("cargo build"));
        assert!(ExecuteCommandTool::is_command_allowed("ls -la"));
        assert!(ExecuteCommandTool::is_command_allowed("pwd"));
    }

    #[test]
    fn test_command_blocked() {
        assert!(!ExecuteCommandTool::is_command_allowed("rm -rf /"));
        assert!(!ExecuteCommandTool::is_command_allowed("sudo apt install"));
        assert!(!ExecuteCommandTool::is_command_allowed("chmod 777 /etc"));
        assert!(!ExecuteCommandTool::is_command_allowed("dd if=/dev/zero"));
    }

    #[test]
    fn test_unknown_command_blocked() {
        assert!(!ExecuteCommandTool::is_command_allowed("virus.exe"));
        assert!(!ExecuteCommandTool::is_command_allowed("random_script"));
    }
}
