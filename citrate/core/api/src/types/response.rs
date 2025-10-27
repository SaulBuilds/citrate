// citrate/core/api/src/types/response.rs
use citrate_consensus::types::{Block, Hash, Transaction};
use citrate_execution::types::Address;
use primitive_types::U256;
use serde::{Deserialize, Serialize};

/// Block response with transactions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockResponse {
    pub hash: Hash,
    pub height: u64,
    pub parent_hash: Hash,
    pub timestamp: u64,
    pub blue_score: u64,
    pub blue_work: u128,
    pub transactions: Vec<TransactionResponse>,
    pub state_root: Hash,
    pub tx_root: Hash,
}

impl From<Block> for BlockResponse {
    fn from(block: Block) -> Self {
        Self {
            hash: block.header.block_hash,
            height: block.header.height,
            parent_hash: block.header.selected_parent_hash,
            timestamp: block.header.timestamp,
            blue_score: block.header.blue_score,
            blue_work: block.header.blue_work,
            transactions: block.transactions.into_iter().map(Into::into).collect(),
            state_root: block.state_root,
            tx_root: block.tx_root,
        }
    }
}

/// Transaction response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionResponse {
    pub hash: Hash,
    pub nonce: u64,
    pub from: String,
    pub to: Option<String>,
    pub value: U256,
    pub gas_limit: u64,
    pub gas_price: u64,
    pub data: Vec<u8>,
}

impl From<Transaction> for TransactionResponse {
    fn from(tx: Transaction) -> Self {
        Self {
            hash: tx.hash,
            nonce: tx.nonce,
            from: hex::encode(tx.from.as_bytes()),
            to: tx.to.map(|t| hex::encode(t.as_bytes())),
            value: U256::from(tx.value),
            gas_limit: tx.gas_limit,
            gas_price: tx.gas_price,
            data: tx.data,
        }
    }
}

/// Account response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountResponse {
    pub address: Address,
    pub balance: U256,
    pub nonce: u64,
    pub code_hash: Hash,
    pub storage_root: Hash,
    pub model_permissions: Vec<String>,
}

/// Sync status response
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SyncStatus {
    Syncing(SyncProgress),
    NotSyncing(bool),
}

/// Sync progress information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncProgress {
    pub starting_block: u64,
    pub current_block: u64,
    pub highest_block: u64,
    pub pulled_states: Option<u64>,
    pub known_states: Option<u64>,
}

/// Node information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeInfo {
    pub version: String,
    pub network_id: u64,
    pub chain_id: u64,
    pub genesis_hash: Hash,
    pub head_hash: Hash,
    pub head_height: u64,
    pub peer_count: usize,
}

/// Mempool status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MempoolStatus {
    pub pending: usize,
    pub queued: usize,
    pub total_size: usize,
    pub max_size: usize,
}
