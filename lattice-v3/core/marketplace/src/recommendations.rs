// lattice-v3/core/marketplace/src/recommendations.rs

use crate::{storage::MarketplaceStorage, types::*};
use anyhow::Result;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tracing::{debug, info};

/// Simple collaborative filtering recommendation engine
pub struct RecommendationEngine {
    storage: Arc<MarketplaceStorage>,
    user_profiles: HashMap<Address, UserProfile>,
    model_similarities: HashMap<ModelId, Vec<(ModelId, f32)>>,
}

#[derive(Debug, Clone)]
struct UserProfile {
    preferences: ModelCategory,
    purchased_models: HashSet<ModelId>,
    viewed_models: HashSet<ModelId>,
    avg_rating_given: f32,
    preferred_frameworks: HashMap<String, u32>,
}

impl RecommendationEngine {
    /// Create a new recommendation engine
    pub async fn new(storage: Arc<MarketplaceStorage>) -> Result<Self> {
        let mut engine = Self {
            storage,
            user_profiles: HashMap::new(),
            model_similarities: HashMap::new(),
        };

        // Initialize with existing data
        engine.build_initial_profiles().await?;
        engine.compute_model_similarities().await?;

        info!("Recommendation engine initialized");
        Ok(engine)
    }

    /// Get personalized recommendations for a user
    pub async fn get_recommendations(&self, user_address: &Address, limit: usize) -> Result<Vec<ModelId>> {
        // Get user profile
        let recommendations = if let Some(profile) = self.user_profiles.get(user_address) {
            // Collaborative filtering: find similar users
            let similar_users = self.find_similar_users(user_address, 10).await?;

            // Get models purchased by similar users but not by this user
            let mut candidate_models = HashMap::new();
            for (similar_user, similarity) in similar_users {
                if let Some(similar_profile) = self.user_profiles.get(&similar_user) {
                    for model_id in &similar_profile.purchased_models {
                        if !profile.purchased_models.contains(model_id)
                            && !profile.viewed_models.contains(model_id) {
                            *candidate_models.entry(*model_id).or_insert(0.0) += similarity;
                        }
                    }
                }
            }

            // Content-based filtering: find models similar to purchased ones
            for purchased_model in &profile.purchased_models {
                if let Some(similar_models) = self.model_similarities.get(purchased_model) {
                    for (model_id, similarity) in similar_models {
                        if !profile.purchased_models.contains(model_id)
                            && !profile.viewed_models.contains(model_id) {
                            *candidate_models.entry(*model_id).or_insert(0.0) += similarity * 0.7; // Weight content-based lower
                        }
                    }
                }
            }

            // Category preference boost
            let models = self.storage.get_all_models().await?;
            for model in models {
                if model.category == profile.preferences
                    && !profile.purchased_models.contains(&model.model_id)
                    && !profile.viewed_models.contains(&model.model_id)
                    && model.active {
                    *candidate_models.entry(model.model_id).or_insert(0.0) += 0.3;
                }
            }

            // Sort by score and return top recommendations
            let mut scored_models: Vec<(ModelId, f32)> = candidate_models.into_iter().collect();
            scored_models.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

            scored_models
                .into_iter()
                .take(limit)
                .map(|(model_id, _)| model_id)
                .collect()
        } else {
            // New user: recommend popular models
            self.get_popular_models(limit).await?
        };

        debug!(
            user = ?user_address,
            recommendations_count = recommendations.len(),
            "Generated recommendations"
        );

        Ok(recommendations)
    }

    /// Update user profile based on new interactions
    pub async fn update_user_profile(&self, _user_address: &Address) -> Result<()> {
        // For now, we'll rebuild profiles periodically
        // In a production system, this would incrementally update the profile
        Ok(())
    }

    /// Get trending models for cold start
    async fn get_popular_models(&self, limit: usize) -> Result<Vec<ModelId>> {
        let stats = self.storage.get_marketplace_stats().await?;
        Ok(stats.top_models.into_iter().take(limit).collect())
    }

    /// Find users with similar preferences
    async fn find_similar_users(&self, user_address: &Address, limit: usize) -> Result<Vec<(Address, f32)>> {
        let target_profile = match self.user_profiles.get(user_address) {
            Some(profile) => profile,
            None => return Ok(Vec::new()),
        };

        let mut similarities = Vec::new();

        for (other_user, other_profile) in &self.user_profiles {
            if other_user != user_address {
                let similarity = self.calculate_user_similarity(target_profile, other_profile);
                if similarity > 0.1 { // Minimum similarity threshold
                    similarities.push((*other_user, similarity));
                }
            }
        }

        // Sort by similarity and take top N
        similarities.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        similarities.truncate(limit);

        Ok(similarities)
    }

    /// Calculate similarity between two users
    fn calculate_user_similarity(&self, profile1: &UserProfile, profile2: &UserProfile) -> f32 {
        let mut similarity = 0.0;

        // Category preference similarity
        if profile1.preferences == profile2.preferences {
            similarity += 0.3;
        }

        // Jaccard similarity for purchased models
        let intersection: HashSet<_> = profile1.purchased_models
            .intersection(&profile2.purchased_models)
            .collect();
        let union: HashSet<_> = profile1.purchased_models
            .union(&profile2.purchased_models)
            .collect();

        if !union.is_empty() {
            let jaccard = intersection.len() as f32 / union.len() as f32;
            similarity += jaccard * 0.4;
        }

        // Framework preference similarity
        let common_frameworks: f32 = profile1.preferred_frameworks
            .iter()
            .map(|(framework, count1)| {
                let count2 = profile2.preferred_frameworks.get(framework).unwrap_or(&0);
                (*count1.min(count2)) as f32
            })
            .sum();

        let total_frameworks: f32 = profile1.preferred_frameworks.values().sum::<u32>() as f32
            + profile2.preferred_frameworks.values().sum::<u32>() as f32;

        if total_frameworks > 0.0 {
            similarity += (common_frameworks / total_frameworks) * 0.2;
        }

        // Rating pattern similarity (how similarly they rate models)
        let rating_diff = (profile1.avg_rating_given - profile2.avg_rating_given).abs();
        let rating_similarity = (5.0 - rating_diff) / 5.0; // Normalize to 0-1
        similarity += rating_similarity * 0.1;

        similarity.min(1.0)
    }

    /// Build initial user profiles from interaction history
    async fn build_initial_profiles(&mut self) -> Result<()> {
        // This is a simplified implementation
        // In production, you'd want to process interaction data more efficiently

        let models = self.storage.get_all_models().await?;

        // Create a map for quick model lookup
        let model_map: HashMap<ModelId, &MarketplaceModel> = models
            .iter()
            .map(|m| (m.model_id, m))
            .collect();

        // For each model owner, create a basic profile
        for model in &models {
            if !self.user_profiles.contains_key(&model.owner) {
                let interactions = self.storage.get_user_interactions(&model.owner, 100).await?;

                let mut purchased_models = HashSet::new();
                let mut viewed_models = HashSet::new();
                let mut framework_prefs = HashMap::new();
                let mut category_counts = HashMap::new();

                for interaction in interactions {
                    match interaction.interaction_type {
                        InteractionType::Purchase => {
                            purchased_models.insert(interaction.model_id);
                        }
                        InteractionType::View => {
                            viewed_models.insert(interaction.model_id);
                        }
                        _ => {}
                    }

                    // Count category preferences
                    if let Some(model) = model_map.get(&interaction.model_id) {
                        *framework_prefs.entry(model.framework.clone()).or_insert(0) += 1;
                        *category_counts.entry(model.category).or_insert(0) += 1;
                    }
                }

                // Determine preferred category
                let preferences = category_counts
                    .into_iter()
                    .max_by_key(|(_, count)| *count)
                    .map(|(category, _)| category)
                    .unwrap_or(ModelCategory::Other);

                let profile = UserProfile {
                    preferences,
                    purchased_models,
                    viewed_models,
                    avg_rating_given: 4.0, // Default
                    preferred_frameworks: framework_prefs,
                };

                self.user_profiles.insert(model.owner, profile);
            }
        }

        info!("Built {} user profiles", self.user_profiles.len());
        Ok(())
    }

    /// Compute model-to-model similarities
    async fn compute_model_similarities(&mut self) -> Result<()> {
        let models = self.storage.get_all_models().await?;

        for model1 in &models {
            let mut similarities = Vec::new();

            for model2 in &models {
                if model1.model_id != model2.model_id {
                    let similarity = self.calculate_model_similarity(model1, model2);
                    if similarity > 0.1 {
                        similarities.push((model2.model_id, similarity));
                    }
                }
            }

            // Sort by similarity and keep top 20
            similarities.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
            similarities.truncate(20);

            self.model_similarities.insert(model1.model_id, similarities);
        }

        info!("Computed similarities for {} models", models.len());
        Ok(())
    }

    /// Calculate similarity between two models
    fn calculate_model_similarity(&self, model1: &MarketplaceModel, model2: &MarketplaceModel) -> f32 {
        let mut similarity = 0.0;

        // Category similarity
        if model1.category == model2.category {
            similarity += 0.4;
        }

        // Framework similarity
        if model1.framework == model2.framework {
            similarity += 0.3;
        }

        // Tag similarity (Jaccard)
        let tags1: HashSet<_> = model1.tags.iter().collect();
        let tags2: HashSet<_> = model2.tags.iter().collect();

        let intersection = tags1.intersection(&tags2).count();
        let union = tags1.union(&tags2).count();

        if union > 0 {
            similarity += (intersection as f32 / union as f32) * 0.2;
        }

        // License similarity
        if model1.license == model2.license {
            similarity += 0.1;
        }

        similarity.min(1.0)
    }
}