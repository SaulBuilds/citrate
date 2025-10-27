// citrate/core/execution/src/lib.rs

// Re-export modules
pub mod address_utils;
pub mod crypto;
pub mod executor;
pub mod inference;
pub mod metrics;
pub mod parallel;
pub mod precompiles;
pub mod state;
pub mod tensor;
pub mod types;
pub mod vm;
pub mod zkp;

// Integration tests
#[cfg(test)]
mod address_derivation_integration_test;

pub use types::{
    AccessPolicy, AccountState, Address, ExecutionError, GasSchedule, JobId, JobStatus, Log,
    ModelId, ModelMetadata, ModelState, TrainingJob, TransactionReceipt, TransactionType,
    UsageStats,
};

// Re-export Hash from consensus for MCP to use
pub use citrate_consensus::types::Hash;

pub use state::{AccountManager, StateDB, StateRoot, Trie};

pub use executor::{ExecutionContext, Executor, InferenceService};
pub use parallel::ParallelExecutor;
pub use precompiles::{PrecompileExecutor, PrecompileResult};
pub use inference::metal_runtime::{MetalRuntime, MetalCapabilities};
