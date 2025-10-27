// citrate/core/execution/src/parallel/mod.rs

// Parallel execution module
// Enables concurrent transaction processing with conflict detection

pub mod conflict;
pub mod executor;

pub use conflict::{AccessSet, AccessSetExtractor, ConflictScheduler, DefaultAccessSetExtractor};
pub use executor::ParallelExecutor;
