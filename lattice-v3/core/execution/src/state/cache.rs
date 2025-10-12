use crate::types::{AccountState, Address};
use lru::LruCache;
use parking_lot::RwLock;
use primitive_types::U256;
use std::num::NonZeroUsize;
use std::sync::Arc;

/// Cache statistics for monitoring
#[derive(Debug, Clone, Default)]
pub struct CacheStats {
    pub account_hits: u64,
    pub account_misses: u64,
    pub storage_hits: u64,
    pub storage_misses: u64,
    pub code_hits: u64,
    pub code_misses: u64,
}

impl CacheStats {
    pub fn account_hit_rate(&self) -> f64 {
        let total = self.account_hits + self.account_misses;
        if total == 0 {
            0.0
        } else {
            self.account_hits as f64 / total as f64
        }
    }

    pub fn storage_hit_rate(&self) -> f64 {
        let total = self.storage_hits + self.storage_misses;
        if total == 0 {
            0.0
        } else {
            self.storage_hits as f64 / total as f64
        }
    }

    pub fn code_hit_rate(&self) -> f64 {
        let total = self.code_hits + self.code_misses;
        if total == 0 {
            0.0
        } else {
            self.code_hits as f64 / total as f64
        }
    }
}

/// Storage key for cache
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct StorageKey {
    pub address: Address,
    pub key: U256,
}

/// Multi-level cache for StateDB
pub struct StateCache {
    /// Account cache
    accounts: Arc<RwLock<LruCache<Address, AccountState>>>,

    /// Storage cache (address + key -> value)
    storage: Arc<RwLock<LruCache<StorageKey, U256>>>,

    /// Code cache
    code: Arc<RwLock<LruCache<Address, Vec<u8>>>>,

    /// Cache statistics
    stats: Arc<RwLock<CacheStats>>,

    /// Prefetch configuration
    prefetch_enabled: bool,
    prefetch_depth: usize,
}

impl StateCache {
    pub fn new(
        account_cache_size: usize,
        storage_cache_size: usize,
        code_cache_size: usize,
    ) -> Self {
        Self {
            accounts: Arc::new(RwLock::new(LruCache::new(
                NonZeroUsize::new(account_cache_size).unwrap(),
            ))),
            storage: Arc::new(RwLock::new(LruCache::new(
                NonZeroUsize::new(storage_cache_size).unwrap(),
            ))),
            code: Arc::new(RwLock::new(LruCache::new(
                NonZeroUsize::new(code_cache_size).unwrap(),
            ))),
            stats: Arc::new(RwLock::new(CacheStats::default())),
            prefetch_enabled: true,
            prefetch_depth: 4,
        }
    }

    /// Get account from cache
    pub fn get_account(&self, address: &Address) -> Option<AccountState> {
        let mut cache = self.accounts.write();
        let mut stats = self.stats.write();

        if let Some(account) = cache.get(address) {
            stats.account_hits += 1;
            Some(account.clone())
        } else {
            stats.account_misses += 1;
            None
        }
    }

    /// Put account into cache
    pub fn put_account(&self, address: Address, account: AccountState) {
        let mut cache = self.accounts.write();
        cache.put(address, account);
    }

    /// Get storage value from cache
    pub fn get_storage(&self, address: &Address, key: &U256) -> Option<U256> {
        let storage_key = StorageKey {
            address: *address,
            key: *key,
        };

        let mut cache = self.storage.write();
        let mut stats = self.stats.write();

        if let Some(value) = cache.get(&storage_key) {
            stats.storage_hits += 1;
            Some(*value)
        } else {
            stats.storage_misses += 1;
            None
        }
    }

    /// Put storage value into cache
    pub fn put_storage(&self, address: Address, key: U256, value: U256) {
        let storage_key = StorageKey { address, key };
        let mut cache = self.storage.write();
        cache.put(storage_key, value);

        // Prefetch adjacent keys if enabled
        if self.prefetch_enabled {
            self.prefetch_storage_keys(address, key);
        }
    }

    /// Prefetch adjacent storage keys
    fn prefetch_storage_keys(&self, address: Address, key: U256) {
        // Simple prefetch strategy: cache nearby sequential keys
        // This helps with array/mapping iterations
        for i in 1..=self.prefetch_depth {
            if let Some(next_key) = key.checked_add(U256::from(i)) {
                let storage_key = StorageKey {
                    address,
                    key: next_key,
                };
                // Mark for prefetch (would trigger async load in production)
                // For now, just ensure the key slot exists
                let _ = storage_key;
            }
        }
    }

    /// Get code from cache
    pub fn get_code(&self, address: &Address) -> Option<Vec<u8>> {
        let mut cache = self.code.write();
        let mut stats = self.stats.write();

        if let Some(code) = cache.get(address) {
            stats.code_hits += 1;
            Some(code.clone())
        } else {
            stats.code_misses += 1;
            None
        }
    }

    /// Put code into cache
    pub fn put_code(&self, address: Address, code: Vec<u8>) {
        let mut cache = self.code.write();
        cache.put(address, code);
    }

    /// Clear all caches
    pub fn clear(&self) {
        self.accounts.write().clear();
        self.storage.write().clear();
        self.code.write().clear();
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        self.stats.read().clone()
    }

    /// Reset statistics
    pub fn reset_stats(&self) {
        *self.stats.write() = CacheStats::default();
    }

    /// Hot/cold separation: mark an address as hot
    pub fn mark_hot(&self, address: &Address) {
        // Promote to front of LRU
        let mut cache = self.accounts.write();
        if let Some(account) = cache.get(address) {
            let _account = account.clone();
            cache.promote(address);
            // In production, we might move to a separate hot cache
        }
    }

    /// Enable or disable prefetching
    pub fn set_prefetch(&mut self, enabled: bool) {
        self.prefetch_enabled = enabled;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lattice_consensus::types::Hash;

    #[test]
    fn test_account_cache() {
        let cache = StateCache::new(10, 100, 10);

        let addr = Address([1u8; 20]);
        let account = AccountState {
            nonce: 1,
            balance: U256::from(1000),
            code_hash: Hash::default(),
            storage_root: Hash::default(),
            model_permissions: vec![],
        };

        // Miss on first access
        assert!(cache.get_account(&addr).is_none());
        assert_eq!(cache.stats().account_misses, 1);

        // Put and hit
        cache.put_account(addr, account.clone());
        assert_eq!(cache.get_account(&addr), Some(account));
        assert_eq!(cache.stats().account_hits, 1);
    }

    #[test]
    fn test_storage_cache() {
        let cache = StateCache::new(10, 100, 10);

        let addr = Address([1u8; 20]);
        let key = U256::from(42);
        let value = U256::from(100);

        // Miss on first access
        assert!(cache.get_storage(&addr, &key).is_none());
        assert_eq!(cache.stats().storage_misses, 1);

        // Put and hit
        cache.put_storage(addr, key, value);
        assert_eq!(cache.get_storage(&addr, &key), Some(value));
        assert_eq!(cache.stats().storage_hits, 1);
    }

    #[test]
    fn test_hit_rates() {
        let cache = StateCache::new(10, 100, 10);
        let addr = Address([1u8; 20]);

        // Initial hit rate is 0
        assert_eq!(cache.stats().account_hit_rate(), 0.0);

        // After 1 miss
        cache.get_account(&addr);
        assert_eq!(cache.stats().account_hit_rate(), 0.0);

        // After 1 miss and 3 hits
        cache.put_account(
            addr,
            AccountState {
                nonce: 0,
                balance: U256::zero(),
                code_hash: Hash::default(),
                storage_root: Hash::default(),
                model_permissions: vec![],
            },
        );
        cache.get_account(&addr);
        cache.get_account(&addr);
        cache.get_account(&addr);

        assert_eq!(cache.stats().account_hit_rate(), 0.75); // 3 hits / 4 total
    }
}
