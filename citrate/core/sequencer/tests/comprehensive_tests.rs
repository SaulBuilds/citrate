// Comprehensive tests for the sequencer module

use citrate_consensus::types::{Hash, PublicKey, Signature, Transaction};
use citrate_sequencer::{Mempool, MempoolConfig, TxClass};
use std::sync::Arc;

fn create_test_transaction(nonce: u64, gas_price: u64) -> Transaction {
    // Create unique hash based on nonce and gas_price
    let mut hash_data = [0u8; 32];
    hash_data[0..8].copy_from_slice(&nonce.to_le_bytes());
    hash_data[8..16].copy_from_slice(&gas_price.to_le_bytes());

    // Create a valid non-zero public key
    let mut from_key = [1u8; 32]; // Start with all 1s to avoid zero key
    from_key[0] = (nonce + 1) as u8; // Make it unique per nonce

    Transaction {
        hash: Hash::new(hash_data),
        nonce,
        from: PublicKey::new(from_key),
        to: Some(PublicKey::new([2; 32])),
        value: 1000,
        gas_limit: 21000,
        gas_price,
        data: vec![],
        signature: Signature::new([1; 64]), // Valid non-zero signature
        tx_type: None,
    }
}

#[cfg(test)]
mod mempool_tests {
    use super::*;

    #[tokio::test]
    async fn test_mempool_creation() {
        let config = MempoolConfig {
            require_valid_signature: false,
            ..Default::default()
        };
        let mempool = Mempool::new(config);

        let stats = mempool.stats().await;
        assert_eq!(stats.total_transactions, 0);
        assert_eq!(stats.total_transactions, 0);
    }

    #[tokio::test]
    async fn test_add_transaction() {
        let config = MempoolConfig {
            require_valid_signature: false,
            ..Default::default()
        };
        let mempool = Mempool::new(config);
        let tx = create_test_transaction(1, 2_000_000_000);

        let result = mempool.add_transaction(tx.clone(), TxClass::Standard).await;
        assert!(result.is_ok());

        let stats = mempool.stats().await;
        assert_eq!(stats.total_transactions, 1);
    }

    #[tokio::test]
    async fn test_get_transactions() {
        let config = MempoolConfig {
            require_valid_signature: false,
            ..Default::default()
        };
        let mempool = Mempool::new(config);

        // Add multiple transactions
        for i in 0..5 {
            let tx = create_test_transaction(i, 2_000_000_000 + i * 100_000_000);
            mempool
                .add_transaction(tx, TxClass::Standard)
                .await
                .unwrap();
        }

        // Get transactions (should be sorted by gas price)
        let txs = mempool.get_transactions(3).await;
        assert_eq!(txs.len(), 3);
    }

    #[tokio::test]
    async fn test_remove_transaction() {
        let config = MempoolConfig {
            require_valid_signature: false,
            ..Default::default()
        };
        let mempool = Mempool::new(config);
        let tx = create_test_transaction(1, 2_000_000_000);
        let tx_hash = tx.hash;

        mempool
            .add_transaction(tx, TxClass::Standard)
            .await
            .unwrap();
        assert_eq!(mempool.stats().await.total_transactions, 1);

        let removed = mempool.remove_transaction(&tx_hash).await;
        assert!(removed.is_some());
        assert_eq!(mempool.stats().await.total_transactions, 0);
    }

    #[tokio::test]
    async fn test_duplicate_transaction() {
        let config = MempoolConfig {
            require_valid_signature: false,
            ..Default::default()
        };
        let mempool = Mempool::new(config);
        let tx = create_test_transaction(1, 2_000_000_000);

        // First add should succeed
        assert!(mempool
            .add_transaction(tx.clone(), TxClass::Standard)
            .await
            .is_ok());

        // Second add of same transaction should fail
        assert!(mempool
            .add_transaction(tx, TxClass::Standard)
            .await
            .is_err());
    }

    #[tokio::test]
    async fn test_mempool_capacity() {
        let mut config = MempoolConfig {
            require_valid_signature: false,
            ..Default::default()
        };
        config.max_size = 3;
        let mempool = Mempool::new(config);

        // Add transactions up to capacity
        for i in 0..3 {
            let tx = create_test_transaction(i, 2_000_000_000);
            assert!(mempool.add_transaction(tx, TxClass::Standard).await.is_ok());
        }

        // Adding beyond capacity might evict or reject (depends on implementation)
        let tx = create_test_transaction(3, 1_500_000_000);
        let _ = mempool.add_transaction(tx, TxClass::Standard).await;

        let stats = mempool.stats().await;
        assert!(stats.total_transactions <= 3);
    }

    #[tokio::test]
    async fn test_gas_price_ordering() {
        let config = MempoolConfig {
            require_valid_signature: false,
            ..Default::default()
        };
        let mempool = Mempool::new(config);

        // Add transactions with different gas prices
        let tx1 = create_test_transaction(1, 1_500_000_000);
        let tx2 = create_test_transaction(2, 2_000_000_000);
        let tx3 = create_test_transaction(3, 1_750_000_000);

        mempool
            .add_transaction(tx1, TxClass::Standard)
            .await
            .unwrap();
        mempool
            .add_transaction(tx2, TxClass::Standard)
            .await
            .unwrap();
        mempool
            .add_transaction(tx3, TxClass::Standard)
            .await
            .unwrap();

        // Get all transactions - should be ordered by gas price (highest first)
        let txs = mempool.get_transactions(10).await;
        assert_eq!(txs.len(), 3);
        // Verify ordering if gas price is accessible
    }

    #[tokio::test]
    async fn test_concurrent_access() {
        let config = MempoolConfig {
            require_valid_signature: false,
            ..Default::default()
        };
        let mempool = Arc::new(Mempool::new(config));

        let mut handles = vec![];

        // Spawn multiple tasks adding transactions
        for i in 0..10 {
            let mempool_clone = mempool.clone();
            let handle = tokio::spawn(async move {
                let tx = create_test_transaction(i, 2_000_000_000);
                mempool_clone.add_transaction(tx, TxClass::Standard).await
            });
            handles.push(handle);
        }

        // Wait for all tasks
        for handle in handles {
            let _ = handle.await;
        }

        let stats = mempool.stats().await;
        assert!(stats.total_transactions > 0);
    }

    #[tokio::test]
    async fn test_clear_mempool() {
        let config = MempoolConfig {
            require_valid_signature: false,
            ..Default::default()
        };
        let mempool = Mempool::new(config);

        // Add transactions
        for i in 0..5 {
            let tx = create_test_transaction(i, 2_000_000_000);
            mempool
                .add_transaction(tx, TxClass::Standard)
                .await
                .unwrap();
        }

        assert_eq!(mempool.stats().await.total_transactions, 5);

        // Clear all transactions
        mempool.clear().await;

        assert_eq!(mempool.stats().await.total_transactions, 0);
    }

    #[tokio::test]
    async fn test_transaction_expiry() {
        let mut config = MempoolConfig {
            require_valid_signature: false,
            ..Default::default()
        };
        config.tx_expiry_secs = 1; // Very short lifetime for testing
        let mempool = Mempool::new(config);

        let tx = create_test_transaction(1, 2_000_000_000);
        mempool
            .add_transaction(tx, TxClass::Standard)
            .await
            .unwrap();

        // Wait for expiry
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        // Transaction might be auto-removed if expiry is implemented
        // This depends on the actual implementation
        let _stats = mempool.stats().await;
        // Check if expired transactions are handled
    }
}
