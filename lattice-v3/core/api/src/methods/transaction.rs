use crate::types::{
    error::ApiError,
    request::{CallRequest, TransactionRequest},
};
use lattice_consensus::types::{Hash, PublicKey, Signature, Transaction};
use lattice_execution::executor::Executor;
use lattice_sequencer::mempool::{Mempool, TxClass};
use std::sync::Arc;

/// Transaction-related API methods
pub struct TransactionApi {
    mempool: Arc<Mempool>,
    executor: Arc<Executor>,
}

impl TransactionApi {
    pub fn new(mempool: Arc<Mempool>, executor: Arc<Executor>) -> Self {
        Self { mempool, executor }
    }

    /// Send raw transaction
    pub async fn send_raw_transaction(&self, raw_tx: Vec<u8>) -> Result<Hash, ApiError> {
        // Deserialize transaction
        let tx: Transaction = bincode::deserialize(&raw_tx)
            .map_err(|e| ApiError::InvalidTransaction(e.to_string()))?;

        let hash = tx.hash;

        // Add to mempool
        self.mempool
            .add_transaction(tx, TxClass::Standard)
            .await
            .map_err(|e| ApiError::InvalidTransaction(e.to_string()))?;

        Ok(hash)
    }

    /// Create and send transaction
    pub async fn send_transaction(&self, request: TransactionRequest) -> Result<Hash, ApiError> {
        // Get nonce if not provided
        let nonce = match request.nonce {
            Some(n) => n,
            None => self.executor.get_nonce(&request.from),
        };

        // Create transaction hash
        let mut hash_data = [0u8; 32];
        hash_data[0..8].copy_from_slice(&nonce.to_le_bytes());
        hash_data[8..16].copy_from_slice(&request.from.0[0..8]);
        if let Some(to) = &request.to {
            hash_data[16..24].copy_from_slice(&to.0[0..8]);
        }

        // Create transaction
        let mut tx = Transaction {
            hash: Hash::new(hash_data),
            nonce,
            from: PublicKey::new([0; 32]), // Would need proper key derivation
            to: request.to.map(|_| PublicKey::new([0; 32])),
            value: request.value.unwrap_or_default().as_u128(),
            gas_limit: request.gas.unwrap_or(21000),
            gas_price: request.gas_price.unwrap_or(1_000_000_000),
            data: request.data.unwrap_or_default(),
            signature: Signature::new([1; 64]), // Would need proper signing
            tx_type: None,
        };

        // Determine transaction type from data
        tx.determine_type();

        let hash = tx.hash;

        // Add to mempool
        self.mempool
            .add_transaction(tx, TxClass::Standard)
            .await
            .map_err(|e| ApiError::InvalidTransaction(e.to_string()))?;

        Ok(hash)
    }

    /// Estimate gas for transaction
    pub async fn estimate_gas(&self, request: CallRequest) -> Result<u64, ApiError> {
        // Basic gas estimation
        let base_gas = 21000u64;
        let data_gas = request.data.as_ref().map_or(0, |d| {
            d.iter()
                .map(|&byte| if byte == 0 { 4 } else { 68 })
                .sum::<u64>()
        });

        Ok(base_gas + data_gas)
    }

    /// Get current gas price
    pub async fn get_gas_price(&self) -> Result<u64, ApiError> {
        // Return minimum gas price for now
        Ok(1_000_000_000) // 1 Gwei
    }

    /// Get transaction count (nonce) for address
    pub async fn get_transaction_count(
        &self,
        address: lattice_execution::types::Address,
    ) -> Result<u64, ApiError> {
        Ok(self.executor.get_nonce(&address))
    }
}
