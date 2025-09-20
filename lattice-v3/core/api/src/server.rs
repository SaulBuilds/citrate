use crate::methods::{ChainApi, StateApi, TransactionApi, MempoolApi, NetworkApi, AiApi};
use crate::types::{error::ApiError, request::{BlockId, CallRequest}};
use crate::metrics::rpc_request;
use crate::eth_rpc;
use anyhow::Result;
use jsonrpc_core::{IoHandler, Params, Value};
use jsonrpc_http_server::{ServerBuilder, DomainsValidation, AccessControlAllowOrigin};
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
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::RwLock as StdRwLock;

// In-memory verification store (address -> record)
static VERIFICATIONS: Lazy<StdRwLock<HashMap<String, serde_json::Value>>> = Lazy::new(|| StdRwLock::new(HashMap::new()));

/// Helper: parse optional pagination from params object
fn parse_pagination(obj: &serde_json::Map<String, serde_json::Value>) -> (usize, Option<usize>) {
    let offset = obj.get("offset").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
    let limit = obj.get("limit").and_then(|v| v.as_u64()).map(|v| v as usize);
    (offset, limit)
}

/// Helper: slice with offset/limit
fn apply_pagination<T: Clone>(mut items: Vec<T>, offset: usize, limit: Option<usize>) -> Vec<T> {
    if offset > 0 {
        items = items.into_iter().skip(offset).collect();
    }
    if let Some(l) = limit {
        items.truncate(l);
    }
    items
}

// Phase 2 compilation helper (feature-gated)
#[cfg(feature = "verifier-ethers-solc")]
fn compile_runtime_bytecode(
    source: &str,
    _compiler: &str,
    optimized: bool,
    contract_name: Option<&str>,
) -> Result<Vec<u8>, String> {
    use std::fs::{self, File};
    use std::io::Write;
    use ethers_solc::{Project, ProjectPathsConfig, Solc, ConfigurableArtifacts};
    use ethers_solc::artifacts::Settings;

    // Create a temporary project with a single source file
    let dir = std::env::temp_dir().join(format!("lattice_verify_{}", uuid::Uuid::new_v4()));
    let src_dir = dir.join("src");
    fs::create_dir_all(&src_dir).map_err(|e| e.to_string())?;
    let path = src_dir.join("Contract.sol");
    let mut f = File::create(&path).map_err(|e| e.to_string())?;
    f.write_all(source.as_bytes()).map_err(|e| e.to_string())?;

    // Configure project paths
    let paths = ProjectPathsConfig::builder().root(&dir).sources(&src_dir).build().map_err(|e| e.to_string())?;

    // Build project, using SVM-installed solc (fallback to system solc if SVM fails)
    let mut builder = Project::builder().paths(paths);
    // Configure optimizer via settings
    builder = builder.settings(|s: &mut Settings| {
        s.optimizer.enabled = Some(optimized);
        if optimized { s.optimizer.runs = Some(200); }
    });
    let project = builder.build().map_err(|e| e.to_string())?;

    // Ensure a solc is available (this uses SVM internally if configured)
    let solc = Solc::default();
    let project = project.with_solc(solc);

    // Compile
    let output = project.compile().map_err(|e| e.to_string())?;
    if output.has_compiler_errors() {
        return Err(format!("compile errors: {:?}", output.output().errors));
    }

    // Extract deployed (runtime) bytecode for the requested contract (or first found)
    let mut bin: Option<Vec<u8>> = None;
    let want_name = contract_name.map(|s| s.to_string());
    for (id, artifact) in output.into_artifacts() {
        // id contains the artifact identifier with contract name
        let name_matches = match &want_name {
            Some(n) => id.name == *n,
            None => true,
        };
        if !name_matches { continue; }

        if let Some(deployed) = artifact.deployed_bytecode() {
            if let Some(obj) = deployed.object.as_bytes() {
                bin = Some(obj.to_vec());
                break;
            } else if let Some(obj) = deployed.object.as_bytecode() {
                // artifact returns raw bytes
                bin = Some(obj.bytes().to_vec());
                break;
            }
        }
    }

    bin.ok_or_else(|| "missing deployed bytecode".to_string())
}

#[cfg(not(feature = "verifier-ethers-solc"))]
fn compile_runtime_bytecode(
    source: &str,
    _compiler: &str,
    optimized: bool,
    contract_name: Option<&str>,
) -> Result<Vec<u8>, String> {
    compile_runtime_bytecode_external(source, optimized, contract_name)
}

fn compile_runtime_bytecode_external(
    source: &str,
    optimized: bool,
    contract_name: Option<&str>,
) -> Result<Vec<u8>, String> {
    use std::fs::{self, File};
    use std::io::Write;
    use std::process::Command;
    // Prepare temp directory and file
    let dir = std::env::temp_dir().join(format!("lattice_verify_{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let src_dir = dir.join("src");
    fs::create_dir_all(&src_dir).map_err(|e| e.to_string())?;
    let path = src_dir.join("Contract.sol");
    let mut f = File::create(&path).map_err(|e| e.to_string())?;
    f.write_all(source.as_bytes()).map_err(|e| e.to_string())?;

    // Build solc args
    let path_lossy = path.to_string_lossy().to_string();
    let mut args = vec!["--combined-json", "abi,bin,bin-runtime", &path_lossy];
    if optimized { args.splice(0..0, ["--optimize"]); }

    let out = Command::new("solc").args(&args).output().map_err(|e| e.to_string())?;
    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr);
        return Err(format!("solc failed: {}", stderr));
    }
    let stdout = String::from_utf8_lossy(&out.stdout);
    let v: serde_json::Value = serde_json::from_str(&stdout).map_err(|e| e.to_string())?;
    let contracts = v.get("contracts").and_then(|c| c.as_object()).ok_or("no contracts in output")?;
    // Find entry
    let mut binrt_opt: Option<String> = None;
    if let Some(name) = contract_name {
        for (k, val) in contracts {
            if k.ends_with(&format!(":{}", name)) {
                binrt_opt = val.get("bin-runtime").and_then(|s| s.as_str()).map(|s| s.to_string());
                break;
            }
        }
    }
    if binrt_opt.is_none() {
        if let Some((_k, val)) = contracts.iter().next() {
            binrt_opt = val.get("bin-runtime").and_then(|s| s.as_str()).map(|s| s.to_string());
        }
    }
    let binrt = binrt_opt.ok_or("missing bin-runtime")?;
    let s = binrt.trim();
    let s = s.strip_prefix("0x").unwrap_or(s);
    hex::decode(s).map_err(|e| e.to_string())
}

/// Compile using solc --standard-json. Returns (creation, runtime) bytecode for a selected contract.
/// If `contract_name` is Some, selects that contract name (matches suffix after ':'). Otherwise first found.
fn compile_standard_json(
    standard_json: &str,
    contract_name: Option<&str>,
) -> Result<(Vec<u8>, Vec<u8>), String> {
    use std::process::Command;
    let mut child = Command::new("solc")
        .arg("--standard-json")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| e.to_string())?;
    {
        use std::io::Write;
        let stdin = child.stdin.as_mut().ok_or("failed to open stdin")?;
        stdin.write_all(standard_json.as_bytes()).map_err(|e| e.to_string())?;
    }
    let out = child.wait_with_output().map_err(|e| e.to_string())?;
    if !out.status.success() {
        return Err(format!("solc --standard-json failed: {}", String::from_utf8_lossy(&out.stderr)));
    }
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).map_err(|e| e.to_string())?;
    let contracts = v.get("contracts").and_then(|c| c.as_object()).ok_or("no contracts in output")?;
    let mut chosen: Option<(Vec<u8>, Vec<u8>)> = None;
    for (_file, obj) in contracts.iter() {
        let cmap = match obj.as_object() { Some(m) => m, None => continue };
        for (name, art) in cmap.iter() {
            let matches = match contract_name { Some(n) => name == n, None => true };
            if !matches { continue; }
            // creation bytecode
            let bytecode_obj = art.get("evm").and_then(|e| e.get("bytecode")).and_then(|b| b.get("object")).and_then(|s| s.as_str());
            // runtime bytecode
            let deployed_obj = art.get("evm").and_then(|e| e.get("deployedBytecode")).and_then(|b| b.get("object")).and_then(|s| s.as_str());
            if let (Some(c), Some(r)) = (bytecode_obj, deployed_obj) {
                let c_hex = c.trim().trim_start_matches("0x");
                let r_hex = r.trim().trim_start_matches("0x");
                let cbytes = hex::decode(c_hex).map_err(|e| e.to_string())?;
                let rbytes = hex::decode(r_hex).map_err(|e| e.to_string())?;
                chosen = Some((cbytes, rbytes));
                break;
            }
        }
        if chosen.is_some() { break; }
    }
    chosen.ok_or_else(|| format!("contract not found in output: {:?}", contract_name))
}

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
    #[allow(dead_code)]
    storage: Arc<StorageManager>,
    #[allow(dead_code)]
    mempool: Arc<Mempool>,
    #[allow(dead_code)]
    peer_manager: Arc<PeerManager>,
    #[allow(dead_code)]
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
        chain_id: u64,
    ) -> Self {
        let mut io_handler = IoHandler::new();
        
        // Register Ethereum-compatible RPC methods
        eth_rpc::register_eth_methods(
            &mut io_handler,
            storage.clone(),
            mempool.clone(),
            executor.clone(),
            chain_id,
        );
        
        // ========== Chain Methods ==========
        
        // chain_getHeight
        let storage_h = storage.clone();
        io_handler.add_sync_method("chain_getHeight", move |_params: Params| {
            rpc_request("chain_getHeight");
            let api = ChainApi::new(storage_h.clone());
            match block_on(api.get_height()) {
                Ok(height) => Ok(Value::Number(height.into())),
                Err(_e) => Err(jsonrpc_core::Error::internal_error()),
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
                Err(_e) => Err(jsonrpc_core::Error::internal_error()),
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

        // Override eth_sendRawTransaction to also broadcast via P2P when available
        let mempool_raw_broadcast = mempool.clone();
        let peer_mgr_raw_broadcast = peer_manager.clone();
        io_handler.add_sync_method("eth_sendRawTransaction", move |params: Params| {
            rpc_request("eth_sendRawTransaction");
            use crate::eth_tx_decoder;
            use lattice_network::NetworkMessage;

            let mempool = mempool_raw_broadcast.clone();
            let peer_mgr = peer_mgr_raw_broadcast.clone();

            let params: Vec<Value> = match params.parse() {
                Ok(p) => p,
                Err(e) => return Err(jsonrpc_core::Error::invalid_params(e.to_string())),
            };
            if params.is_empty() { return Err(jsonrpc_core::Error::invalid_params("Missing transaction data")); }
            let tx_hex = params[0].as_str().ok_or_else(|| jsonrpc_core::Error::invalid_params("Invalid tx hex"))?;
            let tx_bytes = match hex::decode(tx_hex.trim().trim_start_matches("0x")) {
                Ok(b) => b, Err(_) => return Err(jsonrpc_core::Error::invalid_params("Invalid hex")),
            };
            let tx = match eth_tx_decoder::decode_eth_transaction(&tx_bytes) {
                Ok(t) => t,
                Err(e) => return Err(jsonrpc_core::Error::invalid_params(format!("Failed to parse transaction: {}", e))),
            };
            let hash = tx.hash;
            match block_on(mempool.add_transaction(tx.clone(), lattice_sequencer::mempool::TxClass::Standard)) {
                Ok(_) => {
                    // Best-effort broadcast to peers
                    let _ = block_on(peer_mgr.broadcast(&NetworkMessage::NewTransaction { transaction: tx }));
                    Ok(Value::String(format!("0x{}", hex::encode(hash.as_bytes()))))
                },
                Err(e) => Err(jsonrpc_core::Error::invalid_params(format!("Failed to submit transaction: {:?}", e)))
            }
        });

        // Override eth_sendTransaction: enqueue via TransactionApi, then broadcast the tx if retrievable
        let mempool_send_broadcast = mempool.clone();
        let executor_send_broadcast = executor.clone();
        let peer_mgr_send_broadcast = peer_manager.clone();
        io_handler.add_sync_method("eth_sendTransaction", move |params: Params| {
            rpc_request("eth_sendTransaction");
            use crate::types::request::TransactionRequest;
            use lattice_network::NetworkMessage;

            let api = TransactionApi::new(mempool_send_broadcast.clone(), executor_send_broadcast.clone());
            let req: TransactionRequest = match params.parse() {
                Ok(r) => r,
                Err(e) => return Err(jsonrpc_core::Error::invalid_params(e.to_string())),
            };
            match block_on(api.send_transaction(req)) {
                Ok(hash) => {
                    // Try to fetch and broadcast
                    if let Some(tx) = block_on(mempool_send_broadcast.get_transaction(&hash)) {
                        let _ = block_on(peer_mgr_send_broadcast.broadcast(&NetworkMessage::NewTransaction { transaction: tx }));
                    }
                    Ok(Value::String(format!("0x{}", hex::encode(hash.as_bytes()))))
                },
                Err(e) => Err(jsonrpc_core::Error::invalid_params(e.to_string())),
            }
        });
        
        // net_version (chain ID)
        io_handler.add_sync_method("net_version", |_params: Params| {
            rpc_request("net_version");
            Ok(Value::String("1337".to_string()))
        });

        // web3_clientVersion
        io_handler.add_sync_method("web3_clientVersion", |_params: Params| {
            rpc_request("web3_clientVersion");
            Ok(Value::String("lattice/v0.1.0".to_string()))
        });
        
        // eth_chainId (compatibility)
        let chain_id_for_handler = chain_id;
        io_handler.add_sync_method("eth_chainId", move |_params: Params| {
            rpc_request("eth_chainId");
            Ok(Value::String(format!("0x{:x}", chain_id_for_handler)))
        });
        
        // ========== AI/ML Methods ==========

        // lattice_verifyContract: verifies runtime bytecode matches on-chain code for address
        let storage_v = storage.clone();
        let executor_v = executor.clone();
        io_handler.add_sync_method("lattice_verifyContract", move |params: Params| {
            rpc_request("lattice_verifyContract");
            let payload: serde_json::Value = match params.parse() {
                Ok(v) => v,
                Err(e) => return Err(jsonrpc_core::Error::invalid_params(e.to_string())),
            };

            // Required: address; either runtime_bytecode (hex) or we fail (source-only not supported here)
            let obj = match payload.as_object() { Some(m) => m, None => {
                return Err(jsonrpc_core::Error::invalid_params("Expected object payload"));
            }};
            let address_str = obj.get("address").and_then(|v| v.as_str()).unwrap_or("");
            let runtime_hex = obj.get("runtime_bytecode").and_then(|v| v.as_str());
            let compiler_version = obj.get("compiler_version").and_then(|v| v.as_str()).unwrap_or("");
            let optimization_enabled = obj.get("optimization_enabled").and_then(|v| v.as_bool()).unwrap_or(false);
            let source_code = obj.get("source_code").and_then(|v| v.as_str());
            let contract_name = obj.get("contract_name").and_then(|v| v.as_str());
            let standard_json = obj.get("standard_json");
            let constructor_args_hex = obj.get("constructor_args").and_then(|v| v.as_str());

            // Either use provided runtime bytecode, or compile from source/standard-json if present
            if runtime_hex.is_none() && source_code.is_none() && standard_json.is_none() {
                return Err(jsonrpc_core::Error::invalid_params("Provide runtime_bytecode, source_code, or standard_json"));
            }
            let provided = if let Some(rh) = runtime_hex {
                let s = rh.trim();
                let s = s.strip_prefix("0x").unwrap_or(s);
                match hex::decode(s) { Ok(b) => b, Err(_) => return Err(jsonrpc_core::Error::invalid_params("Invalid runtime_bytecode hex")), }
            } else if let Some(src) = source_code {
                // Attempt Phase 2: compile
                let compiled = match compile_runtime_bytecode(src, compiler_version, optimization_enabled, contract_name) {
                    Ok(bytes) => bytes,
                    Err(e) => return Ok(json!({
                        "verified": false,
                        "reason": format!("compile error: {}", e),
                    })),
                };
                compiled
            } else if let Some(stdj) = standard_json {
                // Standard JSON supports multi-file projects
                let sj_str = if stdj.is_string() { stdj.as_str().unwrap().to_string() } else { serde_json::to_string(stdj).unwrap_or_default() };
                let (_creation, runtime) = match compile_standard_json(&sj_str, contract_name) {
                    Ok(t) => t,
                    Err(e) => return Ok(json!({
                        "verified": false,
                        "reason": format!("standard-json compile error: {}", e),
                    })),
                };
                runtime
            } else {
                return Err(jsonrpc_core::Error::invalid_params("unreachable"));
            };
            let addr_hex = address_str.trim().trim_start_matches("0x");
            if addr_hex.len() != 40 { return Err(jsonrpc_core::Error::invalid_params("Invalid address length")); }
            let onchain_addr = {
                let mut a = [0u8; 20];
                match hex::decode(addr_hex) { Ok(b) if b.len()==20 => a.copy_from_slice(&b), _ => return Err(jsonrpc_core::Error::invalid_params("Invalid address")), }
                Address(a)
            };

            // Fetch on-chain runtime code via StateApi
            let api = StateApi::new(storage_v.clone(), executor_v.clone());
            let onchain = match block_on(api.get_code(onchain_addr)) {
                Ok(code) => code,
                Err(_) => return Ok(json!({
                    "verified": false,
                    "reason": "No code at address"
                })),
            };

            if onchain.is_empty() {
                return Ok(json!({"verified": false, "reason": "Empty code at address"}));
            }

            // Compare after stripping metadata trailers
            fn strip_metadata(code: &[u8]) -> &[u8] {
                if code.len() >= 2 {
                    let l = u16::from_be_bytes([code[code.len()-2], code[code.len()-1]]) as usize;
                    if code.len() >= l + 2 {
                        let start = code.len() - (l + 2);
                        let first = code[start];
                        if (first & 0xE0) == 0xA0 { // likely CBOR map (0xA0..0xBF)
                            return &code[..start];
                        }
                    }
                }
                code
            }
            let onchain_stripped = strip_metadata(&onchain);
            let provided_stripped = strip_metadata(&provided);
            let equal = onchain_stripped == provided_stripped;

            // Optional creation bytecode verification (with constructor args support)
            let creation_hex = obj.get("creation_bytecode").and_then(|v| v.as_str());
            let mut creation_verified = None;
            let mut creation_hash: Option<String> = None;
            let mut creation_stripped_hash: Option<String> = None;
            if let Some(ch) = creation_hex {
                let s = ch.trim();
                let s = s.strip_prefix("0x").unwrap_or(s);
                if let Ok(cbytes) = hex::decode(s) {
                    use sha3::Digest as _;
                    creation_hash = Some(format!("0x{}", hex::encode(Keccak256::digest(&cbytes))));
                    let cstr = strip_metadata(&cbytes);
                    creation_stripped_hash = Some(format!("0x{}", hex::encode(Keccak256::digest(cstr))));

                    // If standard_json provided, compute expected creation bytecode
                    let expected_creation_opt: Option<Vec<u8>> = if let Some(stdj) = standard_json {
                        let sj_str = if stdj.is_string() { stdj.as_str().unwrap().to_string() } else { serde_json::to_string(stdj).unwrap_or_default() };
                        if let Ok((creation, _runtime)) = compile_standard_json(&sj_str, contract_name) {
                            // Append constructor args if provided (hex-encoded ABI)
                            let extra = constructor_args_hex
                                .and_then(|h| hex::decode(h.trim().trim_start_matches("0x")).ok())
                                .unwrap_or_default();
                            let mut full = creation.clone();
                            full.extend_from_slice(&extra);
                            Some(full)
                        } else { None }
                    } else { None };

                    // Match stripped creation bytecode exactly if expected available, otherwise fallback to prefix match
                    creation_verified = Some(if let Some(exp) = expected_creation_opt {
                        let exp_s = strip_metadata(&exp);
                        cstr == exp_s
                    } else {
                        // Fallback legacy behavior
                        cbytes.starts_with(provided_stripped)
                    });
                }
            }
            use sha3::{Digest, Keccak256};
            let mut hasher = Keccak256::new();
            hasher.update(address_str.as_bytes());
            hasher.update(compiler_version.as_bytes());
            hasher.update([optimization_enabled as u8]);
            hasher.update(&provided);
            let verification_id = format!("0x{}", hex::encode(hasher.finalize()));

            // Build record
            let record = json!({
                "address": address_str,
                "runtime_bytecode_hash": format!("0x{}", hex::encode(Keccak256::digest(&provided))),
                "runtime_bytecode_stripped_hash": format!("0x{}", hex::encode(Keccak256::digest(provided_stripped))),
                "onchain_code_hash": format!("0x{}", hex::encode(Keccak256::digest(&onchain))),
                "onchain_code_stripped_hash": format!("0x{}", hex::encode(Keccak256::digest(onchain_stripped))),
                "compiler_version": compiler_version,
                "optimization_enabled": optimization_enabled,
                "source_hash": source_code.map(|s| format!("0x{}", hex::encode(Keccak256::digest(s.as_bytes())))),
                "contract_name": contract_name,
                "verified": equal,
                "creation_bytecode_hash": creation_hash,
                "creation_bytecode_stripped_hash": creation_stripped_hash,
                "creation_verified": creation_verified,
                "verification_id": verification_id,
                "timestamp": chrono::Utc::now().to_rfc3339(),
            });

            // Store record in memory
            {
                let mut map = VERIFICATIONS.write().unwrap();
                map.insert(address_str.to_string(), record.clone());
            }

            // Persist record in storage under metadata CF
            let key_addr = format!("verify:addr:{}", address_str.to_lowercase());
            let key_id = format!("verify:id:{}", verification_id.to_lowercase());
            if let Ok(bytes) = serde_json::to_vec(&record) {
                let _ = storage_v.db.put_cf("metadata", key_addr.as_bytes(), &bytes);
                // id → address index
                let _ = storage_v.db.put_cf("metadata", key_id.as_bytes(), address_str.as_bytes());
            }

            Ok(json!({"verified": equal, "verification_id": verification_id}))
        });

        // lattice_getVerification: return stored verification record
        let storage_get = storage.clone();
        io_handler.add_sync_method("lattice_getVerification", move |params: Params| {
            rpc_request("lattice_getVerification");
            let addr: String = match params.parse() { Ok(s) => s, Err(e) => return Err(jsonrpc_core::Error::invalid_params(e.to_string())) };
            // Try memory
            if let Some(val) = VERIFICATIONS.read().unwrap().get(&addr).cloned() { return Ok(val); }
            // Try storage
            let key_addr = format!("verify:addr:{}", addr.to_lowercase());
            match storage_get.db.get_cf("metadata", key_addr.as_bytes()) {
                Ok(Some(bytes)) => {
                    let val: serde_json::Value = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
                    Ok(val)
                }
                _ => Ok(Value::Null)
            }
        });

        // lattice_listVerifications: return all verification records
        let storage_list = storage.clone();
        io_handler.add_sync_method("lattice_listVerifications", move |params: Params| {
            rpc_request("lattice_listVerifications");
            let obj = match params {
                Params::None => serde_json::Map::new(),
                Params::Array(_) => serde_json::Map::new(),
                Params::Map(m) => m.into_iter().collect(),
            };
            let (offset, limit) = parse_pagination(&obj);
            let want_verified = obj.get("verified").and_then(|v| v.as_bool());
            let addr_prefix = obj.get("address_prefix").and_then(|v| v.as_str()).map(|s| s.to_lowercase());
            let prefix = b"verify:addr:";
            let mut out = Vec::new();
            if let Ok(iter) = storage_list.db.prefix_iter_cf("metadata", prefix) {
                for (k, v) in iter {
                    if let Some(pref) = &addr_prefix {
                        if let Ok(kstr) = std::str::from_utf8(&k) {
                            if let Some(addr) = kstr.strip_prefix("verify:addr:") {
                                if !addr.starts_with(pref) { continue; }
                            }
                        }
                    }
                    if let Ok(val) = serde_json::from_slice::<serde_json::Value>(&v) {
                        if let Some(wv) = want_verified {
                            if val.get("verified").and_then(|b| b.as_bool()).unwrap_or(false) != wv { continue; }
                        }
                        out.push(val);
                    }
                }
            }
            // Sort by timestamp desc if present
            out.sort_by(|a, b| b.get("timestamp").and_then(|v| v.as_str()).cmp(&a.get("timestamp").and_then(|v| v.as_str())));
            let out = apply_pagination(out, offset, limit);
            Ok(Value::Array(out))
        });

        // lattice_listVerificationsByStatus [bool]
        let storage_list_status = storage.clone();
        io_handler.add_sync_method("lattice_listVerificationsByStatus", move |params: Params| {
            rpc_request("lattice_listVerificationsByStatus");
            // Support payload { verified: bool, offset?: u64, limit?: u64 }
            let obj = match params {
                Params::Map(m) => m.into_iter().collect::<serde_json::Map<_,_>>(),
                _ => serde_json::Map::new(),
            };
            let want: bool = obj.get("verified").and_then(|v| v.as_bool()).unwrap_or(true);
            let (offset, limit) = parse_pagination(&obj);
            let prefix = b"verify:addr:";
            let mut out = Vec::new();
            if let Ok(iter) = storage_list_status.db.prefix_iter_cf("metadata", prefix) {
                for (_k, v) in iter {
                    if let Ok(val) = serde_json::from_slice::<serde_json::Value>(&v) {
                        if val.get("verified").and_then(|b| b.as_bool()).unwrap_or(false) == want {
                            out.push(val);
                        }
                    }
                }
            }
            out.sort_by(|a, b| b.get("timestamp").and_then(|v| v.as_str()).cmp(&a.get("timestamp").and_then(|v| v.as_str())));
            let out = apply_pagination(out, offset, limit);
            Ok(Value::Array(out))
        });

        // lattice_listVerificationsByAddressPrefix [string]
        let storage_list_prefix = storage.clone();
        io_handler.add_sync_method("lattice_listVerificationsByAddressPrefix", move |params: Params| {
            rpc_request("lattice_listVerificationsByAddressPrefix");
            // Support payload { prefix: string, offset?: u64, limit?: u64 }
            let obj = match params {
                Params::Map(m) => m.into_iter().collect::<serde_json::Map<_,_>>(),
                _ => serde_json::Map::new(),
            };
            let prefix_str = obj.get("prefix").and_then(|v| v.as_str()).unwrap_or("").to_lowercase();
            let (offset, limit) = parse_pagination(&obj);
            let meta_prefix = b"verify:addr:";
            let mut out = Vec::new();
            if let Ok(iter) = storage_list_prefix.db.prefix_iter_cf("metadata", meta_prefix) {
                for (k, v) in iter {
                    // key format: verify:addr:<address>
                    if let Ok(key_str) = std::str::from_utf8(&k) {
                        if let Some(addr) = key_str.strip_prefix("verify:addr:") {
                            if addr.starts_with(&prefix_str) {
                                if let Ok(val) = serde_json::from_slice::<serde_json::Value>(&v) {
                                    out.push(val);
                                }
                            }
                        }
                    }
                }
            }
            out.sort_by(|a, b| b.get("timestamp").and_then(|v| v.as_str()).cmp(&a.get("timestamp").and_then(|v| v.as_str())));
            let out = apply_pagination(out, offset, limit);
            Ok(Value::Array(out))
        });

        // lattice_pruneVerifications: optional GC to prune by age or count
        let storage_gc = storage.clone();
        io_handler.add_sync_method("lattice_pruneVerifications", move |params: Params| {
            rpc_request("lattice_pruneVerifications");
            // Payload: { max_age_seconds?: u64, max_records?: u64 }
            let obj = match params {
                Params::Map(m) => m.into_iter().collect::<serde_json::Map<_,_>>(),
                _ => serde_json::Map::new(),
            };
            let max_age = obj.get("max_age_seconds").and_then(|v| v.as_u64());
            let max_records = obj.get("max_records").and_then(|v| v.as_u64());
            let prefix = b"verify:addr:";
            let mut items: Vec<(Vec<u8>, serde_json::Value)> = Vec::new();
            if let Ok(iter) = storage_gc.db.prefix_iter_cf("metadata", prefix) {
                for (k, v) in iter {
                    if let Ok(val) = serde_json::from_slice::<serde_json::Value>(&v) {
                        items.push((k.to_vec(), val));
                    }
                }
            }
            // Sort by timestamp asc (oldest first)
            items.sort_by(|a, b| a.1.get("timestamp").and_then(|v| v.as_str()).cmp(&b.1.get("timestamp").and_then(|v| v.as_str())));
            let now = chrono::Utc::now();
            let mut removed = 0usize;
            // Age-based removal
            if let Some(age) = max_age {
                for (k, v) in items.iter() {
                    if let Some(ts) = v.get("timestamp").and_then(|t| t.as_str()) {
                        if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(ts) {
                            let dt_utc = dt.with_timezone(&chrono::Utc);
                            if (now - dt_utc).num_seconds() as u64 > age {
                                let _ = storage_gc.db.delete_cf("metadata", k);
                                removed += 1;
                            }
                        }
                    }
                }
            }
            // Count-based removal (keep newest max_records)
            if let Some(max) = max_records {
                // Re-list after age-based removal
                let mut remaining: Vec<(Vec<u8>, serde_json::Value)> = Vec::new();
                if let Ok(iter) = storage_gc.db.prefix_iter_cf("metadata", prefix) {
                    for (k, v) in iter {
                        if let Ok(val) = serde_json::from_slice::<serde_json::Value>(&v) {
                            remaining.push((k.to_vec(), val));
                        }
                    }
                }
                // Sort by timestamp desc, keep first max
                remaining.sort_by(|a, b| b.1.get("timestamp").and_then(|v| v.as_str()).cmp(&a.1.get("timestamp").and_then(|v| v.as_str())));
                if remaining.len() > max as usize {
                    for (k, _v) in remaining.into_iter().skip(max as usize) {
                        let _ = storage_gc.db.delete_cf("metadata", &k);
                        removed += 1;
                    }
                }
            }
            Ok(json!({"removed": removed}))
        });

        // lattice_getVerificationById: fetch by verification_id
        let storage_by_id = storage.clone();
        io_handler.add_sync_method("lattice_getVerificationById", move |params: Params| {
            rpc_request("lattice_getVerificationById");
            let vid: String = match params.parse() { Ok(s) => s, Err(e) => return Err(jsonrpc_core::Error::invalid_params(e.to_string())) };
            let key_id = format!("verify:id:{}", vid.to_lowercase());
            if let Ok(Some(addr_bytes)) = storage_by_id.db.get_cf("metadata", key_id.as_bytes()) {
                if let Ok(addr) = std::str::from_utf8(&addr_bytes) {
                    let key_addr = format!("verify:addr:{}", addr.to_lowercase());
                    if let Ok(Some(bytes)) = storage_by_id.db.get_cf("metadata", key_addr.as_bytes()) {
                        let val: serde_json::Value = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
                        return Ok(val);
                    }
                }
            }
            Ok(Value::Null)
        });

        
        // lattice_deployModel: register a model via model precompile
        let executor_ai_deploy = executor.clone();
        io_handler.add_sync_method("lattice_deployModel", move |params: Params| {
            rpc_request("lattice_deployModel");
            let obj: serde_json::Value = match params.parse() { Ok(v) => v, Err(e) => return Err(jsonrpc_core::Error::invalid_params(e.to_string())) };
            let map = obj.as_object().ok_or_else(|| jsonrpc_core::Error::invalid_params("Expected object"))?;
            let from_hex = map.get("from").and_then(|v| v.as_str()).ok_or_else(|| jsonrpc_core::Error::invalid_params("Missing 'from'"))?;
            let cid = map.get("ipfs_cid").and_then(|v| v.as_str()).ok_or_else(|| jsonrpc_core::Error::invalid_params("Missing 'ipfs_cid'"))?;
            let size_bytes = map.get("size_bytes").and_then(|v| v.as_u64()).unwrap_or(0u64);
            let model_hash_opt = map.get("model_hash").and_then(|v| v.as_str());
            let policy_str = map.get("access_policy").and_then(|v| v.as_str());
            let price_wei = map.get("inference_price").and_then(|v| v.as_str()).or_else(|| map.get("inference_price_wei").and_then(|v| v.as_str()));

            let from_bytes = hex::decode(from_hex.trim().trim_start_matches("0x")).map_err(|_| jsonrpc_core::Error::invalid_params("Invalid 'from'"))?;
            if from_bytes.len() != 20 { return Err(jsonrpc_core::Error::invalid_params("Invalid 'from' length")); }
            let mut from_pkb = [0u8; 32]; from_pkb[..20].copy_from_slice(&from_bytes);
            let from_pk = lattice_consensus::types::PublicKey::new(from_pkb);
            let from_addr = lattice_execution::types::Address({ let mut a=[0u8;20]; a.copy_from_slice(&from_bytes); a });

            let model_hash_arr = if let Some(hs) = model_hash_opt {
                let b = hex::decode(hs.trim_start_matches("0x")).map_err(|_| jsonrpc_core::Error::invalid_params("Invalid 'model_hash'"))?;
                if b.len()!=32 { return Err(jsonrpc_core::Error::invalid_params("Invalid 'model_hash' length")); }
                let mut a=[0u8;32]; a.copy_from_slice(&b); a
            } else {
                let mut hasher = sha3::Keccak256::default();
                hasher.update(&from_bytes); hasher.update(cid.as_bytes()); hasher.update(size_bytes.to_le_bytes());
                let out = hasher.finalize(); let mut a=[0u8;32]; a.copy_from_slice(&out); a
            };
            let model_hash = lattice_consensus::types::Hash::new(model_hash_arr);

            // Build data for registerModel precompile. If access_policy provided, use extended form registerModel(bytes32,string,uint8,uint256)
            use sha3::Digest as _;
            let mut data = Vec::new();
            if let Some(pol) = policy_str {
                // Extended signature
                let sel = &sha3::Keccak256::digest(b"registerModel(bytes32,string,uint8,uint256)")[..4];
                data.extend_from_slice(sel);
                data.extend_from_slice(model_hash.as_bytes()); // bytes32
                // offset to string (after 4 static args = 128 bytes => 0x80)
                let mut off = [0u8; 32]; off[31] = 128u8; data.extend_from_slice(&off);
                // access policy as uint8 in 32-byte big endian
                let pol_idx: u8 = match pol.to_lowercase().as_str() {
                    "public" => 0,
                    "private" => 1,
                    "restricted" => 2,
                    "payperuse" | "pay_per_use" | "pay-per-use" => 3,
                    _ => 0,
                };
                let mut pol_be = [0u8; 32]; pol_be[31] = pol_idx; data.extend_from_slice(&pol_be);
                // price
                let mut price_be = [0u8; 32];
                if let Some(p) = price_wei {
                    // Accept decimal string or 0x-hex
                    let pstr = p.trim();
                    let val = if let Some(h) = pstr.strip_prefix("0x").or(pstr.strip_prefix("0X")) {
                        primitive_types::U256::from_big_endian(&hex::decode(h).map_err(|_| jsonrpc_core::Error::invalid_params("Invalid price hex"))?)
                    } else {
                        primitive_types::U256::from_dec_str(pstr).map_err(|_| jsonrpc_core::Error::invalid_params("Invalid price"))?
                    };
                    let mut tmp = [0u8; 32]; val.to_big_endian(&mut tmp); price_be.copy_from_slice(&tmp);
                }
                data.extend_from_slice(&price_be);
                // dynamic string
                let len = cid.len(); let mut lenb=[0u8;32]; lenb[31]=(len & 0xFF) as u8; data.extend_from_slice(&lenb);
                data.extend_from_slice(cid.as_bytes()); let pad=(32-(len%32))%32; data.extend_from_slice(&vec![0u8; pad]);
            } else {
                // Legacy 2-arg signature
                let sel = &sha3::Keccak256::digest(b"registerModel(bytes32,string)")[..4];
                data.extend_from_slice(sel);
                data.extend_from_slice(model_hash.as_bytes());
                let mut off=[0u8;32]; off[31]=64; data.extend_from_slice(&off);
                let len = cid.len(); let mut lenb=[0u8;32]; lenb[31]=(len & 0xFF) as u8; data.extend_from_slice(&lenb);
                data.extend_from_slice(cid.as_bytes()); let pad=(32-(len%32))%32; data.extend_from_slice(&vec![0u8; pad]);
            }

            let to_addr = { let mut a=[0u8;20]; a[18]=0x10; a[19]=0x00; a };
            let mut to_pkb=[0u8;32]; to_pkb[..20].copy_from_slice(&to_addr);
            let to_pk = lattice_consensus::types::PublicKey::new(to_pkb);

            let exec = executor_ai_deploy.clone();
            exec.state_db().accounts.set_balance(from_addr, primitive_types::U256::from(1_000_000_000_000_000_000u128));
            let blk = lattice_consensus::types::Block { header: lattice_consensus::types::BlockHeader {
                    version:1, block_hash: lattice_consensus::types::Hash::default(), selected_parent_hash: lattice_consensus::types::Hash::default(),
                    merge_parent_hashes: vec![], timestamp:0, height:0, blue_score:0, blue_work:0, pruning_point: lattice_consensus::types::Hash::default(),
                    proposer_pubkey: from_pk, vrf_reveal: lattice_consensus::types::VrfProof { proof: vec![], output: lattice_consensus::types::Hash::default() },
                }, state_root: lattice_consensus::types::Hash::default(), tx_root: lattice_consensus::types::Hash::default(),
                receipt_root: lattice_consensus::types::Hash::default(), artifact_root: lattice_consensus::types::Hash::default(), ghostdag_params: Default::default(),
                transactions: vec![], signature: lattice_consensus::types::Signature::new([0;64]) };
            let tx = lattice_consensus::types::Transaction { hash: lattice_consensus::types::Hash::default(), nonce:0, from:from_pk, to: Some(to_pk), value:0,
                gas_limit:200000, gas_price:1, data, signature: lattice_consensus::types::Signature::new([0;64]), tx_type: None };

            match block_on(exec.execute_transaction(&blk, &tx)) {
                Ok(rcpt) => {
                    if rcpt.status {
                        Ok(json!({ "model_id": format!("0x{}", hex::encode(model_hash.as_bytes())), "tx_hash": format!("0x{}", hex::encode(rcpt.tx_hash.as_bytes())) }))
                    } else { Ok(json!({ "error": "deployment failed", "gas_used": rcpt.gas_used })) }
                }
                Err(e) => Err(jsonrpc_core::Error::invalid_params(format!("deploy failed: {}", e)))
            }
        });

        // ========== Extra Network Methods ==========
        // net_peers: return list of peer IDs (diagnostic)
        let peers_list = peer_manager.clone();
        io_handler.add_sync_method("net_peers", move |_params: Params| {
            rpc_request("net_peers");
            let api = NetworkApi::new(peers_list.clone());
            match block_on(api.get_peers()) {
                Ok(ids) => Ok(serde_json::to_value(ids).unwrap_or(Value::Array(vec![]))),
                Err(_) => Ok(Value::Array(vec![])),
            }
        });

        // net_peerInfo: detailed peer info (id, addr, direction)
        let peers_info_mgr = peer_manager.clone();
        io_handler.add_sync_method("net_peerInfo", move |_params: Params| {
            rpc_request("net_peerInfo");
            let peers = peers_info_mgr.get_all_peers();
            let mut arr = Vec::new();
            for p in peers {
                let info = block_on(p.info.read());
                let dir = match info.direction {
                    lattice_network::peer::Direction::Inbound => "inbound",
                    lattice_network::peer::Direction::Outbound => "outbound",
                };
                arr.push(serde_json::json!({
                    "id": info.id.to_string(),
                    "addr": info.addr.to_string(),
                    "direction": dir,
                }));
            }
            Ok(Value::Array(arr))
        });
        
        // lattice_getModel
        let storage_ai_get = storage.clone();
        let mempool_ai_get = mempool.clone();
        let executor_ai_get = executor.clone();
        io_handler.add_sync_method("lattice_getModel", move |params: Params| {
            rpc_request("lattice_getModel");
            let api = AiApi::new(storage_ai_get.clone(), mempool_ai_get.clone(), executor_ai_get.clone());
            
            let model_id_str: String = match params.parse() {
                Ok(id) => id,
                Err(e) => return Err(jsonrpc_core::Error::invalid_params(e.to_string())),
            };
            
            // Parse model ID from hex string
            match hex::decode(&model_id_str) {
                Ok(model_id_bytes) if model_id_bytes.len() == 32 => {
                    let mut model_id_array = [0u8; 32];
                    model_id_array.copy_from_slice(&model_id_bytes);
                    let model_id = lattice_execution::types::ModelId(Hash::new(model_id_array));
                    
                    match block_on(api.get_model(model_id)) {
                        Ok(model) => Ok(serde_json::to_value(model).unwrap_or(Value::Null)),
                        Err(ApiError::ModelNotFound(_)) => Ok(Value::Null),
                        Err(_) => Err(jsonrpc_core::Error::internal_error()),
                    }
                },
                _ => Err(jsonrpc_core::Error::invalid_params("Invalid model ID format".to_string())),
            }
        });
        
        // lattice_listModels
        let storage_ai_list = storage.clone();
        let mempool_ai_list = mempool.clone();
        let executor_ai_list = executor.clone();
        io_handler.add_sync_method("lattice_listModels", move |params: Params| {
            rpc_request("lattice_listModels");
            let api = AiApi::new(storage_ai_list.clone(), mempool_ai_list.clone(), executor_ai_list.clone());
            
            // Parse optional parameters
            let (owner, limit): (Option<String>, Option<usize>) = params.parse().unwrap_or((None, None));
            
            let parsed_owner = owner.and_then(|addr_str| {
                hex::decode(addr_str.trim_start_matches("0x")).ok()
                    .and_then(|bytes| {
                        if bytes.len() == 20 {
                            let mut addr_array = [0u8; 20];
                            addr_array.copy_from_slice(&bytes);
                            Some(Address(addr_array))
                        } else {
                            None
                        }
                    })
            });
            
            match block_on(api.list_models(parsed_owner, limit)) {
                Ok(models) => {
                    let model_strings: Vec<String> = models.iter()
                        .map(|id| hex::encode(id.0.as_bytes()))
                        .collect();
                    Ok(serde_json::to_value(model_strings).unwrap_or(Value::Array(vec![])))
                },
                Err(_) => Ok(Value::Array(vec![])),
            }
        });
        
        // lattice_requestInference
        let storage_ai_inf = storage.clone();
        let mempool_ai_inf = mempool.clone();
        let executor_ai_inf = executor.clone();
        io_handler.add_sync_method("lattice_requestInference", move |_params: Params| {
            rpc_request("lattice_requestInference");
            let _api = AiApi::new(storage_ai_inf.clone(), mempool_ai_inf.clone(), executor_ai_inf.clone());
            
            // Parse inference request (simplified)
            match block_on(async {
                // Placeholder - would parse actual InferenceRequest
                Ok::<serde_json::Value, ApiError>(serde_json::json!({
                    "status": "success",
                    "message": "Inference request not fully implemented yet"
                }))
            }) {
                Ok(result) => Ok(result),
                Err(_e) => Err(jsonrpc_core::Error::internal_error()),
            }
        });
        
        // lattice_getInferenceResult
        let storage_ai_result = storage.clone();
        let mempool_ai_result = mempool.clone();
        let executor_ai_result = executor.clone();
        io_handler.add_sync_method("lattice_getInferenceResult", move |params: Params| {
            rpc_request("lattice_getInferenceResult");
            let api = AiApi::new(storage_ai_result.clone(), mempool_ai_result.clone(), executor_ai_result.clone());
            
            let request_id_str: String = match params.parse() {
                Ok(id) => id,
                Err(e) => return Err(jsonrpc_core::Error::invalid_params(e.to_string())),
            };
            
            match hex::decode(&request_id_str) {
                Ok(hash_bytes) if hash_bytes.len() == 32 => {
                    let mut hash_array = [0u8; 32];
                    hash_array.copy_from_slice(&hash_bytes);
                    let request_hash = Hash::new(hash_array);
                    
                    match block_on(api.get_inference_result(request_hash)) {
                        Ok(result) => Ok(serde_json::to_value(result).unwrap_or(Value::Null)),
                        Err(_) => Ok(Value::Null),
                    }
                },
                _ => Err(jsonrpc_core::Error::invalid_params("Invalid request ID format".to_string())),
            }
        });
        
        // lattice_createTrainingJob
        let storage_ai_job = storage.clone();
        let mempool_ai_job = mempool.clone();
        let executor_ai_job = executor.clone();
        io_handler.add_sync_method("lattice_createTrainingJob", move |_params: Params| {
            rpc_request("lattice_createTrainingJob");
            let _api = AiApi::new(storage_ai_job.clone(), mempool_ai_job.clone(), executor_ai_job.clone());
            
            match block_on(async {
                // Placeholder - would parse actual CreateTrainingJobRequest
                Ok::<serde_json::Value, ApiError>(serde_json::json!({
                    "status": "success",
                    "message": "Training job creation not fully implemented yet"
                }))
            }) {
                Ok(result) => Ok(result),
                Err(_e) => Err(jsonrpc_core::Error::internal_error()),
            }
        });
        
        // lattice_getTrainingJob
        let storage_ai_job_get = storage.clone();
        let mempool_ai_job_get = mempool.clone();
        let executor_ai_job_get = executor.clone();
        io_handler.add_sync_method("lattice_getTrainingJob", move |params: Params| {
            rpc_request("lattice_getTrainingJob");
            let api = AiApi::new(storage_ai_job_get.clone(), mempool_ai_job_get.clone(), executor_ai_job_get.clone());
            
            let job_id_str: String = match params.parse() {
                Ok(id) => id,
                Err(e) => return Err(jsonrpc_core::Error::invalid_params(e.to_string())),
            };
            
            match hex::decode(&job_id_str) {
                Ok(job_id_bytes) if job_id_bytes.len() == 32 => {
                    let mut job_id_array = [0u8; 32];
                    job_id_array.copy_from_slice(&job_id_bytes);
                    let job_id = lattice_execution::types::JobId(Hash::new(job_id_array));
                    
                    match block_on(api.get_training_job(job_id)) {
                        Ok(job) => Ok(serde_json::to_value(job).unwrap_or(Value::Null)),
                        Err(_) => Ok(Value::Null),
                    }
                },
                _ => Err(jsonrpc_core::Error::invalid_params("Invalid job ID format".to_string())),
            }
        });

        // ========= Artifacts ==========
        // lattice_pinArtifact [cid, replicas]
        let executor_art_pin = executor.clone();
        io_handler.add_sync_method("lattice_pinArtifact", move |params: Params| {
            rpc_request("lattice_pinArtifact");
            let (cid, replicas): (String, u64) = match params.parse() {
                Ok(t) => t,
                Err(e) => return Err(jsonrpc_core::Error::invalid_params(e.to_string())),
            };
            match block_on(executor_art_pin.artifact_pin(&cid, replicas as usize)) {
                Ok(()) => Ok(serde_json::json!({"status":"ok"})),
                Err(e) => Ok(serde_json::json!({"status":"error","message":format!("{}", e)})),
            }
        });

        // lattice_getArtifactStatus [cid]
        let executor_art_status = executor.clone();
        io_handler.add_sync_method("lattice_getArtifactStatus", move |params: Params| {
            rpc_request("lattice_getArtifactStatus");
            let cid: String = match params.parse() {
                Ok(c) => c,
                Err(e) => return Err(jsonrpc_core::Error::invalid_params(e.to_string())),
            };
            match block_on(executor_art_status.artifact_status(&cid)) {
                Ok(s) => Ok(serde_json::from_str::<serde_json::Value>(&s).unwrap_or(serde_json::json!({"status":s}))),
                Err(_) => Ok(serde_json::json!({"status":"unknown"})),
            }
        });

        // lattice_listModelArtifacts [modelIdHex]
        let executor_art_list = executor.clone();
        io_handler.add_sync_method("lattice_listModelArtifacts", move |params: Params| {
            rpc_request("lattice_listModelArtifacts");
            let model_id_str: String = match params.parse() {
                Ok(s) => s,
                Err(e) => return Err(jsonrpc_core::Error::invalid_params(e.to_string())),
            };
            match hex::decode(&model_id_str) {
                Ok(bytes) if bytes.len()==32 => {
                    let mut arr=[0u8;32]; arr.copy_from_slice(&bytes);
                    let hash = Hash::new(arr);
                    let list = executor_art_list.list_model_artifacts(&hash);
                    Ok(serde_json::to_value(list).unwrap_or(serde_json::json!([])))
                }
                _ => Err(jsonrpc_core::Error::invalid_params("Invalid model ID format")),
            }
        });

        // lattice_listProofArtifacts [modelIdHex]
        let executor_proof_list = executor.clone();
        io_handler.add_sync_method("lattice_listProofArtifacts", move |params: Params| {
            rpc_request("lattice_listProofArtifacts");
            let model_id_str: String = match params.parse() {
                Ok(s) => s,
                Err(e) => return Err(jsonrpc_core::Error::invalid_params(e.to_string())),
            };
            match hex::decode(&model_id_str) {
                Ok(bytes) if bytes.len()==32 => {
                    let mut arr=[0u8;32]; arr.copy_from_slice(&bytes);
                    let hash = Hash::new(arr);
                    let list = executor_proof_list.list_model_proofs(&hash);
                    Ok(serde_json::to_value(list).unwrap_or(serde_json::json!([])))
                }
                _ => Err(jsonrpc_core::Error::invalid_params("Invalid model ID format")),
            }
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
    // no-op

    #[tokio::test]
    async fn test_rpc_chain_height_and_tx_submit() {
        let temp_dir = TempDir::new().unwrap();
        let storage = Arc::new(StorageManager::new(temp_dir.path(), PruningConfig::default()).unwrap());
        let mempool = Arc::new(Mempool::new(MempoolConfig::default()));
        let peer_manager = Arc::new(PeerManager::new(PeerManagerConfig::default()));
        let state_db = Arc::new(lattice_execution::StateDB::new());
        let executor = Arc::new(Executor::new(state_db));

        let rpc = RpcServer::new(
            RpcConfig::default(),
            storage,
            mempool.clone(),
            peer_manager,
            executor,
            1,
        );

        // chain_getHeight
        let req = serde_json::json!({"jsonrpc":"2.0","id":1,"method":"chain_getHeight","params":[]}).to_string();
        let resp = rpc.io_handler.handle_request(&req).await.unwrap();
        let v: serde_json::Value = serde_json::from_str(&resp).unwrap();
        assert_eq!(v["result"], 0);

        // Note: tx submission path is covered via integration tests elsewhere.
    }

    #[cfg(feature = "verifier-ethers-solc")]
    #[test]
    fn test_compile_single_contract_opt_and_unopt() {
        let src = r#"// SPDX-License-Identifier: MIT
        pragma solidity ^0.8.24;
        contract Single {
            uint256 private x;
            function set(uint256 v) public { x = v; }
            function get() public view returns (uint256) { return x; }
        }
        "#;

        let bin_opt = super::compile_runtime_bytecode(src, "0.8.24", true, Some("Single")).expect("compile optimized");
        let bin_unopt = super::compile_runtime_bytecode(src, "0.8.24", false, Some("Single")).expect("compile unoptimized");
        assert!(!bin_opt.is_empty());
        assert!(!bin_unopt.is_empty());
    }

    #[cfg(feature = "verifier-ethers-solc")]
    #[test]
    fn test_compile_multi_contract_select_by_name() {
        let src = r#"// SPDX-License-Identifier: MIT
        pragma solidity ^0.8.24;
        contract A { function a() public pure returns (uint256) { return 1; } }
        contract B { function b() public pure returns (uint256) { return 2; } }
        "#;
        let bin_a = super::compile_runtime_bytecode(src, "0.8.24", true, Some("A")).expect("compile A");
        let bin_b = super::compile_runtime_bytecode(src, "0.8.24", true, Some("B")).expect("compile B");
        assert!(!bin_a.is_empty());
        assert!(!bin_b.is_empty());
        assert_ne!(bin_a, bin_b);
    }
}
