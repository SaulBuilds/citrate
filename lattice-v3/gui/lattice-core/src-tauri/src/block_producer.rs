use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{interval, Duration};
use anyhow::Result;
use tracing::{info, warn, error};
use primitive_types::U256;

use lattice_consensus::{
    GhostDag, TipSelector,
    types::{Block, BlockHeader, Hash, Transaction, PublicKey, Signature, VrfProof, GhostDagParams},
};
use lattice_sequencer::Mempool;
use lattice_execution::Executor;
use lattice_execution::types::{TransactionReceipt, Address};
use lattice_storage::StorageManager;
use crate::wallet::WalletManager;
use lattice_network::{PeerManager, NetworkMessage};

pub struct BlockProducer {
    ghostdag: Arc<GhostDag>,
    mempool: Arc<RwLock<Mempool>>,
    executor: Arc<Executor>,
    storage: Arc<StorageManager>,
    reward_address: Arc<RwLock<Option<String>>>,
    running: Arc<RwLock<bool>>,
    wallet_manager: Option<Arc<WalletManager>>,
    peer_manager: Option<Arc<PeerManager>>,
}

impl BlockProducer {
    pub fn new(
        ghostdag: Arc<GhostDag>,
        mempool: Arc<RwLock<Mempool>>,
        executor: Arc<Executor>,
        storage: Arc<StorageManager>,
        reward_address: Arc<RwLock<Option<String>>>,
        wallet_manager: Option<Arc<WalletManager>>,
        peer_manager: Option<Arc<PeerManager>>,
    ) -> Self {
        Self {
            ghostdag,
            mempool,
            executor,
            storage,
            reward_address,
            running: Arc::new(RwLock::new(false)),
            wallet_manager,
            peer_manager,
        }
    }

    /// Expose running flag so callers can stop the loop cleanly
    pub fn running_flag(&self) -> Arc<RwLock<bool>> {
        self.running.clone()
    }

    /// Start the block production loop and return the join handle
    pub async fn start(&self) -> Result<tokio::task::JoinHandle<()>> {
        let mut running_guard = self.running.write().await;
        if *running_guard {
            // Already running; return a no-op handle
            return Ok(tokio::spawn(async {}));
        }
        *running_guard = true;
        
        info!("Starting block producer");

        // Create genesis block if DAG is empty
        self.create_genesis_block_if_needed().await?;

        // Start block production loop
        let producer = self.clone();
        let handle = tokio::spawn(async move {
            producer.production_loop().await;
        });
        Ok(handle)
    }

    /// Stop the block producer
    pub async fn stop(&self) {
        *self.running.write().await = false;
        info!("Block producer stopped");
    }

    /// Main block production loop
    async fn production_loop(&self) {
        let mut interval = interval(Duration::from_secs(2)); // 2 second block time

        while *self.running.read().await {
            interval.tick().await;

            match self.produce_block().await {
                Ok(block) => {
                    info!("Produced block {} at height {}", 
                          hex::encode(&block.header.block_hash.as_bytes()[..8]), 
                          block.header.height);
                }
                Err(e) => {
                    warn!("Failed to produce block: {}", e);
                }
            }
        }
    }

    /// Produce a single block
    pub async fn produce_block(&self) -> Result<Block> {
        // Get current tips from the DAG
        let tips = self.ghostdag.get_tips().await;
        if tips.is_empty() {
            return Err(anyhow::anyhow!("No tips available for block production"));
        }

        // Select parents 
        let (selected_parent, merge_parents) = self.select_parents(&tips).await?;

        // Get transactions from mempool (limit to 100 for now)
        let transactions = {
            let mempool_guard = self.mempool.read().await;
            mempool_guard.get_transactions(100).await
        };
        
        // Calculate new height
        let parent_block = self.storage.blocks.get_block(&selected_parent)?
            .ok_or_else(|| anyhow::anyhow!("Selected parent block not found"))?;
        let height = parent_block.header.height + 1;

        // Execute transactions and get receipts + state root
        let (state_root, receipts) = self.execute_transactions(&transactions, height).await?;

        // Calculate other roots
        let tx_root = self.calculate_tx_root(&transactions);
        let receipt_root = self.calculate_receipt_root(&receipts)?;
        let artifact_root = Hash::default(); // For AI artifacts

        // Calculate blue score (simplified)
        let blue_score = height * 10; // Simplified calculation

        // Get reward address
        let reward_address = self.reward_address.read().await.clone()
            .unwrap_or_default();

        // Create block hash deterministically from parent, height and timestamp
        let block_hash = {
            use sha3::{Digest, Keccak256};
            let mut hasher = Keccak256::new();
            hasher.update(selected_parent.as_bytes());
            for p in &merge_parents { hasher.update(p.as_bytes()); }
            hasher.update(&height.to_le_bytes());
            hasher.update(&blue_score.to_le_bytes());
            let bytes = hasher.finalize();
            Hash::from_bytes(&bytes)
        };

        // Create block header
        let header = BlockHeader {
            version: 1,
            block_hash,
            selected_parent_hash: selected_parent,
            merge_parent_hashes: merge_parents,
            timestamp: chrono::Utc::now().timestamp() as u64,
            height,
            blue_score,
            blue_work: (height as u128) * 1000,
            pruning_point: Hash::default(),
            proposer_pubkey: PublicKey::new([0u8; 32]),
            vrf_reveal: VrfProof {
                proof: vec![0u8; 80],
                output: Hash::default(),
            },
        };

        // Create block
        let block = Block {
            header,
            state_root,
            tx_root,
            receipt_root,
            artifact_root,
            ghostdag_params: GhostDagParams::default(),
            transactions,
            signature: Signature::new([0u8; 64]),
        };

        // Add block to DAG
        self.ghostdag.add_block(&block).await?;

        // Store block
        self.storage.blocks.put_block(&block)?;

        // Store transactions and receipts for RPC visibility
        if !block.transactions.is_empty() {
            self.storage.transactions.put_transactions(&block.transactions)?;
            let pairs: Vec<(Hash, TransactionReceipt)> = block
                .transactions
                .iter()
                .zip(receipts.iter())
                .map(|(tx, rcpt)| (tx.hash, rcpt.clone()))
                .collect();
            if !pairs.is_empty() {
                self.storage.transactions.put_receipts(&pairs)?;
            }

            // Remove processed transactions from mempool
            let mempool_guard = self.mempool.write().await;
            for tx in &block.transactions {
                let _ = mempool_guard.remove_transaction(&tx.hash).await;
            }
        }

        // Add reward transaction to the proposer's address
        if !reward_address.is_empty() {
            self.add_block_reward(&reward_address, height).await?;
        }

        // Broadcast block to network peers
        if let Some(pm) = &self.peer_manager {
            let _ = pm.broadcast(&NetworkMessage::NewBlock { block: block.clone() }).await;
        }

        Ok(block)
    }

    async fn create_genesis_block_if_needed(&self) -> Result<()> {
        // Check if we already have blocks
        if self.storage.blocks.get_latest_height().unwrap_or(0) > 0 {
            return Ok(()); // Genesis already exists
        }

        info!("Creating genesis block");

        // Create genesis block
        let genesis_hash = Hash::new([0u8; 32]);
        let header = BlockHeader {
            version: 1,
            block_hash: genesis_hash,
            selected_parent_hash: Hash::default(),
            merge_parent_hashes: vec![],
            timestamp: chrono::Utc::now().timestamp() as u64,
            height: 0,
            blue_score: 0,
            blue_work: 0,
            pruning_point: Hash::default(),
            proposer_pubkey: PublicKey::new([0u8; 32]),
            vrf_reveal: VrfProof {
                proof: vec![0u8; 80],
                output: Hash::default(),
            },
        };

        let genesis_block = Block {
            header,
            state_root: Hash::default(),
            tx_root: Hash::default(),
            receipt_root: Hash::default(),
            artifact_root: Hash::default(),
            ghostdag_params: GhostDagParams::default(),
            transactions: vec![],
            signature: Signature::new([0u8; 64]),
        };

        // Add genesis to DAG and storage
        self.ghostdag.add_block(&genesis_block).await?;
        self.storage.blocks.put_block(&genesis_block)?;

        info!("Genesis block created");
        Ok(())
    }

    async fn select_parents(&self, tips: &[Hash]) -> Result<(Hash, Vec<Hash>)> {
        if tips.is_empty() {
            return Err(anyhow::anyhow!("No tips available"));
        }

        // Select the tip with highest blue score as selected parent
        let mut best_tip = tips[0];
        let mut best_score = 0u64;

        for tip in tips {
            let score = self.ghostdag.get_blue_score(tip).await?;
            if score > best_score {
                best_score = score;
                best_tip = *tip;
            }
        }

        // Other tips become merge parents (up to max_parents - 1)
        let merge_parents: Vec<Hash> = tips
            .iter()
            .filter(|&&h| h != best_tip)
            .take(9) // max_parents - 1
            .cloned()
            .collect();

        Ok((best_tip, merge_parents))
    }

    async fn execute_transactions(&self, transactions: &[Transaction], height: u64) -> Result<(Hash, Vec<TransactionReceipt>)> {
        // Execute transactions using the real executor to produce receipts
        let mut receipts: Vec<TransactionReceipt> = Vec::new();

        // Create a temporary block context for execution
        let temp_header = BlockHeader {
            version: 1,
            block_hash: Hash::default(),
            selected_parent_hash: Hash::default(),
            merge_parent_hashes: vec![],
            timestamp: chrono::Utc::now().timestamp() as u64,
            height,
            blue_score: height * 10,
            blue_work: (height as u128) * 1000,
            pruning_point: Hash::default(),
            proposer_pubkey: PublicKey::new([0u8; 32]),
            vrf_reveal: VrfProof { proof: vec![0u8; 80], output: Hash::default() },
        };
        let temp_block = Block {
            header: temp_header,
            state_root: Hash::default(),
            tx_root: Hash::default(),
            receipt_root: Hash::default(),
            artifact_root: Hash::default(),
            ghostdag_params: GhostDagParams::default(),
            transactions: vec![],
            signature: Signature::new([0u8; 64]),
        };

        for tx in transactions {
            match self.executor.execute_transaction(&temp_block, tx).await {
                Ok(rcpt) => receipts.push(rcpt),
                Err(e) => {
                    error!("Failed to execute transaction {}: {}", tx.hash, e);
                    receipts.push(TransactionReceipt {
                        tx_hash: tx.hash,
                        block_hash: temp_block.header.block_hash,
                        block_number: temp_block.header.height,
                        from: Address::from_public_key(&tx.from),
                        to: tx.to.map(|pk| Address::from_public_key(&pk)),
                        gas_used: tx.gas_limit,
                        status: false,
                        logs: vec![],
                        output: vec![],
                    });
                }
            }
        }

        // CRITICAL FIX: Commit state changes to persist them
        // Without this, all transaction effects are lost!
        let state_root = self.executor.state_db().commit();
        Ok((state_root, receipts))
    }

    fn calculate_tx_root(&self, transactions: &[Transaction]) -> Hash {
        use sha3::{Digest, Keccak256};
        let mut hasher = Keccak256::new();
        
        for tx in transactions {
            hasher.update(tx.hash.as_bytes());
        }
        
        let result = hasher.finalize();
        Hash::from_bytes(&result)
    }

    /// Calculate receipt root similar to node implementation
    fn calculate_receipt_root(&self, receipts: &[TransactionReceipt]) -> anyhow::Result<Hash> {
        use sha3::{Digest, Keccak256};
        let mut hasher = Keccak256::new();
        for r in receipts {
            hasher.update(r.tx_hash.as_bytes());
            hasher.update(&[if r.status { 1 } else { 0 }]);
            hasher.update(&r.gas_used.to_le_bytes());
        }
        let bytes = hasher.finalize();
        Ok(Hash::from_bytes(&bytes))
    }

    async fn add_block_reward(&self, reward_address: &str, _height: u64) -> Result<()> {
        // Credit block reward directly to blockchain state via executor
        // 10 LAT per block in 18-decimal units (wei)
        const DECIMALS: u128 = 1_000_000_000_000_000_000u128;
        const BLOCK_REWARD_TOKENS: u128 = 10;
        let amount_wei = primitive_types::U256::from(BLOCK_REWARD_TOKENS.saturating_mul(DECIMALS));

        // Parse reward address - handle both hex strings and base58
        let validator_address = if reward_address.starts_with("0x") {
            // Parse as hex ethereum address
            let addr_bytes = hex::decode(&reward_address[2..])
                .map_err(|e| anyhow::anyhow!("Invalid hex address: {}", e))?;
            if addr_bytes.len() != 20 {
                return Err(anyhow::anyhow!("Invalid address length: expected 20 bytes, got {}", addr_bytes.len()));
            }
            let mut addr = [0u8; 20];
            addr.copy_from_slice(&addr_bytes);
            Address(addr)
        } else {
            // Assume it's a public key string, derive address from it
            // For now, take first 20 bytes of the public key as address (simplified)
            let pk_bytes = hex::decode(reward_address)
                .unwrap_or_else(|_| reward_address.as_bytes().to_vec());
            let mut addr = [0u8; 20];
            let len = pk_bytes.len().min(20);
            addr[..len].copy_from_slice(&pk_bytes[..len]);
            Address(addr)
        };

        // Get current balance and add reward
        let current_balance = self.executor.get_balance(&validator_address);
        let new_balance = current_balance + amount_wei;
        self.executor.set_balance(&validator_address, new_balance);
        
        info!("Minted {} LAT ({} wei) to validator {} (new balance: {} wei)", 
              BLOCK_REWARD_TOKENS, 
              amount_wei,
              hex::encode(&validator_address.0), 
              new_balance);

        // Also update wallet manager if present (for UI display)
        if let Some(wm) = &self.wallet_manager {
            if let Some(_account) = wm.get_account(reward_address).await {
                let balance_u128 = if new_balance > primitive_types::U256::from(u128::MAX) {
                    u128::MAX
                } else {
                    new_balance.as_u128()
                };
                wm.update_balance(reward_address, balance_u128).await
                    .map_err(|e| anyhow::anyhow!(e))?;
            }
        }

        Ok(())
    }
}

impl Clone for BlockProducer {
    fn clone(&self) -> Self {
        Self {
            ghostdag: self.ghostdag.clone(),
            mempool: self.mempool.clone(),
            executor: self.executor.clone(),
            storage: self.storage.clone(),
            reward_address: self.reward_address.clone(),
            running: self.running.clone(),
            wallet_manager: self.wallet_manager.clone(),
            peer_manager: self.peer_manager.clone(),
        }
    }
}
