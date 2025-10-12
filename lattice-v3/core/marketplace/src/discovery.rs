// lattice-v3/core/marketplace/src/discovery.rs

use crate::{
    metadata::MetadataCache,
    recommendations::RecommendationEngine,
    search::{SearchEngine, SearchQuery, SearchResult},
    storage::MarketplaceStorage,
    types::*,
};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info, warn};

/// Configuration for the discovery engine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryConfig {
    pub search_index_path: PathBuf,
    pub storage_path: PathBuf,
    pub ipfs_gateways: Vec<String>,
    pub cache_ttl_seconds: u64,
    pub max_cache_size: usize,
    pub enable_recommendations: bool,
}

impl Default for DiscoveryConfig {
    fn default() -> Self {
        Self {
            search_index_path: PathBuf::from("./data/search_index"),
            storage_path: PathBuf::from("./data/marketplace.db"),
            ipfs_gateways: vec![
                "https://ipfs.io/ipfs/".to_string(),
                "https://gateway.pinata.cloud/ipfs/".to_string(),
                "https://cloudflare-ipfs.com/ipfs/".to_string(),
            ],
            cache_ttl_seconds: 3600, // 1 hour
            max_cache_size: 10000,
            enable_recommendations: true,
        }
    }
}

/// Main discovery engine that coordinates search, metadata, and recommendations
pub struct DiscoveryEngine {
    search_engine: Arc<SearchEngine>,
    metadata_cache: Arc<MetadataCache>,
    storage: Arc<MarketplaceStorage>,
    recommendation_engine: Option<Arc<RecommendationEngine>>,
    config: DiscoveryConfig,
    stats: Arc<RwLock<DiscoveryStats>>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct DiscoveryStats {
    pub total_models_indexed: u64,
    pub total_searches: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub ipfs_fetches: u64,
    pub recommendation_requests: u64,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

impl DiscoveryEngine {
    /// Create a new discovery engine
    pub async fn new(config: DiscoveryConfig) -> Result<Self> {
        info!("Initializing Lattice Discovery Engine");

        // Initialize search engine
        let search_engine = Arc::new(SearchEngine::new(&config.search_index_path).await?);
        info!("Search engine initialized");

        // Initialize metadata cache
        let metadata_cache = Arc::new(
            MetadataCache::new()
                .with_gateways(config.ipfs_gateways.clone())
                .with_ttl(std::time::Duration::from_secs(config.cache_ttl_seconds))
                .with_max_cache_size(config.max_cache_size),
        );

        // Start background cleanup task for metadata cache
        let cache_for_cleanup = Arc::clone(&metadata_cache);
        cache_for_cleanup.start_cleanup_task();
        info!("Metadata cache initialized");

        // Initialize storage
        let storage = Arc::new(MarketplaceStorage::new(&config.storage_path).await?);
        info!("Storage initialized");

        // Initialize recommendation engine if enabled
        let recommendation_engine = if config.enable_recommendations {
            let rec_engine = RecommendationEngine::new(Arc::clone(&storage)).await?;
            info!("Recommendation engine initialized");
            Some(Arc::new(rec_engine))
        } else {
            None
        };

        let stats = Arc::new(RwLock::new(DiscoveryStats {
            last_updated: chrono::Utc::now(),
            ..Default::default()
        }));

        Ok(Self {
            search_engine,
            metadata_cache,
            storage,
            recommendation_engine,
            config,
            stats,
        })
    }

    /// Index a new model for discovery
    pub async fn index_model(&self, model: &MarketplaceModel) -> Result<()> {
        // Index for search
        self.search_engine.index_model(model).await?;

        // Store in database
        self.storage.store_model(model).await?;

        // Update stats
        {
            let mut stats = self.stats.write().await;
            stats.total_models_indexed += 1;
            stats.last_updated = chrono::Utc::now();
        }

        // If metadata URI is provided, prefetch and cache it
        if !model.metadata_uri.is_empty() {
            tokio::spawn({
                let cache = Arc::clone(&self.metadata_cache);
                let uri = model.metadata_uri.clone();
                async move {
                    if let Err(e) = cache.get_metadata(&uri).await {
                        warn!(uri = %uri, error = %e, "Failed to prefetch metadata");
                    }
                }
            });
        }

        info!(model_id = ?model.model_id, model_name = %model.name, "Model indexed successfully");
        Ok(())
    }

    /// Remove a model from discovery
    pub async fn remove_model(&self, model_id: &ModelId) -> Result<()> {
        // Remove from search index
        self.search_engine.remove_model(model_id).await?;

        // Remove from storage
        self.storage.remove_model(model_id).await?;

        info!(model_id = ?model_id, "Model removed from discovery");
        Ok(())
    }

    /// Update an existing model
    pub async fn update_model(&self, model: &MarketplaceModel) -> Result<()> {
        // Re-index for search (this will replace the existing entry)
        self.search_engine.index_model(model).await?;

        // Update in storage
        self.storage.update_model(model).await?;

        info!(model_id = ?model.model_id, "Model updated in discovery");
        Ok(())
    }

    /// Search for models
    pub async fn search(&self, query: &SearchQuery) -> Result<Vec<SearchResult>> {
        // Update stats
        {
            let mut stats = self.stats.write().await;
            stats.total_searches += 1;
        }

        // Perform search
        let results = self.search_engine.search(query).await?;

        // Enrich results with complete metadata if available
        let enriched_results = self.enrich_search_results(results).await?;

        Ok(enriched_results)
    }

    /// Get model details with full metadata
    pub async fn get_model_details(&self, model_id: &ModelId) -> Result<Option<(MarketplaceModel, Option<crate::metadata::ModelMetadata>)>> {
        // Get basic model info from storage
        let model = match self.storage.get_model(model_id).await? {
            Some(model) => model,
            None => return Ok(None),
        };

        // Fetch extended metadata from IPFS if available
        let extended_metadata = if !model.metadata_uri.is_empty() {
            match self.get_metadata(&model.metadata_uri).await {
                Ok(metadata) => Some(metadata),
                Err(e) => {
                    warn!(
                        model_id = ?model_id,
                        metadata_uri = %model.metadata_uri,
                        error = %e,
                        "Failed to fetch extended metadata"
                    );
                    None
                }
            }
        } else {
            None
        };

        Ok(Some((model, extended_metadata)))
    }

    /// Get metadata from IPFS with caching
    pub async fn get_metadata(&self, ipfs_cid: &str) -> Result<crate::metadata::ModelMetadata> {
        let result = self.metadata_cache.get_metadata(ipfs_cid).await;

        // Update cache stats
        {
            let mut stats = self.stats.write().await;
            match &result {
                Ok(_) => stats.cache_hits += 1,
                Err(_) => stats.cache_misses += 1,
            }
            stats.ipfs_fetches += 1;
        }

        result
    }

    /// Get trending models
    pub async fn get_trending_models(&self, limit: usize) -> Result<Vec<MarketplaceModel>> {
        let models = self.search_engine.get_trending_models(limit).await?;
        self.enrich_models_with_storage(models).await
    }

    /// Get similar models based on a given model
    pub async fn get_similar_models(&self, model_id: &ModelId, limit: usize) -> Result<Vec<MarketplaceModel>> {
        let models = self.search_engine.get_similar_models(model_id, limit).await?;
        self.enrich_models_with_storage(models).await
    }

    /// Get personalized recommendations for a user
    pub async fn get_recommendations(&self, user_address: &Address, limit: usize) -> Result<Vec<MarketplaceModel>> {
        if let Some(rec_engine) = &self.recommendation_engine {
            // Update stats
            {
                let mut stats = self.stats.write().await;
                stats.recommendation_requests += 1;
            }

            let model_ids = rec_engine.get_recommendations(user_address, limit).await?;

            // Convert model IDs to full models
            let mut models = Vec::new();
            for model_id in model_ids {
                if let Some(model) = self.storage.get_model(&model_id).await? {
                    models.push(model);
                }
            }

            Ok(models)
        } else {
            // Fallback to trending models if recommendations are disabled
            self.get_trending_models(limit).await
        }
    }

    /// Record user interaction for recommendations
    pub async fn record_interaction(&self, interaction: &UserInteraction) -> Result<()> {
        self.storage.record_interaction(interaction).await?;

        // Update recommendation engine if available
        if let Some(rec_engine) = &self.recommendation_engine {
            rec_engine.update_user_profile(&interaction.user).await?;
        }

        Ok(())
    }

    /// Get marketplace statistics
    pub async fn get_marketplace_stats(&self) -> Result<MarketplaceStats> {
        self.storage.get_marketplace_stats().await
    }

    /// Get discovery engine statistics
    pub async fn get_discovery_stats(&self) -> Result<DiscoveryStats> {
        let stats = self.stats.read().await.clone();
        Ok(stats)
    }

    /// Commit all pending changes
    pub async fn commit(&self) -> Result<()> {
        self.search_engine.commit().await?;
        self.storage.flush().await?;
        info!("Discovery engine changes committed");
        Ok(())
    }

    /// Reindex all models from storage
    pub async fn reindex_all(&self) -> Result<()> {
        info!("Starting full reindex of all models");

        let models = self.storage.get_all_models().await?;

        for model in models {
            if let Err(e) = self.search_engine.index_model(&model).await {
                error!(
                    model_id = ?model.model_id,
                    error = %e,
                    "Failed to reindex model"
                );
            }
        }

        self.search_engine.commit().await?;

        info!("Full reindex completed");
        Ok(())
    }

    /// Health check for all components
    pub async fn health_check(&self) -> Result<HealthStatus> {
        let mut status = HealthStatus::default();

        // Check search engine
        match self.search_engine.get_stats().await {
            Ok((docs, segments)) => {
                status.search_engine_docs = docs;
                status.search_engine_segments = segments;
                status.search_engine_healthy = true;
            }
            Err(e) => {
                status.search_engine_healthy = false;
                status.errors.push(format!("Search engine error: {}", e));
            }
        }

        // Check metadata cache
        let (cache_total, cache_expired) = self.metadata_cache.get_cache_stats();
        status.metadata_cache_entries = cache_total;
        status.metadata_cache_expired = cache_expired;
        status.metadata_cache_healthy = true;

        // Check storage
        match self.storage.get_model_count().await {
            Ok(count) => {
                status.storage_model_count = count;
                status.storage_healthy = true;
            }
            Err(e) => {
                status.storage_healthy = false;
                status.errors.push(format!("Storage error: {}", e));
            }
        }

        // Check recommendation engine
        status.recommendations_healthy = self.recommendation_engine.is_some();

        // Overall health
        status.overall_healthy = status.search_engine_healthy
            && status.metadata_cache_healthy
            && status.storage_healthy;

        Ok(status)
    }

    // Private helper methods

    async fn enrich_search_results(&self, results: Vec<SearchResult>) -> Result<Vec<SearchResult>> {
        let mut enriched = Vec::new();

        for mut result in results {
            // Get complete model data from storage
            if let Some(complete_model) = self.storage.get_model(&result.model.model_id).await? {
                result.model = complete_model;
            }
            enriched.push(result);
        }

        Ok(enriched)
    }

    async fn enrich_models_with_storage(&self, models: Vec<MarketplaceModel>) -> Result<Vec<MarketplaceModel>> {
        let mut enriched = Vec::new();

        for model in models {
            // Get complete model data from storage
            if let Some(complete_model) = self.storage.get_model(&model.model_id).await? {
                enriched.push(complete_model);
            } else {
                // Fallback to search index data
                enriched.push(model);
            }
        }

        Ok(enriched)
    }

    /// Get the current configuration
    pub fn get_config(&self) -> &DiscoveryConfig {
        &self.config
    }

    /// Check if recommendations are enabled
    pub fn recommendations_enabled(&self) -> bool {
        self.config.enable_recommendations
    }
}

/// Health status for the discovery engine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    pub overall_healthy: bool,
    pub search_engine_healthy: bool,
    pub search_engine_docs: usize,
    pub search_engine_segments: usize,
    pub metadata_cache_healthy: bool,
    pub metadata_cache_entries: usize,
    pub metadata_cache_expired: usize,
    pub storage_healthy: bool,
    pub storage_model_count: u64,
    pub recommendations_healthy: bool,
    pub errors: Vec<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl Default for HealthStatus {
    fn default() -> Self {
        Self {
            overall_healthy: false,
            search_engine_healthy: false,
            search_engine_docs: 0,
            search_engine_segments: 0,
            metadata_cache_healthy: false,
            metadata_cache_entries: 0,
            metadata_cache_expired: 0,
            storage_healthy: false,
            storage_model_count: 0,
            recommendations_healthy: false,
            errors: Vec::new(),
            timestamp: chrono::Utc::now(),
        }
    }
}