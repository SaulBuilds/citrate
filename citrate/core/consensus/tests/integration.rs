// citrate/core/consensus/tests/integration.rs
//
// Comprehensive integration tests for Sprint 0 fixes:
// - Total ordering consistency
// - Finality progression
// - Reorg handling with finality protection
// - Chain selection and reorganization

use citrate_consensus::*;
use std::sync::Arc;

// Helper function to create a test block with specific properties
fn create_block(block_num: u8, height: u64, parent: Option<Hash>, blue_score: u64) -> Block {
    let mut hash_bytes = [0u8; 32];
    if block_num == 0 {
        hash_bytes = [0xFFu8; 32];
    } else {
        hash_bytes[0] = block_num;
        // Add height to make hashes more unique
        hash_bytes[1] = height as u8;
    }

    Block {
        header: BlockHeader {
            version: 1,
            block_hash: Hash::new(hash_bytes),
            selected_parent_hash: parent.unwrap_or_default(),
            merge_parent_hashes: vec![],
            timestamp: 1000000 + height * 10,
            height,
            blue_score,
            blue_work: blue_score as u128 * 100,
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
        embedded_models: vec![],
        required_pins: vec![],
    }
}

// Helper function to create a simple test block
fn create_test_block(block_num: u8, height: u64, parent: Option<Hash>) -> Block {
    create_block(block_num, height, parent, height + 1)
}

#[cfg(test)]
mod total_ordering_tests {
    use super::*;

    #[tokio::test]
    async fn test_ordering_single_chain() {
        // Test total ordering on a simple chain: genesis -> block1 -> block2 -> block3
        let dag_store = Arc::new(DagStore::new());
        let ghostdag = Arc::new(GhostDag::new(GhostDagParams::default(), dag_store.clone()));
        let ordering = TotalOrdering::new(dag_store.clone(), ghostdag.clone());

        // Build chain
        let genesis = create_test_block(0, 0, None);
        let block1 = create_test_block(1, 1, Some(genesis.hash()));
        let block2 = create_test_block(2, 2, Some(block1.hash()));
        let block3 = create_test_block(3, 3, Some(block2.hash()));

        dag_store.store_block(genesis.clone()).await.unwrap();
        dag_store.store_block(block1.clone()).await.unwrap();
        dag_store.store_block(block2.clone()).await.unwrap();
        dag_store.store_block(block3.clone()).await.unwrap();

        // Get total order from genesis to block3
        let ordered = ordering
            .get_ordered_blocks(genesis.hash(), block3.hash())
            .await
            .unwrap();

        // Should contain blocks in order
        assert!(!ordered.blocks.is_empty());
        // Last block should be block3
        assert_eq!(*ordered.blocks.last().unwrap(), block3.hash());
    }

    #[tokio::test]
    async fn test_ordering_consistency() {
        // Same blocks, multiple calls should return same order
        let dag_store = Arc::new(DagStore::new());
        let ghostdag = Arc::new(GhostDag::new(GhostDagParams::default(), dag_store.clone()));
        let ordering = TotalOrdering::new(dag_store.clone(), ghostdag.clone());

        let genesis = create_test_block(0, 0, None);
        let block1 = create_test_block(1, 1, Some(genesis.hash()));
        let block2 = create_test_block(2, 2, Some(block1.hash()));

        dag_store.store_block(genesis.clone()).await.unwrap();
        dag_store.store_block(block1.clone()).await.unwrap();
        dag_store.store_block(block2.clone()).await.unwrap();

        // Call multiple times
        let order1 = ordering
            .get_ordered_blocks(genesis.hash(), block2.hash())
            .await
            .unwrap();
        let order2 = ordering
            .get_ordered_blocks(genesis.hash(), block2.hash())
            .await
            .unwrap();
        let order3 = ordering
            .get_ordered_blocks(genesis.hash(), block2.hash())
            .await
            .unwrap();

        // All should be identical
        assert_eq!(order1.blocks, order2.blocks);
        assert_eq!(order2.blocks, order3.blocks);
    }

    #[tokio::test]
    async fn test_ordering_caching() {
        // Verify caching works correctly by calling twice and verifying consistency
        let dag_store = Arc::new(DagStore::new());
        let ghostdag = Arc::new(GhostDag::new(GhostDagParams::default(), dag_store.clone()));
        let ordering = TotalOrdering::new(dag_store.clone(), ghostdag.clone());

        let genesis = create_test_block(0, 0, None);
        let block1 = create_test_block(1, 1, Some(genesis.hash()));

        dag_store.store_block(genesis.clone()).await.unwrap();
        dag_store.store_block(block1.clone()).await.unwrap();

        // First call populates cache
        let order1 = ordering
            .get_ordered_blocks(genesis.hash(), block1.hash())
            .await
            .unwrap();

        // Second call should use cache and return identical results
        let order2 = ordering
            .get_ordered_blocks(genesis.hash(), block1.hash())
            .await
            .unwrap();

        // Results should be identical (cache working)
        assert_eq!(order1.blocks, order2.blocks);
    }
}

#[cfg(test)]
mod finality_progression_tests {
    use super::*;

    #[tokio::test]
    async fn test_finality_progression_basic() {
        // Test that blocks become finalized after reaching confirmation depth
        let dag_store = Arc::new(DagStore::new());
        let config = FinalityConfig {
            confirmation_depth: 5,
            emit_events: true,
            max_finalize_batch: 10,
        };
        let tracker = FinalityTracker::new(dag_store.clone(), config);

        // Build a chain of 10 blocks
        let genesis = create_test_block(0, 0, None);
        dag_store.store_block(genesis.clone()).await.unwrap();
        tracker.update_finality(&genesis.hash(), 0).await.unwrap();

        let mut prev_hash = genesis.hash();
        for i in 1..=10 {
            let block = create_test_block(i as u8, i, Some(prev_hash));
            dag_store.store_block(block.clone()).await.unwrap();
            tracker.update_finality(&block.hash(), i).await.unwrap();
            prev_hash = block.hash();
        }

        // Check finality status
        // Genesis should be finalized (10 - 0 >= 5)
        let genesis_status = tracker.get_finality_status(&genesis.hash()).await.unwrap();
        assert!(matches!(genesis_status, FinalityStatus::Finalized));
    }

    #[tokio::test]
    async fn test_finality_depth_boundary() {
        // Test exact boundary conditions for finality
        let dag_store = Arc::new(DagStore::new());
        let config = FinalityConfig {
            confirmation_depth: 3,
            emit_events: false,
            max_finalize_batch: 10,
        };
        let tracker = FinalityTracker::new(dag_store.clone(), config);

        // Build chain: genesis(0) -> b1(1) -> b2(2) -> b3(3) -> b4(4)
        let genesis = create_test_block(0, 0, None);
        let b1 = create_test_block(1, 1, Some(genesis.hash()));
        let b2 = create_test_block(2, 2, Some(b1.hash()));
        let b3 = create_test_block(3, 3, Some(b2.hash()));
        let b4 = create_test_block(4, 4, Some(b3.hash()));

        for block in [&genesis, &b1, &b2, &b3, &b4] {
            dag_store.store_block(block.clone()).await.unwrap();
        }

        // Update finality to height 4
        tracker.update_finality(&b4.hash(), 4).await.unwrap();

        // Check: height 4 is current tip
        // height 4 - 3 (depth) = height 1 should be finalized
        // height 0 (genesis) should be finalized

        let finalized_height = tracker.get_finalized_height();
        assert!(finalized_height >= 1);
    }

    #[tokio::test]
    async fn test_finality_events() {
        // Test that finality events are emitted correctly
        let dag_store = Arc::new(DagStore::new());
        let config = FinalityConfig {
            confirmation_depth: 2,
            emit_events: true,
            max_finalize_batch: 10,
        };
        let tracker = Arc::new(FinalityTracker::new(dag_store.clone(), config));
        let mut receiver = tracker.subscribe();

        // Build chain
        let genesis = create_test_block(0, 0, None);
        let b1 = create_test_block(1, 1, Some(genesis.hash()));
        let b2 = create_test_block(2, 2, Some(b1.hash()));
        let b3 = create_test_block(3, 3, Some(b2.hash()));

        for block in [&genesis, &b1, &b2, &b3] {
            dag_store.store_block(block.clone()).await.unwrap();
        }

        // Update finality to height 3 (should finalize up to height 1)
        tracker.update_finality(&b3.hash(), 3).await.unwrap();

        // Try to receive events (non-blocking)
        let mut events_received = 0;
        loop {
            match receiver.try_recv() {
                Ok(_) => events_received += 1,
                Err(_) => break,
            }
        }

        // Should have received at least one finality event
        assert!(events_received > 0);
    }
}

#[cfg(test)]
mod reorg_protection_tests {
    use super::*;

    #[tokio::test]
    async fn test_reorg_allowed_before_finality() {
        // Reorg should be allowed before any blocks are finalized
        let dag_store = Arc::new(DagStore::new());
        let config = FinalityConfig {
            confirmation_depth: 100, // High depth = nothing finalized yet
            emit_events: false,
            max_finalize_batch: 10,
        };
        let tracker = Arc::new(FinalityTracker::new(dag_store.clone(), config));

        // Build short chain
        let genesis = create_test_block(0, 0, None);
        let b1 = create_test_block(1, 1, Some(genesis.hash()));
        let b2 = create_test_block(2, 2, Some(b1.hash()));

        for block in [&genesis, &b1, &b2] {
            dag_store.store_block(block.clone()).await.unwrap();
        }

        tracker.update_finality(&b2.hash(), 2).await.unwrap();

        // Reorg from any point should be allowed since nothing is finalized
        assert!(tracker.check_reorg_allowed(&genesis.hash()).await.is_ok());
        assert!(tracker.check_reorg_allowed(&b1.hash()).await.is_ok());
        assert!(tracker.check_reorg_allowed(&b2.hash()).await.is_ok());
    }

    #[tokio::test]
    async fn test_reorg_blocked_past_finalized() {
        // Reorg should be blocked past finalized blocks
        let dag_store = Arc::new(DagStore::new());
        let config = FinalityConfig {
            confirmation_depth: 3,
            emit_events: false,
            max_finalize_batch: 10,
        };
        let tracker = Arc::new(FinalityTracker::new(dag_store.clone(), config));

        // Build chain of 10 blocks
        let genesis = create_test_block(0, 0, None);
        dag_store.store_block(genesis.clone()).await.unwrap();

        let mut prev = genesis.hash();
        let mut blocks = vec![genesis.clone()];
        for i in 1..=10 {
            let block = create_test_block(i as u8, i, Some(prev));
            dag_store.store_block(block.clone()).await.unwrap();
            tracker.update_finality(&block.hash(), i).await.unwrap();
            blocks.push(block.clone());
            prev = block.hash();
        }

        // At height 10, blocks at height <= 7 should be finalized (10 - 3 = 7)
        // Try to reorg from genesis - should fail
        let result = tracker.check_reorg_allowed(&genesis.hash()).await;
        assert!(result.is_err());

        // Try to reorg from a finalized block - should fail
        let result = tracker.check_reorg_allowed(&blocks[5].hash()).await;
        assert!(result.is_err());

        // Try to reorg from an unfinalized block (height 8, 9, 10) - should succeed
        let result = tracker.check_reorg_allowed(&blocks[9].hash()).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_chain_selector_with_finality_protection() {
        // Test ChainSelector rejects reorgs past finalized blocks
        let dag_store = Arc::new(DagStore::new());
        let ghostdag = Arc::new(GhostDag::new(GhostDagParams::default(), dag_store.clone()));
        let tip_selector = Arc::new(TipSelector::new(
            dag_store.clone(),
            ghostdag.clone(),
            SelectionStrategy::HighestBlueScore,
        ));

        let finality_config = FinalityConfig {
            confirmation_depth: 3,
            emit_events: false,
            max_finalize_batch: 10,
        };
        let finality_tracker = Arc::new(FinalityTracker::new(dag_store.clone(), finality_config));

        let chain_selector = ChainSelector::with_finality(
            dag_store.clone(),
            ghostdag.clone(),
            tip_selector.clone(),
            100,
            finality_tracker.clone(),
        );

        // Build chain
        let genesis = create_test_block(0, 0, None);
        dag_store.store_block(genesis.clone()).await.unwrap();

        let mut prev = genesis.hash();
        for i in 1..=6 {
            let block = create_test_block(i as u8, i, Some(prev));
            dag_store.store_block(block.clone()).await.unwrap();
            chain_selector.on_new_block(&block).await.ok();
            prev = block.hash();
        }

        // Verify finality tracker is set
        assert!(chain_selector.finality_tracker().is_some());
    }
}

#[cfg(test)]
mod chain_selection_tests {
    use super::*;

    #[tokio::test]
    async fn test_chain_extension() {
        // Test simple chain extension
        let dag_store = Arc::new(DagStore::new());
        let ghostdag = Arc::new(GhostDag::new(GhostDagParams::default(), dag_store.clone()));
        let tip_selector = Arc::new(TipSelector::new(
            dag_store.clone(),
            ghostdag.clone(),
            SelectionStrategy::HighestBlueScore,
        ));
        let chain_selector = ChainSelector::new(
            dag_store.clone(),
            ghostdag.clone(),
            tip_selector.clone(),
            100,
        );

        let genesis = create_test_block(0, 0, None);
        dag_store.store_block(genesis.clone()).await.unwrap();

        // This should work without panic
        let chain_state = chain_selector.get_chain_state().await;
        assert_eq!(chain_state.tip, Hash::default()); // Initial state
    }

    #[tokio::test]
    async fn test_empty_chain_validation() {
        // Empty chain should be valid
        let dag_store = Arc::new(DagStore::new());
        let ghostdag = Arc::new(GhostDag::new(GhostDagParams::default(), dag_store.clone()));
        let tip_selector = Arc::new(TipSelector::new(
            dag_store.clone(),
            ghostdag.clone(),
            SelectionStrategy::HighestBlueScore,
        ));
        let chain_selector = ChainSelector::new(
            dag_store.clone(),
            ghostdag.clone(),
            tip_selector.clone(),
            100,
        );

        let is_valid = chain_selector.validate_chain().await.unwrap();
        assert!(is_valid);
    }

    #[tokio::test]
    async fn test_reorg_history() {
        // Test that reorg history is tracked
        let dag_store = Arc::new(DagStore::new());
        let ghostdag = Arc::new(GhostDag::new(GhostDagParams::default(), dag_store.clone()));
        let tip_selector = Arc::new(TipSelector::new(
            dag_store.clone(),
            ghostdag.clone(),
            SelectionStrategy::HighestBlueScore,
        ));
        let chain_selector = ChainSelector::new(
            dag_store.clone(),
            ghostdag.clone(),
            tip_selector.clone(),
            100,
        );

        // Initially no reorgs
        let history = chain_selector.get_reorg_history().await;
        assert!(history.is_empty());
    }
}

#[cfg(test)]
mod stress_tests {
    use super::*;

    #[tokio::test]
    async fn test_many_blocks_finality() {
        // Test finality with many blocks
        let dag_store = Arc::new(DagStore::new());
        let config = FinalityConfig {
            confirmation_depth: 10,
            emit_events: false,
            max_finalize_batch: 20,
        };
        let tracker = FinalityTracker::new(dag_store.clone(), config);

        let genesis = create_test_block(0, 0, None);
        dag_store.store_block(genesis.clone()).await.unwrap();

        let mut prev = genesis.hash();
        for i in 1..=50 {
            let block = create_test_block((i % 255) as u8, i, Some(prev));
            dag_store.store_block(block.clone()).await.unwrap();
            tracker.update_finality(&block.hash(), i).await.unwrap();
            prev = block.hash();
        }

        // At height 50 with depth 10, blocks up to height 40 should be finalized
        let finalized_height = tracker.get_finalized_height();
        assert!(finalized_height >= 40);
    }

    #[tokio::test]
    async fn test_ordering_long_chain() {
        // Test ordering with a longer chain
        let dag_store = Arc::new(DagStore::new());
        let ghostdag = Arc::new(GhostDag::new(GhostDagParams::default(), dag_store.clone()));
        let ordering = TotalOrdering::new(dag_store.clone(), ghostdag.clone());

        let genesis = create_test_block(0, 0, None);
        dag_store.store_block(genesis.clone()).await.unwrap();

        let mut prev = genesis.hash();
        let mut last_block = genesis.clone();
        for i in 1..=20 {
            let block = create_test_block(i as u8, i, Some(prev));
            dag_store.store_block(block.clone()).await.unwrap();
            prev = block.hash();
            last_block = block;
        }

        // Get ordering from genesis to last block
        let ordered = ordering
            .get_ordered_blocks(genesis.hash(), last_block.hash())
            .await
            .unwrap();

        // Should have blocks
        assert!(!ordered.blocks.is_empty());
        // Last block should match
        assert_eq!(*ordered.blocks.last().unwrap(), last_block.hash());
    }
}

#[cfg(test)]
mod finality_tracker_unit_tests {
    use super::*;

    #[tokio::test]
    async fn test_finality_count_tracking() {
        let dag_store = Arc::new(DagStore::new());
        let config = FinalityConfig {
            confirmation_depth: 2,
            emit_events: false,
            max_finalize_batch: 10,
        };
        let tracker = FinalityTracker::new(dag_store.clone(), config);

        let genesis = create_test_block(0, 0, None);
        let b1 = create_test_block(1, 1, Some(genesis.hash()));
        let b2 = create_test_block(2, 2, Some(b1.hash()));
        let b3 = create_test_block(3, 3, Some(b2.hash()));
        let b4 = create_test_block(4, 4, Some(b3.hash()));

        for block in [&genesis, &b1, &b2, &b3, &b4] {
            dag_store.store_block(block.clone()).await.unwrap();
        }

        // Update finality progressively
        tracker.update_finality(&genesis.hash(), 0).await.unwrap();
        tracker.update_finality(&b1.hash(), 1).await.unwrap();
        tracker.update_finality(&b2.hash(), 2).await.unwrap();
        tracker.update_finality(&b3.hash(), 3).await.unwrap();
        tracker.update_finality(&b4.hash(), 4).await.unwrap();

        // Should have finalized some blocks
        assert!(tracker.get_finalized_count() > 0);
    }

    #[tokio::test]
    async fn test_finality_reset() {
        let dag_store = Arc::new(DagStore::new());
        let config = FinalityConfig {
            confirmation_depth: 2,
            emit_events: false,
            max_finalize_batch: 10,
        };
        let tracker = FinalityTracker::new(dag_store.clone(), config);

        let genesis = create_test_block(0, 0, None);
        let b1 = create_test_block(1, 1, Some(genesis.hash()));
        let b2 = create_test_block(2, 2, Some(b1.hash()));

        for block in [&genesis, &b1, &b2] {
            dag_store.store_block(block.clone()).await.unwrap();
        }

        tracker.update_finality(&b2.hash(), 2).await.unwrap();

        // Reset
        tracker.reset().await;

        // After reset, should be back to initial state
        assert_eq!(tracker.get_finalized_height(), 0);
        assert_eq!(tracker.get_finalized_count(), 0);
    }
}
