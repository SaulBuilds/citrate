//! IPFS integration for distributed model storage
//!
//! This module provides functionality to store and retrieve AI models
//! using IPFS (InterPlanetary File System) for decentralized storage.

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use reqwest::{self, Client};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub mod chunking;
pub mod pinning;

/// IPFS Content Identifier
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct Cid(pub String);

/// Model metadata stored alongside IPFS content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelMetadata {
    pub name: String,
    pub version: String,
    pub framework: ModelFramework,
    pub model_type: ModelType,
    pub size_bytes: u64,
    pub input_shape: Vec<usize>,
    pub output_shape: Vec<usize>,
    pub description: String,
    pub author: String,
    pub license: String,
    pub created_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModelFramework {
    PyTorch,
    TensorFlow,
    ONNX,
    JAX,
    HuggingFace,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModelType {
    Language,
    Vision,
    Audio,
    Multimodal,
    Reinforcement,
    Custom(String),
}

/// IPFS service for model storage and retrieval
pub struct IPFSService {
    api_endpoint: String,
    client: reqwest::Client,
    pinned_models: HashMap<Cid, ModelMetadata>,
    max_pin_size: u64,
    pinning_manager: pinning::PinningManager,
}

impl IPFSService {
    /// Create a new IPFS service instance
    pub fn new(api_endpoint: String) -> Self {
        let client = Client::builder()
            .use_rustls_tls()
            .no_proxy()
            .build()
            .expect("failed to build IPFS HTTP client");

        Self {
            api_endpoint,
            client,
            pinned_models: HashMap::new(),
            max_pin_size: 10 * 1024 * 1024 * 1024, // 10GB default
            pinning_manager: pinning::PinningManager::new(),
        }
    }

    /// Store a model in IPFS
    pub async fn store_model(
        &mut self,
        model_data: Vec<u8>,
        metadata: ModelMetadata,
    ) -> Result<Cid> {
        // Check size limit
        if model_data.len() as u64 > self.max_pin_size {
            return Err(anyhow!("Model exceeds maximum pin size"));
        }

        // For large models, use chunking
        if model_data.len() > 256 * 1024 * 1024 {
            // 256MB
            return self.store_chunked_model(model_data, metadata).await;
        }

        // Add to IPFS
        let cid = self.add_to_ipfs(model_data).await?;

        // Pin locally
        self.pin_add(&cid).await?;

        // Track in local storage
        let metadata_for_record = metadata.clone();
        let pinned_bytes = metadata_for_record.size_bytes;
        self.pinned_models.insert(cid.clone(), metadata);
        self.pinning_manager.record_pin(
            cid.clone(),
            "local-node".to_string(),
            metadata_for_record,
            pinned_bytes,
        );

        Ok(cid)
    }

    /// Retrieve a model from IPFS
    pub async fn retrieve_model(&self, cid: &Cid) -> Result<Vec<u8>> {
        // Check if pinned locally
        if self.pinned_models.contains_key(cid) {
            // Read from local pinned storage
            self.cat_from_ipfs(cid).await
        } else {
            // Fetch from IPFS network
            self.get_from_ipfs(cid).await
        }
    }

    /// Store a large model using chunking
    async fn store_chunked_model(
        &mut self,
        model_data: Vec<u8>,
        metadata: ModelMetadata,
    ) -> Result<Cid> {
        let chunks = chunking::chunk_model(&model_data, 256 * 1024 * 1024)?;
        let mut chunk_cids = Vec::new();

        // Store each chunk
        for chunk in chunks {
            let cid = self.add_to_ipfs(chunk.data).await?;
            chunk_cids.push(cid);
        }

        // Create manifest
        let manifest = chunking::ChunkManifest {
            chunks: chunk_cids.clone(),
            total_size: model_data.len() as u64,
            chunk_size: 256 * 1024 * 1024,
            metadata: metadata.clone(),
            metal_optimized: false,
            unified_memory_compatible: true,
        };

        // Store manifest
        let manifest_data = serde_json::to_vec(&manifest)?;
        let manifest_cid = self.add_to_ipfs(manifest_data).await?;

        // Pin manifest and chunks
        self.pin_add(&manifest_cid).await?;
        for cid in &chunk_cids {
            self.pin_add(cid).await?;
        }

        let metadata_for_record = metadata.clone();
        let pinned_bytes = metadata_for_record.size_bytes;
        self.pinned_models.insert(manifest_cid.clone(), metadata);
        self.pinning_manager.record_pin(
            manifest_cid.clone(),
            "local-node".to_string(),
            metadata_for_record,
            pinned_bytes,
        );
        Ok(manifest_cid)
    }

    /// Add data to IPFS
    async fn add_to_ipfs(&self, data: Vec<u8>) -> Result<Cid> {
        let url = format!("{}/api/v0/add", self.api_endpoint);

        let part = reqwest::multipart::Part::bytes(data).file_name("model.bin");
        let form = reqwest::multipart::Form::new().part("file", part);

        let response = self.client.post(&url).multipart(form).send().await?;

        #[derive(Deserialize)]
        struct AddResponse {
            #[serde(rename = "Hash")]
            hash: String,
        }

        let result: AddResponse = response.json().await?;
        Ok(Cid(result.hash))
    }

    /// Pin content in IPFS
    async fn pin_add(&self, cid: &Cid) -> Result<()> {
        let url = format!("{}/api/v0/pin/add", self.api_endpoint);

        self.client
            .post(&url)
            .query(&[("arg", &cid.0)])
            .send()
            .await?;

        Ok(())
    }

    /// Get content from IPFS
    async fn get_from_ipfs(&self, cid: &Cid) -> Result<Vec<u8>> {
        let url = format!("{}/api/v0/get", self.api_endpoint);

        let response = self
            .client
            .post(&url)
            .query(&[("arg", &cid.0)])
            .send()
            .await?;

        let bytes = response.bytes().await?;
        Ok(bytes.to_vec())
    }

    /// Cat (read) content from IPFS
    async fn cat_from_ipfs(&self, cid: &Cid) -> Result<Vec<u8>> {
        let url = format!("{}/api/v0/cat", self.api_endpoint);

        let response = self
            .client
            .get(&url)
            .query(&[("arg", &cid.0)])
            .send()
            .await?;

        let bytes = response.bytes().await?;
        Ok(bytes.to_vec())
    }

    /// List all pinned models
    pub fn list_pinned_models(&self) -> Vec<(Cid, ModelMetadata)> {
        self.pinned_models
            .iter()
            .map(|(cid, meta)| (cid.clone(), meta.clone()))
            .collect()
    }

    /// Get metadata for a pinned model
    pub fn get_model_metadata(&self, cid: &Cid) -> Option<&ModelMetadata> {
        self.pinned_models.get(cid)
    }

    /// Calculate storage incentive rewards
    pub fn calculate_pin_reward(&self, cid: &Cid, duration_hours: u64) -> u64 {
        if let Some(metadata) = self.pinned_models.get(cid) {
            return pinning::PinningManager::reward_for_duration(metadata, duration_hours);
        }

        if let Some(summary) = self.pinning_manager.summary(cid) {
            pinning::PinningManager::reward_for_duration(&summary.metadata, duration_hours)
        } else {
            0
        }
    }

    /// Provide pinning incentive statistics for a CID, if recorded.
    pub fn pinning_summary(&self, cid: &Cid) -> Option<pinning::PinningSummary> {
        self.pinning_manager.summary(cid)
    }

    /// Fetch raw bytes for a CID without any caching logic.
    pub async fn fetch_raw(&self, cid: &Cid) -> Result<Vec<u8>> {
        match self.cat_from_ipfs(cid).await {
            Ok(bytes) => Ok(bytes),
            Err(_) => self.get_from_ipfs(cid).await,
        }
    }

    /// Record pinning information for an external provider.
    pub fn record_external_pin(
        &mut self,
        cid: Cid,
        pinner_id: String,
        metadata: ModelMetadata,
        pinned_bytes: u64,
    ) -> pinning::PinReward {
        self.pinned_models
            .entry(cid.clone())
            .or_insert(metadata.clone());
        self.pinning_manager
            .record_pin(cid, pinner_id, metadata, pinned_bytes)
    }
}

/// Trait for IPFS operations
#[async_trait]
pub trait IPFSOperations {
    async fn store(&mut self, data: Vec<u8>, metadata: ModelMetadata) -> Result<Cid>;
    async fn retrieve(&self, cid: &Cid) -> Result<Vec<u8>>;
    async fn pin(&mut self, cid: &Cid) -> Result<()>;
    async fn unpin(&mut self, cid: &Cid) -> Result<()>;
}

#[async_trait]
impl IPFSOperations for IPFSService {
    async fn store(&mut self, data: Vec<u8>, metadata: ModelMetadata) -> Result<Cid> {
        self.store_model(data, metadata).await
    }

    async fn retrieve(&self, cid: &Cid) -> Result<Vec<u8>> {
        self.retrieve_model(cid).await
    }

    async fn pin(&mut self, cid: &Cid) -> Result<()> {
        self.pin_add(cid).await
    }

    async fn unpin(&mut self, cid: &Cid) -> Result<()> {
        let url = format!("{}/api/v0/pin/rm", self.api_endpoint);

        self.client
            .post(&url)
            .query(&[("arg", &cid.0)])
            .send()
            .await?;

        self.pinned_models.remove(cid);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cid_creation() {
        let cid = Cid("QmTest123".to_string());
        assert_eq!(cid.0, "QmTest123");
    }

    #[test]
    fn test_model_metadata() {
        let metadata = ModelMetadata {
            name: "GPT-2".to_string(),
            version: "1.0.0".to_string(),
            framework: ModelFramework::ONNX,
            model_type: ModelType::Language,
            size_bytes: 124_000_000,
            input_shape: vec![1, 512],
            output_shape: vec![1, 512, 50257],
            description: "GPT-2 language model".to_string(),
            author: "OpenAI".to_string(),
            license: "MIT".to_string(),
            created_at: 1234567890,
        };

        assert_eq!(metadata.name, "GPT-2");
        assert_eq!(metadata.size_bytes, 124_000_000);
    }

    #[test]
    fn test_pin_reward_calculation() {
        let mut service = IPFSService::new("http://localhost:5001".to_string());

        let cid = Cid("QmTest".to_string());
        let metadata = ModelMetadata {
            name: "Test Model".to_string(),
            version: "1.0".to_string(),
            framework: ModelFramework::ONNX,
            model_type: ModelType::Vision,
            size_bytes: 1_073_741_824, // 1GB
            input_shape: vec![1, 3, 224, 224],
            output_shape: vec![1, 1000],
            description: "Test".to_string(),
            author: "Test".to_string(),
            license: "MIT".to_string(),
            created_at: 0,
        };

        service.pinned_models.insert(cid.clone(), metadata.clone());
        let reward_info = service.record_external_pin(
            cid.clone(),
            "remote-node".to_string(),
            metadata.clone(),
            metadata.size_bytes,
        );
        assert_eq!(reward_info.reward, 3); // 1GB * multiplier 3
        assert_eq!(reward_info.total_replicas, 1);

        // 1GB model, Vision type (3x multiplier), 24 hours
        let reward = service.calculate_pin_reward(&cid, 24);
        assert_eq!(reward, 3); // 1 GB * 1 day * 3x multiplier
    }

    #[test]
    fn test_pinning_summary_includes_external_reports() {
        let mut service = IPFSService::new("http://localhost:5001".to_string());
        let cid = Cid("QmSummary".to_string());
        let metadata = ModelMetadata {
            name: "Summary Model".to_string(),
            version: "2.0".to_string(),
            framework: ModelFramework::PyTorch,
            model_type: ModelType::Multimodal,
            size_bytes: 2_147_483_648, // 2GB
            input_shape: vec![1, 1, 512],
            output_shape: vec![1, 1, 512],
            description: "Summary test".to_string(),
            author: "Tester".to_string(),
            license: "Apache-2.0".to_string(),
            created_at: 0,
        };

        service.record_external_pin(
            cid.clone(),
            "provider-1".to_string(),
            metadata.clone(),
            metadata.size_bytes,
        );

        let summary = service.pinning_summary(&cid).expect("summary exists");
        assert_eq!(summary.total_replicas, 1);
        assert_eq!(summary.total_pinned_bytes, metadata.size_bytes);
        assert_eq!(summary.metadata.name, "Summary Model");
    }
}
