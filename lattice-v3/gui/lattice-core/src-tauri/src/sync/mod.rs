// Sync module for iterative block synchronization
// Avoids stack overflow issues with recursive DAG processing

pub mod iterative_sync;

pub use iterative_sync::{IterativeSyncManager, SyncConfig, SyncState};