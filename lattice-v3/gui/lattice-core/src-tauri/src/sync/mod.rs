// Sync module for iterative block synchronization
// Avoids stack overflow issues with recursive DAG processing

pub mod iterative_sync;

// Re-exports disabled to avoid unused-imports warnings in this crate. Callers can import from submodule.
