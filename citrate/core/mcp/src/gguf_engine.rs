// citrate/core/mcp/src/gguf_engine.rs

/// GGUF Model Inference Engine using llama.cpp
use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::Command;
use tokio::fs;
use tracing::{debug, info, warn};

/// GGUF model types supported
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelType {
    /// Embedding model (e.g., BGE-M3)
    Embedding,
    /// Text generation model (e.g., Mistral, Llama)
    TextGeneration,
}

/// GGUF inference engine configuration
#[derive(Debug, Clone)]
pub struct GGUFEngineConfig {
    /// Path to llama.cpp build directory
    pub llama_cpp_path: PathBuf,
    /// Path to models directory for caching
    pub models_dir: PathBuf,
    /// Number of threads for inference
    pub threads: usize,
    /// Context size for LLMs
    pub context_size: usize,
}

impl Default for GGUFEngineConfig {
    fn default() -> Self {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        Self {
            llama_cpp_path: home.join("llama.cpp"),
            models_dir: home.join(".citrate/models"),
            threads: num_cpus::get(),
            context_size: 2048,
        }
    }
}

/// GGUF inference engine
pub struct GGUFEngine {
    config: GGUFEngineConfig,
}

impl GGUFEngine {
    /// Create a new GGUF inference engine
    pub fn new(config: GGUFEngineConfig) -> Result<Self> {
        // Verify llama.cpp exists
        let main_binary = config.llama_cpp_path.join("build/bin/llama-cli");
        let embedding_binary = config.llama_cpp_path.join("build/bin/llama-embedding");

        if !main_binary.exists() && !config.llama_cpp_path.join("build/bin/main").exists() {
            warn!(
                "llama.cpp binary not found at {:?}, inference will be limited",
                main_binary
            );
        }

        if !embedding_binary.exists() && !config.llama_cpp_path.join("build/bin/embedding").exists() {
            warn!(
                "llama.cpp embedding binary not found at {:?}",
                embedding_binary
            );
        }

        // Create models directory if it doesn't exist
        std::fs::create_dir_all(&config.models_dir)?;

        Ok(Self { config })
    }

    /// Execute text generation inference
    pub async fn generate_text(
        &self,
        model_path: &Path,
        prompt: &str,
        max_tokens: usize,
        temperature: f32,
    ) -> Result<String> {
        info!(
            "Generating text with model: {:?}, max_tokens: {}, temp: {}",
            model_path, max_tokens, temperature
        );

        // Find llama.cpp binary (try both old and new names)
        let binary = self.find_llama_binary("llama-cli", "main")?;

        // Build command
        let output = Command::new(binary)
            .arg("-m")
            .arg(model_path)
            .arg("-p")
            .arg(prompt)
            .arg("-n")
            .arg(max_tokens.to_string())
            .arg("--temp")
            .arg(temperature.to_string())
            .arg("-t")
            .arg(self.config.threads.to_string())
            .arg("-c")
            .arg(self.config.context_size.to_string())
            .arg("--no-display-prompt")
            .output()
            .context("Failed to execute llama.cpp")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("llama.cpp execution failed: {}", stderr));
        }

        let text = String::from_utf8_lossy(&output.stdout);
        Ok(text.trim().to_string())
    }

    /// Execute embedding inference
    pub async fn generate_embeddings(
        &self,
        model_path: &Path,
        texts: &[String],
    ) -> Result<Vec<Vec<f32>>> {
        info!(
            "Generating embeddings with model: {:?} for {} texts",
            model_path,
            texts.len()
        );

        let binary = self.find_llama_binary("llama-embedding", "embedding")?;

        let mut all_embeddings = Vec::new();

        for text in texts {
            let output = Command::new(&binary)
                .arg("-m")
                .arg(model_path)
                .arg("-p")
                .arg(text)
                .arg("-t")
                .arg(self.config.threads.to_string())
                .output()
                .context("Failed to execute llama-embedding")?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                warn!("Embedding generation failed: {}", stderr);
                // Return zero embedding as fallback
                all_embeddings.push(vec![0.0; 1024]);
                continue;
            }

            // Parse embedding output (llama.cpp outputs embeddings as JSON or space-separated)
            let output_str = String::from_utf8_lossy(&output.stdout);
            let embedding = self.parse_embedding_output(&output_str)?;
            all_embeddings.push(embedding);
        }

        Ok(all_embeddings)
    }

    /// Execute chat completion with message history
    pub async fn chat_completion(
        &self,
        model_path: &Path,
        messages: &[ChatMessage],
        max_tokens: usize,
        temperature: f32,
    ) -> Result<String> {
        // Format messages into a prompt
        let prompt = self.format_chat_prompt(messages);

        // Use standard text generation
        self.generate_text(model_path, &prompt, max_tokens, temperature)
            .await
    }

    /// Load model from bytes (for genesis-embedded models)
    pub async fn load_model_from_bytes(
        &self,
        model_id: &str,
        model_bytes: &[u8],
    ) -> Result<PathBuf> {
        let model_path = self.config.models_dir.join(format!("{}.gguf", model_id));

        // Check if already cached
        if model_path.exists() {
            let existing_size = fs::metadata(&model_path).await?.len();
            if existing_size == model_bytes.len() as u64 {
                debug!("Model {} already cached at {:?}", model_id, model_path);
                return Ok(model_path);
            }
        }

        // Write model to disk
        info!("Caching model {} to {:?}", model_id, model_path);
        fs::write(&model_path, model_bytes).await?;

        Ok(model_path)
    }

    /// Get model path from IPFS download
    pub fn get_ipfs_model_path(&self, model_id: &str) -> PathBuf {
        self.config.models_dir.join(format!("{}.gguf", model_id))
    }

    /// Find llama.cpp binary (supporting both old and new naming)
    fn find_llama_binary(&self, new_name: &str, old_name: &str) -> Result<PathBuf> {
        let new_path = self.config.llama_cpp_path.join("build/bin").join(new_name);
        let old_path = self.config.llama_cpp_path.join("build/bin").join(old_name);

        if new_path.exists() {
            Ok(new_path)
        } else if old_path.exists() {
            Ok(old_path)
        } else {
            Err(anyhow!(
                "llama.cpp binary not found. Tried: {:?} and {:?}",
                new_path,
                old_path
            ))
        }
    }

    /// Parse embedding output from llama.cpp
    fn parse_embedding_output(&self, output: &str) -> Result<Vec<f32>> {
        // llama.cpp outputs embeddings in various formats
        // Try JSON first
        if let Ok(json_array) = serde_json::from_str::<Vec<f32>>(output.trim()) {
            return Ok(json_array);
        }

        // Try space-separated
        let values: Result<Vec<f32>, _> = output
            .trim()
            .split_whitespace()
            .map(|s| s.parse::<f32>())
            .collect();

        if let Ok(embedding) = values {
            if !embedding.is_empty() {
                return Ok(embedding);
            }
        }

        // Fallback: return zero embedding
        warn!("Failed to parse embedding output, using zeros");
        Ok(vec![0.0; 1024])
    }

    /// Format chat messages into a prompt
    fn format_chat_prompt(&self, messages: &[ChatMessage]) -> String {
        let mut prompt = String::new();

        for msg in messages {
            match msg.role.as_str() {
                "system" => prompt.push_str(&format!("### System:\n{}\n\n", msg.content)),
                "user" => prompt.push_str(&format!("### User:\n{}\n\n", msg.content)),
                "assistant" => prompt.push_str(&format!("### Assistant:\n{}\n\n", msg.content)),
                _ => prompt.push_str(&format!("### {}:\n{}\n\n", msg.role, msg.content)),
            }
        }

        prompt.push_str("### Assistant:\n");
        prompt
    }
}

/// Chat message for structured conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

/// Compute cosine similarity between two embeddings
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() {
        return 0.0;
    }

    let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let magnitude_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let magnitude_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

    if magnitude_a == 0.0 || magnitude_b == 0.0 {
        return 0.0;
    }

    dot_product / (magnitude_a * magnitude_b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cosine_similarity() {
        let a = vec![1.0, 2.0, 3.0];
        let b = vec![4.0, 5.0, 6.0];
        let sim = cosine_similarity(&a, &b);
        assert!(sim > 0.9); // Similar vectors
    }

    #[test]
    fn test_format_chat_prompt() {
        let config = GGUFEngineConfig::default();
        let engine = GGUFEngine::new(config).unwrap();

        let messages = vec![
            ChatMessage {
                role: "user".to_string(),
                content: "Hello".to_string(),
            },
        ];

        let prompt = engine.format_chat_prompt(&messages);
        assert!(prompt.contains("### User:"));
        assert!(prompt.contains("Hello"));
        assert!(prompt.contains("### Assistant:"));
    }
}
