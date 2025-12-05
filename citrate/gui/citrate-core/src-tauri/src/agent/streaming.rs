//! Streaming response infrastructure for real-time token delivery
//!
//! Provides token-by-token streaming from LLM backends to the frontend
//! via Tauri events.

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::mpsc;

/// A single streamed token
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamToken {
    /// The token text
    pub text: String,
    /// Token index in the response
    pub index: usize,
    /// Whether this is the final token
    pub is_final: bool,
    /// Cumulative text so far
    pub cumulative: Option<String>,
}

/// Stream completion status
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum StreamStatus {
    /// Stream started
    Started { session_id: String, message_id: String },
    /// Token received
    Token(StreamToken),
    /// Stream completed successfully
    Completed {
        session_id: String,
        message_id: String,
        total_tokens: usize,
        finish_reason: String,
    },
    /// Stream errored
    Error {
        session_id: String,
        message_id: String,
        error: String,
    },
    /// Stream was cancelled
    Cancelled { session_id: String, message_id: String },
}

/// Configuration for streaming responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamingConfig {
    /// Whether streaming is enabled
    pub enabled: bool,
    /// Buffer size for token channel
    pub buffer_size: usize,
    /// Debounce interval in ms (for batching rapid tokens)
    pub debounce_ms: u64,
    /// Whether to send cumulative text with each token
    pub include_cumulative: bool,
}

impl Default for StreamingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            buffer_size: 100,
            debounce_ms: 0,
            include_cumulative: false,
        }
    }
}

/// A streaming response handler
pub struct StreamingResponse {
    /// Session ID
    session_id: String,
    /// Message ID for this response
    message_id: String,
    /// Sender for stream tokens
    sender: mpsc::Sender<StreamToken>,
    /// Accumulated text
    accumulated: String,
    /// Token count
    token_count: usize,
    /// Whether to include cumulative text
    include_cumulative: bool,
    /// Whether the stream has been finalized
    finalized: bool,
}

impl StreamingResponse {
    /// Create a new streaming response
    pub fn new(
        session_id: String,
        message_id: String,
        sender: mpsc::Sender<StreamToken>,
        include_cumulative: bool,
    ) -> Self {
        Self {
            session_id,
            message_id,
            sender,
            accumulated: String::new(),
            token_count: 0,
            include_cumulative,
            finalized: false,
        }
    }

    /// Get session ID
    pub fn session_id(&self) -> &str {
        &self.session_id
    }

    /// Get message ID
    pub fn message_id(&self) -> &str {
        &self.message_id
    }

    /// Send a token
    pub async fn send_token(&mut self, text: &str) -> Result<(), StreamError> {
        if self.finalized {
            return Err(StreamError::AlreadyFinalized);
        }

        self.accumulated.push_str(text);
        self.token_count += 1;

        let token = StreamToken {
            text: text.to_string(),
            index: self.token_count - 1,
            is_final: false,
            cumulative: if self.include_cumulative {
                Some(self.accumulated.clone())
            } else {
                None
            },
        };

        self.sender
            .send(token)
            .await
            .map_err(|_| StreamError::ChannelClosed)?;

        Ok(())
    }

    /// Finalize the stream
    pub async fn finalize(&mut self) -> Result<String, StreamError> {
        if self.finalized {
            return Err(StreamError::AlreadyFinalized);
        }

        self.finalized = true;

        // Send final token marker
        let final_token = StreamToken {
            text: String::new(),
            index: self.token_count,
            is_final: true,
            cumulative: Some(self.accumulated.clone()),
        };

        let _ = self.sender.send(final_token).await;

        Ok(self.accumulated.clone())
    }

    /// Get accumulated text
    pub fn accumulated(&self) -> &str {
        &self.accumulated
    }

    /// Get token count
    pub fn token_count(&self) -> usize {
        self.token_count
    }

    /// Check if finalized
    pub fn is_finalized(&self) -> bool {
        self.finalized
    }
}

/// Errors during streaming
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StreamError {
    /// Channel was closed
    ChannelClosed,
    /// Stream already finalized
    AlreadyFinalized,
    /// Timeout waiting for tokens
    Timeout,
    /// Backend error
    BackendError(String),
}

impl std::fmt::Display for StreamError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ChannelClosed => write!(f, "Stream channel closed"),
            Self::AlreadyFinalized => write!(f, "Stream already finalized"),
            Self::Timeout => write!(f, "Stream timeout"),
            Self::BackendError(e) => write!(f, "Backend error: {}", e),
        }
    }
}

impl std::error::Error for StreamError {}

/// Manager for active streams
pub struct StreamManager {
    /// Active streams by message ID
    active_streams: Arc<tokio::sync::RwLock<std::collections::HashMap<String, StreamHandle>>>,
}

/// Handle to an active stream
pub struct StreamHandle {
    /// Sender to cancel the stream
    cancel_tx: mpsc::Sender<()>,
    /// Session ID
    pub session_id: String,
    /// Message ID
    pub message_id: String,
}

impl StreamManager {
    /// Create a new stream manager
    pub fn new() -> Self {
        Self {
            active_streams: Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
        }
    }

    /// Create a new stream
    pub async fn create_stream(
        &self,
        session_id: String,
        message_id: String,
        config: &StreamingConfig,
    ) -> (StreamingResponse, mpsc::Receiver<StreamToken>) {
        let (token_tx, token_rx) = mpsc::channel(config.buffer_size);
        let (cancel_tx, _cancel_rx) = mpsc::channel(1);

        let response = StreamingResponse::new(
            session_id.clone(),
            message_id.clone(),
            token_tx,
            config.include_cumulative,
        );

        let handle = StreamHandle {
            cancel_tx,
            session_id,
            message_id: message_id.clone(),
        };

        self.active_streams
            .write()
            .await
            .insert(message_id, handle);

        (response, token_rx)
    }

    /// Cancel a stream
    pub async fn cancel_stream(&self, message_id: &str) -> bool {
        if let Some(handle) = self.active_streams.write().await.remove(message_id) {
            let _ = handle.cancel_tx.send(()).await;
            true
        } else {
            false
        }
    }

    /// Remove a completed stream
    pub async fn complete_stream(&self, message_id: &str) {
        self.active_streams.write().await.remove(message_id);
    }

    /// Get active stream count
    pub async fn active_count(&self) -> usize {
        self.active_streams.read().await.len()
    }
}

impl Default for StreamManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_streaming_response() {
        let (tx, mut rx) = mpsc::channel(10);
        let mut response =
            StreamingResponse::new("session1".into(), "msg1".into(), tx, true);

        response.send_token("Hello").await.unwrap();
        response.send_token(" world").await.unwrap();

        let token1 = rx.recv().await.unwrap();
        assert_eq!(token1.text, "Hello");
        assert_eq!(token1.index, 0);
        assert!(!token1.is_final);

        let token2 = rx.recv().await.unwrap();
        assert_eq!(token2.text, " world");
        assert_eq!(token2.cumulative, Some("Hello world".to_string()));

        let final_text = response.finalize().await.unwrap();
        assert_eq!(final_text, "Hello world");

        let final_token = rx.recv().await.unwrap();
        assert!(final_token.is_final);
    }

    #[tokio::test]
    async fn test_stream_manager() {
        let manager = StreamManager::new();
        let config = StreamingConfig::default();

        let (mut response, _rx) = manager
            .create_stream("session1".into(), "msg1".into(), &config)
            .await;

        assert_eq!(manager.active_count().await, 1);

        response.finalize().await.unwrap();
        manager.complete_stream("msg1").await;

        assert_eq!(manager.active_count().await, 0);
    }
}
