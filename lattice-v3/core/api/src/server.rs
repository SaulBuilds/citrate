use crate::methods::{ChainApi, StateApi, TransactionApi, MempoolApi, NetworkApi, AiApi};
use crate::types::{error::ApiError, request::{BlockId, TransactionRequest, CallRequest}};
use crate::metrics::rpc_request;
use crate::eth_rpc;
use anyhow::Result;
use jsonrpc_core::{IoHandler, Params, Value};
use jsonrpc_http_server::{ServerBuilder, Server, DomainsValidation, AccessControlAllowOrigin};
use jsonrpc_http_server::CloseHandle;
use lattice_storage::StorageManager;
use lattice_sequencer::mempool::Mempool;
use lattice_network::peer::PeerManager;
use lattice_execution::executor::Executor;
use lattice_execution::types::Address;
use lattice_consensus::types::Hash;
use std::net::SocketAddr;
use std::sync::Arc;
use tracing::info;
use futures::executor::block_on;
use serde_json::json;

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
        let mut io_handler = IoHandler::new();
        
        // Register Ethereum-compatible RPC methods
        eth_rpc::register_eth_methods(
            &mut io_handler,
            storage.clone(),
            mempool.clone(),
            executor.clone(),
        );
        
        // ========== Chain Methods ==========
        
        // chain_getHeight
        let storage_h = storage.clone();
        io_handler.add_sync_method("chain_getHeight", move |_params: Params| {
            rpc_request("chain_getHeight");
            let api = ChainApi::new(storage_h.clone());
            match block_on(api.get_height()) {
                Ok(height) => Ok(Value::Number(height.into())),
                Err(e) => Err(jsonrpc_core::Error::internal_error()),
            }
        });
        
        // chain_getTips
        let storage_t = storage.clone();
        io_handler.add_sync_method("chain_getTips", move |_params: Params| {
            rpc_request("chain_getTips");
            let api = ChainApi::new(storage_t.clone());
            match block_on(api.get_tips()) {
                Ok(tips) => Ok(serde_json::to_value(tips).unwrap_or(Value::Array(vec![]))),
                Err(_) => Ok(Value::Array(vec![])),
            }
        });
        
        // chain_getBlock
        let storage_b = storage.clone();
        io_handler.add_sync_method("chain_getBlock", move |params: Params| {
            rpc_request("chain_getBlock");
            let api = ChainApi::new(storage_b.clone());
            
            let block_id: BlockId = match params.parse() {
                Ok(id) => id,
                Err(e) => return Err(jsonrpc_core::Error::invalid_params(e.to_string())),
            };
            
            match block_on(api.get_block(block_id)) {
                Ok(block) => Ok(serde_json::to_value(block).unwrap_or(Value::Null)),
                Err(ApiError::BlockNotFound(_)) => Ok(Value::Null),
                Err(e) => Err(jsonrpc_core::Error::internal_error()),
            }
        });
        
        // chain_getTransaction
        let storage_tx = storage.clone();
        io_handler.add_sync_method("chain_getTransaction", move |params: Params| {
            rpc_request("chain_getTransaction");
            let api = ChainApi::new(storage_tx.clone());
            
            let hash: Hash = match params.parse() {
                Ok(h) => h,
                Err(e) => return Err(jsonrpc_core::Error::invalid_params(e.to_string())),
            };
            
            match block_on(api.get_transaction(hash)) {
                Ok(tx) => Ok(serde_json::to_value(tx).unwrap_or(Value::Null)),
                Err(ApiError::TransactionNotFound(_)) => Ok(Value::Null),
                Err(_) => Err(jsonrpc_core::Error::internal_error()),
            }
        });
        
        // ========== State Methods ==========
        
        // state_getBalance
        let storage_bal = storage.clone();
        let executor_bal = executor.clone();
        io_handler.add_sync_method("state_getBalance", move |params: Params| {
            rpc_request("state_getBalance");
            let api = StateApi::new(storage_bal.clone(), executor_bal.clone());
            
            let address: Address = match params.parse() {
                Ok(addr) => addr,
                Err(e) => return Err(jsonrpc_core::Error::invalid_params(e.to_string())),
            };
            
            match block_on(api.get_balance(address)) {
                Ok(balance) => Ok(serde_json::to_value(balance).unwrap_or(Value::String("0".to_string()))),
                Err(_) => Ok(Value::String("0".to_string())),
            }
        });
        
        // state_getNonce
        let storage_n = storage.clone();
        let executor_n = executor.clone();
        io_handler.add_sync_method("state_getNonce", move |params: Params| {
            rpc_request("state_getNonce");
            let api = StateApi::new(storage_n.clone(), executor_n.clone());
            
            let address: Address = match params.parse() {
                Ok(addr) => addr,
                Err(e) => return Err(jsonrpc_core::Error::invalid_params(e.to_string())),
            };
            
            match block_on(api.get_nonce(address)) {
                Ok(nonce) => Ok(Value::Number(nonce.into())),
                Err(_) => Ok(Value::Number(0.into())),
            }
        });
        
        // state_getCode
        let storage_c = storage.clone();
        let executor_c = executor.clone();
        io_handler.add_sync_method("state_getCode", move |params: Params| {
            rpc_request("state_getCode");
            let api = StateApi::new(storage_c.clone(), executor_c.clone());
            
            let address: Address = match params.parse() {
                Ok(addr) => addr,
                Err(e) => return Err(jsonrpc_core::Error::invalid_params(e.to_string())),
            };
            
            match block_on(api.get_code(address)) {
                Ok(code) => Ok(Value::String(hex::encode(code))),
                Err(_) => Ok(Value::String("0x".to_string())),
            }
        });
        
        // ========== Transaction Methods ==========
        
        // tx_sendRawTransaction
        let mempool_raw = mempool.clone();
        let executor_raw = executor.clone();
        io_handler.add_sync_method("tx_sendRawTransaction", move |params: Params| {
            rpc_request("tx_sendRawTransaction");
            let api = TransactionApi::new(mempool_raw.clone(), executor_raw.clone());
            
            let raw_hex: String = match params.parse() {
                Ok(hex) => hex,
                Err(e) => return Err(jsonrpc_core::Error::invalid_params(e.to_string())),
            };
            
            let raw_bytes = match hex::decode(raw_hex.trim_start_matches("0x")) {
                Ok(bytes) => bytes,
                Err(e) => return Err(jsonrpc_core::Error::invalid_params(format!("Invalid hex: {}", e))),
            };
            
            match block_on(api.send_raw_transaction(raw_bytes)) {
                Ok(hash) => Ok(Value::String(format!("0x{}", hex::encode(hash.as_bytes())))),
                Err(e) => Err(jsonrpc_core::Error::invalid_params(e.to_string())),
            }
        });
        
        // tx_estimateGas
        let mempool_gas = mempool.clone();
        let executor_gas = executor.clone();
        io_handler.add_sync_method("tx_estimateGas", move |params: Params| {
            rpc_request("tx_estimateGas");
            let api = TransactionApi::new(mempool_gas.clone(), executor_gas.clone());
            
            let request: CallRequest = match params.parse() {
                Ok(req) => req,
                Err(e) => return Err(jsonrpc_core::Error::invalid_params(e.to_string())),
            };
            
            match block_on(api.estimate_gas(request)) {
                Ok(gas) => Ok(Value::String(format!("0x{:x}", gas))),
                Err(_) => Ok(Value::String("0x5208".to_string())), // Default 21000
            }
        });
        
        // tx_getGasPrice
        let mempool_price = mempool.clone();
        let executor_price = executor.clone();
        io_handler.add_sync_method("tx_getGasPrice", move |_params: Params| {
            rpc_request("tx_getGasPrice");
            let api = TransactionApi::new(mempool_price.clone(), executor_price.clone());
            
            match block_on(api.get_gas_price()) {
                Ok(price) => Ok(Value::String(format!("0x{:x}", price))),
                Err(_) => Ok(Value::String("0x3b9aca00".to_string())), // 1 Gwei
            }
        });
        
        // ========== Mempool Methods ==========
        
        // mempool_getStatus
        let mempool_status = mempool.clone();
        io_handler.add_sync_method("mempool_getStatus", move |_params: Params| {
            rpc_request("mempool_getStatus");
            let api = MempoolApi::new(mempool_status.clone());
            
            match block_on(api.get_status()) {
                Ok(status) => Ok(serde_json::to_value(status).unwrap_or(Value::Null)),
                Err(_) => Ok(serde_json::json!({
                    "pending": 0,
                    "queued": 0,
                    "total_size": 0,
                    "max_size": 10000000
                })),
            }
        });
        
        // mempool_getPending
        let mempool_pending = mempool.clone();
        io_handler.add_sync_method("mempool_getPending", move |params: Params| {
            rpc_request("mempool_getPending");
            let api = MempoolApi::new(mempool_pending.clone());
            
            let limit: Option<usize> = params.parse().ok();
            
            match block_on(api.get_pending(limit)) {
                Ok(txs) => Ok(serde_json::to_value(txs).unwrap_or(Value::Array(vec![]))),
                Err(_) => Ok(Value::Array(vec![])),
            }
        });
        
        // ========== Network Methods ==========
        
        // net_peerCount
        let peers_count = peer_manager.clone();
        io_handler.add_sync_method("net_peerCount", move |_params: Params| {
            rpc_request("net_peerCount");
            let api = NetworkApi::new(peers_count.clone());
            
            match block_on(api.get_peer_count()) {
                Ok(count) => Ok(Value::String(format!("0x{:x}", count))),
                Err(_) => Ok(Value::String("0x0".to_string())),
            }
        });
        
        // net_listening
        let peers_listen = peer_manager.clone();
        io_handler.add_sync_method("net_listening", move |_params: Params| {
            rpc_request("net_listening");
            let api = NetworkApi::new(peers_listen.clone());
            
            match block_on(api.is_listening()) {
                Ok(listening) => Ok(Value::Bool(listening)),
                Err(_) => Ok(Value::Bool(true)),
            }
        });
        
        // net_version (chain ID)
        io_handler.add_sync_method("net_version", |_params: Params| {
            rpc_request("net_version");
            Ok(Value::String("1337".to_string()))
        });
        
        // eth_chainId (compatibility)
        io_handler.add_sync_method("eth_chainId", |_params: Params| {
            rpc_request("eth_chainId");
            Ok(Value::String("0x539".to_string())) // 1337 in hex
        });
        
        Self {
            config,
            storage,
            mempool,
            peer_manager,
            executor,
            io_handler,
        }
    }
    
    /// Spawn the RPC server on a dedicated OS thread and return a CloseHandle and JoinHandle.
    /// If startup fails (e.g., port already in use), returns an error instead of panicking.
    pub fn spawn(self) -> Result<(CloseHandle, std::thread::JoinHandle<()>)> {
        let listen_addr = self.config.listen_addr;
        let threads = self.config.threads;
        let cors_any = !self.config.cors_domains.is_empty();
        let io = self.io_handler;

        // Channel to report startup result (CloseHandle or error string)
        let (result_tx, result_rx) = std::sync::mpsc::sync_channel::<Result<CloseHandle, String>>(1);

        let join_handle = std::thread::spawn(move || {
            let mut builder = ServerBuilder::new(io);
            if cors_any {
                builder = builder.cors(DomainsValidation::AllowOnly(vec![AccessControlAllowOrigin::Any]));
            }
            match builder
                .max_request_body_size(10 * 1024 * 1024)
                .threads(threads)
                .start_http(&listen_addr)
            {
                Ok(server) => {
                    info!("RPC server listening on {}", listen_addr);
                    // Send close handle back to caller
                    let _ = result_tx.send(Ok(server.close_handle()));
                    // Block until closed
                    server.wait();
                }
                Err(e) => {
                    let _ = result_tx.send(Err(format!("Failed to start RPC server on {}: {}", listen_addr, e)));
                    // Exit thread
                }
            }
        });

        // Receive startup result
        match result_rx.recv() {
            Ok(Ok(close)) => Ok((close, join_handle)),
            Ok(Err(msg)) => {
                // Ensure the thread has exited before returning error
                let _ = join_handle.join();
                Err(anyhow::anyhow!(msg))
            }
            Err(e) => {
                // Thread ended before sending a result
                let _ = join_handle.join();
                Err(anyhow::anyhow!("RPC thread failed: {}", e))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use lattice_sequencer::mempool::MempoolConfig;
    use lattice_network::peer::PeerManagerConfig;
    use lattice_storage::pruning::PruningConfig;
    use lattice_consensus::types::{Transaction, Hash, PublicKey, Signature};
    use lattice_consensus::crypto::{generate_keypair, sign_transaction};

    #[tokio::test]
    async fn test_rpc_chain_height_and_tx_submit() {
        let temp_dir = TempDir::new().unwrap();
        let storage = Arc::new(StorageManager::new(temp_dir.path(), PruningConfig::default()).unwrap());
        let mempool = Arc::new(Mempool::new(MempoolConfig::default()));
        let peer_manager = Arc::new(PeerManager::new(PeerManagerConfig::default()));
        let state_db = Arc::new(lattice_execution::StateDB::new());
        let executor = Arc::new(Executor::new(state_db));

        let rpc = RpcServer::new(RpcConfig::default(), storage, mempool.clone(), peer_manager, executor);

        // chain_getHeight
        let req = serde_json::json!({"jsonrpc":"2.0","id":1,"method":"chain_getHeight","params":[]}).to_string();
        let resp = rpc.io_handler.handle_request(&req).await.unwrap();
        let v: serde_json::Value = serde_json::from_str(&resp).unwrap();
        assert_eq!(v["result"], 0);

        // Note: tx submission path is covered via integration tests elsewhere.
    }
}
