use anyhow::Result;
use base64::Engine;
use serde::Deserialize;
use std::sync::Arc;
use tauri::{Emitter, Manager, State};
use tokio::sync::RwLock;
use tokio::time::sleep;
use tracing::{info, warn};

mod agent;
mod block_producer;
mod dag;
mod dev_mode;
mod huggingface;
mod ipfs;
mod models;
mod node;
mod rpc_client;
mod sync;
mod terminal;
mod wallet;
mod windows;
// network_service integration is pending; module intentionally not included for now

use agent::AgentState;
use dag::{BlockDetails, DAGData, DAGManager, TipInfo};
use citrate_network::NetworkMessage;
use citrate_sequencer::mempool::TxClass;
use models::{
    InferenceRequest, InferenceResponse, JobStatus, ModelDeployment, ModelInfo, ModelManager,
    TrainingJob,
};
use node::TxActivity;
use node::TxOverview;
use node::{NodeConfig, NodeManager, NodeStatus};
use node::{PeerSummary, PendingTx};
use wallet::{Account, FirstTimeSetupResult, TransactionRequest, WalletManager};
use windows::{WindowManager, WindowType, WindowState};
use terminal::{TerminalManager, TerminalConfig, TerminalInfo};
use ipfs::{IpfsManager, IpfsStatus, IpfsConfig, IpfsAddResult, IpfsContent};
use huggingface::{
    HuggingFaceManager, HFConfig, HFModelInfo, HFModelFile,
    ModelSearchParams, DownloadProgress, AuthState as HFAuthState, OAuthToken
};

// Re-export agent commands
use agent::commands::{
    agent_approve_tool, agent_clear_history, agent_create_session, agent_delete_session,
    agent_get_active_model, agent_get_config, agent_get_messages, agent_get_models_dir,
    agent_get_pending_tools, agent_get_session, agent_get_status, agent_is_ready,
    agent_list_sessions, agent_load_local_model, agent_reject_tool, agent_scan_local_models,
    agent_send_message, agent_set_api_key, agent_set_auto_mode, agent_update_config,
    // Multi-provider AI configuration commands
    get_ai_providers_config, get_ai_provider_keys, update_ai_providers_config,
    save_ai_providers_config, test_ai_provider_connection, pin_local_model_to_ipfs, delete_local_model,
    download_model_from_ipfs, set_preferred_provider_order, set_local_fallback,
    check_onboarding_status, complete_onboarding,
    // First-run and onboarding commands
    check_first_run, setup_bundled_model, get_onboarding_questions,
    process_onboarding_answer, skip_onboarding,
};

// Application state
struct AppState {
    node_manager: Arc<NodeManager>,
    wallet_manager: Arc<WalletManager>,
    model_manager: Arc<ModelManager>,
    dag_manager: Arc<RwLock<Option<Arc<DAGManager>>>>,
    external_rpc: Arc<RwLock<Option<Arc<rpc_client::RpcClient>>>>,
    window_manager: Arc<RwLock<WindowManager>>,
    terminal_manager: Arc<RwLock<TerminalManager>>,
    ipfs_manager: Arc<IpfsManager>,
    hf_manager: Arc<HuggingFaceManager>,
}

// ===== Node Commands =====

#[tauri::command]
async fn start_node(state: State<'_, AppState>) -> Result<String, String> {
    info!("start_node command received");
    tracing::error!("DEBUG: start_node called"); // Add visible debug output

    // Always start embedded node for mining and earning rewards
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
            let storage_opt = state.node_manager.get_storage().await;
            let ghostdag_opt = state.node_manager.get_ghostdag().await;

            tracing::info!("DAG manager initialization: storage={}, ghostdag={}",
                storage_opt.is_some(), ghostdag_opt.is_some());

            if let (Some(storage), Some(ghostdag)) = (storage_opt, ghostdag_opt) {
                let dag_manager = Arc::new(DAGManager::new(storage.clone(), ghostdag.clone()));
                *state.dag_manager.write().await = Some(dag_manager.clone());
                info!("DAG manager initialized successfully");

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
            } else {
                tracing::warn!("DAG manager NOT initialized - storage or ghostdag unavailable");
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
    // Always use embedded node status
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
        // Use OS data dir: e.g., macOS ~/Library/Application Support/citrate-core/testnet
        if let Some(mut base) = dirs::data_dir() {
            base.push("citrate-core");
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

    // Get current config
    let current_config = state.node_manager.get_config().await;

    // Check if already in testnet mode with correct settings
    if current_config.network == "testnet"
        && current_config.mempool.chain_id == 42069
        && current_config.bootnodes.contains(&"127.0.0.1:30303".to_string()) {
        info!("Already in testnet mode with correct configuration, skipping");
        return Ok("Already in testnet mode".to_string());
    }

    // Only stop node if we're actually changing networks
    info!("Network configuration changed, stopping node to apply new settings");
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

    // DON'T auto-start node here - let start_node handle it
    // This allows external RPC to be used when configured

    info!(
        "GUI switched to testnet mode - Chain ID: {}, P2P: {}",
        config.mempool.chain_id, config.p2p_port
    );

    Ok(format!(
        "Testnet mode configured - Chain ID: {}, P2P Port: {} (node not started)",
        config.mempool.chain_id, config.p2p_port
    ))
}

#[tauri::command]
async fn ensure_connectivity(state: State<'_, AppState>) -> Result<String, String> {
    // Get current peer count
    let peer_count = state.node_manager.get_peers_summary().await.len();

    if peer_count == 0 {
        warn!("No peers connected. Attempting to connect to bootnodes...");

        // Try to connect to configured bootnodes
        let connected = state.node_manager.connect_bootnodes_now().await.map_err(|e| e.to_string())?;
        info!("Connected to {} bootnode(s)", connected);

        let new_peer_count = state.node_manager.get_peers_summary().await.len();
        Ok(format!("Connectivity check complete. Connected peers: {} (was {})", new_peer_count, peer_count))
    } else {
        Ok(format!("Connectivity check complete. Connected peers: {}", peer_count))
    }
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
async fn delete_account(
    state: State<'_, AppState>,
    address: String,
) -> Result<(), String> {
    state
        .wallet_manager
        .delete_account(&address)
        .await
        .map_err(|e| e.to_string())
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
    // Always use embedded node for transactions
    let tx = state
        .wallet_manager
        .create_signed_transaction(request.clone(), &password)
        .await
        .map_err(|e| e.to_string())?;
    let tx_hash_hex = hex::encode(tx.hash.as_bytes());

    // Add to local mempool - Mempool is internally synchronized
    if let Some(mempool) = state.node_manager.get_mempool().await {
        let _ = mempool.add_transaction(tx.clone(), TxClass::Standard).await;
    }
    // Broadcast to peers
    let _ = state
        .node_manager
        .broadcast_network(NetworkMessage::NewTransaction { transaction: tx })
        .await;
    Ok(tx_hash_hex)
}

#[derive(Debug, serde::Deserialize)]
struct EthCallRequest {
    to: String,
    data: String,
    from: Option<String>,
}

#[tauri::command]
async fn eth_call(
    state: State<'_, AppState>,
    request: EthCallRequest,
) -> Result<String, String> {
    use citrate_consensus::types::{Hash, PublicKey, Signature, Transaction};

    // Get executor from node manager
    let executor = state.node_manager.get_executor().await
        .ok_or_else(|| "Node not started - executor unavailable".to_string())?;

    // Parse the 'to' address
    let to_bytes = hex::decode(request.to.trim_start_matches("0x"))
        .map_err(|e| format!("Invalid 'to' address: {}", e))?;
    if to_bytes.len() != 20 {
        return Err("'to' address must be 20 bytes".to_string());
    }

    // Parse call data
    let data = hex::decode(request.data.trim_start_matches("0x"))
        .map_err(|e| format!("Invalid call data: {}", e))?;

    // Parse optional 'from' address for PublicKey
    let from_pk = if let Some(from) = request.from {
        let from_bytes = hex::decode(from.trim_start_matches("0x"))
            .map_err(|e| format!("Invalid 'from' address: {}", e))?;
        if from_bytes.len() != 20 {
            return Err("'from' address must be 20 bytes".to_string());
        }
        // Pad 20-byte address to 32-byte pubkey format
        let mut pk_bytes = [0u8; 32];
        pk_bytes[..20].copy_from_slice(&from_bytes);
        PublicKey::new(pk_bytes)
    } else {
        PublicKey::new([0u8; 32]) // Zero address as default sender
    };

    // Create 'to' pubkey - pad 20-byte address to 32-byte format
    let mut to_pk_bytes = [0u8; 32];
    to_pk_bytes[..20].copy_from_slice(&to_bytes);
    let to_pk = PublicKey::new(to_pk_bytes);

    // Create a simulated call transaction (no state changes will be committed)
    let call_tx = Transaction {
        hash: Hash::default(),
        from: from_pk,
        to: Some(to_pk),
        value: 0,
        data: data.clone(),
        nonce: 0,
        gas_price: 0,
        gas_limit: 30_000_000, // High gas limit for calls
        signature: Signature::new([0u8; 64]),
        tx_type: None,
    };

    // Create a minimal block for execution context
    let dummy_block = citrate_consensus::Block {
        header: citrate_consensus::BlockHeader {
            version: 1,
            block_hash: Hash::default(),
            selected_parent_hash: Hash::default(),
            merge_parent_hashes: vec![],
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            height: 0,
            blue_score: 0,
            blue_work: 0,
            pruning_point: Hash::default(),
            proposer_pubkey: PublicKey::new([0u8; 32]),
            vrf_reveal: citrate_consensus::VrfProof {
                proof: vec![],
                output: Hash::default(),
            },
            base_fee_per_gas: 1_000_000_000,
            gas_used: 0,
            gas_limit: 30_000_000,
        },
        state_root: Hash::default(),
        tx_root: Hash::default(),
        receipt_root: Hash::default(),
        artifact_root: Hash::default(),
        ghostdag_params: citrate_consensus::GhostDagParams::default(),
        signature: Signature::new([0u8; 64]),
        transactions: vec![],
        embedded_models: vec![],
        required_pins: vec![],
    };

    // Execute the call transaction
    match executor.execute_transaction(&dummy_block, &call_tx).await {
        Ok(receipt) => {
            // Return the result data from the receipt
            Ok(format!("0x{}", hex::encode(&receipt.output)))
        }
        Err(e) => Err(format!("Call execution failed: {}", e)),
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

// ===== Window Commands =====

#[tauri::command]
async fn create_window(
    state: State<'_, AppState>,
    window_id: String,
    window_type: String,
    title: String,
    width: f64,
    height: f64,
    x: Option<f64>,
    y: Option<f64>,
    data: Option<String>,
) -> Result<WindowState, String> {
    let wtype: WindowType = window_type.parse().map_err(|e: String| e)?;
    let data_value = data
        .map(|s| serde_json::from_str(&s))
        .transpose()
        .map_err(|e| format!("Invalid data JSON: {}", e))?;

    let manager = state.window_manager.read().await;
    manager
        .create_window(&window_id, wtype, &title, width, height, x, y, data_value)
        .await
}

#[tauri::command]
async fn close_window(state: State<'_, AppState>, window_id: String) -> Result<(), String> {
    let manager = state.window_manager.read().await;
    manager.close_window(&window_id).await
}

#[tauri::command]
async fn focus_window(state: State<'_, AppState>, window_id: String) -> Result<(), String> {
    let manager = state.window_manager.read().await;
    manager.focus_window(&window_id).await
}

#[tauri::command]
async fn send_to_window(
    state: State<'_, AppState>,
    window_id: String,
    event: String,
    payload: serde_json::Value,
) -> Result<(), String> {
    let manager = state.window_manager.read().await;
    manager.send_to_window(&window_id, &event, payload).await
}

#[tauri::command]
async fn broadcast_to_windows(
    state: State<'_, AppState>,
    event: String,
    payload: serde_json::Value,
) -> Result<(), String> {
    let manager = state.window_manager.read().await;
    manager.broadcast(&event, payload).await
}

#[tauri::command]
async fn get_window_state(
    state: State<'_, AppState>,
    window_id: String,
) -> Result<Option<WindowState>, String> {
    let manager = state.window_manager.read().await;
    Ok(manager.get_window(&window_id).await)
}

#[tauri::command]
async fn get_all_windows(state: State<'_, AppState>) -> Result<Vec<WindowState>, String> {
    let manager = state.window_manager.read().await;
    Ok(manager.get_all_windows().await)
}

#[tauri::command]
async fn get_windows_by_type(
    state: State<'_, AppState>,
    window_type: String,
) -> Result<Vec<WindowState>, String> {
    let wtype: WindowType = window_type.parse().map_err(|e: String| e)?;
    let manager = state.window_manager.read().await;
    Ok(manager.get_windows_by_type(wtype).await)
}

#[tauri::command]
async fn has_window_type(
    state: State<'_, AppState>,
    window_type: String,
) -> Result<bool, String> {
    let wtype: WindowType = window_type.parse().map_err(|e: String| e)?;
    let manager = state.window_manager.read().await;
    Ok(manager.has_window_type(wtype).await)
}

#[tauri::command]
async fn get_window_count(state: State<'_, AppState>) -> Result<usize, String> {
    let manager = state.window_manager.read().await;
    Ok(manager.window_count().await)
}

// ===== Terminal Commands =====

#[derive(Debug, serde::Deserialize)]
struct CreateTerminalArgs {
    shell: Option<String>,
    cwd: Option<String>,
    cols: Option<u16>,
    rows: Option<u16>,
}

#[tauri::command]
async fn terminal_create(
    state: State<'_, AppState>,
    args: CreateTerminalArgs,
) -> Result<TerminalInfo, String> {
    let config = TerminalConfig {
        shell: args.shell,
        cwd: args.cwd,
        env: vec![],
        cols: args.cols.unwrap_or(80),
        rows: args.rows.unwrap_or(24),
    };

    let manager = state.terminal_manager.read().await;
    manager
        .create_session(config)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn terminal_write(
    state: State<'_, AppState>,
    session_id: String,
    data: String,
) -> Result<(), String> {
    let manager = state.terminal_manager.read().await;
    manager
        .write_input(&session_id, data.as_bytes())
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn terminal_resize(
    state: State<'_, AppState>,
    session_id: String,
    cols: u16,
    rows: u16,
) -> Result<(), String> {
    let manager = state.terminal_manager.read().await;
    manager
        .resize_session(&session_id, cols, rows)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn terminal_close(state: State<'_, AppState>, session_id: String) -> Result<(), String> {
    let manager = state.terminal_manager.read().await;
    manager
        .close_session(&session_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn terminal_list(state: State<'_, AppState>) -> Result<Vec<TerminalInfo>, String> {
    let manager = state.terminal_manager.read().await;
    Ok(manager.list_sessions().await)
}

#[tauri::command]
async fn terminal_get(
    state: State<'_, AppState>,
    session_id: String,
) -> Result<Option<TerminalInfo>, String> {
    let manager = state.terminal_manager.read().await;
    Ok(manager.get_session(&session_id).await)
}

// ===== IPFS Commands =====

#[tauri::command]
async fn ipfs_start(state: State<'_, AppState>) -> Result<IpfsStatus, String> {
    state.ipfs_manager.start().await?;
    Ok(state.ipfs_manager.get_status().await)
}

#[tauri::command]
async fn ipfs_stop(state: State<'_, AppState>) -> Result<(), String> {
    state.ipfs_manager.stop().await
}

#[tauri::command]
async fn ipfs_status(state: State<'_, AppState>) -> Result<IpfsStatus, String> {
    if state.ipfs_manager.is_running().await {
        state.ipfs_manager.refresh_status().await
    } else {
        Ok(state.ipfs_manager.get_status().await)
    }
}

#[tauri::command]
async fn ipfs_get_config(state: State<'_, AppState>) -> Result<IpfsConfig, String> {
    Ok(state.ipfs_manager.get_config().await)
}

#[tauri::command]
async fn ipfs_update_config(state: State<'_, AppState>, config: IpfsConfig) -> Result<(), String> {
    state.ipfs_manager.update_config(config).await;
    Ok(())
}

#[tauri::command]
async fn ipfs_add(
    state: State<'_, AppState>,
    data: String,
    name: Option<String>,
) -> Result<IpfsAddResult, String> {
    // Decode base64 data
    let bytes = base64::Engine::decode(
        &base64::engine::general_purpose::STANDARD,
        &data,
    )
    .map_err(|e| format!("Failed to decode base64 data: {}", e))?;

    state.ipfs_manager.add(bytes, name.as_deref()).await
}

#[tauri::command]
async fn ipfs_add_file(state: State<'_, AppState>, path: String) -> Result<IpfsAddResult, String> {
    let path = std::path::PathBuf::from(path);
    state.ipfs_manager.add_file(&path).await
}

#[tauri::command]
async fn ipfs_get(state: State<'_, AppState>, cid: String) -> Result<IpfsContent, String> {
    state.ipfs_manager.get(&cid).await
}

#[tauri::command]
async fn ipfs_pin(state: State<'_, AppState>, cid: String) -> Result<(), String> {
    state.ipfs_manager.pin(&cid).await
}

#[tauri::command]
async fn ipfs_unpin(state: State<'_, AppState>, cid: String) -> Result<(), String> {
    state.ipfs_manager.unpin(&cid).await
}

#[tauri::command]
async fn ipfs_list_pins(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    state.ipfs_manager.list_pins().await
}

#[tauri::command]
async fn ipfs_get_peers(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    state.ipfs_manager.get_peers().await
}

// ===== HuggingFace Commands =====

#[tauri::command]
async fn hf_get_auth_url(state: State<'_, AppState>) -> Result<String, String> {
    // Generate a random state parameter for CSRF protection
    let random_state = uuid::Uuid::new_v4().to_string();
    state.hf_manager.get_auth_url(&random_state).await
}

#[tauri::command]
async fn hf_exchange_code(
    state: State<'_, AppState>,
    code: String,
) -> Result<HFAuthState, String> {
    let token = state.hf_manager.exchange_code(&code).await?;
    state.hf_manager.set_token(token).await?;
    Ok(state.hf_manager.get_auth_state().await)
}

#[tauri::command]
async fn hf_set_token(state: State<'_, AppState>, token: String) -> Result<(), String> {
    state.hf_manager.set_api_token(&token).await
}

#[tauri::command]
async fn hf_get_auth_state(state: State<'_, AppState>) -> Result<HFAuthState, String> {
    Ok(state.hf_manager.get_auth_state().await)
}

#[tauri::command]
async fn hf_logout(state: State<'_, AppState>) -> Result<(), String> {
    state.hf_manager.logout().await;
    Ok(())
}

#[tauri::command]
async fn hf_get_config(state: State<'_, AppState>) -> Result<HFConfig, String> {
    Ok(state.hf_manager.get_config().await)
}

#[tauri::command]
async fn hf_update_config(state: State<'_, AppState>, config: HFConfig) -> Result<(), String> {
    state.hf_manager.update_config(config).await;
    Ok(())
}

#[tauri::command]
async fn hf_search_models(
    state: State<'_, AppState>,
    params: ModelSearchParams,
) -> Result<Vec<HFModelInfo>, String> {
    state.hf_manager.search_models(params).await
}

#[tauri::command]
async fn hf_get_model_info(
    state: State<'_, AppState>,
    model_id: String,
) -> Result<HFModelInfo, String> {
    state.hf_manager.get_model(&model_id).await
}

#[tauri::command]
async fn hf_get_model_files(
    state: State<'_, AppState>,
    model_id: String,
) -> Result<Vec<HFModelFile>, String> {
    state.hf_manager.list_gguf_files(&model_id).await
}

#[tauri::command]
async fn hf_download_file(
    state: State<'_, AppState>,
    model_id: String,
    filename: String,
) -> Result<String, String> {
    state
        .hf_manager
        .download_file(&model_id, &filename)
        .await
        .map(|p| p.to_string_lossy().to_string())
}

#[tauri::command]
async fn hf_get_downloads(state: State<'_, AppState>) -> Result<Vec<DownloadProgress>, String> {
    Ok(state.hf_manager.get_downloads().await)
}

#[tauri::command]
async fn hf_cancel_download(
    state: State<'_, AppState>,
    model_id: String,
    filename: String,
) -> Result<(), String> {
    state.hf_manager.cancel_download(&model_id, &filename).await;
    Ok(())
}

#[tauri::command]
async fn hf_get_local_models(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    state.hf_manager.list_local_models().await
}

#[tauri::command]
async fn hf_get_models_dir(state: State<'_, AppState>) -> Result<String, String> {
    Ok(state.hf_manager.get_models_dir().await.to_string_lossy().to_string())
}

// ===== Contract Compilation Commands (Foundry CLI) =====

/// Check if Foundry/forge is installed
#[tauri::command]
async fn forge_check_installed() -> Result<ForgeInfo, String> {
    use std::process::Command;

    // Check for forge binary
    let forge_result = Command::new("forge")
        .arg("--version")
        .output();

    match forge_result {
        Ok(output) if output.status.success() => {
            let version = String::from_utf8_lossy(&output.stdout).trim().to_string();

            // Also check for cast
            let cast_available = Command::new("cast")
                .arg("--version")
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false);

            // Check for anvil
            let anvil_available = Command::new("anvil")
                .arg("--version")
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false);

            Ok(ForgeInfo {
                installed: true,
                version: Some(version),
                cast_available,
                anvil_available,
                path: which_forge(),
            })
        }
        _ => Ok(ForgeInfo {
            installed: false,
            version: None,
            cast_available: false,
            anvil_available: false,
            path: None,
        }),
    }
}

/// Compile contracts using forge build
#[tauri::command]
async fn forge_build(
    project_path: String,
    contract_name: Option<String>,
    optimizer_runs: Option<u32>,
) -> Result<ForgeBuildResult, String> {
    use std::process::Command;
    use std::path::Path;

    let project_dir = Path::new(&project_path);
    if !project_dir.exists() {
        return Err(format!("Project directory does not exist: {}", project_path));
    }

    // Check for foundry.toml (used for logging)
    let has_foundry_config = project_dir.join("foundry.toml").exists();
    if !has_foundry_config {
        info!("No foundry.toml found, using default settings");
    }

    let mut cmd = Command::new("forge");
    cmd.current_dir(project_dir);
    cmd.arg("build");
    cmd.arg("--json"); // Output as JSON for parsing

    // Add optimizer settings if provided
    if let Some(runs) = optimizer_runs {
        cmd.arg("--optimize");
        cmd.arg("--optimizer-runs");
        cmd.arg(runs.to_string());
    }

    info!("Running forge build in {}", project_path);

    let output = cmd.output()
        .map_err(|e| format!("Failed to run forge: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    if !output.status.success() {
        return Ok(ForgeBuildResult {
            success: false,
            contracts: vec![],
            errors: vec![stderr],
            warnings: vec![],
            build_time_ms: None,
        });
    }

    // Parse output from the out/ directory
    let out_dir = project_dir.join("out");
    let mut contracts = Vec::new();
    let mut warnings = Vec::new();

    // Extract warnings from stderr
    for line in stderr.lines() {
        if line.contains("Warning") || line.contains("warning") {
            warnings.push(line.to_string());
        }
    }

    // Read compiled artifacts
    if out_dir.exists() {
        if let Ok(entries) = std::fs::read_dir(&out_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    // Each contract has its own directory
                    if let Some(sol_name) = path.file_name().and_then(|n| n.to_str()) {
                        if sol_name.ends_with(".sol") {
                            // Read the JSON artifact
                            if let Ok(artifacts) = std::fs::read_dir(&path) {
                                for artifact in artifacts.flatten() {
                                    let artifact_path = artifact.path();
                                    if artifact_path.extension().map(|e| e == "json").unwrap_or(false) {
                                        if let Ok(content) = std::fs::read_to_string(&artifact_path) {
                                            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                                                let name = artifact_path.file_stem()
                                                    .and_then(|n| n.to_str())
                                                    .unwrap_or("Unknown")
                                                    .to_string();

                                                // Filter by contract name if specified
                                                if let Some(ref filter) = contract_name {
                                                    if &name != filter {
                                                        continue;
                                                    }
                                                }

                                                let bytecode = json.get("bytecode")
                                                    .and_then(|b| b.get("object"))
                                                    .and_then(|o| o.as_str())
                                                    .map(|s| if s.starts_with("0x") { s.to_string() } else { format!("0x{}", s) });

                                                let deployed_bytecode = json.get("deployedBytecode")
                                                    .and_then(|b| b.get("object"))
                                                    .and_then(|o| o.as_str())
                                                    .map(|s| if s.starts_with("0x") { s.to_string() } else { format!("0x{}", s) });

                                                let abi = json.get("abi").cloned();

                                                if bytecode.is_some() || abi.is_some() {
                                                    contracts.push(ForgeContract {
                                                        name,
                                                        source_file: sol_name.to_string(),
                                                        bytecode,
                                                        deployed_bytecode,
                                                        abi,
                                                    });
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(ForgeBuildResult {
        success: true,
        contracts,
        errors: vec![],
        warnings,
        build_time_ms: None,
    })
}

/// Initialize a new Foundry project
#[tauri::command]
async fn forge_init(project_path: String, template: Option<String>) -> Result<String, String> {
    use std::process::Command;
    use std::path::Path;

    let project_dir = Path::new(&project_path);

    // Create parent directory if needed
    if let Some(parent) = project_dir.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create directory: {}", e))?;
    }

    let mut cmd = Command::new("forge");
    cmd.arg("init");
    cmd.arg(&project_path);

    if let Some(ref t) = template {
        cmd.arg("--template");
        cmd.arg(t);
    }

    let output = cmd.output()
        .map_err(|e| format!("Failed to run forge init: {}", e))?;

    if output.status.success() {
        Ok(format!("Foundry project initialized at {}", project_path))
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

/// Run forge tests
#[tauri::command]
async fn forge_test(
    project_path: String,
    test_filter: Option<String>,
    verbosity: Option<u8>,
) -> Result<ForgeTestResult, String> {
    use std::process::Command;
    use std::path::Path;

    let project_dir = Path::new(&project_path);
    if !project_dir.exists() {
        return Err(format!("Project directory does not exist: {}", project_path));
    }

    let mut cmd = Command::new("forge");
    cmd.current_dir(project_dir);
    cmd.arg("test");
    cmd.arg("--json");

    if let Some(filter) = test_filter {
        cmd.arg("--match-test");
        cmd.arg(filter);
    }

    if let Some(v) = verbosity {
        let v_flag = "-".to_string() + &"v".repeat(v.min(5) as usize);
        cmd.arg(v_flag);
    }

    let output = cmd.output()
        .map_err(|e| format!("Failed to run forge test: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    // Parse test results from JSON output
    let mut tests_passed = 0u32;
    let mut tests_failed = 0u32;
    let test_results: Vec<String> = stdout.lines()
        .filter(|line| line.contains("PASS") || line.contains("FAIL"))
        .map(|s| s.to_string())
        .collect();

    for line in &test_results {
        if line.contains("PASS") {
            tests_passed += 1;
        } else if line.contains("FAIL") {
            tests_failed += 1;
        }
    }

    Ok(ForgeTestResult {
        success: output.status.success(),
        tests_passed,
        tests_failed,
        test_results,
        gas_report: None,
        errors: if output.status.success() { vec![] } else { vec![stderr] },
    })
}

// Helper function to find forge binary path
fn which_forge() -> Option<String> {
    use std::process::Command;

    Command::new("which")
        .arg("forge")
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
}

/// Foundry installation info
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct ForgeInfo {
    installed: bool,
    version: Option<String>,
    cast_available: bool,
    anvil_available: bool,
    path: Option<String>,
}

/// Compiled contract from forge
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct ForgeContract {
    name: String,
    source_file: String,
    bytecode: Option<String>,
    deployed_bytecode: Option<String>,
    abi: Option<serde_json::Value>,
}

/// Forge build result
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct ForgeBuildResult {
    success: bool,
    contracts: Vec<ForgeContract>,
    errors: Vec<String>,
    warnings: Vec<String>,
    build_time_ms: Option<u64>,
}

/// Forge test result
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct ForgeTestResult {
    success: bool,
    tests_passed: u32,
    tests_failed: u32,
    test_results: Vec<String>,
    gas_report: Option<String>,
    errors: Vec<String>,
}

// Setup function to initialize node components after startup
async fn setup_node_components(app_handle: tauri::AppHandle) {
    info!("Setting up node components");

    // Get app state
    let state = app_handle.state::<AppState>();
    let mut use_reward_address: Option<String> = None;
    // Prefer environment override
    if let Ok(addr) = std::env::var("CITRATE_REWARD_ADDRESS") {
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
        .with_env_filter("info,citrate_core=debug")
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
    let window_manager = Arc::new(RwLock::new(WindowManager::new()));
    let terminal_manager = Arc::new(RwLock::new(TerminalManager::new()));
    let ipfs_manager = Arc::new(IpfsManager::new());
    let hf_manager = Arc::new(HuggingFaceManager::new());

    // Create agent state (initialized lazily when node starts)
    let agent_state = AgentState::new();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(AppState {
            node_manager,
            wallet_manager,
            model_manager,
            dag_manager: Arc::new(RwLock::new(None)),
            external_rpc: Arc::new(RwLock::new(None)),
            window_manager,
            terminal_manager,
            ipfs_manager: ipfs_manager.clone(),
            hf_manager,
        })
        .manage(agent_state)
        // Expose IPFS manager separately for agent commands
        .manage(ipfs_manager)
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
            delete_account,
            is_first_time_setup,
            perform_first_time_setup,
            get_account,
            send_transaction,
            eth_call,
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
            // Agent commands
            agent_create_session,
            agent_get_session,
            agent_list_sessions,
            agent_delete_session,
            agent_send_message,
            agent_get_messages,
            agent_clear_history,
            agent_get_pending_tools,
            agent_approve_tool,
            agent_reject_tool,
            agent_get_config,
            agent_update_config,
            agent_scan_local_models,
            agent_get_models_dir,
            agent_is_ready,
            agent_get_status,
            agent_load_local_model,
            agent_get_active_model,
            agent_set_api_key,
            agent_set_auto_mode,
            // Multi-provider AI configuration commands
            get_ai_providers_config,
            get_ai_provider_keys,
            update_ai_providers_config,
            save_ai_providers_config,
            test_ai_provider_connection,
            pin_local_model_to_ipfs,
            delete_local_model,
            download_model_from_ipfs,
            set_preferred_provider_order,
            set_local_fallback,
            check_onboarding_status,
            complete_onboarding,
            // Window commands
            create_window,
            close_window,
            focus_window,
            send_to_window,
            broadcast_to_windows,
            get_window_state,
            get_all_windows,
            get_windows_by_type,
            has_window_type,
            get_window_count,
            // Terminal commands
            terminal_create,
            terminal_write,
            terminal_resize,
            terminal_close,
            terminal_list,
            terminal_get,
            // IPFS commands
            ipfs_start,
            ipfs_stop,
            ipfs_status,
            ipfs_get_config,
            ipfs_update_config,
            ipfs_add,
            ipfs_add_file,
            ipfs_get,
            ipfs_pin,
            ipfs_unpin,
            ipfs_list_pins,
            ipfs_get_peers,
            // HuggingFace commands
            hf_get_auth_url,
            hf_exchange_code,
            hf_set_token,
            hf_get_auth_state,
            hf_logout,
            hf_get_config,
            hf_update_config,
            hf_search_models,
            hf_get_model_info,
            hf_get_model_files,
            hf_download_file,
            hf_get_downloads,
            hf_cancel_download,
            hf_get_local_models,
            hf_get_models_dir,
            // Foundry/Contract compilation commands
            forge_check_installed,
            forge_build,
            forge_init,
            forge_test,
        ])
        .setup(|app| {
            // Initialize window manager with app handle
            let app_handle = app.handle().clone();
            {
                let state = app_handle.state::<AppState>();
                tauri::async_runtime::block_on(async {
                    let mut wm = state.window_manager.write().await;
                    wm.set_app_handle(app_handle.clone());

                    // Initialize terminal manager with app handle
                    let mut tm = state.terminal_manager.write().await;
                    tm.set_app_handle(app_handle.clone());
                });
            }

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
            // Initialize agent with managers
            let app_handle3 = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                // Wait a moment for node to initialize
                sleep(std::time::Duration::from_millis(100)).await;
                let app_state = app_handle3.state::<AppState>();
                let agent_state = app_handle3.state::<AgentState>();

                // Create agent manager with references to other managers
                let agent_manager = agent::AgentManager::new(
                    app_state.node_manager.clone(),
                    app_state.wallet_manager.clone(),
                    app_state.model_manager.clone(),
                    app_state.dag_manager.clone(),
                );

                // Check for bundled model in app resources and configure if found
                if let Ok(resource_path) = app_handle3.path().resource_dir() {
                    let bundled_model_path = resource_path.join("models").join("qwen2-0_5b-instruct-q4_k_m.gguf");
                    if bundled_model_path.exists() {
                        info!("Found bundled model at: {:?}", bundled_model_path);

                        // Copy to user models directory for persistent access
                        if let Some(models_dir) = agent::llm::local::get_default_models_dir() {
                            if let Err(e) = std::fs::create_dir_all(&models_dir) {
                                warn!("Failed to create models directory: {}", e);
                            } else {
                                let dest_path = models_dir.join("qwen2-0_5b-instruct-q4_k_m.gguf");
                                if !dest_path.exists() {
                                    info!("Copying bundled model to: {:?}", dest_path);
                                    if let Err(e) = std::fs::copy(&bundled_model_path, &dest_path) {
                                        warn!("Failed to copy bundled model: {}", e);
                                    }
                                }
                                // Configure the agent with the model path
                                if dest_path.exists() {
                                    let model_path = dest_path.to_string_lossy().to_string();
                                    agent_manager.configure_local_model(model_path).await;
                                    info!("Bundled model configured for agent");
                                }
                            }
                        }
                    } else {
                        info!("No bundled model found at {:?}", bundled_model_path);
                    }
                }

                *agent_state.manager.write().await = Some(agent_manager);
                info!("Agent manager initialized");
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
