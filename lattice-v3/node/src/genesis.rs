use lattice_consensus::{Block, BlockHeader, Transaction, Hash, PublicKey, Signature};
use lattice_storage::StorageManager;
use lattice_execution::{Executor, StateDB};
use chrono::Utc;
use std::sync::Arc;

/// Genesis block configuration
pub struct GenesisConfig {
    pub chain_id: u64,
    pub timestamp: u64,
    pub initial_accounts: Vec<(PublicKey, u128)>, // (address, balance)
}

impl Default for GenesisConfig {
    fn default() -> Self {
        Self {
            chain_id: 1337,
            timestamp: Utc::now().timestamp() as u64,
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
        block_hash: Hash::new([0; 32]), // Will be computed
        parent_hashes: vec![], // No parents for genesis
        height: 0,
        timestamp: config.timestamp,
        difficulty: 1,
        nonce: 0,
        merkle_root: Hash::default(),
        state_root: Hash::default(),
        receipts_root: Hash::default(),
        proposer: PublicKey::new([0; 32]),
        signature: Signature::new([0; 64]),
        blue_score: 0,
        blue_work: 0,
        pruning_point: 0,
        
        // DAG specific
        selected_parent: None,
        blues: vec![],
        reds: vec![],
    };
    
    Block {
        header,
        transactions: vec![],
        uncles: vec![],
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
    
    // Initialize accounts with balances
    for (address, balance) in &config.initial_accounts {
        // Convert PublicKey to Address (first 20 bytes of hash)
        let addr_bytes = {
            let hash = Hash::hash(&address.0);
            let mut addr = [0u8; 20];
            addr.copy_from_slice(&hash.0[..20]);
            lattice_execution::Address(addr)
        };
        
        // Set initial balance
        executor.state_db.accounts.set_balance(&addr, *balance);
    }
    
    // Calculate state root after initializing accounts
    let state_root = executor.calculate_state_root();
    genesis.header.state_root = state_root;
    
    // Calculate block hash
    genesis.header.block_hash = genesis.header.compute_hash();
    
    // Store genesis block
    storage.chain.put_block(&genesis)?;
    storage.chain.put_block_header(&genesis.header)?;
    
    // Mark as genesis
    storage.chain.put_genesis_hash(&genesis.header.block_hash)?;
    
    tracing::info!(
        "Genesis block created: {:?} at height 0",
        hex::encode(&genesis.header.block_hash.0[..8])
    );
    
    Ok(genesis.header.block_hash)
}