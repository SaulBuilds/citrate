// citrate/core/marketplace/src/types.rs

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Model identifier (32-byte hash from blockchain)
pub type ModelId = [u8; 32];

/// User address (20-byte Ethereum address)
pub type Address = [u8; 20];

/// IPFS Content Identifier
pub type IpfsCid = String;

/// Model categories for classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum ModelCategory {
    LanguageModel = 0,
    ImageGeneration = 1,
    ImageClassification = 2,
    AudioProcessing = 3,
    VideoProcessing = 4,
    Embedding = 5,
    ObjectDetection = 6,
    TextToSpeech = 7,
    SpeechToText = 8,
    Translation = 9,
    Other = 10,
}

impl From<u8> for ModelCategory {
    fn from(value: u8) -> Self {
        match value {
            0 => ModelCategory::LanguageModel,
            1 => ModelCategory::ImageGeneration,
            2 => ModelCategory::ImageClassification,
            3 => ModelCategory::AudioProcessing,
            4 => ModelCategory::VideoProcessing,
            5 => ModelCategory::Embedding,
            6 => ModelCategory::ObjectDetection,
            7 => ModelCategory::TextToSpeech,
            8 => ModelCategory::SpeechToText,
            9 => ModelCategory::Translation,
            _ => ModelCategory::Other,
        }
    }
}

impl ModelCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            ModelCategory::LanguageModel => "Language Model",
            ModelCategory::ImageGeneration => "Image Generation",
            ModelCategory::ImageClassification => "Image Classification",
            ModelCategory::AudioProcessing => "Audio Processing",
            ModelCategory::VideoProcessing => "Video Processing",
            ModelCategory::Embedding => "Embedding",
            ModelCategory::ObjectDetection => "Object Detection",
            ModelCategory::TextToSpeech => "Text-to-Speech",
            ModelCategory::SpeechToText => "Speech-to-Text",
            ModelCategory::Translation => "Translation",
            ModelCategory::Other => "Other",
        }
    }

    pub fn all() -> &'static [ModelCategory] {
        &[
            ModelCategory::LanguageModel,
            ModelCategory::ImageGeneration,
            ModelCategory::ImageClassification,
            ModelCategory::AudioProcessing,
            ModelCategory::VideoProcessing,
            ModelCategory::Embedding,
            ModelCategory::ObjectDetection,
            ModelCategory::TextToSpeech,
            ModelCategory::SpeechToText,
            ModelCategory::Translation,
            ModelCategory::Other,
        ]
    }
}

/// Complete model information for marketplace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketplaceModel {
    pub model_id: ModelId,
    pub owner: Address,
    pub name: String,
    pub description: String,
    pub category: ModelCategory,

    // Pricing information
    pub base_price: u64, // Wei per inference
    pub discount_price: u64,
    pub minimum_bulk_size: u32,

    // Model details
    pub framework: String,
    pub version: String,
    pub license: String,
    pub tags: Vec<String>,
    pub input_shape: Vec<String>,
    pub output_shape: Vec<String>,
    pub parameters: u64,
    pub size_bytes: u64,

    // IPFS references
    pub model_cid: IpfsCid,
    pub metadata_uri: IpfsCid,

    // Marketplace stats
    pub total_sales: u64,
    pub total_revenue: u64,
    pub rating: f32, // Average rating 0.0-5.0
    pub review_count: u32,
    pub featured: bool,
    pub active: bool,

    // Timestamps
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_sale_at: Option<DateTime<Utc>>,
}

/// User review and rating
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelReview {
    pub model_id: ModelId,
    pub reviewer: Address,
    pub rating: u8, // 1-5 stars
    pub comment: String,
    pub verified: bool, // True if reviewer has purchased the model
    pub created_at: DateTime<Utc>,
}

/// Purchase record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Purchase {
    pub model_id: ModelId,
    pub buyer: Address,
    pub price_per_inference: u64,
    pub quantity: u32,
    pub bulk_discount: bool,
    pub transaction_hash: String,
    pub timestamp: DateTime<Utc>,
}

/// User interaction for recommendations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInteraction {
    pub user: Address,
    pub model_id: ModelId,
    pub interaction_type: InteractionType,
    pub timestamp: DateTime<Utc>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum InteractionType {
    View,
    Purchase,
    Review,
    Bookmark,
    Share,
}

/// Performance metrics for a model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelMetrics {
    pub model_id: ModelId,
    pub average_latency_ms: f32,
    pub success_rate: f32,
    pub quality_score: f32, // Computed from reviews and usage
    pub popularity_score: f32, // Based on views, purchases, etc.
    pub updated_at: DateTime<Utc>,
}

/// Search filters for marketplace queries
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SearchFilters {
    pub categories: Option<Vec<ModelCategory>>,
    pub min_price: Option<u64>,
    pub max_price: Option<u64>,
    pub min_rating: Option<f32>,
    pub frameworks: Option<Vec<String>>,
    pub licenses: Option<Vec<String>>,
    pub tags: Option<Vec<String>>,
    pub featured_only: bool,
    pub verified_reviews_only: bool,
}

/// Sorting options for search results
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum SortBy {
    Relevance,
    Rating,
    Price,
    Sales,
    Newest,
    MostReviewed,
    Popularity,
}

/// User review for a model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserReview {
    pub model_id: ModelId,
    pub reviewer: Address,
    pub rating: f32, // 1.0 to 5.0
    pub title: String,
    pub content: String,
    pub pros: Vec<String>,
    pub cons: Vec<String>,
    pub recommended: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Marketplace statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketplaceStats {
    pub total_models: u64,
    pub total_interactions: u64,
    pub total_reviews: u64,
    pub category_distribution: HashMap<ModelCategory, u64>,
    pub top_models: Vec<ModelId>,
    pub last_updated: DateTime<Utc>,
}

impl Default for MarketplaceStats {
    fn default() -> Self {
        Self {
            total_models: 0,
            total_interactions: 0,
            total_reviews: 0,
            category_distribution: HashMap::new(),
            top_models: Vec::new(),
            last_updated: Utc::now(),
        }
    }
}

/// Error types for marketplace operations
#[derive(Debug, thiserror::Error)]
pub enum MarketplaceError {
    #[error("Model not found: {0:?}")]
    ModelNotFound(ModelId),

    #[error("IPFS error: {0}")]
    IpfsError(String),

    #[error("Search index error: {0}")]
    SearchError(String),

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Invalid metadata: {0}")]
    InvalidMetadata(String),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),
}

impl From<serde_json::Error> for MarketplaceError {
    fn from(err: serde_json::Error) -> Self {
        MarketplaceError::SerializationError(err.to_string())
    }
}

impl From<reqwest::Error> for MarketplaceError {
    fn from(err: reqwest::Error) -> Self {
        MarketplaceError::NetworkError(err.to_string())
    }
}

// Tantivy error conversion removed for simplified implementation