// citrate/core/consensus/src/finality.rs

//! Finality Mechanism for GhostDAG
//!
//! This module implements a depth-based finality mechanism where blocks become
//! final after reaching a certain confirmation depth in the selected-parent chain.
//!
//! Key properties:
//! - Blocks at depth >= `confirmation_depth` from the tip are considered final
//! - Finalized blocks cannot be reorganized (reorg protection)
//! - Finality events are emitted when blocks become final
//! - All ancestors of a finalized block are also final
//!
//! The finality depth is configurable, with a default of 100 blocks which provides
//! strong probabilistic finality guarantees under GhostDAG consensus.

use crate::dag_store::DagStore;
use crate::types::Hash;
use std::sync::atomic::{AtomicU64, Ordering as AtomicOrdering};
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::{broadcast, RwLock};
use tracing::{debug, info, warn};

#[derive(Error, Debug, Clone)]
pub enum FinalityError {
    #[error("Block not found: {0}")]
    BlockNotFound(Hash),

    #[error("Cannot reorg past finalized block: {0}")]
    ReorgPastFinalized(Hash),

    #[error("Finality check failed: {0}")]
    CheckFailed(String),

    #[error("Invalid finality state")]
    InvalidState,
}

/// Finality configuration
#[derive(Debug, Clone)]
pub struct FinalityConfig {
    /// Number of blocks deep before a block is considered final
    /// Default: 100 blocks
    pub confirmation_depth: u64,

    /// Whether to emit finality events
    pub emit_events: bool,

    /// Maximum number of blocks to finalize in a single update
    /// Prevents long blocking during initial sync
    pub max_finalize_batch: usize,
}

impl Default for FinalityConfig {
    fn default() -> Self {
        Self {
            confirmation_depth: 100,
            emit_events: true,
            max_finalize_batch: 1000,
        }
    }
}

impl FinalityConfig {
    /// Create a config for testing with lower confirmation depth
    pub fn for_testing() -> Self {
        Self {
            confirmation_depth: 10,
            emit_events: true,
            max_finalize_batch: 100,
        }
    }
}

/// Event emitted when a block becomes finalized
#[derive(Debug, Clone)]
pub struct FinalityEvent {
    /// Hash of the newly finalized block
    pub block_hash: Hash,

    /// Height of the finalized block
    pub height: u64,

    /// The new finalized tip (highest finalized block)
    pub finalized_tip: Hash,

    /// Total number of finalized blocks
    pub total_finalized: u64,
}

/// Finality tracker for the blockchain
///
/// Tracks which blocks have been finalized and provides reorg protection.
pub struct FinalityTracker {
    /// Configuration
    config: FinalityConfig,

    /// DAG store for block lookups
    dag_store: Arc<DagStore>,

    /// Current finalized tip (highest finalized block in selected-parent chain)
    finalized_tip: RwLock<Option<Hash>>,

    /// Height of the finalized tip
    finalized_height: AtomicU64,

    /// Total count of finalized blocks
    finalized_count: AtomicU64,

    /// Channel for finality events
    event_sender: broadcast::Sender<FinalityEvent>,
}

impl FinalityTracker {
    /// Create a new finality tracker
    pub fn new(dag_store: Arc<DagStore>, config: FinalityConfig) -> Self {
        let (event_sender, _) = broadcast::channel(1024);

        Self {
            config,
            dag_store,
            finalized_tip: RwLock::new(None),
            finalized_height: AtomicU64::new(0),
            finalized_count: AtomicU64::new(0),
            event_sender,
        }
    }

    /// Create with default configuration
    pub fn with_defaults(dag_store: Arc<DagStore>) -> Self {
        Self::new(dag_store, FinalityConfig::default())
    }

    /// Get a receiver for finality events
    pub fn subscribe(&self) -> broadcast::Receiver<FinalityEvent> {
        self.event_sender.subscribe()
    }

    /// Get the current finalized tip
    pub async fn get_finalized_tip(&self) -> Option<Hash> {
        *self.finalized_tip.read().await
    }

    /// Get the height of the finalized tip
    pub fn get_finalized_height(&self) -> u64 {
        self.finalized_height.load(AtomicOrdering::SeqCst)
    }

    /// Get the total count of finalized blocks
    pub fn get_finalized_count(&self) -> u64 {
        self.finalized_count.load(AtomicOrdering::SeqCst)
    }

    /// Check if a specific block is finalized
    pub async fn is_finalized(&self, block_hash: &Hash) -> bool {
        self.dag_store.is_finalized(block_hash).await
    }

    /// Check if a block can be finalized based on current chain state
    pub async fn check_finality(&self, block_hash: &Hash) -> Result<bool, FinalityError> {
        // Get the block
        let block = self
            .dag_store
            .get_block(block_hash)
            .await
            .map_err(|_| FinalityError::BlockNotFound(*block_hash))?;

        let block_height = block.header.height;
        let finalized_height = self.finalized_height.load(AtomicOrdering::SeqCst);

        // Block is final if it's at or below the finalized height
        Ok(block_height <= finalized_height)
    }

    /// Update finality based on a new chain tip
    ///
    /// This should be called whenever the chain tip changes. It will:
    /// 1. Calculate the new finality boundary (tip height - confirmation_depth)
    /// 2. Walk back the selected-parent chain to find blocks to finalize
    /// 3. Mark those blocks as finalized in the DAG store
    /// 4. Emit finality events
    pub async fn update_finality(
        &self,
        current_tip: &Hash,
        current_height: u64,
    ) -> Result<Vec<Hash>, FinalityError> {
        // Calculate the finality boundary
        if current_height < self.config.confirmation_depth {
            debug!(
                "Chain height {} is below finality depth {}, no blocks to finalize",
                current_height, self.config.confirmation_depth
            );
            return Ok(vec![]);
        }

        let target_final_height = current_height - self.config.confirmation_depth;
        let current_final_height = self.finalized_height.load(AtomicOrdering::SeqCst);

        // Check if we need to finalize any blocks
        if target_final_height <= current_final_height {
            return Ok(vec![]);
        }

        info!(
            "Updating finality: current_height={}, target_final_height={}, current_final_height={}",
            current_height, target_final_height, current_final_height
        );

        // Walk back the selected-parent chain from the tip to find blocks to finalize
        let blocks_to_finalize = self
            .find_blocks_to_finalize(current_tip, target_final_height, current_final_height)
            .await?;

        if blocks_to_finalize.is_empty() {
            return Ok(vec![]);
        }

        // Finalize the blocks (in order from oldest to newest)
        let mut newly_finalized = Vec::new();
        for (hash, height) in &blocks_to_finalize {
            // Mark as finalized in DAG store
            if let Err(e) = self.dag_store.finalize_block(hash).await {
                warn!("Failed to finalize block {}: {:?}", hash, e);
                continue;
            }

            newly_finalized.push(*hash);

            // Update finalized height
            self.finalized_height
                .fetch_max(*height, AtomicOrdering::SeqCst);
            self.finalized_count
                .fetch_add(1, AtomicOrdering::SeqCst);

            // Emit event
            if self.config.emit_events {
                let event = FinalityEvent {
                    block_hash: *hash,
                    height: *height,
                    finalized_tip: *hash,
                    total_finalized: self.finalized_count.load(AtomicOrdering::SeqCst),
                };

                // Ignore send errors (no receivers)
                let _ = self.event_sender.send(event);
            }

            debug!("Finalized block {} at height {}", hash, height);
        }

        // Update finalized tip
        if let Some((tip_hash, _)) = blocks_to_finalize.last() {
            *self.finalized_tip.write().await = Some(*tip_hash);
        }

        info!(
            "Finalized {} blocks, new finalized height: {}",
            newly_finalized.len(),
            self.finalized_height.load(AtomicOrdering::SeqCst)
        );

        Ok(newly_finalized)
    }

    /// Find blocks that need to be finalized
    async fn find_blocks_to_finalize(
        &self,
        tip: &Hash,
        target_height: u64,
        current_final_height: u64,
    ) -> Result<Vec<(Hash, u64)>, FinalityError> {
        let mut blocks = Vec::new();
        let mut current = *tip;
        let mut count = 0;

        // Walk back the selected-parent chain
        loop {
            if count >= self.config.max_finalize_batch {
                debug!(
                    "Reached max finalize batch size {}, stopping",
                    self.config.max_finalize_batch
                );
                break;
            }

            let block = self
                .dag_store
                .get_block(&current)
                .await
                .map_err(|_| FinalityError::BlockNotFound(current))?;

            let height = block.header.height;

            // Stop if we've gone past the target height
            if height > target_height {
                if block.is_genesis() {
                    break;
                }
                current = block.selected_parent();
                continue;
            }

            // Stop if we've reached already-finalized blocks
            // Note: current_final_height of 0 means nothing is finalized yet,
            // so we should still process height 0 (genesis)
            if current_final_height > 0 && height <= current_final_height {
                break;
            }

            // Skip if already finalized
            if !self.dag_store.is_finalized(&current).await {
                blocks.push((current, height));
                count += 1;
            }

            if block.is_genesis() {
                break;
            }

            current = block.selected_parent();
        }

        // Reverse to get oldest-first order
        blocks.reverse();

        Ok(blocks)
    }

    /// Check if a reorg would violate finality
    ///
    /// Returns an error if the proposed reorg would reorganize past a finalized block.
    pub async fn check_reorg_allowed(
        &self,
        common_ancestor: &Hash,
    ) -> Result<(), FinalityError> {
        // If there's no finalized tip yet, reorg is allowed
        let finalized_tip = match self.get_finalized_tip().await {
            Some(tip) => tip,
            None => return Ok(()),
        };

        // Check if common ancestor is at or below finalized tip
        // A reorg is only allowed if common ancestor is at or after finalized tip
        let finalized_height = self.finalized_height.load(AtomicOrdering::SeqCst);

        // Get common ancestor height
        let ancestor_block = self
            .dag_store
            .get_block(common_ancestor)
            .await
            .map_err(|_| FinalityError::BlockNotFound(*common_ancestor))?;

        let ancestor_height = ancestor_block.header.height;

        // If common ancestor is below finalized height, this reorg would
        // reorganize finalized blocks - not allowed
        if ancestor_height < finalized_height {
            warn!(
                "Reorg rejected: common ancestor height {} is below finalized height {}",
                ancestor_height, finalized_height
            );
            return Err(FinalityError::ReorgPastFinalized(finalized_tip));
        }

        // Check if the common ancestor itself is finalized
        if self.dag_store.is_finalized(common_ancestor).await {
            // This is fine - we're branching from a finalized point
            // but not reorganizing past it
            return Ok(());
        }

        Ok(())
    }

    /// Get finality status for a block
    pub async fn get_finality_status(&self, block_hash: &Hash) -> Result<FinalityStatus, FinalityError> {
        let block = self
            .dag_store
            .get_block(block_hash)
            .await
            .map_err(|_| FinalityError::BlockNotFound(*block_hash))?;

        let block_height = block.header.height;
        let finalized_height = self.finalized_height.load(AtomicOrdering::SeqCst);

        if self.dag_store.is_finalized(block_hash).await {
            return Ok(FinalityStatus::Finalized);
        }

        if block_height <= finalized_height {
            // Should be finalized but isn't marked yet
            // This can happen for blocks not on the selected-parent chain
            return Ok(FinalityStatus::PendingFinalization);
        }

        // Calculate confirmations
        // We need to know the current tip height for this
        let confirmations = if finalized_height > 0 {
            // Estimate based on finalized height + confirmation depth
            let estimated_tip = finalized_height + self.config.confirmation_depth;
            if estimated_tip > block_height {
                estimated_tip - block_height
            } else {
                0
            }
        } else {
            0
        };

        Ok(FinalityStatus::Unfinalized { confirmations })
    }

    /// Reset finality state (for testing or recovery)
    pub async fn reset(&self) {
        *self.finalized_tip.write().await = None;
        self.finalized_height.store(0, AtomicOrdering::SeqCst);
        self.finalized_count.store(0, AtomicOrdering::SeqCst);
        info!("Finality state reset");
    }
}

/// Status of a block's finality
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FinalityStatus {
    /// Block is finalized and cannot be reorganized
    Finalized,

    /// Block should be finalized but hasn't been processed yet
    PendingFinalization,

    /// Block is not yet finalized
    Unfinalized {
        /// Estimated number of confirmations
        confirmations: u64,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::*;

    fn create_test_block(hash: [u8; 32], height: u64, parent: Hash) -> Block {
        Block {
            header: BlockHeader {
                version: 1,
                block_hash: Hash::new(hash),
                selected_parent_hash: parent,
                merge_parent_hashes: vec![],
                timestamp: 0,
                height,
                blue_score: height,
                blue_work: 0,
                pruning_point: Hash::default(),
                proposer_pubkey: PublicKey::new([0; 32]),
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

    async fn create_chain(dag_store: &DagStore, length: usize) -> Vec<Block> {
        let mut blocks = Vec::new();
        let mut parent = Hash::default();

        for i in 0..length {
            let hash = [i as u8 + 1; 32]; // Start from [1;32] to avoid Hash::default()
            let block = create_test_block(hash, i as u64, parent);
            dag_store.store_block(block.clone()).await.unwrap();
            parent = block.hash();
            blocks.push(block);
        }

        blocks
    }

    #[tokio::test]
    async fn test_finality_tracker_creation() {
        let dag_store = Arc::new(DagStore::new());
        let tracker = FinalityTracker::with_defaults(dag_store);

        assert!(tracker.get_finalized_tip().await.is_none());
        assert_eq!(tracker.get_finalized_height(), 0);
        assert_eq!(tracker.get_finalized_count(), 0);
    }

    #[tokio::test]
    async fn test_no_finality_below_depth() {
        let dag_store = Arc::new(DagStore::new());
        let config = FinalityConfig::for_testing(); // depth = 10
        let tracker = FinalityTracker::new(dag_store.clone(), config);

        // Create a chain of 5 blocks (below finality depth of 10)
        let blocks = create_chain(&dag_store, 5).await;
        let tip = blocks.last().unwrap();

        let finalized = tracker
            .update_finality(&tip.hash(), tip.header.height)
            .await
            .unwrap();

        assert!(finalized.is_empty());
        assert!(tracker.get_finalized_tip().await.is_none());
    }

    #[tokio::test]
    async fn test_finality_at_depth() {
        let dag_store = Arc::new(DagStore::new());
        let config = FinalityConfig::for_testing(); // depth = 10
        let tracker = FinalityTracker::new(dag_store.clone(), config);

        // Create a chain of 15 blocks (5 blocks should be finalized: 0-4)
        let blocks = create_chain(&dag_store, 15).await;
        let tip = blocks.last().unwrap();

        let finalized = tracker
            .update_finality(&tip.hash(), tip.header.height)
            .await
            .unwrap();

        // Blocks 0-4 should be finalized (height 14 - depth 10 = height 4)
        assert_eq!(finalized.len(), 5);
        assert_eq!(tracker.get_finalized_height(), 4);
        assert_eq!(tracker.get_finalized_count(), 5);

        // Check individual blocks
        for i in 0..5 {
            assert!(tracker.is_finalized(&blocks[i].hash()).await);
        }

        // Blocks 5-14 should not be finalized
        for i in 5..15 {
            assert!(!tracker.is_finalized(&blocks[i].hash()).await);
        }
    }

    #[tokio::test]
    async fn test_incremental_finality() {
        let dag_store = Arc::new(DagStore::new());
        let config = FinalityConfig::for_testing(); // depth = 10
        let tracker = FinalityTracker::new(dag_store.clone(), config);

        // Create initial chain of 12 blocks
        let mut blocks = create_chain(&dag_store, 12).await;
        let tip = blocks.last().unwrap();

        // First update: should finalize blocks 0-1
        let finalized = tracker
            .update_finality(&tip.hash(), tip.header.height)
            .await
            .unwrap();
        assert_eq!(finalized.len(), 2);
        assert_eq!(tracker.get_finalized_height(), 1);

        // Add 3 more blocks
        for i in 12..15 {
            let parent = blocks.last().unwrap().hash();
            let block = create_test_block([i as u8 + 1; 32], i as u64, parent);
            dag_store.store_block(block.clone()).await.unwrap();
            blocks.push(block);
        }

        let tip = blocks.last().unwrap();

        // Second update: should finalize blocks 2-4
        let finalized = tracker
            .update_finality(&tip.hash(), tip.header.height)
            .await
            .unwrap();
        assert_eq!(finalized.len(), 3);
        assert_eq!(tracker.get_finalized_height(), 4);
        assert_eq!(tracker.get_finalized_count(), 5);
    }

    #[tokio::test]
    async fn test_reorg_protection() {
        let dag_store = Arc::new(DagStore::new());
        let config = FinalityConfig::for_testing(); // depth = 10
        let tracker = FinalityTracker::new(dag_store.clone(), config);

        // Create chain and finalize some blocks
        // 15 blocks: heights 0-14, with finality depth 10
        // Finalized: heights 0-4 (blocks[0] through blocks[4])
        let blocks = create_chain(&dag_store, 15).await;
        let tip = blocks.last().unwrap();

        tracker
            .update_finality(&tip.hash(), tip.header.height)
            .await
            .unwrap();

        // Finalized height should be 4
        assert_eq!(tracker.get_finalized_height(), 4);

        // Test 1: Reorg from the finalized tip (height 4) - should be OK
        // Branching FROM the finalized tip is allowed
        let result = tracker.check_reorg_allowed(&blocks[4].hash()).await;
        assert!(result.is_ok(), "Branching from finalized tip should be allowed");

        // Test 2: Reorg from an unfinalized block (height 5) - should be OK
        let result = tracker.check_reorg_allowed(&blocks[5].hash()).await;
        assert!(result.is_ok(), "Branching from unfinalized block should be allowed");

        // Test 3: Reorg from a block BELOW finalized height (height 3) - should FAIL
        // This would orphan finalized block at height 4
        let result = tracker.check_reorg_allowed(&blocks[3].hash()).await;
        assert!(
            matches!(result, Err(FinalityError::ReorgPastFinalized(_))),
            "Reorg from below finalized height should fail"
        );

        // Test 4: Reorg from genesis (height 0) - should FAIL
        let result = tracker.check_reorg_allowed(&blocks[0].hash()).await;
        assert!(
            matches!(result, Err(FinalityError::ReorgPastFinalized(_))),
            "Reorg from genesis should fail when genesis is finalized"
        );
    }

    #[tokio::test]
    async fn test_finality_status() {
        let dag_store = Arc::new(DagStore::new());
        let config = FinalityConfig::for_testing();
        let tracker = FinalityTracker::new(dag_store.clone(), config);

        let blocks = create_chain(&dag_store, 15).await;
        let tip = blocks.last().unwrap();

        tracker
            .update_finality(&tip.hash(), tip.header.height)
            .await
            .unwrap();

        // Finalized block
        let status = tracker.get_finality_status(&blocks[0].hash()).await.unwrap();
        assert_eq!(status, FinalityStatus::Finalized);

        // Unfinalized block
        let status = tracker.get_finality_status(&blocks[10].hash()).await.unwrap();
        assert!(matches!(status, FinalityStatus::Unfinalized { .. }));
    }

    #[tokio::test]
    async fn test_finality_events() {
        let dag_store = Arc::new(DagStore::new());
        let config = FinalityConfig::for_testing();
        let tracker = FinalityTracker::new(dag_store.clone(), config);

        let mut receiver = tracker.subscribe();

        let blocks = create_chain(&dag_store, 12).await;
        let tip = blocks.last().unwrap();

        tracker
            .update_finality(&tip.hash(), tip.header.height)
            .await
            .unwrap();

        // Should receive 2 events (blocks 0 and 1)
        let event1 = receiver.try_recv().unwrap();
        assert_eq!(event1.block_hash, blocks[0].hash());
        assert_eq!(event1.height, 0);

        let event2 = receiver.try_recv().unwrap();
        assert_eq!(event2.block_hash, blocks[1].hash());
        assert_eq!(event2.height, 1);
    }

    #[tokio::test]
    async fn test_check_finality() {
        let dag_store = Arc::new(DagStore::new());
        let config = FinalityConfig::for_testing();
        let tracker = FinalityTracker::new(dag_store.clone(), config);

        let blocks = create_chain(&dag_store, 15).await;
        let tip = blocks.last().unwrap();

        tracker
            .update_finality(&tip.hash(), tip.header.height)
            .await
            .unwrap();

        // Blocks at or below finalized height should return true
        assert!(tracker.check_finality(&blocks[0].hash()).await.unwrap());
        assert!(tracker.check_finality(&blocks[4].hash()).await.unwrap());

        // Blocks above finalized height should return false
        assert!(!tracker.check_finality(&blocks[5].hash()).await.unwrap());
        assert!(!tracker.check_finality(&blocks[14].hash()).await.unwrap());
    }

    #[tokio::test]
    async fn test_reset() {
        let dag_store = Arc::new(DagStore::new());
        let config = FinalityConfig::for_testing();
        let tracker = FinalityTracker::new(dag_store.clone(), config);

        let blocks = create_chain(&dag_store, 15).await;
        let tip = blocks.last().unwrap();

        tracker
            .update_finality(&tip.hash(), tip.header.height)
            .await
            .unwrap();

        assert!(tracker.get_finalized_tip().await.is_some());
        assert!(tracker.get_finalized_height() > 0);

        tracker.reset().await;

        assert!(tracker.get_finalized_tip().await.is_none());
        assert_eq!(tracker.get_finalized_height(), 0);
        assert_eq!(tracker.get_finalized_count(), 0);
    }
}
