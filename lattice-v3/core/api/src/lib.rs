pub mod methods;
pub mod server;
pub mod types;
// pub mod websocket;  // Temporarily disabled - needs async runtime integration

pub use server::{RpcServer, RpcConfig};
pub use types::{ApiError, BlockId, BlockTag};
// pub use websocket::WebSocketServer;

use anyhow::Result;
use lattice_storage::StorageManager;
use lattice_sequencer::mempool::Mempool;
use lattice_network::peer::PeerManager;
use lattice_execution::executor::Executor;
use std::sync::Arc;

/// Full API service combining RPC and WebSocket
pub struct ApiService {
    rpc_server: RpcServer,
    // ws_server: WebSocketServer,  // Temporarily disabled
}

impl ApiService {
    /// Create a new API service
    pub fn new(
        rpc_config: RpcConfig,
        _ws_addr: std::net::SocketAddr,  // Will be used when WebSocket is re-enabled
        storage: Arc<StorageManager>,
        mempool: Arc<Mempool>,
        peer_manager: Arc<PeerManager>,
        executor: Arc<Executor>,
    ) -> Self {
        let rpc_server = RpcServer::new(
            rpc_config,
            storage,
            mempool,
            peer_manager,
            executor,
        );
        
        // let ws_server = WebSocketServer::new(ws_addr);
        
        Self {
            rpc_server,
            // ws_server,
        }
    }
    
    /// Start both RPC and WebSocket servers
    pub async fn start(self) -> Result<()> {
        // Start RPC server
        let _rpc = self.rpc_server.start().await?;
        
        // Start WebSocket server (disabled for now)
        // let _ws = self.ws_server.start().await?;
        
        // Keep servers running
        tokio::signal::ctrl_c().await?;
        
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
        
        let _service = ApiService::new(
            rpc_config,
            ws_addr,
            storage,
            mempool,
            peer_manager,
            executor,
        );
    }
}