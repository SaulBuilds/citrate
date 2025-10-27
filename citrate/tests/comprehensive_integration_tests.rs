// Comprehensive integration tests for Citrate V3

use citrate_consensus::{GhostDag, DagStore, GhostDagParams};
use citrate_storage::{StorageManager, pruning::PruningConfig};
use citrate_execution::Executor;
use citrate_sequencer::{Mempool, MempoolConfig};
use citrate_network::PeerManager;
use citrate_consensus::types::*;
use std::sync::Arc;
use tempfile::TempDir;

fn create_test_block(num: u8, height: u64) -> Block {
    let mut hash_bytes = [0u8; 32];
    hash_bytes[0] = num;

    Block {
        header: BlockHeader {
            version: 1,
            block_hash: Hash::new(hash_bytes),
            selected_parent_hash: Hash::default(),
            merge_parent_hashes: vec![],
            timestamp: 1000000 + height * 10,
            height,
            blue_score: height * 10,
            blue_work: (height as u128) * 1000,
            pruning_point: Hash::default(),
            proposer_pubkey: PublicKey::new([0u8; 32]),
            vrf_reveal: VrfProof {
                proof: vec![0u8; 80],
                output: Hash::default(),
            },
        },
        state_root: Hash::default(),
        tx_root: Hash::default(),
        receipt_root: Hash::default(),
        artifact_root: Hash::default(),
        ghostdag_params: GhostDagParams::default(),
        transactions: vec![],
        signature: Signature::new([0u8; 64]),
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_full_block_flow() {
        // Setup storage
        let temp_dir = TempDir::new().unwrap();
        let storage = Arc::new(
            StorageManager::new(temp_dir.path(), PruningConfig::default()).unwrap()
        );

        // Setup consensus
        let dag_store = Arc::new(DagStore::new());
        let ghostdag = Arc::new(GhostDag::new(GhostDagParams::default(), dag_store.clone()));

        // Create and process block
        let block = create_test_block(1, 0);

        // Store in DAG
        dag_store.store_block(block.clone()).await.unwrap();

        // Store in persistent storage
        storage.blocks.put_block(&block).unwrap();

        // Verify block is stored
        let retrieved = storage.blocks.get_block(&block.hash()).unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().hash(), block.hash());
    }

    #[tokio::test]
    async fn test_mempool_to_block_flow() {
        // Setup components
        let mempool = Arc::new(Mempool::new(MempoolConfig::default()));
        let temp_dir = TempDir::new().unwrap();
        let storage = Arc::new(
            StorageManager::new(temp_dir.path(), PruningConfig::default()).unwrap()
        );

        // Add transactions to mempool
        for i in 0..5 {
            let tx = Transaction {
                version: 1,
                inputs: vec![],
                outputs: vec![],
                timestamp: 1000000 + i,
                signature: Signature::new([i as u8; 64]),
            };
            mempool.add_transaction(tx).await.unwrap();
        }

        // Get transactions for block
        let txs = mempool.get_transactions(10).await;
        assert_eq!(txs.len(), 5);

        // Create block with transactions
        let mut block = create_test_block(1, 1);
        block.transactions = txs;

        // Store block
        storage.blocks.put_block(&block).unwrap();

        // Clear mempool
        mempool.clear().await;
        assert_eq!(mempool.stats().await.total_transactions, 0);
    }

    #[tokio::test]
    async fn test_consensus_and_storage_integration() {
        // Setup
        let temp_dir = TempDir::new().unwrap();
        let storage = Arc::new(
            StorageManager::new(temp_dir.path(), PruningConfig::default()).unwrap()
        );
        let dag_store = Arc::new(DagStore::new());
        let ghostdag = Arc::new(GhostDag::new(GhostDagParams::default(), dag_store.clone()));

        // Create chain of blocks
        let mut prev_hash = Hash::default();
        for i in 0..10 {
            let mut block = create_test_block(i, i as u64);
            block.header.selected_parent_hash = prev_hash;

            // Process in consensus
            dag_store.store_block(block.clone()).await.unwrap();

            // Store persistently
            storage.blocks.put_block(&block).unwrap();

            prev_hash = block.hash();
        }

        // Verify chain
        assert_eq!(storage.blocks.get_latest_height().unwrap(), 9);

        // Verify DAG tips
        let tips = dag_store.get_tips().await;
        assert_eq!(tips.len(), 1);
        assert_eq!(tips[0].hash, prev_hash);
    }

    #[tokio::test]
    async fn test_executor_state_persistence() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().to_path_buf();

        // First session - create state
        {
            let executor = Executor::new(&path).unwrap();
            let addr = citrate_execution::types::Address([1u8; 20]);

            executor.set_balance(&addr, primitive_types::U256::from(1000));
            executor.set_nonce(&addr, 5);
            executor.state_db().commit();
        }

        // Second session - verify state persisted
        {
            let executor = Executor::new(&path).unwrap();
            let addr = citrate_execution::types::Address([1u8; 20]);

            assert_eq!(executor.get_balance(&addr), primitive_types::U256::from(1000));
            assert_eq!(executor.get_nonce(&addr), 5);
        }
    }

    #[tokio::test]
    async fn test_network_and_consensus_integration() {
        // Setup network
        let peer_manager = Arc::new(PeerManager::new(9500).unwrap());

        // Setup consensus
        let dag_store = Arc::new(DagStore::new());
        let ghostdag = Arc::new(GhostDag::new(GhostDagParams::default(), dag_store.clone()));

        // Create block
        let block = create_test_block(1, 1);

        // Process block
        dag_store.store_block(block.clone()).await.unwrap();

        // Broadcast to network
        let msg = citrate_network::NetworkMessage::NewBlock {
            block: block.clone()
        };
        let _ = peer_manager.broadcast(&msg).await;

        // Verify block in DAG
        assert!(dag_store.has_block(&block.hash()).await);
    }

    #[tokio::test]
    async fn test_fork_resolution() {
        let dag_store = Arc::new(DagStore::new());
        let ghostdag = Arc::new(GhostDag::new(GhostDagParams::default(), dag_store.clone()));

        // Create genesis
        let genesis = create_test_block(0, 0);
        dag_store.store_block(genesis.clone()).await.unwrap();

        // Create two competing chains
        let mut fork1 = create_test_block(1, 1);
        fork1.header.selected_parent_hash = genesis.hash();
        fork1.header.blue_score = 100;

        let mut fork2 = create_test_block(2, 1);
        fork2.header.selected_parent_hash = genesis.hash();
        fork2.header.blue_score = 50;

        dag_store.store_block(fork1.clone()).await.unwrap();
        dag_store.store_block(fork2.clone()).await.unwrap();

        // Check tips - both should be tips
        let tips = dag_store.get_tips().await;
        assert_eq!(tips.len(), 2);
    }

    #[tokio::test]
    async fn test_pruning_integration() {
        let temp_dir = TempDir::new().unwrap();
        let mut config = PruningConfig::default();
        config.enable_pruning = true;
        config.pruning_height = 50;

        let storage = Arc::new(
            StorageManager::new(temp_dir.path(), config).unwrap()
        );

        // Add many blocks
        for i in 0..100 {
            let block = create_test_block((i % 256) as u8, i);
            storage.blocks.put_block(&block).unwrap();
        }

        // Trigger pruning
        storage.pruner.prune_old_blocks(50).await.unwrap();

        // Old blocks might be pruned (depends on implementation)
        let latest = storage.blocks.get_latest_height().unwrap();
        assert_eq!(latest, 99);
    }

    #[tokio::test]
    async fn test_concurrent_component_access() {
        use std::sync::Arc;
        use tokio::task;

        let temp_dir = TempDir::new().unwrap();
        let storage = Arc::new(
            StorageManager::new(temp_dir.path(), PruningConfig::default()).unwrap()
        );
        let mempool = Arc::new(Mempool::new(MempoolConfig::default()));

        let mut handles = vec![];

        // Storage writer task
        let storage_clone = storage.clone();
        handles.push(task::spawn(async move {
            for i in 0..10 {
                let block = create_test_block(i, i as u64);
                storage_clone.blocks.put_block(&block).unwrap();
            }
        }));

        // Mempool writer task
        let mempool_clone = mempool.clone();
        handles.push(task::spawn(async move {
            for i in 0..10 {
                let tx = Transaction {
                    version: 1,
                    inputs: vec![],
                    outputs: vec![],
                    timestamp: 1000000 + i,
                    signature: Signature::new([i as u8; 64]),
                };
                let _ = mempool_clone.add_transaction(tx).await;
            }
        }));

        // Wait for all tasks
        for handle in handles {
            handle.await.unwrap();
        }

        // Verify data written
        assert!(storage.blocks.get_latest_height().unwrap() > 0);
        assert!(mempool.stats().await.total_transactions > 0);
    }

    #[tokio::test]
    async fn test_state_sync_simulation() {
        // Simulate state sync between two nodes
        let temp_dir1 = TempDir::new().unwrap();
        let temp_dir2 = TempDir::new().unwrap();

        let storage1 = Arc::new(
            StorageManager::new(temp_dir1.path(), PruningConfig::default()).unwrap()
        );
        let storage2 = Arc::new(
            StorageManager::new(temp_dir2.path(), PruningConfig::default()).unwrap()
        );

        // Node 1 creates blocks
        for i in 0..5 {
            let block = create_test_block(i, i as u64);
            storage1.blocks.put_block(&block).unwrap();
        }

        // Simulate sync to Node 2
        for i in 0..5 {
            let block = create_test_block(i, i as u64);
            let retrieved = storage1.blocks.get_block(&block.hash()).unwrap().unwrap();
            storage2.blocks.put_block(&retrieved).unwrap();
        }

        // Verify both nodes have same state
        assert_eq!(
            storage1.blocks.get_latest_height().unwrap(),
            storage2.blocks.get_latest_height().unwrap()
        );
    }
}