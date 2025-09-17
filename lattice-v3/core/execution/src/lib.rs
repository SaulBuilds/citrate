pub mod types;
pub mod state;
pub mod executor;
pub mod parallel;
pub mod vm;
pub mod tensor;
pub mod zkp;
pub mod address_utils;

pub use types::{
    Address, AccountState, ModelId, ModelState, ModelMetadata, 
    AccessPolicy, TrainingJob, JobId, JobStatus, TransactionType,
    TransactionReceipt, Log, GasSchedule, ExecutionError, UsageStats,
};

// Re-export Hash from consensus for MCP to use
pub use lattice_consensus::types::Hash;

pub use state::{
    AccountManager, StateDB, StateRoot, Trie,
};

pub use executor::{Executor, ExecutionContext};
pub use parallel::{ParallelExecutor};
