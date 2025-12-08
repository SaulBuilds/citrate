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
    lora_jobs: Arc<RwLock<HashMap<String, LoraTrainingJob>>>,
    lora_adapters: Arc<RwLock<Vec<LoraAdapterInfo>>>,
    active_lora_processes: Arc<RwLock<HashMap<String, tokio::process::Child>>>,
}

impl ModelManager {
    pub fn new() -> Self {
        Self {
            models: Arc::new(RwLock::new(Self::load_sample_models())),
            deployments: Arc::new(RwLock::new(Vec::new())),
            training_jobs: Arc::new(RwLock::new(Vec::new())),
            lora_jobs: Arc::new(RwLock::new(HashMap::new())),
            lora_adapters: Arc::new(RwLock::new(Vec::new())),
            active_lora_processes: Arc::new(RwLock::new(HashMap::new())),
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

    // ==========================================
    // LoRA Training Methods
    // ==========================================

    /// Create a new LoRA training job
    pub async fn create_lora_job(
        &self,
        base_model_path: String,
        base_model_name: String,
        dataset_path: String,
        dataset_format: DatasetFormat,
        output_dir: String,
        lora_config: Option<LoraConfig>,
        training_config: Option<LoraTrainingConfig>,
    ) -> Result<LoraTrainingJob> {
        // Validate base model exists
        let model_path = PathBuf::from(&base_model_path);
        if !model_path.exists() {
            return Err(anyhow!("Base model not found: {}", base_model_path));
        }

        // Validate dataset exists
        let dataset = PathBuf::from(&dataset_path);
        if !dataset.exists() {
            return Err(anyhow!("Dataset not found: {}", dataset_path));
        }

        // Create output directory if needed
        let output = PathBuf::from(&output_dir);
        if !output.exists() {
            std::fs::create_dir_all(&output)?;
        }

        let job_id = format!("lora_{}_{}",
            chrono::Utc::now().timestamp(),
            uuid::Uuid::new_v4().to_string().split('-').next().unwrap_or("0000")
        );

        let job = LoraTrainingJob {
            id: job_id.clone(),
            base_model_path,
            base_model_name,
            dataset_path,
            dataset_format,
            output_dir,
            lora_config: lora_config.unwrap_or_default(),
            training_config: training_config.unwrap_or_default(),
            status: JobStatus::Queued,
            progress: 0.0,
            current_epoch: 0,
            current_step: 0,
            total_steps: 0,
            train_loss: 0.0,
            val_loss: None,
            metrics_history: Vec::new(),
            error_message: None,
            created_at: chrono::Utc::now().timestamp() as u64,
            started_at: None,
            completed_at: None,
        };

        self.lora_jobs.write().await.insert(job_id.clone(), job.clone());
        info!("Created LoRA training job: {}", job_id);
        Ok(job)
    }

    /// Start a LoRA training job
    pub async fn start_lora_training(&self, job_id: &str) -> Result<()> {
        let mut jobs = self.lora_jobs.write().await;
        let job = jobs.get_mut(job_id)
            .ok_or_else(|| anyhow!("LoRA job not found: {}", job_id))?;

        if !matches!(job.status, JobStatus::Queued) {
            return Err(anyhow!("Job {} is not in queued state", job_id));
        }

        // Calculate total steps based on dataset
        let dataset_lines = self.count_dataset_lines(&job.dataset_path).await?;
        let steps_per_epoch = dataset_lines / job.training_config.batch_size as usize;
        job.total_steps = (steps_per_epoch * job.training_config.epochs as usize) as u64;
        job.status = JobStatus::Running;
        job.started_at = Some(chrono::Utc::now().timestamp() as u64);

        let job_clone = job.clone();
        drop(jobs);

        // Spawn the training process
        self.spawn_lora_training_process(job_clone).await?;

        Ok(())
    }

    /// Spawn the actual training process using llama.cpp finetune
    async fn spawn_lora_training_process(&self, job: LoraTrainingJob) -> Result<()> {
        let home_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));

        // Look for llama.cpp finetune binary
        let finetune_paths = vec![
            home_dir.join("llama.cpp/build/bin/llama-finetune"),
            home_dir.join("llama.cpp/build/bin/finetune"),
            home_dir.join("llama.cpp/finetune"),
            PathBuf::from("/usr/local/bin/llama-finetune"),
        ];

        let finetune_bin = finetune_paths.iter()
            .find(|p| p.exists())
            .ok_or_else(|| anyhow!(
                "llama.cpp finetune not found. Please install llama.cpp with finetune support."
            ))?
            .clone();

        // Build training command arguments
        let output_adapter = PathBuf::from(&job.output_dir)
            .join(format!("{}-lora-ITERATION.bin", job.id));

        let mut cmd = tokio::process::Command::new(&finetune_bin);
        cmd.arg("--model-base").arg(&job.base_model_path)
           .arg("--train-data").arg(&job.dataset_path)
           .arg("--lora-out").arg(&output_adapter)
           .arg("--lora-r").arg(job.lora_config.rank.to_string())
           .arg("--lora-alpha").arg(job.lora_config.alpha.to_string())
           .arg("--ctx").arg(job.training_config.max_seq_length.to_string())
           .arg("--batch").arg(job.training_config.batch_size.to_string())
           .arg("--adam-iter").arg(job.training_config.epochs.to_string())
           .arg("--adam-alpha").arg(job.training_config.learning_rate.to_string())
           .arg("--threads").arg(job.training_config.num_threads.to_string())
           .arg("--seed").arg(job.training_config.seed.to_string())
           .arg("--save-every").arg(job.training_config.save_steps.to_string());

        // Add GPU layers if enabled
        if job.training_config.use_gpu && job.training_config.n_gpu_layers > 0 {
            cmd.arg("--n-gpu-layers").arg(job.training_config.n_gpu_layers.to_string());
        }

        // Configure stdout/stderr capture
        cmd.stdout(std::process::Stdio::piped())
           .stderr(std::process::Stdio::piped());

        let child = cmd.spawn()?;

        // Store the process handle for monitoring
        self.active_lora_processes.write().await.insert(job.id.clone(), child);

        // Spawn background task to monitor progress
        let job_id = job.id.clone();
        let lora_jobs = self.lora_jobs.clone();
        let active_processes = self.active_lora_processes.clone();
        let lora_adapters = self.lora_adapters.clone();
        let output_dir = job.output_dir.clone();
        let base_model = job.base_model_name.clone();
        let lora_config = job.lora_config.clone();

        tokio::spawn(async move {
            Self::monitor_training_progress(
                job_id,
                lora_jobs,
                active_processes,
                lora_adapters,
                output_dir,
                base_model,
                lora_config,
            ).await;
        });

        info!("Started LoRA training process for job: {}", job.id);
        Ok(())
    }

    /// Monitor training progress and update job status
    async fn monitor_training_progress(
        job_id: String,
        lora_jobs: Arc<RwLock<HashMap<String, LoraTrainingJob>>>,
        active_processes: Arc<RwLock<HashMap<String, tokio::process::Child>>>,
        lora_adapters: Arc<RwLock<Vec<LoraAdapterInfo>>>,
        output_dir: String,
        base_model: String,
        lora_config: LoraConfig,
    ) {
        use tokio::io::{AsyncBufReadExt, BufReader};

        // Get the process
        let mut processes = active_processes.write().await;
        let child = match processes.remove(&job_id) {
            Some(c) => c,
            None => {
                warn!("No process found for job {}", job_id);
                return;
            }
        };
        drop(processes);

        let stdout = match child.stdout {
            Some(out) => out,
            None => {
                warn!("No stdout for job {}", job_id);
                return;
            }
        };

        let mut reader = BufReader::new(stdout).lines();
        let mut last_loss = 0.0f32;
        let mut last_step = 0u64;

        while let Ok(Some(line)) = reader.next_line().await {
            // Parse training output for progress
            // llama.cpp finetune outputs lines like: "iter=100, loss=2.345, ..."
            if let Some(loss_str) = line.split("loss=").nth(1) {
                if let Ok(loss) = loss_str.split(',').next().unwrap_or("0").trim().parse::<f32>() {
                    last_loss = loss;
                }
            }
            if let Some(iter_str) = line.split("iter=").nth(1) {
                if let Ok(iter) = iter_str.split(',').next().unwrap_or("0").trim().parse::<u64>() {
                    last_step = iter;
                }
            }

            // Update job progress
            if last_step > 0 {
                let mut jobs = lora_jobs.write().await;
                if let Some(job) = jobs.get_mut(&job_id) {
                    job.current_step = last_step;
                    job.train_loss = last_loss;
                    if job.total_steps > 0 {
                        job.progress = last_step as f32 / job.total_steps as f32;
                    }
                    job.metrics_history.push(TrainingMetricsPoint {
                        step: last_step,
                        epoch: (last_step as f32 / (job.total_steps as f32 / job.training_config.epochs as f32)),
                        train_loss: last_loss,
                        val_loss: None,
                        learning_rate: job.training_config.learning_rate,
                        timestamp: chrono::Utc::now().timestamp() as u64,
                    });
                }
            }
        }

        // Training completed - update job status
        let mut jobs = lora_jobs.write().await;
        if let Some(job) = jobs.get_mut(&job_id) {
            job.status = JobStatus::Completed;
            job.completed_at = Some(chrono::Utc::now().timestamp() as u64);
            job.progress = 1.0;

            // Register the output adapter
            let adapter_path = PathBuf::from(&output_dir);
            if let Ok(entries) = std::fs::read_dir(&adapter_path) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().map_or(false, |e| e == "bin") {
                        let size = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
                        let adapter = LoraAdapterInfo {
                            id: format!("adapter_{}", chrono::Utc::now().timestamp()),
                            name: path.file_stem().unwrap_or_default().to_string_lossy().to_string(),
                            base_model: base_model.clone(),
                            path: path.to_string_lossy().to_string(),
                            size_bytes: size,
                            rank: lora_config.rank,
                            alpha: lora_config.alpha,
                            target_modules: lora_config.target_modules.clone(),
                            created_at: chrono::Utc::now().timestamp() as u64,
                            training_job_id: Some(job_id.clone()),
                            description: None,
                            tags: Vec::new(),
                        };
                        lora_adapters.write().await.push(adapter);
                    }
                }
            }
        }

        info!("LoRA training completed for job: {}", job_id);
    }

    /// Count lines in a dataset file
    async fn count_dataset_lines(&self, path: &str) -> Result<usize> {
        use tokio::io::{AsyncBufReadExt, BufReader};

        let file = tokio::fs::File::open(path).await?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();
        let mut count = 0;
        while lines.next_line().await?.is_some() {
            count += 1;
        }
        Ok(count)
    }

    /// Get a LoRA training job by ID
    pub async fn get_lora_job(&self, job_id: &str) -> Result<Option<LoraTrainingJob>> {
        Ok(self.lora_jobs.read().await.get(job_id).cloned())
    }

    /// Get all LoRA training jobs
    pub async fn get_lora_jobs(&self) -> Result<Vec<LoraTrainingJob>> {
        Ok(self.lora_jobs.read().await.values().cloned().collect())
    }

    /// Cancel a LoRA training job
    pub async fn cancel_lora_job(&self, job_id: &str) -> Result<()> {
        // Kill the process if running
        if let Some(mut child) = self.active_lora_processes.write().await.remove(job_id) {
            let _ = child.kill().await;
        }

        // Update job status
        let mut jobs = self.lora_jobs.write().await;
        if let Some(job) = jobs.get_mut(job_id) {
            job.status = JobStatus::Cancelled;
            job.completed_at = Some(chrono::Utc::now().timestamp() as u64);
        }

        info!("Cancelled LoRA training job: {}", job_id);
        Ok(())
    }

    /// Delete a LoRA training job
    pub async fn delete_lora_job(&self, job_id: &str) -> Result<()> {
        // Cancel first if running
        self.cancel_lora_job(job_id).await?;

        // Remove from jobs
        self.lora_jobs.write().await.remove(job_id);

        info!("Deleted LoRA training job: {}", job_id);
        Ok(())
    }

    /// Get all saved LoRA adapters
    pub async fn get_lora_adapters(&self) -> Result<Vec<LoraAdapterInfo>> {
        // Scan adapters directory and merge with known adapters
        let mut adapters = self.lora_adapters.read().await.clone();

        // Also scan common adapter locations
        let home_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        let scan_dirs = vec![
            home_dir.join(".citrate/adapters"),
            home_dir.join("Models/adapters"),
            PathBuf::from("./adapters"),
        ];

        for dir in scan_dirs {
            if dir.exists() {
                if let Ok(entries) = std::fs::read_dir(&dir) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.extension().map_or(false, |e| e == "bin" || e == "gguf") {
                            // Check if already known
                            let path_str = path.to_string_lossy().to_string();
                            if !adapters.iter().any(|a| a.path == path_str) {
                                let size = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
                                adapters.push(LoraAdapterInfo {
                                    id: format!("adapter_{}", uuid::Uuid::new_v4().to_string().split('-').next().unwrap_or("0000")),
                                    name: path.file_stem().unwrap_or_default().to_string_lossy().to_string(),
                                    base_model: "unknown".to_string(),
                                    path: path_str,
                                    size_bytes: size,
                                    rank: 0, // Unknown
                                    alpha: 0.0,
                                    target_modules: Vec::new(),
                                    created_at: std::fs::metadata(&path)
                                        .and_then(|m| m.created())
                                        .map(|t| t.duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0))
                                        .unwrap_or(0),
                                    training_job_id: None,
                                    description: None,
                                    tags: Vec::new(),
                                });
                            }
                        }
                    }
                }
            }
        }

        Ok(adapters)
    }

    /// Delete a LoRA adapter
    pub async fn delete_lora_adapter(&self, adapter_id: &str) -> Result<()> {
        let mut adapters = self.lora_adapters.write().await;
        if let Some(idx) = adapters.iter().position(|a| a.id == adapter_id) {
            let adapter = adapters.remove(idx);
            // Delete the actual file
            let path = PathBuf::from(&adapter.path);
            if path.exists() {
                std::fs::remove_file(&path)?;
            }
            info!("Deleted LoRA adapter: {} at {}", adapter_id, adapter.path);
        }
        Ok(())
    }

    /// Run inference with a LoRA adapter applied
    pub async fn run_inference_with_lora(
        &self,
        model_path: &str,
        adapter_path: &str,
        prompt: &str,
        max_tokens: usize,
        temperature: f32,
    ) -> Result<String> {
        let home_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));

        // Find llama.cpp binary
        let llama_paths = vec![
            home_dir.join("llama.cpp/build/bin/llama-cli"),
            home_dir.join("llama.cpp/build/bin/main"),
            PathBuf::from("/usr/local/bin/llama-cli"),
        ];

        let llama_bin = llama_paths.iter()
            .find(|p| p.exists())
            .ok_or_else(|| anyhow!("llama.cpp not found"))?
            .clone();

        let output = tokio::process::Command::new(&llama_bin)
            .arg("-m").arg(model_path)
            .arg("--lora").arg(adapter_path)
            .arg("-p").arg(prompt)
            .arg("-n").arg(max_tokens.to_string())
            .arg("--temp").arg(temperature.to_string())
            .arg("-t").arg(num_cpus::get().to_string())
            .arg("--no-display-prompt")
            .output()
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("Inference with LoRA failed: {}", stderr));
        }

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    /// Validate dataset format
    pub async fn validate_dataset(&self, path: &str, format: &DatasetFormat) -> Result<DatasetValidation> {
        let file_path = PathBuf::from(path);
        if !file_path.exists() {
            return Err(anyhow!("Dataset file not found: {}", path));
        }

        let content = tokio::fs::read_to_string(&file_path).await?;
        let lines: Vec<&str> = content.lines().collect();
        let total_lines = lines.len();

        let mut valid_lines = 0;
        let mut errors: Vec<String> = Vec::new();

        match format {
            DatasetFormat::Jsonl => {
                for (i, line) in lines.iter().enumerate() {
                    match serde_json::from_str::<serde_json::Value>(line) {
                        Ok(obj) => {
                            // Check for required fields (text, instruction, etc.)
                            if obj.get("text").is_some()
                                || obj.get("instruction").is_some()
                                || obj.get("prompt").is_some()
                            {
                                valid_lines += 1;
                            } else {
                                errors.push(format!("Line {}: missing required field (text/instruction/prompt)", i + 1));
                            }
                        }
                        Err(e) => {
                            errors.push(format!("Line {}: invalid JSON - {}", i + 1, e));
                        }
                    }
                }
            }
            DatasetFormat::Alpaca => {
                for (i, line) in lines.iter().enumerate() {
                    match serde_json::from_str::<serde_json::Value>(line) {
                        Ok(obj) => {
                            if obj.get("instruction").is_some() && obj.get("output").is_some() {
                                valid_lines += 1;
                            } else {
                                errors.push(format!("Line {}: missing instruction or output field", i + 1));
                            }
                        }
                        Err(e) => {
                            errors.push(format!("Line {}: invalid JSON - {}", i + 1, e));
                        }
                    }
                }
            }
            DatasetFormat::ShareGPT => {
                for (i, line) in lines.iter().enumerate() {
                    match serde_json::from_str::<serde_json::Value>(line) {
                        Ok(obj) => {
                            if obj.get("conversations").is_some() {
                                valid_lines += 1;
                            } else {
                                errors.push(format!("Line {}: missing conversations array", i + 1));
                            }
                        }
                        Err(e) => {
                            errors.push(format!("Line {}: invalid JSON - {}", i + 1, e));
                        }
                    }
                }
            }
            DatasetFormat::Csv => {
                // Check CSV headers in first line
                if lines.is_empty() {
                    errors.push("Empty CSV file".to_string());
                } else {
                    let headers: Vec<&str> = lines[0].split(',').collect();
                    if headers.contains(&"text") || headers.contains(&"instruction") {
                        valid_lines = lines.len() - 1; // Exclude header
                    } else {
                        errors.push("CSV missing required column (text or instruction)".to_string());
                    }
                }
            }
            _ => {
                // Custom/other formats - just count lines
                valid_lines = total_lines;
            }
        }

        Ok(DatasetValidation {
            valid: errors.is_empty(),
            total_samples: total_lines,
            valid_samples: valid_lines,
            errors: errors.into_iter().take(10).collect(), // Limit to first 10 errors
            estimated_tokens: valid_lines * 256, // Rough estimate
        })
    }

    /// Get default LoRA presets for different model sizes
    pub fn get_lora_presets() -> Vec<LoraPreset> {
        vec![
            LoraPreset {
                name: "Quick Fine-tune".to_string(),
                description: "Fast training with minimal resources".to_string(),
                lora_config: LoraConfig {
                    rank: 4,
                    alpha: 8.0,
                    dropout: 0.1,
                    target_modules: vec!["q_proj".to_string(), "v_proj".to_string()],
                    bias: LoRaBias::None,
                    task_type: LoRaTaskType::CausalLM,
                },
                training_config: LoraTrainingConfig {
                    epochs: 1,
                    batch_size: 1,
                    gradient_accumulation_steps: 8,
                    learning_rate: 3e-4,
                    max_seq_length: 512,
                    ..Default::default()
                },
                recommended_vram_gb: 4,
            },
            LoraPreset {
                name: "Balanced".to_string(),
                description: "Good quality/speed tradeoff".to_string(),
                lora_config: LoraConfig {
                    rank: 8,
                    alpha: 16.0,
                    dropout: 0.05,
                    target_modules: vec![
                        "q_proj".to_string(),
                        "k_proj".to_string(),
                        "v_proj".to_string(),
                        "o_proj".to_string(),
                    ],
                    bias: LoRaBias::None,
                    task_type: LoRaTaskType::CausalLM,
                },
                training_config: LoraTrainingConfig {
                    epochs: 3,
                    batch_size: 4,
                    gradient_accumulation_steps: 4,
                    learning_rate: 2e-4,
                    max_seq_length: 1024,
                    ..Default::default()
                },
                recommended_vram_gb: 8,
            },
            LoraPreset {
                name: "High Quality".to_string(),
                description: "Maximum quality, slower training".to_string(),
                lora_config: LoraConfig {
                    rank: 16,
                    alpha: 32.0,
                    dropout: 0.05,
                    target_modules: vec![
                        "q_proj".to_string(),
                        "k_proj".to_string(),
                        "v_proj".to_string(),
                        "o_proj".to_string(),
                        "gate_proj".to_string(),
                        "up_proj".to_string(),
                        "down_proj".to_string(),
                    ],
                    bias: LoRaBias::None,
                    task_type: LoRaTaskType::CausalLM,
                },
                training_config: LoraTrainingConfig {
                    epochs: 5,
                    batch_size: 4,
                    gradient_accumulation_steps: 8,
                    learning_rate: 1e-4,
                    max_seq_length: 2048,
                    gradient_checkpointing: true,
                    ..Default::default()
                },
                recommended_vram_gb: 16,
            },
            LoraPreset {
                name: "Maximum".to_string(),
                description: "Best quality, requires powerful GPU".to_string(),
                lora_config: LoraConfig {
                    rank: 64,
                    alpha: 128.0,
                    dropout: 0.05,
                    target_modules: vec![
                        "q_proj".to_string(),
                        "k_proj".to_string(),
                        "v_proj".to_string(),
                        "o_proj".to_string(),
                        "gate_proj".to_string(),
                        "up_proj".to_string(),
                        "down_proj".to_string(),
                        "lm_head".to_string(),
                    ],
                    bias: LoRaBias::LoraOnly,
                    task_type: LoRaTaskType::CausalLM,
                },
                training_config: LoraTrainingConfig {
                    epochs: 10,
                    batch_size: 8,
                    gradient_accumulation_steps: 4,
                    learning_rate: 5e-5,
                    max_seq_length: 4096,
                    gradient_checkpointing: true,
                    bf16: true,
                    ..Default::default()
                },
                recommended_vram_gb: 24,
            },
        ]
    }
}

/// Dataset validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatasetValidation {
    pub valid: bool,
    pub total_samples: usize,
    pub valid_samples: usize,
    pub errors: Vec<String>,
    pub estimated_tokens: usize,
}

/// LoRA training preset configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoraPreset {
    pub name: String,
    pub description: String,
    pub lora_config: LoraConfig,
    pub training_config: LoraTrainingConfig,
    pub recommended_vram_gb: u32,
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

/// LoRA-specific training configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoraConfig {
    /// Rank of the LoRA decomposition (typical: 4, 8, 16, 32, 64)
    pub rank: u32,
    /// Scaling factor alpha (typically equals rank or 2*rank)
    pub alpha: f32,
    /// Dropout probability for LoRA layers
    pub dropout: f32,
    /// Target modules to apply LoRA (e.g., ["q_proj", "v_proj", "k_proj", "o_proj"])
    pub target_modules: Vec<String>,
    /// Whether to use bias in LoRA layers
    pub bias: LoRaBias,
    /// Task type for optimization
    pub task_type: LoRaTaskType,
}

impl Default for LoraConfig {
    fn default() -> Self {
        Self {
            rank: 8,
            alpha: 16.0,
            dropout: 0.05,
            target_modules: vec![
                "q_proj".to_string(),
                "v_proj".to_string(),
                "k_proj".to_string(),
                "o_proj".to_string(),
            ],
            bias: LoRaBias::None,
            task_type: LoRaTaskType::CausalLM,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum LoRaBias {
    #[default]
    None,
    All,
    LoraOnly,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum LoRaTaskType {
    #[default]
    CausalLM,
    SequenceClassification,
    TokenClassification,
    QuestionAnswering,
    FeatureExtraction,
}

/// LoRA training job with full configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoraTrainingJob {
    pub id: String,
    /// Base model path (GGUF file)
    pub base_model_path: String,
    /// Base model name for display
    pub base_model_name: String,
    /// Dataset path or identifier
    pub dataset_path: String,
    /// Dataset format (jsonl, csv, parquet)
    pub dataset_format: DatasetFormat,
    /// Output directory for adapter
    pub output_dir: String,
    /// LoRA-specific configuration
    pub lora_config: LoraConfig,
    /// Training hyperparameters
    pub training_config: LoraTrainingConfig,
    /// Current job status
    pub status: JobStatus,
    /// Training progress (0.0-1.0)
    pub progress: f32,
    /// Current epoch
    pub current_epoch: u32,
    /// Current step
    pub current_step: u64,
    /// Total steps
    pub total_steps: u64,
    /// Current training loss
    pub train_loss: f32,
    /// Current validation loss
    pub val_loss: Option<f32>,
    /// Training metrics history
    pub metrics_history: Vec<TrainingMetricsPoint>,
    /// Error message if failed
    pub error_message: Option<String>,
    /// Created timestamp
    pub created_at: u64,
    /// Started timestamp
    pub started_at: Option<u64>,
    /// Completed timestamp
    pub completed_at: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum DatasetFormat {
    #[default]
    Jsonl,
    Csv,
    Parquet,
    Alpaca,
    ShareGPT,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoraTrainingConfig {
    /// Number of training epochs
    pub epochs: u32,
    /// Batch size per device
    pub batch_size: u32,
    /// Gradient accumulation steps
    pub gradient_accumulation_steps: u32,
    /// Learning rate
    pub learning_rate: f64,
    /// Learning rate scheduler type
    pub lr_scheduler: LRSchedulerType,
    /// Warmup ratio (fraction of total steps)
    pub warmup_ratio: f32,
    /// Weight decay
    pub weight_decay: f32,
    /// Maximum sequence length
    pub max_seq_length: u32,
    /// Whether to use gradient checkpointing
    pub gradient_checkpointing: bool,
    /// Evaluation strategy
    pub eval_strategy: EvalStrategy,
    /// Evaluation steps (if eval_strategy is Steps)
    pub eval_steps: Option<u32>,
    /// Save steps
    pub save_steps: u32,
    /// Logging steps
    pub logging_steps: u32,
    /// Maximum gradient norm for clipping
    pub max_grad_norm: f32,
    /// Random seed
    pub seed: u64,
    /// Mixed precision training
    pub fp16: bool,
    /// BF16 training (if supported)
    pub bf16: bool,
    /// Number of CPU threads
    pub num_threads: u32,
    /// Use GPU if available
    pub use_gpu: bool,
    /// GPU layers to offload
    pub n_gpu_layers: u32,
}

impl Default for LoraTrainingConfig {
    fn default() -> Self {
        Self {
            epochs: 3,
            batch_size: 4,
            gradient_accumulation_steps: 4,
            learning_rate: 2e-4,
            lr_scheduler: LRSchedulerType::Cosine,
            warmup_ratio: 0.03,
            weight_decay: 0.001,
            max_seq_length: 2048,
            gradient_checkpointing: true,
            eval_strategy: EvalStrategy::Epoch,
            eval_steps: None,
            save_steps: 100,
            logging_steps: 10,
            max_grad_norm: 1.0,
            seed: 42,
            fp16: false,
            bf16: false,
            num_threads: num_cpus::get() as u32,
            use_gpu: false,
            n_gpu_layers: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum LRSchedulerType {
    #[default]
    Cosine,
    Linear,
    Constant,
    ConstantWithWarmup,
    Polynomial,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum EvalStrategy {
    No,
    Steps,
    #[default]
    Epoch,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingMetricsPoint {
    pub step: u64,
    pub epoch: f32,
    pub train_loss: f32,
    pub val_loss: Option<f32>,
    pub learning_rate: f64,
    pub timestamp: u64,
}

/// LoRA adapter info for saved adapters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoraAdapterInfo {
    pub id: String,
    pub name: String,
    pub base_model: String,
    pub path: String,
    pub size_bytes: u64,
    pub rank: u32,
    pub alpha: f32,
    pub target_modules: Vec<String>,
    pub created_at: u64,
    pub training_job_id: Option<String>,
    pub description: Option<String>,
    pub tags: Vec<String>,
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_model_manager_new() {
        let manager = ModelManager::new();
        assert!(manager.models.try_read().is_ok());
        assert!(manager.lora_jobs.try_read().is_ok());
        assert!(manager.lora_adapters.try_read().is_ok());
    }

    #[test]
    fn test_lora_config_defaults() {
        let config = LoraConfig::default();
        assert_eq!(config.rank, 8);
        assert_eq!(config.alpha, 16.0);
        assert_eq!(config.dropout, 0.05);
        assert!(config.target_modules.contains(&"q_proj".to_string()));
        assert!(config.target_modules.contains(&"v_proj".to_string()));
    }

    #[test]
    fn test_lora_training_config_defaults() {
        let config = LoraTrainingConfig::default();
        assert_eq!(config.epochs, 3);
        assert_eq!(config.batch_size, 4);
        assert_eq!(config.learning_rate, 2e-4);
        assert_eq!(config.warmup_ratio, 0.03); // Default warmup ratio
        assert_eq!(config.gradient_accumulation_steps, 4);
    }

    #[test]
    fn test_lora_presets() {
        let presets = ModelManager::get_lora_presets();
        assert!(!presets.is_empty());

        // Check that we have expected presets
        let preset_names: Vec<&str> = presets.iter().map(|p| p.name.as_str()).collect();
        assert!(preset_names.contains(&"Quick Fine-tune"));
        assert!(preset_names.contains(&"Balanced"));
        assert!(preset_names.contains(&"High Quality"));
        assert!(preset_names.contains(&"Maximum"));
    }

    #[test]
    fn test_lora_preset_configs_are_valid() {
        let presets = ModelManager::get_lora_presets();

        for preset in presets {
            // Validate LoRA config
            assert!(preset.lora_config.rank > 0);
            assert!(preset.lora_config.alpha > 0.0);
            assert!(preset.lora_config.dropout >= 0.0 && preset.lora_config.dropout <= 1.0);
            assert!(!preset.lora_config.target_modules.is_empty());

            // Validate training config
            assert!(preset.training_config.epochs > 0);
            assert!(preset.training_config.batch_size > 0);
            assert!(preset.training_config.learning_rate > 0.0);
        }
    }

    #[test]
    fn test_job_status_variants() {
        let statuses = vec![
            JobStatus::Queued,
            JobStatus::Running,
            JobStatus::Completed,
            JobStatus::Failed,
            JobStatus::Cancelled,
        ];

        for status in &statuses {
            // Verify serialization/deserialization works
            let json = serde_json::to_string(status).unwrap();
            let restored: JobStatus = serde_json::from_str(&json).unwrap();
            assert!(matches!(
                (&status, &restored),
                (JobStatus::Queued, JobStatus::Queued)
                | (JobStatus::Running, JobStatus::Running)
                | (JobStatus::Completed, JobStatus::Completed)
                | (JobStatus::Failed, JobStatus::Failed)
                | (JobStatus::Cancelled, JobStatus::Cancelled)
            ));
        }
    }

    #[test]
    fn test_dataset_format_variants() {
        let formats = vec![
            DatasetFormat::Jsonl,
            DatasetFormat::Csv,
            DatasetFormat::Parquet,
            DatasetFormat::Alpaca,
            DatasetFormat::ShareGPT,
            DatasetFormat::Custom,
        ];

        for format in &formats {
            // Verify serialization/deserialization works
            let json = serde_json::to_string(format).unwrap();
            let _restored: DatasetFormat = serde_json::from_str(&json).unwrap();
        }
    }

    #[tokio::test]
    async fn test_create_lora_job_with_real_files() {
        let temp_dir = TempDir::new().unwrap();

        // Create a fake model file
        let model_path = temp_dir.path().join("test-model.gguf");
        std::fs::write(&model_path, b"fake gguf data").unwrap();

        // Create a dataset file
        let dataset_path = temp_dir.path().join("dataset.jsonl");
        std::fs::write(&dataset_path, r#"{"text": "Hello world"}"#).unwrap();

        let output_dir = temp_dir.path().join("output");

        let manager = ModelManager::new();

        let result = manager.create_lora_job(
            model_path.to_str().unwrap().to_string(),
            "test-model".to_string(),
            dataset_path.to_str().unwrap().to_string(),
            DatasetFormat::Jsonl,
            output_dir.to_str().unwrap().to_string(),
            Some(LoraConfig::default()),
            Some(LoraTrainingConfig::default()),
        ).await;

        assert!(result.is_ok());
        let job = result.unwrap();

        assert_eq!(job.base_model_name, "test-model");
        assert!(matches!(job.dataset_format, DatasetFormat::Jsonl));
        assert!(matches!(job.status, JobStatus::Queued));
        assert_eq!(job.progress, 0.0);
    }

    #[tokio::test]
    async fn test_create_lora_job_missing_model() {
        let temp_dir = TempDir::new().unwrap();
        let dataset_path = temp_dir.path().join("dataset.jsonl");
        std::fs::write(&dataset_path, r#"{"text": "test"}"#).unwrap();

        let manager = ModelManager::new();

        let result = manager.create_lora_job(
            "/nonexistent/model.gguf".to_string(),
            "test-model".to_string(),
            dataset_path.to_str().unwrap().to_string(),
            DatasetFormat::Jsonl,
            "/output".to_string(),
            None,
            None,
        ).await;

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[tokio::test]
    async fn test_get_lora_job() {
        let temp_dir = TempDir::new().unwrap();

        let model_path = temp_dir.path().join("model.gguf");
        std::fs::write(&model_path, b"fake gguf").unwrap();

        let dataset_path = temp_dir.path().join("dataset.jsonl");
        std::fs::write(&dataset_path, r#"{"text": "test"}"#).unwrap();

        let output_dir = temp_dir.path().join("output");

        let manager = ModelManager::new();

        let job = manager.create_lora_job(
            model_path.to_str().unwrap().to_string(),
            "test-model".to_string(),
            dataset_path.to_str().unwrap().to_string(),
            DatasetFormat::Jsonl,
            output_dir.to_str().unwrap().to_string(),
            Some(LoraConfig::default()),
            Some(LoraTrainingConfig::default()),
        ).await.unwrap();

        // Get the job by ID
        let retrieved = manager.get_lora_job(&job.id).await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id, job.id);

        // Get non-existent job
        let not_found = manager.get_lora_job("non-existent-id").await.unwrap();
        assert!(not_found.is_none());
    }

    #[tokio::test]
    async fn test_get_all_lora_jobs() {
        let manager = ModelManager::new();

        // Initially empty
        let jobs = manager.get_lora_jobs().await.unwrap();
        assert!(jobs.is_empty());
    }

    #[tokio::test]
    async fn test_cancel_lora_job() {
        let temp_dir = TempDir::new().unwrap();

        let model_path = temp_dir.path().join("model.gguf");
        std::fs::write(&model_path, b"fake gguf").unwrap();

        let dataset_path = temp_dir.path().join("dataset.jsonl");
        std::fs::write(&dataset_path, r#"{"text": "test"}"#).unwrap();

        let output_dir = temp_dir.path().join("output");

        let manager = ModelManager::new();

        let job = manager.create_lora_job(
            model_path.to_str().unwrap().to_string(),
            "test-model".to_string(),
            dataset_path.to_str().unwrap().to_string(),
            DatasetFormat::Jsonl,
            output_dir.to_str().unwrap().to_string(),
            Some(LoraConfig::default()),
            Some(LoraTrainingConfig::default()),
        ).await.unwrap();

        // Cancel the job
        let result = manager.cancel_lora_job(&job.id).await;
        assert!(result.is_ok());

        // Verify status changed
        let updated = manager.get_lora_job(&job.id).await.unwrap().unwrap();
        assert!(matches!(updated.status, JobStatus::Cancelled));

        // Note: cancel_lora_job returns Ok even for non-existent jobs (no-op)
        let result = manager.cancel_lora_job("non-existent").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_delete_lora_job() {
        let temp_dir = TempDir::new().unwrap();

        let model_path = temp_dir.path().join("model.gguf");
        std::fs::write(&model_path, b"fake gguf").unwrap();

        let dataset_path = temp_dir.path().join("dataset.jsonl");
        std::fs::write(&dataset_path, r#"{"text": "test"}"#).unwrap();

        let output_dir = temp_dir.path().join("output");

        let manager = ModelManager::new();

        let job = manager.create_lora_job(
            model_path.to_str().unwrap().to_string(),
            "test-model".to_string(),
            dataset_path.to_str().unwrap().to_string(),
            DatasetFormat::Jsonl,
            output_dir.to_str().unwrap().to_string(),
            Some(LoraConfig::default()),
            Some(LoraTrainingConfig::default()),
        ).await.unwrap();

        // Delete the job
        let result = manager.delete_lora_job(&job.id).await;
        assert!(result.is_ok());

        // Verify job is gone
        let jobs = manager.get_lora_jobs().await.unwrap();
        assert!(jobs.is_empty());

        // Note: delete_lora_job returns Ok even for non-existent jobs (no-op)
        let result = manager.delete_lora_job("non-existent").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_lora_adapters_initially_empty() {
        let manager = ModelManager::new();

        let adapters = manager.get_lora_adapters().await.unwrap();
        assert!(adapters.is_empty());
    }

    #[tokio::test]
    async fn test_validate_dataset_missing_file() {
        let manager = ModelManager::new();

        let result = manager.validate_dataset(
            "/nonexistent/path/dataset.jsonl",
            &DatasetFormat::Jsonl,
        ).await;

        // validate_dataset returns Err for missing files
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[tokio::test]
    async fn test_validate_dataset_jsonl() {
        let temp_dir = TempDir::new().unwrap();
        let dataset_path = temp_dir.path().join("test.jsonl");

        // Create a valid JSONL file
        std::fs::write(
            &dataset_path,
            r#"{"text": "Hello world"}
{"text": "Another line"}
{"text": "Third entry"}
"#,
        ).unwrap();

        let manager = ModelManager::new();
        let result = manager.validate_dataset(
            dataset_path.to_str().unwrap(),
            &DatasetFormat::Jsonl,
        ).await;

        assert!(result.is_ok());
        let validation = result.unwrap();
        assert!(validation.valid);
        assert!(validation.total_samples >= 3);
    }

    #[tokio::test]
    async fn test_validate_dataset_alpaca_format() {
        let temp_dir = TempDir::new().unwrap();
        let dataset_path = temp_dir.path().join("alpaca.jsonl");

        // Create a valid Alpaca format file
        std::fs::write(
            &dataset_path,
            r#"{"instruction": "Summarize this", "input": "Long text here", "output": "Summary"}
{"instruction": "Translate", "input": "Hello", "output": "Hola"}
"#,
        ).unwrap();

        let manager = ModelManager::new();
        let result = manager.validate_dataset(
            dataset_path.to_str().unwrap(),
            &DatasetFormat::Alpaca,
        ).await;

        assert!(result.is_ok());
        let validation = result.unwrap();
        assert!(validation.valid);
    }

    #[tokio::test]
    async fn test_validate_dataset_sharegpt_format() {
        let temp_dir = TempDir::new().unwrap();
        let dataset_path = temp_dir.path().join("sharegpt.jsonl");

        // Create a valid ShareGPT format file
        std::fs::write(
            &dataset_path,
            r#"{"conversations": [{"from": "human", "value": "Hello"}, {"from": "gpt", "value": "Hi!"}]}
{"conversations": [{"from": "human", "value": "What is AI?"}, {"from": "gpt", "value": "Artificial Intelligence"}]}
"#,
        ).unwrap();

        let manager = ModelManager::new();
        let result = manager.validate_dataset(
            dataset_path.to_str().unwrap(),
            &DatasetFormat::ShareGPT,
        ).await;

        assert!(result.is_ok());
        let validation = result.unwrap();
        assert!(validation.valid);
    }

    #[test]
    fn test_lora_bias_serialization() {
        let biases = vec![LoRaBias::None, LoRaBias::All, LoRaBias::LoraOnly];

        for bias in biases {
            let json = serde_json::to_string(&bias).unwrap();
            let restored: LoRaBias = serde_json::from_str(&json).unwrap();
            match (&bias, &restored) {
                (LoRaBias::None, LoRaBias::None) => {}
                (LoRaBias::All, LoRaBias::All) => {}
                (LoRaBias::LoraOnly, LoRaBias::LoraOnly) => {}
                _ => panic!("Bias mismatch"),
            }
        }
    }

    #[test]
    fn test_lora_task_type_serialization() {
        let tasks = vec![
            LoRaTaskType::CausalLM,
            LoRaTaskType::SequenceClassification,
            LoRaTaskType::TokenClassification,
            LoRaTaskType::QuestionAnswering,
            LoRaTaskType::FeatureExtraction,
        ];

        for task in tasks {
            let json = serde_json::to_string(&task).unwrap();
            let _restored: LoRaTaskType = serde_json::from_str(&json).unwrap();
        }
    }

    #[test]
    fn test_lr_scheduler_serialization() {
        let schedulers = vec![
            LRSchedulerType::Linear,
            LRSchedulerType::Cosine,
            LRSchedulerType::Polynomial,
            LRSchedulerType::Constant,
            LRSchedulerType::ConstantWithWarmup,
        ];

        for scheduler in schedulers {
            let json = serde_json::to_string(&scheduler).unwrap();
            let _restored: LRSchedulerType = serde_json::from_str(&json).unwrap();
        }
    }

    #[test]
    fn test_lora_adapter_info_serialization() {
        let adapter = LoraAdapterInfo {
            id: "adapter-123".to_string(),
            name: "Test Adapter".to_string(),
            base_model: "llama-7b".to_string(),
            path: "/path/to/adapter".to_string(),
            size_bytes: 1024000,
            rank: 8,
            alpha: 16.0,
            target_modules: vec!["q_proj".to_string(), "v_proj".to_string()],
            created_at: 1700000000,
            training_job_id: Some("job-456".to_string()),
            description: Some("Test adapter description".to_string()),
            tags: vec!["test".to_string(), "demo".to_string()],
        };

        let json = serde_json::to_string(&adapter).unwrap();
        let restored: LoraAdapterInfo = serde_json::from_str(&json).unwrap();

        assert_eq!(adapter.id, restored.id);
        assert_eq!(adapter.name, restored.name);
        assert_eq!(adapter.base_model, restored.base_model);
        assert_eq!(adapter.size_bytes, restored.size_bytes);
        assert_eq!(adapter.rank, restored.rank);
    }

    #[test]
    fn test_dataset_validation_struct() {
        let validation = DatasetValidation {
            valid: true,
            total_samples: 100,
            valid_samples: 95,
            errors: vec!["Minor issue".to_string()],
            estimated_tokens: 50000,
        };

        let json = serde_json::to_string(&validation).unwrap();
        let restored: DatasetValidation = serde_json::from_str(&json).unwrap();

        assert_eq!(validation.valid, restored.valid);
        assert_eq!(validation.total_samples, restored.total_samples);
        assert_eq!(validation.valid_samples, restored.valid_samples);
        assert_eq!(validation.estimated_tokens, restored.estimated_tokens);
    }
}
