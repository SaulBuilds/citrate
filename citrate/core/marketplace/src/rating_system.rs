// citrate/core/marketplace/src/rating_system.rs

use crate::types::*;
use anyhow::Result;
use chrono::{DateTime, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tracing::{debug, info, warn};

/// Rating system configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RatingConfig {
    pub min_rating: f32,
    pub max_rating: f32,
    pub required_reviews_for_stability: usize,
    pub review_weight_decay_days: u64,
    pub enable_sentiment_analysis: bool,
    pub spam_detection_threshold: f32,
}

impl Default for RatingConfig {
    fn default() -> Self {
        Self {
            min_rating: 1.0,
            max_rating: 5.0,
            required_reviews_for_stability: 10,
            review_weight_decay_days: 365,
            enable_sentiment_analysis: true,
            spam_detection_threshold: 0.3,
        }
    }
}

/// Aggregated rating statistics for a model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelRating {
    pub model_id: ModelId,
    pub average_rating: f32,
    pub weighted_rating: f32, // Weighted by review age and quality
    pub total_reviews: u64,
    pub rating_distribution: [u64; 5], // Count for each star rating (1-5)
    pub confidence_score: f32, // Based on number of reviews and consistency
    pub sentiment_score: f32, // Derived from review content analysis
    pub last_updated: DateTime<Utc>,
}

/// Review quality metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewQuality {
    pub helpfulness_score: f32, // Based on user votes
    pub detail_score: f32, // Based on review length and detail
    pub verified_purchase: bool,
    pub reviewer_credibility: f32, // Based on reviewer history
    pub spam_probability: f32,
}

/// Extended user review with quality metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedUserReview {
    pub review: UserReview,
    pub quality: ReviewQuality,
    pub helpful_votes: u64,
    pub total_votes: u64,
    pub reported_count: u64,
    pub sentiment_analysis: Option<SentimentAnalysis>,
}

/// Sentiment analysis results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SentimentAnalysis {
    pub overall_sentiment: f32, // -1.0 (negative) to 1.0 (positive)
    pub confidence: f32,
    pub emotion_scores: HashMap<String, f32>, // joy, anger, sadness, etc.
    pub key_phrases: Vec<String>,
}

/// Performance metrics for a model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelPerformanceMetrics {
    pub model_id: ModelId,
    pub accuracy_scores: Vec<f32>,
    pub latency_ms: Vec<u64>,
    pub throughput_rps: Vec<f32>,
    pub error_rate: f32,
    pub uptime_percentage: f32,
    pub cost_per_inference: f32,
    pub benchmark_results: HashMap<String, f32>,
    pub hardware_efficiency: f32,
    pub last_measured: DateTime<Utc>,
}

/// Aggregated model statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelStats {
    pub model_id: ModelId,
    pub total_inferences: u64,
    pub unique_users: u64,
    pub revenue_generated: u64, // in wei
    pub avg_session_duration: f32, // in seconds
    pub user_retention_rate: f32,
    pub conversion_rate: f32, // trial to paid
    pub popularity_score: f32,
    pub trending_score: f32,
    pub last_updated: DateTime<Utc>,
}

/// Main rating and review system
pub struct RatingSystem {
    config: RatingConfig,
    model_ratings: Arc<DashMap<ModelId, ModelRating>>,
    enhanced_reviews: Arc<DashMap<(ModelId, Address), EnhancedUserReview>>,
    performance_metrics: Arc<DashMap<ModelId, ModelPerformanceMetrics>>,
    model_stats: Arc<DashMap<ModelId, ModelStats>>,
    reviewer_profiles: Arc<DashMap<Address, ReviewerProfile>>,
}

/// Reviewer profile and credibility tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewerProfile {
    pub reviewer: Address,
    pub total_reviews: u64,
    pub helpful_reviews: u64,
    pub credibility_score: f32,
    pub expertise_areas: HashMap<ModelCategory, f32>,
    pub review_quality_history: VecDeque<f32>,
    pub spam_reports: u64,
    pub verified_purchases: u64,
    pub created_at: DateTime<Utc>,
    pub last_active: DateTime<Utc>,
}

impl RatingSystem {
    /// Create a new rating system
    pub fn new(config: RatingConfig) -> Self {
        Self {
            config,
            model_ratings: Arc::new(DashMap::new()),
            enhanced_reviews: Arc::new(DashMap::new()),
            performance_metrics: Arc::new(DashMap::new()),
            model_stats: Arc::new(DashMap::new()),
            reviewer_profiles: Arc::new(DashMap::new()),
        }
    }

    /// Submit a new review
    pub async fn submit_review(&self, review: UserReview, verified_purchase: bool) -> Result<()> {
        // Create enhanced review with initial quality metrics
        let quality = self.calculate_review_quality(&review, verified_purchase).await?;
        let enhanced_review = EnhancedUserReview {
            review: review.clone(),
            quality,
            helpful_votes: 0,
            total_votes: 0,
            reported_count: 0,
            sentiment_analysis: if self.config.enable_sentiment_analysis {
                Some(self.analyze_sentiment(&review.content).await?)
            } else {
                None
            },
        };

        // Store the enhanced review
        let key = (review.model_id, review.reviewer);
        self.enhanced_reviews.insert(key, enhanced_review);

        // Update reviewer profile
        self.update_reviewer_profile(&review.reviewer, &review, verified_purchase).await?;

        // Recalculate model rating
        self.recalculate_model_rating(&review.model_id).await?;

        info!(
            model_id = ?review.model_id,
            reviewer = ?review.reviewer,
            rating = review.rating,
            "Review submitted"
        );

        Ok(())
    }

    /// Vote on review helpfulness
    pub async fn vote_on_review(&self, model_id: &ModelId, reviewer: &Address, helpful: bool) -> Result<()> {
        let key = (*model_id, *reviewer);
        if let Some(mut review_entry) = self.enhanced_reviews.get_mut(&key) {
            if helpful {
                review_entry.helpful_votes += 1;
            }
            review_entry.total_votes += 1;

            // Update quality score based on new votes
            review_entry.quality.helpfulness_score =
                review_entry.helpful_votes as f32 / review_entry.total_votes.max(1) as f32;

            debug!(
                model_id = ?model_id,
                reviewer = ?reviewer,
                helpful = helpful,
                "Review vote recorded"
            );
        }

        Ok(())
    }

    /// Report a review as spam
    pub async fn report_review(&self, model_id: &ModelId, reviewer: &Address) -> Result<()> {
        let key = (*model_id, *reviewer);
        if let Some(mut review_entry) = self.enhanced_reviews.get_mut(&key) {
            review_entry.reported_count += 1;

            // Update spam probability
            review_entry.quality.spam_probability =
                (review_entry.reported_count as f32 / review_entry.total_votes.max(1) as f32).min(1.0);

            // If spam probability exceeds threshold, mark as low quality
            if review_entry.quality.spam_probability > self.config.spam_detection_threshold {
                warn!(
                    model_id = ?model_id,
                    reviewer = ?reviewer,
                    spam_probability = review_entry.quality.spam_probability,
                    "Review flagged as potential spam"
                );
            }
        }

        Ok(())
    }

    /// Get model rating with full statistics
    pub async fn get_model_rating(&self, model_id: &ModelId) -> Option<ModelRating> {
        self.model_ratings.get(model_id).map(|entry| entry.value().clone())
    }

    /// Get enhanced reviews for a model
    pub async fn get_model_reviews(&self, model_id: &ModelId, limit: usize, sort_by: ReviewSortOrder) -> Vec<EnhancedUserReview> {
        let mut reviews: Vec<EnhancedUserReview> = self.enhanced_reviews
            .iter()
            .filter(|entry| entry.key().0 == *model_id)
            .map(|entry| entry.value().clone())
            .collect();

        // Sort reviews
        match sort_by {
            ReviewSortOrder::MostHelpful => {
                reviews.sort_by(|a, b| b.quality.helpfulness_score.partial_cmp(&a.quality.helpfulness_score).unwrap_or(std::cmp::Ordering::Equal));
            }
            ReviewSortOrder::Newest => {
                reviews.sort_by(|a, b| b.review.created_at.cmp(&a.review.created_at));
            }
            ReviewSortOrder::Oldest => {
                reviews.sort_by(|a, b| a.review.created_at.cmp(&b.review.created_at));
            }
            ReviewSortOrder::HighestRating => {
                reviews.sort_by(|a, b| b.review.rating.partial_cmp(&a.review.rating).unwrap_or(std::cmp::Ordering::Equal));
            }
            ReviewSortOrder::LowestRating => {
                reviews.sort_by(|a, b| a.review.rating.partial_cmp(&b.review.rating).unwrap_or(std::cmp::Ordering::Equal));
            }
        }

        reviews.truncate(limit);
        reviews
    }

    /// Record performance metrics for a model
    pub async fn record_performance_metrics(&self, model_id: &ModelId, metrics: ModelPerformanceMetrics) -> Result<()> {
        debug!(
            model_id = ?model_id,
            accuracy = ?metrics.accuracy_scores.last(),
            latency = ?metrics.latency_ms.last(),
            "Performance metrics recorded"
        );

        self.performance_metrics.insert(*model_id, metrics);

        Ok(())
    }

    /// Get performance metrics for a model
    pub async fn get_performance_metrics(&self, model_id: &ModelId) -> Option<ModelPerformanceMetrics> {
        self.performance_metrics.get(model_id).map(|entry| entry.value().clone())
    }

    /// Update model usage statistics
    pub async fn update_model_stats(&self, model_id: &ModelId, stats: ModelStats) -> Result<()> {
        debug!(
            model_id = ?model_id,
            total_inferences = stats.total_inferences,
            unique_users = stats.unique_users,
            "Model stats updated"
        );

        self.model_stats.insert(*model_id, stats);

        Ok(())
    }

    /// Get comprehensive model statistics
    pub async fn get_model_stats(&self, model_id: &ModelId) -> Option<ModelStats> {
        self.model_stats.get(model_id).map(|entry| entry.value().clone())
    }

    /// Get reviewer profile
    pub async fn get_reviewer_profile(&self, reviewer: &Address) -> Option<ReviewerProfile> {
        self.reviewer_profiles.get(reviewer).map(|entry| entry.value().clone())
    }

    /// Get top rated models
    pub async fn get_top_rated_models(&self, limit: usize, min_reviews: usize) -> Vec<ModelRating> {
        let mut ratings: Vec<ModelRating> = self.model_ratings
            .iter()
            .filter(|entry| entry.value().total_reviews >= min_reviews as u64)
            .map(|entry| entry.value().clone())
            .collect();

        ratings.sort_by(|a, b| b.weighted_rating.partial_cmp(&a.weighted_rating).unwrap_or(std::cmp::Ordering::Equal));
        ratings.truncate(limit);
        ratings
    }

    /// Get trending models based on recent activity and ratings
    pub async fn get_trending_models(&self, limit: usize) -> Vec<ModelId> {
        let mut models: Vec<(ModelId, f32)> = self.model_stats
            .iter()
            .map(|entry| {
                let model_id = *entry.key();
                let stats = entry.value();

                // Calculate trending score based on recent activity and rating
                let rating_score = self.model_ratings.get(&model_id)
                    .map(|r| r.weighted_rating)
                    .unwrap_or(0.0);

                let trending_score = stats.trending_score * 0.6 + rating_score * 0.4;
                (model_id, trending_score)
            })
            .collect();

        models.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        models.truncate(limit);
        models.into_iter().map(|(id, _)| id).collect()
    }

    // Private helper methods

    async fn calculate_review_quality(&self, review: &UserReview, verified_purchase: bool) -> Result<ReviewQuality> {
        // Calculate detail score based on review content
        let detail_score = self.calculate_detail_score(review);

        // Get reviewer credibility
        let reviewer_credibility = self.reviewer_profiles
            .get(&review.reviewer)
            .map(|profile| profile.credibility_score)
            .unwrap_or(0.5); // Default neutral credibility

        Ok(ReviewQuality {
            helpfulness_score: 0.0, // Will be updated based on votes
            detail_score,
            verified_purchase,
            reviewer_credibility,
            spam_probability: 0.0, // Will be updated based on reports
        })
    }

    fn calculate_detail_score(&self, review: &UserReview) -> f32 {
        let mut score = 0.0;

        // Length score (up to 0.3)
        let content_length = review.content.len();
        score += (content_length as f32 / 500.0).min(0.3);

        // Pros and cons (up to 0.3)
        if !review.pros.is_empty() || !review.cons.is_empty() {
            score += 0.15;
            if !review.pros.is_empty() && !review.cons.is_empty() {
                score += 0.15;
            }
        }

        // Title quality (up to 0.2)
        if !review.title.trim().is_empty() && review.title.len() > 10 {
            score += 0.2;
        }

        // Specific feedback indicators (up to 0.2)
        let specific_terms = ["performance", "accuracy", "speed", "quality", "usability", "documentation"];
        let specific_count = specific_terms.iter()
            .filter(|term| review.content.to_lowercase().contains(*term))
            .count();
        score += (specific_count as f32 / specific_terms.len() as f32) * 0.2;

        score.min(1.0)
    }

    async fn analyze_sentiment(&self, content: &str) -> Result<SentimentAnalysis> {
        // Simple sentiment analysis (in production, use ML models)
        let positive_words = ["good", "great", "excellent", "amazing", "love", "perfect", "outstanding"];
        let negative_words = ["bad", "terrible", "awful", "hate", "horrible", "worst", "disappointing"];

        let content_lower = content.to_lowercase();
        let positive_count = positive_words.iter()
            .filter(|word| content_lower.contains(*word))
            .count() as f32;
        let negative_count = negative_words.iter()
            .filter(|word| content_lower.contains(*word))
            .count() as f32;

        let total_sentiment_words = positive_count + negative_count;
        let sentiment = if total_sentiment_words > 0.0 {
            (positive_count - negative_count) / total_sentiment_words
        } else {
            0.0
        };

        Ok(SentimentAnalysis {
            overall_sentiment: sentiment,
            confidence: (total_sentiment_words / 10.0).min(1.0),
            emotion_scores: HashMap::new(), // Simplified for now
            key_phrases: Vec::new(), // Simplified for now
        })
    }

    async fn update_reviewer_profile(&self, reviewer: &Address, review: &UserReview, verified_purchase: bool) -> Result<()> {
        let mut profile = self.reviewer_profiles
            .entry(*reviewer)
            .or_insert_with(|| ReviewerProfile {
                reviewer: *reviewer,
                total_reviews: 0,
                helpful_reviews: 0,
                credibility_score: 0.5,
                expertise_areas: HashMap::new(),
                review_quality_history: VecDeque::new(),
                spam_reports: 0,
                verified_purchases: 0,
                created_at: Utc::now(),
                last_active: Utc::now(),
            });

        profile.total_reviews += 1;
        profile.last_active = Utc::now();

        if verified_purchase {
            profile.verified_purchases += 1;
        }

        // Update expertise areas based on review tags
        // Extract expertise areas from review content using keyword analysis
        let content_lower = review.content.to_lowercase();

        // Map keywords to model categories
        let category_keywords = [
            (ModelCategory::LanguageModel, vec!["language", "text", "nlp", "gpt", "llm", "chat", "conversation"]),
            (ModelCategory::ImageGeneration, vec!["image", "generate", "art", "diffusion", "dall", "midjourney"]),
            (ModelCategory::ImageClassification, vec!["classify", "recognition", "detection", "vision", "cnn"]),
            (ModelCategory::AudioProcessing, vec!["audio", "sound", "music", "voice", "speech"]),
            (ModelCategory::VideoProcessing, vec!["video", "movie", "clip", "frame", "motion"]),
            (ModelCategory::Embedding, vec!["embedding", "vector", "similarity", "semantic"]),
            (ModelCategory::ObjectDetection, vec!["detect", "object", "yolo", "bbox", "localization"]),
            (ModelCategory::TextToSpeech, vec!["tts", "synthesis", "voice", "speak"]),
            (ModelCategory::SpeechToText, vec!["stt", "transcription", "whisper", "asr"]),
            (ModelCategory::Translation, vec!["translate", "translation", "language", "multilingual"]),
        ];

        for (category, keywords) in &category_keywords {
            let matches = keywords.iter().any(|keyword| content_lower.contains(keyword));
            if matches {
                *profile.expertise_areas.entry(*category).or_insert(0.0) += 1.0;
            }
        }

        // Add review quality metrics
        let review_quality = calculate_review_quality(review);
        profile.review_quality_history.push_back(review_quality);
        if profile.review_quality_history.len() > 100 {
            profile.review_quality_history.pop_front();
        }

        // Update credibility score
        let verification_bonus = if verified_purchase { 0.1 } else { 0.0 };
        let quality_bonus = review_quality * 0.2;
        let base_credibility = 0.5 + verification_bonus + quality_bonus;

        if profile.total_reviews == 1 {
            profile.credibility_score = base_credibility;
        } else {
            // Weighted average with existing credibility
            profile.credibility_score = (profile.credibility_score * 0.9) + (base_credibility * 0.1);
        }

        Ok(())
    }

    async fn recalculate_model_rating(&self, model_id: &ModelId) -> Result<()> {
        let reviews: Vec<EnhancedUserReview> = self.enhanced_reviews
            .iter()
            .filter(|entry| entry.key().0 == *model_id)
            .map(|entry| entry.value().clone())
            .collect();

        if reviews.is_empty() {
            return Ok(());
        }

        // Calculate basic statistics
        let total_reviews = reviews.len() as u64;
        let sum_ratings: f32 = reviews.iter().map(|r| r.review.rating).sum();
        let average_rating = sum_ratings / reviews.len() as f32;

        // Calculate weighted rating considering review quality and age
        let mut weighted_sum = 0.0;
        let mut weight_sum = 0.0;

        for review in &reviews {
            let age_days = (Utc::now() - review.review.created_at).num_days() as f32;
            let age_weight = (-age_days / self.config.review_weight_decay_days as f32).exp();

            let quality_weight = (review.quality.helpfulness_score * 0.3) +
                               (review.quality.detail_score * 0.3) +
                               (review.quality.reviewer_credibility * 0.2) +
                               (if review.quality.verified_purchase { 0.2 } else { 0.0 });

            let spam_penalty = 1.0 - review.quality.spam_probability;
            let final_weight = age_weight * quality_weight * spam_penalty;

            weighted_sum += review.review.rating * final_weight;
            weight_sum += final_weight;
        }

        let weighted_rating = if weight_sum > 0.0 {
            weighted_sum / weight_sum
        } else {
            average_rating
        };

        // Calculate rating distribution
        let mut distribution = [0u64; 5];
        for review in &reviews {
            let rating_index = (review.review.rating.round() as usize - 1).min(4);
            distribution[rating_index] += 1;
        }

        // Calculate confidence score
        let confidence_score = (total_reviews as f32 / self.config.required_reviews_for_stability as f32).min(1.0);

        // Calculate sentiment score
        let sentiment_score = if self.config.enable_sentiment_analysis {
            let sentiment_sum: f32 = reviews.iter()
                .filter_map(|r| r.sentiment_analysis.as_ref())
                .map(|s| s.overall_sentiment)
                .sum();
            let sentiment_count = reviews.iter()
                .filter(|r| r.sentiment_analysis.is_some())
                .count();

            if sentiment_count > 0 {
                sentiment_sum / sentiment_count as f32
            } else {
                0.0
            }
        } else {
            0.0
        };

        let model_rating = ModelRating {
            model_id: *model_id,
            average_rating,
            weighted_rating,
            total_reviews,
            rating_distribution: distribution,
            confidence_score,
            sentiment_score,
            last_updated: Utc::now(),
        };

        self.model_ratings.insert(*model_id, model_rating);

        debug!(
            model_id = ?model_id,
            average_rating = average_rating,
            weighted_rating = weighted_rating,
            total_reviews = total_reviews,
            "Model rating recalculated"
        );

        Ok(())
    }
}

/// Calculate the quality of a review based on various factors
fn calculate_review_quality(review: &UserReview) -> f32 {
    let quality_score = 0.5; // Base score

    // Length factor: longer, detailed reviews are higher quality
    let content_length = review.content.len();
    let length_factor = match content_length {
        0..=50 => 0.0,        // Very short reviews
        51..=200 => 0.2,      // Short but meaningful
        201..=500 => 0.4,     // Good detail
        501..=1000 => 0.3,    // Very detailed
        _ => 0.2,             // Potentially too verbose
    };

    // Pros/cons factor: reviews with specific pros/cons are more helpful
    let pros_cons_factor = match (review.pros.len(), review.cons.len()) {
        (0, 0) => 0.0,        // No specific details
        (p, c) if p + c <= 3 => 0.2,  // Some specific points
        (p, c) if p + c <= 6 => 0.3,  // Good balance of details
        _ => 0.1,             // Potentially too verbose
    };

    // Title factor: reviews with meaningful titles are higher quality
    let title_factor = if review.title.len() > 5 &&
                          !review.title.to_lowercase().contains("good") &&
                          !review.title.to_lowercase().contains("bad") {
        0.1
    } else {
        0.0
    };

    // Recommendation factor: decisive recommendations indicate engagement
    let recommendation_factor = if review.recommended { 0.1 } else { 0.05 };

    quality_score + length_factor + pros_cons_factor + title_factor + recommendation_factor
}

/// Review sorting options
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReviewSortOrder {
    MostHelpful,
    Newest,
    Oldest,
    HighestRating,
    LowestRating,
}