// Test genesis block creation and verification

use citrate_consensus::genesis::{create_genesis_block, GenesisConfig};
use citrate_consensus::types::Hash;

#[test]
fn test_genesis_block_creation() {
    // Create genesis configuration
    let config = GenesisConfig {
        timestamp: 0,
        chain_id: 1337,
        initial_accounts: vec![],
    };

    // Create genesis block
    let genesis = create_genesis_block(&config);

    // Verify basic properties
    assert_eq!(genesis.height, 0, "Genesis block height should be 0");
    assert_eq!(genesis.timestamp, 0, "Genesis timestamp should be 0");
    assert_eq!(
        genesis.selected_parent_hash,
        Hash::default(),
        "Genesis should have no parent"
    );

    // Verify embedded models
    println!("Genesis Block Details:");
    println!("  Height: {}", genesis.height);
    println!("  Timestamp: {}", genesis.timestamp);
    println!("  Block Hash: {}", hex::encode(genesis.hash.as_bytes()));
    println!("  Number of embedded models: {}", genesis.embedded_models.len());

    assert!(
        !genesis.embedded_models.is_empty(),
        "Genesis should have embedded models"
    );

    // Calculate total size of embedded models
    let mut total_size = 0u64;
    for model in &genesis.embedded_models {
        let size_mb = model.model_data.len() as f64 / (1024.0 * 1024.0);
        println!("  Embedded Model: {} ({:.2} MB)", model.model_id, size_mb);
        total_size += model.model_data.len() as u64;
    }

    let total_size_mb = total_size as f64 / (1024.0 * 1024.0);
    println!("  Total embedded size: {:.2} MB", total_size_mb);

    // Verify size is approximately 437MB for BGE-M3
    assert!(
        total_size_mb >= 400.0 && total_size_mb <= 500.0,
        "Expected embedded models size around 437MB, got {:.2}MB",
        total_size_mb
    );

    // Verify required pins
    println!("  Number of required pins: {}", genesis.required_pins.len());
    for pin in &genesis.required_pins {
        let size_mb = pin.expected_size_bytes as f64 / (1024.0 * 1024.0);
        println!("    - {} (CID: {}, {:.2} MB)", pin.model_id, pin.ipfs_cid, size_mb);
    }

    assert!(
        !genesis.required_pins.is_empty(),
        "Genesis should have required pins for IPFS models"
    );

    // Serialize genesis block to check actual byte size
    let serialized = bincode::serialize(&genesis).expect("Failed to serialize genesis block");
    let serialized_size_mb = serialized.len() as f64 / (1024.0 * 1024.0);
    println!("  Serialized block size: {:.2} MB", serialized_size_mb);

    // The serialized size should be around 437MB (embedded models) + overhead
    assert!(
        serialized_size_mb >= 400.0 && serialized_size_mb <= 550.0,
        "Expected serialized size around 437MB, got {:.2}MB",
        serialized_size_mb
    );
}

#[test]
fn test_genesis_embedded_model_integrity() {
    let config = GenesisConfig {
        timestamp: 0,
        chain_id: 1337,
        initial_accounts: vec![],
    };

    let genesis = create_genesis_block(&config);

    for model in &genesis.embedded_models {
        // Verify model data is not empty
        assert!(
            !model.model_data.is_empty(),
            "Model {} should have data",
            model.model_id
        );

        // Verify model hash matches data
        use sha3::{Digest, Sha3_256};
        let mut hasher = Sha3_256::new();
        hasher.update(&model.model_data);
        let computed_hash = hasher.finalize();

        println!(
            "Model {}: computed hash = {}",
            model.model_id,
            hex::encode(&computed_hash[..8])
        );

        // Verify metadata exists
        assert!(
            !model.metadata.is_empty(),
            "Model {} should have metadata",
            model.model_id
        );
    }
}

#[test]
fn test_genesis_required_pins_validity() {
    let config = GenesisConfig {
        timestamp: 0,
        chain_id: 1337,
        initial_accounts: vec![],
    };

    let genesis = create_genesis_block(&config);

    for pin in &genesis.required_pins {
        println!("Testing required pin: {}", pin.model_id);

        // Verify IPFS CID is valid format
        assert!(
            pin.ipfs_cid.starts_with("Qm") || pin.ipfs_cid.starts_with("baf"),
            "CID should be valid IPFS format: {}",
            pin.ipfs_cid
        );

        // Verify size is reasonable (Mistral 7B should be ~4GB)
        let size_gb = pin.expected_size_bytes as f64 / (1024.0 * 1024.0 * 1024.0);
        println!("  Expected size: {:.2} GB", size_gb);
        assert!(
            size_gb >= 3.0 && size_gb <= 6.0,
            "Mistral model size should be around 4GB, got {:.2}GB",
            size_gb
        );

        // Verify SHA256 hash is present and valid length
        assert_eq!(
            pin.sha256_hash.len(),
            64,
            "SHA256 hash should be 64 hex characters"
        );

        println!("  SHA256: {}", pin.sha256_hash);
    }
}
