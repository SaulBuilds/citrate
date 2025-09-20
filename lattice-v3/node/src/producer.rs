use lattice_consensus::types::{Block, BlockHeader, Transaction, Hash, PublicKey, Signature, GhostDagParams, VrfProof};
use lattice_consensus::ghostdag::GhostDag;
use lattice_consensus::tip_selection::TipSelector;
use lattice_consensus::chain_selection::ChainSelector;
use lattice_consensus::dag_store::DagStore;
use lattice_storage::{StorageManager, state_manager::StateManager as AIStateManager};
use lattice_execution::Executor;
use lattice_sequencer::mempool::Mempool;
use lattice_economics::{RewardCalculator, RewardConfig};
use lattice_network::{PeerManager, NetworkMessage};
use primitive_types::U256;
use sha3::{Digest, Sha3_256};
use std::sync::Arc;
use tokio::time::{interval, Duration};
use tracing::{info, error};

/// Calculate block header hash using SHA3-256
fn calculate_block_hash_header(header: &BlockHeader) -> Hash {
    let mut hasher = Sha3_256::new();
    
    // Hash header fields
    hasher.update(header.version.to_le_bytes());
    hasher.update(header.selected_parent_hash.as_bytes());
    for parent in &header.merge_parent_hashes {
        hasher.update(parent.as_bytes());
    }
    hasher.update(header.timestamp.to_le_bytes());
    hasher.update(header.height.to_le_bytes());
    hasher.update(header.blue_score.to_le_bytes());
    hasher.update(header.blue_work.to_le_bytes());
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
    #[allow(dead_code)]
    chain_selector: Arc<ChainSelector>,
    ai_state_manager: Arc<AIStateManager>,
    peer_manager: Option<Arc<PeerManager>>,
    coinbase: PublicKey,
    target_block_time: u64,
    reward_calculator: RewardCalculator,
}

impl BlockProducer {
    #[allow(dead_code)]
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
        
        // Create reward calculator with default config
        let reward_config = RewardConfig {
            block_reward: 10,  // 10 LATT per block
            halving_interval: 2_100_000,
            inference_bonus: 1,  // 0.01 LATT per inference  
            model_deployment_bonus: 1,  // 1 LATT per model deployment
            treasury_percentage: 10,
            treasury_address: lattice_execution::types::Address([0x11; 20]),  // Treasury address
        };
        let reward_calculator = RewardCalculator::new(reward_config);
        
        // Create AI state manager
        let ai_state_manager = Arc::new(AIStateManager::new(storage.db.clone()));
        
        Self {
            storage,
            executor,
            mempool,
            dag_store,
            ghostdag,
            tip_selector,
            chain_selector,
            ai_state_manager,
            peer_manager: None,
            coinbase,
            target_block_time,
            reward_calculator,
        }
    }
    
    #[allow(dead_code)]
    pub fn with_peer_manager(
        storage: Arc<StorageManager>,
        executor: Arc<Executor>,
        mempool: Arc<Mempool>,
        peer_manager: Option<Arc<PeerManager>>,
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
        
        // Create reward calculator with default config
        let reward_config = RewardConfig {
            block_reward: 10,  // 10 LATT per block
            halving_interval: 2_100_000,
            inference_bonus: 1,  // 0.01 LATT per inference  
            model_deployment_bonus: 1,  // 1 LATT per model deployment
            treasury_percentage: 10,
            treasury_address: lattice_execution::types::Address([0x11; 20]),  // Treasury address
        };
        let reward_calculator = RewardCalculator::new(reward_config);
        
        // Create AI state manager
        let ai_state_manager = Arc::new(AIStateManager::new(storage.db.clone()));
        
        Self {
            storage,
            executor,
            mempool,
            dag_store,
            ghostdag,
            tip_selector,
            chain_selector,
            ai_state_manager,
            peer_manager,
            coinbase,
            target_block_time,
            reward_calculator,
        }
    }

    /// Create with explicit reward configuration (for governance-driven params)
    pub fn with_peer_manager_and_rewards(
        storage: Arc<StorageManager>,
        executor: Arc<Executor>,
        mempool: Arc<Mempool>,
        peer_manager: Option<Arc<PeerManager>>,
        coinbase: PublicKey,
        target_block_time: u64,
        reward_config: RewardConfig,
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
            100,
        ));

        let reward_calculator = RewardCalculator::new(reward_config);
        let ai_state_manager = Arc::new(AIStateManager::new(storage.db.clone()));

        Self {
            storage,
            executor,
            mempool,
            dag_store,
            ghostdag,
            tip_selector,
            chain_selector,
            ai_state_manager,
            peer_manager,
            coinbase,
            target_block_time,
            reward_calculator,
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
        
        // Select parents using GhostDAG algorithm
        let (selected_parent, merge_parents) = if tips.is_empty() {
            // Genesis case: no parents
            (Hash::default(), vec![])
        } else {
            // Use GhostDAG to select the best parent and merge parents
            self.select_parents_with_ghostdag(&tips).await?
        };
        
        // Calculate blue set for the new block
        let temp_block = lattice_consensus::types::Block {
            header: lattice_consensus::types::BlockHeader {
                version: 1,
                block_hash: Hash::default(),
                selected_parent_hash: selected_parent,
                merge_parent_hashes: merge_parents.clone(),
                timestamp: chrono::Utc::now().timestamp() as u64,
                height: 0, // Will be calculated
                blue_score: 0, // Will be calculated
                blue_work: 0, // Will be calculated
                pruning_point: Hash::default(),
                proposer_pubkey: self.coinbase,
                vrf_reveal: VrfProof {
                    proof: vec![],
                    output: Hash::default(),
                },
            },
            state_root: Hash::default(),
            tx_root: Hash::default(),
            receipt_root: Hash::default(),
            artifact_root: Hash::default(),
            ghostdag_params: lattice_consensus::types::GhostDagParams::default(),
            transactions: vec![],
            signature: Signature::new([0; 64]),
        };
        
        let blue_set = self.ghostdag.calculate_blue_set(&temp_block).await?;
        let blue_score = self.ghostdag.calculate_blue_score(&temp_block).await?;
        
        // Get last block height from selected parent
        let last_height = if selected_parent != Hash::default() {
            // Get parent block from storage to determine height
            self.storage.blocks.get_block(&selected_parent)
                .ok()
                .and_then(|b| b.map(|block| block.header.height))
                .unwrap_or(0)
        } else {
            0
        };
        
        // Get transactions from mempool with AI priority
        let transactions = self.select_transactions_with_ai_priority().await?;
        
        // Blue score and work are already calculated above
        let blue_work = self.calculate_blue_work(&blue_set, blue_score)?;
        
        // Create block header with GhostDAG consensus data
        let mut header = BlockHeader {
            version: 1,
            block_hash: Hash::default(), // Will be computed
            selected_parent_hash: selected_parent,
            merge_parent_hashes: merge_parents,
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
        
        // Execute transactions and calculate state roots
        let (state_root, receipts) = self.execute_block_transactions(&transactions, &header).await?;
        let tx_root = self.calculate_tx_root(&transactions)?;
        let receipt_root = self.calculate_receipt_root(&receipts)?;
        let artifact_root = self.calculate_artifact_root(&transactions)?;
        
        // Create block with all computed data
        let block = Block {
            header: header.clone(),
            state_root,
            tx_root,
            receipt_root,
            artifact_root,
            ghostdag_params: self.ghostdag.params().clone(),
            transactions,
            signature: Signature::new([1; 64]), // Dummy signature for devnet
        };
        
        // Calculate and distribute block rewards
        let reward = self.reward_calculator.calculate_reward(&block);
        
        // Convert PublicKey to Address for coinbase (validator)
        let validator_address = lattice_execution::types::Address(self.coinbase.0[0..20].try_into().unwrap_or([0; 20]));
        let treasury_address = lattice_execution::types::Address([0x11; 20]);
        
        // Apply rewards to state
        if reward.validator_reward > U256::zero() {
            let current_balance = self.executor.get_balance(&validator_address);
            self.executor.set_balance(&validator_address, current_balance + reward.validator_reward);
            info!("Minted {} wei to validator {}", reward.validator_reward, hex::encode(validator_address.0));
        }
        
        if reward.treasury_reward > U256::zero() {
            let current_balance = self.executor.get_balance(&treasury_address);
            self.executor.set_balance(&treasury_address, current_balance + reward.treasury_reward);
            info!("Minted {} wei to treasury", reward.treasury_reward);
        }
        
        // Persist block and related data
        self.storage.blocks.put_block(&block)?;
        
        // Broadcast block to connected peers
        if let Some(peer_manager) = &self.peer_manager {
            let block_msg = NetworkMessage::NewBlock { block: block.clone() };
            tokio::spawn({
                let pm = peer_manager.clone();
                async move {
                    if let Err(e) = pm.broadcast(&block_msg).await {
                        tracing::warn!("Failed to broadcast block to peers: {}", e);
                    } else {
                        tracing::info!("Broadcasted new block to peers");
                    }
                }
            });
        }

        // Store transactions and receipts for RPC visibility
        if !block.transactions.is_empty() {
            // Store transactions
            self.storage.transactions.put_transactions(&block.transactions)?;

            // Pair tx hashes with receipts and store
            let mut pairs: Vec<(Hash, lattice_execution::types::TransactionReceipt)> = Vec::new();
            for (i, tx) in block.transactions.iter().enumerate() {
                if let Some(r) = receipts.get(i) {
                    pairs.push((tx.hash, r.clone()));
                }
            }
            if !pairs.is_empty() {
                self.storage.transactions.put_receipts(&pairs)?;
            }

            // Remove included transactions from mempool
            for tx in &block.transactions {
                let _ = self.mempool.remove_transaction(&tx.hash).await;
            }
        }

        // Update DAG store
        self.dag_store.store_block(block.clone()).await?;

        Ok(header.block_hash)
    }
    
    /// Select parents using GhostDAG algorithm
    async fn select_parents_with_ghostdag(&self, tips: &[lattice_consensus::types::Tip]) -> anyhow::Result<(Hash, Vec<Hash>)> {
        // Convert tips to hashes
        let tip_hashes: Vec<Hash> = tips.iter().map(|tip| tip.hash).collect();
        
        // Use tip selector to find the best tip (highest blue score)
        let selected_parent = self.tip_selector.select_tip(&tip_hashes).await?;
        
        // Select merge parents from remaining tips
        let merge_parents: Vec<Hash> = tip_hashes
            .into_iter()
            .filter(|h| *h != selected_parent)
            .take(self.ghostdag.params().max_parents - 1) // Leave room for selected parent
            .collect();
        
        Ok((selected_parent, merge_parents))
    }
    
    /// Select transactions with AI operation priority
    async fn select_transactions_with_ai_priority(&self) -> anyhow::Result<Vec<Transaction>> {
        let mut selected = Vec::new();
        
        // Define capacity limits
        const MAX_BLOCK_SIZE: usize = 10_000_000; // 10MB
        const MAX_AI_TXS_PER_BLOCK: usize = 10;
        const MAX_STANDARD_TXS: usize = 100;
        
        // Get AI transactions first (model operations, inference requests)
        let ai_txs = self.mempool.get_ai_transactions(MAX_AI_TXS_PER_BLOCK).await;
        selected.extend(ai_txs);
        
        // Fill remaining space with standard transactions
        let standard_txs = self.mempool.get_best_transactions(
            MAX_STANDARD_TXS,
            MAX_BLOCK_SIZE
        ).await;
        selected.extend(standard_txs);
        
        Ok(selected)
    }
    
    /// Execute all transactions in a block
    async fn execute_block_transactions(
        &self,
        transactions: &[Transaction],
        header: &BlockHeader,
    ) -> anyhow::Result<(Hash, Vec<lattice_execution::types::TransactionReceipt>)> {
        let mut receipts = Vec::new();
        
        // Create a temporary block for execution context
        let temp_block = Block {
            header: header.clone(),
            state_root: Hash::default(),
            tx_root: Hash::default(),
            receipt_root: Hash::default(),
            artifact_root: Hash::default(),
            ghostdag_params: self.ghostdag.params().clone(),
            transactions: vec![],
            signature: Signature::new([0; 64]),
        };
        
        // Execute each transaction
        for tx in transactions {
            match self.executor.execute_transaction(&temp_block, tx).await {
                Ok(receipt) => receipts.push(receipt),
                Err(e) => {
                    error!("Failed to execute transaction {}: {}", tx.hash, e);
                    // Create failed receipt
                    receipts.push(lattice_execution::types::TransactionReceipt {
                        tx_hash: tx.hash,
                        block_hash: header.block_hash,
                        block_number: header.height,
                        from: lattice_execution::types::Address::from_public_key(&tx.from),
                        to: tx.to.map(|pk| lattice_execution::types::Address::from_public_key(&pk)),
                        gas_used: tx.gas_limit, // All gas consumed on failure
                        status: false,
                        logs: vec![],
                        output: vec![],
                    });
                }
            }
        }
        
        // Calculate final state root including AI state
        let state_root = self.ai_state_manager.calculate_state_root().await?;
        
        Ok((state_root, receipts))
    }
    
    /// Calculate transaction root
    fn calculate_tx_root(&self, transactions: &[Transaction]) -> anyhow::Result<Hash> {
        use sha3::{Digest, Sha3_256};
        let mut hasher = Sha3_256::new();
        
        for tx in transactions {
            hasher.update(tx.hash.as_bytes());
        }
        
        let hash_bytes = hasher.finalize();
        let mut hash_array = [0u8; 32];
        hash_array.copy_from_slice(&hash_bytes[..32]);
        Ok(Hash::new(hash_array))
    }
    
    /// Calculate receipt root
    fn calculate_receipt_root(&self, receipts: &[lattice_execution::types::TransactionReceipt]) -> anyhow::Result<Hash> {
        use sha3::{Digest, Sha3_256};
        let mut hasher = Sha3_256::new();
        
        for receipt in receipts {
            hasher.update(receipt.tx_hash.as_bytes());
            hasher.update([if receipt.status { 1 } else { 0 }]);
            hasher.update(receipt.gas_used.to_le_bytes());
        }
        
        let hash_bytes = hasher.finalize();
        let mut hash_array = [0u8; 32];
        hash_array.copy_from_slice(&hash_bytes[..32]);
        Ok(Hash::new(hash_array))
    }
    
    /// Calculate artifact root for AI models
    fn calculate_artifact_root(&self, transactions: &[Transaction]) -> anyhow::Result<Hash> {
        use sha3::{Digest, Sha3_256};
        let mut hasher = Sha3_256::new();
        
        // Hash any AI-related transaction data
        for tx in transactions {
            // Check if transaction contains AI operations
            if tx.data.len() >= 4 {
                match &tx.data[0..4] {
                    [0x01, 0x00, 0x00, 0x00] | // Register model
                    [0x02, 0x00, 0x00, 0x00] => { // Inference request
                        hasher.update(&tx.data);
                    }
                    _ => {}
                }
            }
        }
        
        let hash_bytes = hasher.finalize();
        let mut hash_array = [0u8; 32];
        hash_array.copy_from_slice(&hash_bytes[..32]);
        Ok(Hash::new(hash_array))
    }
    
    /// Calculate blue work based on blue set
    fn calculate_blue_work(&self, _blue_set: &lattice_consensus::types::BlueSet, blue_score: u64) -> anyhow::Result<u128> {
        // Simplified calculation: blue_work = blue_score * difficulty
        // In production, this would consider actual proof-of-work
        Ok(blue_score as u128 * 1_000_000)
    }
}
