// citrate/core/consensus/src/ordering.rs

//! Total Ordering for GhostDAG
//!
//! This module provides deterministic total ordering of blocks in a DAG structure.
//! The ordering follows the GhostDAG protocol:
//!
//! 1. Walk the selected-parent chain from genesis to tip
//! 2. For each block on the chain, yield it first
//! 3. Then yield its mergeset (blocks reachable only via merge parents)
//! 4. Mergeset blocks are sorted by (blue_score DESC, hash ASC) for determinism
//! 5. Each block is yielded exactly once (first occurrence wins)
//!
//! This ensures all nodes produce identical orderings for the same DAG state.

use crate::dag_store::DagStore;
use crate::ghostdag::GhostDag;
use crate::types::Hash;
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;
use thiserror::Error;
use tracing::{debug, info};

#[derive(Error, Debug)]
pub enum OrderingError {
    #[error("Block not found: {0}")]
    BlockNotFound(Hash),

    #[error("Invalid ordering state")]
    InvalidState,

    #[error("Cycle detected in ordering")]
    CycleDetected,

    #[error("DAG store error: {0}")]
    DagStoreError(String),
}

/// Result of ordering a range of blocks
#[derive(Debug, Clone)]
pub struct OrderedBlockRange {
    /// Blocks in total order (oldest first)
    pub blocks: Vec<Hash>,

    /// Transactions in total order (for execution)
    pub transaction_order: Vec<TransactionRef>,

    /// Starting block hash
    pub from: Hash,

    /// Ending block hash
    pub to: Hash,
}

/// Reference to a transaction within a block
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TransactionRef {
    /// Block containing the transaction
    pub block_hash: Hash,

    /// Index within the block's transaction list
    pub tx_index: usize,

    /// Transaction hash
    pub tx_hash: Hash,
}

/// Block ordering metadata for sorting
#[derive(Debug, Clone)]
struct BlockOrderingInfo {
    hash: Hash,
    blue_score: u64,
    #[allow(dead_code)]
    height: u64,
    #[allow(dead_code)]
    is_chain_block: bool,
}

impl BlockOrderingInfo {
    /// Compare for ordering: higher blue score first, then lower hash (lexicographic)
    fn ordering_key(&self) -> (std::cmp::Reverse<u64>, Hash) {
        (std::cmp::Reverse(self.blue_score), self.hash)
    }
}

/// Total order iterator for GhostDAG blocks
pub struct TotalOrderIterator {
    /// DAG storage
    dag_store: Arc<DagStore>,

    /// GhostDAG instance for blue score queries (reserved for future optimizations)
    #[allow(dead_code)]
    ghostdag: Arc<GhostDag>,

    /// Blocks already yielded (for deduplication)
    yielded: HashSet<Hash>,

    /// Queue of blocks to process on selected-parent chain
    chain_queue: VecDeque<Hash>,

    /// Queue of mergeset blocks pending for current chain block
    mergeset_queue: VecDeque<Hash>,

    /// Current chain block being processed
    current_chain_block: Option<Hash>,

    /// Whether we're done iterating
    finished: bool,
}

impl TotalOrderIterator {
    /// Create a new iterator from genesis to the specified tip
    pub async fn new(
        dag_store: Arc<DagStore>,
        ghostdag: Arc<GhostDag>,
        tip: Hash,
    ) -> Result<Self, OrderingError> {
        // Build the selected-parent chain from tip back to genesis
        let chain = Self::build_selected_parent_chain(&dag_store, tip).await?;

        let mut chain_queue = VecDeque::new();
        // Add chain blocks oldest first (genesis -> tip)
        for hash in chain.into_iter().rev() {
            chain_queue.push_back(hash);
        }

        Ok(Self {
            dag_store,
            ghostdag,
            yielded: HashSet::new(),
            chain_queue,
            mergeset_queue: VecDeque::new(),
            current_chain_block: None,
            finished: false,
        })
    }

    /// Build selected-parent chain from tip to genesis
    async fn build_selected_parent_chain(
        dag_store: &DagStore,
        tip: Hash,
    ) -> Result<Vec<Hash>, OrderingError> {
        let mut chain = Vec::new();
        let mut current = tip;

        loop {
            chain.push(current);

            let block = dag_store
                .get_block(&current)
                .await
                .map_err(|_| OrderingError::BlockNotFound(current))?;

            if block.is_genesis() {
                break;
            }

            current = block.selected_parent();
        }

        Ok(chain)
    }

    /// Get the next block in total order
    pub async fn next(&mut self) -> Result<Option<Hash>, OrderingError> {
        if self.finished {
            return Ok(None);
        }

        loop {
            // First, try to yield from mergeset queue
            if let Some(hash) = self.next_from_mergeset().await? {
                return Ok(Some(hash));
            }

            // No more mergeset blocks, move to next chain block
            if let Some(chain_hash) = self.chain_queue.pop_front() {
                // Prepare mergeset for this chain block
                self.prepare_mergeset(&chain_hash).await?;

                // Yield the chain block itself if not already yielded
                if !self.yielded.contains(&chain_hash) {
                    self.yielded.insert(chain_hash);
                    self.current_chain_block = Some(chain_hash);
                    return Ok(Some(chain_hash));
                }

                // Chain block already yielded (from earlier mergeset), continue to its mergeset
                self.current_chain_block = Some(chain_hash);
                continue;
            }

            // No more chain blocks, we're done
            self.finished = true;
            return Ok(None);
        }
    }

    /// Get next block from mergeset queue
    async fn next_from_mergeset(&mut self) -> Result<Option<Hash>, OrderingError> {
        while let Some(hash) = self.mergeset_queue.pop_front() {
            if !self.yielded.contains(&hash) {
                self.yielded.insert(hash);
                return Ok(Some(hash));
            }
        }
        Ok(None)
    }

    /// Prepare the mergeset for a chain block
    async fn prepare_mergeset(&mut self, chain_block: &Hash) -> Result<(), OrderingError> {
        let block = self
            .dag_store
            .get_block(chain_block)
            .await
            .map_err(|_| OrderingError::BlockNotFound(*chain_block))?;

        // Collect all blocks reachable via merge parents (the mergeset)
        let mut mergeset = Vec::new();

        for merge_parent in &block.header.merge_parent_hashes {
            // Recursively collect blocks from merge parent's ancestry
            // that aren't already in our yielded set
            self.collect_mergeset_recursive(*merge_parent, &mut mergeset)
                .await?;
        }

        // Sort mergeset by blue score (descending) then hash (ascending)
        let mut ordering_infos: Vec<BlockOrderingInfo> = Vec::new();
        for hash in &mergeset {
            if let Ok(b) = self.dag_store.get_block(hash).await {
                ordering_infos.push(BlockOrderingInfo {
                    hash: *hash,
                    blue_score: b.header.blue_score,
                    height: b.header.height,
                    is_chain_block: false,
                });
            }
        }

        ordering_infos.sort_by_key(|info| info.ordering_key());

        // Add sorted mergeset to queue
        self.mergeset_queue.clear();
        for info in ordering_infos {
            if !self.yielded.contains(&info.hash) {
                self.mergeset_queue.push_back(info.hash);
            }
        }

        Ok(())
    }

    /// Recursively collect mergeset blocks
    async fn collect_mergeset_recursive(
        &self,
        hash: Hash,
        collected: &mut Vec<Hash>,
    ) -> Result<(), OrderingError> {
        // Skip if already collected or yielded
        if self.yielded.contains(&hash) || collected.contains(&hash) {
            return Ok(());
        }

        collected.push(hash);

        // Get the block and recurse to its parents
        if let Ok(block) = self.dag_store.get_block(&hash).await {
            // Only recurse to parents not already yielded
            if !self.yielded.contains(&block.selected_parent()) && !block.is_genesis() {
                // Only include selected parent if it's not in our chain
                // (would have been yielded earlier or will be yielded later)
                // This is handled by the yielded check above
            }

            for merge_parent in &block.header.merge_parent_hashes {
                Box::pin(self.collect_mergeset_recursive(*merge_parent, collected)).await?;
            }
        }

        Ok(())
    }

    /// Collect all remaining blocks into a vector
    pub async fn collect(mut self) -> Result<Vec<Hash>, OrderingError> {
        let mut result = Vec::new();
        while let Some(hash) = self.next().await? {
            result.push(hash);
        }
        Ok(result)
    }
}

/// Total ordering manager
pub struct TotalOrdering {
    dag_store: Arc<DagStore>,
    ghostdag: Arc<GhostDag>,

    /// Cache of ordered ranges
    order_cache: tokio::sync::RwLock<HashMap<(Hash, Hash), Vec<Hash>>>,
}

impl TotalOrdering {
    pub fn new(dag_store: Arc<DagStore>, ghostdag: Arc<GhostDag>) -> Self {
        Self {
            dag_store,
            ghostdag,
            order_cache: tokio::sync::RwLock::new(HashMap::new()),
        }
    }

    /// Get total order of all blocks from genesis to tip
    pub async fn get_total_order(&self, tip: Hash) -> Result<Vec<Hash>, OrderingError> {
        // Check cache
        let cache_key = (Hash::default(), tip);
        if let Some(cached) = self.order_cache.read().await.get(&cache_key) {
            return Ok(cached.clone());
        }

        // Calculate order
        let iterator =
            TotalOrderIterator::new(self.dag_store.clone(), self.ghostdag.clone(), tip).await?;

        let order = iterator.collect().await?;

        // Cache result
        self.order_cache
            .write()
            .await
            .insert(cache_key, order.clone());

        info!(
            "Computed total order of {} blocks ending at {}",
            order.len(),
            tip
        );

        Ok(order)
    }

    /// Get ordered blocks between two points (exclusive of from, inclusive of to)
    pub async fn get_ordered_blocks(
        &self,
        from: Hash,
        to: Hash,
    ) -> Result<OrderedBlockRange, OrderingError> {
        // Get full order to tip
        let full_order = self.get_total_order(to).await?;

        // Find starting point
        let from_idx = if from == Hash::default() {
            0
        } else {
            full_order
                .iter()
                .position(|h| *h == from)
                .ok_or(OrderingError::BlockNotFound(from))?
                + 1 // Exclusive of 'from'
        };

        // Slice the relevant portion
        let blocks: Vec<Hash> = full_order[from_idx..].to_vec();

        // Build transaction order
        let mut transaction_order = Vec::new();
        for block_hash in &blocks {
            let block = self
                .dag_store
                .get_block(block_hash)
                .await
                .map_err(|_| OrderingError::BlockNotFound(*block_hash))?;

            for (tx_index, tx) in block.transactions.iter().enumerate() {
                transaction_order.push(TransactionRef {
                    block_hash: *block_hash,
                    tx_index,
                    tx_hash: tx.hash,
                });
            }
        }

        debug!(
            "Ordered {} blocks with {} transactions from {} to {}",
            blocks.len(),
            transaction_order.len(),
            from,
            to
        );

        Ok(OrderedBlockRange {
            blocks,
            transaction_order,
            from,
            to,
        })
    }

    /// Get transaction execution order for a range
    pub async fn get_transaction_order(
        &self,
        from: Hash,
        to: Hash,
    ) -> Result<Vec<TransactionRef>, OrderingError> {
        let range = self.get_ordered_blocks(from, to).await?;
        Ok(range.transaction_order)
    }

    /// Verify that two orderings are consistent
    pub fn verify_ordering_consistency(order1: &[Hash], order2: &[Hash]) -> bool {
        if order1.len() != order2.len() {
            return false;
        }

        order1.iter().zip(order2.iter()).all(|(a, b)| a == b)
    }

    /// Clear the order cache (e.g., after reorg)
    pub async fn clear_cache(&self) {
        self.order_cache.write().await.clear();
        debug!("Cleared total ordering cache");
    }

    /// Invalidate cache entries that include a specific block
    pub async fn invalidate_block(&self, block: &Hash) {
        let mut cache = self.order_cache.write().await;
        cache.retain(|(_, to), order| *to != *block && !order.contains(block));
        debug!("Invalidated cache entries containing block {}", block);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::*;

    fn create_test_block(
        hash: [u8; 32],
        selected_parent: Hash,
        merge_parents: Vec<Hash>,
        height: u64,
        blue_score: u64,
    ) -> Block {
        Block {
            header: BlockHeader {
                version: 1,
                block_hash: Hash::new(hash),
                selected_parent_hash: selected_parent,
                merge_parent_hashes: merge_parents,
                timestamp: 0,
                height,
                blue_score,
                blue_work: 0,
                pruning_point: Hash::default(),
                proposer_pubkey: PublicKey::new([0; 32]),
                vrf_reveal: VrfProof {
                    proof: vec![],
                    output: Hash::default(),
                },
                base_fee_per_gas: 0,
                gas_used: 0,
                gas_limit: 30_000_000,
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

    #[tokio::test]
    async fn test_linear_chain_ordering() {
        let dag_store = Arc::new(DagStore::new());
        let ghostdag = Arc::new(GhostDag::new(
            GhostDagParams::default(),
            dag_store.clone(),
        ));

        // Create linear chain: G -> A -> B -> C
        // Use non-zero hash for genesis to avoid confusion with Hash::default()
        let genesis = create_test_block([0xFF; 32], Hash::default(), vec![], 0, 1);
        let block_a = create_test_block([1; 32], genesis.hash(), vec![], 1, 2);
        let block_b = create_test_block([2; 32], block_a.hash(), vec![], 2, 3);
        let block_c = create_test_block([3; 32], block_b.hash(), vec![], 3, 4);

        dag_store.store_block(genesis.clone()).await.unwrap();
        dag_store.store_block(block_a.clone()).await.unwrap();
        dag_store.store_block(block_b.clone()).await.unwrap();
        dag_store.store_block(block_c.clone()).await.unwrap();

        let ordering = TotalOrdering::new(dag_store.clone(), ghostdag.clone());
        let order = ordering.get_total_order(block_c.hash()).await.unwrap();

        assert_eq!(order.len(), 4);
        assert_eq!(order[0], genesis.hash());
        assert_eq!(order[1], block_a.hash());
        assert_eq!(order[2], block_b.hash());
        assert_eq!(order[3], block_c.hash());
    }

    #[tokio::test]
    async fn test_simple_fork_ordering() {
        let dag_store = Arc::new(DagStore::new());
        let ghostdag = Arc::new(GhostDag::new(
            GhostDagParams::default(),
            dag_store.clone(),
        ));

        // Create fork:
        //     G
        //    / \
        //   A   B
        //    \ /
        //     C (selects A, merges B)

        let genesis = create_test_block([0xFF; 32], Hash::default(), vec![], 0, 1);
        let block_a = create_test_block([1; 32], genesis.hash(), vec![], 1, 2);
        let block_b = create_test_block([2; 32], genesis.hash(), vec![], 1, 2);
        let block_c =
            create_test_block([3; 32], block_a.hash(), vec![block_b.hash()], 2, 4);

        dag_store.store_block(genesis.clone()).await.unwrap();
        dag_store.store_block(block_a.clone()).await.unwrap();
        dag_store.store_block(block_b.clone()).await.unwrap();
        dag_store.store_block(block_c.clone()).await.unwrap();

        let ordering = TotalOrdering::new(dag_store.clone(), ghostdag.clone());
        let order = ordering.get_total_order(block_c.hash()).await.unwrap();

        // Order should be: G, A, B (via mergeset), C
        // or G, A, C, B depending on when B is picked up
        assert_eq!(order.len(), 4);
        assert_eq!(order[0], genesis.hash()); // Genesis first
        assert_eq!(order[1], block_a.hash()); // Selected parent chain
        // B should appear before or at C since it's in C's mergeset
        assert!(order.contains(&block_b.hash()));
        assert!(order.contains(&block_c.hash()));

        // C must come after A (its selected parent)
        let a_pos = order.iter().position(|h| *h == block_a.hash()).unwrap();
        let c_pos = order.iter().position(|h| *h == block_c.hash()).unwrap();
        assert!(c_pos > a_pos);
    }

    #[tokio::test]
    async fn test_ordering_determinism() {
        let dag_store = Arc::new(DagStore::new());
        let ghostdag = Arc::new(GhostDag::new(
            GhostDagParams::default(),
            dag_store.clone(),
        ));

        // Create a more complex DAG
        let genesis = create_test_block([0xFF; 32], Hash::default(), vec![], 0, 1);
        let block_a = create_test_block([1; 32], genesis.hash(), vec![], 1, 2);
        let block_b = create_test_block([2; 32], genesis.hash(), vec![], 1, 2);
        let block_c = create_test_block([3; 32], block_a.hash(), vec![], 2, 3);
        let block_d =
            create_test_block([4; 32], block_c.hash(), vec![block_b.hash()], 3, 5);

        dag_store.store_block(genesis.clone()).await.unwrap();
        dag_store.store_block(block_a.clone()).await.unwrap();
        dag_store.store_block(block_b.clone()).await.unwrap();
        dag_store.store_block(block_c.clone()).await.unwrap();
        dag_store.store_block(block_d.clone()).await.unwrap();

        let ordering = TotalOrdering::new(dag_store.clone(), ghostdag.clone());

        // Run ordering multiple times to verify determinism
        let order1 = ordering.get_total_order(block_d.hash()).await.unwrap();

        // Clear cache and recompute
        ordering.clear_cache().await;
        let order2 = ordering.get_total_order(block_d.hash()).await.unwrap();

        assert!(TotalOrdering::verify_ordering_consistency(&order1, &order2));
    }

    #[tokio::test]
    async fn test_ordered_blocks_range() {
        let dag_store = Arc::new(DagStore::new());
        let ghostdag = Arc::new(GhostDag::new(
            GhostDagParams::default(),
            dag_store.clone(),
        ));

        // Linear chain
        let genesis = create_test_block([0xFF; 32], Hash::default(), vec![], 0, 1);
        let block_a = create_test_block([1; 32], genesis.hash(), vec![], 1, 2);
        let block_b = create_test_block([2; 32], block_a.hash(), vec![], 2, 3);
        let block_c = create_test_block([3; 32], block_b.hash(), vec![], 3, 4);

        dag_store.store_block(genesis.clone()).await.unwrap();
        dag_store.store_block(block_a.clone()).await.unwrap();
        dag_store.store_block(block_b.clone()).await.unwrap();
        dag_store.store_block(block_c.clone()).await.unwrap();

        let ordering = TotalOrdering::new(dag_store.clone(), ghostdag.clone());

        // Get range from A (exclusive) to C (inclusive)
        let range = ordering
            .get_ordered_blocks(block_a.hash(), block_c.hash())
            .await
            .unwrap();

        assert_eq!(range.blocks.len(), 2); // B and C
        assert_eq!(range.blocks[0], block_b.hash());
        assert_eq!(range.blocks[1], block_c.hash());
    }

    #[tokio::test]
    async fn test_transaction_ordering() {
        let dag_store = Arc::new(DagStore::new());
        let ghostdag = Arc::new(GhostDag::new(
            GhostDagParams::default(),
            dag_store.clone(),
        ));

        // Create blocks with transactions
        let tx1 = Transaction {
            hash: Hash::new([0x11; 32]),
            nonce: 1,
            from: PublicKey::new([1; 32]),
            to: Some(PublicKey::new([2; 32])),
            value: 100,
            gas_limit: 21000,
            gas_price: 1,
            data: vec![],
            signature: Signature::new([0; 64]),
            tx_type: Some(TransactionType::Standard),
        };

        let tx2 = Transaction {
            hash: Hash::new([0x22; 32]),
            nonce: 2,
            from: PublicKey::new([1; 32]),
            to: Some(PublicKey::new([2; 32])),
            value: 200,
            gas_limit: 21000,
            gas_price: 1,
            data: vec![],
            signature: Signature::new([0; 64]),
            tx_type: Some(TransactionType::Standard),
        };

        let mut genesis = create_test_block([0xFF; 32], Hash::default(), vec![], 0, 1);
        genesis.transactions.push(tx1.clone());

        let mut block_a = create_test_block([1; 32], genesis.hash(), vec![], 1, 2);
        block_a.transactions.push(tx2.clone());

        dag_store.store_block(genesis.clone()).await.unwrap();
        dag_store.store_block(block_a.clone()).await.unwrap();

        let ordering = TotalOrdering::new(dag_store.clone(), ghostdag.clone());
        let tx_order = ordering
            .get_transaction_order(Hash::default(), block_a.hash())
            .await
            .unwrap();

        assert_eq!(tx_order.len(), 2);
        assert_eq!(tx_order[0].tx_hash, tx1.hash);
        assert_eq!(tx_order[0].block_hash, genesis.hash());
        assert_eq!(tx_order[1].tx_hash, tx2.hash);
        assert_eq!(tx_order[1].block_hash, block_a.hash());
    }
}
