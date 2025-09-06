use crate::types::ModelId;
use crate::execution::Model;
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::RwLock;
use anyhow::Result;
use tracing::{debug, info};

/// LRU cache for models
pub struct ModelCache {
    cache: Arc<RwLock<HashMap<ModelId, CachedModel>>>,
    lru_queue: Arc<RwLock<VecDeque<ModelId>>>,
    max_size: u64,
    current_size: Arc<RwLock<u64>>,
}

#[derive(Clone)]
struct CachedModel {
    model: Model,
    size: u64,
    last_accessed: u64,
    access_count: u64,
}

impl ModelCache {
    pub fn new(max_size: u64) -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            lru_queue: Arc::new(RwLock::new(VecDeque::new())),
            max_size,
            current_size: Arc::new(RwLock::new(0)),
        }
    }
    
    /// Get model from cache
    pub async fn get(&self, model_id: &ModelId) -> Option<Model> {
        let mut cache = self.cache.write().await;
        
        if let Some(cached) = cache.get_mut(model_id) {
            // Update access info
            cached.last_accessed = chrono::Utc::now().timestamp() as u64;
            cached.access_count += 1;
            
            // Move to front of LRU queue
            self.update_lru(model_id).await;
            
            debug!("Cache hit for model {:?}", hex::encode(&model_id.0[..8]));
            return Some(cached.model.clone());
        }
        
        None
    }
    
    /// Put model in cache
    pub async fn put(&self, model_id: ModelId, model: Model) -> Result<()> {
        let model_size = self.calculate_model_size(&model);
        
        // Check if model fits in cache
        if model_size > self.max_size {
            return Err(anyhow::anyhow!("Model too large for cache"));
        }
        
        // Evict models if necessary
        while *self.current_size.read().await + model_size > self.max_size {
            self.evict_lru().await?;
        }
        
        // Add to cache
        let cached = CachedModel {
            model,
            size: model_size,
            last_accessed: chrono::Utc::now().timestamp() as u64,
            access_count: 1,
        };
        
        self.cache.write().await.insert(model_id, cached);
        self.lru_queue.write().await.push_front(model_id);
        *self.current_size.write().await += model_size;
        
        debug!("Cached model {:?} (size: {} bytes)", 
               hex::encode(&model_id.0[..8]), 
               model_size);
        
        Ok(())
    }
    
    /// Remove model from cache
    pub async fn remove(&self, model_id: &ModelId) -> Option<Model> {
        let mut cache = self.cache.write().await;
        
        if let Some(cached) = cache.remove(model_id) {
            *self.current_size.write().await -= cached.size;
            
            // Remove from LRU queue
            let mut queue = self.lru_queue.write().await;
            queue.retain(|id| id != model_id);
            
            info!("Removed model {:?} from cache", hex::encode(&model_id.0[..8]));
            return Some(cached.model);
        }
        
        None
    }
    
    /// Clear entire cache
    pub async fn clear(&self) {
        self.cache.write().await.clear();
        self.lru_queue.write().await.clear();
        *self.current_size.write().await = 0;
        
        info!("Cache cleared");
    }
    
    /// Get cache statistics
    pub async fn stats(&self) -> CacheStats {
        let cache = self.cache.read().await;
        let current_size = *self.current_size.read().await;
        
        let total_models = cache.len();
        let total_accesses: u64 = cache.values().map(|c| c.access_count).sum();
        
        CacheStats {
            total_models,
            current_size,
            max_size: self.max_size,
            utilization: (current_size as f64 / self.max_size as f64) * 100.0,
            total_accesses,
        }
    }
    
    /// Preload models into cache
    pub async fn preload(&self, models: Vec<(ModelId, Model)>) -> Result<()> {
        for (model_id, model) in models {
            self.put(model_id, model).await?;
        }
        
        info!("Preloaded {} models into cache", self.cache.read().await.len());
        Ok(())
    }
    
    /// Update LRU queue
    async fn update_lru(&self, model_id: &ModelId) {
        let mut queue = self.lru_queue.write().await;
        
        // Remove from current position
        queue.retain(|id| id != model_id);
        
        // Add to front
        queue.push_front(*model_id);
    }
    
    /// Evict least recently used model
    async fn evict_lru(&self) -> Result<()> {
        let mut queue = self.lru_queue.write().await;
        
        if let Some(model_id) = queue.pop_back() {
            if let Some(cached) = self.cache.write().await.remove(&model_id) {
                *self.current_size.write().await -= cached.size;
                
                debug!("Evicted model {:?} from cache (LRU)", 
                       hex::encode(&model_id.0[..8]));
            }
        }
        
        Ok(())
    }
    
    /// Calculate model size
    fn calculate_model_size(&self, model: &Model) -> u64 {
        let mut size = 0u64;
        size += model.architecture.len() as u64;
        size += model.weights.len() as u64;
        size += model.metadata.len() as u64;
        size += 32; // ModelId size
        size
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub total_models: usize,
    pub current_size: u64,
    pub max_size: u64,
    pub utilization: f64,
    pub total_accesses: u64,
}