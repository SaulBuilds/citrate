use citrate_consensus::dag_store::DagStore;
use citrate_consensus::types::{
    Block, BlockHeader, EmbeddedModel, GhostDagParams, Hash, ModelId as ConsensusModelId,
    ModelMetadata as ConsensusModelMetadata, ModelType, PublicKey, RequiredModel, Signature,
    VrfProof,
};
use citrate_economics::genesis::GenesisConfig as EconomicsGenesisConfig;
use citrate_execution::executor::Executor;
use citrate_execution::types::{
    AccessPolicy, Address, ModelId, ModelMetadata, ModelState, UsageStats,
};
use citrate_storage::StorageManager;
use primitive_types::U256;
use sha3::{Digest, Sha3_256};
use std::sync::Arc;

/// Calculate block hash using SHA3-256
fn calculate_block_hash(block: &Block) -> Hash {
    let mut hasher = Sha3_256::new();

    // Hash header fields
    hasher.update(block.header.version.to_le_bytes());
    hasher.update(block.header.selected_parent_hash.as_bytes());
    for parent in &block.header.merge_parent_hashes {
        hasher.update(parent.as_bytes());
    }
    hasher.update(block.header.timestamp.to_le_bytes());
    hasher.update(block.header.height.to_le_bytes());
    hasher.update(block.header.blue_score.to_le_bytes());
    hasher.update(block.header.blue_work.to_le_bytes());
    hasher.update(block.header.pruning_point.as_bytes());

    // Hash roots
    hasher.update(block.state_root.as_bytes());
    hasher.update(block.tx_root.as_bytes());
    hasher.update(block.receipt_root.as_bytes());
    hasher.update(block.artifact_root.as_bytes());

    let hash_bytes = hasher.finalize();
    let mut hash_array = [0u8; 32];
    hash_array.copy_from_slice(&hash_bytes[..32]);
    Hash::new(hash_array)
}

/// Genesis block configuration
pub struct GenesisConfig {
    #[allow(dead_code)]
    pub chain_id: u64,
    pub timestamp: u64,
    pub initial_accounts: Vec<(PublicKey, u128)>, // (address, balance)
}

impl Default for GenesisConfig {
    fn default() -> Self {
        Self {
            chain_id: 1337,
            timestamp: chrono::Utc::now().timestamp() as u64,
            initial_accounts: vec![
                // Dev account with initial balance (ed25519)
                (PublicKey::new([1; 32]), 1_000_000_000_000_000_000), // 1 ETH worth

                // Forge default deployer account (ECDSA - first 20 bytes are the address, rest zeros)
                // Address: 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266
                (PublicKey::new([
                    0xf3, 0x9F, 0xd6, 0xe5, 0x1a, 0xad, 0x88, 0xF6,
                    0xF4, 0xce, 0x6a, 0xB8, 0x82, 0x72, 0x79, 0xcf,
                    0xfF, 0xb9, 0x22, 0x66, 0x00, 0x00, 0x00, 0x00,
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                ]), 100_000_000_000_000_000_000), // 100 ETH for testing

                // Recovered deployer from failed transaction
                // Address: 0xfcad0b19bb29d4674531d6f115237e16afce377c
                (PublicKey::new([
                    0xfc, 0xad, 0x0b, 0x19, 0xbb, 0x29, 0xd4, 0x67,
                    0x45, 0x31, 0xd6, 0xf1, 0x15, 0x23, 0x7e, 0x16,
                    0xaf, 0xce, 0x37, 0x7c, 0x00, 0x00, 0x00, 0x00,
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                ]), 100_000_000_000_000_000_000), // 100 ETH for testing
            ],
        }
    }
}

/// Create embedded BGE-M3 model for genesis block
fn create_embedded_bge_m3() -> EmbeddedModel {
    // Only embed the actual model when the feature flag is enabled
    // This allows contributors to build without downloading the 417 MB model file
    // The genesis block is loaded from the blockchain database at runtime
    #[cfg(feature = "embed-genesis-model")]
    const BGE_M3_Q4: &[u8] = include_bytes!("../assets/bge-m3-q4.gguf");

    #[cfg(not(feature = "embed-genesis-model"))]
    const BGE_M3_Q4: &[u8] = &[];

    EmbeddedModel {
        model_id: ConsensusModelId::from_name("bge-m3"),
        model_type: ModelType::Embeddings,
        weights: BGE_M3_Q4.to_vec(),
        metadata: ConsensusModelMetadata {
            name: "BGE-M3 Embeddings".to_string(),
            version: "1.0.0".to_string(),
            context_length: 8192,
            embedding_dim: Some(1024),
            license: "MIT".to_string(),
            framework: Some("GGUF".to_string()),
        },
    }
}

/// Create required model for Mistral 7B Instruct v0.3
/// Model is pinned on IPFS and validators must maintain the pin
fn create_required_mistral_7b() -> RequiredModel {
    // SHA256: 1270d22c0fbb3d092fb725d4d96c457b7b687a5f5a715abe1e818da303e562b6
    let sha256_bytes: [u8; 32] = [
        0x12, 0x70, 0xd2, 0x2c, 0x0f, 0xbb, 0x3d, 0x09, 0x2f, 0xb7, 0x25, 0xd4, 0xd9, 0x6c, 0x45, 0x7b,
        0x7b, 0x68, 0x7a, 0x5f, 0x5a, 0x71, 0x5a, 0xbe, 0x1e, 0x81, 0x8d, 0xa3, 0x03, 0xe5, 0x62, 0xb6,
    ];

    RequiredModel::new(
        ConsensusModelId::from_name("mistral-7b-instruct-v0.3"),
        "QmUsYyxg71bV8USRQ6Ccm3SdMqeWgEEVnCYkgNDaxvBTZB".to_string(), // IPFS CID
        Hash::new(sha256_bytes),   // SHA256 hash of GGUF file
        4_367_438_912,             // 4.1 GB (exact file size)
        1_000_000_000_000_000_000_000, // 1000 LATT slash penalty
    )
}

/// Create genesis block
pub fn create_genesis_block(config: &GenesisConfig) -> Block {
    let header = BlockHeader {
        version: 1,
        block_hash: Hash::new([0; 32]),        // Will be computed
        selected_parent_hash: Hash::default(), // No parent for genesis
        merge_parent_hashes: vec![],           // No merge parents for genesis
        timestamp: config.timestamp,
        height: 0,
        blue_score: 0,
        blue_work: 0,
        pruning_point: Hash::default(),
        proposer_pubkey: PublicKey::new([0; 32]),
        vrf_reveal: VrfProof {
            proof: vec![],
            output: Hash::default(),
        },
        // EIP-1559 fields - genesis sets initial base fee
        base_fee_per_gas: 1_000_000_000, // 1 gwei initial base fee
        gas_used: 0,                      // No transactions in genesis
        gas_limit: 30_000_000,            // 30M gas limit
    };

    // Create embedded models for genesis
    let embedded_models = vec![create_embedded_bge_m3()];

    // Create required pin models (validators must pin these)
    let required_pins = vec![create_required_mistral_7b()];

    tracing::info!("Creating genesis block with {} embedded models ({} MB total)",
        embedded_models.len(),
        embedded_models.iter().map(|m| m.size_bytes()).sum::<usize>() / 1_000_000
    );

    Block {
        header,
        state_root: Hash::default(),
        tx_root: Hash::default(),
        receipt_root: Hash::default(),
        artifact_root: Hash::default(),
        ghostdag_params: GhostDagParams::default(),
        transactions: vec![],
        signature: Signature::new([0; 64]),
        embedded_models,
        required_pins,
    }
}

/// Initialize genesis state
pub async fn initialize_genesis_state(
    storage: Arc<StorageManager>,
    executor: Arc<Executor>,
    config: &GenesisConfig,
) -> anyhow::Result<Hash> {
    // Create genesis block
    let mut genesis = create_genesis_block(config);

    // Create economics genesis config
    let economics_config = EconomicsGenesisConfig::default();

    // Initialize genesis accounts from economics config
    for account in &economics_config.accounts {
        // Set initial balance using executor
        executor.set_balance(&account.address, account.balance);

        // Set nonce if non-zero
        if account.nonce > 0 {
            executor.set_nonce(&account.address, account.nonce);
        }

        // Deploy code if provided
        if let Some(code) = &account.code {
            executor.set_code(&account.address, code.clone());
        }

        tracing::info!(
            "Initialized genesis account 0x{} with balance {} LATT ({} wei)",
            hex::encode(account.address.0),
            account.balance / U256::from(10).pow(U256::from(18)),
            account.balance
        );
    }

    // Also initialize legacy initial_accounts if any
    for (address, balance) in &config.initial_accounts {
        // Convert PublicKey to Address (first 20 bytes)
        let addr_bytes = Address(address.0[0..20].try_into().unwrap_or([0; 20]));

        // Set initial balance (convert to U256)
        let balance_u256 = U256::from(*balance);
        executor.set_balance(&addr_bytes, balance_u256);

        tracing::info!(
            "Initialized genesis account 0x{} with balance {} ETH",
            hex::encode(addr_bytes.0),
            balance / 1_000_000_000_000_000_000
        );
    }

    // Register a genesis AI model in state (public access)
    if let Err(e) = register_genesis_model(&storage, &executor, config.timestamp) {
        tracing::warn!("Failed to register genesis model: {}", e);
    }

    // Commit state changes to persist genesis balances
    let state_root = executor.state_db().commit();
    genesis.state_root = Hash::new(*state_root.as_bytes());

    // Calculate block hash
    genesis.header.block_hash = calculate_block_hash(&genesis);

    // Store genesis block in persistent storage
    storage.blocks.put_block(&genesis)?;

    tracing::info!(
        "Genesis block created: {:?} at height 0",
        hex::encode(&genesis.header.block_hash.as_bytes()[..8])
    );

    Ok(genesis.header.block_hash)
}

/// Initialize genesis state with DAG tracking
///
/// This variant also stores the genesis block in the DAG store for consensus tracking.
/// The DagStore maintains the DAG structure for GhostDAG consensus.
pub async fn initialize_genesis_with_dag(
    storage: Arc<StorageManager>,
    executor: Arc<Executor>,
    dag_store: Arc<DagStore>,
    config: &GenesisConfig,
) -> anyhow::Result<Hash> {
    // Use the base initialization
    let genesis_hash = initialize_genesis_state(storage.clone(), executor, config).await?;

    // Retrieve genesis block and add to DAG store
    if let Ok(Some(genesis_block)) = storage.blocks.get_block(&genesis_hash) {
        dag_store.store_block(genesis_block.clone()).await?;
        tracing::info!(
            "Genesis block added to DAG store: {:?}",
            hex::encode(&genesis_hash.as_bytes()[..8])
        );
    } else {
        tracing::warn!("Failed to retrieve genesis block for DAG store");
    }

    Ok(genesis_hash)
}

fn register_genesis_model(
    storage: &Arc<StorageManager>,
    executor: &Arc<Executor>,
    created_at: u64,
) -> anyhow::Result<()> {
    // Embed a tiny ONNX artifact; acts as a placeholder for the genesis model
    // The artifact is not executed here; inference uses the configured InferenceService.
    const ONNX: &[u8] = include_bytes!("../../assets/genesis_model.onnx");

    // Hash artifact to obtain a deterministic model hash/id
    use sha3::{Digest, Keccak256};
    let mut hasher = Keccak256::new();
    hasher.update(ONNX);
    let h = hasher.finalize();
    let mut arr = [0u8; 32];
    arr.copy_from_slice(&h[..32]);
    let model_hash = Hash::new(arr);
    let model_id = ModelId(model_hash);

    // Owner zero for now (public model); can be migrated to governance later
    let owner = Address::zero();

    let metadata = ModelMetadata {
        name: "Genesis BERT Tiny".to_string(),
        version: "1.0.0".to_string(),
        description: "Genesis semantic model placeholder".to_string(),
        framework: "ONNX".to_string(),
        input_shape: vec![1, 128],
        output_shape: vec![1, 128],
        size_bytes: ONNX.len() as u64,
        created_at,
    };

    let model_state = ModelState {
        owner,
        model_hash,
        version: 1,
        metadata,
        access_policy: AccessPolicy::Public,
        usage_stats: UsageStats::default(),
    };

    // Best-effort registration (in-memory registry)
    executor
        .state_db()
        .register_model(model_id, model_state)
        .map_err(|e| anyhow::anyhow!(e.to_string()))
        ?;

    // Persist to storage manager AI state for RPC visibility
    if let Some(model) = executor.state_db().get_model(&model_id) {
        storage
            .state
            .put_model(&model_id, &model)
            .map_err(|e| anyhow::anyhow!(e.to_string()))?;
    }
    Ok(())
}
