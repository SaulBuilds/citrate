//! Prometheus-style Metrics Module
//!
//! Provides comprehensive node metrics for monitoring and observability.
//! Exposes metrics in Prometheus format at /metrics endpoint.
//!
//! # Metrics Categories
//! - Node lifecycle (start/stop, uptime)
//! - Peer connections (count, latency)
//! - Mempool (size, transactions)
//! - Block production (height, sync status)
//! - DAG (tips count, blue score)
//! - RPC (request count, latency)
//! - AI inference (requests, latency)
//! - IPFS (operations, latency)
//!
//! # Usage
//! ```rust
//! use citrate_node::metrics::{init_metrics, NODE_UPTIME, BLOCK_HEIGHT};
//!
//! // Initialize metrics server
//! init_metrics("127.0.0.1:9090")?;
//!
//! // Record metrics
//! NODE_UPTIME.set(uptime_seconds);
//! BLOCK_HEIGHT.set(height);
//! ```

use metrics::{counter, gauge, histogram, describe_counter, describe_gauge, describe_histogram, Unit};
use metrics_exporter_prometheus::{PrometheusBuilder, PrometheusHandle};
use once_cell::sync::OnceCell;
use parking_lot::RwLock;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::oneshot;

/// Global Prometheus handle
static PROMETHEUS_HANDLE: OnceCell<PrometheusHandle> = OnceCell::new();

/// Metrics server shutdown signal
static METRICS_SHUTDOWN: OnceCell<RwLock<Option<oneshot::Sender<()>>>> = OnceCell::new();

// ============================================================================
// Metric Names (constants for consistency)
// ============================================================================

// Node Lifecycle
pub const METRIC_NODE_START_TIME: &str = "citrate_node_start_time_seconds";
pub const METRIC_NODE_UPTIME: &str = "citrate_node_uptime_seconds";
pub const METRIC_NODE_INFO: &str = "citrate_node_info";
pub const METRIC_NODE_RESTARTS: &str = "citrate_node_restarts_total";

// Peer Connections
pub const METRIC_PEER_COUNT: &str = "citrate_peer_count";
pub const METRIC_PEER_CONNECTIONS_TOTAL: &str = "citrate_peer_connections_total";
pub const METRIC_PEER_DISCONNECTIONS_TOTAL: &str = "citrate_peer_disconnections_total";
pub const METRIC_PEER_LATENCY: &str = "citrate_peer_latency_seconds";

// Mempool
pub const METRIC_MEMPOOL_SIZE: &str = "citrate_mempool_size";
pub const METRIC_MEMPOOL_BYTES: &str = "citrate_mempool_bytes";
pub const METRIC_TX_RECEIVED_TOTAL: &str = "citrate_transactions_received_total";
pub const METRIC_TX_REJECTED_TOTAL: &str = "citrate_transactions_rejected_total";
pub const METRIC_TX_INCLUDED_TOTAL: &str = "citrate_transactions_included_total";

// Block Production
pub const METRIC_BLOCK_HEIGHT: &str = "citrate_block_height";
pub const METRIC_BLOCKS_PRODUCED_TOTAL: &str = "citrate_blocks_produced_total";
pub const METRIC_BLOCK_BUILD_TIME: &str = "citrate_block_build_time_seconds";
pub const METRIC_BLOCK_SIZE: &str = "citrate_block_size_bytes";
pub const METRIC_TX_PER_BLOCK: &str = "citrate_transactions_per_block";
pub const METRIC_ORPHAN_BLOCKS_TOTAL: &str = "citrate_orphan_blocks_total";

// DAG
pub const METRIC_DAG_TIPS_COUNT: &str = "citrate_dag_tips_count";
pub const METRIC_DAG_BLUE_SCORE: &str = "citrate_dag_blue_score";
pub const METRIC_DAG_WIDTH: &str = "citrate_dag_width";
pub const METRIC_DAG_DEPTH: &str = "citrate_dag_depth";

// Sync
pub const METRIC_SYNC_STATUS: &str = "citrate_sync_status";
pub const METRIC_SYNC_PROGRESS: &str = "citrate_sync_progress";
pub const METRIC_SYNC_PEERS: &str = "citrate_sync_peers";

// RPC
pub const METRIC_RPC_REQUESTS_TOTAL: &str = "citrate_rpc_requests_total";
pub const METRIC_RPC_ERRORS_TOTAL: &str = "citrate_rpc_errors_total";
pub const METRIC_RPC_LATENCY: &str = "citrate_rpc_latency_seconds";
pub const METRIC_RPC_ACTIVE_CONNECTIONS: &str = "citrate_rpc_active_connections";

// AI Inference
pub const METRIC_AI_REQUESTS_TOTAL: &str = "citrate_ai_requests_total";
pub const METRIC_AI_ERRORS_TOTAL: &str = "citrate_ai_errors_total";
pub const METRIC_AI_LATENCY: &str = "citrate_ai_latency_seconds";
pub const METRIC_AI_TOKENS_TOTAL: &str = "citrate_ai_tokens_total";
pub const METRIC_AI_MODELS_LOADED: &str = "citrate_ai_models_loaded";

// IPFS
pub const METRIC_IPFS_UPLOADS_TOTAL: &str = "citrate_ipfs_uploads_total";
pub const METRIC_IPFS_DOWNLOADS_TOTAL: &str = "citrate_ipfs_downloads_total";
pub const METRIC_IPFS_PINS_TOTAL: &str = "citrate_ipfs_pins_total";
pub const METRIC_IPFS_LATENCY: &str = "citrate_ipfs_latency_seconds";
pub const METRIC_IPFS_BYTES_UPLOADED: &str = "citrate_ipfs_bytes_uploaded_total";
pub const METRIC_IPFS_BYTES_DOWNLOADED: &str = "citrate_ipfs_bytes_downloaded_total";

// ============================================================================
// Initialization
// ============================================================================

/// Initialize the metrics system and start the HTTP server.
///
/// # Arguments
/// * `addr` - Socket address for the metrics server (e.g., "127.0.0.1:9090")
///
/// # Returns
/// Result indicating success or failure
pub fn init_metrics(addr: &str) -> anyhow::Result<()> {
    let socket_addr: SocketAddr = addr.parse()?;

    // Build Prometheus exporter
    let builder = PrometheusBuilder::new();
    let handle = builder
        .with_http_listener(socket_addr)
        .install_recorder()?;

    PROMETHEUS_HANDLE
        .set(handle)
        .map_err(|_| anyhow::anyhow!("Metrics already initialized"))?;

    // Register metric descriptions
    register_metric_descriptions();

    // Record node start time
    let start_time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs_f64();
    gauge!(METRIC_NODE_START_TIME).set(start_time);

    tracing::info!("Metrics server started on http://{}/metrics", addr);

    Ok(())
}

/// Register descriptions for all metrics (for Prometheus HELP text)
fn register_metric_descriptions() {
    // Node Lifecycle
    describe_gauge!(
        METRIC_NODE_START_TIME,
        Unit::Seconds,
        "Unix timestamp when the node started"
    );
    describe_gauge!(
        METRIC_NODE_UPTIME,
        Unit::Seconds,
        "Node uptime in seconds"
    );
    describe_gauge!(
        METRIC_NODE_INFO,
        "Node information including version and network"
    );
    describe_counter!(
        METRIC_NODE_RESTARTS,
        "Total number of node restarts"
    );

    // Peer Connections
    describe_gauge!(
        METRIC_PEER_COUNT,
        "Current number of connected peers"
    );
    describe_counter!(
        METRIC_PEER_CONNECTIONS_TOTAL,
        "Total peer connection events"
    );
    describe_counter!(
        METRIC_PEER_DISCONNECTIONS_TOTAL,
        "Total peer disconnection events"
    );
    describe_histogram!(
        METRIC_PEER_LATENCY,
        Unit::Seconds,
        "Peer message round-trip latency"
    );

    // Mempool
    describe_gauge!(
        METRIC_MEMPOOL_SIZE,
        "Current number of transactions in mempool"
    );
    describe_gauge!(
        METRIC_MEMPOOL_BYTES,
        Unit::Bytes,
        "Current mempool size in bytes"
    );
    describe_counter!(
        METRIC_TX_RECEIVED_TOTAL,
        "Total transactions received"
    );
    describe_counter!(
        METRIC_TX_REJECTED_TOTAL,
        "Total transactions rejected"
    );
    describe_counter!(
        METRIC_TX_INCLUDED_TOTAL,
        "Total transactions included in blocks"
    );

    // Block Production
    describe_gauge!(
        METRIC_BLOCK_HEIGHT,
        "Current block height"
    );
    describe_counter!(
        METRIC_BLOCKS_PRODUCED_TOTAL,
        "Total blocks produced by this node"
    );
    describe_histogram!(
        METRIC_BLOCK_BUILD_TIME,
        Unit::Seconds,
        "Time to build a block"
    );
    describe_histogram!(
        METRIC_BLOCK_SIZE,
        Unit::Bytes,
        "Block size distribution"
    );
    describe_histogram!(
        METRIC_TX_PER_BLOCK,
        "Transactions per block distribution"
    );
    describe_counter!(
        METRIC_ORPHAN_BLOCKS_TOTAL,
        "Total orphaned blocks"
    );

    // DAG
    describe_gauge!(
        METRIC_DAG_TIPS_COUNT,
        "Current number of DAG tips"
    );
    describe_gauge!(
        METRIC_DAG_BLUE_SCORE,
        "Current max blue score"
    );
    describe_gauge!(
        METRIC_DAG_WIDTH,
        "Current DAG width (parallel blocks)"
    );
    describe_gauge!(
        METRIC_DAG_DEPTH,
        "Current DAG depth"
    );

    // Sync
    describe_gauge!(
        METRIC_SYNC_STATUS,
        "Sync status (0=synced, 1=syncing)"
    );
    describe_gauge!(
        METRIC_SYNC_PROGRESS,
        "Sync progress percentage (0-100)"
    );
    describe_gauge!(
        METRIC_SYNC_PEERS,
        "Number of peers contributing to sync"
    );

    // RPC
    describe_counter!(
        METRIC_RPC_REQUESTS_TOTAL,
        "Total RPC requests by method"
    );
    describe_counter!(
        METRIC_RPC_ERRORS_TOTAL,
        "Total RPC errors by method and error type"
    );
    describe_histogram!(
        METRIC_RPC_LATENCY,
        Unit::Seconds,
        "RPC request latency by method"
    );
    describe_gauge!(
        METRIC_RPC_ACTIVE_CONNECTIONS,
        "Current active RPC connections"
    );

    // AI Inference
    describe_counter!(
        METRIC_AI_REQUESTS_TOTAL,
        "Total AI inference requests by model"
    );
    describe_counter!(
        METRIC_AI_ERRORS_TOTAL,
        "Total AI inference errors"
    );
    describe_histogram!(
        METRIC_AI_LATENCY,
        Unit::Seconds,
        "AI inference latency by model"
    );
    describe_counter!(
        METRIC_AI_TOKENS_TOTAL,
        "Total tokens processed"
    );
    describe_gauge!(
        METRIC_AI_MODELS_LOADED,
        "Number of AI models currently loaded"
    );

    // IPFS
    describe_counter!(
        METRIC_IPFS_UPLOADS_TOTAL,
        "Total IPFS uploads"
    );
    describe_counter!(
        METRIC_IPFS_DOWNLOADS_TOTAL,
        "Total IPFS downloads"
    );
    describe_counter!(
        METRIC_IPFS_PINS_TOTAL,
        "Total IPFS pin operations"
    );
    describe_histogram!(
        METRIC_IPFS_LATENCY,
        Unit::Seconds,
        "IPFS operation latency"
    );
    describe_counter!(
        METRIC_IPFS_BYTES_UPLOADED,
        Unit::Bytes,
        "Total bytes uploaded to IPFS"
    );
    describe_counter!(
        METRIC_IPFS_BYTES_DOWNLOADED,
        Unit::Bytes,
        "Total bytes downloaded from IPFS"
    );
}

// ============================================================================
// Helper Functions for Recording Metrics
// ============================================================================

/// Record node uptime
pub fn record_uptime(start_time: Instant) {
    gauge!(METRIC_NODE_UPTIME).set(start_time.elapsed().as_secs_f64());
}

/// Record peer count
pub fn record_peer_count(count: usize) {
    gauge!(METRIC_PEER_COUNT).set(count as f64);
}

/// Record peer connection event
pub fn record_peer_connected(peer_id: &str) {
    counter!(METRIC_PEER_CONNECTIONS_TOTAL, "peer_id" => peer_id.to_string()).increment(1);
}

/// Record peer disconnection event
pub fn record_peer_disconnected(peer_id: &str) {
    counter!(METRIC_PEER_DISCONNECTIONS_TOTAL, "peer_id" => peer_id.to_string()).increment(1);
}

/// Record peer latency
pub fn record_peer_latency(latency: Duration) {
    histogram!(METRIC_PEER_LATENCY).record(latency.as_secs_f64());
}

/// Record mempool metrics
pub fn record_mempool_size(tx_count: usize, bytes: usize) {
    gauge!(METRIC_MEMPOOL_SIZE).set(tx_count as f64);
    gauge!(METRIC_MEMPOOL_BYTES).set(bytes as f64);
}

/// Record transaction received
pub fn record_tx_received(tx_type: &str) {
    counter!(METRIC_TX_RECEIVED_TOTAL, "type" => tx_type.to_string()).increment(1);
}

/// Record transaction rejected
pub fn record_tx_rejected(reason: &str) {
    counter!(METRIC_TX_REJECTED_TOTAL, "reason" => reason.to_string()).increment(1);
}

/// Record transaction included in block
pub fn record_tx_included() {
    counter!(METRIC_TX_INCLUDED_TOTAL).increment(1);
}

/// Record block height
pub fn record_block_height(height: u64) {
    gauge!(METRIC_BLOCK_HEIGHT).set(height as f64);
}

/// Record block production
pub fn record_block_produced(build_time: Duration, size_bytes: usize, tx_count: usize) {
    counter!(METRIC_BLOCKS_PRODUCED_TOTAL).increment(1);
    histogram!(METRIC_BLOCK_BUILD_TIME).record(build_time.as_secs_f64());
    histogram!(METRIC_BLOCK_SIZE).record(size_bytes as f64);
    histogram!(METRIC_TX_PER_BLOCK).record(tx_count as f64);
}

/// Record orphan block
pub fn record_orphan_block() {
    counter!(METRIC_ORPHAN_BLOCKS_TOTAL).increment(1);
}

/// Record DAG metrics
pub fn record_dag_metrics(tips: usize, blue_score: u64, width: usize, depth: u64) {
    gauge!(METRIC_DAG_TIPS_COUNT).set(tips as f64);
    gauge!(METRIC_DAG_BLUE_SCORE).set(blue_score as f64);
    gauge!(METRIC_DAG_WIDTH).set(width as f64);
    gauge!(METRIC_DAG_DEPTH).set(depth as f64);
}

/// Record sync status
pub fn record_sync_status(is_syncing: bool, progress: f64, sync_peers: usize) {
    gauge!(METRIC_SYNC_STATUS).set(if is_syncing { 1.0 } else { 0.0 });
    gauge!(METRIC_SYNC_PROGRESS).set(progress);
    gauge!(METRIC_SYNC_PEERS).set(sync_peers as f64);
}

/// Record RPC request
pub fn record_rpc_request(method: &str, latency: Duration, success: bool) {
    counter!(METRIC_RPC_REQUESTS_TOTAL, "method" => method.to_string()).increment(1);
    histogram!(METRIC_RPC_LATENCY, "method" => method.to_string()).record(latency.as_secs_f64());
    if !success {
        counter!(METRIC_RPC_ERRORS_TOTAL, "method" => method.to_string()).increment(1);
    }
}

/// Record RPC error with specific error type
pub fn record_rpc_error(method: &str, error_type: &str) {
    counter!(METRIC_RPC_ERRORS_TOTAL, "method" => method.to_string(), "error" => error_type.to_string()).increment(1);
}

/// Record active RPC connections
pub fn record_rpc_connections(count: usize) {
    gauge!(METRIC_RPC_ACTIVE_CONNECTIONS).set(count as f64);
}

/// Record AI inference request
pub fn record_ai_request(model: &str, latency: Duration, tokens: usize, success: bool) {
    counter!(METRIC_AI_REQUESTS_TOTAL, "model" => model.to_string()).increment(1);
    histogram!(METRIC_AI_LATENCY, "model" => model.to_string()).record(latency.as_secs_f64());
    counter!(METRIC_AI_TOKENS_TOTAL, "model" => model.to_string()).increment(tokens as u64);
    if !success {
        counter!(METRIC_AI_ERRORS_TOTAL, "model" => model.to_string()).increment(1);
    }
}

/// Record loaded AI models count
pub fn record_ai_models_loaded(count: usize) {
    gauge!(METRIC_AI_MODELS_LOADED).set(count as f64);
}

/// Record IPFS upload
pub fn record_ipfs_upload(latency: Duration, bytes: usize) {
    counter!(METRIC_IPFS_UPLOADS_TOTAL).increment(1);
    histogram!(METRIC_IPFS_LATENCY, "operation" => "upload").record(latency.as_secs_f64());
    counter!(METRIC_IPFS_BYTES_UPLOADED).increment(bytes as u64);
}

/// Record IPFS download
pub fn record_ipfs_download(latency: Duration, bytes: usize) {
    counter!(METRIC_IPFS_DOWNLOADS_TOTAL).increment(1);
    histogram!(METRIC_IPFS_LATENCY, "operation" => "download").record(latency.as_secs_f64());
    counter!(METRIC_IPFS_BYTES_DOWNLOADED).increment(bytes as u64);
}

/// Record IPFS pin operation
pub fn record_ipfs_pin(latency: Duration) {
    counter!(METRIC_IPFS_PINS_TOTAL).increment(1);
    histogram!(METRIC_IPFS_LATENCY, "operation" => "pin").record(latency.as_secs_f64());
}

// ============================================================================
// Timer Helper for Automatic Latency Recording
// ============================================================================

/// A timer that automatically records duration when dropped.
/// Useful for measuring operation latency.
pub struct MetricsTimer {
    metric_name: &'static str,
    labels: Vec<(&'static str, String)>,
    start: Instant,
}

impl MetricsTimer {
    /// Create a new timer for a histogram metric.
    pub fn new(metric_name: &'static str) -> Self {
        Self {
            metric_name,
            labels: Vec::new(),
            start: Instant::now(),
        }
    }

    /// Add a label to the timer.
    pub fn with_label(mut self, key: &'static str, value: impl Into<String>) -> Self {
        self.labels.push((key, value.into()));
        self
    }

    /// Get elapsed time without recording.
    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }

    /// Manually record and consume the timer.
    pub fn record(self) {
        // Drop will handle recording
        drop(self);
    }
}

impl Drop for MetricsTimer {
    fn drop(&mut self) {
        let duration = self.start.elapsed().as_secs_f64();
        match self.labels.len() {
            0 => histogram!(self.metric_name).record(duration),
            1 => {
                let (k, v) = &self.labels[0];
                histogram!(self.metric_name, *k => v.clone()).record(duration);
            }
            _ => {
                // For multiple labels, we need to use the macro differently
                // This is a simplified version - full implementation would use dynamic labels
                histogram!(self.metric_name).record(duration);
            }
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_timer() {
        let timer = MetricsTimer::new(METRIC_RPC_LATENCY)
            .with_label("method", "test");

        std::thread::sleep(Duration::from_millis(10));
        assert!(timer.elapsed() >= Duration::from_millis(10));
    }

    #[test]
    fn test_record_functions() {
        // These should not panic even without initialization
        record_peer_count(5);
        record_block_height(100);
        record_mempool_size(10, 1000);
    }
}
