// citrate/core/consensus/src/ghostdag.rs

use crate::dag_store::DagStore;
use crate::types::{Block, BlueSet, DagRelation, GhostDagParams, Hash};
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::RwLock;
use tracing::{debug, info};

#[derive(Error, Debug)]
pub enum GhostDagError {
    #[error("Block not found: {0}")]
    BlockNotFound(Hash),

    #[error("Invalid parent structure")]
    InvalidParents,

    #[error("Cycle detected in DAG")]
    CycleDetected,

    #[error("K-cluster violation")]
    KClusterViolation,
}

/// GhostDAG consensus engine
pub struct GhostDag {
    /// Consensus parameters
    params: GhostDagParams,

    /// DAG storage
    dag_store: Arc<DagStore>,

    /// DAG relations cache
    relations: Arc<RwLock<HashMap<Hash, DagRelation>>>,

    /// Blue set cache for efficiency
    blue_cache: Arc<RwLock<HashMap<Hash, BlueSet>>>,

    /// Current tips of the DAG
    tips: Arc<RwLock<HashSet<Hash>>>,
}

impl GhostDag {
    pub fn new(params: GhostDagParams, dag_store: Arc<DagStore>) -> Self {
        Self {
            params,
            dag_store,
            relations: Arc::new(RwLock::new(HashMap::new())),
            blue_cache: Arc::new(RwLock::new(HashMap::new())),
            tips: Arc::new(RwLock::new(HashSet::new())),
        }
    }

    /// Get consensus parameters
    pub fn params(&self) -> &GhostDagParams {
        &self.params
    }

    /// Calculate blue set for a block following GhostDAG rules
    pub async fn calculate_blue_set(&self, block: &Block) -> Result<BlueSet, GhostDagError> {
        // Check cache first
        if let Some(cached) = self.blue_cache.read().await.get(&block.hash()) {
            return Ok(cached.clone());
        }

        let mut blue_set = BlueSet::new();

        // Genesis block is always blue
        if block.is_genesis() {
            blue_set.insert(block.hash());
            self.blue_cache
                .write()
                .await
                .insert(block.hash(), blue_set.clone());
            return Ok(blue_set);
        }

        // Get selected parent's blue set
        let selected_parent_blue = self
            .get_or_calculate_blue_set(&block.selected_parent())
            .await?;

        // Start with selected parent's blue set
        blue_set.blocks = selected_parent_blue.blocks.clone();
        blue_set.score = selected_parent_blue.score;

        // Calculate blue blocks among merge parents
        let blue_merge_parents = self
            .calculate_blue_merge_parents(block, &selected_parent_blue)
            .await?;

        // Add blue merge parents to blue set
        for parent_hash in &blue_merge_parents {
            let parent_blue_set = self.get_or_calculate_blue_set(parent_hash).await?;
            blue_set.blocks.extend(parent_blue_set.blocks);
        }

        // Add current block to blue set and recompute score from set size
        blue_set.blocks.insert(block.hash());
        blue_set.score = blue_set.blocks.len() as u64;

        // Cache the result
        self.blue_cache
            .write()
            .await
            .insert(block.hash(), blue_set.clone());

        info!(
            "Calculated blue set for block {}: score={}",
            block.hash(),
            blue_set.score
        );
        Ok(blue_set)
    }

    /// Calculate which merge parents are blue according to k-cluster rule
    async fn calculate_blue_merge_parents(
        &self,
        block: &Block,
        selected_parent_blue: &BlueSet,
    ) -> Result<Vec<Hash>, GhostDagError> {
        let mut blue_parents = Vec::new();
        let mut red_parents = Vec::new();

        for merge_parent in &block.header.merge_parent_hashes {
            if self
                .is_blue_candidate(merge_parent, selected_parent_blue, &blue_parents)
                .await?
            {
                blue_parents.push(*merge_parent);
            } else {
                red_parents.push(*merge_parent);
            }
        }

        debug!(
            "Block {} has {} blue and {} red merge parents",
            block.hash(),
            blue_parents.len(),
            red_parents.len()
        );

        Ok(blue_parents)
    }

    /// Check if a block can be blue according to k-cluster rule
    async fn is_blue_candidate(
        &self,
        candidate: &Hash,
        selected_parent_blue: &BlueSet,
        current_blue_parents: &[Hash],
    ) -> Result<bool, GhostDagError> {
        // Count blue anticone size
        let anticone_size = self
            .count_blue_anticone(candidate, selected_parent_blue, current_blue_parents)
            .await?;

        // Check k-cluster rule
        Ok(anticone_size <= self.params.k as usize)
    }

    /// Count blue blocks in anticone
    async fn count_blue_anticone(
        &self,
        block: &Hash,
        reference_blue_set: &BlueSet,
        additional_blues: &[Hash],
    ) -> Result<usize, GhostDagError> {
        let mut count = 0;

        // Check against reference blue set
        for blue_block in &reference_blue_set.blocks {
            if !self.is_ancestor_of(block, blue_block).await?
                && !self.is_ancestor_of(blue_block, block).await?
            {
                count += 1;
            }
        }

        // Check against additional blue blocks
        for blue_block in additional_blues {
            if !self.is_ancestor_of(block, blue_block).await?
                && !self.is_ancestor_of(blue_block, block).await?
            {
                count += 1;
            }
        }

        Ok(count)
    }

    /// Check if `ancestor` is an ancestor of `descendant`
    async fn is_ancestor_of(
        &self,
        ancestor: &Hash,
        descendant: &Hash,
    ) -> Result<bool, GhostDagError> {
        if ancestor == descendant {
            return Ok(true);
        }

        let relations = self.relations.read().await;

        // BFS to find ancestry
        let mut queue = VecDeque::new();
        let mut visited = HashSet::new();
        queue.push_back(*descendant);

        while let Some(current) = queue.pop_front() {
            if visited.contains(&current) {
                continue;
            }
            visited.insert(current);

            if let Some(relation) = relations.get(&current) {
                if relation.selected_parent == *ancestor {
                    return Ok(true);
                }
                if relation.merge_parents.contains(ancestor) {
                    return Ok(true);
                }

                // Add parents to queue
                queue.push_back(relation.selected_parent);
                for parent in &relation.merge_parents {
                    queue.push_back(*parent);
                }
            }
        }

        Ok(false)
    }

    /// Calculate blue score for a block
    pub async fn calculate_blue_score(&self, block: &Block) -> Result<u64, GhostDagError> {
        let blue_set = self.calculate_blue_set(block).await?;
        Ok(blue_set.score)
    }

    /// Get or calculate blue set for a block
    fn get_or_calculate_blue_set<'a>(
        &'a self,
        hash: &'a Hash,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<BlueSet, GhostDagError>> + Send + 'a>,
    > {
        Box::pin(async move {
            // Check cache first
            if let Some(cached) = self.blue_cache.read().await.get(hash) {
                return Ok(cached.clone());
            }

            // Fetch the block from DAG store
            let block = self
                .dag_store
                .get_block(hash)
                .await
                .map_err(|_| GhostDagError::BlockNotFound(*hash))?;

            // Genesis: blue set is itself
            if block.is_genesis() {
                let mut blue = BlueSet::new();
                blue.blocks.insert(*hash);
                blue.score = 1;
                self.blue_cache.write().await.insert(*hash, blue.clone());
                return Ok(blue);
            }

            // Recursively compose from selected parent and qualified merge parents
            let selected_parent_blue = self
                .get_or_calculate_blue_set(&block.selected_parent())
                .await?;

            // Determine blue merge parents using the same rule as top-level calculation
            let blue_merge_parents = self
                .calculate_blue_merge_parents(&block, &selected_parent_blue)
                .await?;

            let mut all_blocks = selected_parent_blue.blocks.clone();
            for p in &blue_merge_parents {
                let pset = self.get_or_calculate_blue_set(p).await?;
                all_blocks.extend(pset.blocks);
            }
            all_blocks.insert(*hash);

            let mut blue = BlueSet::new();
            blue.blocks = all_blocks;
            blue.score = blue.blocks.len() as u64;
            self.blue_cache.write().await.insert(*hash, blue.clone());
            Ok(blue)
        })
    }

    /// Add a block to the DAG
    pub async fn add_block(&self, block: &Block) -> Result<(), GhostDagError> {
        // Validate parent structure
        if !block.is_genesis() && block.selected_parent() == Hash::default() {
            return Err(GhostDagError::InvalidParents);
        }

        // Calculate blue set
        let blue_set = self.calculate_blue_set(block).await?;

        // Create DAG relation
        let relation = DagRelation {
            block: block.hash(),
            selected_parent: block.selected_parent(),
            merge_parents: block.header.merge_parent_hashes.clone(),
            children: Vec::new(),
            blue_set: blue_set.clone(),
            is_chain_block: true, // Will be determined by chain selection
        };

        // Update relations
        let mut relations = self.relations.write().await;
        relations.insert(block.hash(), relation);

        // Update parent's children
        if let Some(parent_relation) = relations.get_mut(&block.selected_parent()) {
            parent_relation.children.push(block.hash());
        }
        for merge_parent in &block.header.merge_parent_hashes {
            if let Some(parent_relation) = relations.get_mut(merge_parent) {
                parent_relation.children.push(block.hash());
            }
        }
        drop(relations);

        // Update tips
        let mut tips = self.tips.write().await;

        // Remove parents from tips
        tips.remove(&block.selected_parent());
        for merge_parent in &block.header.merge_parent_hashes {
            tips.remove(merge_parent);
        }

        // Add new block as tip
        tips.insert(block.hash());

        info!(
            "Added block {} to DAG with blue score {}",
            block.hash(),
            blue_set.score
        );
        Ok(())
    }

    /// Select the best tip based on blue score
    pub async fn select_tip(&self) -> Result<Hash, GhostDagError> {
        let tips = self.tips.read().await;
        let relations = self.relations.read().await;

        let mut best_tip = None;
        let mut best_score = 0;

        for tip in tips.iter() {
            if let Some(relation) = relations.get(tip) {
                if relation.blue_set.score > best_score {
                    best_score = relation.blue_set.score;
                    best_tip = Some(*tip);
                }
            }
        }

        best_tip.ok_or(GhostDagError::BlockNotFound(Hash::default()))
    }

    /// Get current tips
    pub async fn get_tips(&self) -> Vec<Hash> {
        self.tips.read().await.iter().copied().collect()
    }

    /// Get blue score for a block
    pub async fn get_blue_score(&self, hash: &Hash) -> Result<u64, GhostDagError> {
        self.relations
            .read()
            .await
            .get(hash)
            .map(|r| r.blue_set.score)
            .ok_or(GhostDagError::BlockNotFound(*hash))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::*;

    fn create_test_block_with_parents(
        hash: [u8; 32],
        selected_parent: Hash,
        merge_parents: Vec<Hash>,
        blue_score: u64,
    ) -> Block {
        Block {
            header: BlockHeader {
                version: 1,
                block_hash: Hash::new(hash),
                selected_parent_hash: selected_parent,
                merge_parent_hashes: merge_parents,
                timestamp: 0,
                height: 0,
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
    async fn test_genesis_block_blue_set() {
        let params = GhostDagParams::default();
        let dag_store = Arc::new(DagStore::new());
        let ghostdag = GhostDag::new(params, dag_store.clone());

        let genesis = create_test_block_with_parents([0; 32], Hash::default(), vec![], 0);

        // Store genesis in dag_store
        dag_store.store_block(genesis.clone()).await.unwrap();

        let blue_set = ghostdag.calculate_blue_set(&genesis).await.unwrap();
        assert_eq!(blue_set.score, 1);
        assert!(blue_set.contains(&genesis.hash()));
    }

    #[tokio::test]
    async fn test_add_block() {
        let params = GhostDagParams::default();
        let dag_store = Arc::new(DagStore::new());
        let ghostdag = GhostDag::new(params, dag_store.clone());

        // Add genesis
        let genesis = create_test_block_with_parents([0; 32], Hash::default(), vec![], 0);

        // Store genesis in dag_store
        dag_store.store_block(genesis.clone()).await.unwrap();

        // Manually add genesis to cache
        let mut blue_set = BlueSet::new();
        blue_set.insert(genesis.hash());
        ghostdag
            .blue_cache
            .write()
            .await
            .insert(genesis.hash(), blue_set);

        // Add genesis relation
        let genesis_relation = DagRelation {
            block: genesis.hash(),
            selected_parent: Hash::default(),
            merge_parents: vec![],
            children: vec![],
            blue_set: BlueSet::new(),
            is_chain_block: true,
        };
        ghostdag
            .relations
            .write()
            .await
            .insert(genesis.hash(), genesis_relation);

        // Add child block
        let block1 = create_test_block_with_parents([1; 32], genesis.hash(), vec![], 1);

        // Store block1 in dag_store
        dag_store.store_block(block1.clone()).await.unwrap();

        ghostdag.add_block(&block1).await.unwrap();

        // Verify tips
        let tips = ghostdag.get_tips().await;
        assert_eq!(tips.len(), 1);
        assert!(tips.contains(&block1.hash()));
    }

    #[tokio::test]
    async fn test_tip_selection() {
        let params = GhostDagParams::default();
        let dag_store = Arc::new(DagStore::new());
        let ghostdag = GhostDag::new(params, dag_store.clone());

        // Create simple chain
        let genesis = create_test_block_with_parents([0; 32], Hash::default(), vec![], 0);

        // Add genesis manually
        let mut genesis_blue = BlueSet::new();
        genesis_blue.insert(genesis.hash());
        genesis_blue.score = 1;

        let genesis_relation = DagRelation {
            block: genesis.hash(),
            selected_parent: Hash::default(),
            merge_parents: vec![],
            children: vec![],
            blue_set: genesis_blue.clone(),
            is_chain_block: true,
        };

        ghostdag
            .relations
            .write()
            .await
            .insert(genesis.hash(), genesis_relation);
        ghostdag
            .blue_cache
            .write()
            .await
            .insert(genesis.hash(), genesis_blue);
        ghostdag.tips.write().await.insert(genesis.hash());

        let best_tip = ghostdag.select_tip().await.unwrap();
        assert_eq!(best_tip, genesis.hash());
    }

    #[tokio::test]
    async fn test_blue_set_with_merge_parents_unions_parents() {
        let params = GhostDagParams::default();
        let dag_store = Arc::new(DagStore::new());
        let ghostdag = GhostDag::new(params, dag_store.clone());

        // Genesis
        let genesis = create_test_block_with_parents([0xAA; 32], Hash::default(), vec![], 0);
        dag_store.store_block(genesis.clone()).await.unwrap();

        // Prime relations/cache for genesis
        let mut gset = BlueSet::new();
        gset.insert(genesis.hash());
        ghostdag
            .blue_cache
            .write()
            .await
            .insert(genesis.hash(), gset.clone());
        let grel = DagRelation {
            block: genesis.hash(),
            selected_parent: Hash::default(),
            merge_parents: vec![],
            children: vec![],
            blue_set: gset,
            is_chain_block: true,
        };
        ghostdag
            .relations
            .write()
            .await
            .insert(genesis.hash(), grel);
        ghostdag.tips.write().await.insert(genesis.hash());

        // Two parallel children of genesis: b and c
        let b = create_test_block_with_parents([0xB1; 32], genesis.hash(), vec![], 1);
        let c = create_test_block_with_parents([0xC1; 32], genesis.hash(), vec![], 1);
        dag_store.store_block(b.clone()).await.unwrap();
        dag_store.store_block(c.clone()).await.unwrap();
        ghostdag.add_block(&b).await.unwrap();
        ghostdag.add_block(&c).await.unwrap();

        // d references b as selected parent and c as merge parent
        let d = create_test_block_with_parents([0xD1; 32], b.hash(), vec![c.hash()], 2);
        dag_store.store_block(d.clone()).await.unwrap();
        ghostdag.add_block(&d).await.unwrap();

        // Blue set for d should include at least {genesis, b, c, d}
        let blue = ghostdag.calculate_blue_set(&d).await.unwrap();
        assert!(blue.contains(&genesis.hash()));
        assert!(blue.contains(&b.hash()));
        assert!(blue.contains(&c.hash()));
        assert!(blue.contains(&d.hash()));
        assert!(blue.score >= 4);
    }
}
