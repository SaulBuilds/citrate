// citrate/core/consensus/src/chain_selection.rs

use crate::dag_store::DagStore;
use crate::ghostdag::GhostDag;
use crate::tip_selection::TipSelector;
use crate::types::{Block, Hash};
use std::collections::HashSet;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::RwLock;
use tracing::{info, warn};

#[derive(Error, Debug)]
pub enum ChainSelectionError {
    #[error("Block not found: {0}")]
    BlockNotFound(Hash),

    #[error("Invalid chain state")]
    InvalidChainState,

    #[error("Reorganization depth exceeded")]
    ReorgDepthExceeded,

    #[error("DAG error: {0}")]
    DagError(String),
}

/// Chain state tracking
#[derive(Debug, Clone)]
pub struct ChainState {
    /// Current chain tip
    pub tip: Hash,

    /// Chain height
    pub height: u64,

    /// Total blue score
    pub blue_score: u64,

    /// Total blue work
    pub blue_work: u128,

    /// Selected parent chain
    pub selected_chain: Vec<Hash>,
}

/// Chain selection and reorganization manager
pub struct ChainSelector {
    dag_store: Arc<DagStore>,
    ghostdag: Arc<GhostDag>,
    _tip_selector: Arc<TipSelector>,
    current_chain: Arc<RwLock<ChainState>>,
    max_reorg_depth: u64,
    reorg_history: Arc<RwLock<Vec<ReorgEvent>>>,
}

#[derive(Debug, Clone)]
pub struct ReorgEvent {
    pub timestamp: u64,
    pub old_tip: Hash,
    pub new_tip: Hash,
    pub depth: u64,
    pub reason: String,
}

impl ChainSelector {
    pub fn new(
        dag_store: Arc<DagStore>,
        ghostdag: Arc<GhostDag>,
        tip_selector: Arc<TipSelector>,
        max_reorg_depth: u64,
    ) -> Self {
        Self {
            dag_store,
            ghostdag,
            _tip_selector: tip_selector,
            current_chain: Arc::new(RwLock::new(ChainState {
                tip: Hash::default(),
                height: 0,
                blue_score: 0,
                blue_work: 0,
                selected_chain: Vec::new(),
            })),
            max_reorg_depth,
            reorg_history: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Update chain selection based on new block
    pub async fn on_new_block(&self, block: &Block) -> Result<bool, ChainSelectionError> {
        let new_blue_score = self
            .ghostdag
            .get_blue_score(&block.hash())
            .await
            .map_err(|e| ChainSelectionError::DagError(e.to_string()))?;

        let current_chain = self.current_chain.read().await;
        let current_score = current_chain.blue_score;
        drop(current_chain);

        // Check if new block creates a better chain
        if new_blue_score > current_score {
            info!(
                "New block {} has higher blue score ({} > {}), considering reorg",
                block.hash(),
                new_blue_score,
                current_score
            );

            return self.attempt_reorganization(block).await;
        }

        // Check if new block extends current chain
        if self.extends_current_chain(block).await? {
            self.extend_chain(block).await?;
            return Ok(false); // No reorg needed
        }

        Ok(false)
    }

    /// Check if block extends current chain
    async fn extends_current_chain(&self, block: &Block) -> Result<bool, ChainSelectionError> {
        let current_chain = self.current_chain.read().await;

        // If there's no current chain, this is the first block
        if current_chain.tip == Hash::default() {
            return Ok(true);
        }

        // Check if selected parent is current tip
        if block.selected_parent() == current_chain.tip {
            return Ok(true);
        }

        // Check if any parent is current tip (for merge blocks)
        for parent in block.parents() {
            if parent == current_chain.tip {
                return Ok(true);
            }
        }

        Ok(false)
    }

    /// Extend current chain with new block
    async fn extend_chain(&self, block: &Block) -> Result<(), ChainSelectionError> {
        let mut chain = self.current_chain.write().await;

        chain.tip = block.hash();
        chain.height = block.header.height;
        chain.blue_score = block.header.blue_score;
        chain.blue_work = block.header.blue_work;
        chain.selected_chain.push(block.hash());

        info!(
            "Extended chain to block {} at height {}",
            block.hash(),
            block.header.height
        );

        Ok(())
    }

    /// Attempt chain reorganization
    async fn attempt_reorganization(
        &self,
        new_tip_block: &Block,
    ) -> Result<bool, ChainSelectionError> {
        let current_chain = self.current_chain.read().await;
        let old_tip = current_chain.tip;
        drop(current_chain);

        // Find common ancestor
        let (common_ancestor, reorg_depth) = self
            .find_common_ancestor(old_tip, new_tip_block.hash())
            .await?;

        // Check reorg depth limit
        if reorg_depth > self.max_reorg_depth {
            warn!(
                "Reorg depth {} exceeds maximum {}, rejecting",
                reorg_depth, self.max_reorg_depth
            );
            return Err(ChainSelectionError::ReorgDepthExceeded);
        }

        // Build new chain from common ancestor to new tip
        let new_chain = self
            .build_chain(common_ancestor, new_tip_block.hash())
            .await?;

        // Perform reorganization
        self.perform_reorg(old_tip, new_tip_block.hash(), new_chain, reorg_depth)
            .await?;

        Ok(true)
    }

    /// Find common ancestor between two chains
    async fn find_common_ancestor(
        &self,
        tip1: Hash,
        tip2: Hash,
    ) -> Result<(Hash, u64), ChainSelectionError> {
        // Handle case where one tip is the default hash (empty chain)
        if tip1 == Hash::default() || tip2 == Hash::default() {
            return Ok((Hash::default(), 0));
        }

        let mut ancestors1 = HashSet::new();
        let mut ancestors2 = HashSet::new();

        let mut current1 = Some(tip1);
        let mut current2 = Some(tip2);
        let mut depth = 0;

        // Walk back both chains until we find common ancestor
        while current1.is_some() || current2.is_some() {
            if let Some(hash1) = current1 {
                if ancestors2.contains(&hash1) {
                    return Ok((hash1, depth));
                }
                ancestors1.insert(hash1);

                let block = self
                    .dag_store
                    .get_block(&hash1)
                    .await
                    .map_err(|_| ChainSelectionError::BlockNotFound(hash1))?;

                current1 = if block.is_genesis() {
                    None
                } else {
                    Some(block.selected_parent())
                };
            }

            if let Some(hash2) = current2 {
                if ancestors1.contains(&hash2) {
                    return Ok((hash2, depth));
                }
                ancestors2.insert(hash2);

                let block = self
                    .dag_store
                    .get_block(&hash2)
                    .await
                    .map_err(|_| ChainSelectionError::BlockNotFound(hash2))?;

                current2 = if block.is_genesis() {
                    None
                } else {
                    Some(block.selected_parent())
                };
            }

            depth += 1;
        }

        Err(ChainSelectionError::InvalidChainState)
    }

    /// Build chain from ancestor to tip
    async fn build_chain(
        &self,
        ancestor: Hash,
        tip: Hash,
    ) -> Result<Vec<Hash>, ChainSelectionError> {
        let mut chain = Vec::new();
        let mut current = tip;

        // Walk back from tip to ancestor
        while current != ancestor {
            chain.push(current);

            let block = self
                .dag_store
                .get_block(&current)
                .await
                .map_err(|_| ChainSelectionError::BlockNotFound(current))?;

            if block.is_genesis() {
                break;
            }

            current = block.selected_parent();
        }

        // Reverse to get ancestor -> tip order
        chain.reverse();
        Ok(chain)
    }

    /// Perform the actual reorganization
    async fn perform_reorg(
        &self,
        old_tip: Hash,
        new_tip: Hash,
        new_chain: Vec<Hash>,
        depth: u64,
    ) -> Result<(), ChainSelectionError> {
        info!(
            "Performing reorganization: {} -> {} (depth: {})",
            old_tip, new_tip, depth
        );

        // Get new tip block for chain state
        let new_tip_block = self
            .dag_store
            .get_block(&new_tip)
            .await
            .map_err(|_| ChainSelectionError::BlockNotFound(new_tip))?;

        // Update chain state
        let mut chain = self.current_chain.write().await;
        chain.tip = new_tip;
        chain.height = new_tip_block.header.height;
        chain.blue_score = new_tip_block.header.blue_score;
        chain.blue_work = new_tip_block.header.blue_work;
        chain.selected_chain = new_chain;
        drop(chain);

        // Record reorg event
        let event = ReorgEvent {
            timestamp: chrono::Utc::now().timestamp() as u64,
            old_tip,
            new_tip,
            depth,
            reason: format!("Higher blue score: {}", new_tip_block.header.blue_score),
        };

        self.reorg_history.write().await.push(event);

        Ok(())
    }

    /// Get current chain state
    pub async fn get_chain_state(&self) -> ChainState {
        self.current_chain.read().await.clone()
    }

    /// Get selected chain from genesis to tip
    pub async fn get_selected_chain(&self) -> Vec<Hash> {
        self.current_chain.read().await.selected_chain.clone()
    }

    /// Get reorganization history
    pub async fn get_reorg_history(&self) -> Vec<ReorgEvent> {
        self.reorg_history.read().await.clone()
    }

    /// Validate chain consistency
    pub async fn validate_chain(&self) -> Result<bool, ChainSelectionError> {
        let chain_state = self.current_chain.read().await;

        if chain_state.selected_chain.is_empty() {
            return Ok(true); // Empty chain is valid
        }

        let mut prev_hash = Hash::default();
        let mut prev_height = 0;

        for hash in &chain_state.selected_chain {
            let block = self
                .dag_store
                .get_block(hash)
                .await
                .map_err(|_| ChainSelectionError::BlockNotFound(*hash))?;

            // Check height increases
            if block.header.height <= prev_height && prev_height > 0 {
                warn!("Invalid chain: height not increasing at {}", hash);
                return Ok(false);
            }

            // Check parent linkage (except for first block)
            if prev_hash != Hash::default() {
                let parents = block.parents();
                if !parents.contains(&prev_hash) {
                    warn!("Invalid chain: broken parent link at {}", hash);
                    return Ok(false);
                }
            }

            prev_hash = *hash;
            prev_height = block.header.height;
        }

        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tip_selection::SelectionStrategy;
    use crate::types::*;

    async fn setup_test_env() -> (
        Arc<DagStore>,
        Arc<GhostDag>,
        Arc<TipSelector>,
        Arc<ChainSelector>,
    ) {
        let dag_store = Arc::new(DagStore::new());
        let ghostdag = Arc::new(GhostDag::new(GhostDagParams::default(), dag_store.clone()));
        let tip_selector = Arc::new(TipSelector::new(
            dag_store.clone(),
            ghostdag.clone(),
            SelectionStrategy::HighestBlueScore,
        ));
        let chain_selector = Arc::new(ChainSelector::new(
            dag_store.clone(),
            ghostdag.clone(),
            tip_selector.clone(),
            100, // max reorg depth
        ));

        (dag_store, ghostdag, tip_selector, chain_selector)
    }

    #[tokio::test]
    async fn test_chain_extension() {
        let (dag_store, _, _, chain_selector) = setup_test_env().await;

        // Create genesis block
        let genesis = Block {
            header: BlockHeader {
                version: 1,
                block_hash: Hash::new([0xFF; 32]),
                selected_parent_hash: Hash::default(),
                merge_parent_hashes: vec![],
                timestamp: 0,
                height: 0,
                blue_score: 1,
                blue_work: 1,
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
        };

        dag_store.store_block(genesis.clone()).await.unwrap();

        // Extend chain
        chain_selector.extend_chain(&genesis).await.unwrap();

        let chain_state = chain_selector.get_chain_state().await;
        assert_eq!(chain_state.tip, genesis.hash());
        assert_eq!(chain_state.height, 0);
    }

    #[tokio::test]
    async fn test_chain_validation() {
        let (_, _, _, chain_selector) = setup_test_env().await;

        // Empty chain should be valid
        assert!(chain_selector.validate_chain().await.unwrap());
    }
}
