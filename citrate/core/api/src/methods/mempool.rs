// citrate/core/api/src/methods/mempool.rs
use crate::types::{
    error::ApiError,
    response::{MempoolStatus, TransactionResponse},
};
use citrate_consensus::types::Hash;
use citrate_sequencer::mempool::Mempool;
use std::sync::Arc;

/// Mempool-related API methods
pub struct MempoolApi {
    mempool: Arc<Mempool>,
}

impl MempoolApi {
    pub fn new(mempool: Arc<Mempool>) -> Self {
        Self { mempool }
    }

    /// Get mempool status
    pub async fn get_status(&self) -> Result<MempoolStatus, ApiError> {
        let stats = self.mempool.stats().await;

        Ok(MempoolStatus {
            pending: stats.total_transactions,
            queued: 0, // Not tracking queued separately for now
            total_size: stats.total_size,
            max_size: 10_000_000, // 10MB default max size
        })
    }

    /// Get pending transaction
    pub async fn get_transaction(
        &self,
        hash: Hash,
    ) -> Result<Option<TransactionResponse>, ApiError> {
        let tx = self.mempool.get_transaction(&hash).await;
        Ok(tx.map(Into::into))
    }

    /// List pending transactions
    pub async fn get_pending(
        &self,
        limit: Option<usize>,
    ) -> Result<Vec<TransactionResponse>, ApiError> {
        let limit = limit.unwrap_or(100).min(1000);
        let txs = self.mempool.get_transactions(limit).await;
        Ok(txs.into_iter().map(Into::into).collect())
    }

    /// Clear mempool (admin only)
    pub async fn clear(&self) -> Result<(), ApiError> {
        self.mempool.clear().await;
        Ok(())
    }
}
