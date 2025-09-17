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
        let mempool_config = MempoolConfig { require_valid_signature: false, ..Default::default() };
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
            tx_type: None,
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
    async fn test_bundling_preserves_priority_order_within_class() {
        let (builder, mempool) = setup_test_builder().await;

        // Use high/low gas price to influence priority order
        let mut tx_high = create_test_tx(0, 50_000_000_000);
        tx_high.data = b"inference".to_vec();
        let mut tx_low = create_test_tx(1, 2_000_000_000);
        tx_low.data = b"inference".to_vec();

        mempool.add_transaction(tx_high.clone(), TxClass::Inference).await.unwrap();
        mempool.add_transaction(tx_low.clone(), TxClass::Inference).await.unwrap();

        let bundles = builder.bundle_transactions().await.unwrap();
        let inf = bundles.into_iter().find(|b| b.class == TxClass::Inference).unwrap();
        assert!(inf.transactions.len() >= 2);
        // Expect higher gas price first given mempool priority ordering
        assert!(inf.transactions[0].gas_price >= inf.transactions[1].gas_price);
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
    async fn test_gas_cap_prioritizes_high_class() {
        let (base_builder, mempool) = setup_test_builder().await;
        // Tight gas cap: only 2 tx of 21k each will fit
        let mut cfg = base_builder.config.clone();
        cfg.max_gas_per_block = 42_000;
        let builder = BlockBuilder::new(cfg, mempool.clone(), base_builder.proposer_key);

        // Two System txs, two Standard txs with identical gas/price
        let mut sys1 = create_test_tx(0, 1_000_000_000);
        sys1.hash = Hash::new([0xA1; 32]);
        let mut sys2 = create_test_tx(1, 1_000_000_000);
        sys2.hash = Hash::new([0xA2; 32]);
        let mut std1 = create_test_tx(0, 1_000_000_000);
        std1.from = PublicKey::new([5; 32]);
        std1.hash = Hash::new([0xB1; 32]);
        let mut std2 = create_test_tx(1, 1_000_000_000);
        std2.from = PublicKey::new([6; 32]);
        std2.hash = Hash::new([0xB2; 32]);

        mempool.add_transaction(sys1.clone(), TxClass::System).await.unwrap();
        mempool.add_transaction(sys2.clone(), TxClass::System).await.unwrap();
        mempool.add_transaction(std1.clone(), TxClass::Standard).await.unwrap();
        mempool.add_transaction(std2.clone(), TxClass::Standard).await.unwrap();

        let parent = Hash::new([0xCC; 32]);
        let vrf = VrfProof { proof: vec![0; 32], output: Hash::new([0; 32]) };
        let block = builder.build_block(parent, vec![], 0, 1, vrf).await.unwrap();

        // Only the two System txs should be included under the gas cap
        assert_eq!(block.transactions.len(), 2);
        let included: Vec<Hash> = block.transactions.iter().map(|t| t.hash).collect();
        assert!(included.contains(&sys1.hash));
        assert!(included.contains(&sys2.hash));
    }

    #[tokio::test]
    async fn test_multiclass_block_selection_order() {
        let (builder0, mempool) = setup_test_builder().await;
        let parent = Hash::new([0xDD; 32]);
        let vrf = VrfProof { proof: vec![0; 32], output: Hash::new([0; 32]) };

        // Five txs from distinct senders, same gas price/limit, different classes
        let mk = |nonce: u64, from_b: u8, hash_b: u8| {
            let mut t = create_test_tx(nonce, 1_000_000_000);
            t.from = PublicKey::new([from_b; 32]);
            t.hash = Hash::new([hash_b; 32]);
            t
        };
        let sys = mk(0, 1, 0xC1);
        let mu = mk(0, 2, 0xC2);
        let comp = mk(0, 3, 0xC3);
        let inf = mk(0, 4, 0xC4);
        let std = mk(0, 5, 0xC5);

        mempool.add_transaction(sys.clone(), TxClass::System).await.unwrap();
        mempool.add_transaction(mu.clone(), TxClass::ModelUpdate).await.unwrap();
        mempool.add_transaction(comp.clone(), TxClass::Compute).await.unwrap();
        mempool.add_transaction(inf.clone(), TxClass::Inference).await.unwrap();
        mempool.add_transaction(std.clone(), TxClass::Standard).await.unwrap();

        let block = builder0.build_block(parent, vec![], 0, 1, vrf).await.unwrap();

        let hashes: Vec<Hash> = block.transactions.iter().map(|t| t.hash).collect();
        // Expect order by class priority at equal gas price: System > ModelUpdate > Compute > Inference > Standard
        assert_eq!(hashes[0], sys.hash);
        assert_eq!(hashes[1], mu.hash);
        assert_eq!(hashes[2], comp.hash);
        assert_eq!(hashes[3], inf.hash);
        assert_eq!(hashes[4], std.hash);
    }

    #[tokio::test]
    async fn test_block_size_cap_limits_selection() {
        let (builder0, mempool) = setup_test_builder().await;
        let mut cfg = builder0.config.clone();
        // Set very small block size so only first two small txs fit
        cfg.max_block_size = 400; // bytes
        let builder = BlockBuilder::new(cfg, mempool.clone(), builder0.proposer_key);

        // Create 4 txs: two small (empty data), two large (big data payload)
        let mk = |nonce: u64, from_b: u8, data_len: usize, hash_b: u8| {
            let mut t = create_test_tx(nonce, 1_000_000_000);
            t.from = PublicKey::new([from_b; 32]);
            t.hash = Hash::new([hash_b; 32]);
            t.data = vec![0xAA; data_len];
            t
        };
        let small1 = mk(0, 10, 0, 0xD1);
        let small2 = mk(0, 11, 0, 0xD2);
        let large1 = mk(0, 12, 1024, 0xD3);
        let large2 = mk(0, 13, 2048, 0xD4);

        mempool.add_transaction(small1.clone(), TxClass::Standard).await.unwrap();
        mempool.add_transaction(small2.clone(), TxClass::Standard).await.unwrap();
        mempool.add_transaction(large1.clone(), TxClass::Standard).await.unwrap();
        mempool.add_transaction(large2.clone(), TxClass::Standard).await.unwrap();

        let parent = Hash::new([0xEE; 32]);
        let vrf = VrfProof { proof: vec![0; 32], output: Hash::new([0; 32]) };
        let block = builder.build_block(parent, vec![], 0, 1, vrf).await.unwrap();

        // Due to size cap, only the two small txs should be included
        assert_eq!(block.transactions.len(), 2);
        let included: Vec<Hash> = block.transactions.iter().map(|t| t.hash).collect();
        assert!(included.contains(&small1.hash));
        assert!(included.contains(&small2.hash));
        assert!(!included.contains(&large1.hash));
        assert!(!included.contains(&large2.hash));
    }

    #[tokio::test]
    async fn test_gas_and_size_caps_with_class_priority() {
        let (builder0, mempool) = setup_test_builder().await;
        let mut cfg = builder0.config.clone();
        cfg.max_block_size = 500; // bytes
        cfg.max_gas_per_block = 21_000; // allow only one tx by gas
        let builder = BlockBuilder::new(cfg, mempool.clone(), builder0.proposer_key);

        // Two transactions with same gas/price, different classes
        let mut std_tx = create_test_tx(0, 1_000_000_000);
        std_tx.from = PublicKey::new([30; 32]);
        std_tx.hash = Hash::new([0xE1; 32]);
        let mut sys_tx = create_test_tx(0, 1_000_000_000);
        sys_tx.from = PublicKey::new([31; 32]);
        sys_tx.hash = Hash::new([0xE2; 32]);

        mempool.add_transaction(std_tx.clone(), TxClass::Standard).await.unwrap();
        mempool.add_transaction(sys_tx.clone(), TxClass::System).await.unwrap();

        let parent = Hash::new([0xEF; 32]);
        let vrf = VrfProof { proof: vec![0; 32], output: Hash::new([0; 32]) };
        let block = builder.build_block(parent, vec![], 0, 1, vrf).await.unwrap();

        assert_eq!(block.transactions.len(), 1);
        assert_eq!(block.transactions[0].hash, sys_tx.hash);
    }

    #[tokio::test]
    async fn test_bundle_boundary_class_order_under_gas_cap() {
        let (builder0, mempool) = setup_test_builder().await;
        let mut cfg = builder0.config.clone();
        cfg.bundle_size = 3;
        // Allow only 3 txs by gas
        cfg.max_gas_per_block = 63_000; // 3 * 21000
        let builder = BlockBuilder::new(cfg, mempool.clone(), builder0.proposer_key);

        // Insert 6 txs across classes, distinct senders
        let mk = |nonce: u64, from_b: u8, class: TxClass, hash_b: u8| {
            let mut t = create_test_tx(nonce, 1_000_000_000);
            t.from = PublicKey::new([from_b; 32]);
            t.hash = Hash::new([hash_b; 32]);
            (t, class)
        };
        let items = vec![
            mk(0, 1, TxClass::Inference, 0xF1),
            mk(0, 2, TxClass::Standard, 0xF2),
            mk(0, 3, TxClass::Compute, 0xF3),
            mk(0, 4, TxClass::ModelUpdate, 0xF4),
            mk(0, 5, TxClass::System, 0xF5),
            mk(0, 6, TxClass::Standard, 0xF6),
        ];
        for (t, c) in items.clone() { mempool.add_transaction(t, c).await.unwrap(); }

        let parent = Hash::new([0xFA; 32]);
        let vrf = VrfProof { proof: vec![0; 32], output: Hash::new([0; 32]) };
        let block = builder.build_block(parent, vec![], 0, 1, vrf).await.unwrap();

        assert_eq!(block.transactions.len(), 3);
        let hashes: Vec<Hash> = block.transactions.iter().map(|t| t.hash).collect();
        // Expect top 3 classes: System, ModelUpdate, Compute (in that order)
        let get_hash = |b: u8| Hash::new([b; 32]);
        assert_eq!(hashes[0], get_hash(0xF5));
        assert_eq!(hashes[1], get_hash(0xF4));
        assert_eq!(hashes[2], get_hash(0xF3));
    }

    #[tokio::test]
    async fn test_nonce_ordering_overrides_gas_price_for_same_sender() {
        let (builder0, mempool) = setup_test_builder().await;
        let parent = Hash::new([0xBA; 32]);
        let vrf = VrfProof { proof: vec![0; 32], output: Hash::new([0; 32]) };

        // Same sender: nonce 1 has higher gas price than nonce 0
        let sender = [0xCC; 32];
        let mut tx0 = create_test_tx(0, 1_000_000_000);
        tx0.from = PublicKey::new(sender);
        tx0.hash = Hash::new([0xA0; 32]);
        let mut tx1 = create_test_tx(1, 50_000_000_000);
        tx1.from = PublicKey::new(sender);
        tx1.hash = Hash::new([0xA1; 32]);

        mempool.add_transaction(tx0.clone(), TxClass::Standard).await.unwrap();
        mempool.add_transaction(tx1.clone(), TxClass::Standard).await.unwrap();

        // Build with gas cap allowing both; ensure generous limits
        let mut cfg = builder0.config.clone();
        cfg.max_gas_per_block = 100_000; // allow at least two standard txs
        cfg.max_transactions = 10;
        cfg.max_block_size = 1_000_000; // ample size to include both
        cfg.enable_bundling = false; // disable bundling to avoid class grouping side-effects
        let builder = BlockBuilder::new(cfg, mempool.clone(), builder0.proposer_key);
        let block = builder.build_block(parent, vec![], 0, 1, vrf).await.unwrap();
        assert!(block.transactions.len() >= 2);
        // The lower nonce must appear before the higher nonce
        let i0 = block.transactions.iter().position(|t| t.hash == tx0.hash).unwrap();
        let i1 = block.transactions.iter().position(|t| t.hash == tx1.hash).unwrap();
        assert!(i0 < i1);
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

    #[tokio::test]
    async fn test_large_batch_inference_ordering_across_bundles() {
        // Configure small bundle size to force multiple bundles
        let (mut builder0, mempool) = setup_test_builder().await;
        let mut cfg = builder0.config.clone();
        cfg.bundle_size = 5;
        let builder = BlockBuilder::new(cfg, mempool.clone(), builder0.proposer_key);

        // Insert 13 inference-class transactions with descending gas prices
        for i in 0..13u64 {
            let mut tx = create_test_tx(i, 100_000_000_000 - (i as u64) * 1_000_000);
            tx.data = b"inference".to_vec(); // Ensure classified as Inference
            // Unique sender per tx to avoid nonce constraints affecting order
            tx.from = PublicKey::new([i as u8 + 1; 32]);
            tx.hash = Hash::new([i as u8; 32]);
            mempool.add_transaction(tx, TxClass::Inference).await.unwrap();
        }

        let bundles = builder.bundle_transactions().await.unwrap();
        let mut inference_txs = Vec::new();
        for b in bundles.into_iter().filter(|b| b.class == TxClass::Inference) {
            inference_txs.extend(b.transactions);
        }
        assert_eq!(inference_txs.len(), 13);
        // Verify non-increasing gas price order across concatenated bundles
        for i in 1..inference_txs.len() {
            assert!(inference_txs[i - 1].gas_price >= inference_txs[i].gas_price);
        }
    }

    #[tokio::test]
    async fn test_standard_nonce_ordering_within_bundles() {
        let (mut builder0, mempool) = setup_test_builder().await;
        let mut cfg = builder0.config.clone();
        cfg.bundle_size = 3;
        let builder = BlockBuilder::new(cfg, mempool.clone(), builder0.proposer_key);

        // Same sender, nonces 0..5, varied gas prices; must respect nonce order
        let sender = [7u8; 32];
        for n in 0..6u64 {
            let mut tx = create_test_tx(n, 1_000_000_000 + (5 - n) * 100_000);
            tx.from = PublicKey::new(sender);
            tx.hash = Hash::new([n as u8; 32]);
            mempool.add_transaction(tx, TxClass::Standard).await.unwrap();
        }

        let bundles = builder.bundle_transactions().await.unwrap();
        let mut standard = Vec::new();
        for b in bundles.into_iter().filter(|b| b.class == TxClass::Standard) {
            standard.extend(b.transactions);
        }
        assert_eq!(standard.len(), 6);
        // Nonces must be strictly increasing
        for i in 1..standard.len() {
            assert_eq!(standard[i].nonce, standard[i - 1].nonce + 1);
        }
    }
    #[tokio::test]
    async fn test_exact_gas_limit_accepted() {
        let (builder0, mempool) = setup_test_builder().await;
        let mut cfg = builder0.config.clone();
        cfg.max_gas_per_block = 100_000; // Small custom limit
        let builder = BlockBuilder::new(cfg, mempool.clone(), builder0.proposer_key);

        // Two txs summing exactly to the block gas limit
        let mut tx1 = create_test_tx(0, 2_000_000_000);
        tx1.from = PublicKey::new([9; 32]);
        tx1.gas_limit = 60_000;
        let mut tx2 = create_test_tx(1, 2_000_000_000);
        tx2.from = PublicKey::new([8; 32]);
        tx2.gas_limit = 40_000;
        mempool.add_transaction(tx1, TxClass::Standard).await.unwrap();
        mempool.add_transaction(tx2, TxClass::Standard).await.unwrap();

        let parent = Hash::new([0xAA; 32]);
        let vrf_proof = VrfProof { proof: vec![0; 32], output: Hash::new([0; 32]) };

        let block = builder.build_block(parent, vec![], 0, 1, vrf_proof).await.unwrap();
        let total_gas: u64 = block.transactions.iter().map(|tx| tx.gas_limit).sum();
        assert_eq!(total_gas, 100_000);
    }
}
