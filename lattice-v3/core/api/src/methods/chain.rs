use crate::types::{
    error::ApiError,
    request::{BlockId, BlockTag},
    response::{BlockResponse, TransactionResponse},
};
use lattice_consensus::types::{BlockHeader, Hash};
use lattice_execution::types::TransactionReceipt;
use lattice_storage::StorageManager;
use std::sync::Arc;

/// Chain-related API methods
pub struct ChainApi {
    storage: Arc<StorageManager>,
}

impl ChainApi {
    pub fn new(storage: Arc<StorageManager>) -> Self {
        Self { storage }
    }
    
    /// Get block by ID
    pub async fn get_block(&self, block_id: BlockId) -> Result<BlockResponse, ApiError> {
        let block = match block_id {
            BlockId::Hash(hash) => {
                self.storage.blocks
                    .get_block(&hash)
                    .map_err(|e| ApiError::InternalError(e.to_string()))?
                    .ok_or_else(|| ApiError::BlockNotFound(format!("{:?}", hash)))?
            }
            BlockId::Number(height) => {
                let hash = self.storage.blocks
                    .get_block_by_height(height)
                    .map_err(|e| ApiError::InternalError(e.to_string()))?
                    .ok_or_else(|| ApiError::BlockNotFound(format!("height {}", height)))?;
                
                self.storage.blocks
                    .get_block(&hash)
                    .map_err(|e| ApiError::InternalError(e.to_string()))?
                    .ok_or_else(|| ApiError::BlockNotFound(format!("{:?}", hash)))?
            }
            BlockId::Tag(tag) => {
                let height = match tag {
                    BlockTag::Latest => self.get_latest_height().await?,
                    BlockTag::Earliest => 0,
                    BlockTag::Pending => return Err(ApiError::InvalidParams("Pending blocks not supported".into())),
                };
                
                let hash = self.storage.blocks
                    .get_block_by_height(height)
                    .map_err(|e| ApiError::InternalError(e.to_string()))?
                    .ok_or_else(|| ApiError::BlockNotFound(format!("height {}", height)))?;
                
                self.storage.blocks
                    .get_block(&hash)
                    .map_err(|e| ApiError::InternalError(e.to_string()))?
                    .ok_or_else(|| ApiError::BlockNotFound(format!("{:?}", hash)))?
            }
        };
        
        Ok(block.into())
    }
    
    /// Get block header by ID
    pub async fn get_block_header(&self, block_id: BlockId) -> Result<BlockHeader, ApiError> {
        let header = match block_id {
            BlockId::Hash(hash) => {
                self.storage.blocks
                    .get_header(&hash)
                    .map_err(|e| ApiError::InternalError(e.to_string()))?
                    .ok_or_else(|| ApiError::BlockNotFound(format!("{:?}", hash)))?
            }
            BlockId::Number(height) => {
                let hash = self.storage.blocks
                    .get_block_by_height(height)
                    .map_err(|e| ApiError::InternalError(e.to_string()))?
                    .ok_or_else(|| ApiError::BlockNotFound(format!("height {}", height)))?;
                
                self.storage.blocks
                    .get_header(&hash)
                    .map_err(|e| ApiError::InternalError(e.to_string()))?
                    .ok_or_else(|| ApiError::BlockNotFound(format!("{:?}", hash)))?
            }
            BlockId::Tag(tag) => {
                let height = match tag {
                    BlockTag::Latest => self.get_latest_height().await?,
                    BlockTag::Earliest => 0,
                    BlockTag::Pending => return Err(ApiError::InvalidParams("Pending blocks not supported".into())),
                };
                
                let hash = self.storage.blocks
                    .get_block_by_height(height)
                    .map_err(|e| ApiError::InternalError(e.to_string()))?
                    .ok_or_else(|| ApiError::BlockNotFound(format!("height {}", height)))?;
                
                self.storage.blocks
                    .get_header(&hash)
                    .map_err(|e| ApiError::InternalError(e.to_string()))?
                    .ok_or_else(|| ApiError::BlockNotFound(format!("{:?}", hash)))?
            }
        };
        
        Ok(header)
    }
    
    /// Get transaction by hash
    pub async fn get_transaction(&self, hash: Hash) -> Result<TransactionResponse, ApiError> {
        let tx = self.storage.transactions
            .get_transaction(&hash)
            .map_err(|e| ApiError::InternalError(e.to_string()))?
            .ok_or_else(|| ApiError::TransactionNotFound(format!("{:?}", hash)))?;
        
        Ok(tx.into())
    }
    
    /// Get transaction receipt
    pub async fn get_receipt(&self, hash: Hash) -> Result<TransactionReceipt, ApiError> {
        self.storage.transactions
            .get_receipt(&hash)
            .map_err(|e| ApiError::InternalError(e.to_string()))?
            .ok_or_else(|| ApiError::TransactionNotFound(format!("Receipt for {:?}", hash)))
    }
    
    /// Get current chain height
    pub async fn get_height(&self) -> Result<u64, ApiError> {
        self.get_latest_height().await
    }
    
    /// Get current DAG tips
    pub async fn get_tips(&self) -> Result<Vec<Hash>, ApiError> {
        // In a real implementation, this would query the DAG store for current tips
        // For now, return the latest block as the only tip
        let height = self.get_latest_height().await?;
        let hash = self.storage.blocks
            .get_block_by_height(height)
            .map_err(|e| ApiError::InternalError(e.to_string()))?
            .ok_or_else(|| ApiError::BlockNotFound(format!("height {}", height)))?;
        
        Ok(vec![hash])
    }
    
    // Helper method
    async fn get_latest_height(&self) -> Result<u64, ApiError> {
        self.storage.blocks
            .get_latest_height()
            .map_err(|e| ApiError::InternalError(e.to_string()))
    }
}
