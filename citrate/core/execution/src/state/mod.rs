// citrate/core/execution/src/state/mod.rs

// State management module
// Handles account states, caching, and trie operations

pub mod account;
pub mod cache;
pub mod state_db;
pub mod trie;

pub use account::AccountManager;
pub use state_db::{StateDB, StateRoot};
pub use trie::{Trie, TrieNode};
