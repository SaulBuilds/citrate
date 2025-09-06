use crate::methods::{ChainApi, StateApi, TransactionApi, MempoolApi, NetworkApi, AiApi};
use crate::types::error::ApiError;
use anyhow::Result;
use jsonrpc_core::{IoHandler, Params, Value, Error as RpcError};
use jsonrpc_http_server::{ServerBuilder, Server, DomainsValidation, AccessControlAllowOrigin};
use lattice_storage::StorageManager;
use lattice_sequencer::mempool::Mempool;
use lattice_network::peer::PeerManager;
use lattice_execution::executor::Executor;
use std::net::SocketAddr;
use std::sync::Arc;
use tracing::{info, error};

/// RPC Server configuration
#[derive(Clone)]
pub struct RpcConfig {
    pub listen_addr: SocketAddr,
    pub max_connections: u32,
    pub cors_domains: Vec<String>,
    pub threads: usize,
}

impl Default for RpcConfig {
    fn default() -> Self {
        Self {
            listen_addr: "127.0.0.1:8545".parse().unwrap(),
            max_connections: 100,
            cors_domains: vec!["*".to_string()],
            threads: 4,
        }
    }
}

/// RPC Server
pub struct RpcServer {
    config: RpcConfig,
    storage: Arc<StorageManager>,
    mempool: Arc<Mempool>,
    peer_manager: Arc<PeerManager>,
    executor: Arc<Executor>,
    io_handler: IoHandler,
}

impl RpcServer {
    pub fn new(
        config: RpcConfig,
        storage: Arc<StorageManager>,
        mempool: Arc<Mempool>,
        peer_manager: Arc<PeerManager>,
        executor: Arc<Executor>,
    ) -> Self {
        let io_handler = IoHandler::new();
        
        // API instances are created but not yet integrated
        // This is a placeholder implementation to make Sprint 7 compile
        // In production, these would be properly integrated with async handlers
        let _chain_api = ChainApi::new(storage.clone());
        let _state_api = StateApi::new(storage.clone(), executor.clone());
        let _tx_api = TransactionApi::new(mempool.clone(), executor.clone());
        let _mempool_api = MempoolApi::new(mempool.clone());
        let _network_api = NetworkApi::new(peer_manager.clone());
        let _ai_api = AiApi::new(storage.clone());
        
        // Note: Method registration requires async runtime integration
        // which would be completed in the next sprint
        
        Self {
            config,
            storage,
            mempool,
            peer_manager,
            executor,
            io_handler,
        }
    }
    
    /// Start the RPC server
    pub async fn start(self) -> Result<Server> {
        let mut builder = ServerBuilder::new(self.io_handler);
        
        // Configure CORS if domains are specified
        if !self.config.cors_domains.is_empty() {
            builder = builder.cors(DomainsValidation::AllowOnly(vec![AccessControlAllowOrigin::Any]));
        }
        
        let server = builder
            .max_request_body_size(10 * 1024 * 1024) // 10MB
            .threads(self.config.threads)
            .start_http(&self.config.listen_addr)?;
        
        info!("RPC server listening on {}", self.config.listen_addr);
        Ok(server)
    }
}