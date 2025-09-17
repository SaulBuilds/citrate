use anyhow::Result;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::{error, info};

use lattice_api::{ApiService, RpcConfig};
use lattice_execution::{Executor, StateDB};
use lattice_network::peer::{PeerManager, PeerManagerConfig};
use lattice_sequencer::mempool::{Mempool, MempoolConfig};
use lattice_storage::{StorageManager};
use lattice_storage::pruning::PruningConfig;
use axum::{routing::get, Router, response::IntoResponse};
use prometheus::{Encoder, TextEncoder, Registry, default_registry, gather};

fn data_dir() -> PathBuf {
    std::env::var_os("LATTICE_DATA_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/data"))
}

fn rpc_addr() -> SocketAddr {
    std::env::var("LATTICE_RPC_ADDR")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or_else(|| "0.0.0.0:8545".parse().unwrap())
}

fn metrics_addr() -> SocketAddr {
    std::env::var("LATTICE_METRICS_ADDR")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or_else(|| "0.0.0.0:9100".parse().unwrap())
}

async fn metrics_handler() -> impl IntoResponse {
    let encoder = TextEncoder::new();
    let metric_families = gather();
    let mut buf = Vec::new();
    if let Err(e) = encoder.encode(&metric_families, &mut buf) {
        return (axum::http::StatusCode::INTERNAL_SERVER_ERROR, format!("encode error: {}", e));
    }
    (axum::http::StatusCode::OK, String::from_utf8(buf).unwrap_or_default())
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<()> {
    // Logging
    tracing_subscriber::fmt()
        .with_env_filter(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info,lattice=info".into()),
        )
        .with_target(false)
        .init();

    // Storage
    let data_dir = data_dir();
    std::fs::create_dir_all(&data_dir)?;
    let pruning = PruningConfig::default();
    let storage = Arc::new(StorageManager::new(&data_dir, pruning)?);

    // Executor
    let state_db = Arc::new(StateDB::new());
    let executor = Arc::new(Executor::new(state_db));

    // Mempool (default config)
    let mempool = Arc::new(Mempool::new(MempoolConfig::default()));

    // Network (placeholder manager)
    let peer_manager = Arc::new(PeerManager::new(PeerManagerConfig::default()));

    // RPC config
    let mut rpc_cfg = RpcConfig::default();
    rpc_cfg.listen_addr = rpc_addr();
    info!("Starting Lattice RPC on {} (data_dir={:?})", rpc_cfg.listen_addr, data_dir);

    // Start metrics server
    let maddr = metrics_addr();
    tokio::spawn(async move {
        let app = Router::new().route("/metrics", get(metrics_handler));
        let listener = tokio::net::TcpListener::bind(&maddr).await.unwrap();
        info!("Metrics server listening on {}", maddr);
        if let Err(e) = axum::serve(listener, app).await {
            tracing::error!("metrics server error: {}", e);
        }
    });

    // API service (WebSocket and REST addresses)
    let ws_addr: SocketAddr = "0.0.0.0:8546".parse().unwrap();
    let rest_addr: SocketAddr = "0.0.0.0:3000".parse().unwrap();
    let api = ApiService::new(
        rpc_cfg,
        ws_addr,
        rest_addr,
        storage,
        mempool,
        peer_manager,
        executor,
        1,
    );

    // Start
    if let Err(e) = api.start().await {
        error!("API service exited with error: {e}");
    }
    Ok(())
}
