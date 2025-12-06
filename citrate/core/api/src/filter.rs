// citrate/core/api/src/filter.rs

use citrate_consensus::types::Hash;
use citrate_execution::types::Address;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::RwLock;
use std::time::{Duration, Instant};

/// Filter types supported by eth_newFilter and related methods
#[derive(Clone, Debug)]
pub enum FilterType {
    /// Log filter with address and topic criteria
    Log {
        from_block: Option<u64>,
        to_block: Option<u64>,
        addresses: Vec<Address>,
        topics: Vec<Option<Vec<Hash>>>,
    },
    /// Block filter - returns new block hashes
    Block,
    /// Pending transaction filter - returns new pending tx hashes
    PendingTransaction,
}

/// A registered filter with metadata
#[derive(Clone, Debug)]
pub struct Filter {
    pub filter_type: FilterType,
    pub last_poll_block: u64,
    pub created_at: Instant,
    pub last_polled_at: Instant,
}

/// Filter registry for managing eth_newFilter/eth_getFilterChanges state
pub struct FilterRegistry {
    filters: RwLock<HashMap<u64, Filter>>,
    next_id: AtomicU64,
    /// Filters older than this are eligible for cleanup
    max_filter_age: Duration,
}

impl Default for FilterRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl FilterRegistry {
    /// Create a new filter registry
    pub fn new() -> Self {
        Self {
            filters: RwLock::new(HashMap::new()),
            next_id: AtomicU64::new(1),
            max_filter_age: Duration::from_secs(5 * 60), // 5 minute timeout
        }
    }

    /// Create a new log filter and return its ID
    pub fn new_log_filter(
        &self,
        from_block: Option<u64>,
        to_block: Option<u64>,
        addresses: Vec<Address>,
        topics: Vec<Option<Vec<Hash>>>,
        current_block: u64,
    ) -> u64 {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        let now = Instant::now();

        let filter = Filter {
            filter_type: FilterType::Log {
                from_block,
                to_block,
                addresses,
                topics,
            },
            last_poll_block: current_block,
            created_at: now,
            last_polled_at: now,
        };

        self.filters.write().unwrap().insert(id, filter);
        id
    }

    /// Create a new block filter and return its ID
    pub fn new_block_filter(&self, current_block: u64) -> u64 {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        let now = Instant::now();

        let filter = Filter {
            filter_type: FilterType::Block,
            last_poll_block: current_block,
            created_at: now,
            last_polled_at: now,
        };

        self.filters.write().unwrap().insert(id, filter);
        id
    }

    /// Create a new pending transaction filter and return its ID
    pub fn new_pending_transaction_filter(&self) -> u64 {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        let now = Instant::now();

        let filter = Filter {
            filter_type: FilterType::PendingTransaction,
            last_poll_block: 0,
            created_at: now,
            last_polled_at: now,
        };

        self.filters.write().unwrap().insert(id, filter);
        id
    }

    /// Get a filter by ID and update its last polled time
    pub fn get_filter(&self, id: u64) -> Option<Filter> {
        let mut filters = self.filters.write().unwrap();
        if let Some(filter) = filters.get_mut(&id) {
            filter.last_polled_at = Instant::now();
            Some(filter.clone())
        } else {
            None
        }
    }

    /// Update the last poll block for a filter
    pub fn update_last_poll_block(&self, id: u64, block: u64) {
        if let Some(filter) = self.filters.write().unwrap().get_mut(&id) {
            filter.last_poll_block = block;
            filter.last_polled_at = Instant::now();
        }
    }

    /// Uninstall (remove) a filter
    pub fn uninstall_filter(&self, id: u64) -> bool {
        self.filters.write().unwrap().remove(&id).is_some()
    }

    /// Clean up stale filters that haven't been polled recently
    pub fn cleanup_stale_filters(&self) {
        let now = Instant::now();
        let mut filters = self.filters.write().unwrap();
        filters.retain(|_, filter| {
            now.duration_since(filter.last_polled_at) < self.max_filter_age
        });
    }

    /// Get the number of active filters
    pub fn filter_count(&self) -> usize {
        self.filters.read().unwrap().len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_filter_creation() {
        let registry = FilterRegistry::new();
        let id = registry.new_log_filter(Some(0), Some(100), vec![], vec![], 50);
        assert!(id > 0);

        let filter = registry.get_filter(id);
        assert!(filter.is_some());

        if let Some(f) = filter {
            match f.filter_type {
                FilterType::Log { from_block, to_block, .. } => {
                    assert_eq!(from_block, Some(0));
                    assert_eq!(to_block, Some(100));
                }
                _ => panic!("Expected Log filter type"),
            }
        }
    }

    #[test]
    fn test_block_filter_creation() {
        let registry = FilterRegistry::new();
        let id = registry.new_block_filter(100);

        let filter = registry.get_filter(id);
        assert!(filter.is_some());
        assert!(matches!(filter.unwrap().filter_type, FilterType::Block));
    }

    #[test]
    fn test_filter_uninstall() {
        let registry = FilterRegistry::new();
        let id = registry.new_block_filter(100);

        assert!(registry.get_filter(id).is_some());
        assert!(registry.uninstall_filter(id));
        assert!(registry.get_filter(id).is_none());
        assert!(!registry.uninstall_filter(id)); // Second uninstall should return false
    }

    #[test]
    fn test_filter_update() {
        let registry = FilterRegistry::new();
        let id = registry.new_block_filter(100);

        registry.update_last_poll_block(id, 200);

        let filter = registry.get_filter(id);
        assert!(filter.is_some());
        assert_eq!(filter.unwrap().last_poll_block, 200);
    }
}
