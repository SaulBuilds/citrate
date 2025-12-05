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

/// GGUF model backend for local inference
pub struct GGUFBackend {
    config: LLMConfig,
    model_path: Option<PathBuf>,
    loaded: Arc<RwLock<bool>>,
    // When local-llm feature is enabled, this would hold the actual model
    #[cfg(feature = "local-llm")]
    model: Option<Arc<RwLock<LlamaModel>>>,
}

impl GGUFBackend {
    pub fn new(config: LLMConfig) -> Self {
        let model_path = config.local_model_path.as_ref().map(PathBuf::from);
        Self {
            config,
            model_path,
            loaded: Arc::new(RwLock::new(false)),
            #[cfg(feature = "local-llm")]
            model: None,
        }
    }

    /// Load the model from path
    pub async fn load_model(&mut self) -> Result<(), LLMError> {
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
            // Load actual model using llama-cpp-2
            // This would be implemented when the feature is enabled
            self.load_model_impl(model_path).await?;
        }

        #[cfg(not(feature = "local-llm"))]
        {
            // Mark as "loaded" for testing purposes
            // In production, this would fail without the feature
            tracing::warn!(
                "Local LLM feature not enabled. Model loading is simulated. \
                 Enable 'local-llm' feature for actual inference."
            );
        }

        *self.loaded.write().await = true;
        tracing::info!("Model loaded: {}", model_path.display());

        Ok(())
    }

    #[cfg(feature = "local-llm")]
    async fn load_model_impl(&mut self, model_path: &Path) -> Result<(), LLMError> {
        // When llama-cpp-2 is available, this would:
        // 1. Initialize llama context
        // 2. Load the model file
        // 3. Configure sampling parameters
        // 4. Store in self.model

        // Placeholder for actual implementation:
        // use llama_cpp_2::model::LlamaModel;
        // let model = LlamaModel::load_from_file(model_path, Default::default())
        //     .map_err(|e| LLMError(format!("Failed to load model: {}", e)))?;
        // self.model = Some(Arc::new(RwLock::new(model)));

        Ok(())
    }

    /// Unload the model
    pub async fn unload_model(&mut self) {
        *self.loaded.write().await = false;
        #[cfg(feature = "local-llm")]
        {
            self.model = None;
        }
        tracing::info!("Model unloaded");
    }

    /// Check if model is loaded
    pub async fn is_loaded(&self) -> bool {
        *self.loaded.read().await
    }

    /// Format prompt for the model (ChatML format)
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

        #[cfg(feature = "local-llm")]
        {
            return self.complete_impl(context).await;
        }

        #[cfg(not(feature = "local-llm"))]
        {
            // Without the feature, return a helpful message
            let prompt = self.format_prompt(context);
            Err(LLMError(format!(
                "Local LLM inference not available. Enable 'local-llm' feature to use GGUF models.\n\
                 Formatted prompt ({} chars) would be:\n{}...",
                prompt.len(),
                &prompt[..prompt.len().min(200)]
            )))
        }
    }

    #[cfg(feature = "local-llm")]
    async fn complete_impl(&self, context: &ContextWindow) -> Result<String, LLMError> {
        let prompt = self.format_prompt(context);

        // When llama-cpp-2 is available:
        // let model = self.model.as_ref()
        //     .ok_or_else(|| LLMError("Model not initialized".to_string()))?;
        //
        // let model_guard = model.read().await;
        // let mut ctx = model_guard.create_context(Default::default())
        //     .map_err(|e| LLMError(format!("Context creation failed: {}", e)))?;
        //
        // let tokens = ctx.tokenize(&prompt, true)
        //     .map_err(|e| LLMError(format!("Tokenization failed: {}", e)))?;
        //
        // let mut output = String::new();
        // for token in ctx.generate(tokens, self.config.max_tokens as usize) {
        //     let text = ctx.decode(&[token])
        //         .map_err(|e| LLMError(format!("Decode failed: {}", e)))?;
        //     output.push_str(&text);
        // }
        //
        // Ok(output)

        Err(LLMError("Local LLM implementation pending".to_string()))
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
