use crate::mempool::{Mempool, TxClass};
use lattice_consensus::{
    Block, BlockHeader, Hash, PublicKey, Transaction, 
    VrfProof, GhostDagParams, Signature
};
use lattice_execution::parallel::ParallelExecutor;
use lattice_execution::executor::Executor;
use serde::{Deserialize, Serialize};
use sha3::{Digest, Sha3_256, Keccak256};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{debug, info};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum BlockBuilderError {
    #[error("No transactions available")]
    NoTransactions,
    
    #[error("Block size limit exceeded")]
    BlockSizeExceeded,
    
    #[error("Gas limit exceeded")]
    GasLimitExceeded,
    
    #[error("Invalid parent block")]
    InvalidParent,
    
    #[error("Builder not ready")]
    NotReady,
    
    #[error("State root calculation failed")]
    StateRootError,
}

/// Block builder configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockBuilderConfig {
    /// Maximum block size in bytes
    pub max_block_size: usize,
    
    /// Maximum gas per block
    pub max_gas_per_block: u64,
    
    /// Minimum transactions per block
    pub min_transactions: usize,
    
    /// Maximum transactions per block
    pub max_transactions: usize,
    
    /// Block time target in seconds
    pub block_time_target: u64,
    
    /// Enable transaction bundling
    pub enable_bundling: bool,
    
    /// Bundle size for similar transactions
    pub bundle_size: usize,
}

impl Default for BlockBuilderConfig {
    fn default() -> Self {
        Self {
            max_block_size: 1_000_000, // 1 MB
            max_gas_per_block: 30_000_000, // 30M gas
            min_transactions: 1,
            max_transactions: 5000,
            block_time_target: 2, // 2 seconds
            enable_bundling: true,
            bundle_size: 10,
        }
    }
}

/// Transaction bundle for grouped execution
#[derive(Debug, Clone)]
pub struct TxBundle {
    pub class: TxClass,
    pub transactions: Vec<Transaction>,
    pub total_gas: u64,
    pub total_fees: u128,
}

impl TxBundle {
    pub fn new(class: TxClass) -> Self {
        Self {
            class,
            transactions: Vec::new(),
            total_gas: 0,
            total_fees: 0,
        }
    }
    
    pub fn add_transaction(&mut self, tx: Transaction) {
        self.total_gas += tx.gas_limit;
        self.total_fees += (tx.gas_price * tx.gas_limit) as u128;
        self.transactions.push(tx);
    }
    
    pub fn is_full(&self, max_size: usize) -> bool {
        self.transactions.len() >= max_size
    }
}

/// Block builder for assembling new blocks
pub struct BlockBuilder {
    config: BlockBuilderConfig,
    mempool: Arc<Mempool>,
    proposer_key: PublicKey,
    executor: Option<Arc<Executor>>,
    parallel_executor: Arc<ParallelExecutor>,
}

impl BlockBuilder {
    pub fn new(
        config: BlockBuilderConfig,
        mempool: Arc<Mempool>,
        proposer_key: PublicKey,
    ) -> Self {
        Self {
            config,
            mempool,
            proposer_key,
            executor: None,
            parallel_executor: Arc::new(ParallelExecutor::new()),
        }
    }
    
    /// Set the executor for parallel transaction execution
    pub fn with_executor(mut self, executor: Arc<Executor>) -> Self {
        self.executor = Some(executor);
        self
    }
    
    /// Build a new block
    pub async fn build_block(
        &self,
        selected_parent: Hash,
        merge_parents: Vec<Hash>,
        parent_height: u64,
        parent_blue_score: u64,
        vrf_proof: VrfProof,
    ) -> Result<Block, BlockBuilderError> {
        info!("Building new block with parent {}", selected_parent);
        
        // Get transactions from mempool
        let transactions = self.select_transactions().await?;
        
        if transactions.is_empty() && self.config.min_transactions > 0 {
            return Err(BlockBuilderError::NoTransactions);
        }
        
        // Calculate roots
        let tx_root = self.calculate_tx_root(&transactions);
        let state_root = self.calculate_state_root(&transactions)?;
        let receipt_root = self.calculate_receipt_root(&transactions);
        let artifact_root = Hash::default(); // Placeholder for AI artifacts
        
        // Build block header
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let header = BlockHeader {
            version: 1,
            block_hash: Hash::default(), // Will be calculated after
            selected_parent_hash: selected_parent,
            merge_parent_hashes: merge_parents,
            timestamp,
            height: parent_height + 1,
            blue_score: parent_blue_score + 1, // Will be properly calculated by consensus
            blue_work: 0, // Will be calculated by consensus
            pruning_point: Hash::default(),
            proposer_pubkey: self.proposer_key,
            vrf_reveal: vrf_proof,
        };
        
        // Create block
        let mut block = Block {
            header,
            state_root,
            tx_root,
            receipt_root,
            artifact_root,
            ghostdag_params: GhostDagParams::default(),
            transactions,
            signature: Signature::new([1; 64]), // Placeholder signature
        };
        
        // Calculate block hash
        block.header.block_hash = self.calculate_block_hash(&block);
        
        info!(
            "Built block {} at height {} with {} transactions",
            block.header.block_hash,
            block.header.height,
            block.transactions.len()
        );
        
        Ok(block)
    }
    
    /// Select transactions for inclusion
    async fn select_transactions(&self) -> Result<Vec<Transaction>, BlockBuilderError> {
        let max_gas = self.config.max_gas_per_block;
        let max_size = self.config.max_block_size;
        let max_count = self.config.max_transactions;
        
        // Get best transactions from mempool
        let transactions = self.mempool
            .get_best_transactions(max_count, max_size)
            .await;
        
        // Apply gas limit
        let mut total_gas = 0;
        let mut selected = Vec::new();
        
        for tx in transactions {
            if total_gas + tx.gas_limit > max_gas {
                break;
            }
            total_gas += tx.gas_limit;
            selected.push(tx);
        }
        
        debug!("Selected {} transactions with total gas {}", selected.len(), total_gas);
        
        Ok(selected)
    }
    
    /// Bundle transactions by class
    pub async fn bundle_transactions(&self) -> Result<Vec<TxBundle>, BlockBuilderError> {
        if !self.config.enable_bundling {
            return Ok(Vec::new());
        }
        
        let transactions = self.select_transactions().await?;
        let mut bundles: Vec<TxBundle> = Vec::new();
        
        for tx in transactions {
            // Determine transaction class (simplified - would need actual analysis)
            let class = self.classify_transaction(&tx);
            
            // Find or create bundle for this class
            // Find existing bundle or create new one
            let needs_new = !bundles
                .iter()
                .any(|b| b.class == class && !b.is_full(self.config.bundle_size));
            
            if needs_new {
                let new_bundle = TxBundle::new(class);
                bundles.push(new_bundle);
            }
            
            let bundle = bundles
                .iter_mut()
                .find(|b| b.class == class && !b.is_full(self.config.bundle_size))
                .unwrap();
            
            bundle.add_transaction(tx);
        }
        
        info!("Created {} transaction bundles", bundles.len());
        
        Ok(bundles)
    }
    
    /// Classify transaction into a class
    fn classify_transaction(&self, tx: &Transaction) -> TxClass {
        // Simplified classification based on data field
        // In production, would analyze the actual call data
        
        if tx.data.is_empty() {
            TxClass::Standard
        } else if tx.data.len() > 10000 {
            TxClass::ModelUpdate
        } else if tx.data.starts_with(b"inference") {
            TxClass::Inference
        } else if tx.data.starts_with(b"training") {
            TxClass::Training
        } else if tx.data.starts_with(b"storage") {
            TxClass::Storage
        } else {
            TxClass::Standard
        }
    }
    
    /// Calculate transaction root
    fn calculate_tx_root(&self, transactions: &[Transaction]) -> Hash {
        // Use Keccak-256 for transaction root to align with tx.hash generation
        let mut hasher = Keccak256::new();
        
        for tx in transactions {
            hasher.update(tx.hash.as_bytes());
        }
        
        Hash::from_bytes(&hasher.finalize())
    }
    
    /// Calculate state root (placeholder)
    fn calculate_state_root(&self, _transactions: &[Transaction]) -> Result<Hash, BlockBuilderError> {
        // In production, this would calculate the actual state root
        // after applying transactions to the state
        Ok(Hash::new([1; 32]))
    }
    
    /// Calculate receipt root (placeholder)
    fn calculate_receipt_root(&self, _transactions: &[Transaction]) -> Hash {
        // In production, this would calculate receipt root from execution receipts
        Hash::new([2; 32])
    }
    
    /// Calculate block hash
    fn calculate_block_hash(&self, block: &Block) -> Hash {
        let mut hasher = Sha3_256::new();
        
        // Hash header fields
        hasher.update(&block.header.version.to_le_bytes());
        hasher.update(block.header.selected_parent_hash.as_bytes());
        for parent in &block.header.merge_parent_hashes {
            hasher.update(parent.as_bytes());
        }
        hasher.update(&block.header.timestamp.to_le_bytes());
        hasher.update(&block.header.height.to_le_bytes());
        hasher.update(&block.header.blue_score.to_le_bytes());
        
        // Hash content roots
        hasher.update(block.state_root.as_bytes());
        hasher.update(block.tx_root.as_bytes());
        hasher.update(block.receipt_root.as_bytes());
        hasher.update(block.artifact_root.as_bytes());
        
        Hash::from_bytes(&hasher.finalize())
    }
    
    /// Validate a block before proposing
    pub fn validate_block(&self, block: &Block) -> Result<(), BlockBuilderError> {
        // Check block size
        let block_size = self.estimate_block_size(block);
        if block_size > self.config.max_block_size {
            return Err(BlockBuilderError::BlockSizeExceeded);
        }
        
        // Check gas limit
        let total_gas: u64 = block.transactions.iter().map(|tx| tx.gas_limit).sum();
        if total_gas > self.config.max_gas_per_block {
            return Err(BlockBuilderError::GasLimitExceeded);
        }
        
        // Check transaction count
        if block.transactions.len() > self.config.max_transactions {
            return Err(BlockBuilderError::BlockSizeExceeded);
        }
        
        Ok(())
    }
    
    /// Estimate block size in bytes
    fn estimate_block_size(&self, block: &Block) -> usize {
        // Header size estimate
        let mut size = 200; // Approximate header size
        
        // Add transaction sizes
        for tx in &block.transactions {
            size += 32 + 8 + 32 + 32 + 16 + 8 + 8 + tx.data.len() + 64;
        }
        
        size
    }
}

/// Block template for mining/proposing
#[derive(Debug, Clone)]
pub struct BlockTemplate {
    pub block: Block,
    pub transactions: Vec<Transaction>,
    pub fees: u128,
    pub timestamp: u64,
}

impl BlockTemplate {
    pub fn new(block: Block) -> Self {
        let fees = block.transactions
            .iter()
            .map(|tx| (tx.gas_price * tx.gas_limit) as u128)
            .sum();
        
        Self {
            transactions: block.transactions.clone(),
            timestamp: block.header.timestamp,
            block,
            fees,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mempool::MempoolConfig;
    
    async fn setup_test_builder() -> (BlockBuilder, Arc<Mempool>) {
        let config = BlockBuilderConfig::default();
        let mempool_config = MempoolConfig::default();
        let mempool = Arc::new(Mempool::new(mempool_config));
        let proposer = PublicKey::new([1; 32]);
        
        let builder = BlockBuilder::new(config, mempool.clone(), proposer);
        (builder, mempool)
    }
    
    fn create_test_tx(nonce: u64, gas_price: u64) -> Transaction {
        Transaction {
            hash: Hash::new([nonce as u8; 32]),
            nonce,
            from: PublicKey::new([1; 32]),
            to: Some(PublicKey::new([2; 32])),
            value: 1000,
            gas_limit: 21000,
            gas_price,
            data: vec![],
            signature: Signature::new([1; 64]), // Non-zero signature for tests
        }
    }
    
    #[tokio::test]
    async fn test_build_empty_block() {
        let (builder, _mempool) = setup_test_builder().await;
        
        let parent = Hash::new([0xFF; 32]);
        let vrf_proof = VrfProof {
            proof: vec![0; 32],
            output: Hash::new([0; 32]),
        };
        
        // Allow empty blocks for testing
        let mut config = builder.config.clone();
        config.min_transactions = 0;
        let builder = BlockBuilder::new(config, builder.mempool.clone(), builder.proposer_key);
        
        let block = builder.build_block(
            parent,
            vec![],
            0,
            1,
            vrf_proof,
        ).await.unwrap();
        
        assert_eq!(block.header.height, 1);
        assert_eq!(block.header.selected_parent_hash, parent);
        assert_eq!(block.transactions.len(), 0);
    }
    
    #[tokio::test]
    async fn test_build_block_with_transactions() {
        let (builder, mempool) = setup_test_builder().await;
        
        // Add transactions to mempool
        for i in 0..5 {
            let tx = create_test_tx(i, 2_000_000_000);
            mempool.add_transaction(tx, TxClass::Standard).await.unwrap();
        }
        
        let parent = Hash::new([0xFF; 32]);
        let vrf_proof = VrfProof {
            proof: vec![0; 32],
            output: Hash::new([0; 32]),
        };
        
        let block = builder.build_block(
            parent,
            vec![],
            0,
            1,
            vrf_proof,
        ).await.unwrap();
        
        assert_eq!(block.header.height, 1);
        assert_eq!(block.transactions.len(), 5);
    }
    
    #[tokio::test]
    async fn test_transaction_bundling() {
        let (builder, mempool) = setup_test_builder().await;
        
        // Add various transaction types
        let mut tx1 = create_test_tx(0, 2_000_000_000);
        tx1.data = b"inference".to_vec();
        
        let mut tx2 = create_test_tx(1, 2_000_000_000);
        tx2.data = b"inference".to_vec();
        
        let tx3 = create_test_tx(2, 2_000_000_000);
        
        mempool.add_transaction(tx1, TxClass::Inference).await.unwrap();
        mempool.add_transaction(tx2, TxClass::Inference).await.unwrap();
        mempool.add_transaction(tx3, TxClass::Standard).await.unwrap();
        
        let bundles = builder.bundle_transactions().await.unwrap();
        
        // Should have bundles for different classes
        assert!(bundles.len() >= 1);
        
        // Check bundle grouping
        for bundle in bundles {
            for tx in &bundle.transactions {
                let class = builder.classify_transaction(tx);
                assert_eq!(class, bundle.class);
            }
        }
    }
    
    #[tokio::test]
    async fn test_block_validation() {
        let (builder, _) = setup_test_builder().await;
        
        let mut block = Block {
            header: BlockHeader {
                version: 1,
                block_hash: Hash::default(),
                selected_parent_hash: Hash::default(),
                merge_parent_hashes: vec![],
                timestamp: 0,
                height: 1,
                blue_score: 1,
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
            signature: Signature::new([1; 64]), // Non-zero signature for tests
        };
        
        // Valid block
        assert!(builder.validate_block(&block).is_ok());
        
        // Add too many transactions
        for i in 0..6000 {
            block.transactions.push(create_test_tx(i, 1_000_000_000));
        }
        
        assert!(matches!(
            builder.validate_block(&block),
            Err(BlockBuilderError::BlockSizeExceeded)
        ));
    }
    
    #[tokio::test]
    async fn test_gas_limit() {
        let (builder, mempool) = setup_test_builder().await;
        
        // Add transactions that exceed gas limit (use different senders to avoid sender limit)
        for i in 0..200 {
            let mut tx = create_test_tx(0, 2_000_000_000);
            tx.from = PublicKey::new([(i % 256) as u8; 32]); // Different sender for each tx
            tx.hash = Hash::new([i as u8; 32]); // Unique hash
            tx.gas_limit = 50000; // High gas usage
            let _ = mempool.add_transaction(tx, TxClass::Standard).await;
        }
        
        let parent = Hash::new([0xFF; 32]);
        let vrf_proof = VrfProof {
            proof: vec![0; 32],
            output: Hash::new([0; 32]),
        };
        
        let block = builder.build_block(
            parent,
            vec![],
            0,
            1,
            vrf_proof,
        ).await.unwrap();
        
        // Should respect gas limit
        let total_gas: u64 = block.transactions.iter().map(|tx| tx.gas_limit).sum();
        assert!(total_gas <= builder.config.max_gas_per_block);
    }
}
