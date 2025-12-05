//! Terminal Session
//!
//! Manages a single PTY session with shell process.

use anyhow::{anyhow, Result};
use portable_pty::{native_pty_system, CommandBuilder, PtyPair, PtySize};
use std::io::{Read, Write};
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex, RwLock};
use tracing::{debug, error, info};

/// Terminal configuration
#[derive(Debug, Clone)]
pub struct TerminalConfig {
    pub shell: Option<String>,
    pub cwd: Option<String>,
    pub env: Vec<(String, String)>,
    pub cols: u16,
    pub rows: u16,
}

impl Default for TerminalConfig {
    fn default() -> Self {
        Self {
            shell: None,
            cwd: None,
            env: vec![],
            cols: 80,
            rows: 24,
        }
    }
}

/// Terminal session managing a PTY
pub struct TerminalSession {
    pub id: String,
    pub shell: String,
    pub cwd: String,
    pub created_at: u64,

    cols: Arc<RwLock<u16>>,
    rows: Arc<RwLock<u16>>,
    pty_pair: Arc<Mutex<Option<PtyPair>>>,
    writer: Arc<Mutex<Option<Box<dyn Write + Send>>>>,
    output_tx: mpsc::UnboundedSender<Vec<u8>>,
    output_rx: Arc<Mutex<mpsc::UnboundedReceiver<Vec<u8>>>>,
    is_active: Arc<RwLock<bool>>,
}

impl TerminalSession {
    /// Create a new terminal session
    pub fn new(id: String, config: TerminalConfig) -> Result<Self> {
        let (output_tx, output_rx) = mpsc::unbounded_channel();

        // Determine shell
        let shell = config.shell.unwrap_or_else(|| {
            if cfg!(windows) {
                std::env::var("COMSPEC").unwrap_or_else(|_| "cmd.exe".to_string())
            } else {
                std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string())
            }
        });

        // Determine working directory
        let cwd = config.cwd.unwrap_or_else(|| {
            std::env::current_dir()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|_| {
                    dirs::home_dir()
                        .map(|p| p.to_string_lossy().to_string())
                        .unwrap_or_else(|| "/".to_string())
                })
        });

        Ok(Self {
            id,
            shell,
            cwd,
            cols: Arc::new(RwLock::new(config.cols)),
            rows: Arc::new(RwLock::new(config.rows)),
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            pty_pair: Arc::new(Mutex::new(None)),
            writer: Arc::new(Mutex::new(None)),
            output_tx,
            output_rx: Arc::new(Mutex::new(output_rx)),
            is_active: Arc::new(RwLock::new(false)),
        })
    }

    /// Get current cols
    pub async fn cols(&self) -> u16 {
        *self.cols.read().await
    }

    /// Get current rows
    pub async fn rows(&self) -> u16 {
        *self.rows.read().await
    }

    /// Start the terminal session
    pub async fn start(&self) -> Result<()> {
        let pty_system = native_pty_system();

        let cols = *self.cols.read().await;
        let rows = *self.rows.read().await;

        // Create PTY with specified size
        let pair = pty_system.openpty(PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        })?;

        // Build command
        let mut cmd = CommandBuilder::new(&self.shell);
        cmd.cwd(&self.cwd);

        // Add environment variables
        cmd.env("TERM", "xterm-256color");
        cmd.env("COLORTERM", "truecolor");

        // Spawn the shell
        let child = pair.slave.spawn_command(cmd)?;
        info!("Spawned shell {} with PID {:?}", self.shell, child.process_id());

        // Get writer for input
        let writer = pair.master.take_writer()?;

        // Get reader for output
        let mut reader = pair.master.try_clone_reader()?;

        // Store PTY pair and writer
        *self.pty_pair.lock().await = Some(pair);
        *self.writer.lock().await = Some(writer);
        *self.is_active.write().await = true;

        // Start output reader task
        let output_tx = self.output_tx.clone();
        let is_active = self.is_active.clone();
        let session_id = self.id.clone();

        std::thread::spawn(move || {
            let mut buf = [0u8; 4096];
            loop {
                match reader.read(&mut buf) {
                    Ok(0) => {
                        debug!("Terminal {} reader EOF", session_id);
                        break;
                    }
                    Ok(n) => {
                        let data = buf[..n].to_vec();
                        if output_tx.send(data).is_err() {
                            debug!("Terminal {} output channel closed", session_id);
                            break;
                        }
                    }
                    Err(e) => {
                        if e.kind() != std::io::ErrorKind::WouldBlock {
                            error!("Terminal {} read error: {}", session_id, e);
                            break;
                        }
                    }
                }
            }

            // Mark as inactive
            let rt = tokio::runtime::Handle::try_current();
            if let Ok(handle) = rt {
                let is_active = is_active.clone();
                handle.spawn(async move {
                    *is_active.write().await = false;
                });
            }
        });

        info!("Terminal session {} started", self.id);
        Ok(())
    }

    /// Write input to the terminal
    pub async fn write(&self, data: &[u8]) -> Result<()> {
        let mut writer_guard = self.writer.lock().await;
        if let Some(ref mut writer) = *writer_guard {
            writer.write_all(data)?;
            writer.flush()?;
            Ok(())
        } else {
            Err(anyhow!("Terminal writer not available"))
        }
    }

    /// Resize the terminal
    pub async fn resize(&self, cols: u16, rows: u16) -> Result<()> {
        *self.cols.write().await = cols;
        *self.rows.write().await = rows;

        let pty_guard = self.pty_pair.lock().await;
        if let Some(ref pair) = *pty_guard {
            pair.master.resize(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })?;
            info!("Terminal {} resized to {}x{}", self.id, cols, rows);
        }
        Ok(())
    }

    /// Receive output from the terminal
    pub async fn recv_output(&self) -> Option<Vec<u8>> {
        let mut rx = self.output_rx.lock().await;
        rx.recv().await
    }

    /// Check if session is active
    pub async fn is_active(&self) -> bool {
        *self.is_active.read().await
    }

    /// Stop the terminal session
    pub async fn stop(&self) {
        *self.is_active.write().await = false;

        // Close writer
        *self.writer.lock().await = None;

        // Kill the PTY
        if let Some(pair) = self.pty_pair.lock().await.take() {
            // The PTY will be closed when dropped
            drop(pair);
        }

        info!("Terminal session {} stopped", self.id);
    }
}

impl Drop for TerminalSession {
    fn drop(&mut self) {
        debug!("Dropping terminal session {}", self.id);
    }
}
