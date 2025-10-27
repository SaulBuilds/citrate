use citrate_consensus::types::{Block, BlockHeader, Hash, PublicKey, Signature, VrfProof, GhostDagParams};
use citrate_consensus::dag_store::DagStore;
use citrate_storage::StorageManager;
use citrate_storage::pruning::PruningConfig;
use tempfile::TempDir;
use std::sync::Arc;

fn make_block(h: u64, parent: Hash) -> Block {
    Block {
        header: BlockHeader {
            version: 1,
            block_hash: Hash::new([h as u8; 32]),
            selected_parent_hash: parent,
            merge_parent_hashes: vec![],
            timestamp: 1_000_000 + h,
            height: h,
            blue_score: h,
            blue_work: h as u128,
            pruning_point: Hash::default(),
            proposer_pubkey: PublicKey::new([0; 32]),
            vrf_reveal: VrfProof { proof: vec![], output: Hash::default() },
        },
        state_root: Hash::default(),
        tx_root: Hash::default(),
        receipt_root: Hash::default(),
        artifact_root: Hash::default(),
        ghostdag_params: GhostDagParams::default(),
        transactions: vec![],
        signature: Signature::new([0; 64]),
    }
}

#[tokio::test]
async fn test_consensus_dag_and_storage_alignment() {
    // Storage
    let tmp = TempDir::new().unwrap();
    let storage = Arc::new(StorageManager::new(tmp.path(), PruningConfig::default()).unwrap());

    // Consensus DAG
    let dag = DagStore::new();

    // Seed chain of 4 blocks
    let genesis = make_block(0, Hash::default());
    let b1 = make_block(1, genesis.hash());
    let b2 = make_block(2, b1.hash());
    let b3 = make_block(3, b2.hash());

    for b in [&genesis, &b1, &b2, &b3] {
        storage.blocks.put_block(b).unwrap();
        dag.store_block(b.clone()).await.unwrap();
    }

    // Storage latest height
    let latest = storage.blocks.get_latest_height().unwrap();
    assert_eq!(latest, 3);

    // DAG tips should include only the last block
    let tips = dag.get_tips().await;
    assert_eq!(tips.len(), 1);
    let tip_hash = tips[0].hash;

    // Tip must match storage latest hash
    let latest_hash = storage.blocks.get_block_by_height(latest).unwrap().unwrap();
    assert_eq!(tip_hash, latest_hash);

    // Children relationships coherent
    let children_of_b2 = dag.get_children(&b2.hash()).await;
    assert_eq!(children_of_b2, vec![b3.hash()]);
}

#[tokio::test]
async fn test_branch_two_tips_and_storage_latest_height() {
    // Storage
    let tmp = TempDir::new().unwrap();
    let storage = Arc::new(StorageManager::new(tmp.path(), PruningConfig::default()).unwrap());

    // Consensus DAG
    let dag = DagStore::new();

    // Genesis and two competing children at same height
    let genesis = make_block(0, Hash::default());
    let b1a = make_block(1, genesis.hash());
    let b1b = make_block(1, genesis.hash());

    for b in [&genesis, &b1a, &b1b] {
        storage.blocks.put_block(b).unwrap();
        dag.store_block(b.clone()).await.unwrap();
    }

    // Latest height should be 1; block_by_height should yield the last stored block at that height
    let latest_h = storage.blocks.get_latest_height().unwrap();
    assert_eq!(latest_h, 1);
    let latest_hash = storage.blocks.get_block_by_height(latest_h).unwrap().unwrap();

    // DAG tips should include both branches at height 1
    let tips = dag.get_tips().await;
    assert_eq!(tips.len(), 2);
    let tip_hashes: Vec<Hash> = tips.into_iter().map(|t| t.hash).collect();
    assert!(tip_hashes.contains(&b1a.hash()));
    assert!(tip_hashes.contains(&b1b.hash()));

    // The storage latest hash must be one of the current tips
    assert!(tip_hashes.contains(&latest_hash));
}
