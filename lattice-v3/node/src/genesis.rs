use lattice_consensus::types::{
    Block, BlockHeader, GhostDagParams, Hash, PublicKey, Signature, VrfProof,
};
use lattice_economics::genesis::GenesisConfig as EconomicsGenesisConfig;
use lattice_execution::executor::Executor;
use lattice_execution::types::Address;
use lattice_storage::StorageManager;
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
                // Dev account with initial balance
                (PublicKey::new([1; 32]), 1_000_000_000_000_000_000), // 1 ETH worth
            ],
        }
    }
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
    }

    // Calculate state root after initializing accounts
    // TODO: Need public API to calculate state root
    genesis.state_root = Hash::default();

    // Calculate block hash
    genesis.header.block_hash = calculate_block_hash(&genesis);

    // Store genesis block
    storage.blocks.put_block(&genesis)?;

    // TODO: Initialize DAG tracking for genesis block
    // This would be done through consensus module, not storage directly

    tracing::info!(
        "Genesis block created: {:?} at height 0",
        hex::encode(&genesis.header.block_hash.as_bytes()[..8])
    );

    Ok(genesis.header.block_hash)
}
