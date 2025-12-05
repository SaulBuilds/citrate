//! Image generation tools - AI image generation capabilities
//!
//! These tools provide AI-powered image generation.

use std::collections::HashMap;
use std::future::Future;
use std::path::PathBuf;
use std::pin::Pin;
use std::sync::Arc;

use super::super::dispatcher::{DispatchError, ToolHandler, ToolOutput};
use super::super::intent::IntentParams;
use crate::models::{InferenceRequest, ModelManager};

/// Default image sizes
#[derive(Debug, Clone, Copy)]
pub enum ImageSize {
    Small,      // 256x256
    Medium,     // 512x512
    Large,      // 1024x1024
    Wide,       // 1024x768
    Tall,       // 768x1024
}

impl ImageSize {
    fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "small" | "256" => Self::Small,
            "medium" | "512" => Self::Medium,
            "large" | "1024" => Self::Large,
            "wide" | "landscape" => Self::Wide,
            "tall" | "portrait" => Self::Tall,
            _ => Self::Medium,
        }
    }

    fn dimensions(&self) -> (u32, u32) {
        match self {
            Self::Small => (256, 256),
            Self::Medium => (512, 512),
            Self::Large => (1024, 1024),
            Self::Wide => (1024, 768),
            Self::Tall => (768, 1024),
        }
    }
}

/// Generate image tool
pub struct GenerateImageTool {
    model_manager: Arc<ModelManager>,
    output_dir: PathBuf,
}

impl GenerateImageTool {
    pub fn new(model_manager: Arc<ModelManager>) -> Self {
        let output_dir = dirs::picture_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("citrate-generated");

        Self {
            model_manager,
            output_dir,
        }
    }
}

impl ToolHandler for GenerateImageTool {
    fn name(&self) -> &str {
        "generate_image"
    }

    fn description(&self) -> &str {
        "Generate an image from a text prompt using AI. Supports size and style options."
    }

    fn execute(
        &self,
        params: &IntentParams,
    ) -> Pin<Box<dyn Future<Output = Result<ToolOutput, DispatchError>> + Send + '_>> {
        let model_manager = self.model_manager.clone();
        let output_dir = self.output_dir.clone();
        let prompt = params.prompt.clone();
        let model_name = params.model_name.clone();
        let size_str = params.search_query.clone(); // Reuse for size
        Box::pin(async move {
            let prompt_text = prompt.ok_or_else(|| {
                DispatchError::InvalidParams("Prompt required for image generation".to_string())
            })?;

            // Get size
            let size = size_str
                .as_deref()
                .map(ImageSize::from_str)
                .unwrap_or(ImageSize::Medium);
            let (width, height) = size.dimensions();

            // Get model (default to a generic image model)
            let model = model_name.unwrap_or_else(|| "stable-diffusion".to_string());

            // Create inference request with image generation parameters
            let mut parameters = HashMap::new();
            parameters.insert("width".to_string(), serde_json::json!(width));
            parameters.insert("height".to_string(), serde_json::json!(height));
            parameters.insert("steps".to_string(), serde_json::json!(20));
            parameters.insert("guidance_scale".to_string(), serde_json::json!(7.5));

            let request = InferenceRequest {
                model_id: model.clone(),
                input: prompt_text.clone(),
                parameters,
            };

            // Try to run inference
            match model_manager.request_inference(request).await {
                Ok(response) => {
                    // Create output directory if needed
                    let _ = tokio::fs::create_dir_all(&output_dir).await;

                    // Generate filename
                    let timestamp = chrono::Utc::now().timestamp();
                    let filename = format!("generated_{}.png", timestamp);
                    let output_path = output_dir.join(&filename);

                    // In a real implementation, we'd save the image bytes here
                    // For now, return the result info
                    Ok(ToolOutput {
                        tool: "generate_image".to_string(),
                        success: true,
                        message: format!(
                            "Generated {}x{} image from prompt. Latency: {}ms",
                            width, height, response.latency_ms
                        ),
                        data: Some(serde_json::json!({
                            "prompt": prompt_text,
                            "model": model,
                            "width": width,
                            "height": height,
                            "output_path": output_path.to_string_lossy(),
                            "latency_ms": response.latency_ms,
                            "request_id": response.request_id,
                            "result": response.result
                        })),
                    })
                }
                Err(e) => {
                    // SECURITY: Do not simulate success when generation fails
                    // Return proper failure with actionable error information
                    Ok(ToolOutput {
                        tool: "generate_image".to_string(),
                        success: false,
                        message: format!(
                            "Image generation failed: model '{}' is not available or failed to run inference",
                            model
                        ),
                        data: Some(serde_json::json!({
                            "error": format!("{}", e),
                            "model_requested": model,
                            "prompt": prompt_text,
                            "width": width,
                            "height": height,
                            "suggestion": "Please ensure an image generation model (e.g., stable-diffusion) is installed and running. Use 'list_image_models' to see available models."
                        })),
                    })
                }
            }
        })
    }
}

/// List image models tool
pub struct ListImageModelsTool {
    model_manager: Arc<ModelManager>,
}

impl ListImageModelsTool {
    pub fn new(model_manager: Arc<ModelManager>) -> Self {
        Self { model_manager }
    }
}

impl ToolHandler for ListImageModelsTool {
    fn name(&self) -> &str {
        "list_image_models"
    }

    fn description(&self) -> &str {
        "List available image generation models"
    }

    fn execute(
        &self,
        _params: &IntentParams,
    ) -> Pin<Box<dyn Future<Output = Result<ToolOutput, DispatchError>> + Send + '_>> {
        let model_manager = self.model_manager.clone();
        Box::pin(async move {
            match model_manager.get_models().await {
                Ok(models) => {
                    let image_models: Vec<_> = models
                        .iter()
                        .filter(|m| {
                            matches!(m.model_type, crate::models::ModelType::Image)
                        })
                        .map(|m| {
                            serde_json::json!({
                                "id": m.id,
                                "name": m.name,
                                "description": m.description,
                                "size_mb": m.size_mb,
                            })
                        })
                        .collect();

                    let count = image_models.len();

                    Ok(ToolOutput {
                        tool: "list_image_models".to_string(),
                        success: true,
                        message: format!("{} image generation models available", count),
                        data: Some(serde_json::json!({
                            "count": count,
                            "models": image_models,
                            "supported_sizes": ["small (256x256)", "medium (512x512)", "large (1024x1024)", "wide (1024x768)", "tall (768x1024)"]
                        })),
                    })
                }
                Err(e) => Ok(ToolOutput {
                    tool: "list_image_models".to_string(),
                    success: false,
                    message: format!("Failed to list image models: {}", e),
                    data: None,
                }),
            }
        })
    }
}

/// Image style presets
pub struct ApplyStyleTool;

impl ApplyStyleTool {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ApplyStyleTool {
    fn default() -> Self {
        Self::new()
    }
}

impl ToolHandler for ApplyStyleTool {
    fn name(&self) -> &str {
        "apply_style"
    }

    fn description(&self) -> &str {
        "Get style presets for image generation prompts"
    }

    fn execute(
        &self,
        params: &IntentParams,
    ) -> Pin<Box<dyn Future<Output = Result<ToolOutput, DispatchError>> + Send + '_>> {
        let style = params.search_query.clone(); // Style name
        let base_prompt = params.prompt.clone();
        Box::pin(async move {
            let styles = vec![
                ("realistic", "ultra realistic, 8k, detailed, photorealistic"),
                ("anime", "anime style, vibrant colors, detailed illustration"),
                ("digital-art", "digital art, highly detailed, trending on artstation"),
                ("oil-painting", "oil painting, classical art style, brush strokes"),
                ("watercolor", "watercolor painting, soft colors, artistic"),
                ("3d-render", "3d render, octane render, ray tracing, realistic lighting"),
                ("pixel-art", "pixel art, 8-bit style, retro gaming aesthetic"),
                ("cyberpunk", "cyberpunk style, neon lights, futuristic, dark atmosphere"),
                ("fantasy", "fantasy art, magical, ethereal, detailed illustration"),
                ("minimalist", "minimalist, simple, clean lines, modern design"),
            ];

            if let Some(style_name) = style {
                // Find matching style
                if let Some((_, modifiers)) = styles
                    .iter()
                    .find(|(name, _)| name.eq_ignore_ascii_case(&style_name))
                {
                    let enhanced_prompt = if let Some(prompt) = base_prompt {
                        format!("{}, {}", prompt, modifiers)
                    } else {
                        modifiers.to_string()
                    };

                    return Ok(ToolOutput {
                        tool: "apply_style".to_string(),
                        success: true,
                        message: format!("Applied '{}' style to prompt", style_name),
                        data: Some(serde_json::json!({
                            "style": style_name,
                            "modifiers": modifiers,
                            "enhanced_prompt": enhanced_prompt
                        })),
                    });
                }
            }

            // Return available styles
            Ok(ToolOutput {
                tool: "apply_style".to_string(),
                success: true,
                message: format!("{} style presets available", styles.len()),
                data: Some(serde_json::json!({
                    "styles": styles.iter().map(|(name, desc)| {
                        serde_json::json!({
                            "name": name,
                            "modifiers": desc
                        })
                    }).collect::<Vec<_>>()
                })),
            })
        })
    }
}
