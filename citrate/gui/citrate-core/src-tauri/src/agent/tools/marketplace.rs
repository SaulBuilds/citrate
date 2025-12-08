//! Marketplace tools - search and discovery of models/assets
//!
//! These tools provide marketplace interaction capabilities.

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use super::super::dispatcher::{DispatchError, ToolHandler, ToolOutput};
use super::super::intent::IntentParams;
use crate::models::ModelManager;

/// Search marketplace tool - find models and assets
pub struct SearchMarketplaceTool {
    model_manager: Arc<ModelManager>,
}

impl SearchMarketplaceTool {
    pub fn new(model_manager: Arc<ModelManager>) -> Self {
        Self { model_manager }
    }
}

impl ToolHandler for SearchMarketplaceTool {
    fn name(&self) -> &str {
        "search_marketplace"
    }

    fn description(&self) -> &str {
        "Search the Citrate marketplace for AI models and assets. Supports filtering by type, price, and keywords."
    }

    fn execute(
        &self,
        params: &IntentParams,
    ) -> Pin<Box<dyn Future<Output = Result<ToolOutput, DispatchError>> + Send + '_>> {
        let model_manager = self.model_manager.clone();
        let query = params.search_query.clone();
        let model_type = params.model_name.clone(); // Reuse as filter type
        Box::pin(async move {
            // Get all models
            match model_manager.get_models().await {
                Ok(models) => {
                    // Filter by query if provided
                    let filtered: Vec<_> = models
                        .iter()
                        .filter(|m| {
                            if let Some(ref q) = query {
                                let q_lower = q.to_lowercase();
                                m.name.to_lowercase().contains(&q_lower)
                                    || m.description.to_lowercase().contains(&q_lower)
                            } else {
                                true
                            }
                        })
                        .filter(|m| {
                            if let Some(ref t) = model_type {
                                let type_str = format!("{:?}", m.model_type).to_lowercase();
                                match t.to_lowercase().as_str() {
                                    "language" | "llm" | "text" => type_str == "language",
                                    "image" | "vision" | "diffusion" => type_str == "image",
                                    "audio" | "speech" => type_str == "audio",
                                    "video" => type_str == "video",
                                    "multimodal" => type_str == "multimodal",
                                    _ => true,
                                }
                            } else {
                                true
                            }
                        })
                        .collect();

                    let results: Vec<serde_json::Value> = filtered
                        .iter()
                        .take(20) // Limit results
                        .map(|m| {
                            serde_json::json!({
                                "id": m.id,
                                "name": m.name,
                                "type": format!("{:?}", m.model_type),
                                "description": m.description,
                                "size_mb": m.size_mb,
                                "parameters": m.parameters,
                                "owner": m.owner,
                                "price": "Free", // Placeholder - would come from on-chain data
                            })
                        })
                        .collect();

                    let count = results.len();
                    let total = filtered.len();

                    Ok(ToolOutput {
                        tool: "search_marketplace".to_string(),
                        success: true,
                        message: if count == 0 {
                            format!(
                                "No models found matching '{}'",
                                query.as_deref().unwrap_or("all")
                            )
                        } else {
                            format!(
                                "Found {} models{}",
                                count,
                                if total > count {
                                    format!(" (showing first {})", count)
                                } else {
                                    String::new()
                                }
                            )
                        },
                        data: Some(serde_json::json!({
                            "query": query,
                            "filter_type": model_type,
                            "count": count,
                            "total": total,
                            "results": results
                        })),
                    })
                }
                Err(e) => Ok(ToolOutput {
                    tool: "search_marketplace".to_string(),
                    success: false,
                    message: format!("Failed to search marketplace: {}", e),
                    data: None,
                }),
            }
        })
    }
}

/// Get listing details tool
pub struct GetListingTool {
    model_manager: Arc<ModelManager>,
}

impl GetListingTool {
    pub fn new(model_manager: Arc<ModelManager>) -> Self {
        Self { model_manager }
    }
}

impl ToolHandler for GetListingTool {
    fn name(&self) -> &str {
        "get_listing"
    }

    fn description(&self) -> &str {
        "Get detailed information about a specific marketplace listing by ID"
    }

    fn execute(
        &self,
        params: &IntentParams,
    ) -> Pin<Box<dyn Future<Output = Result<ToolOutput, DispatchError>> + Send + '_>> {
        let model_manager = self.model_manager.clone();
        let listing_id = params.model_name.clone(); // Reuse model_name for listing ID
        Box::pin(async move {
            let id = listing_id.ok_or_else(|| {
                DispatchError::InvalidParams("Listing ID required".to_string())
            })?;

            match model_manager.get_model(&id).await {
                Ok(Some(model)) => Ok(ToolOutput {
                    tool: "get_listing".to_string(),
                    success: true,
                    message: format!(
                        "Listing '{}': {} ({} MB)",
                        model.name,
                        model.description,
                        model.size_mb
                    ),
                    data: Some(serde_json::json!({
                        "id": model.id,
                        "name": model.name,
                        "type": format!("{:?}", model.model_type),
                        "description": model.description,
                        "version": model.version,
                        "size_mb": model.size_mb,
                        "parameters": model.parameters,
                        "architecture": model.architecture,
                        "owner": model.owner,
                        "created_at": model.created_at,
                        "hash": model.hash,
                        "pricing": {
                            "base_price": "Free",
                            "per_inference": "0.0001 SALT"
                        },
                        "stats": {
                            "downloads": 0,
                            "rating": 0.0,
                            "reviews": 0
                        }
                    })),
                }),
                Ok(None) => Ok(ToolOutput {
                    tool: "get_listing".to_string(),
                    success: false,
                    message: format!("Listing '{}' not found", id),
                    data: None,
                }),
                Err(e) => Ok(ToolOutput {
                    tool: "get_listing".to_string(),
                    success: false,
                    message: format!("Failed to get listing: {}", e),
                    data: None,
                }),
            }
        })
    }
}

/// Browse marketplace categories tool
pub struct BrowseCategoryTool {
    model_manager: Arc<ModelManager>,
}

impl BrowseCategoryTool {
    pub fn new(model_manager: Arc<ModelManager>) -> Self {
        Self { model_manager }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_marketplace_tool_name() {
        let model_manager = Arc::new(ModelManager::new());
        let tool = SearchMarketplaceTool::new(model_manager);
        assert_eq!(tool.name(), "search_marketplace");
    }

    #[test]
    fn test_search_marketplace_tool_description() {
        let model_manager = Arc::new(ModelManager::new());
        let tool = SearchMarketplaceTool::new(model_manager);
        assert!(tool.description().contains("Search"));
        assert!(tool.description().contains("marketplace"));
    }

    #[test]
    fn test_get_listing_tool_name() {
        let model_manager = Arc::new(ModelManager::new());
        let tool = GetListingTool::new(model_manager);
        assert_eq!(tool.name(), "get_listing");
    }

    #[test]
    fn test_get_listing_tool_description() {
        let model_manager = Arc::new(ModelManager::new());
        let tool = GetListingTool::new(model_manager);
        assert!(tool.description().contains("listing"));
    }

    #[test]
    fn test_browse_category_tool_name() {
        let model_manager = Arc::new(ModelManager::new());
        let tool = BrowseCategoryTool::new(model_manager);
        assert_eq!(tool.name(), "browse_category");
    }

    #[test]
    fn test_browse_category_tool_description() {
        let model_manager = Arc::new(ModelManager::new());
        let tool = BrowseCategoryTool::new(model_manager);
        assert!(tool.description().contains("Browse"));
        assert!(tool.description().contains("category"));
    }

    #[tokio::test]
    async fn test_search_marketplace_no_query() {
        let model_manager = Arc::new(ModelManager::new());
        let tool = SearchMarketplaceTool::new(model_manager);
        let params = IntentParams::default();

        let result = tool.execute(&params).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(output.success);
        assert!(output.data.is_some());
    }

    #[tokio::test]
    async fn test_search_marketplace_with_query() {
        let model_manager = Arc::new(ModelManager::new());
        let tool = SearchMarketplaceTool::new(model_manager);
        let mut params = IntentParams::default();
        params.search_query = Some("citrate".to_string());

        let result = tool.execute(&params).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(output.success);
        assert!(output.data.is_some());
    }

    #[tokio::test]
    async fn test_search_marketplace_with_type_filter() {
        let model_manager = Arc::new(ModelManager::new());
        let tool = SearchMarketplaceTool::new(model_manager);
        let mut params = IntentParams::default();
        params.model_name = Some("language".to_string());

        let result = tool.execute(&params).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(output.success);

        let data = output.data.unwrap();
        assert!(data.get("filter_type").is_some());
    }

    #[tokio::test]
    async fn test_get_listing_missing_id() {
        let model_manager = Arc::new(ModelManager::new());
        let tool = GetListingTool::new(model_manager);
        let params = IntentParams::default();

        let result = tool.execute(&params).await;
        assert!(result.is_err());

        if let Err(DispatchError::InvalidParams(msg)) = result {
            assert!(msg.contains("required"));
        } else {
            panic!("Expected InvalidParams error");
        }
    }

    #[tokio::test]
    async fn test_get_listing_found() {
        let model_manager = Arc::new(ModelManager::new());
        let tool = GetListingTool::new(model_manager);
        let mut params = IntentParams::default();
        params.model_name = Some("gpt-citrate".to_string());

        let result = tool.execute(&params).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(output.success);
        assert!(output.data.is_some());

        let data = output.data.unwrap();
        assert!(data.get("id").is_some());
        assert!(data.get("pricing").is_some());
    }

    #[tokio::test]
    async fn test_get_listing_not_found() {
        let model_manager = Arc::new(ModelManager::new());
        let tool = GetListingTool::new(model_manager);
        let mut params = IntentParams::default();
        params.model_name = Some("nonexistent-model".to_string());

        let result = tool.execute(&params).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(!output.success);
        assert!(output.message.contains("not found"));
    }

    #[tokio::test]
    async fn test_browse_category_all() {
        let model_manager = Arc::new(ModelManager::new());
        let tool = BrowseCategoryTool::new(model_manager);
        let params = IntentParams::default();

        let result = tool.execute(&params).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(output.success);
        assert!(output.data.is_some());

        let data = output.data.unwrap();
        assert!(data.get("categories").is_some());
        assert!(data.get("total").is_some());
    }

    #[tokio::test]
    async fn test_browse_category_specific() {
        let model_manager = Arc::new(ModelManager::new());
        let tool = BrowseCategoryTool::new(model_manager);
        let mut params = IntentParams::default();
        params.search_query = Some("language".to_string());

        let result = tool.execute(&params).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(output.success);
        assert!(output.data.is_some());

        let data = output.data.unwrap();
        assert!(data.get("category").is_some());
        assert!(data.get("count").is_some());
    }

    #[tokio::test]
    async fn test_browse_category_image() {
        let model_manager = Arc::new(ModelManager::new());
        let tool = BrowseCategoryTool::new(model_manager);
        let mut params = IntentParams::default();
        params.search_query = Some("image".to_string());

        let result = tool.execute(&params).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(output.success);

        let data = output.data.unwrap();
        let category = data.get("category").and_then(|c| c.as_str()).unwrap_or("");
        assert_eq!(category, "Image");
    }
}

impl ToolHandler for BrowseCategoryTool {
    fn name(&self) -> &str {
        "browse_category"
    }

    fn description(&self) -> &str {
        "Browse marketplace by category (language, image, embedding, audio)"
    }

    fn execute(
        &self,
        params: &IntentParams,
    ) -> Pin<Box<dyn Future<Output = Result<ToolOutput, DispatchError>> + Send + '_>> {
        let model_manager = self.model_manager.clone();
        let category = params.search_query.clone();
        Box::pin(async move {
            match model_manager.get_models().await {
                Ok(models) => {
                    // Group by type
                    let mut language = Vec::new();
                    let mut image = Vec::new();
                    let mut audio = Vec::new();
                    let mut other = Vec::new();

                    for m in &models {
                        let type_str = format!("{:?}", m.model_type).to_lowercase();
                        match type_str.as_str() {
                            "language" => language.push(m.name.clone()),
                            "image" => image.push(m.name.clone()),
                            "audio" => audio.push(m.name.clone()),
                            _ => other.push(m.name.clone()),
                        }
                    }

                    // If specific category requested, return just that
                    if let Some(ref cat) = category {
                        let cat_lower = cat.to_lowercase();
                        let (cat_name, cat_models): (&str, &Vec<String>) = match cat_lower.as_str() {
                            "language" | "llm" | "text" => ("Language", &language),
                            "image" | "vision" | "diffusion" => ("Image", &image),
                            "audio" | "speech" => ("Audio", &audio),
                            _ => ("Other", &other),
                        };

                        return Ok(ToolOutput {
                            tool: "browse_category".to_string(),
                            success: true,
                            message: format!(
                                "{} category: {} models available",
                                cat_name,
                                cat_models.len()
                            ),
                            data: Some(serde_json::json!({
                                "category": cat_name,
                                "count": cat_models.len(),
                                "models": cat_models
                            })),
                        });
                    }

                    // Return all categories
                    Ok(ToolOutput {
                        tool: "browse_category".to_string(),
                        success: true,
                        message: format!(
                            "Marketplace categories: {} language, {} image, {} audio, {} other",
                            language.len(),
                            image.len(),
                            audio.len(),
                            other.len()
                        ),
                        data: Some(serde_json::json!({
                            "categories": {
                                "language": {
                                    "count": language.len(),
                                    "models": language
                                },
                                "image": {
                                    "count": image.len(),
                                    "models": image
                                },
                                "audio": {
                                    "count": audio.len(),
                                    "models": audio
                                },
                                "other": {
                                    "count": other.len(),
                                    "models": other
                                }
                            },
                            "total": models.len()
                        })),
                    })
                }
                Err(e) => Ok(ToolOutput {
                    tool: "browse_category".to_string(),
                    success: false,
                    message: format!("Failed to browse categories: {}", e),
                    data: None,
                }),
            }
        })
    }
}
