// lattice-v3/core/marketplace/src/search_simple.rs

use crate::types::*;
use anyhow::Result;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::Path;
use std::sync::Arc;
use tracing::{debug, info};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchQuery {
    pub text: String,
    pub category: Option<ModelCategory>,
    pub framework: Option<String>,
    pub tags: Vec<String>,
    pub min_price: Option<u64>,
    pub max_price: Option<u64>,
    pub sort_by: Option<SortOrder>,
    pub limit: usize,
    pub offset: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SortOrder {
    Relevance,
    Price,
    Rating,
    Recent,
    Downloads,
}

impl Default for SearchQuery {
    fn default() -> Self {
        Self {
            text: String::new(),
            category: None,
            framework: None,
            tags: Vec::new(),
            min_price: None,
            max_price: None,
            sort_by: Some(SortOrder::Relevance),
            limit: 20,
            offset: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub model: MarketplaceModel,
    pub score: f32,
    pub snippet: String,
}

/// Simple in-memory search engine
pub struct SearchEngine {
    models: Arc<DashMap<ModelId, MarketplaceModel>>,
    text_index: Arc<DashMap<String, HashSet<ModelId>>>,
}

impl SearchEngine {
    /// Create a new search engine
    pub async fn new<P: AsRef<Path>>(_index_path: P) -> Result<Self> {
        info!("Search engine initialized (in-memory)");
        Ok(Self {
            models: Arc::new(DashMap::new()),
            text_index: Arc::new(DashMap::new()),
        })
    }

    /// Index a model
    pub async fn index_model(&self, model: &MarketplaceModel) -> Result<()> {
        // Store the model
        self.models.insert(model.model_id, model.clone());

        // Build text index
        let text_content = format!(
            "{} {} {} {} {}",
            model.name,
            model.description,
            model.framework,
            model.license,
            model.tags.join(" ")
        );

        // Simple tokenization
        let tokens = self.tokenize(&text_content);

        for token in tokens {
            self.text_index
                .entry(token)
                .or_insert_with(HashSet::new)
                .insert(model.model_id);
        }

        debug!(model_id = ?model.model_id, "Model indexed");
        Ok(())
    }

    /// Remove a model from the index
    pub async fn remove_model(&self, model_id: &ModelId) -> Result<()> {
        // Remove from models
        if let Some((_, model)) = self.models.remove(model_id) {
            // Remove from text index
            let text_content = format!(
                "{} {} {} {} {}",
                model.name,
                model.description,
                model.framework,
                model.license,
                model.tags.join(" ")
            );

            let tokens = self.tokenize(&text_content);
            for token in tokens {
                if let Some(mut entry) = self.text_index.get_mut(&token) {
                    entry.remove(model_id);
                    if entry.is_empty() {
                        drop(entry);
                        self.text_index.remove(&token);
                    }
                }
            }
        }

        debug!(model_id = ?model_id, "Model removed from index");
        Ok(())
    }

    /// Search for models
    pub async fn search(&self, query: &SearchQuery) -> Result<Vec<SearchResult>> {
        let mut candidates = HashSet::new();

        // Text search
        if !query.text.trim().is_empty() {
            let tokens = self.tokenize(&query.text);
            for token in tokens {
                if let Some(model_ids) = self.text_index.get(&token) {
                    candidates.extend(model_ids.iter());
                }
            }
        } else {
            // If no text query, include all models
            candidates.extend(self.models.iter().map(|entry| *entry.key()));
        }

        // Apply filters and collect results
        let mut results = Vec::new();

        for model_id in candidates {
            if let Some(model) = self.models.get(&model_id) {
                let model = model.value();

                // Filter by category
                if let Some(category) = &query.category {
                    if model.category != *category {
                        continue;
                    }
                }

                // Filter by framework
                if let Some(framework) = &query.framework {
                    if model.framework != *framework {
                        continue;
                    }
                }

                // Filter by tags
                if !query.tags.is_empty() {
                    let has_any_tag = query.tags.iter().any(|tag| model.tags.contains(tag));
                    if !has_any_tag {
                        continue;
                    }
                }

                // Filter by price
                if let Some(min_price) = query.min_price {
                    if model.base_price < min_price {
                        continue;
                    }
                }

                if let Some(max_price) = query.max_price {
                    if model.base_price > max_price {
                        continue;
                    }
                }

                // Calculate relevance score
                let score = self.calculate_score(model, query);

                // Create snippet
                let snippet = if query.text.trim().is_empty() {
                    model.description.chars().take(200).collect()
                } else {
                    self.create_snippet(&model.description, &query.text)
                };

                results.push(SearchResult {
                    model: model.clone(),
                    score,
                    snippet,
                });
            }
        }

        // Sort results
        self.sort_results(&mut results, query);

        // Apply pagination
        let start = query.offset;
        let end = (start + query.limit).min(results.len());

        Ok(results[start..end].to_vec())
    }

    /// Get trending models (most interacted with)
    pub async fn get_trending_models(&self, limit: usize) -> Result<Vec<MarketplaceModel>> {
        // For simplicity, just return models sorted by name for now
        let mut models: Vec<MarketplaceModel> = self.models
            .iter()
            .map(|entry| entry.value().clone())
            .collect();

        models.sort_by(|a, b| a.name.cmp(&b.name));
        models.truncate(limit);

        Ok(models)
    }

    /// Get similar models based on tags and category
    pub async fn get_similar_models(&self, model_id: &ModelId, limit: usize) -> Result<Vec<MarketplaceModel>> {
        let target_model = match self.models.get(model_id) {
            Some(model) => model.value().clone(),
            None => return Ok(Vec::new()),
        };

        let mut scores = Vec::new();

        for entry in self.models.iter() {
            let model = entry.value();
            if model.model_id == *model_id {
                continue;
            }

            let score = self.calculate_similarity(&target_model, model);
            if score > 0.1 {
                scores.push((model.clone(), score));
            }
        }

        // Sort by similarity score
        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scores.truncate(limit);

        Ok(scores.into_iter().map(|(model, _)| model).collect())
    }

    /// Get search statistics
    pub async fn get_stats(&self) -> Result<(usize, usize)> {
        let docs = self.models.len();
        let segments = 1; // Simple in-memory has just one "segment"
        Ok((docs, segments))
    }

    /// Commit changes (no-op for in-memory)
    pub async fn commit(&self) -> Result<()> {
        debug!("Search index commit (no-op for in-memory)");
        Ok(())
    }

    /// Optimize index (no-op for in-memory)
    pub async fn optimize(&self) -> Result<()> {
        debug!("Search index optimization (no-op for in-memory)");
        Ok(())
    }

    // Private helper methods

    fn tokenize(&self, text: &str) -> Vec<String> {
        text.to_lowercase()
            .split_whitespace()
            .filter(|token| token.len() > 2)
            .map(|token| {
                // Remove punctuation
                token.chars()
                    .filter(|c| c.is_alphanumeric())
                    .collect::<String>()
            })
            .filter(|token: &String| !token.is_empty())
            .collect()
    }

    fn calculate_score(&self, model: &MarketplaceModel, query: &SearchQuery) -> f32 {
        let mut score = 0.0;

        if !query.text.trim().is_empty() {
            let query_tokens = self.tokenize(&query.text);
            let model_text = format!(
                "{} {} {} {}",
                model.name, model.description, model.framework, model.tags.join(" ")
            );
            let model_tokens = self.tokenize(&model_text);

            // Simple TF scoring
            for query_token in &query_tokens {
                let count = model_tokens.iter().filter(|&token| token == query_token).count();
                score += count as f32;
            }

            // Boost for exact name matches
            if model.name.to_lowercase().contains(&query.text.to_lowercase()) {
                score += 10.0;
            }
        } else {
            score = 1.0; // Base score when no text query
        }

        // Category match boost
        if let Some(category) = &query.category {
            if model.category == *category {
                score += 5.0;
            }
        }

        // Framework match boost
        if let Some(framework) = &query.framework {
            if model.framework == *framework {
                score += 3.0;
            }
        }

        // Tag match boost
        for tag in &query.tags {
            if model.tags.contains(tag) {
                score += 2.0;
            }
        }

        score
    }

    fn calculate_similarity(&self, model1: &MarketplaceModel, model2: &MarketplaceModel) -> f32 {
        let mut similarity = 0.0;

        // Category similarity
        if model1.category == model2.category {
            similarity += 0.4;
        }

        // Framework similarity
        if model1.framework == model2.framework {
            similarity += 0.3;
        }

        // Tag similarity (Jaccard index)
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

    fn create_snippet(&self, text: &str, query: &str) -> String {
        let query_lower = query.to_lowercase();
        let text_lower = text.to_lowercase();

        if let Some(pos) = text_lower.find(&query_lower) {
            let start = pos.saturating_sub(50);
            let end = (pos + query.len() + 50).min(text.len());

            let mut snippet = text[start..end].to_string();
            if start > 0 {
                snippet = format!("...{}", snippet);
            }
            if end < text.len() {
                snippet = format!("{}...", snippet);
            }

            snippet
        } else {
            text.chars().take(200).collect()
        }
    }

    fn sort_results(&self, results: &mut [SearchResult], query: &SearchQuery) {
        match query.sort_by.as_ref().unwrap_or(&SortOrder::Relevance) {
            SortOrder::Relevance => {
                results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
            }
            SortOrder::Price => {
                results.sort_by(|a, b| a.model.base_price.cmp(&b.model.base_price));
            }
            SortOrder::Rating => {
                // For now, sort by model ID as a proxy
                results.sort_by(|a, b| a.model.model_id.cmp(&b.model.model_id));
            }
            SortOrder::Recent => {
                results.sort_by(|a, b| b.model.created_at.cmp(&a.model.created_at));
            }
            SortOrder::Downloads => {
                // For now, sort by model ID as a proxy
                results.sort_by(|a, b| a.model.model_id.cmp(&b.model.model_id));
            }
        }
    }
}