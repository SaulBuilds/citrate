//! Terminal Module
//!
//! Sprint 5: Multi-Window & Terminal Integration
//!
//! Provides PTY-based terminal functionality using portable-pty.

pub mod manager;
pub mod session;

pub use manager::TerminalManager;
pub use session::{TerminalSession, TerminalConfig};

use serde::{Deserialize, Serialize};

/// Terminal output event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalOutput {
    pub session_id: String,
    pub data: String, // Base64 encoded
    pub timestamp: u64,
}

/// Terminal resize event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalResize {
    pub session_id: String,
    pub cols: u16,
    pub rows: u16,
}

/// Terminal session info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalInfo {
    pub session_id: String,
    pub shell: String,
    pub cwd: String,
    pub cols: u16,
    pub rows: u16,
    pub created_at: u64,
    pub is_active: bool,
}

impl TerminalOutput {
    pub fn new(session_id: &str, data: &[u8]) -> Self {
        Self {
            session_id: session_id.to_string(),
            data: base64::Engine::encode(&base64::engine::general_purpose::STANDARD, data),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
        }
    }
}
