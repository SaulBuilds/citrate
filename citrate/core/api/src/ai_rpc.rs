// citrate/core/api/src/ai_rpc.rs

use crate::methods::ai::{AiApi, ChatCompletionRequest, EmbeddingsRequest};
use futures::executor::block_on;
use jsonrpc_core::{IoHandler, Params, Value};
use citrate_execution::executor::Executor;
use citrate_sequencer::mempool::Mempool;
use citrate_storage::StorageManager;
use serde_json::json;
use std::sync::Arc;

/// Register AI-related RPC methods
pub fn register_ai_methods(
    io_handler: &mut IoHandler,
    storage: Arc<StorageManager>,
    mempool: Arc<Mempool>,
    executor: Arc<Executor>,
) {
    let ai_api = Arc::new(AiApi::new(storage.clone(), mempool.clone(), executor.clone()));

    // citrate_getTextEmbedding - Generate embeddings for text
    let ai_api_embed = ai_api.clone();
    io_handler.add_sync_method("citrate_getTextEmbedding", move |params: Params| {
        // Parse params - expecting a string or array of strings
        let params_value: Vec<serde_json::Value> = params.parse()?;

        if params_value.is_empty() {
            return Err(jsonrpc_core::Error::invalid_params(
                "Missing text parameter",
            ));
        }

        // Handle both single string and array of strings
        let input_texts = if params_value[0].is_string() {
            vec![params_value[0]
                .as_str()
                .ok_or_else(|| jsonrpc_core::Error::invalid_params("Invalid text parameter"))?
                .to_string()]
        } else if params_value[0].is_array() {
            params_value[0]
                .as_array()
                .ok_or_else(|| jsonrpc_core::Error::invalid_params("Invalid array parameter"))?
                .iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        } else {
            return Err(jsonrpc_core::Error::invalid_params(
                "Parameter must be string or array of strings",
            ));
        };

        // Create embeddings request
        let request = EmbeddingsRequest {
            model: "bge-m3".to_string(), // Use genesis-embedded model
            input: input_texts,
            encoding_format: None,
        };

        // Execute embedding generation
        let api = ai_api_embed.clone();
        match block_on(api.embeddings(request, None)) {
            Ok(response) => {
                // Return just the embedding vectors
                if response.data.len() == 1 {
                    // Single text input - return single embedding
                    Ok(json!(response.data[0].embedding))
                } else {
                    // Multiple texts - return array of embeddings
                    let embeddings: Vec<Vec<f32>> = response
                        .data
                        .into_iter()
                        .map(|d| d.embedding)
                        .collect();
                    Ok(json!(embeddings))
                }
            }
            Err(e) => Err(jsonrpc_core::Error {
                code: jsonrpc_core::ErrorCode::ServerError(-32000),
                message: format!("Embedding generation failed: {}", e),
                data: Some(json!({"error": e.to_string()})),
            }),
        }
    });

    // citrate_semanticSearch - Semantic search using embeddings
    let ai_api_search = ai_api.clone();
    io_handler.add_sync_method("citrate_semanticSearch", move |params: Params| {
        // Parse params: query (string), documents (array of strings), top_k (optional number)
        let params_value: Vec<serde_json::Value> = params.parse()?;

        if params_value.len() < 2 {
            return Err(jsonrpc_core::Error::invalid_params(
                "Expected parameters: query, documents, [top_k]",
            ));
        }

        let query = params_value[0]
            .as_str()
            .ok_or_else(|| jsonrpc_core::Error::invalid_params("Query must be a string"))?
            .to_string();

        let documents: Vec<String> = params_value[1]
            .as_array()
            .ok_or_else(|| jsonrpc_core::Error::invalid_params("Documents must be an array"))?
            .iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect();

        let top_k = params_value
            .get(2)
            .and_then(|v| v.as_u64())
            .unwrap_or(documents.len() as u64)
            .min(documents.len() as u64) as usize;

        // Generate embeddings for query and documents
        let mut all_texts = vec![query.clone()];
        all_texts.extend(documents.clone());

        let request = EmbeddingsRequest {
            model: "bge-m3".to_string(),
            input: all_texts,
            encoding_format: None,
        };

        let api = ai_api_search.clone();
        match block_on(api.embeddings(request, None)) {
            Ok(response) => {
                if response.data.is_empty() {
                    return Err(jsonrpc_core::Error::internal_error());
                }

                // Extract query embedding and document embeddings
                let query_embedding = &response.data[0].embedding;
                let doc_embeddings: Vec<&Vec<f32>> = response
                    .data
                    .iter()
                    .skip(1)
                    .map(|d| &d.embedding)
                    .collect();

                // Calculate cosine similarity scores
                let mut scored_docs: Vec<(usize, f32, String)> = documents
                    .iter()
                    .enumerate()
                    .map(|(idx, doc)| {
                        let doc_embedding = &doc_embeddings[idx];
                        let similarity = cosine_similarity(query_embedding, doc_embedding);
                        (idx, similarity, doc.clone())
                    })
                    .collect();

                // Sort by similarity (descending)
                scored_docs.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

                // Take top_k results
                let results: Vec<serde_json::Value> = scored_docs
                    .into_iter()
                    .take(top_k)
                    .map(|(idx, score, text)| {
                        json!({
                            "index": idx,
                            "score": score,
                            "text": text
                        })
                    })
                    .collect();

                Ok(json!(results))
            }
            Err(e) => Err(jsonrpc_core::Error {
                code: jsonrpc_core::ErrorCode::ServerError(-32000),
                message: format!("Semantic search failed: {}", e),
                data: Some(json!({"error": e.to_string()})),
            }),
        }
    });

    // citrate_chatCompletion - LLM chat completion
    let ai_api_chat = ai_api.clone();
    io_handler.add_sync_method("citrate_chatCompletion", move |params: Params| {
        // Parse params - expecting ChatCompletionRequest
        let params_value: Vec<serde_json::Value> = params.parse()?;

        if params_value.is_empty() {
            return Err(jsonrpc_core::Error::invalid_params(
                "Missing chat completion parameters",
            ));
        }

        // Try to parse as full ChatCompletionRequest, or construct from simple params
        let request: ChatCompletionRequest = if params_value[0].is_object() {
            serde_json::from_value(params_value[0].clone()).map_err(|e| {
                jsonrpc_core::Error::invalid_params(format!("Invalid request format: {}", e))
            })?
        } else {
            // Simple format: [prompt, max_tokens, temperature]
            let prompt = params_value[0]
                .as_str()
                .ok_or_else(|| jsonrpc_core::Error::invalid_params("Prompt must be a string"))?
                .to_string();

            let max_tokens = params_value
                .get(1)
                .and_then(|v| v.as_u64())
                .map(|v| v as u32);

            let temperature = params_value
                .get(2)
                .and_then(|v| v.as_f64())
                .map(|v| v as f32);

            ChatCompletionRequest {
                model: "mistral-7b-instruct-v0.3".to_string(),
                messages: vec![crate::methods::ai::ChatMessage {
                    role: "user".to_string(),
                    content: prompt,
                }],
                max_tokens,
                temperature,
                top_p: None,
                n: None,
                stop: None,
                stream: Some(false),
            }
        };

        // Execute chat completion
        let api = ai_api_chat.clone();
        match block_on(api.chat_completions(request, None)) {
            Ok(response) => {
                // Return full response
                Ok(serde_json::to_value(response).map_err(|e| {
                    jsonrpc_core::Error {
                        code: jsonrpc_core::ErrorCode::ServerError(-32000),
                        message: "Failed to serialize response".to_string(),
                        data: Some(json!({"error": e.to_string()})),
                    }
                })?)
            }
            Err(e) => Err(jsonrpc_core::Error {
                code: jsonrpc_core::ErrorCode::ServerError(-32000),
                message: format!("Chat completion failed: {}", e),
                data: Some(json!({"error": e.to_string()})),
            }),
        }
    });
}

/// Calculate cosine similarity between two vectors
fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
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
