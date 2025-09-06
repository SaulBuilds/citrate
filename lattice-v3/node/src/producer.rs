use lattice_consensus::types::{Block, BlockHeader, Transaction, Hash, PublicKey, Signature, GhostDagParams, VrfProof};
use lattice_consensus::ghostdag::GhostDag;
use lattice_consensus::tip_selection::TipSelector;
use lattice_consensus::chain_selection::ChainSelector;
use lattice_consensus::dag_store::DagStore;
use lattice_storage::StorageManager;
use lattice_execution::Executor;
use lattice_sequencer::mempool::Mempool;
use sha3::{Digest, Sha3_256};
use std::sync::Arc;
use tokio::time::{interval, Duration};
use tracing::{info, error};

/// Calculate block header hash using SHA3-256
fn calculate_block_hash_header(header: &BlockHeader) -> Hash {
    let mut hasher = Sha3_256::new();
    
    // Hash header fields
    hasher.update(&header.version.to_le_bytes());
    hasher.update(header.selected_parent_hash.as_bytes());
    for parent in &header.merge_parent_hashes {
        hasher.update(parent.as_bytes());
    }
    hasher.update(&header.timestamp.to_le_bytes());
    hasher.update(&header.height.to_le_bytes());
    hasher.update(&header.blue_score.to_le_bytes());
    hasher.update(&header.blue_work.to_le_bytes());
    hasher.update(header.pruning_point.as_bytes());
    
    let hash_bytes = hasher.finalize();
    let mut hash_array = [0u8; 32];
    hash_array.copy_from_slice(&hash_bytes[..32]);
    Hash::new(hash_array)
}

/// Block producer for mining new blocks
pub struct BlockProducer {
    storage: Arc<StorageManager>,
    executor: Arc<Executor>,
    mempool: Arc<Mempool>,
    dag_store: Arc<DagStore>,
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
        // Create consensus components with a new DAG store
        let dag_store = Arc::new(DagStore::new());
        let _chain_store = storage.blocks.clone();
        
        let ghostdag = Arc::new(GhostDag::new(GhostDagParams::default(), dag_store.clone()));
        let tip_selector = Arc::new(TipSelector::new(
            dag_store.clone(),
            ghostdag.clone(),
            lattice_consensus::tip_selection::SelectionStrategy::HighestBlueScore,
        ));
        let chain_selector = Arc::new(ChainSelector::new(
            dag_store.clone(),
            ghostdag.clone(),
            tip_selector.clone(),
            100, // finality depth
        ));
        
        Self {
            storage,
            executor,
            mempool,
            dag_store,
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
                        hex::encode(&block_hash.as_bytes()[..8]),
                        0, // We'll get tx count from block
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
        let tips = self.dag_store.get_tips().await;
        
        // Select parents using GhostDAG
        let parent_hashes = if tips.is_empty() {
            // If no tips, use a default genesis hash
            // TODO: Load actual genesis hash from storage
            vec![Hash::default()]
        } else {
            // Convert tips to hashes
            tips.into_iter().map(|tip| tip.hash).collect()
        };
        
        // Get the selected parent (highest blue score)
        let selected_parent = if parent_hashes.is_empty() {
            None
        } else {
            Some(parent_hashes[0])
        };
        
        // For now, we'll use empty blue/red sets
        // TODO: Implement proper GhostDAG blue/red set calculation
        let blues: Vec<Hash> = vec![];
        let reds: Vec<Hash> = vec![];
        
        // Get last block height
        let last_height = if let Some(parent) = selected_parent {
            // TODO: Get actual parent height from storage
            0
        } else {
            0
        };
        
        // Get transactions from mempool
        let transactions = self.mempool.get_best_transactions(100, 10_000_000).await;
        
        // Calculate blue score and work (simplified for now)
        let blue_score = 0u64;
        let blue_work = 0u128;
        
        // Create block header
        let mut header = BlockHeader {
            version: 1,
            block_hash: Hash::default(), // Will be computed
            selected_parent_hash: selected_parent.unwrap_or_default(),
            merge_parent_hashes: if selected_parent.is_some() {
                parent_hashes[1..].to_vec()
            } else {
                vec![]
            },
            timestamp: chrono::Utc::now().timestamp() as u64,
            height: last_height + 1,
            blue_score,
            blue_work,
            pruning_point: Hash::default(),
            proposer_pubkey: self.coinbase,
            vrf_reveal: VrfProof {
                proof: vec![],
                output: Hash::default(),
            },
        };
        
        // Compute block hash (simplified)
        header.block_hash = calculate_block_hash_header(&header);
        
        // Create block
        let block = Block {
            header: header.clone(),
            state_root: Hash::default(), // TODO: Calculate actual state root
            tx_root: Hash::default(), // TODO: Calculate actual tx root
            receipt_root: Hash::default(), // TODO: Calculate actual receipt root
            artifact_root: Hash::default(), // TODO: Calculate actual artifact root
            ghostdag_params: GhostDagParams::default(),
            transactions,
            signature: Signature::new([1; 64]), // Dummy signature for devnet
        };
        
        // Store block
        self.storage.blocks.put_block(&block)?;
        
        // Update DAG store
        self.dag_store.store_block(block.clone()).await?;
        
        Ok(header.block_hash)
    }
}