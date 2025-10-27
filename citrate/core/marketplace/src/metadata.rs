// citrate/core/marketplace/src/metadata.rs

use crate::types::*;
use anyhow::Result;
use dashmap::DashMap;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::{interval, Instant};
use tracing::{debug, error, info, warn};

/// Extended model metadata from IPFS
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelMetadata {
    pub name: String,
    pub description: String,
    pub version: String,
    pub author: String,
    pub license: String,
    pub tags: Vec<String>,

    // Technical specifications
    pub framework: String,
    pub model_type: String,
    pub architecture: String,
    pub parameters: u64,
    pub size_bytes: u64,

    // Input/output specifications
    pub input_spec: InputOutputSpec,
    pub output_spec: InputOutputSpec,

    // Performance metrics
    pub benchmarks: Vec<Benchmark>,
    pub hardware_requirements: HardwareRequirements,

    // Usage information
    pub examples: Vec<UsageExample>,
    pub documentation_url: Option<String>,
    pub paper_url: Option<String>,

    // Marketplace specific
    pub thumbnail_url: Option<String>,
    pub demo_url: Option<String>,
    pub pricing_notes: Option<String>,

    // Creation metadata
    pub created_at: String,
    pub updated_at: String,
    pub ipfs_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputOutputSpec {
    pub format: String, // "tensor", "text", "image", "audio", etc.
    pub shape: Vec<String>,
    pub dtype: String,
    pub description: String,
    pub example: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Benchmark {
    pub name: String,
    pub dataset: String,
    pub metric: String,
    pub value: f64,
    pub units: String,
    pub hardware: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareRequirements {
    pub min_memory_gb: f32,
    pub recommended_memory_gb: f32,
    pub gpu_required: bool,
    pub min_gpu_memory_gb: Option<f32>,
    pub supported_platforms: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageExample {
    pub title: String,
    pub description: String,
    pub input: serde_json::Value,
    pub output: serde_json::Value,
    pub code: Option<String>,
}

/// Cached metadata entry
#[derive(Debug, Clone)]
struct CacheEntry {
    metadata: ModelMetadata,
    fetched_at: Instant,
    ttl: Duration,
}

impl CacheEntry {
    fn is_expired(&self) -> bool {
        self.fetched_at.elapsed() > self.ttl
    }
}

/// IPFS metadata cache and fetcher
pub struct MetadataCache {
    client: Client,
    cache: Arc<DashMap<IpfsCid, CacheEntry>>,
    ipfs_gateways: Vec<String>,
    default_ttl: Duration,
    max_cache_size: usize,
}

impl MetadataCache {
    /// Create a new metadata cache
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        let ipfs_gateways = vec![
            "https://ipfs.io/ipfs/".to_string(),
            "https://gateway.pinata.cloud/ipfs/".to_string(),
            "https://cloudflare-ipfs.com/ipfs/".to_string(),
            "https://dweb.link/ipfs/".to_string(),
        ];

        Self {
            client,
            cache: Arc::new(DashMap::new()),
            ipfs_gateways,
            default_ttl: Duration::from_secs(3600), // 1 hour
            max_cache_size: 10000,
        }
    }

    /// Configure IPFS gateways
    pub fn with_gateways(mut self, gateways: Vec<String>) -> Self {
        self.ipfs_gateways = gateways;
        self
    }

    /// Configure cache TTL
    pub fn with_ttl(mut self, ttl: Duration) -> Self {
        self.default_ttl = ttl;
        self
    }

    /// Configure maximum cache size
    pub fn with_max_cache_size(mut self, size: usize) -> Self {
        self.max_cache_size = size;
        self
    }

    /// Fetch metadata for a model from IPFS
    pub async fn get_metadata(&self, ipfs_cid: &str) -> Result<ModelMetadata> {
        // Check cache first
        if let Some(entry) = self.cache.get(ipfs_cid) {
            if !entry.is_expired() {
                debug!(cid = ipfs_cid, "Cache hit for metadata");
                return Ok(entry.metadata.clone());
            } else {
                debug!(cid = ipfs_cid, "Cache entry expired");
            }
        }

        // Fetch from IPFS
        let metadata = self.fetch_from_ipfs(ipfs_cid).await?;

        // Store in cache
        self.cache_metadata(ipfs_cid.to_string(), metadata.clone()).await;

        Ok(metadata)
    }

    /// Prefetch metadata for multiple models
    pub async fn prefetch_metadata(&self, cids: Vec<String>) -> Result<Vec<(String, Result<ModelMetadata>)>> {
        let mut handles = Vec::new();

        for cid in cids {
            let cache = self.clone();
            let handle = tokio::spawn(async move {
                let result = cache.get_metadata(&cid).await;
                (cid, result)
            });
            handles.push(handle);
        }

        let mut results = Vec::new();
        for handle in handles {
            match handle.await {
                Ok((cid, result)) => results.push((cid, result)),
                Err(e) => {
                    error!(error = %e, "Failed to join prefetch task");
                }
            }
        }

        Ok(results)
    }

    /// Clear expired entries from cache
    pub async fn cleanup_cache(&self) {
        let mut expired_keys = Vec::new();

        // Collect expired keys
        for entry in self.cache.iter() {
            if entry.value().is_expired() {
                expired_keys.push(entry.key().clone());
            }
        }

        // Remove expired entries
        for key in expired_keys {
            self.cache.remove(&key);
        }

        // Enforce max cache size
        if self.cache.len() > self.max_cache_size {
            let excess = self.cache.len() - self.max_cache_size;
            let mut removed = 0;

            // Remove oldest entries (this is not perfect LRU but good enough)
            let mut to_remove = Vec::new();
            for entry in self.cache.iter() {
                to_remove.push(entry.key().clone());
                removed += 1;
                if removed >= excess {
                    break;
                }
            }

            for key in to_remove {
                self.cache.remove(&key);
            }
        }

        debug!(
            cache_size = self.cache.len(),
            max_size = self.max_cache_size,
            "Cleaned up metadata cache"
        );
    }

    /// Start a background cleanup task
    pub fn start_cleanup_task(self: Arc<Self>) {
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(300)); // 5 minutes

            loop {
                interval.tick().await;
                self.cleanup_cache().await;
            }
        });
    }

    /// Get cache statistics
    pub fn get_cache_stats(&self) -> (usize, usize) {
        let total_entries = self.cache.len();
        let expired_entries = self.cache.iter()
            .filter(|entry| entry.value().is_expired())
            .count();

        (total_entries, expired_entries)
    }

    // Private methods

    async fn fetch_from_ipfs(&self, cid: &str) -> Result<ModelMetadata> {
        let mut last_error = None;

        // Try each gateway
        for gateway in &self.ipfs_gateways {
            let url = format!("{}{}", gateway, cid);

            debug!(url = %url, "Fetching metadata from IPFS gateway");

            match self.client.get(&url).send().await {
                Ok(response) => {
                    if response.status().is_success() {
                        match response.json::<ModelMetadata>().await {
                            Ok(metadata) => {
                                info!(cid = cid, gateway = %gateway, "Successfully fetched metadata");
                                return Ok(metadata);
                            }
                            Err(e) => {
                                warn!(
                                    cid = cid,
                                    gateway = %gateway,
                                    error = %e,
                                    "Failed to parse metadata JSON"
                                );
                                last_error = Some(e.into());
                            }
                        }
                    } else {
                        warn!(
                            cid = cid,
                            gateway = %gateway,
                            status = %response.status(),
                            "HTTP error from IPFS gateway"
                        );
                        last_error = Some(anyhow::anyhow!(
                            "HTTP {} from gateway {}",
                            response.status(),
                            gateway
                        ));
                    }
                }
                Err(e) => {
                    warn!(
                        cid = cid,
                        gateway = %gateway,
                        error = %e,
                        "Network error fetching from IPFS gateway"
                    );
                    last_error = Some(e.into());
                }
            }
        }

        // If all gateways failed, return the last error
        Err(last_error.unwrap_or_else(|| {
            anyhow::anyhow!("All IPFS gateways failed for CID: {}", cid)
        }))
    }

    async fn cache_metadata(&self, cid: String, metadata: ModelMetadata) {
        let entry = CacheEntry {
            metadata,
            fetched_at: Instant::now(),
            ttl: self.default_ttl,
        };

        self.cache.insert(cid, entry);
    }
}

impl Clone for MetadataCache {
    fn clone(&self) -> Self {
        Self {
            client: self.client.clone(),
            cache: Arc::clone(&self.cache),
            ipfs_gateways: self.ipfs_gateways.clone(),
            default_ttl: self.default_ttl,
            max_cache_size: self.max_cache_size,
        }
    }
}

/// Validate metadata structure
pub fn validate_metadata(metadata: &ModelMetadata) -> Result<()> {
    if metadata.name.trim().is_empty() {
        return Err(anyhow::anyhow!("Model name cannot be empty"));
    }

    if metadata.description.trim().is_empty() {
        return Err(anyhow::anyhow!("Model description cannot be empty"));
    }

    if metadata.framework.trim().is_empty() {
        return Err(anyhow::anyhow!("Model framework cannot be empty"));
    }

    if metadata.parameters == 0 {
        return Err(anyhow::anyhow!("Model parameters must be greater than 0"));
    }

    if metadata.size_bytes == 0 {
        return Err(anyhow::anyhow!("Model size must be greater than 0"));
    }

    // Validate input spec
    if metadata.input_spec.format.trim().is_empty() {
        return Err(anyhow::anyhow!("Input format cannot be empty"));
    }

    // Validate output spec
    if metadata.output_spec.format.trim().is_empty() {
        return Err(anyhow::anyhow!("Output format cannot be empty"));
    }

    // Validate hardware requirements
    if metadata.hardware_requirements.min_memory_gb <= 0.0 {
        return Err(anyhow::anyhow!("Minimum memory requirement must be positive"));
    }

    if metadata.hardware_requirements.recommended_memory_gb < metadata.hardware_requirements.min_memory_gb {
        return Err(anyhow::anyhow!("Recommended memory must be >= minimum memory"));
    }

    Ok(())
}

/// Create example metadata for testing
pub fn create_example_metadata(model_name: &str, framework: &str) -> ModelMetadata {
    ModelMetadata {
        name: model_name.to_string(),
        description: format!("A powerful {} model for various AI tasks", framework),
        version: "1.0.0".to_string(),
        author: "Citrate AI".to_string(),
        license: "MIT".to_string(),
        tags: vec!["ai".to_string(), "ml".to_string(), framework.to_lowercase()],

        framework: framework.to_string(),
        model_type: "neural_network".to_string(),
        architecture: "transformer".to_string(),
        parameters: 7_000_000_000,
        size_bytes: 14_000_000_000,

        input_spec: InputOutputSpec {
            format: "text".to_string(),
            shape: vec!["variable".to_string()],
            dtype: "string".to_string(),
            description: "Input text for processing".to_string(),
            example: Some(serde_json::json!("Hello, world!")),
        },

        output_spec: InputOutputSpec {
            format: "text".to_string(),
            shape: vec!["variable".to_string()],
            dtype: "string".to_string(),
            description: "Generated text response".to_string(),
            example: Some(serde_json::json!("Hello! How can I help you today?")),
        },

        benchmarks: vec![
            Benchmark {
                name: "MMLU".to_string(),
                dataset: "Massive Multitask Language Understanding".to_string(),
                metric: "accuracy".to_string(),
                value: 0.85,
                units: "score".to_string(),
                hardware: "A100 GPU".to_string(),
            }
        ],

        hardware_requirements: HardwareRequirements {
            min_memory_gb: 16.0,
            recommended_memory_gb: 32.0,
            gpu_required: true,
            min_gpu_memory_gb: Some(8.0),
            supported_platforms: vec![
                "linux-x86_64".to_string(),
                "macos-arm64".to_string(),
            ],
        },

        examples: vec![
            UsageExample {
                title: "Basic Chat".to_string(),
                description: "Simple conversational interaction".to_string(),
                input: serde_json::json!({"text": "What is AI?"}),
                output: serde_json::json!({"text": "AI stands for Artificial Intelligence..."}),
                code: Some("model.predict(\"What is AI?\")".to_string()),
            }
        ],

        documentation_url: Some("https://docs.citrate.ai/models".to_string()),
        paper_url: None,
        thumbnail_url: None,
        demo_url: None,
        pricing_notes: Some("Pay per inference".to_string()),

        created_at: chrono::Utc::now().to_rfc3339(),
        updated_at: chrono::Utc::now().to_rfc3339(),
        ipfs_hash: "QmExampleHash123".to_string(),
    }
}