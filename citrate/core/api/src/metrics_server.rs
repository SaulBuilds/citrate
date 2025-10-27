// citrate/core/api/src/metrics_server.rs

use axum::{body::Body, http::StatusCode, response::Response, routing::get, Router};
use once_cell::sync::Lazy;
use prometheus::{
    register_counter_vec, register_gauge_vec, register_histogram_vec, Encoder, TextEncoder,
};
use std::net::SocketAddr;
use tracing::info;

// RPC Metrics
pub static RPC_REQUEST_DURATION: Lazy<prometheus::HistogramVec> = Lazy::new(|| {
    register_histogram_vec!(
        "citrate_rpc_request_duration_seconds",
        "RPC request duration in seconds",
        &["method"]
    )
    .expect("Failed to register RPC request duration metric")
});

pub static RPC_REQUEST_COUNT: Lazy<prometheus::CounterVec> = Lazy::new(|| {
    register_counter_vec!(
        "citrate_rpc_requests_total",
        "Total number of RPC requests",
        &["method", "status"]
    )
    .expect("Failed to register RPC request count metric")
});

// Mempool Metrics
pub static MEMPOOL_SIZE: Lazy<prometheus::GaugeVec> = Lazy::new(|| {
    register_gauge_vec!("citrate_mempool_size", "Current mempool size", &["class"])
        .expect("Failed to register mempool size metric")
});

pub static MEMPOOL_BYTES: Lazy<prometheus::GaugeVec> = Lazy::new(|| {
    register_gauge_vec!(
        "citrate_mempool_bytes",
        "Current mempool size in bytes",
        &["class"]
    )
    .expect("Failed to register mempool bytes metric")
});

// Storage Metrics
pub static STORAGE_READ_DURATION: Lazy<prometheus::HistogramVec> = Lazy::new(|| {
    register_histogram_vec!(
        "citrate_storage_read_duration_seconds",
        "Storage read duration in seconds",
        &["cf"]
    )
    .expect("Failed to register storage read duration metric")
});

pub static STORAGE_WRITE_DURATION: Lazy<prometheus::HistogramVec> = Lazy::new(|| {
    register_histogram_vec!(
        "citrate_storage_write_duration_seconds",
        "Storage write duration in seconds",
        &["cf"]
    )
    .expect("Failed to register storage write duration metric")
});

// Cache Metrics
pub static CACHE_HIT_RATE: Lazy<prometheus::GaugeVec> = Lazy::new(|| {
    register_gauge_vec!("citrate_cache_hit_rate", "Cache hit rate", &["cache_type"])
        .expect("Failed to register cache hit rate metric")
});

pub static CACHE_SIZE: Lazy<prometheus::GaugeVec> = Lazy::new(|| {
    register_gauge_vec!("citrate_cache_size", "Current cache size", &["cache_type"])
        .expect("Failed to register cache size metric")
});

// DAG Metrics
pub static DAG_HEIGHT: Lazy<prometheus::GaugeVec> = Lazy::new(|| {
    register_gauge_vec!("citrate_dag_height", "Current DAG height", &[])
        .expect("Failed to register DAG height metric")
});

pub static DAG_TIPS_COUNT: Lazy<prometheus::GaugeVec> = Lazy::new(|| {
    register_gauge_vec!("citrate_dag_tips_count", "Number of current tips", &[])
        .expect("Failed to register DAG tips count metric")
});

pub static DAG_BLUE_SCORE: Lazy<prometheus::GaugeVec> = Lazy::new(|| {
    register_gauge_vec!("citrate_dag_blue_score", "Current blue score", &[])
        .expect("Failed to register DAG blue score metric")
});

// Execution Metrics
pub static EXECUTION_TIME: Lazy<prometheus::HistogramVec> = Lazy::new(|| {
    register_histogram_vec!(
        "citrate_execution_time_seconds",
        "Transaction execution time",
        &["tx_type"]
    )
    .expect("Failed to register execution time metric")
});

pub static PARALLEL_EXECUTION_GROUPS: Lazy<prometheus::HistogramVec> = Lazy::new(|| {
    register_histogram_vec!(
        "citrate_parallel_execution_groups",
        "Number of parallel execution groups",
        &[]
    )
    .expect("Failed to register parallel execution groups metric")
});

// Network Metrics
pub static PEER_COUNT: Lazy<prometheus::GaugeVec> = Lazy::new(|| {
    register_gauge_vec!(
        "citrate_peer_count",
        "Number of connected peers",
        &["state"]
    )
    .expect("Failed to register peer count metric")
});

pub static NETWORK_BYTES_RECEIVED: Lazy<prometheus::CounterVec> = Lazy::new(|| {
    register_counter_vec!(
        "citrate_network_bytes_received_total",
        "Total bytes received",
        &["protocol"]
    )
    .expect("Failed to register network bytes received metric")
});

pub static NETWORK_BYTES_SENT: Lazy<prometheus::CounterVec> = Lazy::new(|| {
    register_counter_vec!(
        "citrate_network_bytes_sent_total",
        "Total bytes sent",
        &["protocol"]
    )
    .expect("Failed to register network bytes sent metric")
});

/// Metrics server configuration
pub struct MetricsServer {
    addr: SocketAddr,
}

impl MetricsServer {
    pub fn new(addr: SocketAddr) -> Self {
        Self { addr }
    }

    /// Start the metrics server
    pub async fn start(self) -> anyhow::Result<()> {
        let app = Router::new()
            .route("/metrics", get(metrics_handler))
            .route("/health", get(health_handler));

        info!("Starting metrics server on {}", self.addr);

        let listener = tokio::net::TcpListener::bind(self.addr).await?;
        axum::serve(listener, app).await?;

        Ok(())
    }
}

/// Handler for /metrics endpoint
async fn metrics_handler() -> Response<Body> {
    let encoder = TextEncoder::new();
    let metric_families = prometheus::gather();

    let mut buffer = Vec::new();
    match encoder.encode(&metric_families, &mut buffer) {
        Ok(_) => Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", encoder.format_type())
            .body(Body::from(buffer))
            .unwrap(),
        Err(e) => Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(Body::from(format!("Error encoding metrics: {}", e)))
            .unwrap(),
    }
}

/// Handler for /health endpoint
async fn health_handler() -> Response<Body> {
    Response::builder()
        .status(StatusCode::OK)
        .body(Body::from("{\"status\":\"healthy\"}"))
        .unwrap()
}

/// Update mempool metrics
pub fn update_mempool_metrics(standard: usize, model: usize, inference: usize) {
    MEMPOOL_SIZE
        .with_label_values(&["standard"])
        .set(standard as f64);
    MEMPOOL_SIZE.with_label_values(&["model"]).set(model as f64);
    MEMPOOL_SIZE
        .with_label_values(&["inference"])
        .set(inference as f64);
}

/// Update cache metrics
pub fn update_cache_metrics(cache_type: &str, hit_rate: f64, size: usize) {
    CACHE_HIT_RATE
        .with_label_values(&[cache_type])
        .set(hit_rate);
    CACHE_SIZE.with_label_values(&[cache_type]).set(size as f64);
}

/// Update DAG metrics
pub fn update_dag_metrics(height: u64, tips: usize, blue_score: u64) {
    DAG_HEIGHT.with_label_values(&[]).set(height as f64);
    DAG_TIPS_COUNT.with_label_values(&[]).set(tips as f64);
    DAG_BLUE_SCORE.with_label_values(&[]).set(blue_score as f64);
}

/// Record RPC request
pub fn record_rpc_request(method: &str, duration: f64, success: bool) {
    RPC_REQUEST_DURATION
        .with_label_values(&[method])
        .observe(duration);
    let status = if success { "success" } else { "error" };
    RPC_REQUEST_COUNT.with_label_values(&[method, status]).inc();
}

/// Record execution time
pub fn record_execution_time(tx_type: &str, duration: f64) {
    EXECUTION_TIME
        .with_label_values(&[tx_type])
        .observe(duration);
}

/// Record parallel execution groups
pub fn record_parallel_groups(count: usize) {
    PARALLEL_EXECUTION_GROUPS
        .with_label_values(&[])
        .observe(count as f64);
}
