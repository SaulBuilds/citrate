use crate::types::{
    error::ApiError,
    response::AccountResponse,
};
use lattice_consensus::types::Hash;
use lattice_execution::{
    executor::Executor,
    types::{Address, AccountState},
};
use lattice_storage::StorageManager;
use primitive_types::U256;
use std::sync::Arc;

/// State-related API methods
pub struct StateApi {
    storage: Arc<StorageManager>,
    executor: Arc<Executor>,
}

impl StateApi {
    pub fn new(storage: Arc<StorageManager>, executor: Arc<Executor>) -> Self {
        Self { storage, executor }
    }
    
    /// Get account state
    pub async fn get_account(&self, address: Address) -> Result<AccountResponse, ApiError> {
        let account = self.storage.state
            .get_account(&address)
            .map_err(|e| ApiError::InternalError(e.to_string()))?
            .unwrap_or_default();
        
        Ok(AccountResponse {
            address,
            balance: account.balance,
            nonce: account.nonce,
            code_hash: account.code_hash,
            storage_root: account.storage_root,
            model_permissions: account.model_permissions
                .into_iter()
                .map(|id| format!("{:?}", id))
                .collect(),
        })
    }
    
    /// Get account balance
    pub async fn get_balance(&self, address: Address) -> Result<U256, ApiError> {
        let balance = self.executor.get_balance(&address);
        Ok(balance)
    }
    
    /// Get account nonce
    pub async fn get_nonce(&self, address: Address) -> Result<u64, ApiError> {
        let nonce = self.executor.get_nonce(&address);
        Ok(nonce)
    }
    
    /// Get contract code
    pub async fn get_code(&self, address: Address) -> Result<Vec<u8>, ApiError> {
        let code_hash = self.executor.get_code_hash(&address);
        
        if code_hash == Hash::default() {
            return Ok(Vec::new());
        }
        
        self.storage.state
            .get_code(&code_hash)
            .map_err(|e| ApiError::InternalError(e.to_string()))?
            .ok_or_else(|| ApiError::InternalError("Code not found".into()))
    }
    
    /// Get storage value at key
    pub async fn get_storage(&self, address: Address, key: Vec<u8>) -> Result<Vec<u8>, ApiError> {
        let result = self.storage.state
            .get_storage(&address, &key)
            .map_err(|e| ApiError::InternalError(e.to_string()))?;
        
        Ok(result.unwrap_or_else(|| Vec::new()))
    }
    
    /// Get state root
    pub async fn get_state_root(&self) -> Result<Hash, ApiError> {
        let root = self.executor.calculate_state_root();
        Ok(root)
    }
}