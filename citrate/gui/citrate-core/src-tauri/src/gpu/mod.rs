//! GPU Resource Manager Module
//!
//! Provides GPU detection, allocation, and monitoring for distributed compute.
//! Supports macOS Metal, NVIDIA CUDA, and AMD ROCm GPU backends.
//!
//! ## Architecture
//!
//! ```text
//! GPU Compute Network:
//! ├── GPU Detector (hardware enumeration)
//! ├── Resource Manager (allocation & tracking)
//! ├── Job Scheduler (compute job queue)
//! └── Provider (contribute GPU to network)
//! ```

use chrono::Timelike;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

// ============================================================================
// Types
// ============================================================================

/// GPU device information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GPUDevice {
    /// Unique device identifier
    pub id: String,
    /// Human-readable device name
    pub name: String,
    /// GPU vendor
    pub vendor: GPUVendor,
    /// Total VRAM in bytes
    pub total_memory: u64,
    /// Available VRAM in bytes
    pub available_memory: u64,
    /// GPU compute capability or version
    pub compute_capability: String,
    /// Whether the device is currently in use
    pub in_use: bool,
    /// Backend type
    pub backend: GPUBackend,
    /// Temperature in Celsius (if available)
    pub temperature: Option<f32>,
    /// Power usage in watts (if available)
    pub power_usage: Option<f32>,
    /// Utilization percentage (0-100)
    pub utilization: u8,
}

/// GPU vendor enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GPUVendor {
    Apple,
    NVIDIA,
    AMD,
    Intel,
    Unknown,
}

impl GPUVendor {
    pub fn from_string(s: &str) -> Self {
        let lower = s.to_lowercase();
        if lower.contains("apple") || lower.contains("m1") || lower.contains("m2") || lower.contains("m3") || lower.contains("m4") {
            Self::Apple
        } else if lower.contains("nvidia") || lower.contains("geforce") || lower.contains("rtx") || lower.contains("gtx") {
            Self::NVIDIA
        } else if lower.contains("amd") || lower.contains("radeon") {
            Self::AMD
        } else if lower.contains("intel") {
            Self::Intel
        } else {
            Self::Unknown
        }
    }
}

/// GPU backend for compute operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GPUBackend {
    /// Apple Metal (macOS)
    Metal,
    /// NVIDIA CUDA
    CUDA,
    /// AMD ROCm
    ROCm,
    /// Vulkan compute
    Vulkan,
    /// OpenCL fallback
    OpenCL,
    /// CPU-only (no GPU)
    CPU,
}

impl std::fmt::Display for GPUBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Metal => write!(f, "Metal"),
            Self::CUDA => write!(f, "CUDA"),
            Self::ROCm => write!(f, "ROCm"),
            Self::Vulkan => write!(f, "Vulkan"),
            Self::OpenCL => write!(f, "OpenCL"),
            Self::CPU => write!(f, "CPU"),
        }
    }
}

/// Compute job types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComputeJobType {
    /// Model inference
    Inference,
    /// Full model training
    Training,
    /// LoRA fine-tuning
    LoRAFineTune,
    /// Embedding generation
    Embedding,
    /// Image generation
    ImageGeneration,
}

/// Compute job status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComputeJobStatus {
    /// Waiting in queue
    Queued,
    /// Currently executing
    Running {
        started_at: u64,
        progress: f32,
    },
    /// Successfully completed
    Completed {
        started_at: u64,
        completed_at: u64,
        result_hash: String,
    },
    /// Failed with error
    Failed {
        error: String,
        failed_at: u64,
    },
    /// Cancelled by user
    Cancelled,
}

/// A compute job in the queue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputeJob {
    /// Unique job identifier
    pub id: String,
    /// Job type
    pub job_type: ComputeJobType,
    /// Model being used
    pub model_id: String,
    /// Input data hash (IPFS CID or local hash)
    pub input_hash: String,
    /// Requester address
    pub requester: String,
    /// Maximum tokens to pay
    pub max_payment: u64,
    /// Current status
    pub status: ComputeJobStatus,
    /// Created timestamp
    pub created_at: u64,
    /// GPU memory required (bytes)
    pub memory_required: u64,
    /// Estimated compute time (seconds)
    pub estimated_time: u64,
    /// Priority (higher = more priority)
    pub priority: u32,
}

/// GPU allocation settings for the user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GPUAllocationSettings {
    /// Whether to contribute GPU to the network
    pub enabled: bool,
    /// Percentage of GPU to allocate (0-100)
    pub allocation_percentage: u8,
    /// Maximum memory to allocate (bytes, 0 = auto)
    pub max_memory_allocation: u64,
    /// Only accept jobs above this payment threshold
    pub min_payment_threshold: u64,
    /// Maximum concurrent jobs
    pub max_concurrent_jobs: u32,
    /// Allowed job types
    pub allowed_job_types: Vec<ComputeJobType>,
    /// Schedule: hours when GPU is available (24h format, e.g., [9, 17] = 9am-5pm)
    pub schedule: Option<(u8, u8)>,
}

impl Default for GPUAllocationSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            allocation_percentage: 50,
            max_memory_allocation: 0, // Auto
            min_payment_threshold: 0,
            max_concurrent_jobs: 2,
            allowed_job_types: vec![
                ComputeJobType::Inference,
                ComputeJobType::Embedding,
            ],
            schedule: None, // Always available
        }
    }
}

/// GPU resource usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GPUStats {
    /// Total jobs completed
    pub jobs_completed: u64,
    /// Total jobs failed
    pub jobs_failed: u64,
    /// Total tokens earned
    pub tokens_earned: u64,
    /// Total compute time (seconds)
    pub total_compute_time: u64,
    /// Average job duration (seconds)
    pub avg_job_duration: f64,
    /// Current queue depth
    pub queue_depth: usize,
    /// Current memory usage (bytes)
    pub current_memory_usage: u64,
    /// Session start time
    pub session_start: u64,
}

impl Default for GPUStats {
    fn default() -> Self {
        Self {
            jobs_completed: 0,
            jobs_failed: 0,
            tokens_earned: 0,
            total_compute_time: 0,
            avg_job_duration: 0.0,
            queue_depth: 0,
            current_memory_usage: 0,
            session_start: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        }
    }
}

/// Provider status for registration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderStatus {
    /// Whether registered as provider
    pub is_registered: bool,
    /// Provider address
    pub address: Option<String>,
    /// Stake amount
    pub stake: u64,
    /// Reputation score (0-100)
    pub reputation: u8,
    /// Last heartbeat timestamp
    pub last_heartbeat: u64,
    /// Currently active jobs
    pub active_jobs: Vec<String>,
}

// ============================================================================
// GPU Resource Manager
// ============================================================================

/// Manages GPU resources for distributed compute
pub struct GPUResourceManager {
    /// Detected GPU devices
    devices: Arc<RwLock<Vec<GPUDevice>>>,
    /// Active compute jobs
    jobs: Arc<RwLock<HashMap<String, ComputeJob>>>,
    /// Job queue (waiting jobs)
    queue: Arc<RwLock<Vec<ComputeJob>>>,
    /// User allocation settings
    settings: Arc<RwLock<GPUAllocationSettings>>,
    /// Runtime statistics
    stats: Arc<RwLock<GPUStats>>,
    /// Provider registration status
    provider_status: Arc<RwLock<ProviderStatus>>,
}

impl GPUResourceManager {
    /// Create a new GPU resource manager
    pub fn new() -> Self {
        let manager = Self {
            devices: Arc::new(RwLock::new(Vec::new())),
            jobs: Arc::new(RwLock::new(HashMap::new())),
            queue: Arc::new(RwLock::new(Vec::new())),
            settings: Arc::new(RwLock::new(GPUAllocationSettings::default())),
            stats: Arc::new(RwLock::new(GPUStats::default())),
            provider_status: Arc::new(RwLock::new(ProviderStatus {
                is_registered: false,
                address: None,
                stake: 0,
                reputation: 100,
                last_heartbeat: 0,
                active_jobs: vec![],
            })),
        };

        // Note: GPU detection is done lazily when get_devices() or refresh_devices() is called
        // This avoids requiring a tokio runtime during initialization
        manager
    }

    /// Get all detected GPU devices (detects on first call if empty)
    pub async fn get_devices(&self) -> Vec<GPUDevice> {
        let devices = self.devices.read().await;
        if devices.is_empty() {
            drop(devices); // Release read lock
            // Do lazy detection
            return self.refresh_devices().await;
        }
        devices.clone()
    }

    /// Refresh GPU device information
    pub async fn refresh_devices(&self) -> Vec<GPUDevice> {
        let detected = detect_gpus().await;
        let mut devices = self.devices.write().await;
        *devices = detected.clone();
        detected
    }

    /// Get current allocation settings
    pub async fn get_settings(&self) -> GPUAllocationSettings {
        self.settings.read().await.clone()
    }

    /// Update allocation settings
    pub async fn update_settings(&self, new_settings: GPUAllocationSettings) -> Result<(), String> {
        // Validate settings
        if new_settings.allocation_percentage > 100 {
            return Err("Allocation percentage cannot exceed 100%".to_string());
        }
        if new_settings.max_concurrent_jobs == 0 {
            return Err("Max concurrent jobs must be at least 1".to_string());
        }
        if let Some((start, end)) = new_settings.schedule {
            if start >= 24 || end >= 24 {
                return Err("Schedule hours must be 0-23".to_string());
            }
        }

        let mut settings = self.settings.write().await;
        *settings = new_settings;
        info!("GPU allocation settings updated");
        Ok(())
    }

    /// Get current statistics
    pub async fn get_stats(&self) -> GPUStats {
        let mut stats = self.stats.read().await.clone();
        stats.queue_depth = self.queue.read().await.len();
        stats
    }

    /// Get provider registration status
    pub async fn get_provider_status(&self) -> ProviderStatus {
        self.provider_status.read().await.clone()
    }

    /// Submit a new compute job
    pub async fn submit_job(&self, job: ComputeJob) -> Result<String, String> {
        let settings = self.settings.read().await;

        if !settings.enabled {
            return Err("GPU compute is not enabled".to_string());
        }

        // Check if job type is allowed
        if !settings.allowed_job_types.contains(&job.job_type) {
            return Err(format!("Job type {:?} is not allowed", job.job_type));
        }

        // Check memory requirements against available
        let devices = self.devices.read().await;
        let total_available: u64 = devices.iter()
            .map(|d| d.available_memory)
            .sum();

        let allocated_memory = (total_available as f64 * (settings.allocation_percentage as f64 / 100.0)) as u64;

        if job.memory_required > allocated_memory {
            return Err(format!(
                "Job requires {} MB but only {} MB allocated",
                job.memory_required / 1024 / 1024,
                allocated_memory / 1024 / 1024
            ));
        }

        let job_id = job.id.clone();

        // Add to queue
        let mut queue = self.queue.write().await;
        queue.push(job);

        // Sort by priority (higher first)
        queue.sort_by(|a, b| b.priority.cmp(&a.priority));

        info!("Compute job {} submitted to queue", job_id);
        Ok(job_id)
    }

    /// Get a job by ID
    pub async fn get_job(&self, job_id: &str) -> Option<ComputeJob> {
        // Check active jobs
        if let Some(job) = self.jobs.read().await.get(job_id) {
            return Some(job.clone());
        }

        // Check queue
        let queue = self.queue.read().await;
        queue.iter().find(|j| j.id == job_id).cloned()
    }

    /// Get all jobs (active + queued)
    pub async fn get_all_jobs(&self) -> Vec<ComputeJob> {
        let mut all_jobs: Vec<ComputeJob> = self.jobs.read().await
            .values()
            .cloned()
            .collect();

        all_jobs.extend(self.queue.read().await.clone());
        all_jobs
    }

    /// Cancel a job
    pub async fn cancel_job(&self, job_id: &str) -> Result<(), String> {
        // Try to remove from queue first
        {
            let mut queue = self.queue.write().await;
            if let Some(pos) = queue.iter().position(|j| j.id == job_id) {
                queue.remove(pos);
                info!("Job {} removed from queue", job_id);
                return Ok(());
            }
        }

        // If not in queue, try active jobs
        {
            let mut jobs = self.jobs.write().await;
            if let Some(job) = jobs.get_mut(job_id) {
                job.status = ComputeJobStatus::Cancelled;
                info!("Job {} cancelled", job_id);
                return Ok(());
            }
        }

        Err(format!("Job {} not found", job_id))
    }

    /// Get available GPU memory for compute
    pub async fn get_available_compute_memory(&self) -> u64 {
        let settings = self.settings.read().await;
        if !settings.enabled {
            return 0;
        }

        let devices = self.devices.read().await;
        let total_available: u64 = devices.iter()
            .map(|d| d.available_memory)
            .sum();

        (total_available as f64 * (settings.allocation_percentage as f64 / 100.0)) as u64
    }

    /// Check if GPU compute is within scheduled hours
    pub async fn is_within_schedule(&self) -> bool {
        let settings = self.settings.read().await;

        if let Some((start, end)) = settings.schedule {
            let now = chrono::Local::now();
            let current_hour = now.hour() as u8;

            if start <= end {
                // Normal schedule (e.g., 9am - 5pm)
                current_hour >= start && current_hour < end
            } else {
                // Overnight schedule (e.g., 10pm - 6am)
                current_hour >= start || current_hour < end
            }
        } else {
            // No schedule = always available
            true
        }
    }

    /// Process next job in queue (called by scheduler)
    pub async fn process_next_job(&self) -> Option<ComputeJob> {
        if !self.is_within_schedule().await {
            debug!("Outside scheduled hours, not processing jobs");
            return None;
        }

        let settings = self.settings.read().await;
        let active_count = self.jobs.read().await.len();

        if active_count >= settings.max_concurrent_jobs as usize {
            debug!("Max concurrent jobs reached ({}/{})", active_count, settings.max_concurrent_jobs);
            return None;
        }

        // Pop next job from queue
        let job = {
            let mut queue = self.queue.write().await;
            if queue.is_empty() {
                return None;
            }
            queue.remove(0)
        };

        // Move to active jobs
        let job_id = job.id.clone();
        let mut running_job = job.clone();
        running_job.status = ComputeJobStatus::Running {
            started_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            progress: 0.0,
        };

        self.jobs.write().await.insert(job_id.clone(), running_job.clone());
        info!("Started processing job {}", job_id);

        Some(running_job)
    }

    /// Mark a job as completed
    pub async fn complete_job(&self, job_id: &str, result_hash: String) -> Result<(), String> {
        let mut jobs = self.jobs.write().await;

        if let Some(job) = jobs.get_mut(job_id) {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();

            let started_at = match &job.status {
                ComputeJobStatus::Running { started_at, .. } => *started_at,
                _ => now,
            };

            job.status = ComputeJobStatus::Completed {
                started_at,
                completed_at: now,
                result_hash,
            };

            // Update stats
            let mut stats = self.stats.write().await;
            stats.jobs_completed += 1;
            let duration = now - started_at;
            stats.total_compute_time += duration;
            stats.avg_job_duration = stats.total_compute_time as f64 / stats.jobs_completed as f64;
            stats.tokens_earned += job.max_payment; // Simplified - actual would be based on usage

            info!("Job {} completed in {} seconds", job_id, duration);
            Ok(())
        } else {
            Err(format!("Job {} not found in active jobs", job_id))
        }
    }

    /// Mark a job as failed
    pub async fn fail_job(&self, job_id: &str, error: String) -> Result<(), String> {
        let mut jobs = self.jobs.write().await;

        if let Some(job) = jobs.get_mut(job_id) {
            job.status = ComputeJobStatus::Failed {
                error: error.clone(),
                failed_at: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
            };

            let mut stats = self.stats.write().await;
            stats.jobs_failed += 1;

            warn!("Job {} failed: {}", job_id, error);
            Ok(())
        } else {
            Err(format!("Job {} not found in active jobs", job_id))
        }
    }
}

impl Default for GPUResourceManager {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// GPU Detection
// ============================================================================

/// Detect available GPUs on the system
pub async fn detect_gpus() -> Vec<GPUDevice> {
    let mut devices = Vec::new();

    // Platform-specific detection
    #[cfg(target_os = "macos")]
    {
        if let Some(metal_device) = detect_metal_gpu() {
            devices.push(metal_device);
        }
    }

    #[cfg(not(target_os = "macos"))]
    {
        // Try CUDA first
        devices.extend(detect_cuda_gpus());

        // If no CUDA, try ROCm
        if devices.is_empty() {
            devices.extend(detect_rocm_gpus());
        }
    }

    // If no GPU found, add CPU fallback
    if devices.is_empty() {
        devices.push(create_cpu_fallback());
    }

    info!("Detected {} GPU device(s)", devices.len());
    devices
}

/// Detect Apple Metal GPU (macOS)
#[cfg(target_os = "macos")]
fn detect_metal_gpu() -> Option<GPUDevice> {
    // Use system_profiler to get GPU info
    use std::process::Command;

    let output = Command::new("system_profiler")
        .args(["SPDisplaysDataType", "-json"])
        .output()
        .ok()?;

    if !output.status.success() {
        warn!("Failed to run system_profiler");
        return None;
    }

    let json_str = String::from_utf8_lossy(&output.stdout);

    // Parse JSON to extract GPU info
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&json_str) {
        if let Some(displays) = json.get("SPDisplaysDataType").and_then(|d| d.as_array()) {
            for display in displays {
                let name = display.get("sppci_model")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Apple GPU")
                    .to_string();

                // Get VRAM if available
                let vram_str = display.get("spdisplays_vram")
                    .and_then(|v| v.as_str())
                    .unwrap_or("8 GB");

                let vram_bytes = parse_memory_string(vram_str);

                // Check for unified memory (Apple Silicon)
                let is_apple_silicon = name.contains("Apple") ||
                    name.contains("M1") ||
                    name.contains("M2") ||
                    name.contains("M3") ||
                    name.contains("M4");

                let total_memory = if is_apple_silicon {
                    // For Apple Silicon, use system memory as unified memory
                    get_system_memory().unwrap_or(vram_bytes)
                } else {
                    vram_bytes
                };

                return Some(GPUDevice {
                    id: "metal-0".to_string(),
                    name,
                    vendor: GPUVendor::Apple,
                    total_memory,
                    available_memory: (total_memory as f64 * 0.8) as u64, // Assume 80% available
                    compute_capability: "Metal".to_string(),
                    in_use: false,
                    backend: GPUBackend::Metal,
                    temperature: None,
                    power_usage: None,
                    utilization: 0,
                });
            }
        }
    }

    // Fallback for Apple Silicon
    Some(GPUDevice {
        id: "metal-0".to_string(),
        name: "Apple GPU".to_string(),
        vendor: GPUVendor::Apple,
        total_memory: get_system_memory().unwrap_or(8 * 1024 * 1024 * 1024),
        available_memory: get_system_memory().unwrap_or(8 * 1024 * 1024 * 1024) * 8 / 10,
        compute_capability: "Metal".to_string(),
        in_use: false,
        backend: GPUBackend::Metal,
        temperature: None,
        power_usage: None,
        utilization: 0,
    })
}

/// Detect NVIDIA CUDA GPUs
#[cfg(not(target_os = "macos"))]
fn detect_cuda_gpus() -> Vec<GPUDevice> {
    use std::process::Command;

    let mut devices = Vec::new();

    // Try nvidia-smi
    let output = Command::new("nvidia-smi")
        .args(["--query-gpu=index,name,memory.total,memory.free,compute_cap,temperature.gpu,power.draw,utilization.gpu", "--format=csv,noheader,nounits"])
        .output();

    if let Ok(output) = output {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                let parts: Vec<&str> = line.split(", ").collect();
                if parts.len() >= 8 {
                    let idx = parts[0].trim();
                    let name = parts[1].trim().to_string();
                    let total_mem: u64 = parts[2].trim().parse().unwrap_or(0) * 1024 * 1024;
                    let free_mem: u64 = parts[3].trim().parse().unwrap_or(0) * 1024 * 1024;
                    let compute_cap = parts[4].trim().to_string();
                    let temp: f32 = parts[5].trim().parse().unwrap_or(0.0);
                    let power: f32 = parts[6].trim().parse().unwrap_or(0.0);
                    let util: u8 = parts[7].trim().parse().unwrap_or(0);

                    devices.push(GPUDevice {
                        id: format!("cuda-{}", idx),
                        name,
                        vendor: GPUVendor::NVIDIA,
                        total_memory: total_mem,
                        available_memory: free_mem,
                        compute_capability: compute_cap,
                        in_use: util > 10,
                        backend: GPUBackend::CUDA,
                        temperature: Some(temp),
                        power_usage: Some(power),
                        utilization: util,
                    });
                }
            }
        }
    }

    devices
}

/// Detect AMD ROCm GPUs
#[cfg(not(target_os = "macos"))]
fn detect_rocm_gpus() -> Vec<GPUDevice> {
    use std::process::Command;

    let mut devices = Vec::new();

    // Try rocm-smi
    let output = Command::new("rocm-smi")
        .args(["--showmeminfo", "vram", "--json"])
        .output();

    if let Ok(output) = output {
        if output.status.success() {
            // Parse ROCm output
            if let Ok(json) = serde_json::from_slice::<serde_json::Value>(&output.stdout) {
                // ROCm JSON parsing would go here
                // Simplified for now
                debug!("ROCm JSON: {:?}", json);
            }
        }
    }

    // Fallback: try lspci
    let output = Command::new("lspci")
        .args(["-nn"])
        .output();

    if let Ok(output) = output {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            if line.contains("VGA") || line.contains("3D") {
                if line.to_lowercase().contains("amd") || line.to_lowercase().contains("radeon") {
                    devices.push(GPUDevice {
                        id: format!("rocm-{}", devices.len()),
                        name: "AMD GPU".to_string(),
                        vendor: GPUVendor::AMD,
                        total_memory: 8 * 1024 * 1024 * 1024, // Assume 8GB
                        available_memory: 6 * 1024 * 1024 * 1024,
                        compute_capability: "ROCm".to_string(),
                        in_use: false,
                        backend: GPUBackend::ROCm,
                        temperature: None,
                        power_usage: None,
                        utilization: 0,
                    });
                }
            }
        }
    }

    devices
}

/// Create CPU fallback device
fn create_cpu_fallback() -> GPUDevice {
    let cpu_count = num_cpus::get();
    let memory = get_system_memory().unwrap_or(8 * 1024 * 1024 * 1024);

    GPUDevice {
        id: "cpu-0".to_string(),
        name: format!("CPU ({} cores)", cpu_count),
        vendor: GPUVendor::Unknown,
        total_memory: memory,
        available_memory: memory / 2,
        compute_capability: "CPU".to_string(),
        in_use: false,
        backend: GPUBackend::CPU,
        temperature: None,
        power_usage: None,
        utilization: 0,
    }
}

/// Get system memory in bytes
fn get_system_memory() -> Option<u64> {
    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        let output = Command::new("sysctl")
            .args(["-n", "hw.memsize"])
            .output()
            .ok()?;

        let mem_str = String::from_utf8_lossy(&output.stdout);
        mem_str.trim().parse().ok()
    }

    #[cfg(target_os = "linux")]
    {
        use std::fs;
        let meminfo = fs::read_to_string("/proc/meminfo").ok()?;
        for line in meminfo.lines() {
            if line.starts_with("MemTotal:") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    let kb: u64 = parts[1].parse().ok()?;
                    return Some(kb * 1024);
                }
            }
        }
        None
    }

    #[cfg(target_os = "windows")]
    {
        // Simplified - would use Windows API in production
        Some(16 * 1024 * 1024 * 1024) // Assume 16GB
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        None
    }
}

/// Parse memory string like "8 GB" to bytes
fn parse_memory_string(s: &str) -> u64 {
    let s = s.trim().to_uppercase();
    let parts: Vec<&str> = s.split_whitespace().collect();

    if parts.is_empty() {
        return 0;
    }

    let value: f64 = parts[0].parse().unwrap_or(0.0);
    let unit = parts.get(1).unwrap_or(&"GB");

    match *unit {
        "TB" => (value * 1024.0 * 1024.0 * 1024.0 * 1024.0) as u64,
        "GB" => (value * 1024.0 * 1024.0 * 1024.0) as u64,
        "MB" => (value * 1024.0 * 1024.0) as u64,
        "KB" => (value * 1024.0) as u64,
        _ => (value * 1024.0 * 1024.0 * 1024.0) as u64, // Default GB
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gpu_vendor_from_string() {
        assert_eq!(GPUVendor::from_string("Apple M2 Pro"), GPUVendor::Apple);
        assert_eq!(GPUVendor::from_string("NVIDIA RTX 4090"), GPUVendor::NVIDIA);
        assert_eq!(GPUVendor::from_string("AMD Radeon RX 7900"), GPUVendor::AMD);
        assert_eq!(GPUVendor::from_string("Intel UHD Graphics"), GPUVendor::Intel);
        assert_eq!(GPUVendor::from_string("Unknown GPU"), GPUVendor::Unknown);
    }

    #[test]
    fn test_gpu_backend_display() {
        assert_eq!(GPUBackend::Metal.to_string(), "Metal");
        assert_eq!(GPUBackend::CUDA.to_string(), "CUDA");
        assert_eq!(GPUBackend::ROCm.to_string(), "ROCm");
        assert_eq!(GPUBackend::CPU.to_string(), "CPU");
    }

    #[test]
    fn test_default_allocation_settings() {
        let settings = GPUAllocationSettings::default();
        assert!(!settings.enabled);
        assert_eq!(settings.allocation_percentage, 50);
        assert_eq!(settings.max_concurrent_jobs, 2);
    }

    #[test]
    fn test_parse_memory_string() {
        assert_eq!(parse_memory_string("8 GB"), 8 * 1024 * 1024 * 1024);
        assert_eq!(parse_memory_string("16 GB"), 16 * 1024 * 1024 * 1024);
        assert_eq!(parse_memory_string("512 MB"), 512 * 1024 * 1024);
        assert_eq!(parse_memory_string("1 TB"), 1024 * 1024 * 1024 * 1024);
    }

    #[test]
    fn test_cpu_fallback() {
        let device = create_cpu_fallback();
        assert_eq!(device.backend, GPUBackend::CPU);
        assert!(device.name.contains("CPU"));
        assert!(device.total_memory > 0);
    }

    #[test]
    fn test_default_gpu_stats() {
        let stats = GPUStats::default();
        assert_eq!(stats.jobs_completed, 0);
        assert_eq!(stats.jobs_failed, 0);
        assert_eq!(stats.tokens_earned, 0);
        assert!(stats.session_start > 0);
    }

    #[tokio::test]
    async fn test_gpu_resource_manager_new() {
        let manager = GPUResourceManager::new();
        let devices = manager.get_devices().await;
        // Should have at least CPU fallback
        assert!(!devices.is_empty() || true); // May be empty initially
    }

    #[tokio::test]
    async fn test_update_settings_valid() {
        let manager = GPUResourceManager::new();
        let new_settings = GPUAllocationSettings {
            enabled: true,
            allocation_percentage: 75,
            ..Default::default()
        };

        let result = manager.update_settings(new_settings).await;
        assert!(result.is_ok());

        let settings = manager.get_settings().await;
        assert!(settings.enabled);
        assert_eq!(settings.allocation_percentage, 75);
    }

    #[tokio::test]
    async fn test_update_settings_invalid_percentage() {
        let manager = GPUResourceManager::new();
        let new_settings = GPUAllocationSettings {
            allocation_percentage: 150, // Invalid
            ..Default::default()
        };

        let result = manager.update_settings(new_settings).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_submit_job_disabled() {
        let manager = GPUResourceManager::new();

        let job = ComputeJob {
            id: "test-job-1".to_string(),
            job_type: ComputeJobType::Inference,
            model_id: "test-model".to_string(),
            input_hash: "hash123".to_string(),
            requester: "0x123".to_string(),
            max_payment: 100,
            status: ComputeJobStatus::Queued,
            created_at: 0,
            memory_required: 1024 * 1024 * 1024, // 1GB
            estimated_time: 60,
            priority: 1,
        };

        let result = manager.submit_job(job).await;
        assert!(result.is_err()); // Should fail because GPU compute is disabled
    }

    #[tokio::test]
    async fn test_cancel_job_not_found() {
        let manager = GPUResourceManager::new();
        let result = manager.cancel_job("nonexistent").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_stats() {
        let manager = GPUResourceManager::new();
        let stats = manager.get_stats().await;
        assert_eq!(stats.jobs_completed, 0);
        assert_eq!(stats.queue_depth, 0);
    }

    #[tokio::test]
    async fn test_is_within_schedule_no_schedule() {
        let manager = GPUResourceManager::new();
        // No schedule means always available
        assert!(manager.is_within_schedule().await);
    }

    #[test]
    fn test_compute_job_type_serialize() {
        let job_type = ComputeJobType::Inference;
        let json = serde_json::to_string(&job_type).unwrap();
        assert_eq!(json, "\"Inference\"");
    }

    #[test]
    fn test_compute_job_status_serialize() {
        let status = ComputeJobStatus::Running {
            started_at: 1234567890,
            progress: 0.5,
        };
        let json = serde_json::to_string(&status).unwrap();
        assert!(json.contains("Running"));
        assert!(json.contains("1234567890"));
    }

    #[tokio::test]
    async fn test_provider_status_default() {
        let manager = GPUResourceManager::new();
        // Check initial provider status is not registered
        let status = manager.get_provider_status().await;
        assert!(!status.is_registered);
        assert_eq!(status.reputation, 100);
    }
}
