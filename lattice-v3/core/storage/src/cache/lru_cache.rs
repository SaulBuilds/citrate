use lru::LruCache;
use parking_lot::RwLock;
use std::hash::Hash;
use std::num::NonZeroUsize;
use std::sync::Arc;

/// Thread-safe LRU cache wrapper
pub struct Cache<K: Hash + Eq + Clone, V: Clone> {
    inner: Arc<RwLock<LruCache<K, V>>>,
}

impl<K: Hash + Eq + Clone, V: Clone> Cache<K, V> {
    /// Create a new cache with specified capacity
    pub fn new(capacity: usize) -> Self {
        let capacity = NonZeroUsize::new(capacity).expect("Cache capacity must be non-zero");
        Self {
            inner: Arc::new(RwLock::new(LruCache::new(capacity))),
        }
    }

    /// Get a value from cache
    pub fn get(&self, key: &K) -> Option<V> {
        self.inner.write().get(key).cloned()
    }

    /// Put a value into cache
    pub fn put(&self, key: K, value: V) -> Option<V> {
        self.inner.write().put(key, value)
    }

    /// Remove a value from cache
    pub fn remove(&self, key: &K) -> Option<V> {
        self.inner.write().pop(key)
    }

    /// Check if key exists
    pub fn contains(&self, key: &K) -> bool {
        self.inner.read().contains(key)
    }

    /// Clear all entries
    pub fn clear(&self) {
        self.inner.write().clear();
    }

    /// Get current size
    pub fn len(&self) -> usize {
        self.inner.read().len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.inner.read().is_empty()
    }
}

impl<K: Hash + Eq + Clone, V: Clone> Clone for Cache<K, V> {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

/// Block cache entry
#[derive(Clone)]
pub struct BlockCacheEntry {
    pub data: Vec<u8>,
    pub height: u64,
    pub blue_score: u64,
}

/// State cache entry
#[derive(Clone)]
pub struct StateCacheEntry {
    pub data: Vec<u8>,
    pub version: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lru_cache() {
        let cache: Cache<String, String> = Cache::new(3);

        // Add items
        cache.put("key1".to_string(), "value1".to_string());
        cache.put("key2".to_string(), "value2".to_string());
        cache.put("key3".to_string(), "value3".to_string());

        // Get items
        assert_eq!(cache.get(&"key1".to_string()), Some("value1".to_string()));
        assert_eq!(cache.get(&"key2".to_string()), Some("value2".to_string()));

        // Add fourth item (should evict least recently used)
        cache.put("key4".to_string(), "value4".to_string());

        // key3 should be evicted (least recently used)
        assert_eq!(cache.get(&"key3".to_string()), None);
        assert_eq!(cache.get(&"key4".to_string()), Some("value4".to_string()));

        // Test size
        assert_eq!(cache.len(), 3);
    }

    #[test]
    fn test_cache_operations() {
        let cache: Cache<u64, String> = Cache::new(10);

        // Test empty
        assert!(cache.is_empty());

        // Add and check contains
        cache.put(1, "one".to_string());
        assert!(cache.contains(&1));
        assert!(!cache.contains(&2));

        // Remove
        assert_eq!(cache.remove(&1), Some("one".to_string()));
        assert!(!cache.contains(&1));

        // Clear
        cache.put(1, "one".to_string());
        cache.put(2, "two".to_string());
        cache.clear();
        assert!(cache.is_empty());
    }
}
