// citrate/core/sequencer/src/mempool_tests.rs

#[cfg(test)]
mod tests {
    use super::super::*;
    use citrate_consensus::types::{
        Transaction, TransactionClass, Hash, PublicKey, Signature,
    };
    use std::sync::Arc;
    use tokio::time::{sleep, Duration};

    fn create_test_tx(
        sender: PublicKey,
        nonce: u64,
        gas_price: u64,
        class: TransactionClass,
    ) -> Transaction {
        Transaction {
            hash: Hash::random(),
            sender,
            nonce,
            gas_price,
            gas_limit: 100000,
            class,
            data: vec![],
            signature: Signature::default(),
        }
    }

    #[tokio::test]
    async fn test_mempool_add_and_get() {
        let config = MempoolConfig::default();
        let mempool = Mempool::new(config);

        let sender = PublicKey::default();
        let tx = create_test_tx(sender, 0, 100, TransactionClass::Standard);
        let tx_hash = tx.hash;

        mempool.add_transaction(tx.clone()).await.unwrap();

        let retrieved = mempool.get_transaction(&tx_hash).await.unwrap();
        assert_eq!(retrieved.hash, tx_hash);
    }

    #[tokio::test]
    async fn test_mempool_priority_ordering() {
        let config = MempoolConfig::default();
        let mempool = Mempool::new(config);

        let sender = PublicKey::default();
        
        let tx1 = create_test_tx(sender, 0, 50, TransactionClass::Standard);
        let tx2 = create_test_tx(sender, 1, 100, TransactionClass::Standard);
        let tx3 = create_test_tx(sender, 2, 75, TransactionClass::ModelUpdate);

        mempool.add_transaction(tx1.clone()).await.unwrap();
        mempool.add_transaction(tx2.clone()).await.unwrap();
        mempool.add_transaction(tx3.clone()).await.unwrap();

        let ready = mempool.get_ready_transactions(10).await.unwrap();
        
        assert_eq!(ready.len(), 3);
        assert_eq!(ready[0].hash, tx3.hash);
        assert_eq!(ready[1].hash, tx2.hash);
        assert_eq!(ready[2].hash, tx1.hash);
    }

    #[tokio::test]
    async fn test_mempool_nonce_ordering() {
        let config = MempoolConfig::default();
        let mempool = Mempool::new(config);

        let sender = PublicKey::default();
        
        let tx2 = create_test_tx(sender, 2, 100, TransactionClass::Standard);
        let tx0 = create_test_tx(sender, 0, 100, TransactionClass::Standard);
        let tx1 = create_test_tx(sender, 1, 100, TransactionClass::Standard);

        mempool.add_transaction(tx2.clone()).await.unwrap();
        mempool.add_transaction(tx0.clone()).await.unwrap();
        mempool.add_transaction(tx1.clone()).await.unwrap();

        let ready = mempool.get_ready_transactions(10).await.unwrap();
        
        assert_eq!(ready.len(), 3);
        assert_eq!(ready[0].nonce, 0);
        assert_eq!(ready[1].nonce, 1);
        assert_eq!(ready[2].nonce, 2);
    }

    #[tokio::test]
    async fn test_mempool_duplicate_rejection() {
        let config = MempoolConfig::default();
        let mempool = Mempool::new(config);

        let sender = PublicKey::default();
        let tx = create_test_tx(sender, 0, 100, TransactionClass::Standard);

        mempool.add_transaction(tx.clone()).await.unwrap();
        
        let result = mempool.add_transaction(tx.clone()).await;
        assert!(matches!(result, Err(MempoolError::DuplicateTransaction(_))));
    }

    #[tokio::test]
    async fn test_mempool_size_limit() {
        let mut config = MempoolConfig::default();
        config.max_size = 500;
        let mempool = Mempool::new(config);

        let sender = PublicKey::default();
        
        for i in 0..10 {
            let tx = create_test_tx(sender, i, 100 - i, TransactionClass::Standard);
            let _ = mempool.add_transaction(tx).await;
        }

        let stats = mempool.get_stats().await;
        assert!(stats.total_size <= 500);
    }

    #[tokio::test]
    async fn test_mempool_remove_transaction() {
        let config = MempoolConfig::default();
        let mempool = Mempool::new(config);

        let sender = PublicKey::default();
        let tx = create_test_tx(sender, 0, 100, TransactionClass::Standard);
        let tx_hash = tx.hash;

        mempool.add_transaction(tx).await.unwrap();
        assert!(mempool.get_transaction(&tx_hash).await.is_some());

        mempool.remove_transaction(&tx_hash).await.unwrap();
        assert!(mempool.get_transaction(&tx_hash).await.is_none());
    }

    #[tokio::test]
    async fn test_mempool_clear() {
        let config = MempoolConfig::default();
        let mempool = Mempool::new(config);

        let sender = PublicKey::default();
        
        for i in 0..5 {
            let tx = create_test_tx(sender, i, 100, TransactionClass::Standard);
            mempool.add_transaction(tx).await.unwrap();
        }

        let stats = mempool.get_stats().await;
        assert_eq!(stats.transaction_count, 5);

        mempool.clear().await;

        let stats = mempool.get_stats().await;
        assert_eq!(stats.transaction_count, 0);
    }

    #[tokio::test]
    async fn test_mempool_eviction() {
        let mut config = MempoolConfig::default();
        config.max_transactions = 3;
        let mempool = Mempool::new(config);

        let sender = PublicKey::default();
        
        for i in 0..5 {
            let tx = create_test_tx(sender, i, 100 - i * 10, TransactionClass::Standard);
            let _ = mempool.add_transaction(tx).await;
        }

        let stats = mempool.get_stats().await;
        assert_eq!(stats.transaction_count, 3);
        
        let ready = mempool.get_ready_transactions(10).await.unwrap();
        assert_eq!(ready[0].gas_price, 100);
    }

    #[tokio::test]
    async fn test_mempool_transaction_classes() {
        let config = MempoolConfig::default();
        let mempool = Mempool::new(config);

        let sender = PublicKey::default();
        
        let classes = vec![
            TransactionClass::Standard,
            TransactionClass::ModelUpdate,
            TransactionClass::Inference,
            TransactionClass::Training,
            TransactionClass::Storage,
            TransactionClass::System,
        ];

        for (i, class) in classes.iter().enumerate() {
            let tx = create_test_tx(sender, i as u64, 100, class.clone());
            mempool.add_transaction(tx).await.unwrap();
        }

        let ready = mempool.get_ready_transactions(10).await.unwrap();
        assert_eq!(ready.len(), 6);
        
        assert_eq!(ready[0].class, TransactionClass::System);
        assert_eq!(ready[5].class, TransactionClass::Standard);
    }

    #[tokio::test]
    async fn test_mempool_concurrent_operations() {
        let config = MempoolConfig::default();
        let mempool = Arc::new(Mempool::new(config));

        let mut handles = vec![];

        for i in 0..10 {
            let mempool_clone = mempool.clone();
            let handle = tokio::spawn(async move {
                let sender = PublicKey::random();
                let tx = create_test_tx(sender, 0, 100 + i, TransactionClass::Standard);
                mempool_clone.add_transaction(tx).await
            });
            handles.push(handle);
        }

        for handle in handles {
            let _ = handle.await.unwrap();
        }

        let stats = mempool.get_stats().await;
        assert!(stats.transaction_count > 0);
    }
}