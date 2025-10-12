// Iterative sync implementation for GUI to avoid stack overflow
// Uses bounded queues and iterative processing

use anyhow::Result;
use lattice_consensus::types::{Block, Hash};
use lattice_network::{NetworkMessage, PeerManager};
use lattice_storage::StorageManager;
use std::collections::{HashSet, VecDeque};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Configuration for sync behavior
#[derive(Clone)]
pub struct SyncConfig {
    /// Maximum blocks to request in a single batch
    pub batch_size: usize,
    /// Maximum blocks to keep in memory at once
    pub max_queue_size: usize,
    /// Interval between sync attempts (seconds)
    pub sync_interval: u64,
    /// Maximum retries for failed blocks
    pub max_retries: usize,
    /// Use sparse sync (skip blocks for faster initial sync)
    pub sparse_sync: bool,
    /// Sparse sync step (e.g., 10 = every 10th block)
    pub sparse_step: usize,
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            batch_size: 50,      // Smaller batches to avoid overwhelming
            max_queue_size: 200, // Limit memory usage
            sync_interval: 5,    // Sync every 5 seconds
            max_retries: 3,
            sparse_sync: true, // Use sparse sync for initial catch-up
            sparse_step: 10,   // Every 10th block initially
        }
    }
}

/// Block with retry count for failed processing
struct BlockWithRetry {
    block: Block,
    retries: usize,
}

/// Iterative sync manager for the GUI
pub struct IterativeSyncManager {
    storage: Arc<StorageManager>,
    peer_manager: Arc<PeerManager>,
    config: SyncConfig,
    /// Blocks waiting to be processed
    pending_blocks: Arc<RwLock<VecDeque<BlockWithRetry>>>,
    /// Blocks we've seen to avoid duplicates
    seen_blocks: Arc<RwLock<HashSet<Hash>>>,
    /// Current sync state
    sync_state: Arc<RwLock<SyncState>>,
    /// Failed blocks for retry
    failed_blocks: Arc<RwLock<VecDeque<BlockWithRetry>>>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct SyncState {
    pub syncing: bool,
    pub current_height: u64,
    pub target_height: u64,
    pub processed_blocks: usize,
    pub failed_blocks: usize,
    pub pending_count: usize,
}

impl IterativeSyncManager {
    pub fn new(
        storage: Arc<StorageManager>,
        peer_manager: Arc<PeerManager>,
        config: Option<SyncConfig>,
    ) -> Self {
        Self {
            storage,
            peer_manager,
            config: config.unwrap_or_default(),
            pending_blocks: Arc::new(RwLock::new(VecDeque::new())),
            seen_blocks: Arc::new(RwLock::new(HashSet::new())),
            sync_state: Arc::new(RwLock::new(SyncState {
                syncing: false,
                current_height: 0,
                target_height: 0,
                processed_blocks: 0,
                failed_blocks: 0,
                pending_count: 0,
            })),
            failed_blocks: Arc::new(RwLock::new(VecDeque::new())),
        }
    }

    /// Start the sync process
    pub async fn start_sync(&self) -> Result<()> {
        info!("Starting iterative sync manager");

        // Update sync state
        {
            let mut state = self.sync_state.write().await;
            state.syncing = true;
            state.current_height = self.storage.blocks.get_latest_height().unwrap_or(0);
        }

        // Start sync loop
        let storage = self.storage.clone();
        let peer_manager = self.peer_manager.clone();
        let config = self.config.clone();
        let pending_blocks = self.pending_blocks.clone();
        let seen_blocks = self.seen_blocks.clone();
        let sync_state = self.sync_state.clone();
        let failed_blocks = self.failed_blocks.clone();

        tokio::spawn(async move {
            loop {
                if let Err(e) = Self::sync_iteration(
                    storage.clone(),
                    peer_manager.clone(),
                    &config,
                    pending_blocks.clone(),
                    seen_blocks.clone(),
                    sync_state.clone(),
                    failed_blocks.clone(),
                )
                .await
                {
                    warn!("Sync iteration failed: {}", e);
                }

                tokio::time::sleep(tokio::time::Duration::from_secs(config.sync_interval)).await;
            }
        });

        // Start block processor
        self.start_block_processor().await;

        Ok(())
    }

    /// Single sync iteration - request blocks from peers
    async fn sync_iteration(
        storage: Arc<StorageManager>,
        peer_manager: Arc<PeerManager>,
        config: &SyncConfig,
        pending_blocks: Arc<RwLock<VecDeque<BlockWithRetry>>>,
        _seen_blocks: Arc<RwLock<HashSet<Hash>>>,
        sync_state: Arc<RwLock<SyncState>>,
        failed_blocks: Arc<RwLock<VecDeque<BlockWithRetry>>>,
    ) -> Result<()> {
        let current_height = storage.blocks.get_latest_height().unwrap_or(0);

        // Update sync state
        {
            let mut state = sync_state.write().await;
            state.current_height = current_height;
            state.pending_count = pending_blocks.read().await.len();
        }

        // Check if we need more blocks
        let pending_count = pending_blocks.read().await.len();
        if pending_count >= config.max_queue_size {
            debug!(
                "Pending queue full ({}/{}), skipping sync request",
                pending_count, config.max_queue_size
            );
            return Ok(());
        }

        // Determine what blocks to request
        let (from_hash, request_count) = if current_height == 0 {
            // No blocks yet, request from genesis
            info!("Requesting blocks from genesis");
            (Hash::new([0u8; 32]), config.batch_size as u32)
        } else {
            // Request blocks after our current height
            let current_hash = storage
                .blocks
                .get_block_by_height(current_height)?
                .ok_or_else(|| anyhow::anyhow!("Current block not found"))?;

            let request_count =
                (config.max_queue_size - pending_count).min(config.batch_size) as u32;

            info!(
                "Requesting {} blocks after height {}",
                request_count, current_height
            );
            (current_hash, request_count)
        };

        // Send request to peers
        let step = if config.sparse_sync && current_height < 1000 {
            config.sparse_step as u32
        } else {
            1
        };

        peer_manager
            .broadcast(&NetworkMessage::GetBlocks {
                from: from_hash,
                count: request_count,
                step,
            })
            .await?;

        // Retry failed blocks periodically
        Self::retry_failed_blocks(failed_blocks, pending_blocks, config.max_retries).await;

        Ok(())
    }

    /// Process blocks from the pending queue iteratively
    async fn start_block_processor(&self) {
        info!("Starting block processor");

        let storage = self.storage.clone();
        let pending_blocks = self.pending_blocks.clone();
        let seen_blocks = self.seen_blocks.clone();
        let sync_state = self.sync_state.clone();
        let failed_blocks = self.failed_blocks.clone();
        let max_retries = self.config.max_retries;

        tokio::spawn(async move {
            loop {
                // Process one block at a time to avoid stack issues
                let block_to_process = {
                    let mut pending = pending_blocks.write().await;
                    pending.pop_front()
                };

                if let Some(block_with_retry) = block_to_process {
                    match Self::process_block_iterative(&storage, block_with_retry.block.clone())
                        .await
                    {
                        Ok(processed) => {
                            if processed {
                                let mut state = sync_state.write().await;
                                state.processed_blocks += 1;
                                debug!(
                                    "Processed block at height {}",
                                    block_with_retry.block.header.height
                                );
                            } else {
                                // Block was skipped (already exists)
                                debug!("Skipped existing block");
                            }
                        }
                        Err(e) => {
                            warn!("Failed to process block: {}", e);

                            // Add to failed queue for retry
                            if block_with_retry.retries < max_retries {
                                let mut failed = failed_blocks.write().await;
                                failed.push_back(BlockWithRetry {
                                    block: block_with_retry.block,
                                    retries: block_with_retry.retries + 1,
                                });
                            } else {
                                let mut state = sync_state.write().await;
                                state.failed_blocks += 1;
                            }
                        }
                    }
                } else {
                    // No blocks to process, wait a bit
                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                }

                // Clean up seen blocks periodically
                if seen_blocks.read().await.len() > 10000 {
                    let mut seen = seen_blocks.write().await;
                    seen.clear();
                    debug!("Cleared seen blocks cache");
                }
            }
        });
    }

    /// Process a single block iteratively (no recursion)
    async fn process_block_iterative(storage: &Arc<StorageManager>, block: Block) -> Result<bool> {
        // Check if block already exists
        if storage.blocks.has_block(&block.header.block_hash)? {
            return Ok(false);
        }

        // Validate block basics
        if !Self::validate_block(&block) {
            return Err(anyhow::anyhow!("Block validation failed"));
        }

        // Check parents exist (but don't recurse into them)
        let parents_exist = Self::check_parents_exist(storage, &block).await?;
        if !parents_exist {
            return Err(anyhow::anyhow!("Missing parent blocks"));
        }

        // Calculate blue score using iterative BFS
        let _blue_score = Self::calculate_blue_score_iterative(storage, &block).await?;

        // Store the block
        storage.blocks.put_block(&block)?;

        Ok(true)
    }

    /// Validate block without processing parents
    fn validate_block(_block: &Block) -> bool {
        // Basic validation
        // Genesis block has height 0, others must have positive height
        true // Simplified validation for now
    }

    /// Check if parent blocks exist
    async fn check_parents_exist(storage: &Arc<StorageManager>, block: &Block) -> Result<bool> {
        if block.header.height > 0 {
            // For non-genesis blocks, check if parent exists
            // Note: The block header structure doesn't have explicit parent fields
            // We'll check by height
            let parent_height = block.header.height - 1;
            if let Ok(None) = storage.blocks.get_block_by_height(parent_height) {
                return Ok(false);
            }
        }

        Ok(true)
    }

    /// Calculate blue score iteratively using BFS
    async fn calculate_blue_score_iterative(
        storage: &Arc<StorageManager>,
        block: &Block,
    ) -> Result<u64> {
        let mut blue_score = 0u64;
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        const MAX_DEPTH: usize = 100; // Limit depth to avoid issues

        // Start with parent blocks (by height)
        if block.header.height > 0 {
            // Get parent by height
            if let Ok(Some(parent_hash)) =
                storage.blocks.get_block_by_height(block.header.height - 1)
            {
                queue.push_back((parent_hash, 0));
            }
        }

        while let Some((hash, depth)) = queue.pop_front() {
            if depth >= MAX_DEPTH || visited.contains(&hash) {
                continue;
            }
            visited.insert(hash);

            if let Ok(Some(ancestor)) = storage.blocks.get_block(&hash) {
                // Increment blue score (simplified - all ancestors count)
                // In actual GhostDAG, this would check blue set membership
                blue_score += 1;

                // Add parent to queue (limited depth)
                if ancestor.header.height > 0 && depth + 1 < MAX_DEPTH {
                    if let Ok(Some(parent_hash)) = storage
                        .blocks
                        .get_block_by_height(ancestor.header.height - 1)
                    {
                        queue.push_back((parent_hash, depth + 1));
                    }
                }
            }
        }

        Ok(blue_score)
    }

    /// Retry failed blocks
    async fn retry_failed_blocks(
        failed_blocks: Arc<RwLock<VecDeque<BlockWithRetry>>>,
        pending_blocks: Arc<RwLock<VecDeque<BlockWithRetry>>>,
        max_retries: usize,
    ) {
        let mut failed = failed_blocks.write().await;
        let mut pending = pending_blocks.write().await;

        // Move some failed blocks back to pending for retry
        let retry_count = failed.len().min(5); // Retry up to 5 blocks at a time
        for _ in 0..retry_count {
            if let Some(block) = failed.pop_front() {
                if block.retries < max_retries {
                    debug!("Retrying block (attempt {})", block.retries + 1);
                    pending.push_back(block);
                }
            }
        }
    }

    /// Handle incoming blocks from network
    pub async fn handle_blocks(&self, blocks: Vec<Block>) -> Result<()> {
        let mut pending = self.pending_blocks.write().await;
        let mut seen = self.seen_blocks.write().await;

        for block in blocks {
            if !seen.contains(&block.header.block_hash) {
                seen.insert(block.header.block_hash);
                pending.push_back(BlockWithRetry { block, retries: 0 });

                // Limit queue size
                if pending.len() > self.config.max_queue_size * 2 {
                    warn!("Pending queue overflow, dropping oldest blocks");
                    pending.drain(0..self.config.max_queue_size);
                }
            }
        }

        Ok(())
    }

    /// Get current sync state
    #[allow(dead_code)]
    pub async fn get_sync_state(&self) -> SyncState {
        self.sync_state.read().await.clone()
    }

    /// Stop syncing
    #[allow(dead_code)]
    pub async fn stop_sync(&self) {
        let mut state = self.sync_state.write().await;
        state.syncing = false;
        info!("Sync stopped");
    }
}
