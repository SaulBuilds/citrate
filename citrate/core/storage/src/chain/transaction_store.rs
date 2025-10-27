// citrate/core/storage/src/chain/transaction_store.rs

use crate::db::{column_families::*, RocksDB};
use anyhow::Result;
use citrate_consensus::types::{Hash, Transaction};
use citrate_execution::types::TransactionReceipt;
use std::sync::Arc;
use tracing::debug;

/// Transaction and receipt storage
pub struct TransactionStore {
    db: Arc<RocksDB>,
}

impl TransactionStore {
    pub fn new(db: Arc<RocksDB>) -> Self {
        Self { db }
    }

    /// Store a transaction
    pub fn put_transaction(&self, tx: &Transaction) -> Result<()> {
        let tx_bytes = bincode::serialize(tx)?;
        self.db
            .put_cf(CF_TRANSACTIONS, tx.hash.as_bytes(), &tx_bytes)?;

        // Index by sender
        let sender_key = sender_tx_key(&tx.from, tx.nonce);
        self.db
            .put_cf(CF_METADATA, &sender_key, tx.hash.as_bytes())?;

        debug!("Stored transaction {}", tx.hash);
        Ok(())
    }

    /// Store multiple transactions in batch
    pub fn put_transactions(&self, txs: &[Transaction]) -> Result<()> {
        let mut batch = self.db.batch();

        for tx in txs {
            let tx_bytes = bincode::serialize(tx)?;
            self.db
                .batch_put_cf(&mut batch, CF_TRANSACTIONS, tx.hash.as_bytes(), &tx_bytes)?;

            let sender_key = sender_tx_key(&tx.from, tx.nonce);
            self.db
                .batch_put_cf(&mut batch, CF_METADATA, &sender_key, tx.hash.as_bytes())?;
        }

        self.db.write_batch(batch)?;
        debug!("Stored {} transactions", txs.len());
        Ok(())
    }

    /// Get a transaction by hash
    pub fn get_transaction(&self, hash: &Hash) -> Result<Option<Transaction>> {
        match self.db.get_cf(CF_TRANSACTIONS, hash.as_bytes()) {
            Ok(Some(bytes)) => Ok(Some(bincode::deserialize(&bytes)?)),
            Ok(None) => Ok(None),
            Err(_e) => {
                // Fallback: iterate CF to find the key (workaround for rare I/O errors)
                for (k, v) in self.db.iter_cf(CF_TRANSACTIONS)? {
                    if k.as_ref() == hash.as_bytes() {
                        let tx: Transaction = bincode::deserialize(&v)?;
                        return Ok(Some(tx));
                    }
                }
                Ok(None)
            }
        }
    }

    /// Check if transaction exists
    pub fn has_transaction(&self, hash: &Hash) -> Result<bool> {
        self.db.exists_cf(CF_TRANSACTIONS, hash.as_bytes())
    }

    /// Store a transaction receipt
    pub fn put_receipt(&self, tx_hash: &Hash, receipt: &TransactionReceipt) -> Result<()> {
        let receipt_bytes = bincode::serialize(receipt)?;
        self.db
            .put_cf(CF_RECEIPTS, tx_hash.as_bytes(), &receipt_bytes)?;

        // Index by block
        let block_tx_key = block_tx_key(&receipt.block_hash, tx_hash);
        self.db.put_cf(CF_METADATA, &block_tx_key, &[])?;

        debug!("Stored receipt for transaction {}", tx_hash);
        Ok(())
    }

    /// Store multiple receipts in batch
    pub fn put_receipts(&self, receipts: &[(Hash, TransactionReceipt)]) -> Result<()> {
        let mut batch = self.db.batch();

        for (tx_hash, receipt) in receipts {
            let receipt_bytes = bincode::serialize(receipt)?;
            self.db
                .batch_put_cf(&mut batch, CF_RECEIPTS, tx_hash.as_bytes(), &receipt_bytes)?;

            let block_tx_key = block_tx_key(&receipt.block_hash, tx_hash);
            self.db
                .batch_put_cf(&mut batch, CF_METADATA, &block_tx_key, &[])?;
        }

        self.db.write_batch(batch)?;
        debug!("Stored {} receipts", receipts.len());
        Ok(())
    }

    /// Get a transaction receipt
    pub fn get_receipt(&self, tx_hash: &Hash) -> Result<Option<TransactionReceipt>> {
        match self.db.get_cf(CF_RECEIPTS, tx_hash.as_bytes())? {
            Some(bytes) => Ok(Some(bincode::deserialize(&bytes)?)),
            None => Ok(None),
        }
    }

    /// Get all transactions in a block
    pub fn get_block_transactions(&self, block_hash: &Hash) -> Result<Vec<Hash>> {
        let prefix = block_tx_prefix(block_hash);
        let mut tx_hashes = Vec::new();

        for (key, _) in self.db.prefix_iter_cf(CF_METADATA, &prefix)? {
            // Ensure the key actually starts with the expected prefix, since
            // RocksDB prefix iterator requires a configured prefix extractor.
            if key.starts_with(&prefix) && key.len() > prefix.len() {
                let tx_hash_bytes = &key[prefix.len()..];
                if tx_hash_bytes.len() == 32 {
                    let mut hash_array = [0u8; 32];
                    hash_array.copy_from_slice(tx_hash_bytes);
                    tx_hashes.push(Hash::new(hash_array));
                }
            }
        }

        Ok(tx_hashes)
    }

    /// Delete a transaction and its receipt
    pub fn delete_transaction(&self, hash: &Hash) -> Result<()> {
        let mut batch = self.db.batch();

        // Delete transaction
        self.db
            .batch_delete_cf(&mut batch, CF_TRANSACTIONS, hash.as_bytes())?;

        // Delete receipt if exists
        if let Some(receipt) = self.get_receipt(hash)? {
            self.db
                .batch_delete_cf(&mut batch, CF_RECEIPTS, hash.as_bytes())?;

            // Delete block index
            let block_tx_key = block_tx_key(&receipt.block_hash, hash);
            self.db
                .batch_delete_cf(&mut batch, CF_METADATA, &block_tx_key)?;
        }

        self.db.write_batch(batch)?;
        Ok(())
    }

    /// Compact transaction storage
    pub fn compact(&self) -> Result<()> {
        self.db.compact_cf(CF_TRANSACTIONS)?;
        self.db.compact_cf(CF_RECEIPTS)?;
        Ok(())
    }
}

// Key generation helpers
fn sender_tx_key(sender: &citrate_consensus::types::PublicKey, nonce: u64) -> Vec<u8> {
    let mut key = vec![b's'];
    key.extend_from_slice(sender.as_bytes());
    key.extend_from_slice(&nonce.to_be_bytes());
    key
}

fn block_tx_key(block_hash: &Hash, tx_hash: &Hash) -> Vec<u8> {
    let mut key = vec![b't'];
    key.extend_from_slice(block_hash.as_bytes());
    key.extend_from_slice(tx_hash.as_bytes());
    key
}

fn block_tx_prefix(block_hash: &Hash) -> Vec<u8> {
    let mut prefix = vec![b't'];
    prefix.extend_from_slice(block_hash.as_bytes());
    prefix
}

#[cfg(test)]
mod tests {
    use super::*;
    use citrate_consensus::types::{PublicKey, Signature};
    use citrate_execution::types::Address;
    use tempfile::TempDir;

    fn create_test_transaction(nonce: u64) -> Transaction {
        Transaction {
            hash: Hash::new([nonce as u8; 32]),
            nonce,
            from: PublicKey::new([1; 32]),
            to: Some(PublicKey::new([2; 32])),
            value: 1000,
            gas_limit: 100000,
            gas_price: 1000000000,
            data: vec![],
            signature: Signature::new([1; 64]),
            tx_type: None,
        }
    }

    fn create_test_receipt(tx_hash: Hash, block_hash: Hash) -> TransactionReceipt {
        TransactionReceipt {
            tx_hash,
            block_hash,
            block_number: 1,
            from: Address([1; 20]),
            to: Some(Address([2; 20])),
            gas_used: 21000,
            status: true,
            logs: vec![],
            output: vec![],
        }
    }

    #[test]
    fn test_transaction_storage() {
        let temp_dir = TempDir::new().unwrap();
        let db = Arc::new(RocksDB::open(temp_dir.path()).unwrap());
        let store = TransactionStore::new(db);

        // Store transaction
        let tx = create_test_transaction(1);
        store.put_transaction(&tx).unwrap();

        // Check existence
        assert!(store.has_transaction(&tx.hash).unwrap());
    }

    #[test]
    fn test_receipt_storage() {
        let temp_dir = TempDir::new().unwrap();
        let db = Arc::new(RocksDB::open(temp_dir.path()).unwrap());
        let store = TransactionStore::new(db);

        let tx_hash = Hash::new([1; 32]);
        let block_hash = Hash::new([2; 32]);
        let receipt = create_test_receipt(tx_hash, block_hash);

        // Store receipt
        store.put_receipt(&tx_hash, &receipt).unwrap();

        // Retrieve receipt
        let retrieved = store.get_receipt(&tx_hash).unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().block_hash, block_hash);

        // Get block transactions
        let block_txs = store.get_block_transactions(&block_hash).unwrap();
        assert_eq!(block_txs.len(), 1);
        assert_eq!(block_txs[0], tx_hash);
    }

    #[test]
    fn test_block_tx_prefix_roundtrip_multiple_blocks() {
        let temp_dir = TempDir::new().unwrap();
        let db = Arc::new(RocksDB::open(temp_dir.path()).unwrap());
        let store = TransactionStore::new(db);

        // Two blocks, two transactions each
        let block_a = Hash::new([0xA; 32]);
        let block_b = Hash::new([0xB; 32]);

        let tx_a1 = create_test_transaction(10);
        let tx_a2 = create_test_transaction(11);
        let tx_b1 = create_test_transaction(20);
        let tx_b2 = create_test_transaction(21);

        store.put_transaction(&tx_a1).unwrap();
        store.put_transaction(&tx_a2).unwrap();
        store.put_transaction(&tx_b1).unwrap();
        store.put_transaction(&tx_b2).unwrap();

        // Receipts map tx → block
        let rc_a1 = create_test_receipt(tx_a1.hash, block_a);
        let rc_a2 = create_test_receipt(tx_a2.hash, block_a);
        let rc_b1 = create_test_receipt(tx_b1.hash, block_b);
        let rc_b2 = create_test_receipt(tx_b2.hash, block_b);
        store.put_receipt(&tx_a1.hash, &rc_a1).unwrap();
        store.put_receipt(&tx_a2.hash, &rc_a2).unwrap();
        store.put_receipt(&tx_b1.hash, &rc_b1).unwrap();
        store.put_receipt(&tx_b2.hash, &rc_b2).unwrap();

        let block_a_txs = store.get_block_transactions(&block_a).unwrap();
        let block_b_txs = store.get_block_transactions(&block_b).unwrap();

        assert_eq!(block_a_txs.len(), 2);
        assert!(block_a_txs.contains(&tx_a1.hash));
        assert!(block_a_txs.contains(&tx_a2.hash));
        assert_eq!(block_b_txs.len(), 2);
        assert!(block_b_txs.contains(&tx_b1.hash));
        assert!(block_b_txs.contains(&tx_b2.hash));
    }
}
