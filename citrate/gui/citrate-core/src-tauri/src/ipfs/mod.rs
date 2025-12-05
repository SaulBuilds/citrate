//! IPFS Node Management Module
//!
//! Provides embedded IPFS node management for Citrate.
//! Handles daemon lifecycle, content operations, and gateway configuration.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// IPFS daemon status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpfsStatus {
    pub running: bool,
    pub peer_id: Option<String>,
    pub addresses: Vec<String>,
    pub repo_size: Option<u64>,
    pub num_objects: Option<u64>,
    pub version: Option<String>,
}

/// IPFS configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpfsConfig {
    /// Path to IPFS binary (kubo)
    pub binary_path: Option<String>,
    /// IPFS repository path
    pub repo_path: PathBuf,
    /// API port (default 5001)
    pub api_port: u16,
    /// Gateway port (default 8080)
    pub gateway_port: u16,
    /// Swarm port (default 4001)
    pub swarm_port: u16,
    /// External gateways for fallback
    pub external_gateways: Vec<String>,
    /// Enable pubsub (for real-time features)
    pub enable_pubsub: bool,
    /// Bootstrap peers
    pub bootstrap_peers: Vec<String>,
}

impl Default for IpfsConfig {
    fn default() -> Self {
        let repo_path = dirs::data_local_dir()
            .map(|d| d.join("citrate").join("ipfs"))
            .unwrap_or_else(|| PathBuf::from(".citrate/ipfs"));

        Self {
            binary_path: None, // Will auto-detect
            repo_path,
            api_port: 5001,
            gateway_port: 8080,
            swarm_port: 4001,
            external_gateways: vec![
                "https://ipfs.io/ipfs/".to_string(),
                "https://gateway.pinata.cloud/ipfs/".to_string(),
                "https://cloudflare-ipfs.com/ipfs/".to_string(),
                "https://dweb.link/ipfs/".to_string(),
            ],
            enable_pubsub: true,
            bootstrap_peers: vec![
                // Official IPFS bootstrap nodes
                "/dnsaddr/bootstrap.libp2p.io/p2p/QmNnooDu7bfjPFoTZYxMNLWUQJyrVwtbZg5gBMjTezGAJN".to_string(),
                "/dnsaddr/bootstrap.libp2p.io/p2p/QmQCU2EcMqAqQPR2i9bChDtGNJchTbq5TbXJJ16u19uLTa".to_string(),
                "/dnsaddr/bootstrap.libp2p.io/p2p/QmbLHAnMoJPWSCR5Zhtx6BHJX9KiKNN6tpvbUcqanj75Nb".to_string(),
            ],
        }
    }
}

/// IPFS content result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpfsContent {
    pub cid: String,
    pub size: u64,
    pub content_type: Option<String>,
    pub data: Option<Vec<u8>>,
    pub gateway_url: String,
}

/// IPFS add result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpfsAddResult {
    pub cid: String,
    pub size: u64,
    pub name: String,
    pub gateway_url: String,
}

/// IPFS Node Manager
pub struct IpfsManager {
    config: Arc<RwLock<IpfsConfig>>,
    daemon_process: Arc<RwLock<Option<Child>>>,
    status: Arc<RwLock<IpfsStatus>>,
    http_client: reqwest::Client,
}

impl IpfsManager {
    /// Create new IPFS manager with default config
    pub fn new() -> Self {
        Self::with_config(IpfsConfig::default())
    }

    /// Create IPFS manager with custom config
    pub fn with_config(config: IpfsConfig) -> Self {
        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        Self {
            config: Arc::new(RwLock::new(config)),
            daemon_process: Arc::new(RwLock::new(None)),
            status: Arc::new(RwLock::new(IpfsStatus {
                running: false,
                peer_id: None,
                addresses: vec![],
                repo_size: None,
                num_objects: None,
                version: None,
            })),
            http_client,
        }
    }

    /// Get current configuration
    pub async fn get_config(&self) -> IpfsConfig {
        self.config.read().await.clone()
    }

    /// Update configuration
    pub async fn update_config(&self, config: IpfsConfig) {
        *self.config.write().await = config;
    }

    /// Get current status
    pub async fn get_status(&self) -> IpfsStatus {
        self.status.read().await.clone()
    }

    /// Find IPFS binary path
    fn find_ipfs_binary(&self, config: &IpfsConfig) -> Option<PathBuf> {
        // Check configured path first
        if let Some(ref path) = config.binary_path {
            let p = PathBuf::from(path);
            if p.exists() {
                return Some(p);
            }
        }

        // Check common locations
        let common_paths = [
            "/usr/local/bin/ipfs",
            "/usr/bin/ipfs",
            "/opt/homebrew/bin/ipfs",
            "~/.local/bin/ipfs",
        ];

        for path in common_paths {
            let expanded = shellexpand::tilde(path);
            let p = PathBuf::from(expanded.as_ref());
            if p.exists() {
                return Some(p);
            }
        }

        // Try PATH
        if let Ok(output) = Command::new("which").arg("ipfs").output() {
            if output.status.success() {
                let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !path.is_empty() {
                    return Some(PathBuf::from(path));
                }
            }
        }

        None
    }

    /// Initialize IPFS repository if needed
    async fn init_repo(&self) -> Result<(), String> {
        let config = self.config.read().await;
        let repo_path = &config.repo_path;

        // Check if repo already exists
        if repo_path.join("config").exists() {
            debug!("IPFS repo already initialized at {:?}", repo_path);
            return Ok(());
        }

        // Create directory
        std::fs::create_dir_all(repo_path)
            .map_err(|e| format!("Failed to create IPFS repo directory: {}", e))?;

        let ipfs_binary = self
            .find_ipfs_binary(&config)
            .ok_or_else(|| "IPFS binary not found. Please install kubo.".to_string())?;

        // Initialize repo
        info!("Initializing IPFS repo at {:?}", repo_path);
        let output = Command::new(&ipfs_binary)
            .env("IPFS_PATH", repo_path)
            .args(["init", "--profile", "lowpower"])
            .output()
            .map_err(|e| format!("Failed to run ipfs init: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            // Ignore "already initialized" error
            if !stderr.contains("already") {
                return Err(format!("ipfs init failed: {}", stderr));
            }
        }

        info!("IPFS repo initialized successfully");
        Ok(())
    }

    /// Start the IPFS daemon
    pub async fn start(&self) -> Result<(), String> {
        // Check if already running
        if self.is_running().await {
            info!("IPFS daemon already running");
            return Ok(());
        }

        let config = self.config.read().await.clone();

        // Find binary
        let ipfs_binary = self
            .find_ipfs_binary(&config)
            .ok_or_else(|| "IPFS binary not found. Please install kubo.".to_string())?;

        // Initialize repo if needed
        drop(config);
        self.init_repo().await?;
        let config = self.config.read().await.clone();

        info!("Starting IPFS daemon...");

        // Build daemon command
        let mut cmd = Command::new(&ipfs_binary);
        cmd.env("IPFS_PATH", &config.repo_path)
            .args(["daemon", "--enable-gc"])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        if config.enable_pubsub {
            cmd.arg("--enable-pubsub-experiment");
        }

        // Start daemon
        let child = cmd
            .spawn()
            .map_err(|e| format!("Failed to start IPFS daemon: {}", e))?;

        *self.daemon_process.write().await = Some(child);

        // Wait for daemon to be ready
        let api_url = format!("http://127.0.0.1:{}/api/v0/id", config.api_port);
        let mut attempts = 0;
        let max_attempts = 30;

        while attempts < max_attempts {
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;

            if let Ok(response) = self.http_client.post(&api_url).send().await {
                if response.status().is_success() {
                    info!("IPFS daemon started successfully");
                    self.refresh_status().await?;
                    return Ok(());
                }
            }
            attempts += 1;
        }

        Err("IPFS daemon failed to start within timeout".to_string())
    }

    /// Stop the IPFS daemon
    pub async fn stop(&self) -> Result<(), String> {
        let config = self.config.read().await;

        // Try graceful shutdown via API
        let shutdown_url = format!(
            "http://127.0.0.1:{}/api/v0/shutdown",
            config.api_port
        );

        let _ = self.http_client.post(&shutdown_url).send().await;

        // Wait a moment for graceful shutdown
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;

        // Force kill if still running
        if let Some(mut child) = self.daemon_process.write().await.take() {
            let _ = child.kill();
            let _ = child.wait();
        }

        *self.status.write().await = IpfsStatus {
            running: false,
            peer_id: None,
            addresses: vec![],
            repo_size: None,
            num_objects: None,
            version: None,
        };

        info!("IPFS daemon stopped");
        Ok(())
    }

    /// Check if daemon is running
    pub async fn is_running(&self) -> bool {
        let config = self.config.read().await;
        let api_url = format!("http://127.0.0.1:{}/api/v0/id", config.api_port);

        if let Ok(response) = self
            .http_client
            .post(&api_url)
            .timeout(std::time::Duration::from_secs(2))
            .send()
            .await
        {
            return response.status().is_success();
        }

        false
    }

    /// Refresh status from daemon
    pub async fn refresh_status(&self) -> Result<IpfsStatus, String> {
        let config = self.config.read().await;
        let api_base = format!("http://127.0.0.1:{}/api/v0", config.api_port);

        // Get peer ID and addresses
        let id_response: serde_json::Value = self
            .http_client
            .post(format!("{}/id", api_base))
            .send()
            .await
            .map_err(|e| format!("Failed to get IPFS ID: {}", e))?
            .json()
            .await
            .map_err(|e| format!("Failed to parse IPFS ID: {}", e))?;

        let peer_id = id_response
            .get("ID")
            .and_then(|v| v.as_str())
            .map(String::from);

        let addresses: Vec<String> = id_response
            .get("Addresses")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();

        // Get version
        let version_response: serde_json::Value = self
            .http_client
            .post(format!("{}/version", api_base))
            .send()
            .await
            .ok()
            .and_then(|r| futures::executor::block_on(r.json()).ok())
            .unwrap_or_default();

        let version = version_response
            .get("Version")
            .and_then(|v| v.as_str())
            .map(String::from);

        // Get repo stats
        let stat_response: serde_json::Value = self
            .http_client
            .post(format!("{}/repo/stat", api_base))
            .send()
            .await
            .ok()
            .and_then(|r| futures::executor::block_on(r.json()).ok())
            .unwrap_or_default();

        let repo_size = stat_response
            .get("RepoSize")
            .and_then(|v| v.as_u64());

        let num_objects = stat_response
            .get("NumObjects")
            .and_then(|v| v.as_u64());

        let status = IpfsStatus {
            running: true,
            peer_id,
            addresses,
            repo_size,
            num_objects,
            version,
        };

        *self.status.write().await = status.clone();
        Ok(status)
    }

    /// Add content to IPFS
    pub async fn add(&self, data: Vec<u8>, name: Option<&str>) -> Result<IpfsAddResult, String> {
        let config = self.config.read().await;
        let api_url = format!("http://127.0.0.1:{}/api/v0/add", config.api_port);

        let file_name = name.unwrap_or("file");
        let part = reqwest::multipart::Part::bytes(data.clone()).file_name(file_name.to_string());
        let form = reqwest::multipart::Form::new().part("file", part);

        let response: serde_json::Value = self
            .http_client
            .post(&api_url)
            .multipart(form)
            .send()
            .await
            .map_err(|e| format!("Failed to add to IPFS: {}", e))?
            .json()
            .await
            .map_err(|e| format!("Failed to parse IPFS response: {}", e))?;

        let cid = response
            .get("Hash")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "No CID in response".to_string())?
            .to_string();

        let size = response
            .get("Size")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(data.len() as u64);

        let gateway_url = format!("{}{}", config.external_gateways[0], cid);

        Ok(IpfsAddResult {
            cid,
            size,
            name: file_name.to_string(),
            gateway_url,
        })
    }

    /// Add file from path
    pub async fn add_file(&self, path: &std::path::Path) -> Result<IpfsAddResult, String> {
        let data = tokio::fs::read(path)
            .await
            .map_err(|e| format!("Failed to read file: {}", e))?;

        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string());

        self.add(data, name.as_deref()).await
    }

    /// Get content from IPFS (tries local first, then gateways)
    pub async fn get(&self, cid: &str) -> Result<IpfsContent, String> {
        let config = self.config.read().await;

        // Try local daemon first
        let api_url = format!(
            "http://127.0.0.1:{}/api/v0/cat?arg={}",
            config.api_port, cid
        );

        if let Ok(response) = self.http_client.post(&api_url).send().await {
            if response.status().is_success() {
                let content_type = response
                    .headers()
                    .get("content-type")
                    .and_then(|h| h.to_str().ok())
                    .map(String::from);

                let data = response
                    .bytes()
                    .await
                    .map_err(|e| format!("Failed to read IPFS content: {}", e))?
                    .to_vec();

                return Ok(IpfsContent {
                    cid: cid.to_string(),
                    size: data.len() as u64,
                    content_type,
                    data: Some(data),
                    gateway_url: format!("{}{}", config.external_gateways[0], cid),
                });
            }
        }

        // Fall back to gateways
        for gateway in &config.external_gateways {
            let url = format!("{}{}", gateway, cid);

            if let Ok(response) = self
                .http_client
                .get(&url)
                .timeout(std::time::Duration::from_secs(30))
                .send()
                .await
            {
                if response.status().is_success() {
                    let content_type = response
                        .headers()
                        .get("content-type")
                        .and_then(|h| h.to_str().ok())
                        .map(String::from);

                    let data = response.bytes().await.ok().map(|b| b.to_vec());
                    let size = data.as_ref().map(|d| d.len() as u64).unwrap_or(0);

                    return Ok(IpfsContent {
                        cid: cid.to_string(),
                        size,
                        content_type,
                        data,
                        gateway_url: url,
                    });
                }
            }
        }

        Err(format!("Failed to retrieve CID {} from any source", cid))
    }

    /// Pin content to local node
    pub async fn pin(&self, cid: &str) -> Result<(), String> {
        let config = self.config.read().await;
        let api_url = format!(
            "http://127.0.0.1:{}/api/v0/pin/add?arg={}",
            config.api_port, cid
        );

        let response = self
            .http_client
            .post(&api_url)
            .send()
            .await
            .map_err(|e| format!("Failed to pin: {}", e))?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(format!("Pin failed: {}", response.status()))
        }
    }

    /// Unpin content from local node
    pub async fn unpin(&self, cid: &str) -> Result<(), String> {
        let config = self.config.read().await;
        let api_url = format!(
            "http://127.0.0.1:{}/api/v0/pin/rm?arg={}",
            config.api_port, cid
        );

        let response = self
            .http_client
            .post(&api_url)
            .send()
            .await
            .map_err(|e| format!("Failed to unpin: {}", e))?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(format!("Unpin failed: {}", response.status()))
        }
    }

    /// List pinned content
    pub async fn list_pins(&self) -> Result<Vec<String>, String> {
        let config = self.config.read().await;
        let api_url = format!(
            "http://127.0.0.1:{}/api/v0/pin/ls",
            config.api_port
        );

        let response: serde_json::Value = self
            .http_client
            .post(&api_url)
            .send()
            .await
            .map_err(|e| format!("Failed to list pins: {}", e))?
            .json()
            .await
            .map_err(|e| format!("Failed to parse pins: {}", e))?;

        let keys = response
            .get("Keys")
            .and_then(|v| v.as_object())
            .map(|obj| obj.keys().cloned().collect())
            .unwrap_or_default();

        Ok(keys)
    }

    /// Get connected peers
    pub async fn get_peers(&self) -> Result<Vec<String>, String> {
        let config = self.config.read().await;
        let api_url = format!(
            "http://127.0.0.1:{}/api/v0/swarm/peers",
            config.api_port
        );

        let response: serde_json::Value = self
            .http_client
            .post(&api_url)
            .send()
            .await
            .map_err(|e| format!("Failed to get peers: {}", e))?
            .json()
            .await
            .map_err(|e| format!("Failed to parse peers: {}", e))?;

        let peers = response
            .get("Peers")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|p| {
                        p.get("Peer")
                            .and_then(|v| v.as_str())
                            .map(String::from)
                    })
                    .collect()
            })
            .unwrap_or_default();

        Ok(peers)
    }
}

impl Default for IpfsManager {
    fn default() -> Self {
        Self::new()
    }
}

// Cleanup on drop
impl Drop for IpfsManager {
    fn drop(&mut self) {
        // Try to stop daemon gracefully using blocking read
        // Note: We use try_write() to avoid deadlocks in drop
        if let Ok(mut guard) = self.daemon_process.try_write() {
            if let Some(mut child) = guard.take() {
                let _ = child.kill();
            }
        }
    }
}
