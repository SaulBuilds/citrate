//! Image Model Module
//!
//! Provides image generation and model training capabilities using Stable Diffusion
//! and other image models. Supports local GGUF models and remote API providers.
//!
//! ## Architecture
//!
//! ```text
//! Image Model System:
//! ├── Model Registry (local/remote models)
//! ├── Image Generator (inference engine)
//! ├── Training Manager (fine-tuning jobs)
//! └── Gallery Manager (generated images)
//! ```

use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};

// ============================================================================
// Types
// ============================================================================

/// Image model information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageModel {
    /// Unique model identifier
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Model type
    pub model_type: ImageModelType,
    /// Model architecture
    pub architecture: ImageArchitecture,
    /// Description
    pub description: String,
    /// File path (for local models)
    pub path: Option<String>,
    /// File size in bytes
    pub size_bytes: u64,
    /// Resolution capabilities
    pub supported_resolutions: Vec<ImageResolution>,
    /// Model version
    pub version: String,
    /// Whether model is downloaded
    pub is_downloaded: bool,
    /// Last used timestamp
    pub last_used: Option<u64>,
    /// Creation timestamp
    pub created_at: u64,
}

/// Image model types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ImageModelType {
    /// Text-to-image generation
    TextToImage,
    /// Image-to-image transformation
    ImageToImage,
    /// Inpainting/outpainting
    Inpainting,
    /// Upscaling
    Upscaler,
    /// Control models (ControlNet, IP-Adapter)
    ControlModel,
}

/// Image generation architecture
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ImageArchitecture {
    /// Stable Diffusion 1.x
    StableDiffusion1,
    /// Stable Diffusion 2.x
    StableDiffusion2,
    /// Stable Diffusion XL
    SDXL,
    /// Stable Diffusion 3
    SD3,
    /// Flux models
    Flux,
    /// DALL-E style
    DALLE,
    /// Custom/other
    Custom,
}

/// Image resolution preset
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImageResolution {
    pub width: u32,
    pub height: u32,
}

impl ImageResolution {
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }

    /// Common presets
    pub fn square_512() -> Self {
        Self::new(512, 512)
    }

    pub fn square_768() -> Self {
        Self::new(768, 768)
    }

    pub fn square_1024() -> Self {
        Self::new(1024, 1024)
    }

    pub fn landscape_16_9() -> Self {
        Self::new(1920, 1080)
    }

    pub fn portrait_9_16() -> Self {
        Self::new(1080, 1920)
    }
}

/// Image generation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageGenerationRequest {
    /// Model to use
    pub model_id: String,
    /// Text prompt
    pub prompt: String,
    /// Negative prompt
    pub negative_prompt: Option<String>,
    /// Output resolution
    pub resolution: ImageResolution,
    /// Number of images to generate
    pub num_images: u32,
    /// Random seed (for reproducibility)
    pub seed: Option<u64>,
    /// Guidance scale (classifier-free guidance)
    pub guidance_scale: f32,
    /// Number of inference steps
    pub num_steps: u32,
    /// Scheduler type
    pub scheduler: Scheduler,
    /// Optional input image for img2img
    pub input_image: Option<String>,
    /// Denoising strength for img2img
    pub strength: Option<f32>,
    /// LoRA weights to apply
    pub lora_weights: Vec<LoRAWeight>,
}

impl Default for ImageGenerationRequest {
    fn default() -> Self {
        Self {
            model_id: String::new(),
            prompt: String::new(),
            negative_prompt: None,
            resolution: ImageResolution::square_512(),
            num_images: 1,
            seed: None,
            guidance_scale: 7.5,
            num_steps: 30,
            scheduler: Scheduler::EulerAncestral,
            input_image: None,
            strength: None,
            lora_weights: vec![],
        }
    }
}

/// LoRA weight configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoRAWeight {
    /// LoRA adapter path or ID
    pub adapter_id: String,
    /// Weight multiplier (0.0 - 2.0)
    pub weight: f32,
}

/// Scheduler/sampler types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Scheduler {
    /// Euler Ancestral
    EulerAncestral,
    /// Euler
    Euler,
    /// DPM++ 2M Karras
    DPMPlusPlus2MKarras,
    /// DPM++ SDE Karras
    DPMPlusPlusSDEKarras,
    /// DDIM
    DDIM,
    /// PNDM
    PNDM,
    /// LMS
    LMS,
}

/// Generated image result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedImage {
    /// Unique image identifier
    pub id: String,
    /// Generation request that produced this
    pub request: ImageGenerationRequest,
    /// Image data (base64 encoded PNG)
    pub image_data: String,
    /// File path (if saved locally)
    pub file_path: Option<String>,
    /// Generation timestamp
    pub generated_at: u64,
    /// Generation time in milliseconds
    pub generation_time_ms: u64,
    /// IPFS CID (if uploaded)
    pub ipfs_cid: Option<String>,
}

/// Image generation status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GenerationStatus {
    /// Queued for processing
    Queued,
    /// Currently generating
    Generating {
        progress: f32,
        current_step: u32,
        total_steps: u32,
    },
    /// Successfully completed
    Completed {
        images: Vec<GeneratedImage>,
    },
    /// Failed with error
    Failed {
        error: String,
    },
    /// Cancelled
    Cancelled,
}

/// Generation job
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationJob {
    /// Unique job identifier
    pub id: String,
    /// Generation request
    pub request: ImageGenerationRequest,
    /// Current status
    pub status: GenerationStatus,
    /// Created timestamp
    pub created_at: u64,
    /// Completed timestamp
    pub completed_at: Option<u64>,
}

/// Image training job configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageTrainingConfig {
    /// Base model to fine-tune
    pub base_model_id: String,
    /// Training dataset path
    pub dataset_path: String,
    /// Instance prompt (for Dreambooth)
    pub instance_prompt: String,
    /// Class prompt (for prior preservation)
    pub class_prompt: Option<String>,
    /// Number of training steps
    pub training_steps: u32,
    /// Learning rate
    pub learning_rate: f64,
    /// Batch size
    pub batch_size: u32,
    /// Use LoRA training
    pub use_lora: bool,
    /// LoRA rank (if using LoRA)
    pub lora_rank: u32,
    /// Output directory
    pub output_dir: String,
    /// Resolution to train at
    pub resolution: ImageResolution,
    /// Use gradient checkpointing
    pub gradient_checkpointing: bool,
    /// Use mixed precision
    pub mixed_precision: bool,
}

impl Default for ImageTrainingConfig {
    fn default() -> Self {
        Self {
            base_model_id: String::new(),
            dataset_path: String::new(),
            instance_prompt: String::new(),
            class_prompt: None,
            training_steps: 1000,
            learning_rate: 1e-4,
            batch_size: 1,
            use_lora: true,
            lora_rank: 8,
            output_dir: String::new(),
            resolution: ImageResolution::square_512(),
            gradient_checkpointing: true,
            mixed_precision: true,
        }
    }
}

/// Image training job status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrainingStatus {
    /// Preparing dataset
    Preparing,
    /// Training in progress
    Training {
        current_step: u32,
        total_steps: u32,
        loss: f64,
        eta_seconds: u64,
    },
    /// Saving model
    Saving,
    /// Completed
    Completed {
        output_path: String,
        final_loss: f64,
    },
    /// Failed
    Failed {
        error: String,
    },
    /// Cancelled
    Cancelled,
}

/// Image training job
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageTrainingJob {
    /// Unique job identifier
    pub id: String,
    /// Training configuration
    pub config: ImageTrainingConfig,
    /// Current status
    pub status: TrainingStatus,
    /// Created timestamp
    pub created_at: u64,
    /// Completed timestamp
    pub completed_at: Option<u64>,
}

// ============================================================================
// Image Model Manager
// ============================================================================

/// Manages image models, generation, and training
pub struct ImageModelManager {
    /// Registered models
    models: Arc<RwLock<HashMap<String, ImageModel>>>,
    /// Active generation jobs
    generation_jobs: Arc<RwLock<HashMap<String, GenerationJob>>>,
    /// Active training jobs
    training_jobs: Arc<RwLock<HashMap<String, ImageTrainingJob>>>,
    /// Generated image gallery
    gallery: Arc<RwLock<Vec<GeneratedImage>>>,
    /// Models directory
    models_dir: PathBuf,
    /// Output directory
    output_dir: PathBuf,
}

impl ImageModelManager {
    /// Create a new image model manager
    pub fn new() -> Self {
        let home_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        let models_dir = home_dir.join(".citrate").join("image_models");
        let output_dir = home_dir.join(".citrate").join("generated_images");

        // Create directories
        let _ = std::fs::create_dir_all(&models_dir);
        let _ = std::fs::create_dir_all(&output_dir);

        // Initialize default models synchronously (no tokio runtime needed)
        let mut default_models = HashMap::new();
        Self::add_default_models(&mut default_models);

        Self {
            models: Arc::new(RwLock::new(default_models)),
            generation_jobs: Arc::new(RwLock::new(HashMap::new())),
            training_jobs: Arc::new(RwLock::new(HashMap::new())),
            gallery: Arc::new(RwLock::new(Vec::new())),
            models_dir,
            output_dir,
        }
    }

    fn add_default_models(models: &mut HashMap<String, ImageModel>) {
        // Stable Diffusion 1.5
        models.insert(
            "sd-1.5".to_string(),
            ImageModel {
                id: "sd-1.5".to_string(),
                name: "Stable Diffusion 1.5".to_string(),
                model_type: ImageModelType::TextToImage,
                architecture: ImageArchitecture::StableDiffusion1,
                description: "Classic Stable Diffusion 1.5 model, great for general purpose generation".to_string(),
                path: None,
                size_bytes: 4_000_000_000, // ~4GB
                supported_resolutions: vec![
                    ImageResolution::square_512(),
                    ImageResolution::new(512, 768),
                    ImageResolution::new(768, 512),
                ],
                version: "1.5".to_string(),
                is_downloaded: false,
                last_used: None,
                created_at: Utc::now().timestamp() as u64,
            },
        );

        // SDXL
        models.insert(
            "sdxl-base".to_string(),
            ImageModel {
                id: "sdxl-base".to_string(),
                name: "Stable Diffusion XL Base".to_string(),
                model_type: ImageModelType::TextToImage,
                architecture: ImageArchitecture::SDXL,
                description: "High resolution SDXL model for detailed image generation".to_string(),
                path: None,
                size_bytes: 6_500_000_000, // ~6.5GB
                supported_resolutions: vec![
                    ImageResolution::square_1024(),
                    ImageResolution::new(1024, 768),
                    ImageResolution::new(768, 1024),
                    ImageResolution::new(1216, 832),
                    ImageResolution::new(832, 1216),
                ],
                version: "1.0".to_string(),
                is_downloaded: false,
                last_used: None,
                created_at: Utc::now().timestamp() as u64,
            },
        );

        // SDXL Refiner
        models.insert(
            "sdxl-refiner".to_string(),
            ImageModel {
                id: "sdxl-refiner".to_string(),
                name: "Stable Diffusion XL Refiner".to_string(),
                model_type: ImageModelType::ImageToImage,
                architecture: ImageArchitecture::SDXL,
                description: "SDXL refiner for enhancing generated images".to_string(),
                path: None,
                size_bytes: 6_000_000_000, // ~6GB
                supported_resolutions: vec![
                    ImageResolution::square_1024(),
                ],
                version: "1.0".to_string(),
                is_downloaded: false,
                last_used: None,
                created_at: Utc::now().timestamp() as u64,
            },
        );

        // Upscaler
        models.insert(
            "real-esrgan-4x".to_string(),
            ImageModel {
                id: "real-esrgan-4x".to_string(),
                name: "Real-ESRGAN 4x".to_string(),
                model_type: ImageModelType::Upscaler,
                architecture: ImageArchitecture::Custom,
                description: "4x upscaler for enhancing image resolution".to_string(),
                path: None,
                size_bytes: 67_000_000, // ~67MB
                supported_resolutions: vec![], // Dynamic
                version: "4.0".to_string(),
                is_downloaded: false,
                last_used: None,
                created_at: Utc::now().timestamp() as u64,
            },
        );

        info!("Initialized {} default image models", models.len());
    }

    /// Get all registered models
    pub async fn get_models(&self) -> Vec<ImageModel> {
        self.models.read().await.values().cloned().collect()
    }

    /// Get a specific model by ID
    pub async fn get_model(&self, model_id: &str) -> Option<ImageModel> {
        self.models.read().await.get(model_id).cloned()
    }

    /// Register a new model
    pub async fn register_model(&self, model: ImageModel) -> Result<(), String> {
        let mut models = self.models.write().await;
        if models.contains_key(&model.id) {
            return Err(format!("Model {} already exists", model.id));
        }
        info!("Registered image model: {}", model.name);
        models.insert(model.id.clone(), model);
        Ok(())
    }

    /// Scan local models directory
    pub async fn scan_local_models(&self) -> Vec<ImageModel> {
        let mut found_models = Vec::new();

        if let Ok(entries) = std::fs::read_dir(&self.models_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    let extension = path.extension().and_then(|e| e.to_str());
                    if matches!(extension, Some("safetensors") | Some("ckpt") | Some("pt")) {
                        let file_name = path.file_stem()
                            .and_then(|n| n.to_str())
                            .unwrap_or("unknown");

                        let model = ImageModel {
                            id: format!("local-{}", file_name),
                            name: file_name.to_string(),
                            model_type: ImageModelType::TextToImage,
                            architecture: ImageArchitecture::Custom,
                            description: format!("Local model from {}", path.display()),
                            path: Some(path.to_string_lossy().to_string()),
                            size_bytes: path.metadata().map(|m| m.len()).unwrap_or(0),
                            supported_resolutions: vec![ImageResolution::square_512()],
                            version: "local".to_string(),
                            is_downloaded: true,
                            last_used: None,
                            created_at: Utc::now().timestamp() as u64,
                        };
                        found_models.push(model);
                    }
                }
            }
        }

        // Register found models
        let mut models = self.models.write().await;
        for model in &found_models {
            if !models.contains_key(&model.id) {
                models.insert(model.id.clone(), model.clone());
            }
        }

        found_models
    }

    /// Create a generation job
    pub async fn create_generation_job(&self, request: ImageGenerationRequest) -> Result<String, String> {
        // Validate model exists
        if self.get_model(&request.model_id).await.is_none() {
            return Err(format!("Model {} not found", request.model_id));
        }

        let job_id = uuid::Uuid::new_v4().to_string();
        let job = GenerationJob {
            id: job_id.clone(),
            request,
            status: GenerationStatus::Queued,
            created_at: Utc::now().timestamp() as u64,
            completed_at: None,
        };

        self.generation_jobs.write().await.insert(job_id.clone(), job);
        info!("Created image generation job: {}", job_id);

        Ok(job_id)
    }

    /// Get generation job status
    pub async fn get_generation_job(&self, job_id: &str) -> Option<GenerationJob> {
        self.generation_jobs.read().await.get(job_id).cloned()
    }

    /// Get all generation jobs
    pub async fn get_generation_jobs(&self) -> Vec<GenerationJob> {
        self.generation_jobs.read().await.values().cloned().collect()
    }

    /// Cancel a generation job
    pub async fn cancel_generation_job(&self, job_id: &str) -> Result<(), String> {
        let mut jobs = self.generation_jobs.write().await;
        if let Some(job) = jobs.get_mut(job_id) {
            match &job.status {
                GenerationStatus::Queued | GenerationStatus::Generating { .. } => {
                    job.status = GenerationStatus::Cancelled;
                    info!("Cancelled generation job: {}", job_id);
                    Ok(())
                }
                _ => Err("Job cannot be cancelled in current state".to_string()),
            }
        } else {
            Err(format!("Job {} not found", job_id))
        }
    }

    /// Create a training job
    pub async fn create_training_job(&self, config: ImageTrainingConfig) -> Result<String, String> {
        // Validate base model exists
        if self.get_model(&config.base_model_id).await.is_none() {
            return Err(format!("Base model {} not found", config.base_model_id));
        }

        // Validate dataset path
        if !std::path::Path::new(&config.dataset_path).exists() {
            return Err(format!("Dataset path {} does not exist", config.dataset_path));
        }

        let job_id = uuid::Uuid::new_v4().to_string();
        let job = ImageTrainingJob {
            id: job_id.clone(),
            config,
            status: TrainingStatus::Preparing,
            created_at: Utc::now().timestamp() as u64,
            completed_at: None,
        };

        self.training_jobs.write().await.insert(job_id.clone(), job);
        info!("Created image training job: {}", job_id);

        Ok(job_id)
    }

    /// Get training job status
    pub async fn get_training_job(&self, job_id: &str) -> Option<ImageTrainingJob> {
        self.training_jobs.read().await.get(job_id).cloned()
    }

    /// Get all training jobs
    pub async fn get_training_jobs(&self) -> Vec<ImageTrainingJob> {
        self.training_jobs.read().await.values().cloned().collect()
    }

    /// Cancel a training job
    pub async fn cancel_training_job(&self, job_id: &str) -> Result<(), String> {
        let mut jobs = self.training_jobs.write().await;
        if let Some(job) = jobs.get_mut(job_id) {
            match &job.status {
                TrainingStatus::Preparing | TrainingStatus::Training { .. } => {
                    job.status = TrainingStatus::Cancelled;
                    info!("Cancelled training job: {}", job_id);
                    Ok(())
                }
                _ => Err("Job cannot be cancelled in current state".to_string()),
            }
        } else {
            Err(format!("Job {} not found", job_id))
        }
    }

    /// Get generated images gallery
    pub async fn get_gallery(&self) -> Vec<GeneratedImage> {
        self.gallery.read().await.clone()
    }

    /// Add image to gallery
    pub async fn add_to_gallery(&self, image: GeneratedImage) {
        self.gallery.write().await.push(image);
    }

    /// Delete image from gallery
    pub async fn delete_from_gallery(&self, image_id: &str) -> Result<(), String> {
        let mut gallery = self.gallery.write().await;
        if let Some(pos) = gallery.iter().position(|img| img.id == image_id) {
            let image = gallery.remove(pos);
            // Delete file if exists
            if let Some(path) = &image.file_path {
                let _ = std::fs::remove_file(path);
            }
            Ok(())
        } else {
            Err(format!("Image {} not found", image_id))
        }
    }

    /// Get models directory
    pub fn get_models_dir(&self) -> &PathBuf {
        &self.models_dir
    }

    /// Get output directory
    pub fn get_output_dir(&self) -> &PathBuf {
        &self.output_dir
    }

    /// Simulate generation (for testing/demo)
    pub async fn simulate_generation(&self, job_id: &str) -> Result<(), String> {
        let mut jobs = self.generation_jobs.write().await;
        let job = jobs.get_mut(job_id)
            .ok_or_else(|| format!("Job {} not found", job_id))?;

        // Simulate generation progress
        job.status = GenerationStatus::Generating {
            progress: 0.0,
            current_step: 0,
            total_steps: job.request.num_steps,
        };

        // In real implementation, this would spawn async task for actual generation
        // For now, simulate completion
        let images = vec![GeneratedImage {
            id: uuid::Uuid::new_v4().to_string(),
            request: job.request.clone(),
            image_data: "".to_string(), // Would be base64 encoded image
            file_path: None,
            generated_at: Utc::now().timestamp() as u64,
            generation_time_ms: 5000,
            ipfs_cid: None,
        }];

        job.status = GenerationStatus::Completed { images };
        job.completed_at = Some(Utc::now().timestamp() as u64);

        Ok(())
    }
}

impl Default for ImageModelManager {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_image_resolution_presets() {
        let r512 = ImageResolution::square_512();
        assert_eq!(r512.width, 512);
        assert_eq!(r512.height, 512);

        let r1024 = ImageResolution::square_1024();
        assert_eq!(r1024.width, 1024);
        assert_eq!(r1024.height, 1024);

        let landscape = ImageResolution::landscape_16_9();
        assert_eq!(landscape.width, 1920);
        assert_eq!(landscape.height, 1080);
    }

    #[test]
    fn test_default_generation_request() {
        let req = ImageGenerationRequest::default();
        assert_eq!(req.num_images, 1);
        assert_eq!(req.guidance_scale, 7.5);
        assert_eq!(req.num_steps, 30);
        assert!(req.lora_weights.is_empty());
    }

    #[test]
    fn test_default_training_config() {
        let config = ImageTrainingConfig::default();
        assert_eq!(config.training_steps, 1000);
        assert!(config.use_lora);
        assert_eq!(config.lora_rank, 8);
        assert!(config.gradient_checkpointing);
    }

    #[tokio::test]
    async fn test_image_model_manager_new() {
        let manager = ImageModelManager::new();
        // Give time for async initialization
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        let models = manager.get_models().await;
        assert!(!models.is_empty());
    }

    #[tokio::test]
    async fn test_get_default_models() {
        let manager = ImageModelManager::new();
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        let sd15 = manager.get_model("sd-1.5").await;
        assert!(sd15.is_some());
        assert_eq!(sd15.unwrap().architecture, ImageArchitecture::StableDiffusion1);

        let sdxl = manager.get_model("sdxl-base").await;
        assert!(sdxl.is_some());
        assert_eq!(sdxl.unwrap().architecture, ImageArchitecture::SDXL);
    }

    #[tokio::test]
    async fn test_register_model() {
        let manager = ImageModelManager::new();

        let model = ImageModel {
            id: "test-model".to_string(),
            name: "Test Model".to_string(),
            model_type: ImageModelType::TextToImage,
            architecture: ImageArchitecture::Custom,
            description: "Test".to_string(),
            path: None,
            size_bytes: 1000,
            supported_resolutions: vec![ImageResolution::square_512()],
            version: "1.0".to_string(),
            is_downloaded: false,
            last_used: None,
            created_at: 0,
        };

        let result = manager.register_model(model.clone()).await;
        assert!(result.is_ok());

        let retrieved = manager.get_model("test-model").await;
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, "Test Model");
    }

    #[tokio::test]
    async fn test_create_generation_job() {
        let manager = ImageModelManager::new();
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        let request = ImageGenerationRequest {
            model_id: "sd-1.5".to_string(),
            prompt: "A beautiful sunset".to_string(),
            ..Default::default()
        };

        let result = manager.create_generation_job(request).await;
        assert!(result.is_ok());

        let job_id = result.unwrap();
        let job = manager.get_generation_job(&job_id).await;
        assert!(job.is_some());
        assert!(matches!(job.unwrap().status, GenerationStatus::Queued));
    }

    #[tokio::test]
    async fn test_create_generation_job_invalid_model() {
        let manager = ImageModelManager::new();

        let request = ImageGenerationRequest {
            model_id: "nonexistent-model".to_string(),
            prompt: "Test".to_string(),
            ..Default::default()
        };

        let result = manager.create_generation_job(request).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_cancel_generation_job() {
        let manager = ImageModelManager::new();
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        let request = ImageGenerationRequest {
            model_id: "sd-1.5".to_string(),
            prompt: "Test".to_string(),
            ..Default::default()
        };

        let job_id = manager.create_generation_job(request).await.unwrap();
        let result = manager.cancel_generation_job(&job_id).await;
        assert!(result.is_ok());

        let job = manager.get_generation_job(&job_id).await.unwrap();
        assert!(matches!(job.status, GenerationStatus::Cancelled));
    }

    #[tokio::test]
    async fn test_get_gallery() {
        let manager = ImageModelManager::new();
        let gallery = manager.get_gallery().await;
        assert!(gallery.is_empty());
    }

    #[tokio::test]
    async fn test_add_to_gallery() {
        let manager = ImageModelManager::new();

        let image = GeneratedImage {
            id: "test-image".to_string(),
            request: ImageGenerationRequest::default(),
            image_data: "base64data".to_string(),
            file_path: None,
            generated_at: 0,
            generation_time_ms: 1000,
            ipfs_cid: None,
        };

        manager.add_to_gallery(image).await;
        let gallery = manager.get_gallery().await;
        assert_eq!(gallery.len(), 1);
    }

    #[tokio::test]
    async fn test_delete_from_gallery() {
        let manager = ImageModelManager::new();

        let image = GeneratedImage {
            id: "test-image".to_string(),
            request: ImageGenerationRequest::default(),
            image_data: "base64data".to_string(),
            file_path: None,
            generated_at: 0,
            generation_time_ms: 1000,
            ipfs_cid: None,
        };

        manager.add_to_gallery(image).await;
        let result = manager.delete_from_gallery("test-image").await;
        assert!(result.is_ok());

        let gallery = manager.get_gallery().await;
        assert!(gallery.is_empty());
    }

    #[test]
    fn test_scheduler_variants() {
        let schedulers = [
            Scheduler::EulerAncestral,
            Scheduler::Euler,
            Scheduler::DPMPlusPlus2MKarras,
            Scheduler::DDIM,
        ];

        for scheduler in schedulers {
            let json = serde_json::to_string(&scheduler).unwrap();
            assert!(!json.is_empty());
        }
    }

    #[test]
    fn test_image_model_type_variants() {
        let types = [
            ImageModelType::TextToImage,
            ImageModelType::ImageToImage,
            ImageModelType::Inpainting,
            ImageModelType::Upscaler,
            ImageModelType::ControlModel,
        ];

        for t in types {
            let json = serde_json::to_string(&t).unwrap();
            assert!(!json.is_empty());
        }
    }
}
