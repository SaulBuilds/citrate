// lattice-v3/core/marketplace/src/storage_simple.rs

use crate::types::*;
use anyhow::Result;
use chrono::Utc;
use dashmap::DashMap;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};

/// Simple in-memory marketplace storage for demonstration
pub struct MarketplaceStorage {
    models: Arc<DashMap<ModelId, MarketplaceModel>>,
    interactions: Arc<RwLock<Vec<UserInteraction>>>,
    reviews: Arc<DashMap<(ModelId, Address), UserReview>>,
    stats: Arc<RwLock<MarketplaceStats>>,
}

impl MarketplaceStorage {
    /// Create a new storage instance
    pub async fn new<P: AsRef<Path>>(_db_path: P) -> Result<Self> {
        info!("Marketplace storage initialized (in-memory)");
        Ok(Self {
            models: Arc::new(DashMap::new()),
            interactions: Arc::new(RwLock::new(Vec::new())),
            reviews: Arc::new(DashMap::new()),
            stats: Arc::new(RwLock::new(MarketplaceStats::default())),
        })
    }

    /// Store a model
    pub async fn store_model(&self, model: &MarketplaceModel) -> Result<()> {
        self.models.insert(model.model_id, model.clone());
        debug!(model_id = ?model.model_id, "Model stored");
        Ok(())
    }

    /// Update a model
    pub async fn update_model(&self, model: &MarketplaceModel) -> Result<()> {
        self.models.insert(model.model_id, model.clone());
        debug!(model_id = ?model.model_id, "Model updated");
        Ok(())
    }

    /// Get a model by ID
    pub async fn get_model(&self, model_id: &ModelId) -> Result<Option<MarketplaceModel>> {
        Ok(self.models.get(model_id).map(|entry| entry.value().clone()))
    }

    /// Remove a model
    pub async fn remove_model(&self, model_id: &ModelId) -> Result<()> {
        self.models.remove(model_id);
        debug!(model_id = ?model_id, "Model removed");
        Ok(())
    }

    /// Get all models
    pub async fn get_all_models(&self) -> Result<Vec<MarketplaceModel>> {
        Ok(self.models.iter().map(|entry| entry.value().clone()).collect())
    }

    /// Get models by category
    pub async fn get_models_by_category(&self, category: ModelCategory) -> Result<Vec<MarketplaceModel>> {
        Ok(self.models
            .iter()
            .filter(|entry| entry.value().category == category)
            .map(|entry| entry.value().clone())
            .collect())
    }

    /// Get models by owner
    pub async fn get_models_by_owner(&self, owner: &Address) -> Result<Vec<MarketplaceModel>> {
        Ok(self.models
            .iter()
            .filter(|entry| entry.value().owner == *owner)
            .map(|entry| entry.value().clone())
            .collect())
    }

    /// Search models by name or description
    pub async fn search_models(&self, query: &str) -> Result<Vec<MarketplaceModel>> {
        let query_lower = query.to_lowercase();
        Ok(self.models
            .iter()
            .filter(|entry| {
                let model = entry.value();
                model.name.to_lowercase().contains(&query_lower)
                    || model.description.to_lowercase().contains(&query_lower)
            })
            .map(|entry| entry.value().clone())
            .collect())
    }

    /// Get model count
    pub async fn get_model_count(&self) -> Result<u64> {
        Ok(self.models.len() as u64)
    }

    /// Record user interaction
    pub async fn record_interaction(&self, interaction: &UserInteraction) -> Result<()> {
        let mut interactions = self.interactions.write().await;
        interactions.push(interaction.clone());
        debug!(user = ?interaction.user, model_id = ?interaction.model_id, "Interaction recorded");
        Ok(())
    }

    /// Get user interactions
    pub async fn get_user_interactions(&self, user: &Address, limit: usize) -> Result<Vec<UserInteraction>> {
        let interactions = self.interactions.read().await;
        Ok(interactions
            .iter()
            .filter(|interaction| interaction.user == *user)
            .take(limit)
            .cloned()
            .collect())
    }

    /// Get model interactions
    pub async fn get_model_interactions(&self, model_id: &ModelId, limit: usize) -> Result<Vec<UserInteraction>> {
        let interactions = self.interactions.read().await;
        Ok(interactions
            .iter()
            .filter(|interaction| interaction.model_id == *model_id)
            .take(limit)
            .cloned()
            .collect())
    }

    /// Store a review
    pub async fn store_review(&self, review: &UserReview) -> Result<()> {
        let key = (review.model_id, review.reviewer);
        self.reviews.insert(key, review.clone());
        debug!(model_id = ?review.model_id, reviewer = ?review.reviewer, "Review stored");
        Ok(())
    }

    /// Get reviews for a model
    pub async fn get_model_reviews(&self, model_id: &ModelId, limit: usize) -> Result<Vec<UserReview>> {
        Ok(self.reviews
            .iter()
            .filter(|entry| entry.key().0 == *model_id)
            .take(limit)
            .map(|entry| entry.value().clone())
            .collect())
    }

    /// Get reviews by user
    pub async fn get_user_reviews(&self, user: &Address, limit: usize) -> Result<Vec<UserReview>> {
        Ok(self.reviews
            .iter()
            .filter(|entry| entry.key().1 == *user)
            .take(limit)
            .map(|entry| entry.value().clone())
            .collect())
    }

    /// Calculate average rating for a model
    pub async fn get_model_rating(&self, model_id: &ModelId) -> Result<Option<f32>> {
        let reviews: Vec<f32> = self.reviews
            .iter()
            .filter(|entry| entry.key().0 == *model_id)
            .map(|entry| entry.value().rating)
            .collect();

        if reviews.is_empty() {
            Ok(None)
        } else {
            let average = reviews.iter().sum::<f32>() / reviews.len() as f32;
            Ok(Some(average))
        }
    }

    /// Get marketplace statistics
    pub async fn get_marketplace_stats(&self) -> Result<MarketplaceStats> {
        let mut stats = self.stats.write().await;

        // Update stats
        stats.total_models = self.models.len() as u64;
        stats.total_interactions = self.interactions.read().await.len() as u64;
        stats.total_reviews = self.reviews.len() as u64;

        // Calculate top models by interaction count
        let interactions = self.interactions.read().await;
        let mut model_interaction_counts: HashMap<ModelId, u64> = HashMap::new();

        for interaction in interactions.iter() {
            *model_interaction_counts.entry(interaction.model_id).or_insert(0) += 1;
        }

        let mut top_models: Vec<(ModelId, u64)> = model_interaction_counts.into_iter().collect();
        top_models.sort_by(|a, b| b.1.cmp(&a.1));
        stats.top_models = top_models.into_iter().take(10).map(|(id, _)| id).collect();

        // Calculate category distribution
        let mut category_counts: HashMap<ModelCategory, u64> = HashMap::new();
        for model in self.models.iter() {
            *category_counts.entry(model.value().category).or_insert(0) += 1;
        }
        stats.category_distribution = category_counts;

        stats.last_updated = Utc::now();

        Ok(stats.clone())
    }

    /// Update marketplace statistics
    pub async fn update_stats(&self, stats: &MarketplaceStats) -> Result<()> {
        let mut current_stats = self.stats.write().await;
        *current_stats = stats.clone();
        Ok(())
    }

    /// Flush any pending changes (no-op for in-memory storage)
    pub async fn flush(&self) -> Result<()> {
        debug!("Storage flush (no-op for in-memory)");
        Ok(())
    }

    /// Close the storage (no-op for in-memory storage)
    pub async fn close(&self) -> Result<()> {
        debug!("Storage close (no-op for in-memory)");
        Ok(())
    }

    /// Health check
    pub async fn health_check(&self) -> Result<bool> {
        Ok(true)
    }

    /// Clear all data (for testing)
    pub async fn clear_all(&self) -> Result<()> {
        self.models.clear();
        self.interactions.write().await.clear();
        self.reviews.clear();
        *self.stats.write().await = MarketplaceStats::default();
        info!("Storage cleared");
        Ok(())
    }
}