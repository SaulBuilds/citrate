use lattice_consensus::types::{Block, BlockHeader, Hash, PublicKey, Signature, Transaction, VrfProof, GhostDagParams};
use lattice_execution::types::Address;
use lattice_execution::executor::Executor;
use lattice_sequencer::mempool::{Mempool, MempoolConfig, TxClass};
use lattice_sequencer::block_builder::{BlockBuilder, BlockBuilderConfig};
use std::sync::Arc;

fn make_block_header(parent: Hash, height: u64) -> BlockHeader {
    BlockHeader {
        version: 1,
        block_hash: Hash::new([0xEE; 32]),
        selected_parent_hash: parent,
        merge_parent_hashes: vec![],
        timestamp: 1_000_000 + height,
        height,
        blue_score: height,
        blue_work: height as u128,
        pruning_point: Hash::default(),
        proposer_pubkey: PublicKey::new([0; 32]),
        vrf_reveal: VrfProof { proof: vec![], output: Hash::default() },
    }
}

fn make_tx(nonce: u64, gas_price: u64, from: [u8; 32], to: [u8; 32], data: Vec<u8>) -> Transaction {
    // Ensure unique hash per tx even when nonce collides across different senders
    let mut h = [nonce as u8; 32];
    h[31] = from[0];
    Transaction {
        hash: Hash::new(h),
        nonce,
        from: PublicKey::new(from),
        to: Some(PublicKey::new(to)),
        value: 1000,
        gas_limit: 21000,
        gas_price,
        data,
        signature: Signature::new([1; 64]),
        tx_type: None,
    }
}

#[tokio::test]
async fn test_build_block_and_execute_transactions() {
    // Mempool and builder
    let mempool = Arc::new(Mempool::new(MempoolConfig { require_valid_signature: false, ..Default::default() }));
    let cfg = BlockBuilderConfig { enable_bundling: true, ..Default::default() };
    let proposer = PublicKey::new([7; 32]);
    let builder = BlockBuilder::new(cfg, mempool.clone(), proposer);

    // Three txs, distinct senders; one classified as inference by builder
    let tx1 = make_tx(0, 2_000_000_000, [1; 32], [2; 32], vec![]);
    let mut tx2 = make_tx(0, 2_000_000_000, [3; 32], [4; 32], b"inference".to_vec());
    let tx3 = make_tx(0, 2_000_000_000, [5; 32], [6; 32], vec![]);

    mempool.add_transaction(tx1.clone(), TxClass::Standard).await.unwrap();
    mempool.add_transaction(tx2.clone(), TxClass::Inference).await.unwrap();
    mempool.add_transaction(tx3.clone(), TxClass::Standard).await.unwrap();

    let parent = Hash::new([0xAA; 32]);
    let vrf = VrfProof { proof: vec![0; 32], output: Hash::new([0; 32]) };
    let block = builder.build_block(parent, vec![], 0, 1, vrf).await.unwrap();

    // Validate block constraints
    builder.validate_block(&block).unwrap();

    // Verify tx_root matches keccak of tx hashes in order
    use sha3::{Digest, Keccak256};
    let mut hasher = Keccak256::new();
    for tx in &block.transactions {
        hasher.update(tx.hash.as_bytes());
    }
    let expected_root = Hash::from_bytes(&hasher.finalize());
    assert_eq!(block.tx_root, expected_root);

    // Execute transactions using a fresh executor (clone txs for execution with zero price/value)
    let state_db = Arc::new(lattice_execution::StateDB::new());
    let exec = Executor::new(state_db.clone());

    for tx in &block.transactions {
        let addr = Address::from_public_key(&tx.from);
        state_db.accounts.create_account_if_not_exists(addr);
        state_db.accounts.set_nonce(addr, tx.nonce);
    }

    for tx in &block.transactions {
        let mut etx = tx.clone();
        etx.gas_price = 0;
        etx.value = 0;
        let rc = exec.execute_transaction(&block, &etx).await.expect("exec ok");
        assert_eq!(rc.block_number, block.header.height);
        assert!(rc.status);
    }
}
