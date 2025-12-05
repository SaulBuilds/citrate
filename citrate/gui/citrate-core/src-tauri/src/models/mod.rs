use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};

/// Manages AI models in the Citrate network
pub struct ModelManager {
    models: Arc<RwLock<HashMap<String, ModelInfo>>>,
    deployments: Arc<RwLock<Vec<ModelDeployment>>>,
    training_jobs: Arc<RwLock<Vec<TrainingJob>>>,
}

impl ModelManager {
    pub fn new() -> Self {
        Self {
            models: Arc::new(RwLock::new(Self::load_sample_models())),
            deployments: Arc::new(RwLock::new(Vec::new())),
            training_jobs: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Get all registered models
    pub async fn get_models(&self) -> Result<Vec<ModelInfo>> {
        Ok(self.models.read().await.values().cloned().collect())
    }

    /// Get a specific model by ID
    pub async fn get_model(&self, model_id: &str) -> Result<Option<ModelInfo>> {
        Ok(self.models.read().await.get(model_id).cloned())
    }

    /// Deploy a model to the network
    pub async fn deploy_model(&self, deployment: ModelDeployment) -> Result<String> {
        let deployment_id = format!("deploy_{}", chrono::Utc::now().timestamp());

        // Add to deployments
        self.deployments.write().await.push(deployment.clone());

        info!(
            "Deployed model: {} with ID: {}",
            deployment.model_id, deployment_id
        );
        Ok(deployment_id)
    }

    /// Get all deployments
    pub async fn get_deployments(&self) -> Result<Vec<ModelDeployment>> {
        Ok(self.deployments.read().await.clone())
    }

    /// Start a training job
    pub async fn start_training(&self, job: TrainingJob) -> Result<String> {
        let job_id = format!("job_{}", chrono::Utc::now().timestamp());

        // Add to training jobs
        self.training_jobs.write().await.push(job.clone());

        info!("Started training job: {}", job_id);
        Ok(job_id)
    }

    /// Get all training jobs
    pub async fn get_training_jobs(&self) -> Result<Vec<TrainingJob>> {
        Ok(self.training_jobs.read().await.clone())
    }

    /// Get training job status
    pub async fn get_job_status(&self, job_id: &str) -> Result<Option<JobStatus>> {
        let jobs = self.training_jobs.read().await;
        let job = jobs.iter().find(|j| j.id == job_id);
        Ok(job.map(|j| j.status.clone()))
    }

    /// Request inference from a model
    pub async fn request_inference(&self, request: InferenceRequest) -> Result<InferenceResponse> {
        let start = std::time::Instant::now();

        // Resolve model path
        let model_path = self.resolve_model_path(&request.model_id)?;

        // Get inference parameters
        let max_tokens = request.parameters
            .get("max_tokens")
            .and_then(|v| v.as_u64())
            .unwrap_or(512) as usize;
        let temperature = request.parameters
            .get("temperature")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.7) as f32;

        // Run inference using llama.cpp
        let result = self.run_llama_inference(&model_path, &request.input, max_tokens, temperature).await?;

        let latency_ms = start.elapsed().as_millis() as u64;

        Ok(InferenceResponse {
            request_id: format!("inf_{}", chrono::Utc::now().timestamp()),
            model_id: request.model_id,
            result,
            confidence: 0.95,
            latency_ms,
            cost: 0.0, // Free for local inference
        })
    }

    /// Resolve model path from model ID
    fn resolve_model_path(&self, model_id: &str) -> Result<PathBuf> {
        // Handle full paths
        let path = PathBuf::from(model_id);
        if path.exists() && path.extension().map_or(false, |e| e == "gguf") {
            return Ok(path);
        }

        // Map model aliases to filenames
        let model_filename = match model_id {
            "mistral-7b-instruct-v0.3" | "mistral-7b" => "Mistral-7B-Instruct-v0.3-Q4_K_M.gguf",
            "qwen2-0.5b" | "qwen" => "qwen2-0.5b-q4.gguf",
            "bge-m3" => "bge-m3-fp16.gguf",
            other => other,
        };

        // Search in multiple locations
        let home_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        let search_paths = vec![
            PathBuf::from("./models").join(model_filename),
            PathBuf::from("../../../models").join(model_filename),
            home_dir.join("Models").join(model_filename),
            home_dir.join(".citrate/models").join(model_filename),
            home_dir.join(".ipfs/models").join(model_filename),
            // Check project models directory (during dev)
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../models").join(model_filename),
        ];

        for path in &search_paths {
            if path.exists() {
                info!("Found model at: {:?}", path);
                return Ok(path.clone());
            }
        }

        Err(anyhow!(
            "Model '{}' not found. Searched: {:?}",
            model_id,
            search_paths.iter().map(|p| p.display().to_string()).collect::<Vec<_>>()
        ))
    }

    /// Run inference using llama.cpp CLI
    async fn run_llama_inference(
        &self,
        model_path: &PathBuf,
        prompt: &str,
        max_tokens: usize,
        temperature: f32
    ) -> Result<String> {
        // Find llama.cpp binary
        let home_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        let llama_cpp_bin = home_dir.join("llama.cpp/build/bin/llama-cli");
        let llama_cpp_main = home_dir.join("llama.cpp/build/bin/main");

        let binary = if llama_cpp_bin.exists() {
            llama_cpp_bin
        } else if llama_cpp_main.exists() {
            llama_cpp_main
        } else {
            return Err(anyhow!(
                "llama.cpp not found. Please install llama.cpp to ~/llama.cpp"
            ));
        };

        info!(
            "Running inference with model: {:?}, max_tokens: {}, temp: {}",
            model_path, max_tokens, temperature
        );

        // Build command
        let output = tokio::task::spawn_blocking({
            let binary = binary.clone();
            let model_path = model_path.clone();
            let prompt = prompt.to_string();
            let threads = num_cpus::get();

            move || {
                Command::new(&binary)
                    .arg("-m")
                    .arg(&model_path)
                    .arg("-p")
                    .arg(&prompt)
                    .arg("-n")
                    .arg(max_tokens.to_string())
                    .arg("--temp")
                    .arg(temperature.to_string())
                    .arg("-t")
                    .arg(threads.to_string())
                    .arg("-c")
                    .arg("2048")
                    .arg("--no-display-prompt")
                    .output()
            }
        }).await??;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("llama.cpp execution failed: {}", stderr));
        }

        let text = String::from_utf8_lossy(&output.stdout);
        Ok(text.trim().to_string())
    }

    fn load_sample_models() -> HashMap<String, ModelInfo> {
        let mut models = HashMap::new();

        // Add sample models
        models.insert(
            "gpt-citrate".to_string(),
            ModelInfo {
                id: "gpt-citrate".to_string(),
                name: "GPT-Citrate".to_string(),
                description: "Language model optimized for Citrate".to_string(),
                model_type: ModelType::Language,
                version: "1.0.0".to_string(),
                size_mb: 4096,
                parameters: 7_000_000_000,
                architecture: "Transformer".to_string(),
                owner: "0x1234...".to_string(),
                created_at: chrono::Utc::now().timestamp() as u64,
                updated_at: chrono::Utc::now().timestamp() as u64,
                hash: "QmXyz...".to_string(),
                metadata: HashMap::new(),
            },
        );

        models.insert(
            "stable-citrate".to_string(),
            ModelInfo {
                id: "stable-citrate".to_string(),
                name: "Stable-Citrate".to_string(),
                description: "Image generation model".to_string(),
                model_type: ModelType::Image,
                version: "1.0.0".to_string(),
                size_mb: 2048,
                parameters: 1_000_000_000,
                architecture: "Diffusion".to_string(),
                owner: "0x5678...".to_string(),
                created_at: chrono::Utc::now().timestamp() as u64,
                updated_at: chrono::Utc::now().timestamp() as u64,
                hash: "QmAbc...".to_string(),
                metadata: HashMap::new(),
            },
        );

        models
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub model_type: ModelType,
    pub version: String,
    pub size_mb: u64,
    pub parameters: u64,
    pub architecture: String,
    pub owner: String,
    pub created_at: u64,
    pub updated_at: u64,
    pub hash: String,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModelType {
    Language,
    Image,
    Audio,
    Video,
    Multimodal,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelDeployment {
    pub id: String,
    pub model_id: String,
    pub endpoint: String,
    pub status: DeploymentStatus,
    pub replicas: u32,
    pub memory_mb: u64,
    pub cpu_cores: u32,
    pub gpu_count: u32,
    pub created_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeploymentStatus {
    Pending,
    Running,
    Stopped,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingJob {
    pub id: String,
    pub model_id: String,
    pub dataset_id: String,
    pub status: JobStatus,
    pub epochs: u32,
    pub batch_size: u32,
    pub learning_rate: f32,
    pub loss: f32,
    pub accuracy: f32,
    pub started_at: u64,
    pub completed_at: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum JobStatus {
    Queued,
    Running,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceRequest {
    pub model_id: String,
    pub input: String,
    pub parameters: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceResponse {
    pub request_id: String,
    pub model_id: String,
    pub result: String,
    pub confidence: f32,
    pub latency_ms: u64,
    pub cost: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelMetrics {
    pub model_id: String,
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub average_latency_ms: f64,
    pub total_cost: f64,
    pub uptime_seconds: u64,
}
