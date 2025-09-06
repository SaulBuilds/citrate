use crate::chain::BlockStore;
use crate::db::RocksDB;
use crate::state::StateStore;
use anyhow::Result;
use lattice_consensus::types::Hash;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::time::interval;
use tracing::{debug, info, warn};

/// Pruning configuration
#[derive(Clone)]
pub struct PruningConfig {
    /// Number of blocks to keep
    pub keep_blocks: u64,
    /// Number of states to keep
    pub keep_states: u64,
    /// Pruning interval
    pub interval: Duration,
    /// Maximum items to prune per batch
    pub batch_size: usize,
    /// Enable automatic pruning
    pub auto_prune: bool,
}

impl Default for PruningConfig {
    fn default() -> Self {
        Self {
            keep_blocks: 100_000,
            keep_states: 10_000,
            interval: Duration::from_secs(3600), // 1 hour
            batch_size: 1000,
            auto_prune: true,
        }
    }
}

/// Storage pruner for removing old data
pub struct Pruner {
    db: Arc<RocksDB>,
    block_store: Arc<BlockStore>,
    state_store: Arc<StateStore>,
    config: PruningConfig,
}

impl Pruner {
    pub fn new(
        db: Arc<RocksDB>,
        block_store: Arc<BlockStore>,
        state_store: Arc<StateStore>,
        config: PruningConfig,
    ) -> Self {
        Self {
            db,
            block_store,
            state_store,
            config,
        }
    }
    
    /// Start automatic pruning task
    pub async fn start_auto_pruning(self: Arc<Self>) {
        if !self.config.auto_prune {
            info!("Automatic pruning disabled");
            return;
        }
        
        let mut ticker = interval(self.config.interval);
        
        loop {
            ticker.tick().await;
            
            info!("Starting pruning cycle");
            let start = Instant::now();
            
            match self.prune().await {
                Ok(stats) => {
                    info!(
                        "Pruning completed in {:?}: {} blocks, {} states pruned",
                        start.elapsed(),
                        stats.blocks_pruned,
                        stats.states_pruned
                    );
                }
                Err(e) => {
                    warn!("Pruning failed: {}", e);
                }
            }
        }
    }
    
    /// Perform pruning
    pub async fn prune(&self) -> Result<PruningStats> {
        let mut stats = PruningStats::default();
        
        // Get current height
        let current_height = self.block_store.get_latest_height()?;
        
        if current_height > self.config.keep_blocks {
            let prune_height = current_height - self.config.keep_blocks;
            stats.blocks_pruned = self.prune_blocks_before(prune_height).await?;
        }
        
        if current_height > self.config.keep_states {
            let prune_state_height = current_height - self.config.keep_states;
            stats.states_pruned = self.prune_states_before(prune_state_height).await?;
        }
        
        // Compact database after pruning
        self.compact().await?;
        
        Ok(stats)
    }
    
    /// Prune blocks before specified height
    async fn prune_blocks_before(&self, height: u64) -> Result<usize> {
        let mut pruned = 0;
        let mut batch_count = 0;
        
        for h in 0..height {
            if let Some(hash) = self.block_store.get_block_by_height(h)? {
                self.block_store.delete_block(&hash)?;
                pruned += 1;
                batch_count += 1;
                
                if batch_count >= self.config.batch_size {
                    // Yield to prevent blocking
                    tokio::task::yield_now().await;
                    batch_count = 0;
                }
            }
        }
        
        debug!("Pruned {} blocks before height {}", pruned, height);
        Ok(pruned)
    }
    
    /// Prune state snapshots before specified height
    async fn prune_states_before(&self, height: u64) -> Result<usize> {
        // In a real implementation, this would prune old state snapshots
        // For now, we'll just return 0 as we don't have state snapshot height tracking
        Ok(0)
    }
    
    /// Compact the database
    async fn compact(&self) -> Result<()> {
        info!("Compacting database");
        
        self.block_store.compact()?;
        self.state_store.compact()?;
        self.db.flush()?;
        
        info!("Database compaction completed");
        Ok(())
    }
    
    /// Force immediate pruning
    pub async fn force_prune(&self) -> Result<PruningStats> {
        info!("Force pruning initiated");
        self.prune().await
    }
    
    /// Get pruning statistics
    pub fn get_config(&self) -> &PruningConfig {
        &self.config
    }
    
    /// Update pruning configuration
    pub fn update_config(&mut self, config: PruningConfig) {
        self.config = config;
        info!("Pruning configuration updated");
    }
}

/// Pruning statistics
#[derive(Default, Debug)]
pub struct PruningStats {
    pub blocks_pruned: usize,
    pub states_pruned: usize,
    pub transactions_pruned: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chain::BlockStore;
    use crate::state::StateStore;
    use tempfile::TempDir;
    
    #[tokio::test]
    async fn test_pruning_config() {
        let config = PruningConfig::default();
        assert_eq!(config.keep_blocks, 100_000);
        assert_eq!(config.keep_states, 10_000);
        assert!(config.auto_prune);
    }
    
    #[tokio::test]
    async fn test_pruner_creation() {
        let temp_dir = TempDir::new().unwrap();
        let db = Arc::new(RocksDB::open(temp_dir.path()).unwrap());
        let block_store = Arc::new(BlockStore::new(db.clone()));
        let state_store = Arc::new(StateStore::new(db.clone()));
        
        let config = PruningConfig {
            keep_blocks: 1000,
            keep_states: 100,
            interval: Duration::from_secs(60),
            batch_size: 100,
            auto_prune: false,
        };
        
        let pruner = Pruner::new(db, block_store, state_store, config);
        assert_eq!(pruner.get_config().keep_blocks, 1000);
    }
}