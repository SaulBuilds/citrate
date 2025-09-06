use lattice_consensus::{Block, BlockHeader, Transaction, Hash, PublicKey, Signature};
use lattice_consensus::{GhostDag, TipSelector, ChainSelector};
use lattice_storage::StorageManager;
use lattice_execution::Executor;
use lattice_sequencer::mempool::Mempool;
use chrono::Utc;
use std::sync::Arc;
use tokio::time::{interval, Duration};
use tracing::{info, error, debug};

/// Block producer for mining new blocks
pub struct BlockProducer {
    storage: Arc<StorageManager>,
    executor: Arc<Executor>,
    mempool: Arc<Mempool>,
    ghostdag: Arc<GhostDag>,
    tip_selector: Arc<TipSelector>,
    chain_selector: Arc<ChainSelector>,
    coinbase: PublicKey,
    target_block_time: u64,
}

impl BlockProducer {
    pub fn new(
        storage: Arc<StorageManager>,
        executor: Arc<Executor>,
        mempool: Arc<Mempool>,
        coinbase: PublicKey,
        target_block_time: u64,
    ) -> Self {
        let ghostdag = Arc::new(GhostDag::new(18, storage.dag.clone()));
        let tip_selector = Arc::new(TipSelector::new(storage.chain.clone()));
        let chain_selector = Arc::new(ChainSelector::new(
            storage.chain.clone(),
            ghostdag.clone(),
        ));
        
        Self {
            storage,
            executor,
            mempool,
            ghostdag,
            tip_selector,
            chain_selector,
            coinbase,
            target_block_time,
        }
    }
    
    /// Start block production loop
    pub async fn start(self: Arc<Self>) {
        let mut interval = interval(Duration::from_secs(self.target_block_time));
        let mut block_count = 0u64;
        
        loop {
            interval.tick().await;
            
            match self.produce_block().await {
                Ok(block_hash) => {
                    block_count += 1;
                    info!(
                        "Produced block #{} hash={} txs={}",
                        block_count,
                        hex::encode(&block_hash.0[..8]),
                        0, // We'll add transaction count later
                    );
                }
                Err(e) => {
                    error!("Failed to produce block: {}", e);
                }
            }
        }
    }
    
    /// Produce a single block
    async fn produce_block(&self) -> anyhow::Result<Hash> {
        // Get current tips for parent selection
        let tips = self.tip_selector.get_tips().await;
        
        // Select parents using GhostDAG
        let parent_hashes = if tips.is_empty() {
            // If no tips, use genesis
            vec![self.storage.chain.get_genesis_hash()?
                .ok_or_else(|| anyhow::anyhow!("No genesis block"))?]
        } else {
            tips
        };
        
        // Get the selected parent (highest blue score)
        let selected_parent = if parent_hashes.is_empty() {
            None
        } else {
            Some(parent_hashes[0])
        };
        
        // Calculate blue and red sets
        let (blues, reds) = if let Some(parent) = selected_parent {
            self.ghostdag.calculate_blue_red_sets(&parent, &parent_hashes).await?
        } else {
            (vec![], vec![])
        };
        
        // Get last block height
        let last_height = if let Some(parent) = selected_parent {
            self.storage.chain.get_block_header(&parent)?
                .map(|h| h.height)
                .unwrap_or(0)
        } else {
            0
        };
        
        // Get transactions from mempool
        let transactions = self.mempool.get_best_transactions(100, 10_000_000).await;
        
        // Calculate blue score and work
        let (blue_score, blue_work) = self.calculate_dag_metrics(&parent_hashes).await?;
        
        // Create block header
        let mut header = BlockHeader {
            version: 1,
            block_hash: Hash::default(), // Will be computed
            parent_hashes: parent_hashes.clone(),
            height: last_height + 1,
            timestamp: Utc::now().timestamp() as u64,
            difficulty: 1, // Fixed difficulty for devnet
            nonce: 0,
            merkle_root: self.calculate_merkle_root(&transactions),
            state_root: self.executor.calculate_state_root(),
            receipts_root: Hash::default(), // TODO: Calculate from receipts
            proposer: self.coinbase,
            signature: Signature::new([1; 64]), // Dummy signature for devnet
            blue_score,
            blue_work,
            pruning_point: 0,
            selected_parent,
            blues,
            reds,
        };
        
        // Compute block hash
        header.block_hash = header.compute_hash();
        
        // Create block
        let block = Block {
            header: header.clone(),
            transactions,
            uncles: vec![],
        };
        
        // Store block
        self.storage.chain.put_block(&block)?;
        self.storage.chain.put_block_header(&header)?;
        
        // Update tips
        self.tip_selector.update_tips(vec![header.block_hash]).await;
        
        // Update DAG
        self.storage.dag.add_block(
            header.block_hash,
            parent_hashes,
            header.height,
            header.timestamp,
        )?;
        
        Ok(header.block_hash)
    }
    
    /// Calculate merkle root of transactions
    fn calculate_merkle_root(&self, transactions: &[Transaction]) -> Hash {
        if transactions.is_empty() {
            return Hash::default();
        }
        
        let hashes: Vec<Hash> = transactions.iter().map(|tx| tx.hash).collect();
        
        // Simple merkle root calculation
        if hashes.len() == 1 {
            hashes[0]
        } else {
            // For now, just hash all transaction hashes together
            let mut data = Vec::new();
            for hash in hashes {
                data.extend_from_slice(&hash.0);
            }
            Hash::hash(&data)
        }
    }
    
    /// Calculate DAG metrics
    async fn calculate_dag_metrics(&self, parent_hashes: &[Hash]) -> anyhow::Result<(u64, u128)> {
        if parent_hashes.is_empty() {
            return Ok((0, 0));
        }
        
        // Get parent with highest blue score
        let mut max_blue_score = 0u64;
        let mut total_work = 0u128;
        
        for parent_hash in parent_hashes {
            if let Some(header) = self.storage.chain.get_block_header(parent_hash)? {
                if header.blue_score > max_blue_score {
                    max_blue_score = header.blue_score;
                }
                total_work = total_work.saturating_add(header.blue_work);
            }
        }
        
        Ok((max_blue_score + 1, total_work + 1))
    }
}