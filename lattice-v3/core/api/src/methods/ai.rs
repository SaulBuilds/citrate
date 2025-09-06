use crate::types::error::ApiError;
use lattice_execution::types::{ModelId, ModelState, TrainingJob, JobId};
use lattice_storage::StorageManager;
use std::sync::Arc;

/// AI/ML-related API methods
pub struct AiApi {
    storage: Arc<StorageManager>,
}

impl AiApi {
    pub fn new(storage: Arc<StorageManager>) -> Self {
        Self { storage }
    }
    
    /// Get model information
    pub async fn get_model(&self, model_id: ModelId) -> Result<ModelState, ApiError> {
        self.storage.state
            .get_model(&model_id)
            .map_err(|e| ApiError::InternalError(e.to_string()))?
            .ok_or_else(|| ApiError::ModelNotFound(format!("{:?}", model_id)))
    }
    
    /// List registered models
    pub async fn list_models(&self, owner: Option<lattice_execution::types::Address>) -> Result<Vec<ModelId>, ApiError> {
        if let Some(addr) = owner {
            self.storage.state
                .get_models_by_owner(&addr)
                .map_err(|e| ApiError::InternalError(e.to_string()))
        } else {
            // Would need to implement a method to list all models
            Ok(Vec::new())
        }
    }
    
    /// Submit inference request
    pub async fn submit_inference(
        &self,
        model_id: ModelId,
        input_data: Vec<u8>,
    ) -> Result<lattice_consensus::types::Hash, ApiError> {
        // This would create an inference transaction and submit it
        // For now, return a placeholder hash
        Ok(lattice_consensus::types::Hash::default())
    }
    
    /// Get training job status
    pub async fn get_training_job(&self, job_id: JobId) -> Result<TrainingJob, ApiError> {
        self.storage.state
            .get_training_job(&job_id)
            .map_err(|e| ApiError::InternalError(e.to_string()))?
            .ok_or_else(|| ApiError::InternalError(format!("Training job {:?} not found", job_id)))
    }
    
    /// List training jobs
    pub async fn list_training_jobs(&self, model_id: Option<ModelId>) -> Result<Vec<JobId>, ApiError> {
        // Would need to implement filtering by model
        Ok(Vec::new())
    }
}