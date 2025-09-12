use lattice_consensus::types::{Transaction, Hash, TransactionType, PublicKey, Signature};
use lattice_execution::{ModelId, ModelState, ModelMetadata, Address, AccessPolicy, UsageStats};
use lattice_storage::state_manager::StateManager;
use lattice_network::{PeerManager, AINetworkHandler, BlockPropagation, TransactionGossip};
use lattice_api::methods::ai::AIMethods;
use std::sync::Arc;
use tokio;
use primitive_types::U256;

#[tokio::test]
async fn test_full_ai_model_lifecycle() {
    // Initialize components
    let db = Arc::new(lattice_storage::db::RocksDB::open_temp().unwrap());
    let state_manager = Arc::new(StateManager::new(db.clone()));
    let peer_manager = Arc::new(PeerManager::new(Default::default()));
    let ai_handler = AINetworkHandler::new(state_manager.clone(), peer_manager.clone());
    
    // Test 1: Deploy a model
    let model_id = ModelId(Hash::new([1; 32]));
    let model_metadata = ModelMetadata {
        name: "GPT-Test".to_string(),
        version: "1.0".to_string(),
        description: "Test model".to_string(),
        framework: "PyTorch".to_string(),
        input_shape: vec![1, 512],
        output_shape: vec![1, 768],
        size_bytes: 1_000_000,
        created_at: 12345,
    };
    
    let model_state = ModelState {
        owner: Address([1; 20]),
        model_hash: Hash::new([2; 32]),
        version: 1,
        metadata: model_metadata.clone(),
        access_policy: AccessPolicy::Public,
        usage_stats: UsageStats::default(),
    };
    
    // Register model
    state_manager.register_model(model_id, model_state.clone(), "QmTest123".to_string()).unwrap();
    
    // Test 2: Retrieve model
    let retrieved = state_manager.get_model(&model_id);
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().owner, model_state.owner);
    
    // Test 3: Broadcast model announcement
    ai_handler.broadcast_model(
        Hash::new([1; 32]),
        Hash::new([2; 32]),
        vec![1; 20],
        lattice_network::ModelMetadata {
            name: model_metadata.name,
            version: model_metadata.version,
            description: model_metadata.description,
            framework: model_metadata.framework,
            input_shape: model_metadata.input_shape,
            output_shape: model_metadata.output_shape,
            size_bytes: model_metadata.size_bytes,
            created_at: model_metadata.created_at,
        },
        "QmTest123".to_string(),
    ).await.unwrap();
    
    // Test 4: Request inference
    let request_id = ai_handler.request_inference(
        Hash::new([1; 32]),
        Hash::new([3; 32]),
        vec![1; 20],
        1000000,
    ).await.unwrap();
    
    assert_ne!(request_id, Hash::new([0; 32]));
    
    println!("✅ AI Model Lifecycle Test Passed");
}

#[tokio::test]
async fn test_ghostdag_consensus() {
    use lattice_consensus::ghostdag::GhostDag;
    use lattice_consensus::types::{Block, BlockHeader, GhostDagParams, VrfProof};
    
    let ghostdag = GhostDag::new(16, 100_000);
    
    // Create test blocks
    let genesis = Block {
        header: BlockHeader {
            version: 1,
            block_hash: Hash::new([0; 32]),
            selected_parent_hash: Hash::new([0; 32]),
            merge_parent_hashes: vec![],
            timestamp: 0,
            height: 0,
            blue_score: 0,
            blue_work: 0,
            pruning_point: Hash::new([0; 32]),
            proposer_pubkey: PublicKey::new([0; 32]),
            vrf_reveal: VrfProof {
                proof: vec![],
                output: Hash::new([0; 32]),
            },
        },
        state_root: Hash::new([0; 32]),
        tx_root: Hash::new([0; 32]),
        receipt_root: Hash::new([0; 32]),
        artifact_root: Hash::new([0; 32]),
        ghostdag_params: GhostDagParams::default(),
        transactions: vec![],
        signature: Signature::new([0; 64]),
    };
    
    // Calculate blue set
    let blue_set = ghostdag.calculate_blue_set(&genesis);
    assert!(!blue_set.blocks.is_empty());
    
    // Calculate blue score
    let blue_score = ghostdag.calculate_blue_score(&genesis);
    assert_eq!(blue_score, 0); // Genesis has blue score 0
    
    println!("✅ GhostDAG Consensus Test Passed");
}

#[tokio::test]
async fn test_ai_transaction_types() {
    use lattice_sequencer::mempool::Mempool;
    
    let mempool = Arc::new(tokio::sync::RwLock::new(Mempool::new(Default::default())));
    
    // Create different transaction types
    let tx_types = vec![
        TransactionType::Standard,
        TransactionType::ModelDeploy,
        TransactionType::ModelUpdate,
        TransactionType::InferenceRequest,
        TransactionType::TrainingJob,
        TransactionType::LoraAdapter,
    ];
    
    for (i, tx_type) in tx_types.iter().enumerate() {
        let tx = Transaction {
            hash: Hash::new([i as u8; 32]),
            nonce: i as u64,
            from: PublicKey::new([i as u8; 32]),
            to: Some(PublicKey::new([(i + 1) as u8; 32])),
            value: 1000,
            gas_limit: 21000,
            gas_price: 100,
            data: vec![],
            signature: Signature::new([0; 64]),
            tx_type: Some(tx_type.clone()),
        };
        
        mempool.write().await.add_transaction(tx.clone()).unwrap();
        
        // Verify transaction was added
        assert!(mempool.read().await.has_transaction(&tx.hash));
    }
    
    // Check that we have all transaction types
    let all_txs = mempool.read().await.get_transactions(10);
    assert_eq!(all_txs.len(), 6);
    
    println!("✅ AI Transaction Types Test Passed");
}

#[tokio::test]
async fn test_ai_state_management() {
    let db = Arc::new(lattice_storage::db::RocksDB::open_temp().unwrap());
    let state_manager = Arc::new(StateManager::new(db.clone()));
    
    // Test AI state operations
    let stats = state_manager.get_ai_stats();
    assert_eq!(stats.total_models, 0);
    
    // Add a model
    let model_id = ModelId(Hash::new([10; 32]));
    let model_state = ModelState {
        owner: Address([5; 20]),
        model_hash: Hash::new([11; 32]),
        version: 1,
        metadata: lattice_execution::ModelMetadata {
            name: "TestModel".to_string(),
            version: "1.0".to_string(),
            description: "Test".to_string(),
            framework: "TensorFlow".to_string(),
            input_shape: vec![1, 224, 224, 3],
            output_shape: vec![1, 1000],
            size_bytes: 5_000_000,
            created_at: 54321,
        },
        access_policy: AccessPolicy::Public,
        usage_stats: UsageStats::default(),
    };
    
    state_manager.register_model(model_id, model_state, "QmModel".to_string()).unwrap();
    
    // Check stats updated
    let stats = state_manager.get_ai_stats();
    assert_eq!(stats.total_models, 1);
    
    // Calculate state root
    let state_root = state_manager.calculate_state_root().await.unwrap();
    assert_ne!(state_root, Hash::new([0; 32]));
    
    println!("✅ AI State Management Test Passed");
}

#[tokio::test]
async fn test_network_propagation() {
    let peer_manager = Arc::new(PeerManager::new(Default::default()));
    let block_prop = BlockPropagation::new(peer_manager.clone());
    let tx_gossip = TransactionGossip::new(peer_manager.clone(), Default::default());
    
    // Test block propagation
    let block = Block {
        header: BlockHeader {
            version: 1,
            block_hash: Hash::new([100; 32]),
            selected_parent_hash: Hash::new([99; 32]),
            merge_parent_hashes: vec![],
            timestamp: 1000,
            height: 10,
            blue_score: 5,
            blue_work: 100,
            pruning_point: Hash::new([0; 32]),
            proposer_pubkey: PublicKey::new([1; 32]),
            vrf_reveal: VrfProof {
                proof: vec![],
                output: Hash::new([0; 32]),
            },
        },
        state_root: Hash::new([1; 32]),
        tx_root: Hash::new([2; 32]),
        receipt_root: Hash::new([3; 32]),
        artifact_root: Hash::new([4; 32]),
        ghostdag_params: GhostDagParams::default(),
        transactions: vec![],
        signature: Signature::new([0; 64]),
    };
    
    // Broadcast block
    block_prop.broadcast_block(block.clone()).await.unwrap();
    
    // Test transaction gossip
    let tx = Transaction {
        hash: Hash::new([200; 32]),
        nonce: 1,
        from: PublicKey::new([10; 32]),
        to: Some(PublicKey::new([11; 32])),
        value: 5000,
        gas_limit: 50000,
        gas_price: 200,
        data: vec![1, 2, 3],
        signature: Signature::new([0; 64]),
        tx_type: Some(TransactionType::ModelDeploy),
    };
    
    // Broadcast transaction
    tx_gossip.broadcast_transaction(tx).await.unwrap();
    
    println!("✅ Network Propagation Test Passed");
}

#[tokio::test]
async fn test_api_endpoints() {
    // This would normally test against a running server
    // For unit testing, we test the method implementations directly
    
    let db = Arc::new(lattice_storage::db::RocksDB::open_temp().unwrap());
    let state_manager = Arc::new(StateManager::new(db.clone()));
    let sequencer = Arc::new(tokio::sync::RwLock::new(
        lattice_sequencer::Sequencer::new(Default::default(), state_manager.clone())
    ));
    
    let ai_methods = AIMethods::new(state_manager.clone(), sequencer.clone());
    
    // Test model deployment
    let model_id = ModelId(Hash::new([50; 32]));
    let result = ai_methods.deploy_model(
        Address([1; 20]),
        Hash::new([51; 32]),
        "TestModel".to_string(),
        "1.0".to_string(),
        "PyTorch".to_string(),
        vec![1, 512],
        vec![1, 768],
        "QmWeight123".to_string(),
    ).await;
    
    assert!(result.is_ok());
    
    // Test model retrieval
    let model = ai_methods.get_model(model_id).await;
    assert!(model.is_ok());
    
    // Test model listing
    let models = ai_methods.list_models(None, Some(10)).await;
    assert!(models.is_ok());
    
    println!("✅ API Endpoints Test Passed");
}

#[tokio::test]
async fn test_end_to_end_workflow() {
    println!("\n🚀 Running End-to-End Integration Test...\n");
    
    // Initialize full stack
    let db = Arc::new(lattice_storage::db::RocksDB::open_temp().unwrap());
    let state_manager = Arc::new(StateManager::new(db.clone()));
    let peer_manager = Arc::new(PeerManager::new(Default::default()));
    let mempool = Arc::new(tokio::sync::RwLock::new(lattice_sequencer::mempool::Mempool::new(Default::default())));
    
    // Step 1: Deploy a model
    println!("1️⃣ Deploying AI Model...");
    let model_tx = Transaction {
        hash: Hash::new([255; 32]),
        nonce: 0,
        from: PublicKey::new([1; 32]),
        to: None,
        value: 0,
        gas_limit: 1_000_000,
        gas_price: 100,
        data: vec![1, 2, 3], // Model deployment data
        signature: Signature::new([0; 64]),
        tx_type: Some(TransactionType::ModelDeploy),
    };
    
    mempool.write().await.add_transaction(model_tx).unwrap();
    
    // Step 2: Request inference
    println!("2️⃣ Requesting Inference...");
    let inference_tx = Transaction {
        hash: Hash::new([254; 32]),
        nonce: 1,
        from: PublicKey::new([1; 32]),
        to: Some(PublicKey::new([2; 32])),
        value: 1000,
        gas_limit: 500_000,
        gas_price: 150,
        data: vec![4, 5, 6], // Inference input data
        signature: Signature::new([0; 64]),
        tx_type: Some(TransactionType::InferenceRequest),
    };
    
    mempool.write().await.add_transaction(inference_tx).unwrap();
    
    // Step 3: Create training job
    println!("3️⃣ Creating Training Job...");
    let training_tx = Transaction {
        hash: Hash::new([253; 32]),
        nonce: 2,
        from: PublicKey::new([1; 32]),
        to: None,
        value: 10000,
        gas_limit: 2_000_000,
        gas_price: 200,
        data: vec![7, 8, 9], // Training parameters
        signature: Signature::new([0; 64]),
        tx_type: Some(TransactionType::TrainingJob),
    };
    
    mempool.write().await.add_transaction(training_tx).unwrap();
    
    // Step 4: Verify all transactions in mempool
    println!("4️⃣ Verifying Mempool...");
    let pending_txs = mempool.read().await.get_transactions(10);
    assert_eq!(pending_txs.len(), 3);
    
    // Step 5: Calculate state root
    println!("5️⃣ Calculating State Root...");
    let state_root = state_manager.calculate_state_root().await.unwrap();
    assert_ne!(state_root, Hash::new([0; 32]));
    
    println!("\n✅ End-to-End Workflow Test Passed!\n");
}

fn main() {
    println!("Run tests with: cargo test --test integration_test");
}