//! Terminal Manager
//!
//! Manages multiple terminal sessions.

use super::{TerminalConfig, TerminalInfo, TerminalOutput, TerminalSession};
use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use tokio::sync::RwLock;
use tracing::{debug, error, info};

/// Terminal Manager
///
/// Manages all terminal sessions and their lifecycle.
pub struct TerminalManager {
    sessions: Arc<RwLock<HashMap<String, Arc<TerminalSession>>>>,
    app_handle: Option<AppHandle>,
}

impl TerminalManager {
    /// Create a new terminal manager
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            app_handle: None,
        }
    }

    /// Set the app handle for emitting events
    pub fn set_app_handle(&mut self, app: AppHandle) {
        self.app_handle = Some(app);
    }

    /// Create a new terminal session
    pub async fn create_session(&self, config: TerminalConfig) -> Result<TerminalInfo> {
        let session_id = format!(
            "term_{}_{:x}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis(),
            rand::random::<u32>()
        );

        let cols = config.cols;
        let rows = config.rows;
        let session = TerminalSession::new(session_id.clone(), config)?;
        let session_arc = Arc::new(session);

        // Start the session
        session_arc.start().await?;

        // Start output forwarding
        self.start_output_forwarding(session_arc.clone());

        // Store session
        {
            let mut sessions = self.sessions.write().await;
            sessions.insert(session_id.clone(), session_arc.clone());
        }

        info!("Created terminal session: {}", session_id);

        Ok(TerminalInfo {
            session_id,
            shell: session_arc.shell.clone(),
            cwd: session_arc.cwd.clone(),
            cols,
            rows,
            created_at: session_arc.created_at,
            is_active: true,
        })
    }

    /// Start forwarding output from terminal to frontend
    fn start_output_forwarding(&self, session: Arc<TerminalSession>) {
        let app_handle = self.app_handle.clone();
        let session_id = session.id.clone();

        tokio::spawn(async move {
            loop {
                match session.recv_output().await {
                    Some(data) => {
                        let output = TerminalOutput::new(&session_id, &data);

                        // Emit to frontend
                        if let Some(ref app) = app_handle {
                            if let Err(e) = app.emit("terminal-output", &output) {
                                error!("Failed to emit terminal output: {}", e);
                            }
                        }
                    }
                    None => {
                        debug!("Terminal {} output channel closed", session_id);
                        break;
                    }
                }
            }

            // Notify that session ended
            if let Some(ref app) = app_handle {
                let _ = app.emit(
                    "terminal-closed",
                    serde_json::json!({ "session_id": session_id }),
                );
            }
        });
    }

    /// Write input to a terminal session
    pub async fn write_input(&self, session_id: &str, data: &[u8]) -> Result<()> {
        let sessions = self.sessions.read().await;
        let session = sessions
            .get(session_id)
            .ok_or_else(|| anyhow!("Session not found: {}", session_id))?;

        session.write(data).await
    }

    /// Resize a terminal session
    pub async fn resize_session(&self, session_id: &str, cols: u16, rows: u16) -> Result<()> {
        let sessions = self.sessions.read().await;
        let session = sessions
            .get(session_id)
            .ok_or_else(|| anyhow!("Session not found: {}", session_id))?;

        session.resize(cols, rows).await
    }

    /// Get session info
    pub async fn get_session(&self, session_id: &str) -> Option<TerminalInfo> {
        let sessions = self.sessions.read().await;
        if let Some(s) = sessions.get(session_id) {
            Some(TerminalInfo {
                session_id: s.id.clone(),
                shell: s.shell.clone(),
                cwd: s.cwd.clone(),
                cols: s.cols().await,
                rows: s.rows().await,
                created_at: s.created_at,
                is_active: s.is_active().await,
            })
        } else {
            None
        }
    }

    /// List all sessions
    pub async fn list_sessions(&self) -> Vec<TerminalInfo> {
        let sessions = self.sessions.read().await;
        let mut infos = Vec::new();
        for s in sessions.values() {
            infos.push(TerminalInfo {
                session_id: s.id.clone(),
                shell: s.shell.clone(),
                cwd: s.cwd.clone(),
                cols: s.cols().await,
                rows: s.rows().await,
                created_at: s.created_at,
                is_active: s.is_active().await,
            });
        }
        infos
    }

    /// Close a terminal session
    pub async fn close_session(&self, session_id: &str) -> Result<()> {
        let session = {
            let mut sessions = self.sessions.write().await;
            sessions.remove(session_id)
        };

        if let Some(session) = session {
            session.stop().await;
            info!("Closed terminal session: {}", session_id);
            Ok(())
        } else {
            Err(anyhow!("Session not found: {}", session_id))
        }
    }

    /// Close all sessions
    pub async fn close_all(&self) {
        let sessions: Vec<Arc<TerminalSession>> = {
            let mut sessions_map = self.sessions.write().await;
            sessions_map.drain().map(|(_, s)| s).collect()
        };

        for session in sessions {
            session.stop().await;
        }

        info!("Closed all terminal sessions");
    }

    /// Get number of active sessions
    pub async fn session_count(&self) -> usize {
        self.sessions.read().await.len()
    }
}

impl Default for TerminalManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_terminal_manager_create() {
        let manager = TerminalManager::new();
        assert_eq!(manager.session_count().await, 0);
    }
}
