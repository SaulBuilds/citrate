# IPFS Integration Technical Specification

**Version**: 1.0
**Date**: 2025-12-03
**Status**: Planning

---

## Overview

Citrate requires IPFS integration for:
1. Model weight storage and distribution
2. Training data storage
3. Decentralized asset management
4. Off-chain data for smart contracts

---

## Architecture

### System Components

```
┌─────────────────────────────────────────────────────────────────┐
│                     Citrate Application                         │
├─────────────────────────────────────────────────────────────────┤
│  ┌─────────────────┐    ┌─────────────────┐                     │
│  │  IPFS Manager   │───▶│  Local Daemon   │                     │
│  │  (Rust)         │    │  (kubo)         │                     │
│  └────────┬────────┘    └────────┬────────┘                     │
│           │                      │                              │
│           │                      ▼                              │
│           │             ┌─────────────────┐                     │
│           │             │  IPFS Network   │                     │
│           │             │  (DHT)          │                     │
│           │             └─────────────────┘                     │
│           │                      │                              │
│           │                      ▼                              │
│           │             ┌─────────────────┐                     │
│           └────────────▶│  Gateway        │◀───── Fallback     │
│                         │  (configurable) │                     │
│                         └─────────────────┘                     │
└─────────────────────────────────────────────────────────────────┘
```

### Configuration Options

```toml
# citrate.toml
[ipfs]
# Local daemon settings
daemon_enabled = true
daemon_path = "ipfs"  # or full path to kubo binary
api_port = 5001
gateway_port = 8080

# External gateway fallback
external_gateway = "https://lola-subcorneous-tucker.ngrok-free.dev"
fallback_gateway = "https://ipfs.io"

# Model storage settings
models_path = "~/.citrate/models"
max_model_size_gb = 20
chunk_size_mb = 256

# Pinning settings
auto_pin = true
pin_timeout_seconds = 300
```

---

## Implementation Plan

### Phase 1: Daemon Management

```rust
// src-tauri/src/ipfs/daemon.rs

use std::process::{Command, Child};
use tokio::sync::RwLock;

pub struct IPFSDaemon {
    process: Option<Child>,
    api_url: String,
    status: RwLock<DaemonStatus>,
}

#[derive(Clone, Debug)]
pub enum DaemonStatus {
    NotInstalled,
    Stopped,
    Starting,
    Running { peer_id: String, addresses: Vec<String> },
    Error(String),
}

impl IPFSDaemon {
    /// Check if IPFS is installed
    pub async fn check_installation() -> Result<bool, IPFSError> {
        match Command::new("ipfs").arg("--version").output() {
            Ok(output) => Ok(output.status.success()),
            Err(_) => Ok(false),
        }
    }

    /// Start the IPFS daemon
    pub async fn start(&mut self) -> Result<(), IPFSError> {
        // Check if already running
        if self.is_running().await? {
            return Ok(());
        }

        // Initialize if needed
        if !Self::is_initialized().await? {
            Self::initialize().await?;
        }

        // Start daemon
        let child = Command::new("ipfs")
            .arg("daemon")
            .arg("--enable-gc")
            .spawn()?;

        self.process = Some(child);

        // Wait for API to be available
        self.wait_for_api(30).await?;

        Ok(())
    }

    /// Stop the daemon
    pub async fn stop(&mut self) -> Result<(), IPFSError> {
        if let Some(mut process) = self.process.take() {
            process.kill()?;
        }
        Ok(())
    }

    /// Health check
    pub async fn health(&self) -> Result<DaemonStatus, IPFSError> {
        let client = reqwest::Client::new();
        let response = client
            .post(format!("{}/api/v0/id", self.api_url))
            .send()
            .await?;

        if response.status().is_success() {
            let info: PeerInfo = response.json().await?;
            Ok(DaemonStatus::Running {
                peer_id: info.id,
                addresses: info.addresses,
            })
        } else {
            Ok(DaemonStatus::Stopped)
        }
    }
}
```

### Phase 2: Model Download

```rust
// src-tauri/src/ipfs/downloader.rs

use futures::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};

pub struct ModelDownloader {
    ipfs_api: String,
    gateway_url: String,
    models_dir: PathBuf,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DownloadProgress {
    pub cid: String,
    pub total_bytes: u64,
    pub downloaded_bytes: u64,
    pub progress_percent: f32,
    pub speed_mbps: f32,
    pub eta_seconds: u64,
    pub status: DownloadStatus,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum DownloadStatus {
    Queued,
    Connecting,
    Downloading,
    Verifying,
    Complete,
    Failed(String),
    Paused,
}

impl ModelDownloader {
    /// Download a model by CID
    pub async fn download(
        &self,
        cid: &str,
        filename: &str,
        progress_tx: mpsc::Sender<DownloadProgress>,
    ) -> Result<PathBuf, IPFSError> {
        let dest_path = self.models_dir.join(filename);

        // Try local daemon first
        if let Ok(result) = self.download_from_daemon(cid, &dest_path, &progress_tx).await {
            return Ok(result);
        }

        // Fallback to gateway
        self.download_from_gateway(cid, &dest_path, &progress_tx).await
    }

    async fn download_from_daemon(
        &self,
        cid: &str,
        dest: &Path,
        progress_tx: &mpsc::Sender<DownloadProgress>,
    ) -> Result<PathBuf, IPFSError> {
        let client = reqwest::Client::new();

        // Get file size first
        let stat_url = format!("{}/api/v0/files/stat?arg=/ipfs/{}", self.ipfs_api, cid);
        let stat_response: FileStat = client.post(&stat_url).send().await?.json().await?;
        let total_size = stat_response.cumulative_size;

        // Stream download
        let cat_url = format!("{}/api/v0/cat?arg={}", self.ipfs_api, cid);
        let response = client.post(&cat_url).send().await?;

        let mut file = tokio::fs::File::create(dest).await?;
        let mut stream = response.bytes_stream();
        let mut downloaded = 0u64;
        let start_time = std::time::Instant::now();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            file.write_all(&chunk).await?;
            downloaded += chunk.len() as u64;

            let elapsed = start_time.elapsed().as_secs_f32();
            let speed = downloaded as f32 / elapsed / 1_000_000.0;
            let eta = if speed > 0.0 {
                ((total_size - downloaded) as f32 / speed / 1_000_000.0) as u64
            } else {
                0
            };

            progress_tx.send(DownloadProgress {
                cid: cid.to_string(),
                total_bytes: total_size,
                downloaded_bytes: downloaded,
                progress_percent: (downloaded as f32 / total_size as f32) * 100.0,
                speed_mbps: speed,
                eta_seconds: eta,
                status: DownloadStatus::Downloading,
            }).await?;
        }

        // Verify CID
        progress_tx.send(DownloadProgress {
            status: DownloadStatus::Verifying,
            ..Default::default()
        }).await?;

        self.verify_cid(dest, cid).await?;

        Ok(dest.to_path_buf())
    }

    /// Verify downloaded file matches CID
    async fn verify_cid(&self, path: &Path, expected_cid: &str) -> Result<(), IPFSError> {
        let client = reqwest::Client::new();

        // Use IPFS to add and check CID without actually adding
        let add_url = format!("{}/api/v0/add?only-hash=true&quiet=true", self.ipfs_api);

        let file = tokio::fs::read(path).await?;
        let form = reqwest::multipart::Form::new()
            .part("file", reqwest::multipart::Part::bytes(file));

        let response = client.post(&add_url)
            .multipart(form)
            .send()
            .await?;

        let result: AddResult = response.json().await?;

        if result.hash != expected_cid {
            return Err(IPFSError::CIDMismatch {
                expected: expected_cid.to_string(),
                actual: result.hash,
            });
        }

        Ok(())
    }
}
```

### Phase 3: First-Run Setup UI

```typescript
// src/components/setup/IPFSSetup.tsx

import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';

interface IPFSStatus {
  installed: boolean;
  running: boolean;
  peerId?: string;
  addresses?: string[];
}

export function IPFSSetup() {
  const [status, setStatus] = useState<IPFSStatus | null>(null);
  const [installing, setInstalling] = useState(false);
  const [externalGateway, setExternalGateway] = useState('');

  useEffect(() => {
    checkIPFSStatus();
  }, []);

  const checkIPFSStatus = async () => {
    const result = await invoke<IPFSStatus>('ipfs_check_status');
    setStatus(result);
  };

  const installIPFS = async () => {
    setInstalling(true);
    try {
      await invoke('ipfs_install');
      await checkIPFSStatus();
    } finally {
      setInstalling(false);
    }
  };

  const startDaemon = async () => {
    await invoke('ipfs_start_daemon');
    await checkIPFSStatus();
  };

  const configureExternalGateway = async () => {
    await invoke('ipfs_configure_gateway', { url: externalGateway });
  };

  if (!status) {
    return <div>Checking IPFS status...</div>;
  }

  return (
    <div className="ipfs-setup">
      <h2>IPFS Setup</h2>

      {!status.installed ? (
        <div className="setup-step">
          <h3>Step 1: Install IPFS</h3>
          <p>IPFS is required to download and manage AI models.</p>
          <button onClick={installIPFS} disabled={installing}>
            {installing ? 'Installing...' : 'Install IPFS'}
          </button>
          <p className="hint">
            Or install manually from{' '}
            <a href="https://docs.ipfs.tech/install/" target="_blank">
              docs.ipfs.tech
            </a>
          </p>
        </div>
      ) : !status.running ? (
        <div className="setup-step">
          <h3>Step 2: Start IPFS Daemon</h3>
          <p>Start the IPFS daemon to enable model downloads.</p>
          <button onClick={startDaemon}>Start Daemon</button>
        </div>
      ) : (
        <div className="setup-complete">
          <h3>IPFS is Running</h3>
          <p>Peer ID: {status.peerId}</p>
          <p>Addresses: {status.addresses?.join(', ')}</p>
        </div>
      )}

      <div className="external-gateway">
        <h3>External Gateway (Optional)</h3>
        <p>Configure a fallback gateway for when local daemon is unavailable.</p>
        <input
          type="url"
          value={externalGateway}
          onChange={(e) => setExternalGateway(e.target.value)}
          placeholder="https://your-gateway.example.com"
        />
        <button onClick={configureExternalGateway}>Save Gateway</button>
      </div>
    </div>
  );
}
```

---

## Model Storage Schema

### Directory Structure

```
~/.citrate/
├── models/
│   ├── registry.json         # Local model registry
│   ├── qwen2.5-coder-7b/
│   │   ├── model.gguf        # Model weights
│   │   ├── metadata.json     # Model info
│   │   └── config.json       # Inference config
│   └── llama-3-8b/
│       ├── model.gguf
│       ├── metadata.json
│       └── config.json
├── ipfs/
│   ├── config               # IPFS repo config
│   └── datastore/           # IPFS data
└── training/
    ├── datasets/            # Training datasets
    └── checkpoints/         # Training checkpoints
```

### Registry Schema

```json
{
  "version": "1.0",
  "models": [
    {
      "id": "qwen2.5-coder-7b",
      "name": "Qwen 2.5 Coder 7B",
      "cid": "QmXyz...",
      "size_bytes": 5100000000,
      "quantization": "Q4_K_M",
      "downloaded_at": "2025-12-03T00:00:00Z",
      "path": "models/qwen2.5-coder-7b/model.gguf",
      "source": "huggingface",
      "source_url": "https://huggingface.co/Qwen/Qwen2.5-Coder-7B-Instruct-GGUF",
      "verified": true
    }
  ]
}
```

---

## API Endpoints

### Tauri Commands

```rust
#[tauri::command]
async fn ipfs_check_status() -> Result<IPFSStatus, String>;

#[tauri::command]
async fn ipfs_install() -> Result<(), String>;

#[tauri::command]
async fn ipfs_start_daemon() -> Result<(), String>;

#[tauri::command]
async fn ipfs_stop_daemon() -> Result<(), String>;

#[tauri::command]
async fn ipfs_configure_gateway(url: String) -> Result<(), String>;

#[tauri::command]
async fn ipfs_download_model(
    cid: String,
    filename: String,
) -> Result<String, String>;

#[tauri::command]
async fn ipfs_pin(cid: String) -> Result<(), String>;

#[tauri::command]
async fn ipfs_unpin(cid: String) -> Result<(), String>;

#[tauri::command]
async fn ipfs_get_pins() -> Result<Vec<String>, String>;
```

---

## External Gateway Configuration

### Using Your ngrok Gateway

```toml
[ipfs]
external_gateway = "https://lola-subcorneous-tucker.ngrok-free.dev"
use_external_by_default = false  # Use local daemon first
```

### Fallback Logic

```rust
async fn download_with_fallback(cid: &str) -> Result<Vec<u8>, IPFSError> {
    // 1. Try local daemon
    if let Ok(data) = download_from_daemon(cid).await {
        return Ok(data);
    }

    // 2. Try configured external gateway
    if let Some(gateway) = &config.external_gateway {
        if let Ok(data) = download_from_gateway(gateway, cid).await {
            return Ok(data);
        }
    }

    // 3. Try public gateways
    for gateway in PUBLIC_GATEWAYS {
        if let Ok(data) = download_from_gateway(gateway, cid).await {
            return Ok(data);
        }
    }

    Err(IPFSError::AllGatewaysFailed)
}
```

---

## Security Considerations

1. **CID Verification**: Always verify downloaded content matches CID
2. **Gateway Trust**: External gateways can serve malicious content - verify CIDs
3. **Local Daemon**: Prefer local daemon for sensitive operations
4. **Rate Limiting**: Implement rate limiting for downloads
5. **Size Limits**: Enforce maximum model size limits
6. **Quarantine**: New downloads should be verified before use

---

## Testing

```bash
# Test IPFS integration
cargo test -p citrate-core --test ipfs_integration

# Test with mock daemon
IPFS_MOCK=true cargo test -p citrate-core --test ipfs_mock

# Manual testing
ipfs cat QmXyz... > test_file
```

---

*Document maintained by Citrate development team*
