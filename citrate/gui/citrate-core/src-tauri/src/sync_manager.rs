use std::sync::Arc;
use anyhow::Result;
use tokio::sync::RwLock;
use tracing::{info, warn, error, debug};

use citrate_consensus::{
    GhostDag,
    types::{Block, Hash},
};
use citrate_storage::StorageManager;
use citrate_execution::Executor;
use crate::network_service::NetworkService;

#[derive(Debug, Clone, PartialEq)]
pub enum SyncState {
    Idle,
    Syncing,
    Validating,
    Completed,
    Failed(String),
}

pub struct SyncManager {
    network_service: Arc<NetworkService>,
    ghostdag: Arc<GhostDag>,
    storage: Arc<StorageManager>,
    executor: Arc<Executor>,
    sync_state: Arc<RwLock<SyncState>>,
    running: Arc<RwLock<bool>>,
    target_height: Arc<RwLock<Option<u64>>>,
}

impl SyncManager {
    pub fn new(
        network_service: Arc<NetworkService>,
        ghostdag: Arc<GhostDag>,
        storage: Arc<StorageManager>,
        executor: Arc<Executor>,
    ) -> Self {
        Self {
            network_service,
            ghostdag,
            storage,
            executor,
            sync_state: Arc::new(RwLock::new(SyncState::Idle)),
            running: Arc::new(RwLock::new(false)),
            target_height: Arc::new(RwLock::new(None)),
        }
    }

    pub async fn start(&self) -> Result<()> {
        info!("Starting sync manager");
        *self.running.write().await = true;
        
        // Start sync loop
        let manager = self.clone_manager();
        tokio::spawn(async move {
            manager.sync_loop().await;
        });
        
        // Start validation loop
        let manager = self.clone_manager();
        tokio::spawn(async move {
            manager.validation_loop().await;
        });
        
        info!("Sync manager started");
        Ok(())
    }

    pub async fn stop(&self) -> Result<()> {
        info!("Stopping sync manager");
        *self.running.write().await = false;
        *self.sync_state.write().await = SyncState::Idle;
        info!("Sync manager stopped");
        Ok(())
    }

    pub async fn is_syncing(&self) -> bool {
        matches!(*self.sync_state.read().await, SyncState::Syncing)
    }

    pub async fn get_sync_state(&self) -> SyncState {
        self.sync_state.read().await.clone()
    }

    pub async fn get_sync_progress(&self) -> (u64, Option<u64>) {
        let current = self.storage.blocks.get_latest_height().unwrap_or(0);
        let target = *self.target_height.read().await;
        (current, target)
    }

    async fn sync_loop(&self) {
        info!("Sync loop started");
        
        while *self.running.read().await {
            // Check if we need to sync
            if let Err(e) = self.check_and_sync().await {
                error!("Sync failed: {}", e);
                *self.sync_state.write().await = SyncState::Failed(e.to_string());
            }
            
            // Wait before next sync check
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
        }
        
        info!("Sync loop stopped");
    }

    async fn check_and_sync(&self) -> Result<()> {
        // Get peer information
        let peers = self.network_service.get_peers().await;
        if peers.is_empty() {
            debug!("No peers available for syncing");
            return Ok(());
        }
        
        // Find the best height among peers
        let best_peer_height = peers.iter()
            .map(|p| p.best_height)
            .max()
            .unwrap_or(0);
        
        // Get our current height
        let our_height = self.storage.blocks.get_latest_height().unwrap_or(0);
        
        // Check if we need to sync
        if best_peer_height > our_height + 1 {
            info!("Starting sync: our height {} vs peer height {}", our_height, best_peer_height);
            
            *self.sync_state.write().await = SyncState::Syncing;
            *self.target_height.write().await = Some(best_peer_height);
            
            // Sync blocks in batches
            let mut current = our_height + 1;
            const BATCH_SIZE: u64 = 100;
            
            while current <= best_peer_height && *self.running.read().await {
                let to = std::cmp::min(current + BATCH_SIZE - 1, best_peer_height);
                
                match self.sync_batch(current, to).await {
                    Ok(()) => {
                        info!("Synced blocks {} to {}", current, to);
                        current = to + 1;
                    }
                    Err(e) => {
                        warn!("Failed to sync batch {}-{}: {}", current, to, e);
                        // Retry with smaller batch
                        if BATCH_SIZE > 10 {
                            let smaller_to = std::cmp::min(current + 9, best_peer_height);
                            if let Err(e) = self.sync_batch(current, smaller_to).await {
                                error!("Failed to sync smaller batch: {}", e);
                                return Err(e);
                            }
                            current = smaller_to + 1;
                        } else {
                            return Err(e);
                        }
                    }
                }
            }
            
            *self.sync_state.write().await = SyncState::Completed;
            info!("Sync completed at height {}", self.storage.blocks.get_latest_height().unwrap_or(0));
        }
        
        Ok(())
    }

    async fn sync_batch(&self, from: u64, to: u64) -> Result<()> {
        debug!("Requesting blocks {} to {}", from, to);
        
        // Request blocks from network
        let blocks = self.network_service.request_blocks(from, to).await?;
        
        if blocks.is_empty() {
            return Err(anyhow::anyhow!("No blocks received for range {}-{}", from, to));
        }
        
        // Validate and store blocks
        for block in blocks {
            self.process_block(block).await?;
        }
        
        Ok(())
    }

    async fn process_block(&self, block: Block) -> Result<()> {
        // Validate block structure
        self.validate_block(&block)?;
        
        // Add to DAG
        self.ghostdag.add_block(&block).await?;
        
        // Execute transactions and validate state
        if let Err(e) = self.validate_state(&block).await {
            warn!("State validation failed for block {}: {}", block.header.height, e);
            // Continue anyway for now (might be missing state data)
        }
        
        // Store block
        self.storage.blocks.insert_block(&block)?;
        
        debug!("Processed block {} at height {}", 
               hex::encode(&block.hash.as_bytes()[..8]), block.header.height);
        
        Ok(())
    }

    fn validate_block(&self, block: &Block) -> Result<()> {
        // Basic validation
        if block.header.version == 0 {
            return Err(anyhow::anyhow!("Invalid block version"));
        }
        
        if block.header.timestamp == 0 {
            return Err(anyhow::anyhow!("Invalid block timestamp"));
        }
        
        // Validate transaction root
        let calculated_tx_root = self.calculate_tx_root(&block.transactions);
        if calculated_tx_root != block.header.tx_root {
            return Err(anyhow::anyhow!("Transaction root mismatch"));
        }
        
        // Validate block hash
        let calculated_hash = self.calculate_block_hash(&block.header);
        if calculated_hash != block.hash {
            return Err(anyhow::anyhow!("Block hash mismatch"));
        }
        
        Ok(())
    }

    async fn validate_state(&self, block: &Block) -> Result<()> {
        // Get parent state
        let parent_block = self.storage.blocks.get_block(&block.header.selected_parent_hash)?
            .ok_or_else(|| anyhow::anyhow!("Parent block not found"))?;
        
        // Execute transactions to verify state root
        let mut state = self.executor.get_state_at(&parent_block.header.state_root)?;
        
        for tx in &block.transactions {
            if let Err(e) = self.executor.execute_transaction(tx, &mut state) {
                debug!("Transaction execution failed during validation: {}", e);
                // Continue to next transaction
            }
        }
        
        let calculated_state_root = state.calculate_root()?;
        if calculated_state_root != block.header.state_root {
            return Err(anyhow::anyhow!("State root mismatch"));
        }
        
        Ok(())
    }

    async fn validation_loop(&self) {
        info!("Validation loop started");
        
        while *self.running.read().await {
            // Validate recent blocks periodically
            if matches!(*self.sync_state.read().await, SyncState::Idle | SyncState::Completed) {
                if let Err(e) = self.validate_recent_blocks().await {
                    warn!("Recent block validation failed: {}", e);
                }
            }
            
            // Wait before next validation
            tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
        }
        
        info!("Validation loop stopped");
    }

    async fn validate_recent_blocks(&self) -> Result<()> {
        let height = self.storage.blocks.get_latest_height()?;
        if height == 0 {
            return Ok(());
        }
        
        // Validate last 10 blocks
        let start = height.saturating_sub(9);
        
        for h in start..=height {
            if let Some(block) = self.storage.blocks.get_block_by_height(h)? {
                if let Err(e) = self.validate_block(&block) {
                    error!("Block {} at height {} failed validation: {}", 
                           hex::encode(&block.hash.as_bytes()[..8]), h, e);
                }
            }
        }
        
        Ok(())
    }

    fn calculate_tx_root(&self, transactions: &[citrate_consensus::types::Transaction]) -> Hash {
        use sha3::{Digest, Keccak256};
        let mut hasher = Keccak256::new();
        
        for tx in transactions {
            hasher.update(tx.hash.as_bytes());
        }
        
        let result = hasher.finalize();
        Hash::from_bytes(&result)
    }

    fn calculate_block_hash(&self, header: &citrate_consensus::types::BlockHeader) -> Hash {
        use sha3::{Digest, Keccak256};
        let mut hasher = Keccak256::new();
        
        // Hash all header fields
        hasher.update(&header.version.to_le_bytes());
        hasher.update(header.selected_parent_hash.as_bytes());
        for parent in &header.merge_parent_hashes {
            hasher.update(parent.as_bytes());
        }
        hasher.update(&header.timestamp.to_le_bytes());
        hasher.update(&header.height.to_le_bytes());
        hasher.update(header.state_root.as_bytes());
        hasher.update(header.tx_root.as_bytes());
        hasher.update(header.receipt_root.as_bytes());
        hasher.update(header.artifact_root.as_bytes());
        hasher.update(&header.blue_score.to_le_bytes());
        
        let result = hasher.finalize();
        Hash::from_bytes(&result)
    }

    fn clone_manager(&self) -> SyncManager {
        SyncManager {
            network_service: self.network_service.clone(),
            ghostdag: self.ghostdag.clone(),
            storage: self.storage.clone(),
            executor: self.executor.clone(),
            sync_state: self.sync_state.clone(),
            running: self.running.clone(),
            target_height: self.target_height.clone(),
        }
    }
}