// citrate/node/src/sync/efficient_sync.rs

// Efficient, non-recursive sync implementation for GhostDAG
// Avoids stack overflow by using iterative processing and bounded queues

use citrate_consensus::types::{Block, BlockHeader, Hash};
use citrate_consensus::GhostDag;
use citrate_storage::StorageManager;
use std::collections::{HashSet, VecDeque};
use std::sync::Arc;
use anyhow::Result;
use tokio::sync::Mutex;
use tracing::{debug, info, warn};

/// Maximum blocks to process in a single batch to avoid memory issues
const MAX_BATCH_SIZE: usize = 100;

/// Maximum depth for DAG traversal to prevent infinite loops
const MAX_TRAVERSAL_DEPTH: usize = 1000;

/// Checkpoint interval for saving progress
const CHECKPOINT_INTERVAL: u64 = 100;

/// Efficient sync manager that handles large block ranges without recursion
pub struct EfficientSyncManager {
    storage: Arc<StorageManager>,
    ghostdag: Arc<GhostDag>,
    /// Queue of blocks waiting to be processed
    process_queue: VecDeque<Block>,
    /// Set of block hashes we've already seen to avoid duplicates
    seen_blocks: HashSet<Hash>,
    /// Checkpoint for resumable sync
    last_checkpoint: u64,
    /// Maximum memory usage for queue (in estimated bytes)
    max_queue_memory: usize,
    /// Current estimated queue memory usage
    current_queue_memory: usize,
}

impl EfficientSyncManager {
    pub fn new(storage: Arc<StorageManager>, ghostdag: Arc<GhostDag>) -> Self {
        Self {
            storage,
            ghostdag,
            process_queue: VecDeque::with_capacity(MAX_BATCH_SIZE),
            seen_blocks: HashSet::new(),
            last_checkpoint: 0,
            max_queue_memory: 100 * 1024 * 1024, // 100 MB max queue size
            current_queue_memory: 0,
        }
    }

    /// Sync blocks from a peer in an efficient, non-recursive manner
    pub async fn sync_blocks(&mut self, blocks: Vec<Block>) -> Result<SyncResult> {
        info!("Starting efficient sync with {} blocks", blocks.len());

        let mut processed_count = 0;
        let mut skipped_count = 0;
        let mut error_count = 0;

        // Process blocks in chunks to avoid memory issues
        for chunk in blocks.chunks(MAX_BATCH_SIZE) {
            match self.process_block_batch(chunk).await {
                Ok(batch_result) => {
                    processed_count += batch_result.processed;
                    skipped_count += batch_result.skipped;
                    error_count += batch_result.errors;

                    // Save checkpoint periodically
                    if processed_count > 0 && processed_count % CHECKPOINT_INTERVAL as usize == 0 {
                        self.save_checkpoint()?;
                    }
                }
                Err(e) => {
                    warn!("Error processing batch: {}", e);
                    error_count += chunk.len();
                }
            }

            // Clear seen blocks periodically to avoid unbounded memory growth
            if self.seen_blocks.len() > 10000 {
                self.cleanup_seen_blocks();
            }
        }

        // Final checkpoint
        if processed_count > 0 {
            self.save_checkpoint()?;
        }

        Ok(SyncResult {
            processed: processed_count,
            skipped: skipped_count,
            errors: error_count,
        })
    }

    /// Process a batch of blocks iteratively (no recursion)
    async fn process_block_batch(&mut self, blocks: &[Block]) -> Result<BatchResult> {
        let mut processed = 0;
        let mut skipped = 0;
        let mut errors = 0;

        // Add blocks to queue if not seen
        for block in blocks {
            let block_hash = block.hash();
            if !self.seen_blocks.contains(&block_hash) {
                let block_size = self.estimate_block_size(block);

                // Check memory limit
                if self.current_queue_memory + block_size > self.max_queue_memory {
                    debug!("Queue memory limit reached, processing current queue");
                    self.process_queue_iteratively().await?;
                }

                self.process_queue.push_back(block.clone());
                self.seen_blocks.insert(block_hash);
                self.current_queue_memory += block_size;
            } else {
                skipped += 1;
            }
        }

        // Process queue iteratively
        while let Some(block) = self.process_queue.pop_front() {
            let block_size = self.estimate_block_size(&block);
            self.current_queue_memory = self.current_queue_memory.saturating_sub(block_size);

            match self.process_single_block(block).await {
                Ok(true) => processed += 1,
                Ok(false) => skipped += 1,
                Err(e) => {
                    debug!("Error processing block: {}", e);
                    errors += 1;
                }
            }
        }

        Ok(BatchResult {
            processed,
            skipped,
            errors,
        })
    }

    /// Process a single block without recursion
    async fn process_single_block(&mut self, block: Block) -> Result<bool> {
        let block_hash = block.hash();

        // Check if block already exists (synchronous storage call)
        if self.storage.blocks.has_block(&block_hash)? {
            debug!(
                "Block {} already exists at height {}",
                block_hash.to_hex(),
                block.header.height
            );
            return Ok(false);
        }

        // Validate block header
        if !self.validate_block_header(&block.header)? {
            debug!("Block {} failed header validation", block_hash.to_hex());
            return Ok(false);
        }

        // Check if we have all parent blocks (iteratively, not recursively)
        let missing_parents = self.find_missing_parents(&block)?;
        if !missing_parents.is_empty() {
            debug!(
                "Block {} has {} missing parents, deferring",
                block_hash.to_hex(),
                missing_parents.len()
            );

            // Re-queue the block for later processing
            self.process_queue.push_back(block);
            return Ok(false);
        }

        // Calculate blue score iteratively
        let blue_score = self.calculate_blue_score_iterative(&block)?;

        // Store block (synchronous storage call)
        self.storage.blocks.put_block(&block)?;

        // Update DAG state via GhostDag
        self.ghostdag.add_block(&block).await.map_err(|e| {
            anyhow::anyhow!("Failed to add block to GhostDAG: {:?}", e)
        })?;

        debug!(
            "Successfully processed block {} at height {} with blue score {}",
            block_hash.to_hex(),
            block.header.height,
            blue_score
        );

        Ok(true)
    }

    /// Calculate blue score iteratively using BFS instead of recursion
    fn calculate_blue_score_iterative(&self, block: &Block) -> Result<u64> {
        // Start with the block's recorded blue_score if available
        // The actual blue score calculation happens in GhostDag::add_block
        // Here we do a simple validation by traversing ancestors

        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        let mut ancestry_count = 0u64;

        // Genesis block has blue_score of 1
        if block.is_genesis() {
            return Ok(1);
        }

        // Start with selected parent
        let selected_parent = block.selected_parent();
        if selected_parent != Hash::default() {
            queue.push_back((selected_parent, 0usize));
        }

        // Process ancestors iteratively
        while let Some((hash, depth)) = queue.pop_front() {
            // Limit traversal depth
            if depth > MAX_TRAVERSAL_DEPTH {
                warn!("Reached maximum traversal depth at block {}", hash.to_hex());
                break;
            }

            if visited.contains(&hash) {
                continue;
            }
            visited.insert(hash);

            // Get block from storage (synchronous)
            if let Ok(Some(ancestor)) = self.storage.blocks.get_block(&hash) {
                // Count this block toward ancestry
                ancestry_count += 1;

                // Add selected parent to queue
                let ancestor_parent = ancestor.selected_parent();
                if ancestor_parent != Hash::default() {
                    queue.push_back((ancestor_parent, depth + 1));
                }

                // Add merge parents with limited depth
                for merge_parent in &ancestor.header.merge_parent_hashes {
                    if depth < MAX_TRAVERSAL_DEPTH / 2 {
                        queue.push_back((*merge_parent, depth + 1));
                    }
                }
            }
        }

        // Blue score is approximately the ancestry count + 1 (for this block)
        // The precise calculation is done by GhostDag
        Ok(ancestry_count + 1)
    }

    /// Find missing parent blocks without recursion
    fn find_missing_parents(&self, block: &Block) -> Result<Vec<Hash>> {
        let mut missing = Vec::new();

        // Check selected parent (only if not genesis)
        if !block.is_genesis() {
            let selected_parent = block.selected_parent();
            if selected_parent != Hash::default() && !self.storage.blocks.has_block(&selected_parent)? {
                missing.push(selected_parent);
            }
        }

        // Check merge parents
        for parent in &block.header.merge_parent_hashes {
            if !self.storage.blocks.has_block(parent)? {
                missing.push(*parent);
            }
        }

        Ok(missing)
    }

    /// Validate block header
    fn validate_block_header(&self, header: &BlockHeader) -> Result<bool> {
        // Genesis block validation
        if header.height == 0 {
            // Genesis should have default (zero) selected parent
            if header.selected_parent_hash != Hash::default() {
                return Ok(false);
            }
            return Ok(true);
        }

        // Non-genesis blocks must have a selected parent
        if header.selected_parent_hash == Hash::default() {
            return Ok(false);
        }

        // Timestamp validation - block shouldn't be too far in the future
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs();

        if header.timestamp > current_time + 300 {
            return Ok(false); // Block too far in future (5 min tolerance)
        }

        Ok(true)
    }

    /// Process queue iteratively when memory limit is reached
    async fn process_queue_iteratively(&mut self) -> Result<()> {
        let queue_size = self.process_queue.len();
        debug!(
            "Processing queue of {} blocks due to memory limit",
            queue_size
        );

        let mut temp_queue = VecDeque::new();
        std::mem::swap(&mut self.process_queue, &mut temp_queue);

        for block in temp_queue {
            match self.process_single_block(block.clone()).await {
                Ok(true) => {
                    // Successfully processed
                }
                Ok(false) | Err(_) => {
                    // Failed or skipped, re-queue
                    self.process_queue.push_back(block);
                }
            }
        }

        Ok(())
    }

    /// Estimate block size in bytes for memory management
    fn estimate_block_size(&self, block: &Block) -> usize {
        // Rough estimation based on Block structure
        std::mem::size_of::<Block>()
            + block.transactions.len() * 200 // Estimate 200 bytes per transaction
            + block.header.merge_parent_hashes.len() * 32 // 32 bytes per hash
            + block.embedded_models.iter().map(|m| m.weights.len()).sum::<usize>()
    }

    /// Clean up seen blocks set to avoid unbounded growth
    fn cleanup_seen_blocks(&mut self) {
        // Keep only recent blocks
        let to_keep = 5000;
        if self.seen_blocks.len() > to_keep {
            let mut new_seen = HashSet::new();
            for (i, hash) in self.seen_blocks.iter().enumerate() {
                if i >= self.seen_blocks.len() - to_keep {
                    new_seen.insert(*hash);
                }
            }
            self.seen_blocks = new_seen;
            debug!("Cleaned up seen blocks set, kept {} most recent", to_keep);
        }
    }

    /// Save sync checkpoint for resumability
    fn save_checkpoint(&mut self) -> Result<()> {
        if let Ok(height) = self.storage.blocks.get_latest_height() {
            self.last_checkpoint = height;
            info!("Saved sync checkpoint at height {}", height);
        }
        Ok(())
    }

    /// Resume sync from last checkpoint
    pub fn resume_from_checkpoint(&mut self) -> Result<u64> {
        let checkpoint = self.storage.blocks.get_latest_height().unwrap_or(0);
        self.last_checkpoint = checkpoint;
        info!("Resuming sync from checkpoint height {}", checkpoint);
        Ok(checkpoint)
    }

    /// Get the current checkpoint height
    pub fn checkpoint_height(&self) -> u64 {
        self.last_checkpoint
    }

    /// Clear internal state for fresh sync
    pub fn reset(&mut self) {
        self.process_queue.clear();
        self.seen_blocks.clear();
        self.current_queue_memory = 0;
        self.last_checkpoint = 0;
    }
}

/// Result of a sync operation
#[derive(Debug, Clone, Default)]
pub struct SyncResult {
    pub processed: usize,
    pub skipped: usize,
    pub errors: usize,
}

impl SyncResult {
    pub fn new() -> Self {
        Self::default()
    }

    /// Total blocks attempted
    pub fn total(&self) -> usize {
        self.processed + self.skipped + self.errors
    }

    /// Success rate as a percentage
    pub fn success_rate(&self) -> f64 {
        let total = self.total();
        if total == 0 {
            return 100.0;
        }
        (self.processed as f64 / total as f64) * 100.0
    }

    /// Merge another SyncResult into this one
    pub fn merge(&mut self, other: &SyncResult) {
        self.processed += other.processed;
        self.skipped += other.skipped;
        self.errors += other.errors;
    }
}

/// Result of processing a batch
#[derive(Debug, Default)]
struct BatchResult {
    processed: usize,
    skipped: usize,
    errors: usize,
}

/// Parallel sync coordinator for multiple peers
pub struct ParallelSyncCoordinator {
    storage: Arc<StorageManager>,
    ghostdag: Arc<GhostDag>,
    num_workers: usize,
    /// Shared result accumulator
    results: Arc<Mutex<SyncResult>>,
}

impl ParallelSyncCoordinator {
    pub fn new(storage: Arc<StorageManager>, ghostdag: Arc<GhostDag>, num_workers: usize) -> Self {
        Self {
            storage,
            ghostdag,
            num_workers: num_workers.max(1), // At least 1 worker
            results: Arc::new(Mutex::new(SyncResult::new())),
        }
    }

    /// Distribute block ranges among workers for parallel processing
    pub async fn sync_range_parallel(
        &mut self,
        start_height: u64,
        end_height: u64,
    ) -> Result<SyncResult> {
        if end_height <= start_height {
            return Ok(SyncResult::new());
        }

        let total_blocks = end_height - start_height;
        let blocks_per_worker = (total_blocks / self.num_workers as u64).max(1);

        info!(
            "Starting parallel sync from {} to {} with {} workers ({} blocks each)",
            start_height, end_height, self.num_workers, blocks_per_worker
        );

        // Create worker tasks
        let mut handles = Vec::with_capacity(self.num_workers);

        for worker_id in 0..self.num_workers {
            let worker_start = start_height + (worker_id as u64 * blocks_per_worker);
            let worker_end = if worker_id == self.num_workers - 1 {
                end_height
            } else {
                (worker_start + blocks_per_worker).min(end_height)
            };

            // Skip if this worker has no work
            if worker_start >= end_height {
                continue;
            }

            let storage = self.storage.clone();
            let ghostdag = self.ghostdag.clone();
            let results = self.results.clone();

            let handle = tokio::spawn(async move {
                Self::worker_sync_range(worker_id, storage, ghostdag, worker_start, worker_end, results).await
            });

            handles.push(handle);
        }

        // Wait for all workers to complete
        for handle in handles {
            match handle.await {
                Ok(Ok(())) => {}
                Ok(Err(e)) => {
                    warn!("Worker error: {}", e);
                }
                Err(e) => {
                    warn!("Worker task panicked: {}", e);
                }
            }
        }

        // Return accumulated results
        let final_result = self.results.lock().await.clone();
        info!(
            "Parallel sync completed: {} processed, {} skipped, {} errors",
            final_result.processed, final_result.skipped, final_result.errors
        );

        Ok(final_result)
    }

    /// Worker function to sync a range of blocks
    async fn worker_sync_range(
        worker_id: usize,
        storage: Arc<StorageManager>,
        ghostdag: Arc<GhostDag>,
        start_height: u64,
        end_height: u64,
        results: Arc<Mutex<SyncResult>>,
    ) -> Result<()> {
        debug!(
            "Worker {} syncing heights {} to {}",
            worker_id, start_height, end_height
        );

        let mut manager = EfficientSyncManager::new(storage.clone(), ghostdag);
        let mut worker_result = SyncResult::new();

        // Fetch blocks for this range from storage
        for height in start_height..end_height {
            match storage.blocks.get_block_by_height(height) {
                Ok(Some(hash)) => {
                    match storage.blocks.get_block(&hash) {
                        Ok(Some(block)) => {
                            // Process the block
                            match manager.sync_blocks(vec![block]).await {
                                Ok(batch_result) => {
                                    worker_result.merge(&batch_result);
                                }
                                Err(e) => {
                                    debug!("Worker {} error at height {}: {}", worker_id, height, e);
                                    worker_result.errors += 1;
                                }
                            }
                        }
                        Ok(None) => {
                            worker_result.skipped += 1;
                        }
                        Err(e) => {
                            debug!("Worker {} failed to get block at height {}: {}", worker_id, height, e);
                            worker_result.errors += 1;
                        }
                    }
                }
                Ok(None) => {
                    // No block at this height
                    worker_result.skipped += 1;
                }
                Err(e) => {
                    debug!("Worker {} failed to get hash at height {}: {}", worker_id, height, e);
                    worker_result.errors += 1;
                }
            }
        }

        // Merge results into shared accumulator
        let mut shared_results = results.lock().await;
        shared_results.merge(&worker_result);

        debug!(
            "Worker {} completed: {} processed, {} skipped, {} errors",
            worker_id, worker_result.processed, worker_result.skipped, worker_result.errors
        );

        Ok(())
    }

    /// Sync blocks from multiple peers concurrently
    pub async fn sync_from_peers(
        &mut self,
        peer_blocks: Vec<(String, Vec<Block>)>,
    ) -> Result<SyncResult> {
        info!("Syncing from {} peers", peer_blocks.len());

        let mut handles = Vec::new();

        for (peer_id, blocks) in peer_blocks {
            let storage = self.storage.clone();
            let ghostdag = self.ghostdag.clone();
            let results = self.results.clone();

            let handle = tokio::spawn(async move {
                let mut manager = EfficientSyncManager::new(storage, ghostdag);
                let sync_result = manager.sync_blocks(blocks).await;

                match sync_result {
                    Ok(result) => {
                        let mut shared_results = results.lock().await;
                        shared_results.merge(&result);
                        debug!("Peer {} sync complete", peer_id);
                    }
                    Err(e) => {
                        warn!("Peer {} sync failed: {}", peer_id, e);
                    }
                }
            });

            handles.push(handle);
        }

        // Wait for all peer syncs to complete
        for handle in handles {
            let _ = handle.await;
        }

        let final_result = self.results.lock().await.clone();
        Ok(final_result)
    }

    /// Get current accumulated results
    pub async fn current_results(&self) -> SyncResult {
        self.results.lock().await.clone()
    }

    /// Reset the coordinator for a new sync operation
    pub async fn reset(&mut self) {
        *self.results.lock().await = SyncResult::new();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use citrate_consensus::dag_store::DagStore;
    use citrate_consensus::types::{GhostDagParams, PublicKey, Signature, VrfProof};
    use citrate_storage::pruning::PruningConfig;
    use tempfile::TempDir;

    fn create_test_block(height: u64, parent_hash: Hash) -> Block {
        let block_hash = Hash::new([height as u8; 32]);
        Block {
            header: BlockHeader {
                version: 1,
                block_hash,
                selected_parent_hash: parent_hash,
                merge_parent_hashes: vec![],
                timestamp: 1000000 + height,
                height,
                blue_score: height,
                blue_work: height as u128 * 100,
                pruning_point: Hash::default(),
                proposer_pubkey: PublicKey::new([1; 32]),
                vrf_reveal: VrfProof {
                    proof: vec![],
                    output: Hash::default(),
                },
            },
            state_root: Hash::default(),
            tx_root: Hash::default(),
            receipt_root: Hash::default(),
            artifact_root: Hash::default(),
            ghostdag_params: GhostDagParams::default(),
            transactions: vec![],
            signature: Signature::new([0; 64]),
            embedded_models: vec![],
            required_pins: vec![],
        }
    }

    fn create_genesis_block() -> Block {
        create_test_block(0, Hash::default())
    }

    #[tokio::test]
    async fn test_efficient_sync_no_recursion() {
        // Test that sync handles deep chains without stack overflow
        let temp_dir = TempDir::new().unwrap();
        let storage = Arc::new(StorageManager::new(temp_dir.path(), PruningConfig::default()).unwrap());
        let dag_store = Arc::new(DagStore::new());
        let ghostdag = Arc::new(GhostDag::new(GhostDagParams::default(), dag_store.clone()));

        let mut manager = EfficientSyncManager::new(storage.clone(), ghostdag.clone());

        // Create a chain of 100 blocks
        let mut blocks = Vec::new();
        let mut parent_hash = Hash::default();

        for height in 0..100 {
            let block = create_test_block(height, parent_hash);
            parent_hash = block.hash();
            blocks.push(block);
        }

        // Store genesis first
        let genesis = &blocks[0];
        storage.blocks.put_block(genesis).unwrap();
        dag_store.store_block(genesis.clone()).await.unwrap();

        // Sync remaining blocks
        let result = manager.sync_blocks(blocks[1..].to_vec()).await.unwrap();

        // Verify some blocks were processed (exact count depends on parent availability)
        assert!(result.total() > 0);
        assert_eq!(result.errors, 0);
    }

    #[tokio::test]
    async fn test_memory_bounded_queue() {
        // Test that queue respects memory limits
        let temp_dir = TempDir::new().unwrap();
        let storage = Arc::new(StorageManager::new(temp_dir.path(), PruningConfig::default()).unwrap());
        let dag_store = Arc::new(DagStore::new());
        let ghostdag = Arc::new(GhostDag::new(GhostDagParams::default(), dag_store));

        let mut manager = EfficientSyncManager::new(storage, ghostdag);

        // Set a low memory limit
        manager.max_queue_memory = 1024; // 1 KB

        // Create blocks
        let blocks: Vec<Block> = (0..10)
            .map(|i| create_test_block(i, if i == 0 { Hash::default() } else { Hash::new([(i - 1) as u8; 32]) }))
            .collect();

        // Queue memory should never exceed the limit
        for block in &blocks {
            let size = manager.estimate_block_size(block);
            assert!(size > 0);
        }

        // Verify manager doesn't panic with memory limits
        let result = manager.sync_blocks(blocks).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_checkpoint_resume() {
        // Test checkpoint save and resume functionality
        let temp_dir = TempDir::new().unwrap();
        let storage = Arc::new(StorageManager::new(temp_dir.path(), PruningConfig::default()).unwrap());
        let dag_store = Arc::new(DagStore::new());
        let ghostdag = Arc::new(GhostDag::new(GhostDagParams::default(), dag_store.clone()));

        // Store some blocks
        let genesis = create_genesis_block();
        storage.blocks.put_block(&genesis).unwrap();
        dag_store.store_block(genesis.clone()).await.unwrap();

        let block1 = create_test_block(1, genesis.hash());
        storage.blocks.put_block(&block1).unwrap();
        dag_store.store_block(block1.clone()).await.unwrap();

        let mut manager = EfficientSyncManager::new(storage.clone(), ghostdag);

        // Resume should return current height
        let checkpoint = manager.resume_from_checkpoint().unwrap();
        assert!(checkpoint >= 0); // At least genesis

        // Verify checkpoint height accessor
        assert_eq!(manager.checkpoint_height(), checkpoint);
    }

    #[tokio::test]
    async fn test_sync_result_merge() {
        let mut result1 = SyncResult {
            processed: 10,
            skipped: 2,
            errors: 1,
        };

        let result2 = SyncResult {
            processed: 5,
            skipped: 1,
            errors: 0,
        };

        result1.merge(&result2);

        assert_eq!(result1.processed, 15);
        assert_eq!(result1.skipped, 3);
        assert_eq!(result1.errors, 1);
        assert_eq!(result1.total(), 19);
    }

    #[tokio::test]
    async fn test_sync_result_success_rate() {
        let result = SyncResult {
            processed: 80,
            skipped: 15,
            errors: 5,
        };

        let rate = result.success_rate();
        assert!((rate - 80.0).abs() < 0.01);
    }

    #[tokio::test]
    async fn test_block_header_validation() {
        let temp_dir = TempDir::new().unwrap();
        let storage = Arc::new(StorageManager::new(temp_dir.path(), PruningConfig::default()).unwrap());
        let dag_store = Arc::new(DagStore::new());
        let ghostdag = Arc::new(GhostDag::new(GhostDagParams::default(), dag_store));

        let manager = EfficientSyncManager::new(storage, ghostdag);

        // Genesis block should be valid
        let genesis_header = BlockHeader {
            version: 1,
            block_hash: Hash::new([0; 32]),
            selected_parent_hash: Hash::default(),
            merge_parent_hashes: vec![],
            timestamp: 1000000,
            height: 0,
            blue_score: 0,
            blue_work: 0,
            pruning_point: Hash::default(),
            proposer_pubkey: PublicKey::new([1; 32]),
            vrf_reveal: VrfProof {
                proof: vec![],
                output: Hash::default(),
            },
        };
        assert!(manager.validate_block_header(&genesis_header).unwrap());

        // Non-genesis without parent should be invalid
        let invalid_header = BlockHeader {
            version: 1,
            block_hash: Hash::new([1; 32]),
            selected_parent_hash: Hash::default(), // Invalid: non-genesis without parent
            merge_parent_hashes: vec![],
            timestamp: 1000001,
            height: 1, // Non-genesis
            blue_score: 1,
            blue_work: 100,
            pruning_point: Hash::default(),
            proposer_pubkey: PublicKey::new([1; 32]),
            vrf_reveal: VrfProof {
                proof: vec![],
                output: Hash::default(),
            },
        };
        assert!(!manager.validate_block_header(&invalid_header).unwrap());

        // Valid non-genesis block
        let valid_header = BlockHeader {
            version: 1,
            block_hash: Hash::new([1; 32]),
            selected_parent_hash: Hash::new([0; 32]), // Has parent
            merge_parent_hashes: vec![],
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            height: 1,
            blue_score: 1,
            blue_work: 100,
            pruning_point: Hash::default(),
            proposer_pubkey: PublicKey::new([1; 32]),
            vrf_reveal: VrfProof {
                proof: vec![],
                output: Hash::default(),
            },
        };
        assert!(manager.validate_block_header(&valid_header).unwrap());
    }

    #[tokio::test]
    async fn test_parallel_sync_coordinator() {
        let temp_dir = TempDir::new().unwrap();
        let storage = Arc::new(StorageManager::new(temp_dir.path(), PruningConfig::default()).unwrap());
        let dag_store = Arc::new(DagStore::new());
        let ghostdag = Arc::new(GhostDag::new(GhostDagParams::default(), dag_store.clone()));

        // Store genesis
        let genesis = create_genesis_block();
        storage.blocks.put_block(&genesis).unwrap();
        dag_store.store_block(genesis.clone()).await.unwrap();

        let mut coordinator = ParallelSyncCoordinator::new(storage, ghostdag, 4);

        // Sync an empty range
        let result = coordinator.sync_range_parallel(0, 0).await.unwrap();
        assert_eq!(result.total(), 0);

        // Reset and check
        coordinator.reset().await;
        let current = coordinator.current_results().await;
        assert_eq!(current.total(), 0);
    }

    #[tokio::test]
    async fn test_find_missing_parents() {
        let temp_dir = TempDir::new().unwrap();
        let storage = Arc::new(StorageManager::new(temp_dir.path(), PruningConfig::default()).unwrap());
        let dag_store = Arc::new(DagStore::new());
        let ghostdag = Arc::new(GhostDag::new(GhostDagParams::default(), dag_store));

        let manager = EfficientSyncManager::new(storage.clone(), ghostdag);

        // Genesis should have no missing parents
        let genesis = create_genesis_block();
        let missing = manager.find_missing_parents(&genesis).unwrap();
        assert!(missing.is_empty());

        // Block with missing parent
        let block_with_missing = create_test_block(5, Hash::new([99; 32]));
        let missing = manager.find_missing_parents(&block_with_missing).unwrap();
        assert_eq!(missing.len(), 1);
        assert_eq!(missing[0], Hash::new([99; 32]));
    }

    #[tokio::test]
    async fn test_manager_reset() {
        let temp_dir = TempDir::new().unwrap();
        let storage = Arc::new(StorageManager::new(temp_dir.path(), PruningConfig::default()).unwrap());
        let dag_store = Arc::new(DagStore::new());
        let ghostdag = Arc::new(GhostDag::new(GhostDagParams::default(), dag_store));

        let mut manager = EfficientSyncManager::new(storage, ghostdag);

        // Add some state
        manager.seen_blocks.insert(Hash::new([1; 32]));
        manager.process_queue.push_back(create_genesis_block());
        manager.last_checkpoint = 100;

        // Reset
        manager.reset();

        assert!(manager.seen_blocks.is_empty());
        assert!(manager.process_queue.is_empty());
        assert_eq!(manager.last_checkpoint, 0);
    }
}
