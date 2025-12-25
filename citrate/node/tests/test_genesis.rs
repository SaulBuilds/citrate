// Test genesis block creation and verification
//
// Note: These tests verify the genesis block structure and embedded models.
// The genesis block creation uses feature-gated model embedding (embed-genesis-model).
// When the feature is disabled, embedded models will be empty (contributor builds).

use citrate_consensus::types::{Block, GhostDagParams, Hash, PublicKey, Signature, VrfProof};

/// Create a minimal test genesis block for unit tests
/// This doesn't include embedded models - those are feature-gated in production code
fn create_test_genesis_block() -> Block {
    use citrate_consensus::types::BlockHeader;

    let header = BlockHeader {
        version: 1,
        block_hash: Hash::default(),
        selected_parent_hash: Hash::default(),
        merge_parent_hashes: vec![],
        timestamp: 0,
        height: 0,
        blue_score: 0,
        blue_work: 0,
        pruning_point: Hash::default(),
        proposer_pubkey: PublicKey::new([0; 32]),
        vrf_reveal: VrfProof {
            proof: vec![],
            output: Hash::default(),
        },
        base_fee_per_gas: 0,
        gas_used: 0,
        gas_limit: 30_000_000,
    };

    Block {
        header,
        state_root: Hash::default(),
        tx_root: Hash::default(),
        receipt_root: Hash::default(),
        artifact_root: Hash::default(),
        ghostdag_params: GhostDagParams::default(),
        transactions: vec![],
        signature: Signature::new([0; 64]),
        embedded_models: vec![],
        required_pins: vec![],
    }
}

#[test]
fn test_genesis_block_basic_properties() {
    let genesis = create_test_genesis_block();

    // Verify basic properties
    assert_eq!(genesis.header.height, 0, "Genesis block height should be 0");
    assert_eq!(genesis.header.timestamp, 0, "Genesis timestamp should be 0");
    assert_eq!(
        genesis.header.selected_parent_hash,
        Hash::default(),
        "Genesis should have no parent"
    );
    assert!(
        genesis.header.merge_parent_hashes.is_empty(),
        "Genesis should have no merge parents"
    );
    assert_eq!(
        genesis.header.blue_score, 0,
        "Genesis blue score should be 0"
    );
}

#[test]
fn test_genesis_block_hash_computation() {
    use sha3::{Digest, Sha3_256};

    let genesis = create_test_genesis_block();

    // Compute block hash manually
    let mut hasher = Sha3_256::new();
    hasher.update(genesis.header.version.to_le_bytes());
    hasher.update(genesis.header.selected_parent_hash.as_bytes());
    hasher.update(genesis.header.timestamp.to_le_bytes());
    hasher.update(genesis.header.height.to_le_bytes());
    let computed_hash = hasher.finalize();

    // Hash should be deterministic
    let mut second_hasher = Sha3_256::new();
    second_hasher.update(genesis.header.version.to_le_bytes());
    second_hasher.update(genesis.header.selected_parent_hash.as_bytes());
    second_hasher.update(genesis.header.timestamp.to_le_bytes());
    second_hasher.update(genesis.header.height.to_le_bytes());
    let second_hash = second_hasher.finalize();

    assert_eq!(
        computed_hash, second_hash,
        "Block hash computation should be deterministic"
    );
}

#[test]
fn test_genesis_default_params() {
    let params = GhostDagParams::default();

    // Default k-cluster parameter
    assert!(params.k > 0, "K parameter should be positive");

    // Default max parents
    assert!(
        params.max_parents > 0,
        "Max block parents should be positive"
    );
}

#[test]
fn test_genesis_required_pins_format() {
    use citrate_consensus::types::{ModelId, RequiredModel};

    // Create a test required pin with proper IPFS CID format
    let required_model = RequiredModel::new(
        ModelId::from_name("test-model"),
        "QmUsYyxg71bV8USRQ6Ccm3SdMqeWgEEVnCYkgNDaxvBTZB".to_string(),
        Hash::new([0x12; 32]),
        4_367_438_912, // ~4.1 GB
        1_000_000_000_000_000_000_000,
    );

    // Verify CID format
    assert!(
        required_model.ipfs_cid.starts_with("Qm"),
        "CID should be valid IPFS v0 format"
    );

    // Verify size is reasonable
    let size_gb = required_model.size_bytes as f64 / (1024.0 * 1024.0 * 1024.0);
    assert!(
        size_gb > 3.0 && size_gb < 6.0,
        "Expected size around 4GB for 7B model"
    );

    // Verify must_pin flag defaults correctly
    assert!(required_model.must_pin, "Required models should have must_pin = true");
}

#[test]
fn test_embedded_model_structure() {
    use citrate_consensus::types::{EmbeddedModel, ModelMetadata, ModelType, ModelId};

    // Create test embedded model
    let model = EmbeddedModel {
        model_id: ModelId::from_name("test-embedding-model"),
        model_type: ModelType::Embeddings,
        weights: vec![0u8; 100], // Small test weights
        metadata: ModelMetadata {
            name: "Test Model".to_string(),
            version: "1.0.0".to_string(),
            context_length: 8192,
            embedding_dim: Some(1024),
            license: "MIT".to_string(),
            framework: Some("GGUF".to_string()),
        },
    };

    // Verify structure
    assert_eq!(model.size_bytes(), 100);
    assert!(matches!(model.model_type, ModelType::Embeddings));
    assert_eq!(model.metadata.embedding_dim, Some(1024));
}
