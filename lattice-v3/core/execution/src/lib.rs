pub mod address_utils;
pub mod executor;
pub mod metrics;
pub mod parallel;
pub mod state;
pub mod tensor;
pub mod types;
pub mod vm;
pub mod zkp;

pub use types::{
    AccessPolicy, AccountState, Address, ExecutionError, GasSchedule, JobId, JobStatus, Log,
    ModelId, ModelMetadata, ModelState, TrainingJob, TransactionReceipt, TransactionType,
    UsageStats,
};

// Re-export Hash from consensus for MCP to use
pub use lattice_consensus::types::Hash;

pub use state::{AccountManager, StateDB, StateRoot, Trie};

pub use executor::{ExecutionContext, Executor, InferenceService};
pub use parallel::ParallelExecutor;
