//! Local GGUF model backend
//!
//! Provides local LLM inference using GGUF models.
//! When the `local-llm` feature is enabled, uses llama-cpp-2 bindings.
//! Otherwise, provides a graceful fallback.

use async_trait::async_trait;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;

use super::{LLMBackend, LLMConfig, LLMError};
use crate::agent::context::ContextWindow;

#[cfg(feature = "local-llm")]
use llama_cpp_2::{
    context::params::LlamaContextParams,
    llama_backend::LlamaBackend,
    llama_batch::LlamaBatch,
    model::{params::LlamaModelParams, AddBos, LlamaModel},
    sampling::LlamaSampler,
};

#[cfg(feature = "local-llm")]
use std::num::NonZeroU32;

#[cfg(feature = "local-llm")]
use std::sync::OnceLock;

/// Global llama backend singleton - can only be initialized once per process
#[cfg(feature = "local-llm")]
static LLAMA_BACKEND: OnceLock<LlamaBackend> = OnceLock::new();

/// Holds the loaded model for inference
#[cfg(feature = "local-llm")]
struct LoadedModel {
    model: LlamaModel,
}

/// GGUF model backend for local inference
pub struct GGUFBackend {
    config: LLMConfig,
    model_path: Option<PathBuf>,
    loaded: Arc<RwLock<bool>>,
    #[cfg(feature = "local-llm")]
    model: Arc<RwLock<Option<LoadedModel>>>,
}

impl GGUFBackend {
    pub fn new(config: LLMConfig) -> Self {
        let model_path = config.local_model_path.as_ref().map(PathBuf::from);

        // Check if model exists and auto-mark as loaded if file is valid
        let model_exists = if let Some(ref path) = model_path {
            if path.exists() && path.extension().map_or(false, |ext| ext == "gguf") {
                tracing::info!("GGUF model file found: {}", path.display());
                true
            } else {
                tracing::warn!("GGUF model path provided but file not found or invalid: {}", path.display());
                false
            }
        } else {
            false
        };

        #[cfg(feature = "local-llm")]
        let (model, loaded) = if model_exists {
            if let Some(ref path) = model_path {
                tracing::info!("Attempting to load GGUF model from: {}", path.display());

                match Self::load_llama_model_sync(path) {
                    Ok(m) => {
                        tracing::info!("Successfully loaded GGUF model: {}", path.display());
                        (Arc::new(RwLock::new(Some(m))), Arc::new(RwLock::new(true)))
                    }
                    Err(e) => {
                        tracing::error!("Failed to load GGUF model: {}", e);
                        (Arc::new(RwLock::new(None)), Arc::new(RwLock::new(false)))
                    }
                }
            } else {
                tracing::warn!("No model path provided");
                (Arc::new(RwLock::new(None)), Arc::new(RwLock::new(false)))
            }
        } else {
            tracing::warn!("Model file does not exist or is not a .gguf file");
            (Arc::new(RwLock::new(None)), Arc::new(RwLock::new(false)))
        };

        #[cfg(not(feature = "local-llm"))]
        let loaded = Arc::new(RwLock::new(model_exists));

        Self {
            config,
            model_path,
            loaded,
            #[cfg(feature = "local-llm")]
            model,
        }
    }

    #[cfg(feature = "local-llm")]
    fn load_llama_model_sync(path: &Path) -> Result<LoadedModel, String> {
        // Ensure path is valid UTF-8 for llama.cpp
        let path_str = path.to_str()
            .ok_or_else(|| format!("Model path is not valid UTF-8: {:?}", path))?;

        if path_str.is_empty() {
            return Err("Model path is empty".to_string());
        }

        // Verify file exists and is readable
        if !path.exists() {
            return Err(format!("Model file does not exist: {}", path_str));
        }

        let file_size = std::fs::metadata(path)
            .map(|m| m.len())
            .map_err(|e| format!("Cannot read model file metadata: {}", e))?;

        if file_size == 0 {
            return Err("Model file is empty".to_string());
        }

        tracing::info!(
            "Loading GGUF model: path={}, size={} bytes",
            path_str, file_size
        );

        // Get or initialize the global llama.cpp backend (singleton)
        let backend = LLAMA_BACKEND.get_or_init(|| {
            tracing::info!("Initializing LlamaBackend (singleton)");
            LlamaBackend::init().expect("Failed to initialize llama backend")
        });

        // Create model parameters - CPU inference by default
        // On macOS with Apple Silicon, Metal acceleration is used automatically
        let model_params = LlamaModelParams::default();

        // Attempt to load the model - this may take a few seconds for large models
        tracing::info!("Loading model file...");

        let model = LlamaModel::load_from_file(backend, path_str, &model_params)
            .map_err(|e| format!("Failed to load GGUF model: {:?}", e))?;

        tracing::info!("GGUF model loaded successfully!");

        Ok(LoadedModel { model })
    }

    /// Load the model from path
    pub async fn load_model(&self) -> Result<(), LLMError> {
        let model_path = self
            .model_path
            .as_ref()
            .ok_or_else(|| LLMError("No model path configured".to_string()))?;

        // Check if file exists
        if !model_path.exists() {
            return Err(LLMError(format!(
                "Model file not found: {}",
                model_path.display()
            )));
        }

        // Verify it's a GGUF file
        if model_path.extension().map_or(true, |ext| ext != "gguf") {
            return Err(LLMError("Model must be a .gguf file".to_string()));
        }

        #[cfg(feature = "local-llm")]
        {
            // Check if already loaded
            if self.model.read().await.is_some() {
                tracing::info!("Model already loaded");
                return Ok(());
            }

            // Load the model in a blocking task to avoid blocking async runtime
            let path = model_path.clone();
            let loaded_model = tokio::task::spawn_blocking(move || {
                Self::load_llama_model_sync(&path)
            })
            .await
            .map_err(|e| LLMError(format!("Task join error: {}", e)))?
            .map_err(|e| LLMError(e))?;

            *self.model.write().await = Some(loaded_model);
            *self.loaded.write().await = true;
            tracing::info!("Model loaded: {}", model_path.display());
        }

        #[cfg(not(feature = "local-llm"))]
        {
            // Mark as "loaded" for testing purposes
            tracing::warn!(
                "Local LLM feature not enabled. Model loading is simulated. \
                 Enable 'local-llm' feature for actual inference."
            );
            *self.loaded.write().await = true;
        }

        Ok(())
    }

    /// Unload the model
    pub async fn unload_model(&self) {
        *self.loaded.write().await = false;
        #[cfg(feature = "local-llm")]
        {
            *self.model.write().await = None;
        }
        tracing::info!("Model unloaded");
    }

    /// Check if model is loaded
    pub async fn is_loaded(&self) -> bool {
        *self.loaded.read().await
    }

    /// Format prompt for the model (ChatML format for Qwen2)
    fn format_prompt(&self, context: &ContextWindow) -> String {
        let mut prompt = String::new();

        // System prompt in ChatML format
        prompt.push_str("<|im_start|>system\n");
        prompt.push_str(&context.system_prompt);

        if let Some(ref ctx) = context.system_context {
            prompt.push_str("\n\nCurrent context:\n");
            prompt.push_str(&ctx.to_context_string());
        }

        prompt.push_str("<|im_end|>\n");

        // Conversation messages
        for msg in &context.messages {
            let role = match msg.role.as_str() {
                "assistant" => "assistant",
                "system" => "system",
                _ => "user",
            };
            prompt.push_str(&format!(
                "<|im_start|>{}\n{}<|im_end|>\n",
                role, msg.content
            ));
        }

        // Start of assistant response
        prompt.push_str("<|im_start|>assistant\n");

        prompt
    }

    /// Get model info
    pub fn get_model_info(&self) -> Option<GGUFModelInfo> {
        self.model_path.as_ref().map(|path| {
            let name = path
                .file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| "unknown".to_string());

            let size = std::fs::metadata(path)
                .map(|m| m.len())
                .unwrap_or(0);

            GGUFModelInfo {
                name,
                size_bytes: size,
                context_length: self.config.context_size.unwrap_or(4096) as usize,
                quantization: extract_quantization(&path.to_string_lossy()),
                path: path.to_string_lossy().to_string(),
            }
        })
    }

    #[cfg(feature = "local-llm")]
    async fn run_inference(&self, prompt: &str) -> Result<String, LLMError> {
        tracing::debug!("Starting inference, prompt length: {} chars", prompt.len());

        // Clone values needed for the blocking task
        let model_arc = self.model.clone();
        let max_tokens = self.config.max_tokens;
        let context_size = self.config.context_size.unwrap_or(4096) as u32;
        let prompt_owned = prompt.to_string();

        // Run inference in a blocking task since llama.cpp is synchronous
        let result = tokio::task::spawn_blocking(move || {
            // Get the model
            let model_guard = model_arc.blocking_read();
            let loaded = model_guard.as_ref()
                .ok_or_else(|| "Model not loaded".to_string())?;

            // Get the global backend
            let backend = LLAMA_BACKEND.get()
                .ok_or_else(|| "Llama backend not initialized".to_string())?;

            // Create context parameters
            let ctx_params = LlamaContextParams::default()
                .with_n_ctx(NonZeroU32::new(context_size));

            // Create inference context
            let mut ctx = loaded.model.new_context(backend, ctx_params)
                .map_err(|e| format!("Failed to create context: {:?}", e))?;

            // Tokenize the prompt
            let tokens = loaded.model.str_to_token(&prompt_owned, AddBos::Always)
                .map_err(|e| format!("Failed to tokenize prompt: {:?}", e))?;

            if tokens.is_empty() {
                return Err("Empty prompt after tokenization".to_string());
            }

            // Create batch and add tokens - use larger batch for the prompt
            let batch_size = std::cmp::max(tokens.len() + 256, 512);
            let mut batch = LlamaBatch::new(batch_size, 1);
            let last_idx = tokens.len() as i32 - 1;
            for (i, token) in tokens.iter().enumerate() {
                batch.add(*token, i as i32, &[0], i as i32 == last_idx)
                    .map_err(|e| format!("Failed to add token to batch: {:?}", e))?;
            }

            // Process the prompt (prefill)
            ctx.decode(&mut batch)
                .map_err(|e| format!("Failed to decode prompt: {:?}", e))?;

            // Create sampler chain - use temperature for variety, then greedy selection
            let mut sampler = LlamaSampler::chain_simple([
                LlamaSampler::dist(1234),  // seed for reproducibility
                LlamaSampler::greedy(),    // final selection
            ]);

            // Generate tokens
            let mut output = String::new();
            let mut n_cur = batch.n_tokens();
            let n_len = n_cur + max_tokens as i32;

            tracing::info!("Starting token generation: n_cur={}, n_len={}, max_tokens={}", n_cur, n_len, max_tokens);

            while n_cur <= n_len {
                // Sample next token from the last position in the batch
                let token = sampler.sample(&ctx, batch.n_tokens() - 1);
                sampler.accept(token);

                tracing::debug!("Sampled token: {:?}", token);

                // Check for end-of-generation
                if loaded.model.is_eog_token(token) {
                    tracing::info!("EOG token detected, stopping generation");
                    break;
                }

                // Decode token to bytes and then to string
                match loaded.model.token_to_bytes(token, llama_cpp_2::model::Special::Tokenize) {
                    Ok(bytes) => {
                        let output_string = String::from_utf8_lossy(&bytes);

                        // Check for ChatML end tokens
                        if output_string.contains("<|im_end|>") || output_string.contains("<|endoftext|>") {
                            if let Some(pos) = output_string.find("<|im_end|>") {
                                output.push_str(&output_string[..pos]);
                            } else if let Some(pos) = output_string.find("<|endoftext|>") {
                                output.push_str(&output_string[..pos]);
                            }
                            break;
                        }
                        output.push_str(&output_string);
                    }
                    Err(_) => {
                        // Skip tokens that can't be decoded
                    }
                }

                // Prepare batch for next token
                batch.clear();
                batch.add(token, n_cur, &[0], true)
                    .map_err(|e| format!("Failed to add generated token: {:?}", e))?;

                n_cur += 1;

                // Decode the new token to get logits for next sample
                ctx.decode(&mut batch)
                    .map_err(|e| format!("Failed to decode token: {:?}", e))?;
            }

            // Clean up any partial ChatML tags at the end
            if let Some(pos) = output.rfind("<|im_") {
                output.truncate(pos);
            }

            tracing::info!("Generation complete: {} chars", output.len());
            Ok::<String, String>(output.trim().to_string())
        })
        .await
        .map_err(|e| LLMError(format!("Task join error: {}", e)))?
        .map_err(|e| LLMError(e))?;

        tracing::debug!("Generated {} chars", result.len());
        Ok(result)
    }
}

#[async_trait]
impl LLMBackend for GGUFBackend {
    fn name(&self) -> &str {
        "local-gguf"
    }

    async fn complete(&self, context: &ContextWindow) -> Result<String, LLMError> {
        if !*self.loaded.read().await {
            return Err(LLMError(
                "Model not loaded. Call load_model() first or use API backend.".to_string(),
            ));
        }

        let prompt = self.format_prompt(context);
        tracing::debug!("Generated prompt ({} chars)", prompt.len());

        #[cfg(feature = "local-llm")]
        {
            return self.run_inference(&prompt).await;
        }

        #[cfg(not(feature = "local-llm"))]
        {
            // Without the feature, return a helpful message
            Err(LLMError(format!(
                "Local LLM inference not available. Enable 'local-llm' feature to use GGUF models.\n\
                 Formatted prompt ({} chars) would be:\n{}...",
                prompt.len(),
                &prompt[..prompt.len().min(200)]
            )))
        }
    }

    fn is_available(&self) -> bool {
        self.model_path.as_ref().map_or(false, |p| p.exists())
    }

    fn config(&self) -> &LLMConfig {
        &self.config
    }
}

/// GGUF model metadata
#[derive(Debug, Clone)]
pub struct GGUFModelInfo {
    pub name: String,
    pub size_bytes: u64,
    pub context_length: usize,
    pub quantization: String,
    pub path: String,
}

impl GGUFModelInfo {
    /// Format size as human-readable string
    pub fn size_human(&self) -> String {
        let bytes = self.size_bytes as f64;
        if bytes >= 1_000_000_000.0 {
            format!("{:.1} GB", bytes / 1_000_000_000.0)
        } else if bytes >= 1_000_000.0 {
            format!("{:.1} MB", bytes / 1_000_000.0)
        } else {
            format!("{:.0} KB", bytes / 1_000.0)
        }
    }
}

/// Scan directory for GGUF models
pub fn scan_for_models(directory: &str) -> Vec<GGUFModelInfo> {
    let path = Path::new(directory);
    if !path.exists() || !path.is_dir() {
        return Vec::new();
    }

    let mut models = Vec::new();

    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "gguf") {
                if let Some(filename) = path.file_name() {
                    let name = filename.to_string_lossy().to_string();
                    let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
                    let quantization = extract_quantization(&name);

                    models.push(GGUFModelInfo {
                        name: name.trim_end_matches(".gguf").to_string(),
                        size_bytes: size,
                        context_length: 4096, // Default, would read from model metadata
                        quantization,
                        path: path.to_string_lossy().to_string(),
                    });
                }
            }
        }
    }

    // Sort by size (largest first, usually best quality)
    models.sort_by(|a, b| b.size_bytes.cmp(&a.size_bytes));

    models
}

/// Extract quantization type from model filename
fn extract_quantization(filename: &str) -> String {
    let quantization_patterns = [
        "q2_k", "q3_k_s", "q3_k_m", "q3_k_l", "q4_0", "q4_1", "q4_k_s", "q4_k_m", "q5_0", "q5_1",
        "q5_k_s", "q5_k_m", "q6_k", "q8_0", "f16", "f32", "iq2_xxs", "iq2_xs", "iq3_xxs", "iq3_s",
        "iq4_xs", "iq4_nl",
    ];

    let filename_lower = filename.to_lowercase();
    for pattern in quantization_patterns {
        if filename_lower.contains(pattern) {
            return pattern.to_uppercase().replace('_', "_");
        }
    }

    "unknown".to_string()
}

/// Get the default models directory
pub fn get_default_models_dir() -> Option<PathBuf> {
    dirs::data_local_dir().map(|d| d.join("citrate").join("models"))
}

/// Ensure the models directory exists
pub fn ensure_models_dir() -> Result<PathBuf, std::io::Error> {
    let dir = get_default_models_dir()
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "No data directory"))?;

    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quantization_extraction() {
        assert_eq!(extract_quantization("llama-7b-q4_0.gguf"), "Q4_0");
        assert_eq!(extract_quantization("mistral-q5_k_m.gguf"), "Q5_K_M");
        assert_eq!(extract_quantization("model-f16.gguf"), "F16");
        assert_eq!(extract_quantization("model-iq4_xs.gguf"), "IQ4_XS");
        assert_eq!(extract_quantization("unknown-model.gguf"), "unknown");
    }

    #[test]
    fn test_prompt_formatting() {
        let config = LLMConfig {
            local_model_path: Some("/tmp/model.gguf".to_string()),
            ..Default::default()
        };
        let backend = GGUFBackend::new(config);

        let context = ContextWindow {
            system_prompt: "You are helpful.".to_string(),
            system_context: None,
            messages: vec![crate::agent::context::ContextMessage {
                role: "user".to_string(),
                content: "Hello".to_string(),
                name: None,
                tool_call_id: None,
            }],
            estimated_tokens: 0,
            was_truncated: false,
        };

        let prompt = backend.format_prompt(&context);
        assert!(prompt.contains("<|im_start|>system"));
        assert!(prompt.contains("You are helpful."));
        assert!(prompt.contains("<|im_start|>user"));
        assert!(prompt.contains("Hello"));
        assert!(prompt.contains("<|im_start|>assistant"));
    }

    #[test]
    fn test_model_info_size_human() {
        let info = GGUFModelInfo {
            name: "test".to_string(),
            size_bytes: 4_500_000_000,
            context_length: 4096,
            quantization: "Q4_0".to_string(),
            path: "/tmp/test.gguf".to_string(),
        };
        assert_eq!(info.size_human(), "4.5 GB");

        let info2 = GGUFModelInfo {
            size_bytes: 500_000_000,
            ..info.clone()
        };
        assert_eq!(info2.size_human(), "500.0 MB");
    }
}
