// lattice-v3/core/api/src/types/request.rs
use lattice_consensus::types::Hash;
use lattice_execution::types::Address;
use primitive_types::U256;
use serde::{Deserialize, Serialize};

/// Block identifier - can be hash, number, or tag
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum BlockId {
    Hash(Hash),
    Number(u64),
    Tag(BlockTag),
}

/// Block tags for special blocks
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BlockTag {
    Latest,
    Earliest,
    Pending,
}

/// Transaction request for sending
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionRequest {
    pub from: Address,
    pub to: Option<Address>,
    pub value: Option<U256>,
    pub gas: Option<u64>,
    pub gas_price: Option<u64>,
    pub nonce: Option<u64>,
    pub data: Option<Vec<u8>>,
}

/// Call request for read-only execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallRequest {
    pub from: Option<Address>,
    pub to: Address,
    pub value: Option<U256>,
    pub gas: Option<u64>,
    pub gas_price: Option<u64>,
    pub data: Option<Vec<u8>>,
}

/// Filter for logs/events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogFilter {
    pub from_block: Option<BlockId>,
    pub to_block: Option<BlockId>,
    pub address: Option<Vec<Address>>,
    pub topics: Option<Vec<Option<Vec<Hash>>>>,
}

/// Subscription types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SubscriptionType {
    NewHeads,
    NewPendingTransactions,
    Logs(LogFilter),
    Syncing,
}
