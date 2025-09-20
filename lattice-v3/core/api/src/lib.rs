pub mod methods;
pub mod server;
pub mod types;
pub mod metrics;
pub mod metrics_server;
pub mod eth_rpc_simple;
pub mod eth_rpc;
pub mod eth_tx_decoder;
pub mod websocket;
pub mod openai_api;

pub use server::{RpcServer, RpcConfig};
pub use types::{ApiError, BlockId, BlockTag};
pub use websocket::WebSocketServer;
pub use openai_api::OpenAiRestServer;

use anyhow::Result;
use lattice_storage::StorageManager;
use lattice_sequencer::mempool::Mempool;
use lattice_network::peer::PeerManager;
use lattice_execution::executor::Executor;
use std::sync::Arc;

/// Full API service combining RPC, WebSocket, and REST API
pub struct ApiService {
    rpc_server: RpcServer,
    ws_server: WebSocketServer,
    rest_server: OpenAiRestServer,
    rest_addr: std::net::SocketAddr,
}

impl ApiService {
    /// Create a new API service
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        rpc_config: RpcConfig,
        ws_addr: std::net::SocketAddr,
        rest_addr: std::net::SocketAddr,
        storage: Arc<StorageManager>,
        mempool: Arc<Mempool>,
        peer_manager: Arc<PeerManager>,
        executor: Arc<Executor>,
        chain_id: u64,
    ) -> Self {
        let rpc_server = RpcServer::new(
            rpc_config,
            storage.clone(),
            mempool.clone(),
            peer_manager,
            executor.clone(),
            chain_id,
        );
        
        let ws_server = WebSocketServer::new(ws_addr);
        let rest_server = OpenAiRestServer::new(storage, mempool, executor);
        
        Self {
            rpc_server,
            ws_server,
            rest_server,
            rest_addr,
        }
    }
    
    /// Start RPC, WebSocket, and REST API servers
    pub async fn start(self) -> Result<()> {
        // Start RPC server on a dedicated OS thread
        let (close_handle, join_handle) = self.rpc_server.spawn()?;
        
        // Start WebSocket server
        let ws_server = self.ws_server;
        tokio::spawn(async move {
            if let Err(e) = ws_server.start().await {
                tracing::error!("WebSocket server error: {}", e);
            }
        });
        
        // Start REST API server
        let rest_server = self.rest_server;
        let rest_addr = self.rest_addr;
        tokio::spawn(async move {
            if let Err(e) = rest_server.start(rest_addr).await {
                tracing::error!("REST API server error: {}", e);
            }
        });

        // Wait for shutdown signal
        tokio::signal::ctrl_c().await?;

        // Signal server to close and join its OS thread without blocking this async task.
        close_handle.close();
        tokio::task::spawn_blocking(move || {
            let _ = join_handle.join();
        })
        .await
        .ok();

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use lattice_sequencer::mempool::MempoolConfig;
    use lattice_network::peer::PeerManagerConfig;
    use lattice_storage::pruning::PruningConfig;
    
    #[tokio::test]
    async fn test_api_service_creation() {
        let temp_dir = TempDir::new().unwrap();
        
        // Create dependencies
        let storage = Arc::new(
            StorageManager::new(temp_dir.path(), PruningConfig::default()).unwrap()
        );
        let mempool = Arc::new(Mempool::new(MempoolConfig::default()));
        let peer_manager = Arc::new(PeerManager::new(PeerManagerConfig::default()));
        let state_db = Arc::new(lattice_execution::StateDB::new());
        let executor = Arc::new(Executor::new(state_db));
        
        // Create API service
        let rpc_config = RpcConfig::default();
        let ws_addr = "127.0.0.1:8546".parse().unwrap();
        let rest_addr = "127.0.0.1:3000".parse().unwrap();
        
        let _service = ApiService::new(
            rpc_config,
            ws_addr,
            rest_addr,
            storage,
            mempool,
            peer_manager,
            executor,
            1,
        );
    }
}
