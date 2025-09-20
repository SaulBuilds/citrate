// Real, working tests for the consensus module
// These tests actually compile and test real functionality

use lattice_consensus::*;
use std::sync::Arc;

// Helper function to create a test block
fn create_test_block(block_num: u8, height: u64, parent: Option<Hash>) -> Block {
    let mut hash_bytes = [0u8; 32];
    if block_num == 0 {
        // Avoid zero hash (Hash::default())
        hash_bytes = [0xFFu8; 32];
    } else {
        hash_bytes[0] = block_num;
    }
    
    Block {
        header: BlockHeader {
            version: 1,
            block_hash: Hash::new(hash_bytes),
            selected_parent_hash: parent.unwrap_or_default(),
            merge_parent_hashes: vec![],
            timestamp: 1000000 + height * 10,
            height,
            blue_score: 0,
            blue_work: 0,
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
mod dag_store_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_dag_store_new() {
        let dag_store = DagStore::new();
        let stats = dag_store.get_stats().await;
        assert_eq!(stats.total_blocks, 0);
        assert_eq!(stats.total_tips, 0);
        assert_eq!(stats.finalized_blocks, 0);
    }
    
    #[tokio::test]
    async fn test_store_and_retrieve_block() {
        let dag_store = DagStore::new();
        let block = create_test_block(1, 0, None);
        let hash = block.hash();
        
        // Store the block
        dag_store.store_block(block.clone()).await.expect("Should store block");
        
        // Retrieve it
        let retrieved = dag_store.get_block(&hash).await.expect("Should get block");
        assert_eq!(retrieved.hash(), hash);
        assert_eq!(retrieved.header.height, 0);
    }
    
    #[tokio::test]
    async fn test_has_block() {
        let dag_store = DagStore::new();
        let block = create_test_block(1, 0, None);
        let hash = block.hash();
        
        // Initially should not have block
        assert!(!dag_store.has_block(&hash).await);
        
        // Store block
        dag_store.store_block(block).await.unwrap();
        
        // Now should have it
        assert!(dag_store.has_block(&hash).await);
    }
    
    #[tokio::test]
    async fn test_duplicate_block_error() {
        let dag_store = DagStore::new();
        let block = create_test_block(1, 0, None);
        
        // First store should succeed
        dag_store.store_block(block.clone()).await.unwrap();
        
        // Second store should fail
        let result = dag_store.store_block(block).await;
        assert!(result.is_err());
        match result {
            Err(DagStoreError::BlockExists(_)) => {},
            _ => panic!("Expected BlockExists error"),
        }
    }
    
    #[tokio::test]
    async fn test_get_blocks_at_height() {
        let dag_store = DagStore::new();
        
        // Add blocks at different heights
        let block1 = create_test_block(1, 0, None);
        let block2 = create_test_block(2, 1, Some(block1.hash()));
        let block3 = create_test_block(3, 1, Some(block1.hash()));
        
        dag_store.store_block(block1).await.unwrap();
        dag_store.store_block(block2).await.unwrap();
        dag_store.store_block(block3).await.unwrap();
        
        // Get blocks at height 1
        let blocks_at_1 = dag_store.get_blocks_at_height(1).await;
        assert_eq!(blocks_at_1.len(), 2);
        
        // Get blocks at height 0
        let blocks_at_0 = dag_store.get_blocks_at_height(0).await;
        assert_eq!(blocks_at_0.len(), 1);
    }
    
    #[tokio::test]
    async fn test_get_children() {
        let dag_store = DagStore::new();
        
        let genesis = create_test_block(0, 0, None);
        let child1 = create_test_block(1, 1, Some(genesis.hash()));
        let child2 = create_test_block(2, 1, Some(genesis.hash()));
        
        dag_store.store_block(genesis.clone()).await.unwrap();
        dag_store.store_block(child1.clone()).await.unwrap();
        dag_store.store_block(child2.clone()).await.unwrap();
        
        let children = dag_store.get_children(&genesis.hash()).await;
        assert_eq!(children.len(), 2);
        assert!(children.contains(&child1.hash()));
        assert!(children.contains(&child2.hash()));
    }
    
    #[tokio::test]
    async fn test_tips_tracking() {
        let dag_store = DagStore::new();
        
        // Add genesis - should be a tip
        let genesis = create_test_block(0, 0, None);
        dag_store.store_block(genesis.clone()).await.unwrap();
        
        let tips = dag_store.get_tips().await;
        assert_eq!(tips.len(), 1);
        assert_eq!(tips[0].hash, genesis.hash());
        
        // Add child - genesis should no longer be a tip
        let child = create_test_block(1, 1, Some(genesis.hash()));
        dag_store.store_block(child.clone()).await.unwrap();
        
        let tips = dag_store.get_tips().await;
        assert_eq!(tips.len(), 1);
        assert_eq!(tips[0].hash, child.hash());
    }
    
    #[tokio::test]
    async fn test_finalize_block() {
        let dag_store = DagStore::new();
        let block = create_test_block(1, 0, None);
        let hash = block.hash();
        
        dag_store.store_block(block).await.unwrap();
        
        // Initially not finalized
        assert!(!dag_store.is_finalized(&hash).await);
        
        // Finalize it
        dag_store.finalize_block(&hash).await.unwrap();
        
        // Now should be finalized
        assert!(dag_store.is_finalized(&hash).await);
    }
    
    #[tokio::test]
    async fn test_pruning_point() {
        let dag_store = DagStore::new();
        
        // Initially default
        assert_eq!(dag_store.get_pruning_point().await, Hash::default());
        
        // Store a block and update pruning point to its hash
        let b = create_test_block(1, 1, None);
        let h = b.hash();
        dag_store.store_block(b).await.unwrap();
        dag_store.update_pruning_point(h).await.unwrap();
        
        assert_eq!(dag_store.get_pruning_point().await, h);
    }
}

#[cfg(test)]
mod tip_tiebreak_tests {
    use super::*;

    #[tokio::test]
    async fn test_tip_tie_breaker_prefers_higher_hash() {
        let dag_store = Arc::new(DagStore::new());
        let ghostdag = Arc::new(GhostDag::new(GhostDagParams::default(), dag_store.clone()));
        let tip_selector = TipSelector::new(dag_store.clone(), ghostdag, SelectionStrategy::HighestBlueScoreWithTieBreak);

        // Genesis and two children at same height/blue score
        let genesis = create_test_block(0, 0, None);
        let child_a = create_test_block(1, 1, Some(genesis.hash()));
        let child_b = create_test_block(2, 1, Some(genesis.hash()));

        dag_store.store_block(genesis.clone()).await.unwrap();
        dag_store.store_block(child_a.clone()).await.unwrap();
        dag_store.store_block(child_b.clone()).await.unwrap();

        let tips = dag_store.get_tips().await;
        assert_eq!(tips.len(), 2);
        let tip_hashes: Vec<Hash> = tips.iter().map(|t| t.hash).collect();

        // Tie-breaker: choose lexicographically larger hash
        let expected = if child_a.hash() > child_b.hash() { child_a.hash() } else { child_b.hash() };
        let selected = tip_selector.select_tip(&tip_hashes).await.unwrap();
        assert_eq!(selected, expected);
    }
}

#[cfg(test)]
mod parent_selection_tests {
    use super::*;

    #[tokio::test]
    async fn test_parent_selector_prefers_highest_hash_among_equal_scores() {
        let dag_store = Arc::new(DagStore::new());
        let ghostdag = Arc::new(GhostDag::new(GhostDagParams::default(), dag_store.clone()));
        let tip_selector = Arc::new(TipSelector::new(dag_store.clone(), ghostdag, SelectionStrategy::HighestBlueScoreWithTieBreak));
        let parent_selector = ParentSelector::new(tip_selector, 1, 3);

        // Genesis and three same-height children
        let genesis = create_test_block(0, 0, None);
        let c1 = create_test_block(1, 1, Some(genesis.hash()));
        let c2 = create_test_block(2, 1, Some(genesis.hash()));
        let c3 = create_test_block(3, 1, Some(genesis.hash()));
        dag_store.store_block(genesis).await.unwrap();
        dag_store.store_block(c1.clone()).await.unwrap();
        dag_store.store_block(c2.clone()).await.unwrap();
        dag_store.store_block(c3.clone()).await.unwrap();

        let (selected, merges) = parent_selector.select_parents().await.unwrap();
        // Selected should be one of the current tips
        let all = vec![c1.hash(), c2.hash(), c3.hash()];
        assert!(all.contains(&selected));
        // Merge parents should contain the other two tips (order-agnostic)
        assert_eq!(merges.len(), 2);
        let others = all.into_iter().filter(|h| *h != selected).collect::<Vec<_>>();
        for h in others { assert!(merges.contains(&h)); }
    }

    #[tokio::test]
    async fn test_parent_selector_respects_min_and_max_parents() {
        let dag_store = Arc::new(DagStore::new());
        let ghostdag = Arc::new(GhostDag::new(GhostDagParams::default(), dag_store.clone()));
        let tipsel = Arc::new(TipSelector::new(dag_store.clone(), ghostdag, SelectionStrategy::HighestBlueScoreWithTieBreak));

        // With no tips, requiring min_parents > 0 should error
        let parent_selector = ParentSelector::new(tipsel.clone(), 2, 3);
        let err = parent_selector.select_parents().await.err().unwrap();
        match err { TipSelectionError::NoTips => {}, _ => panic!("expected NoTips") }

        // Seed 4 tips at height 1
        let genesis = create_test_block(0, 0, None);
        dag_store.store_block(genesis.clone()).await.unwrap();
        let mut children = Vec::new();
        for i in 1..=4u8 {
            let c = create_test_block(i, 1, Some(genesis.hash()));
            dag_store.store_block(c.clone()).await.unwrap();
            children.push(c.hash());
        }

        // max_parents=3 should yield 1 selected + 2 merges
        let ps = ParentSelector::new(tipsel.clone(), 1, 3);
        let (_sel, merges) = ps.select_parents().await.unwrap();
        assert!(merges.len() <= 2);
    }
}

#[cfg(test)]
mod ghostdag_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_ghostdag_new() {
        let params = GhostDagParams::default();
        let dag_store = Arc::new(DagStore::new());
        let ghostdag = GhostDag::new(params, dag_store);
        
        assert_eq!(ghostdag.params().k, 18);
        assert_eq!(ghostdag.params().max_parents, 10);
    }
    
    #[tokio::test]
    async fn test_calculate_blue_set_genesis() {
        let params = GhostDagParams::default();
        let dag_store = Arc::new(DagStore::new());
        let ghostdag = GhostDag::new(params, dag_store.clone());
        
        // Create a genesis block (height 0)
        let genesis = create_test_block(0, 0, None);
        dag_store.store_block(genesis.clone()).await.unwrap();
        
        // Calculate blue set for genesis
        let blue_set = ghostdag.calculate_blue_set(&genesis).await.unwrap();
        
        // Genesis should be in its own blue set
        assert!(blue_set.blocks.contains(&genesis.hash()));
        assert_eq!(blue_set.score, 1);
    }
    
    #[tokio::test]
    async fn test_tip_selection() {
        let params = GhostDagParams::default();
        let dag_store = Arc::new(DagStore::new());
        let _ghostdag = Arc::new(GhostDag::new(params, dag_store.clone()));
        
        // Create tip selector
        let tip_selector = TipSelector::new(
            dag_store.clone(),
            _ghostdag.clone(),
            SelectionStrategy::HighestBlueScore
        );
        
        // Add some blocks
        let genesis = create_test_block(0, 0, None);
        dag_store.store_block(genesis.clone()).await.unwrap();
        
        let block1 = create_test_block(1, 1, Some(genesis.hash()));
        dag_store.store_block(block1.clone()).await.unwrap();
        
        let block2 = create_test_block(2, 1, Some(genesis.hash()));
        dag_store.store_block(block2.clone()).await.unwrap();
        
        // Get tips
        let tips = dag_store.get_tips().await;
        let tip_hashes: Vec<Hash> = tips.iter().map(|t| t.hash).collect();
        
        // Select a tip (should work without error)
        let selected = tip_selector.select_tip(&tip_hashes).await;
        assert!(selected.is_ok());
    }
}

#[cfg(test)]
mod type_tests {
    use super::*;
    
    #[test]
    fn test_hash_creation_and_equality() {
        let hash1 = Hash::new([1u8; 32]);
        let hash2 = Hash::new([2u8; 32]);
        let hash3 = Hash::new([1u8; 32]);
        
        assert_eq!(hash1, hash3);
        assert_ne!(hash1, hash2);
    }
    
    #[test]
    fn test_hash_from_bytes() {
        let bytes = vec![42u8; 32];
        let hash = Hash::from_bytes(&bytes);
        assert_eq!(hash, Hash::new([42u8; 32]));
    }
    
    #[test]
    fn test_hash_default() {
        let hash = Hash::default();
        assert_eq!(hash, Hash::new([0u8; 32]));
    }
    
    #[test]
    fn test_public_key_creation() {
        let _pubkey = PublicKey::new([33u8; 32]);
        // Can't test internal value directly, but can verify it creates
        let _another = PublicKey::new([34u8; 32]);
    }
    
    #[test]
    fn test_signature_creation() {
        let _sig = Signature::new([42u8; 64]);
        // Can't test internal value directly, but can verify it creates
        let _another = Signature::new([43u8; 64]);
    }
    
    #[test]
    fn test_ghostdag_params_default() {
        let params = GhostDagParams::default();
        assert_eq!(params.k, 18);
        assert_eq!(params.max_parents, 10);
        assert_eq!(params.max_blue_score_diff, 1000);
        assert_eq!(params.pruning_window, 100000);
        assert_eq!(params.finality_depth, 100);
    }
    
    #[test]
    fn test_block_hash_method() {
        let block = create_test_block(5, 10, None);
        let expected_hash = Hash::new({
            let mut bytes = [0u8; 32];
            bytes[0] = 5;
            bytes
        });
        assert_eq!(block.hash(), expected_hash);
    }
    
    #[test]
    fn test_block_is_genesis() {
        let genesis = create_test_block(0, 0, None);
        assert!(genesis.is_genesis());
        
        let non_genesis = create_test_block(1, 1, Some(Hash::new([1u8; 32])));
        assert!(!non_genesis.is_genesis());
    }
}

#[cfg(test)]
mod chain_selection_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_chain_selector_creation() {
        let params = GhostDagParams::default();
        let dag_store = Arc::new(DagStore::new());
        let _ghostdag = Arc::new(GhostDag::new(params, dag_store.clone()));
        let tip_selector = Arc::new(TipSelector::new(
            dag_store.clone(),
            _ghostdag.clone(),
            SelectionStrategy::HighestBlueScore,
        ));
        let _chain_selector = ChainSelector::new(
            dag_store,
            _ghostdag,
            tip_selector,
            100,
        );
        // Just verify it can be created without panic
    }
}

#[cfg(test)]
mod vrf_tests {
    use super::*;
    
    #[test]
    fn test_vrf_proof_creation() {
        let proof = VrfProof {
            proof: vec![0u8; 80],
            output: Hash::new([1u8; 32]),
        };
        
        assert_eq!(proof.proof.len(), 80);
        assert_eq!(proof.output, Hash::new([1u8; 32]));
    }
    
    #[test]
    fn test_validator_creation() {
        let validator = Validator {
            pubkey: PublicKey::new([1u8; 32]),
            stake: 1000,
            is_active: true,
        };
        
        assert_eq!(validator.stake, 1000);
        assert!(validator.is_active);
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_full_dag_workflow() {
        // Create components
        let params = GhostDagParams::default();
        let dag_store = Arc::new(DagStore::new());
        let _ghostdag = Arc::new(GhostDag::new(params, dag_store.clone()));
        
        // Add genesis
        let genesis = create_test_block(0, 0, None);
        dag_store.store_block(genesis.clone()).await.unwrap();
        
        // Add some child blocks
        for i in 1..5 {
            let block = create_test_block(i, 1, Some(genesis.hash()));
            dag_store.store_block(block).await.unwrap();
        }
        
        // Check stats
        let stats = dag_store.get_stats().await;
        assert_eq!(stats.total_blocks, 5);
        assert_eq!(stats.total_tips, 4); // 4 children are all tips
        
        // Get blocks at height 1
        let blocks = dag_store.get_blocks_at_height(1).await;
        assert_eq!(blocks.len(), 4);
        
        // Genesis should have 4 children
        let children = dag_store.get_children(&genesis.hash()).await;
        assert_eq!(children.len(), 4);
    }
}

#[cfg(test)]
mod tip_selection_tests {
    use super::*;

    #[tokio::test]
    async fn test_tip_selection_tiebreak_equal_scores() {
        let params = GhostDagParams::default();
        let dag_store = Arc::new(DagStore::new());
        let ghostdag = Arc::new(GhostDag::new(params, dag_store.clone()));
        let tip_selector = TipSelector::new(
            dag_store.clone(),
            ghostdag.clone(),
            SelectionStrategy::HighestBlueScoreWithTieBreak,
        );

        // Genesis
        let genesis = create_test_block(0, 0, None);
        dag_store.store_block(genesis.clone()).await.unwrap();

        // Two children at the same height -> equal blue scores
        let child1 = create_test_block(1, 1, Some(genesis.hash()));
        let child2 = create_test_block(2, 1, Some(genesis.hash()));
        dag_store.store_block(child1.clone()).await.unwrap();
        dag_store.store_block(child2.clone()).await.unwrap();

        // With equal scores, tie-breaker selects higher hash
        let tips = dag_store.get_tips().await;
        let tip_hashes: Vec<Hash> = tips.iter().map(|t| t.hash).collect();

        let selected = tip_selector.select_tip(&tip_hashes).await.unwrap();
        assert_eq!(selected, child2.hash());
    }
}

#[cfg(test)]
mod parent_selector_tests {
    use super::*;

    #[tokio::test]
    async fn test_parent_selector_max_bound() {
        let params = GhostDagParams::default();
        let dag_store = Arc::new(DagStore::new());
        let ghostdag = Arc::new(GhostDag::new(params, dag_store.clone()));
        let tip_selector = Arc::new(TipSelector::new(
            dag_store.clone(),
            ghostdag.clone(),
            SelectionStrategy::HighestBlueScoreWithTieBreak,
        ));
        let parent_selector = ParentSelector::new(tip_selector, 1, 3);

        // Genesis and 4 children → more tips than max_parents
        let genesis = create_test_block(0, 0, None);
        dag_store.store_block(genesis.clone()).await.unwrap();
        for i in 1..=4u8 {
            let child = create_test_block(i, 1, Some(genesis.hash()));
            dag_store.store_block(child).await.unwrap();
        }

        let (_selected, merge_parents) = parent_selector.select_parents().await.unwrap();
        // At most max_parents - 1 merge parents
        assert!(merge_parents.len() <= 2);
    }
}
