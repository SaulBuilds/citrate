use lattice_execution::types::{Address, Hash};
use primitive_types::U256;
use serde::{Deserialize, Serialize};

/// Model identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ModelId(pub [u8; 32]);

impl ModelId {
    pub fn from_hash(hash: &Hash) -> Self {
        let mut id = [0u8; 32];
        id.copy_from_slice(hash.as_bytes());
        Self(id)
    }
    
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

/// Model metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelMetadata {
    pub id: ModelId,
    pub owner: Address,
    pub name: String,
    pub version: String,
    pub hash: Hash,
    pub size: u64,
    pub compute_requirements: ComputeRequirements,
    pub pricing: PricingModel,
}

/// Compute requirements for a model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputeRequirements {
    pub min_memory: u64,        // Minimum RAM in bytes
    pub min_compute: u64,        // Minimum compute units
    pub gpu_required: bool,      // Whether GPU is required
    pub supported_hardware: Vec<HardwareType>,
}

/// Hardware types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HardwareType {
    CPU,
    GPU(String),    // GPU model (e.g., "NVIDIA A100")
    TPU(String),    // TPU version
    Custom(String), // Custom accelerator
}

/// Pricing model for compute
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PricingModel {
    pub base_price: U256,        // Base price per inference
    pub per_token_price: U256,   // Price per token (for LLMs)
    pub per_second_price: U256,  // Price per second of compute
    pub currency: Currency,
}

/// Supported currencies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Currency {
    LAT,    // Native Lattice token
    ETH,    // Ethereum
    USDC,   // USD Coin
}

/// Provider information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderInfo {
    pub address: Address,
    pub name: String,
    pub endpoint: String,
    pub capacity: ComputeCapacity,
    pub reputation: u64,
    pub total_executions: u64,
}

/// Compute capacity of a provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputeCapacity {
    pub total_memory: u64,
    pub available_memory: u64,
    pub total_compute: u64,
    pub available_compute: u64,
    pub hardware: Vec<HardwareType>,
}

/// Execution request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionRequest {
    pub id: RequestId,
    pub model_id: ModelId,
    pub input_hash: Hash,
    pub requester: Address,
    pub provider: Address,
    pub max_price: U256,
    pub status: RequestStatus,
    pub created_at: u64,
}

/// Request identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RequestId(pub [u8; 32]);

/// Request status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RequestStatus {
    Pending,
    Assigned(Address),  // Assigned to provider
    Executing,
    Completed(Hash),    // Result hash
    Failed(String),     // Error message
    Cancelled,
}

/// Execution proof
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionProof {
    pub model_hash: Hash,
    pub input_hash: Hash,
    pub output_hash: Hash,
    pub io_commitment: Hash,
    pub statement: Vec<u8>,
    pub proof_data: Vec<u8>,
    pub timestamp: u64,
    pub provider: Address,
}