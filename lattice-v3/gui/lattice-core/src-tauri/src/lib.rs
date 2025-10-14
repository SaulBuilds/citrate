use anyhow::Result;
use serde::Deserialize;
use std::sync::Arc;
use tauri::{Emitter, Manager, State};
use tokio::sync::RwLock;
use tokio::time::sleep;
use tracing::info;

mod block_producer;
mod dag;
mod models;
mod node;
mod rpc_client;
mod sync;
mod wallet;
// network_service integration is pending; module intentionally not included for now

use dag::{BlockDetails, DAGData, DAGManager, TipInfo};
use lattice_network::NetworkMessage;
use lattice_sequencer::mempool::TxClass;
use models::{
    InferenceRequest, InferenceResponse, JobStatus, ModelDeployment, ModelInfo, ModelManager,
    TrainingJob,
};
use node::TxActivity;
use node::TxOverview;
use node::{NodeConfig, NodeManager, NodeStatus};
use node::{PeerSummary, PendingTx};
use wallet::{Account, FirstTimeSetupResult, TransactionRequest, WalletManager};

// Application state
struct AppState {
    node_manager: Arc<NodeManager>,
    wallet_manager: Arc<WalletManager>,
    model_manager: Arc<ModelManager>,
    dag_manager: Arc<RwLock<Option<Arc<DAGManager>>>>,
    external_rpc: Arc<RwLock<Option<Arc<rpc_client::RpcClient>>>>,
}

// ===== Node Commands =====

#[tauri::command]
async fn start_node(state: State<'_, AppState>) -> Result<String, String> {
    info!("start_node command received");
    tracing::error!("DEBUG: start_node called"); // Add visible debug output

    match state.node_manager.start().await {
        Ok(_) => {
            // Auto-connect to bootnodes if networking is enabled
            let cfg = state.node_manager.get_config().await;
            if cfg.enable_network && !cfg.bootnodes.is_empty() {
                tauri::async_runtime::spawn({
                    let nm = state.node_manager.clone();
                    async move {
                        // small delay to allow services to init
                        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                        let _ = nm.connect_bootnodes_now().await;
                    }
                });
            }
            // Initialize DAG manager with node's storage and ghostdag
            if let (Some(storage), Some(ghostdag)) = (
                state.node_manager.get_storage().await,
                state.node_manager.get_ghostdag().await,
            ) {
                let dag_manager = Arc::new(DAGManager::new(storage.clone(), ghostdag.clone()));
                *state.dag_manager.write().await = Some(dag_manager.clone());
                info!("DAG manager initialized");

                // Start a task to periodically refresh DAG manager to pick up synced blocks
                let _dag_for_refresh = dag_manager.clone();
                let storage_for_refresh = storage.clone();
                let _ghostdag_for_refresh = ghostdag.clone();
                tokio::spawn(async move {
                    loop {
                        tokio::time::sleep(std::time::Duration::from_secs(5)).await;

                        // Reload blocks into DAG if new ones arrived
                        if let Ok(latest_height) = storage_for_refresh.blocks.get_latest_height() {
                            // Just trigger a refresh - the DAG manager will read from storage
                            // which now contains synced blocks
                            tracing::debug!("DAG refresh: latest height = {}", latest_height);
                        }
                    }
                });
            }

            info!("Node started successfully");
            tracing::error!("DEBUG: Node started OK"); // Debug output
            Ok("Node started successfully".to_string())
        }
        Err(e) => {
            tracing::error!("Failed to start node: {}", e);
            tracing::error!("DEBUG: Node start failed with error: {}", e); // Debug output
            Err(e.to_string())
        }
    }
}

#[tauri::command]
async fn stop_node(state: State<'_, AppState>) -> Result<String, String> {
    // Clear DAG manager when stopping node
    *state.dag_manager.write().await = None;

    state
        .node_manager
        .stop()
        .await
        .map(|_| "Node stopped successfully".to_string())
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_node_status(state: State<'_, AppState>) -> Result<NodeStatus, String> {
    state
        .node_manager
        .get_status()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_node_config(state: State<'_, AppState>) -> Result<NodeConfig, String> {
    Ok(state.node_manager.get_config().await)
}

#[tauri::command]
async fn update_node_config(
    state: State<'_, AppState>,
    config: NodeConfig,
) -> Result<String, String> {
    state
        .node_manager
        .update_config(config)
        .await
        .map(|_| "Config updated successfully".to_string())
        .map_err(|e| e.to_string())
}

// ===== Network/Bootnode Commands =====

#[tauri::command]
async fn get_bootnodes(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    Ok(state.node_manager.get_bootnodes().await)
}

#[tauri::command]
async fn add_bootnode(state: State<'_, AppState>, entry: String) -> Result<String, String> {
    state
        .node_manager
        .add_bootnode_entry(&entry)
        .await
        .map(|_| "Bootnode added".to_string())
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn remove_bootnode(state: State<'_, AppState>, entry: String) -> Result<String, String> {
    state
        .node_manager
        .remove_bootnode_entry(&entry)
        .await
        .map(|_| "Bootnode removed".to_string())
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn connect_bootnodes(state: State<'_, AppState>) -> Result<usize, String> {
    state
        .node_manager
        .connect_bootnodes_now()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn connect_peer(state: State<'_, AppState>, entry: String) -> Result<String, String> {
    state
        .node_manager
        .connect_peer(&entry)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn disconnect_peer(state: State<'_, AppState>, peer_id: String) -> Result<(), String> {
    state
        .node_manager
        .disconnect_peer(&peer_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_peers(state: State<'_, AppState>) -> Result<Vec<PeerSummary>, String> {
    Ok(state.node_manager.get_peers_summary().await)
}

// ===== Wallet Activity =====

#[tauri::command]
async fn get_account_activity(
    state: State<'_, AppState>,
    address: String,
    block_window: Option<u64>,
    limit: Option<usize>,
) -> Result<Vec<TxActivity>, String> {
    let bw = block_window.unwrap_or(256);
    let lim = limit.unwrap_or(100);
    state
        .node_manager
        .get_account_activity(&address, bw, lim)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_tx_overview(state: State<'_, AppState>) -> Result<TxOverview, String> {
    state
        .node_manager
        .get_tx_overview()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_mempool_pending(
    state: State<'_, AppState>,
    limit: Option<usize>,
) -> Result<Vec<PendingTx>, String> {
    state
        .node_manager
        .get_mempool_pending(limit.unwrap_or(50))
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_address_observed_balance(
    state: State<'_, AppState>,
    address: String,
    block_window: Option<u64>,
) -> Result<String, String> {
    state
        .node_manager
        .get_observed_balance(&address, block_window.unwrap_or(256))
        .await
        .map_err(|e| e.to_string())
}

#[derive(Debug, Clone, Deserialize)]
struct JoinTestnetArgs {
    chain_id: Option<u64>,
    data_dir: Option<String>,
    rpc_port: Option<u16>,
    ws_port: Option<u16>,
    p2p_port: Option<u16>,
    rest_port: Option<u16>,
    bootnodes: Option<Vec<String>>,
    clear_chain: Option<bool>,
    seed_from: Option<String>,
}

fn copy_dir_all(src: &std::path::Path, dst: &std::path::Path) -> std::io::Result<()> {
    if !dst.exists() {
        std::fs::create_dir_all(dst)?;
    }
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let to_path = dst.join(entry.file_name());
        if ty.is_dir() {
            copy_dir_all(&entry.path(), &to_path)?;
        } else if ty.is_file() {
            std::fs::copy(entry.path(), to_path)?;
        }
    }
    Ok(())
}

#[tauri::command]
async fn join_testnet(state: State<'_, AppState>, args: JoinTestnetArgs) -> Result<String, String> {
    // Stop node if running
    let _ = state.node_manager.stop().await.map_err(|e| e.to_string());

    // Load current config
    let mut cfg = state.node_manager.get_config().await;

    // Apply testnet defaults
    let chain_id = args.chain_id.unwrap_or(42069);
    cfg.network = "testnet".to_string();
    cfg.enable_network = true;
    cfg.discovery = true;
    cfg.max_peers = cfg.max_peers.max(200);
    cfg.rpc_port = args.rpc_port.unwrap_or(18545);
    cfg.ws_port = args.ws_port.unwrap_or(18546);
    cfg.p2p_port = args.p2p_port.unwrap_or(30304);
    cfg.rest_port = args.rest_port.unwrap_or(3001);
    cfg.mempool.chain_id = chain_id;
    cfg.mempool.require_valid_signature = true;
    cfg.mempool.min_gas_price = cfg.mempool.min_gas_price.max(1_000_000_000);
    cfg.consensus.block_time_seconds = cfg.consensus.block_time_seconds.max(5);
    if let Some(bn) = args.bootnodes {
        cfg.bootnodes = bn;
    }
    // Resolve a safe dataDir: avoid src-tauri (dev watcher) and relative paths
    let mut desired_data_dir = args
        .data_dir
        .clone()
        .unwrap_or_else(|| cfg.data_dir.clone());
    let lower = desired_data_dir.to_lowercase();
    let is_relative = std::path::Path::new(&desired_data_dir).is_relative();
    let under_src_tauri = lower.contains("src-tauri");
    if is_relative || under_src_tauri {
        // Use OS data dir: e.g., macOS ~/Library/Application Support/lattice-core/testnet
        if let Some(mut base) = dirs::data_dir() {
            base.push("lattice-core");
            base.push("testnet");
            desired_data_dir = base.to_string_lossy().to_string();
        } else {
            // Fallback to current working directory under gui-data/testnet
            desired_data_dir = "./gui-data/testnet".into();
        }
        info!("Relocating GUI dataDir to {}", desired_data_dir);
        cfg.data_dir = desired_data_dir.clone();
    } else {
        cfg.data_dir = desired_data_dir.clone();
    }

    // Validate and save config
    state
        .node_manager
        .update_config(cfg.clone())
        .await
        .map_err(|e| e.to_string())?;

    // Prepare chain dir
    let chain_dir = std::path::PathBuf::from(&cfg.data_dir).join("chain");
    let clear_chain = args.clear_chain.unwrap_or(true);
    if clear_chain && chain_dir.exists() {
        std::fs::remove_dir_all(&chain_dir).map_err(|e| e.to_string())?;
    }
    if let Some(seed) = args.seed_from.as_ref() {
        let seed_path = std::path::PathBuf::from(seed);
        if seed_path.exists() {
            std::fs::create_dir_all(&chain_dir).map_err(|e| e.to_string())?;
            copy_dir_all(&seed_path, &chain_dir).map_err(|e| e.to_string())?;
        }
    }

    // Start node and attempt to connect bootnodes
    state
        .node_manager
        .start()
        .await
        .map_err(|e| e.to_string())?;
    // Auto-connect after start if bootnodes present
    let cfg_after = state.node_manager.get_config().await;
    if cfg_after.enable_network && !cfg_after.bootnodes.is_empty() {
        let ok = state
            .node_manager
            .connect_bootnodes_now()
            .await
            .map_err(|e| e.to_string())?;
        info!("Auto-connected to {} bootnodes", ok);
    }

    Ok("Joined testnet and started node".to_string())
}

#[tauri::command]
async fn connect_to_external_testnet(
    state: State<'_, AppState>,
    rpc_url: String,
) -> Result<String, String> {
    info!("Connecting to external testnet at: {}", rpc_url);

    // Create RPC client
    let client = Arc::new(rpc_client::RpcClient::new(rpc_url.clone()));

    // Test connection and get chain ID
    let chain_id = client
        .get_chain_id()
        .await
        .map_err(|e| format!("Failed to connect to RPC: {}", e))?;

    let block_number = client
        .get_block_number()
        .await
        .map_err(|e| format!("Failed to get block number: {}", e))?;

    // Store the external RPC client
    *state.external_rpc.write().await = Some(client);

    // Stop embedded node if running
    let _ = state.node_manager.stop().await;

    info!(
        "Connected to external testnet - Chain ID: {}, Block: {}",
        chain_id, block_number
    );

    Ok(format!(
        "Connected to testnet - Chain ID: {}, Current Block: {}",
        chain_id, block_number
    ))
}

#[tauri::command]
async fn disconnect_external_rpc(state: State<'_, AppState>) -> Result<String, String> {
    *state.external_rpc.write().await = None;
    Ok("Disconnected from external RPC".to_string())
}

#[tauri::command]
async fn switch_to_testnet(state: State<'_, AppState>) -> Result<String, String> {
    info!("Switching GUI to testnet mode");

    // Stop current node if running
    let _ = state.node_manager.stop().await;

    // Get current config and configure for testnet
    let mut config = state.node_manager.get_config().await;
    config.configure_for_testnet();

    // Update the config
    state
        .node_manager
        .update_config(config.clone())
        .await
        .map_err(|e| e.to_string())?;

    // Start node with testnet configuration
    state
        .node_manager
        .start()
        .await
        .map_err(|e| e.to_string())?;

    info!(
        "GUI switched to testnet mode - Chain ID: {}, P2P: {}",
        config.mempool.chain_id, config.p2p_port
    );

    Ok(format!(
        "Connected to testnet - Chain ID: {}, P2P Port: {}",
        config.mempool.chain_id, config.p2p_port
    ))
}

#[tauri::command]
async fn ensure_connectivity(state: State<'_, AppState>) -> Result<String, String> {
    state
        .node_manager
        .ensure_testnet_connectivity()
        .await
        .map_err(|e| e.to_string())?;

    let peer_count = state.node_manager.get_peers_summary().await.len();
    Ok(format!("Connectivity check complete. Connected peers: {}", peer_count))
}

#[tauri::command]
async fn check_first_time_and_setup_if_needed(
    state: State<'_, AppState>,
) -> Result<Option<FirstTimeSetupResult>, String> {
    // Check if this is first time setup
    if state.wallet_manager.is_first_time_setup().await {
        info!("First-time user detected. Performing automatic setup...");

        // Use a default password for automatic setup - in production, this should be user-provided
        let default_password = "secure_default_password_2024";
        let setup_result = state
            .wallet_manager
            .perform_first_time_setup(default_password)
            .await
            .map_err(|e| e.to_string())?;

        // Automatically set the generated address as the reward address
        let reward_address = setup_result.primary_address.clone();
        state
            .node_manager
            .set_reward_address(reward_address.clone())
            .await;

        // Update the node config to persist the reward address
        let mut config = state.node_manager.get_config().await;
        config.reward_address = Some(reward_address);
        let _ = config.save();

        info!(
            "Automatic first-time setup completed. Reward address: {}",
            setup_result.primary_address
        );

        Ok(Some(setup_result))
    } else {
        // Not first time - check if we have a reward address set
        if let Some(primary_address) = state.wallet_manager.get_primary_reward_address().await {
            let current_reward = state.node_manager.get_reward_address().await;
            if current_reward != Some(primary_address.clone()) {
                info!("Setting primary wallet address as reward address: {}", primary_address);
                state.node_manager.set_reward_address(primary_address.clone()).await;

                let mut config = state.node_manager.get_config().await;
                config.reward_address = Some(primary_address);
                let _ = config.save();
            }
        }

        Ok(None)
    }
}

fn detect_local_ipv4() -> Option<String> {
    use std::net::{IpAddr, UdpSocket};
    if let Ok(s) = UdpSocket::bind("0.0.0.0:0") {
        if s.connect("1.1.1.1:80").is_ok() {
            if let Ok(addr) = s.local_addr() {
                if let IpAddr::V4(ipv4) = addr.ip() {
                    return Some(ipv4.to_string());
                }
            }
        }
    }
    None
}

#[tauri::command]
async fn auto_add_bootnodes(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    // Determine an IPv4 to suggest; fallback to 127.0.0.1
    let ip = detect_local_ipv4().unwrap_or_else(|| "127.0.0.1".to_string());
    let ports = [30303u16, 30304, 30305, 30306, 30307];
    let entries: Vec<String> = ports.iter().map(|p| format!("{}:{}", ip, p)).collect();

    // Stop node to modify bootnodes in config
    let _ = state.node_manager.stop().await.map_err(|e| e.to_string());

    // Add entries to config (dedup)
    let mut cfg = state.node_manager.get_config().await;
    for e in &entries {
        if !cfg.bootnodes.contains(e) {
            cfg.bootnodes.push(e.clone());
        }
    }
    state
        .node_manager
        .update_config(cfg.clone())
        .await
        .map_err(|e| e.to_string())?;

    // Start and connect
    state
        .node_manager
        .start()
        .await
        .map_err(|e| e.to_string())?;
    let _ = state.node_manager.connect_bootnodes_now().await;

    Ok(entries)
}

// Reward address controls
#[tauri::command]
async fn set_reward_address(state: State<'_, AppState>, address: String) -> Result<String, String> {
    // Set in node manager (starts producer if running), then persist to config
    state.node_manager.set_reward_address(address.clone()).await;
    let mut cfg = state.node_manager.get_config().await;
    if cfg.reward_address.as_deref() != Some(&address) {
        cfg.reward_address = Some(address.clone());
        // Best effort save; ignore if node running check blocks update (we already applied at runtime)
        let _ = state.node_manager.update_config(cfg).await;
    }
    Ok("Reward address set".into())
}

#[tauri::command]
async fn get_reward_address(state: State<'_, AppState>) -> Result<Option<String>, String> {
    Ok(state.node_manager.get_reward_address().await)
}

// ===== Wallet Commands =====

#[tauri::command]
async fn create_account(
    state: State<'_, AppState>,
    label: String,
    password: String,
) -> Result<Account, String> {
    state
        .wallet_manager
        .create_account(label, &password)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn create_account_extended(
    state: State<'_, AppState>,
    label: String,
    password: String,
) -> Result<(Account, String, String), String> {
    state
        .wallet_manager
        .create_account_with_credentials(label, &password)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn import_account(
    state: State<'_, AppState>,
    private_key: String,
    label: String,
    password: String,
) -> Result<Account, String> {
    state
        .wallet_manager
        .import_account(&private_key, label, &password)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn import_account_from_mnemonic(
    state: State<'_, AppState>,
    mnemonic: String,
    label: String,
    password: String,
) -> Result<Account, String> {
    state
        .wallet_manager
        .import_account_from_mnemonic(&mnemonic, label, &password)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_accounts(state: State<'_, AppState>) -> Result<Vec<Account>, String> {
    Ok(state.wallet_manager.get_accounts().await)
}

#[tauri::command]
async fn is_first_time_setup(state: State<'_, AppState>) -> Result<bool, String> {
    Ok(state.wallet_manager.is_first_time_setup().await)
}

#[tauri::command]
async fn perform_first_time_setup(
    state: State<'_, AppState>,
    password: String,
) -> Result<FirstTimeSetupResult, String> {
    let setup_result = state
        .wallet_manager
        .perform_first_time_setup(&password)
        .await
        .map_err(|e| e.to_string())?;

    // Automatically set the generated address as the reward address
    let reward_address = setup_result.primary_address.clone();
    state
        .node_manager
        .set_reward_address(reward_address.clone())
        .await;

    // Update the node config to persist the reward address
    let mut config = state.node_manager.get_config().await;
    config.reward_address = Some(reward_address);
    let _ = config.save();

    info!(
        "First-time setup completed. Reward address set to: {}",
        setup_result.primary_address
    );

    Ok(setup_result)
}

#[tauri::command]
async fn get_account(
    state: State<'_, AppState>,
    address: String,
) -> Result<Option<Account>, String> {
    Ok(state.wallet_manager.get_account(&address).await)
}

#[tauri::command]
async fn send_transaction(
    state: State<'_, AppState>,
    request: TransactionRequest,
    password: String,
) -> Result<String, String> {
    // Check if we're connected to external RPC
    let external_rpc = state.external_rpc.read().await;
    if let Some(rpc_client) = external_rpc.as_ref() {
        // Use external RPC to send transaction
        info!("Sending transaction via external RPC");

        // Create and sign transaction
        let tx = state
            .wallet_manager
            .create_signed_transaction(request.clone(), &password)
            .await
            .map_err(|e| e.to_string())?;

        // Serialize transaction to hex for eth_sendRawTransaction
        let tx_bytes = bincode::serialize(&tx)
            .map_err(|e| format!("Failed to serialize transaction: {}", e))?;
        let tx_hex = format!("0x{}", hex::encode(&tx_bytes));

        // Send via external RPC
        let tx_hash = rpc_client
            .send_raw_transaction(&tx_hex)
            .await
            .map_err(|e| format!("Failed to send transaction: {}", e))?;

        info!("Transaction sent via external RPC: {}", tx_hash);
        Ok(tx_hash)
    } else {
        // Use embedded node (original behavior)
        let tx = state
            .wallet_manager
            .create_signed_transaction(request.clone(), &password)
            .await
            .map_err(|e| e.to_string())?;
        let tx_hash_hex = hex::encode(tx.hash.as_bytes());

        // Add to local mempool
        if let Some(mempool) = state.node_manager.get_mempool().await {
            let _ = mempool
                .read()
                .await
                .add_transaction(tx.clone(), TxClass::Standard)
                .await;
        }
        // Broadcast to peers
        let _ = state
            .node_manager
            .broadcast_network(NetworkMessage::NewTransaction { transaction: tx })
            .await;
        Ok(tx_hash_hex)
    }
}

#[tauri::command]
async fn sign_message(
    state: State<'_, AppState>,
    message: String,
    address: String,
    password: String,
) -> Result<String, String> {
    state
        .wallet_manager
        .sign_message(message.as_bytes(), &address, &password)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn verify_signature(
    state: State<'_, AppState>,
    message: String,
    signature: String,
    address: String,
) -> Result<bool, String> {
    state
        .wallet_manager
        .verify_signature(message.as_bytes(), &signature, &address)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn export_private_key(
    state: State<'_, AppState>,
    address: String,
    password: String,
) -> Result<String, String> {
    state
        .wallet_manager
        .export_private_key(&address, &password)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn update_balance(
    state: State<'_, AppState>,
    address: String,
    balance: String,
) -> Result<(), String> {
    let balance_u128 = balance
        .parse::<u128>()
        .map_err(|e| format!("Invalid balance: {}", e))?;
    state
        .wallet_manager
        .update_balance(&address, balance_u128)
        .await
        .map_err(|e| e.to_string())
}

// ===== DAG Commands =====

#[tauri::command]
async fn get_dag_data(
    state: State<'_, AppState>,
    limit: usize,
    start_height: Option<u64>,
) -> Result<DAGData, String> {
    let dag_manager_opt = state.dag_manager.read().await;
    if let Some(dag_manager) = dag_manager_opt.as_ref() {
        dag_manager
            .get_dag_data(limit, start_height)
            .await
            .map_err(|e| e.to_string())
    } else {
        // Return empty data if node is not started
        Ok(DAGData {
            nodes: vec![],
            links: vec![],
            statistics: dag::DAGStatistics {
                total_blocks: 0,
                blue_blocks: 0,
                red_blocks: 0,
                current_tips: 0,
                average_blue_score: 0.0,
                max_height: 0,
            },
            tips: vec![],
        })
    }
}

#[tauri::command]
async fn get_block_details(
    state: State<'_, AppState>,
    hash: String,
) -> Result<BlockDetails, String> {
    let dag_manager_opt = state.dag_manager.read().await;
    if let Some(dag_manager) = dag_manager_opt.as_ref() {
        dag_manager
            .get_block_details(&hash)
            .await
            .map_err(|e| e.to_string())
    } else {
        Err("Node is not running. Please start the node first.".to_string())
    }
}

#[tauri::command]
async fn get_blue_set(
    state: State<'_, AppState>,
    block_hash: String,
) -> Result<Vec<String>, String> {
    let dag_manager_opt = state.dag_manager.read().await;
    if let Some(dag_manager) = dag_manager_opt.as_ref() {
        dag_manager
            .get_blue_set(&block_hash)
            .await
            .map_err(|e| e.to_string())
    } else {
        Err("Node is not running. Please start the node first.".to_string())
    }
}

#[tauri::command]
async fn get_current_tips(state: State<'_, AppState>) -> Result<Vec<TipInfo>, String> {
    let dag_manager_opt = state.dag_manager.read().await;
    if let Some(dag_manager) = dag_manager_opt.as_ref() {
        dag_manager
            .get_current_tips()
            .await
            .map_err(|e| e.to_string())
    } else {
        Ok(vec![])
    }
}

#[tauri::command]
async fn calculate_blue_score(
    state: State<'_, AppState>,
    block_hash: String,
) -> Result<u64, String> {
    let dag_manager_opt = state.dag_manager.read().await;
    if let Some(dag_manager) = dag_manager_opt.as_ref() {
        dag_manager
            .calculate_blue_score(&block_hash)
            .await
            .map_err(|e| e.to_string())
    } else {
        Err("Node is not running. Please start the node first.".to_string())
    }
}

#[tauri::command]
async fn get_block_path(
    state: State<'_, AppState>,
    block_hash: String,
) -> Result<Vec<String>, String> {
    let dag_manager_opt = state.dag_manager.read().await;
    if let Some(dag_manager) = dag_manager_opt.as_ref() {
        dag_manager
            .get_block_path(&block_hash)
            .await
            .map_err(|e| e.to_string())
    } else {
        Err("Node is not running. Please start the node first.".to_string())
    }
}

// ===== Model Commands =====

#[tauri::command]
async fn deploy_model(
    state: State<'_, AppState>,
    deployment: ModelDeployment,
) -> Result<String, String> {
    state
        .model_manager
        .deploy_model(deployment)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn run_inference(
    state: State<'_, AppState>,
    request: InferenceRequest,
) -> Result<InferenceResponse, String> {
    state
        .model_manager
        .request_inference(request)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn start_training(state: State<'_, AppState>, job: TrainingJob) -> Result<String, String> {
    state
        .model_manager
        .start_training(job)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_model_info(
    state: State<'_, AppState>,
    model_id: String,
) -> Result<Option<ModelInfo>, String> {
    state
        .model_manager
        .get_model(&model_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn list_models(state: State<'_, AppState>) -> Result<Vec<ModelInfo>, String> {
    state
        .model_manager
        .get_models()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_training_jobs(state: State<'_, AppState>) -> Result<Vec<TrainingJob>, String> {
    state
        .model_manager
        .get_training_jobs()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_job_status(
    state: State<'_, AppState>,
    job_id: String,
) -> Result<Option<JobStatus>, String> {
    state
        .model_manager
        .get_job_status(&job_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_deployments(state: State<'_, AppState>) -> Result<Vec<ModelDeployment>, String> {
    state
        .model_manager
        .get_deployments()
        .await
        .map_err(|e| e.to_string())
}

// Setup function to initialize node components after startup
async fn setup_node_components(app_handle: tauri::AppHandle) {
    info!("Setting up node components");

    // Get app state
    let state = app_handle.state::<AppState>();
    let mut use_reward_address: Option<String> = None;
    // Prefer environment override
    if let Ok(addr) = std::env::var("LATTICE_REWARD_ADDRESS") {
        use_reward_address = Some(addr);
    }
    // Otherwise prefer config reward address
    if use_reward_address.is_none() {
        let cfg = state.node_manager.get_config().await;
        if let Some(addr) = cfg.reward_address.clone() {
            use_reward_address = Some(addr);
        }
    }

    // Do not auto-create wallets; if one exists use it for rewards, otherwise require user setup
    let wallets = state.wallet_manager.get_accounts().await;
    if !wallets.is_empty() {
        let reward_address = wallets[0].address.clone();
        info!("Using existing wallet for rewards: {}", reward_address);
        if use_reward_address.is_none() {
            use_reward_address = Some(reward_address);
        }
    } else {
        info!("No local wallet found. Block production will be disabled until a reward address is configured.");
    }
    // Apply chosen reward address and persist in config
    if let Some(addr) = use_reward_address.clone() {
        state.node_manager.set_reward_address(addr.clone()).await;
        let mut cfg = state.node_manager.get_config().await;
        if cfg.reward_address.as_deref() != Some(&addr) {
            cfg.reward_address = Some(addr);
            let _ = state.node_manager.update_config(cfg).await; // safe here (node not running)
        }
    }

    info!("Node components setup complete");
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter("info,lattice_core=debug")
        .init();

    // Create managers
    let node_manager = Arc::new(NodeManager::new().expect("Failed to create node manager"));
    let wallet_manager = Arc::new(WalletManager::new().expect("Failed to create wallet manager"));
    // Attach wallet manager so producer can credit rewards
    {
        let nm = node_manager.clone();
        let wm = wallet_manager.clone();
        tauri::async_runtime::block_on(async move {
            nm.attach_wallet_manager(wm).await;
        });
    }
    let model_manager = Arc::new(ModelManager::new());

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(AppState {
            node_manager,
            wallet_manager,
            model_manager,
            dag_manager: Arc::new(RwLock::new(None)),
            external_rpc: Arc::new(RwLock::new(None)),
        })
        .invoke_handler(tauri::generate_handler![
            // Node commands
            start_node,
            stop_node,
            get_node_status,
            get_node_config,
            update_node_config,
            join_testnet,
            auto_add_bootnodes,
            connect_to_external_testnet,
            disconnect_external_rpc,
            switch_to_testnet,
            ensure_connectivity,
            check_first_time_and_setup_if_needed,
            // Network/Bootnode commands
            get_bootnodes,
            add_bootnode,
            remove_bootnode,
            connect_bootnodes,
            connect_peer,
            disconnect_peer,
            get_peers,
            // Wallet activity
            get_account_activity,
            get_tx_overview,
            get_mempool_pending,
            get_address_observed_balance,
            set_reward_address,
            get_reward_address,
            // Wallet commands
            create_account,
            create_account_extended,
            import_account,
            import_account_from_mnemonic,
            get_accounts,
            is_first_time_setup,
            perform_first_time_setup,
            get_account,
            send_transaction,
            sign_message,
            verify_signature,
            export_private_key,
            update_balance,
            // DAG commands
            get_dag_data,
            get_block_details,
            get_blue_set,
            get_current_tips,
            calculate_blue_score,
            get_block_path,
            // Model commands
            deploy_model,
            run_inference,
            start_training,
            get_model_info,
            list_models,
            get_training_jobs,
            get_job_status,
            get_deployments,
        ])
        .setup(|app| {
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                setup_node_components(app_handle).await;
            });
            // Periodic node status broadcaster
            let app_handle2 = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                loop {
                    // Emit node status periodically (1s)
                    let state = app_handle2.state::<AppState>();
                    if let Ok(status) = state.node_manager.get_status().await {
                        let _ = app_handle2.emit("node-status", status);
                    }
                    sleep(std::time::Duration::from_secs(1)).await;
                }
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
