use crate::types::{ModelId, ModelMetadata, ExecutionRequest, RequestId, RequestStatus};
use lattice_execution::{Address, Hash};
use lattice_storage::StorageManager;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use anyhow::Result;
use tracing::{info, debug};

/// Model registry for tracking AI models
pub struct ModelRegistry {
    storage: Arc<StorageManager>,
    models: Arc<RwLock<HashMap<ModelId, ModelRecord>>>,
    providers: Arc<RwLock<HashMap<ModelId, Vec<Address>>>>,
    requests: Arc<RwLock<HashMap<RequestId, ExecutionRequest>>>,
}

#[derive(Debug, Clone)]
pub struct ModelRecord {
    pub metadata: ModelMetadata,
    pub providers: Vec<Address>,
    pub created_at: u64,
    pub total_executions: u64,
    pub average_latency: u64,
    pub success_rate: f64,
}

impl ModelRegistry {
    pub fn new(storage: Arc<StorageManager>) -> Self {
        Self {
            storage,
            models: Arc::new(RwLock::new(HashMap::new())),
            providers: Arc::new(RwLock::new(HashMap::new())),
            requests: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Register a new model
    pub async fn register(
        &self,
        metadata: ModelMetadata,
        providers: Vec<Address>,
    ) -> Result<ModelId> {
        // Validate metadata
        self.validate_metadata(&metadata)?;
        
        // Generate model ID from hash
        let model_id = ModelId::from_hash(&metadata.hash);
        
        // Check if model already exists
        if self.models.read().await.contains_key(&model_id) {
            return Err(anyhow::anyhow!("Model already registered"));
        }
        
        // Create model record
        let record = ModelRecord {
            metadata: metadata.clone(),
            providers: providers.clone(),
            created_at: chrono::Utc::now().timestamp() as u64,
            total_executions: 0,
            average_latency: 0,
            success_rate: 100.0,
        };
        
        // Store in memory
        self.models.write().await.insert(model_id, record);
        self.providers.write().await.insert(model_id, providers);
        
        // Persist to storage
        self.persist_model(&model_id, &metadata).await?;
        
        info!("Model registered: {:?} with {} providers", 
              hex::encode(&model_id.0[..8]), 
              self.providers.read().await.get(&model_id).map(|p| p.len()).unwrap_or(0));
        
        Ok(model_id)
    }
    
    /// Get model metadata
    pub async fn get_model(&self, model_id: &ModelId) -> Result<ModelMetadata> {
        self.models
            .read()
            .await
            .get(model_id)
            .map(|r| r.metadata.clone())
            .ok_or_else(|| anyhow::anyhow!("Model not found"))
    }
    
    /// Get providers for a model
    pub async fn get_providers(&self, model_id: &ModelId) -> Result<Vec<Address>> {
        self.providers
            .read()
            .await
            .get(model_id)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("No providers for model"))
    }
    
    /// Create an execution request
    pub async fn create_request(
        &self,
        model_id: ModelId,
        input_hash: Hash,
        requester: Address,
        max_price: primitive_types::U256,
    ) -> Result<RequestId> {
        // Check model exists
        if !self.models.read().await.contains_key(&model_id) {
            return Err(anyhow::anyhow!("Model not found"));
        }
        
        // Select provider (simple round-robin for now)
        let providers = self.get_providers(&model_id).await?;
        if providers.is_empty() {
            return Err(anyhow::anyhow!("No providers available"));
        }
        
        let provider = providers[0]; // Simple selection
        
        // Generate request ID
        let request_id = self.generate_request_id(&model_id, &input_hash);
        
        // Create request
        let request = ExecutionRequest {
            id: request_id,
            model_id,
            input_hash,
            requester,
            provider,
            max_price,
            status: RequestStatus::Pending,
            created_at: chrono::Utc::now().timestamp() as u64,
        };
        
        // Store request
        self.requests.write().await.insert(request_id, request);
        
        debug!("Execution request created: {:?}", hex::encode(&request_id.0[..8]));
        
        Ok(request_id)
    }
    
    /// Update request status
    pub async fn update_request_status(
        &self,
        request_id: RequestId,
        status: RequestStatus,
    ) -> Result<()> {
        let mut requests = self.requests.write().await;
        let request = requests
            .get_mut(&request_id)
            .ok_or_else(|| anyhow::anyhow!("Request not found"))?;
        
        request.status = status;
        
        // Update model statistics if completed
        if let RequestStatus::Completed(_) = request.status {
            if let Some(record) = self.models.write().await.get_mut(&request.model_id) {
                record.total_executions += 1;
            }
        }
        
        Ok(())
    }
    
    /// Validate model metadata
    fn validate_metadata(&self, metadata: &ModelMetadata) -> Result<()> {
        if metadata.name.is_empty() {
            return Err(anyhow::anyhow!("Model name cannot be empty"));
        }
        
        if metadata.size == 0 {
            return Err(anyhow::anyhow!("Model size cannot be zero"));
        }
        
        if metadata.compute_requirements.min_memory == 0 {
            return Err(anyhow::anyhow!("Minimum memory requirement cannot be zero"));
        }
        
        Ok(())
    }
    
    /// Persist model to storage
    async fn persist_model(&self, model_id: &ModelId, metadata: &ModelMetadata) -> Result<()> {
        // Serialize metadata
        let data = bincode::serialize(metadata)?;
        
        // Store in database directly with prefix using the state column family
        let key = format!("mcp:model:{}", hex::encode(model_id.as_bytes()));
        self.storage.db.put_cf("state", key.as_bytes(), &data)?;
        
        Ok(())
    }
    
    /// Generate request ID
    fn generate_request_id(&self, model_id: &ModelId, input_hash: &Hash) -> RequestId {
        use sha3::{Digest, Sha3_256};
        
        let mut hasher = Sha3_256::new();
        hasher.update(model_id.as_bytes());
        hasher.update(input_hash.as_bytes());
        hasher.update(&chrono::Utc::now().timestamp().to_le_bytes());
        
        let hash = hasher.finalize();
        let mut id = [0u8; 32];
        id.copy_from_slice(&hash[..32]);
        
        RequestId(id)
    }
}