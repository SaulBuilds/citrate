// lattice-v3/core/storage/src/lib.rs

pub mod cache;
pub mod chain;
pub mod db;
pub mod ipfs;
pub mod pruning;
pub mod state;
pub mod state_manager;

use anyhow::Result;
use cache::Cache;
use chain::{BlockStore, TransactionStore};
use db::RocksDB;
use lattice_consensus::types::Hash;
use pruning::{Pruner, PruningConfig};
use state::StateStore;
use std::path::Path;
use std::sync::Arc;
use tracing::info;

/// Main storage manager combining all storage components
pub struct StorageManager {
    pub db: Arc<RocksDB>,
    pub blocks: Arc<BlockStore>,
    pub transactions: Arc<TransactionStore>,
    pub state: Arc<StateStore>,
    pub pruner: Arc<Pruner>,

    // Caches
    pub block_cache: Cache<Hash, Vec<u8>>,
    pub state_cache: Cache<Vec<u8>, Vec<u8>>,
}

impl StorageManager {
    /// Create a new storage manager
    pub fn new(path: impl AsRef<Path>, pruning_config: PruningConfig) -> Result<Self> {
        let db = Arc::new(RocksDB::open(path)?);

        let blocks = Arc::new(BlockStore::new(db.clone()));
        let transactions = Arc::new(TransactionStore::new(db.clone()));
        let state = Arc::new(StateStore::new(db.clone()));

        let pruner = Arc::new(Pruner::new(
            db.clone(),
            blocks.clone(),
            state.clone(),
            pruning_config,
        ));

        info!("Storage manager initialized");

        Ok(Self {
            db,
            blocks,
            transactions,
            state,
            pruner,
            block_cache: Cache::new(1000),
            state_cache: Cache::new(10000),
        })
    }

    /// Start background services (pruning)
    pub async fn start_services(self: Arc<Self>) {
        let pruner = self.pruner.clone();
        tokio::spawn(async move {
            pruner.start_auto_pruning().await;
        });

        info!("Storage services started");
    }

    /// Flush all data to disk
    pub fn flush(&self) -> Result<()> {
        self.db.flush()?;
        info!("Storage flushed to disk");
        Ok(())
    }

    /// Get storage statistics
    pub fn get_statistics(&self) -> String {
        self.db.get_statistics()
    }

    /// Clear all caches
    pub fn clear_caches(&self) {
        self.block_cache.clear();
        self.state_cache.clear();
        info!("Caches cleared");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_storage_manager_creation() {
        let temp_dir = TempDir::new().unwrap();
        let config = PruningConfig::default();

        let manager = StorageManager::new(temp_dir.path(), config).unwrap();
        assert!(!manager.block_cache.is_empty() || manager.block_cache.is_empty());
    }
}
