// lattice-v3/core/execution/src/types.rs

// Types for representing addresses, models, and transactions
use lattice_consensus::types::{Hash, PublicKey};
use primitive_types::U256;
use serde::{Deserialize, Serialize};

/// Account address (20 bytes, similar to Ethereum)
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct Address(pub [u8; 20]);

impl Address {
    pub fn from_public_key(pubkey: &PublicKey) -> Self {
        // Check if this is an embedded EVM address (first 20 bytes non-zero, last 12 bytes zero)
        let is_evm_address =
            pubkey.0[20..].iter().all(|&b| b == 0) && !pubkey.0[..20].iter().all(|&b| b == 0);

        if is_evm_address {
            // This is an embedded 20-byte EVM address, use it directly
            let mut addr = [0u8; 20];
            addr.copy_from_slice(&pubkey.0[..20]);
            Address(addr)
        } else {
            // This is a full 32-byte public key, derive address by hashing
            use sha3::{Digest, Keccak256};
            let mut hasher = Keccak256::default();
            hasher.update(pubkey.0);
            let hash = hasher.finalize();

            let mut addr = [0u8; 20];
            addr.copy_from_slice(&hash[12..32]);
            Address(addr)
        }
    }

    pub fn zero() -> Self {
        Address([0u8; 20])
    }
}

impl std::fmt::Display for Address {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "0x{}", hex::encode(self.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_address_from_public_key_embedded_evm() {
        // First 20 bytes non-zero, last 12 bytes zero → embedded address
        let mut bytes = [0u8; 32];
        for (i, b) in bytes.iter_mut().enumerate().take(20) {
            *b = (i as u8) + 1;
        }
        // last 12 remain zero
        let pk = PublicKey::new(bytes);
        let addr = Address::from_public_key(&pk);
        assert_eq!(addr.0, bytes[0..20]);
    }

    #[test]
    fn test_address_from_public_key_hashed() {
        // Non-zero across 32 bytes → derive from Keccak256
        let bytes = [0x55u8; 32];
        let pk = PublicKey::new(bytes);
        let addr = Address::from_public_key(&pk);
        assert_ne!(addr.0, bytes[0..20]);
    }
}

/// Model identifier
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct ModelId(pub Hash);

/// Training job identifier
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct JobId(pub Hash);

/// Account state
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AccountState {
    pub nonce: u64,
    pub balance: U256,
    pub storage_root: Hash,
    pub code_hash: Hash,
    pub model_permissions: Vec<ModelId>,
}

impl Default for AccountState {
    fn default() -> Self {
        Self {
            nonce: 0,
            balance: U256::zero(),
            storage_root: Hash::default(),
            code_hash: Hash::default(),
            model_permissions: Vec::new(),
        }
    }
}

/// Model metadata
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct ModelMetadata {
    pub name: String,
    pub version: String,
    pub description: String,
    pub framework: String,
    pub input_shape: Vec<usize>,
    pub output_shape: Vec<usize>,
    pub size_bytes: u64,
    pub created_at: u64,
}

/// Access policy for models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AccessPolicy {
    Public,
    Private,
    Restricted(Vec<Address>),
    PayPerUse { fee: U256 },
}

/// Model state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelState {
    pub owner: Address,
    pub model_hash: Hash,
    pub version: u32,
    pub metadata: ModelMetadata,
    pub access_policy: AccessPolicy,
    pub usage_stats: UsageStats,
}

/// Usage statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UsageStats {
    pub total_inferences: u64,
    pub total_gas_used: u64,
    pub total_fees_earned: U256,
    pub last_used: u64,
}

/// Training job
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingJob {
    pub id: JobId,
    pub owner: Address,
    pub model_id: ModelId,
    pub dataset_hash: Hash,
    pub participants: Vec<Address>,
    pub gradients_submitted: u32,
    pub gradients_required: u32,
    pub reward_pool: U256,
    pub status: JobStatus,
    pub created_at: u64,
    pub completed_at: Option<u64>,
}

/// Job status
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum JobStatus {
    Pending,
    Active,
    Completed,
    Failed,
    Cancelled,
}

/// Transaction types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransactionType {
    /// Standard value transfer
    Transfer { to: Address, value: U256 },

    /// Deploy contract
    Deploy { code: Vec<u8>, init_data: Vec<u8> },

    /// Call contract
    Call {
        to: Address,
        data: Vec<u8>,
        value: U256,
    },

    /// Register a new model
    RegisterModel {
        model_hash: Hash,
        metadata: ModelMetadata,
        access_policy: AccessPolicy,
        artifact_cid: Option<String>,
    },

    /// Update model version
    UpdateModel {
        model_id: ModelId,
        metadata: ModelMetadata,
        artifact_cid: Option<String>,
    },

    /// Request inference
    InferenceRequest {
        model_id: ModelId,
        input_data: Vec<u8>,
        max_gas: u64,
    },

    /// Submit training gradient
    SubmitGradient {
        job_id: JobId,
        gradient_data: Vec<u8>,
        proof: Vec<u8>,
    },
}

/// Transaction receipt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionReceipt {
    pub tx_hash: Hash,
    pub block_hash: Hash,
    pub block_number: u64,
    pub from: Address,
    pub to: Option<Address>,
    pub gas_used: u64,
    pub status: bool,
    pub logs: Vec<Log>,
    pub output: Vec<u8>,
}

/// Event log
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Log {
    pub address: Address,
    pub topics: Vec<Hash>,
    pub data: Vec<u8>,
}

/// Gas schedule
#[derive(Debug, Clone)]
pub struct GasSchedule {
    // Basic operations
    pub transfer: u64,
    pub sstore: u64,
    pub sload: u64,
    pub create: u64,
    pub call: u64,

    // AI operations
    pub model_register: u64,
    pub model_update: u64,
    pub inference_base: u64,
    pub inference_per_mb: u64,
    pub training_submit: u64,

    // AI opcodes
    pub tensor_op: u64,
    pub model_load: u64,
    pub model_exec: u64,
    pub zk_prove: u64,
    pub zk_verify: u64,

    // Arithmetic operations
    pub add: u64,
    pub mul: u64,
    pub sub: u64,
    pub div: u64,
    pub exp: u64,
    pub sha3: u64,

    // Stack operations
    pub push: u64,
    pub pop: u64,
    pub mload: u64,
    pub mstore: u64,

    // Control flow
    pub jump: u64,
    pub jumpi: u64,
}

impl Default for GasSchedule {
    fn default() -> Self {
        Self {
            // Basic operations
            transfer: 21_000,
            sstore: 20_000,
            sload: 800,
            create: 32_000,
            call: 700,

            // AI operations
            model_register: 100_000,
            model_update: 50_000,
            inference_base: 50_000,
            inference_per_mb: 10_000,
            training_submit: 200_000,

            // AI opcodes
            tensor_op: 10_000,
            model_load: 50_000,
            model_exec: 100_000,
            zk_prove: 200_000,
            zk_verify: 50_000,

            // Arithmetic operations
            add: 3,
            mul: 5,
            sub: 3,
            div: 5,
            exp: 10,
            sha3: 30,
            push: 3,
            pop: 2,
            mload: 3,
            mstore: 3,
            jump: 8,
            jumpi: 10,
        }
    }
}

/// Execution error
#[derive(Debug, thiserror::Error)]
pub enum ExecutionError {
    #[error("Insufficient balance: need {need}, have {have}")]
    InsufficientBalance { need: U256, have: U256 },

    #[error("Invalid nonce: expected {expected}, got {got}")]
    InvalidNonce { expected: u64, got: u64 },

    #[error("Out of gas")]
    OutOfGas,

    #[error("Stack overflow")]
    StackOverflow,

    #[error("Invalid opcode: {0}")]
    InvalidOpcode(u8),

    #[error("Account not found: {0}")]
    AccountNotFound(Address),

    #[error("Model not found: {0:?}")]
    ModelNotFound(ModelId),

    #[error("Access denied")]
    AccessDenied,

    #[error("Invalid input data")]
    InvalidInput,

    #[error("Execution reverted: {0}")]
    Reverted(String),

    #[error("Stack underflow")]
    StackUnderflow,

    #[error("Invalid model")]
    InvalidModel,

    #[error("Invalid tensor")]
    InvalidTensor,

    #[error("Tensor shape mismatch")]
    TensorShapeMismatch,

    #[error("Invalid opcode")]
    InvalidOpcodeGeneric,
}
