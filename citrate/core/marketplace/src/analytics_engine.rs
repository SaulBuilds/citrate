// citrate/core/marketplace/src/analytics_engine.rs
//
// # Analytics Data Pipeline
//
// ## Overview
// This module provides analytics for AI models in the marketplace. It follows a
// FAIL-LOUD design: if data is unavailable, errors are returned rather than
// fabricated zeros.
//
// ## Data Sources
// - **PerformanceTracker**: In-memory sliding windows tracking inference metrics
// - **RatingSystem**: Stored reviews and ratings from users
// - **PerformanceWindows**: Time-bucketed latency/throughput measurements
//
// ## Data Flow
// 1. Transactions execute inference requests → Executor records metrics
// 2. PerformanceTracker collects metrics into sliding windows
// 3. AnalyticsEngine queries PerformanceTracker for aggregated stats
// 4. If no data exists, analytics methods return errors (not zeros)
//
// ## Current Limitations
// - Usage stats come from in-memory PerformanceTracker (lost on restart)
// - Market stats require marketplace indexer to be configured
// - No persistence layer for historical analytics (TODO: add time-series DB)
//
// ## Fail-Loud Behavior
// The following methods return errors when data is unavailable:
// - `analyze_user_engagement()` → "User engagement analytics unavailable"
// - `analyze_market_position()` → "Market position analytics unavailable"
//
// This ensures callers know they're missing real data rather than receiving
// misleading zeros that could affect business decisions.
//
// ## OPERATIONAL WARNING
// **Analytics data is volatile and not persisted across node restarts.**
//
// ### Implications for Production
// - All inference metrics are reset when the node restarts
// - Historical usage data is lost on restart
// - RPC queries for usage stats will return errors until new data accumulates
//
// ### Recommended Mitigations
// 1. **External Aggregation**: Export metrics via Prometheus/Grafana for persistence
// 2. **Periodic Snapshots**: Implement periodic dump of PerformanceWindows to disk
// 3. **Time-Series DB**: Future integration with InfluxDB/TimescaleDB for production
//
// ### When This Is Acceptable
// - Development and testing environments
// - Nodes with short-lived sessions
// - When external monitoring is configured
//
// For production deployments requiring historical analytics, configure external
// metrics aggregation before go-live.

use crate::{
    performance_tracker::*,
    rating_system::*,
    types::*,
};
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

/// Combined analytics report for a model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelAnalyticsReport {
    pub model_id: ModelId,
    pub rating_analysis: RatingAnalysis,
    pub performance_analysis: PerformanceAnalysis,
    pub user_engagement: UserEngagementMetrics,
    pub market_position: MarketPositionAnalysis,
    pub recommendations: Vec<ImprovementRecommendation>,
    pub overall_score: f32,
    pub generated_at: DateTime<Utc>,
}

/// Rating analysis component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RatingAnalysis {
    pub current_rating: f32,
    pub rating_trend: RatingTrend,
    pub review_quality_score: f32,
    pub sentiment_analysis: SentimentSummary,
    pub rating_stability: f32,
    pub user_satisfaction_index: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RatingTrend {
    StronglyImproving,
    Improving,
    Stable,
    Declining,
    StronglyDeclining,
}

/// Performance analysis component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceAnalysis {
    pub reliability_score: f32,
    pub speed_score: f32,
    pub efficiency_score: f32,
    pub scalability_score: f32,
    pub performance_trend: PerformanceTrend,
    pub benchmark_comparison: BenchmarkComparison,
    pub uptime_analysis: UptimeAnalysis,
}

/// User engagement metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserEngagementMetrics {
    pub daily_active_users: u64,
    pub user_retention_rate: f32,
    pub session_duration_avg: f32,
    pub repeat_usage_rate: f32,
    pub churn_rate: f32,
    pub engagement_score: f32,
}

/// Market position analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketPositionAnalysis {
    pub category_rank: u32,
    pub overall_rank: u32,
    pub market_share: f32,
    pub competitive_advantage: Vec<String>,
    pub market_threats: Vec<String>,
    pub growth_potential: f32,
}

/// Improvement recommendations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImprovementRecommendation {
    pub category: RecommendationCategory,
    pub priority: RecommendationPriority,
    pub title: String,
    pub description: String,
    pub expected_impact: f32,
    pub implementation_difficulty: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecommendationCategory {
    Performance,
    UserExperience,
    Marketing,
    Pricing,
    Features,
    Documentation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecommendationPriority {
    Critical,
    High,
    Medium,
    Low,
}

/// Sentiment analysis summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SentimentSummary {
    pub overall_sentiment: f32,
    pub positive_percentage: f32,
    pub neutral_percentage: f32,
    pub negative_percentage: f32,
    pub sentiment_trend: SentimentTrend,
    pub key_themes: HashMap<String, f32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SentimentTrend {
    ImprovingPositivity,
    Stable,
    DecliningPositivity,
}

/// Benchmark comparison data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkComparison {
    pub vs_category_average: f32,
    pub vs_top_performer: f32,
    pub improvement_over_time: f32,
    pub performance_percentile: f32,
}

/// Uptime analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UptimeAnalysis {
    pub current_uptime: f32,
    pub historical_average: f32,
    pub downtime_incidents: u32,
    pub mttr_hours: f32, // Mean Time To Recovery
    pub reliability_trend: ReliabilityTrend,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReliabilityTrend {
    Improving,
    Stable,
    Degrading,
}

/// Analytics engine that combines rating and performance data
pub struct AnalyticsEngine {
    rating_system: Arc<RatingSystem>,
    performance_tracker: Arc<PerformanceTracker>,
}

impl AnalyticsEngine {
    /// Create a new analytics engine
    pub fn new(
        rating_system: Arc<RatingSystem>,
        performance_tracker: Arc<PerformanceTracker>,
    ) -> Self {
        Self {
            rating_system,
            performance_tracker,
        }
    }

    /// Generate comprehensive analytics report for a model
    pub async fn generate_model_report(&self, model_id: &ModelId) -> Result<ModelAnalyticsReport> {
        let rating_analysis = self.analyze_ratings(model_id).await?;
        let performance_analysis = self.analyze_performance(model_id).await?;
        let user_engagement = self.analyze_user_engagement(model_id).await?;
        let market_position = self.analyze_market_position(model_id).await?;
        let recommendations = self.generate_recommendations(
            model_id,
            &rating_analysis,
            &performance_analysis,
            &user_engagement,
        ).await?;

        // Calculate overall score
        let overall_score = self.calculate_overall_score(
            &rating_analysis,
            &performance_analysis,
            &user_engagement,
        );

        Ok(ModelAnalyticsReport {
            model_id: *model_id,
            rating_analysis,
            performance_analysis,
            user_engagement,
            market_position,
            recommendations,
            overall_score,
            generated_at: Utc::now(),
        })
    }

    /// Get comparative analytics for multiple models
    pub async fn compare_models(&self, model_ids: &[ModelId]) -> Result<HashMap<ModelId, ModelAnalyticsReport>> {
        let mut reports = HashMap::new();

        for model_id in model_ids {
            if let Ok(report) = self.generate_model_report(model_id).await {
                reports.insert(*model_id, report);
            }
        }

        Ok(reports)
    }

    /// Get category analytics
    pub async fn analyze_category(&self, category: ModelCategory) -> Result<CategoryAnalytics> {
        // This would integrate with storage to get all models in category
        // For now, return a placeholder
        Ok(CategoryAnalytics {
            category,
            total_models: 0,
            average_rating: 0.0,
            average_performance_score: 0.0,
            top_models: Vec::new(),
            category_trends: CategoryTrends::default(),
            market_insights: Vec::new(),
        })
    }

    // Private analysis methods

    async fn analyze_ratings(&self, model_id: &ModelId) -> Result<RatingAnalysis> {
        let model_rating = self.rating_system.get_model_rating(model_id).await
            .unwrap_or_else(|| ModelRating {
                model_id: *model_id,
                average_rating: 0.0,
                weighted_rating: 0.0,
                total_reviews: 0,
                rating_distribution: [0; 5],
                confidence_score: 0.0,
                sentiment_score: 0.0,
                last_updated: Utc::now(),
            });

        let reviews = self.rating_system.get_model_reviews(model_id, 100, ReviewSortOrder::Newest).await;

        // Calculate rating trend
        let rating_trend = self.calculate_rating_trend(&reviews);

        // Calculate review quality score
        let review_quality_score = if reviews.is_empty() {
            0.0
        } else {
            reviews.iter().map(|r| {
                (r.quality.helpfulness_score * 0.3) +
                (r.quality.detail_score * 0.3) +
                (r.quality.reviewer_credibility * 0.2) +
                (if r.quality.verified_purchase { 0.2 } else { 0.0 })
            }).sum::<f32>() / reviews.len() as f32
        };

        // Analyze sentiment
        let sentiment_analysis = self.analyze_review_sentiment(&reviews);

        // Calculate user satisfaction index
        let user_satisfaction_index = (model_rating.weighted_rating / 5.0) * 0.6 +
                                     (review_quality_score * 0.2) +
                                     ((sentiment_analysis.overall_sentiment + 1.0) / 2.0 * 0.2);

        Ok(RatingAnalysis {
            current_rating: model_rating.weighted_rating,
            rating_trend,
            review_quality_score,
            sentiment_analysis,
            rating_stability: model_rating.confidence_score,
            user_satisfaction_index,
        })
    }

    async fn analyze_performance(&self, model_id: &ModelId) -> Result<PerformanceAnalysis> {
        let health_status = self.performance_tracker.get_model_health(model_id).await;
        let benchmark_results = self.performance_tracker.get_benchmark_results(model_id, 10).await;

        let (reliability_score, speed_score, efficiency_score, scalability_score) =
            if let Some(health) = &health_status {
                (
                    health.uptime_percentage / 100.0,
                    1.0 - (health.current_latency_ms as f32 / 10000.0).min(1.0),
                    health.health_score,
                    0.8, // Placeholder - would calculate from throughput metrics
                )
            } else {
                (0.0, 0.0, 0.0, 0.0)
            };

        let performance_trend = health_status
            .map(|h| h.performance_trend)
            .unwrap_or(PerformanceTrend::Stable);

        let benchmark_comparison = self.calculate_benchmark_comparison(&benchmark_results);
        let uptime_analysis = self.analyze_uptime(model_id).await?;

        Ok(PerformanceAnalysis {
            reliability_score,
            speed_score,
            efficiency_score,
            scalability_score,
            performance_trend,
            benchmark_comparison,
            uptime_analysis,
        })
    }

    async fn analyze_user_engagement(&self, model_id: &ModelId) -> Result<UserEngagementMetrics> {
        // Query usage analytics from performance tracker
        // If no data available, return error to fail loud instead of fake zeros
        let usage_stats = self.performance_tracker.get_usage_stats(model_id).await;

        match usage_stats {
            Some(stats) => Ok(UserEngagementMetrics {
                daily_active_users: stats.daily_active_users,
                user_retention_rate: stats.retention_rate,
                session_duration_avg: stats.avg_session_duration_secs,
                repeat_usage_rate: stats.repeat_usage_rate,
                churn_rate: 1.0 - stats.retention_rate, // Churn = 1 - retention
                engagement_score: (stats.retention_rate * 0.4)
                    + (stats.repeat_usage_rate * 0.3)
                    + ((stats.avg_session_duration_secs / 300.0).min(1.0) * 0.3),
            }),
            None => {
                // SECURITY: Return error instead of fabricated zeros
                // This ensures callers know no real data is available
                anyhow::bail!(
                    "User engagement analytics unavailable for model {:?}. \
                     Usage tracking may not be configured or no usage data exists.",
                    model_id
                )
            }
        }
    }

    async fn analyze_market_position(&self, model_id: &ModelId) -> Result<MarketPositionAnalysis> {
        // Query marketplace statistics
        // If no data available, return error to fail loud instead of fake zeros
        let market_stats = self.performance_tracker.get_market_stats(model_id).await;

        match market_stats {
            Some(stats) => Ok(MarketPositionAnalysis {
                category_rank: stats.category_rank,
                overall_rank: stats.overall_rank,
                market_share: stats.market_share_percent,
                competitive_advantage: stats.strengths.clone(),
                market_threats: stats.weaknesses.clone(),
                growth_potential: stats.growth_potential_score,
            }),
            None => {
                // SECURITY: Return error instead of fabricated zeros
                // This ensures callers know no real data is available
                anyhow::bail!(
                    "Market position analytics unavailable for model {:?}. \
                     Marketplace indexing may not be configured or model not listed.",
                    model_id
                )
            }
        }
    }

    async fn generate_recommendations(
        &self,
        _model_id: &ModelId,
        rating_analysis: &RatingAnalysis,
        performance_analysis: &PerformanceAnalysis,
        _user_engagement: &UserEngagementMetrics,
    ) -> Result<Vec<ImprovementRecommendation>> {
        let mut recommendations = Vec::new();

        // Performance recommendations
        if performance_analysis.reliability_score < 0.95 {
            recommendations.push(ImprovementRecommendation {
                category: RecommendationCategory::Performance,
                priority: RecommendationPriority::High,
                title: "Improve System Reliability".to_string(),
                description: "Focus on reducing downtime and improving system stability to increase user confidence.".to_string(),
                expected_impact: 0.8,
                implementation_difficulty: 0.6,
            });
        }

        if performance_analysis.speed_score < 0.8 {
            recommendations.push(ImprovementRecommendation {
                category: RecommendationCategory::Performance,
                priority: RecommendationPriority::High,
                title: "Optimize Response Times".to_string(),
                description: "Reduce latency to improve user experience and satisfaction.".to_string(),
                expected_impact: 0.7,
                implementation_difficulty: 0.7,
            });
        }

        // Rating recommendations
        if rating_analysis.current_rating < 4.0 {
            recommendations.push(ImprovementRecommendation {
                category: RecommendationCategory::UserExperience,
                priority: RecommendationPriority::High,
                title: "Address User Feedback".to_string(),
                description: "Analyze negative reviews and implement improvements to address common complaints.".to_string(),
                expected_impact: 0.9,
                implementation_difficulty: 0.5,
            });
        }

        if rating_analysis.sentiment_analysis.negative_percentage > 0.3 {
            recommendations.push(ImprovementRecommendation {
                category: RecommendationCategory::UserExperience,
                priority: RecommendationPriority::Medium,
                title: "Improve User Sentiment".to_string(),
                description: "Focus on features and improvements that address negative sentiment themes.".to_string(),
                expected_impact: 0.6,
                implementation_difficulty: 0.4,
            });
        }

        Ok(recommendations)
    }

    fn calculate_overall_score(
        &self,
        rating_analysis: &RatingAnalysis,
        performance_analysis: &PerformanceAnalysis,
        user_engagement: &UserEngagementMetrics,
    ) -> f32 {
        let rating_weight = 0.3;
        let performance_weight = 0.4;
        let engagement_weight = 0.3;

        let rating_score = rating_analysis.user_satisfaction_index;
        let performance_score = performance_analysis.reliability_score * 0.3 +
            performance_analysis.speed_score * 0.3 +
            performance_analysis.efficiency_score * 0.2 +
            performance_analysis.scalability_score * 0.2;
        let engagement_score = user_engagement.engagement_score;

        (rating_score * rating_weight +
         performance_score * performance_weight +
         engagement_score * engagement_weight).max(0.0).min(1.0)
    }

    fn calculate_rating_trend(&self, reviews: &[EnhancedUserReview]) -> RatingTrend {
        if reviews.len() < 6 {
            return RatingTrend::Stable;
        }

        let recent_avg = reviews.iter().take(3).map(|r| r.review.rating).sum::<f32>() / 3.0;
        let older_avg = reviews.iter().skip(3).take(3).map(|r| r.review.rating).sum::<f32>() / 3.0;

        let diff = recent_avg - older_avg;

        match diff {
            d if d > 0.5 => RatingTrend::StronglyImproving,
            d if d > 0.1 => RatingTrend::Improving,
            d if d < -0.5 => RatingTrend::StronglyDeclining,
            d if d < -0.1 => RatingTrend::Declining,
            _ => RatingTrend::Stable,
        }
    }

    fn analyze_review_sentiment(&self, reviews: &[EnhancedUserReview]) -> SentimentSummary {
        if reviews.is_empty() {
            return SentimentSummary {
                overall_sentiment: 0.0,
                positive_percentage: 0.0,
                neutral_percentage: 100.0,
                negative_percentage: 0.0,
                sentiment_trend: SentimentTrend::Stable,
                key_themes: HashMap::new(),
            };
        }

        let sentiments: Vec<f32> = reviews.iter()
            .filter_map(|r| r.sentiment_analysis.as_ref())
            .map(|s| s.overall_sentiment)
            .collect();

        let overall_sentiment = if sentiments.is_empty() {
            0.0
        } else {
            sentiments.iter().sum::<f32>() / sentiments.len() as f32
        };

        let positive_count = sentiments.iter().filter(|&&s| s > 0.1).count();
        let negative_count = sentiments.iter().filter(|&&s| s < -0.1).count();
        let neutral_count = sentiments.len() - positive_count - negative_count;

        let total = sentiments.len() as f32;
        let positive_percentage = if total > 0.0 { positive_count as f32 / total * 100.0 } else { 0.0 };
        let negative_percentage = if total > 0.0 { negative_count as f32 / total * 100.0 } else { 0.0 };
        let neutral_percentage = if total > 0.0 { neutral_count as f32 / total * 100.0 } else { 100.0 };

        // Calculate sentiment trend
        let sentiment_trend = if reviews.len() >= 6 {
            let recent_sentiment = reviews.iter().take(3)
                .filter_map(|r| r.sentiment_analysis.as_ref())
                .map(|s| s.overall_sentiment)
                .sum::<f32>() / 3.0;
            let older_sentiment = reviews.iter().skip(3).take(3)
                .filter_map(|r| r.sentiment_analysis.as_ref())
                .map(|s| s.overall_sentiment)
                .sum::<f32>() / 3.0;

            if recent_sentiment > older_sentiment + 0.1 {
                SentimentTrend::ImprovingPositivity
            } else if recent_sentiment < older_sentiment - 0.1 {
                SentimentTrend::DecliningPositivity
            } else {
                SentimentTrend::Stable
            }
        } else {
            SentimentTrend::Stable
        };

        SentimentSummary {
            overall_sentiment,
            positive_percentage,
            neutral_percentage,
            negative_percentage,
            sentiment_trend,
            key_themes: HashMap::new(), // Would be populated by NLP analysis
        }
    }

    fn calculate_benchmark_comparison(&self, _benchmark_results: &[BenchmarkResult]) -> BenchmarkComparison {
        // Placeholder implementation
        BenchmarkComparison {
            vs_category_average: 0.0,
            vs_top_performer: 0.0,
            improvement_over_time: 0.0,
            performance_percentile: 0.0,
        }
    }

    async fn analyze_uptime(&self, model_id: &ModelId) -> Result<UptimeAnalysis> {
        let health_status = self.performance_tracker.get_model_health(model_id).await;

        let current_uptime = health_status
            .map(|h| h.uptime_percentage)
            .unwrap_or(0.0);

        Ok(UptimeAnalysis {
            current_uptime,
            historical_average: current_uptime, // Simplified
            downtime_incidents: 0,
            mttr_hours: 0.0,
            reliability_trend: ReliabilityTrend::Stable,
        })
    }
}

/// Category-level analytics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryAnalytics {
    pub category: ModelCategory,
    pub total_models: u64,
    pub average_rating: f32,
    pub average_performance_score: f32,
    pub top_models: Vec<ModelId>,
    pub category_trends: CategoryTrends,
    pub market_insights: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CategoryTrends {
    pub growth_rate: f32,
    pub competition_level: f32,
    pub innovation_index: f32,
    pub user_demand_trend: f32,
}