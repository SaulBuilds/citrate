// lattice-v3/core/network/src/sync.rs

// Synchronization manager for block and header downloads
use crate::{
    peer::{Peer, PeerId},
    NetworkError, NetworkMessage,
};
use lattice_consensus::types::{Block, BlockHeader, Hash};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Synchronization state
#[derive(Debug, Clone, PartialEq)]
pub enum SyncState {
    /// Not syncing
    Idle,

    /// Downloading headers
    DownloadingHeaders {
        from: Hash,
        target_height: u64,
        progress: f32,
    },

    /// Downloading blocks
    DownloadingBlocks {
        from_height: u64,
        to_height: u64,
        progress: f32,
    },

    /// Verifying downloaded data
    Verifying {
        blocks_verified: u64,
        total_blocks: u64,
    },

    /// Synchronized with network
    Synced,
}

/// Block download request
#[derive(Debug)]
#[allow(dead_code)]
struct BlockRequest {
    hash: Hash,
    peer_id: PeerId,
    requested_at: Instant,
    retries: u32,
}

/// Synchronization manager
pub struct SyncManager {
    config: SyncConfig,
    state: Arc<RwLock<SyncState>>,

    // Current sync progress
    current_height: Arc<RwLock<u64>>,
    target_height: Arc<RwLock<u64>>,

    // Download queue
    header_queue: Arc<RwLock<VecDeque<Hash>>>,
    block_queue: Arc<RwLock<VecDeque<Hash>>>,

    // Pending requests
    pending_headers: Arc<RwLock<HashMap<Hash, BlockRequest>>>,
    pending_blocks: Arc<RwLock<HashMap<Hash, BlockRequest>>>,

    // Downloaded but not yet processed
    downloaded_headers: Arc<RwLock<Vec<BlockHeader>>>,
    downloaded_blocks: Arc<RwLock<Vec<Block>>>,
}

#[derive(Debug, Clone)]
pub struct SyncConfig {
    /// Maximum concurrent block downloads
    pub max_concurrent_downloads: usize,

    /// Block request timeout
    pub request_timeout: Duration,

    /// Maximum retries for a block
    pub max_retries: u32,

    /// Batch size for header downloads
    pub header_batch_size: u32,

    /// Batch size for block downloads  
    pub block_batch_size: u32,

    /// Sync interval
    pub sync_interval: Duration,
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            max_concurrent_downloads: 16,
            request_timeout: Duration::from_secs(30),
            max_retries: 3,
            header_batch_size: 2000,
            block_batch_size: 128,
            sync_interval: Duration::from_secs(1),
        }
    }
}

impl SyncManager {
    pub fn new(config: SyncConfig) -> Self {
        Self {
            config,
            state: Arc::new(RwLock::new(SyncState::Idle)),
            current_height: Arc::new(RwLock::new(0)),
            target_height: Arc::new(RwLock::new(0)),
            header_queue: Arc::new(RwLock::new(VecDeque::new())),
            block_queue: Arc::new(RwLock::new(VecDeque::new())),
            pending_headers: Arc::new(RwLock::new(HashMap::new())),
            pending_blocks: Arc::new(RwLock::new(HashMap::new())),
            downloaded_headers: Arc::new(RwLock::new(Vec::new())),
            downloaded_blocks: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Get current sync state
    pub async fn get_state(&self) -> SyncState {
        self.state.read().await.clone()
    }

    /// Check if synced
    pub async fn is_synced(&self) -> bool {
        matches!(*self.state.read().await, SyncState::Synced)
    }

    /// Start synchronization
    pub async fn start_sync(&self, peer_height: u64, peer_hash: Hash) -> Result<(), NetworkError> {
        let current = *self.current_height.read().await;

        if peer_height <= current {
            *self.state.write().await = SyncState::Synced;
            return Ok(());
        }

        *self.target_height.write().await = peer_height;

        // Start with header download
        *self.state.write().await = SyncState::DownloadingHeaders {
            from: peer_hash,
            target_height: peer_height,
            progress: 0.0,
        };

        info!("Starting sync from height {} to {}", current, peer_height);

        // Queue initial header requests
        self.queue_header_downloads(peer_hash, peer_height - current)
            .await;

        Ok(())
    }

    /// Queue header downloads
    async fn queue_header_downloads(&self, from: Hash, count: u64) {
        let mut queue = self.header_queue.write().await;

        // Add to queue in batches
        let batches = count.div_ceil(self.config.header_batch_size as u64);

        for _ in 0..batches.min(10) {
            queue.push_back(from);
        }
    }

    /// Process header download request
    pub async fn request_headers(&self, peer: &Peer, from: Hash) -> Result<(), NetworkError> {
        let peer_id = peer.info.read().await.id.clone();

        // Check if already pending
        if self.pending_headers.read().await.contains_key(&from) {
            return Ok(());
        }

        // Send request
        peer.send(NetworkMessage::GetHeaders {
            from,
            count: self.config.header_batch_size,
        })
        .await?;

        // Track request
        self.pending_headers.write().await.insert(
            from,
            BlockRequest {
                hash: from,
                peer_id,
                requested_at: Instant::now(),
                retries: 0,
            },
        );

        Ok(())
    }

    /// Process block download request
    pub async fn request_blocks(&self, peer: &Peer, from: Hash) -> Result<(), NetworkError> {
        let peer_id = peer.info.read().await.id.clone();

        // Check if already pending
        if self.pending_blocks.read().await.contains_key(&from) {
            return Ok(());
        }

        // Send request
        peer.send(NetworkMessage::GetBlocks {
            from,
            count: self.config.block_batch_size,
            step: 1,
        })
        .await?;

        // Track request
        self.pending_blocks.write().await.insert(
            from,
            BlockRequest {
                hash: from,
                peer_id,
                requested_at: Instant::now(),
                retries: 0,
            },
        );

        Ok(())
    }

    /// Handle received headers
    pub async fn handle_headers(&self, headers: Vec<BlockHeader>) -> Result<(), NetworkError> {
        if headers.is_empty() {
            return Ok(());
        }

        let count = headers.len();
        let first_height = headers.first().unwrap().height;
        let last_height = headers.last().unwrap().height;
        let first_hash = headers.first().map(|h| h.block_hash).unwrap_or_default();

        // Store headers
        self.downloaded_headers.write().await.extend(headers);

        // Update progress
        let current = *self.current_height.read().await;
        let target = *self.target_height.read().await;
        let progress = ((last_height - current) as f32 / (target - current) as f32) * 100.0;

        *self.state.write().await = SyncState::DownloadingHeaders {
            from: first_hash,
            target_height: target,
            progress,
        };

        info!(
            "Downloaded {} headers (height {}-{}), progress: {:.1}%",
            count, first_height, last_height, progress
        );

        // Transition to block download if headers complete
        if last_height >= target {
            self.start_block_download().await?;
        }

        Ok(())
    }

    /// Handle received blocks
    pub async fn handle_blocks(&self, blocks: Vec<Block>) -> Result<(), NetworkError> {
        if blocks.is_empty() {
            return Ok(());
        }

        let count = blocks.len();
        let first_height = blocks.first().unwrap().header.height;
        let last_height = blocks.last().unwrap().header.height;

        // Store blocks
        self.downloaded_blocks.write().await.extend(blocks);

        // Update progress
        let current = *self.current_height.read().await;
        let target = *self.target_height.read().await;
        let progress = ((last_height - current) as f32 / (target - current) as f32) * 100.0;

        *self.state.write().await = SyncState::DownloadingBlocks {
            from_height: current,
            to_height: target,
            progress,
        };

        // Update current height
        *self.current_height.write().await = last_height;

        info!(
            "Downloaded {} blocks (height {}-{}), progress: {:.1}%",
            count, first_height, last_height, progress
        );

        // Check if sync complete
        if last_height >= target {
            *self.state.write().await = SyncState::Synced;
            info!("Synchronization complete at height {}", last_height);
        }

        Ok(())
    }

    /// Start block download phase
    async fn start_block_download(&self) -> Result<(), NetworkError> {
        let current = *self.current_height.read().await;
        let target = *self.target_height.read().await;

        *self.state.write().await = SyncState::DownloadingBlocks {
            from_height: current,
            to_height: target,
            progress: 0.0,
        };

        info!(
            "Starting block download from height {} to {}",
            current, target
        );

        // Queue block downloads based on headers
        let headers = self.downloaded_headers.read().await;
        let mut block_queue = self.block_queue.write().await;

        // Queue blocks from downloaded headers
        for header in headers.iter().take(self.config.max_concurrent_downloads) {
            block_queue.push_back(header.block_hash);
        }

        debug!("Queued {} blocks for download", block_queue.len());

        Ok(())
    }

    /// Check for timed out requests
    pub async fn check_timeouts(&self) -> Vec<Hash> {
        let mut timed_out = Vec::new();
        let now = Instant::now();

        // Check header requests
        {
            let mut pending = self.pending_headers.write().await;
            pending.retain(|hash, request| {
                if now.duration_since(request.requested_at) > self.config.request_timeout {
                    timed_out.push(*hash);
                    false
                } else {
                    true
                }
            });
        }

        // Check block requests
        {
            let mut pending = self.pending_blocks.write().await;
            pending.retain(|hash, request| {
                if now.duration_since(request.requested_at) > self.config.request_timeout {
                    timed_out.push(*hash);
                    false
                } else {
                    true
                }
            });
        }

        if !timed_out.is_empty() {
            warn!("Sync requests timed out: {} items", timed_out.len());
        }

        timed_out
    }

    /// Get sync progress
    pub async fn get_progress(&self) -> (u64, u64, f32) {
        let current = *self.current_height.read().await;
        let target = *self.target_height.read().await;

        let progress = if target > current {
            ((current as f32 / target as f32) * 100.0).min(100.0)
        } else {
            100.0
        };

        (current, target, progress)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_sync_state_transitions() {
        let sync = SyncManager::new(SyncConfig::default());

        // Initially idle
        assert_eq!(sync.get_state().await, SyncState::Idle);

        // Start sync
        sync.start_sync(100, Hash::default()).await.unwrap();

        // Should be downloading headers
        match sync.get_state().await {
            SyncState::DownloadingHeaders { target_height, .. } => {
                assert_eq!(target_height, 100);
            }
            _ => panic!("Expected DownloadingHeaders state"),
        }
    }

    #[tokio::test]
    async fn test_sync_progress() {
        let sync = SyncManager::new(SyncConfig::default());

        *sync.current_height.write().await = 50;
        *sync.target_height.write().await = 100;

        let (current, target, progress) = sync.get_progress().await;

        assert_eq!(current, 50);
        assert_eq!(target, 100);
        assert_eq!(progress, 50.0);
    }
}
