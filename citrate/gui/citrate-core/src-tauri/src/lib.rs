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
mod gpu;
mod huggingface;
mod image_models;
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
    TrainingJob, LoraConfig, LoraTrainingConfig, LoraTrainingJob, LoraAdapterInfo,
    DatasetFormat, DatasetValidation, LoraPreset,
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
    ModelSearchParams, DownloadProgress, AuthState as HFAuthState, OAuthToken,
    GGUFModelInfo, GGUFFileInfo, LocalModelInfo,
};
use gpu::{
    GPUResourceManager, GPUDevice, GPUAllocationSettings, GPUStats,
    ProviderStatus, ComputeJob, ComputeJobType, ComputeJobStatus,
};
use image_models::{
    ImageModelManager, ImageModel, ImageGenerationRequest, GenerationJob,
    ImageTrainingConfig, ImageTrainingJob, GeneratedImage, ImageResolution,
    Scheduler as ImageScheduler,
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
    // Secure API key management commands
    secure_store_api_key, validate_api_key_format, validate_stored_api_key,
    has_secure_api_key, delete_secure_api_key, load_secure_api_keys, get_secure_api_key_status,
    // Enhanced model download commands
    download_enhanced_model, check_enhanced_model_status, cancel_model_download,
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
    gpu_manager: Arc<GPUResourceManager>,
    image_model_manager: Arc<ImageModelManager>,
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

// ===== Tracked Addresses =====

/// Get the path to the tracked addresses file
fn tracked_addresses_path() -> std::path::PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("citrate-core")
        .join("tracked_addresses.json")
}

#[tauri::command]
async fn get_tracked_addresses() -> Result<Vec<String>, String> {
    let path = tracked_addresses_path();
    if !path.exists() {
        return Ok(Vec::new());
    }
    let contents = std::fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read tracked addresses: {}", e))?;
    let addresses: Vec<String> = serde_json::from_str(&contents)
        .map_err(|e| format!("Failed to parse tracked addresses: {}", e))?;
    Ok(addresses)
}

#[tauri::command]
async fn save_tracked_addresses(addresses: Vec<String>) -> Result<(), String> {
    let path = tracked_addresses_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create directory: {}", e))?;
    }
    let contents = serde_json::to_string_pretty(&addresses)
        .map_err(|e| format!("Failed to serialize addresses: {}", e))?;
    std::fs::write(&path, contents)
        .map_err(|e| format!("Failed to save tracked addresses: {}", e))?;
    Ok(())
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

/// Check if first-time setup is needed (returns true if no wallet exists)
/// SECURITY: Does NOT auto-create wallet - requires user to provide password through onboarding
#[tauri::command]
async fn check_first_time_and_setup_if_needed(
    state: State<'_, AppState>,
) -> Result<Option<FirstTimeSetupResult>, String> {
    // Check if this is first time setup
    if state.wallet_manager.is_first_time_setup().await {
        info!("First-time user detected. Wallet setup required via onboarding.");

        // SECURITY FIX: Do NOT auto-create wallet with hardcoded password
        // Return None to indicate setup is needed, but actual wallet creation
        // must happen through the proper onboarding flow with user-provided password
        //
        // The frontend should:
        // 1. Detect this is first-time setup
        // 2. Show password setup screen in onboarding
        // 3. Call perform_first_time_setup with user-provided password

        Ok(None)
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

/// Validate password strength before wallet creation
/// Returns password strength information including score and any issues
#[tauri::command]
async fn validate_password_strength(
    password: String,
) -> Result<wallet::PasswordStrength, String> {
    Ok(wallet::WalletManager::check_password_strength(&password))
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

/// Delete an account (requires password for re-authentication)
/// This is an irreversible operation
#[tauri::command]
async fn delete_account(
    state: State<'_, AppState>,
    address: String,
    password: String,
) -> Result<(), String> {
    state
        .wallet_manager
        .delete_account(&address, &password)
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

// ===== Session Management Commands =====

/// Get remaining session time in seconds for an address
#[tauri::command]
async fn get_session_remaining(
    state: State<'_, AppState>,
    address: String,
) -> Result<u64, String> {
    match state.wallet_manager.get_session_remaining(&address).await {
        Some(remaining) => Ok(remaining),
        None => Ok(0), // No active session
    }
}

/// Check if a session is currently active for an address
#[tauri::command]
async fn is_session_active(
    state: State<'_, AppState>,
    address: String,
) -> Result<bool, String> {
    Ok(state.wallet_manager.is_session_valid(&address).await)
}

/// Lock wallet (end session) for an address
#[tauri::command]
async fn lock_wallet(
    state: State<'_, AppState>,
    address: String,
) -> Result<(), String> {
    state.wallet_manager.lock_wallet(&address).await;
    info!("Wallet locked via Tauri command for address: {}", address);
    Ok(())
}

/// Lock all wallets (end all sessions)
#[tauri::command]
async fn lock_all_wallets(
    state: State<'_, AppState>,
) -> Result<u32, String> {
    let accounts = state.wallet_manager.get_accounts().await;
    let mut locked_count = 0u32;
    for account in accounts {
        if state.wallet_manager.is_session_valid(&account.address).await {
            state.wallet_manager.lock_wallet(&account.address).await;
            locked_count += 1;
        }
    }
    info!("Locked {} wallet sessions via lock_all_wallets command", locked_count);
    Ok(locked_count)
}

/// Check if password is required for a transaction
/// Returns true if password needed, false if session can be used
#[tauri::command]
async fn check_password_required(
    state: State<'_, AppState>,
    address: String,
    value: String,
) -> Result<bool, String> {
    // Parse value to check if high-value transaction
    let value_u128: u128 = value.parse().unwrap_or(0);
    let requires_reauth = wallet::WalletManager::requires_reauth(
        value_u128,
        wallet::SensitiveOperation::SignTransaction,
    );

    // If high-value, always require password
    if requires_reauth {
        return Ok(true);
    }

    // Check if session is active with cached key
    let has_session = state.wallet_manager.is_session_valid(&address).await;
    let has_cached_key = state.wallet_manager.get_cached_signing_key(&address).await.is_some();

    // Password required if no active session or no cached key
    Ok(!has_session || !has_cached_key)
}

#[tauri::command]
async fn send_transaction(
    state: State<'_, AppState>,
    request: TransactionRequest,
    password: Option<String>,
) -> Result<String, String> {
    // Use empty string if password not provided (will try session-based signing)
    let pwd = password.unwrap_or_default();

    // Always use embedded node for transactions
    let tx = state
        .wallet_manager
        .create_signed_transaction(request.clone(), &pwd)
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

// ===== LoRA Training Commands =====

/// Create a new LoRA training job
#[tauri::command]
async fn create_lora_job(
    state: State<'_, AppState>,
    base_model_path: String,
    base_model_name: String,
    dataset_path: String,
    dataset_format: DatasetFormat,
    output_dir: String,
    lora_config: Option<LoraConfig>,
    training_config: Option<LoraTrainingConfig>,
) -> Result<LoraTrainingJob, String> {
    state
        .model_manager
        .create_lora_job(
            base_model_path,
            base_model_name,
            dataset_path,
            dataset_format,
            output_dir,
            lora_config,
            training_config,
        )
        .await
        .map_err(|e| e.to_string())
}

/// Start a queued LoRA training job
#[tauri::command]
async fn start_lora_training(state: State<'_, AppState>, job_id: String) -> Result<(), String> {
    state
        .model_manager
        .start_lora_training(&job_id)
        .await
        .map_err(|e| e.to_string())
}

/// Get a specific LoRA training job by ID
#[tauri::command]
async fn get_lora_job(
    state: State<'_, AppState>,
    job_id: String,
) -> Result<Option<LoraTrainingJob>, String> {
    state
        .model_manager
        .get_lora_job(&job_id)
        .await
        .map_err(|e| e.to_string())
}

/// Get all LoRA training jobs
#[tauri::command]
async fn get_lora_jobs(state: State<'_, AppState>) -> Result<Vec<LoraTrainingJob>, String> {
    state
        .model_manager
        .get_lora_jobs()
        .await
        .map_err(|e| e.to_string())
}

/// Cancel a running LoRA training job
#[tauri::command]
async fn cancel_lora_job(state: State<'_, AppState>, job_id: String) -> Result<(), String> {
    state
        .model_manager
        .cancel_lora_job(&job_id)
        .await
        .map_err(|e| e.to_string())
}

/// Delete a LoRA training job
#[tauri::command]
async fn delete_lora_job(state: State<'_, AppState>, job_id: String) -> Result<(), String> {
    state
        .model_manager
        .delete_lora_job(&job_id)
        .await
        .map_err(|e| e.to_string())
}

/// Get all saved LoRA adapters
#[tauri::command]
async fn get_lora_adapters(state: State<'_, AppState>) -> Result<Vec<LoraAdapterInfo>, String> {
    state
        .model_manager
        .get_lora_adapters()
        .await
        .map_err(|e| e.to_string())
}

/// Delete a LoRA adapter
#[tauri::command]
async fn delete_lora_adapter(state: State<'_, AppState>, adapter_id: String) -> Result<(), String> {
    state
        .model_manager
        .delete_lora_adapter(&adapter_id)
        .await
        .map_err(|e| e.to_string())
}

/// Run inference with a LoRA adapter
#[tauri::command]
async fn run_inference_with_lora(
    state: State<'_, AppState>,
    model_path: String,
    adapter_path: String,
    prompt: String,
    max_tokens: Option<usize>,
    temperature: Option<f32>,
) -> Result<String, String> {
    state
        .model_manager
        .run_inference_with_lora(
            &model_path,
            &adapter_path,
            &prompt,
            max_tokens.unwrap_or(512),
            temperature.unwrap_or(0.7),
        )
        .await
        .map_err(|e| e.to_string())
}

/// Validate a dataset for LoRA training
#[tauri::command]
async fn validate_dataset(
    state: State<'_, AppState>,
    path: String,
    format: DatasetFormat,
) -> Result<DatasetValidation, String> {
    state
        .model_manager
        .validate_dataset(&path, &format)
        .await
        .map_err(|e| e.to_string())
}

/// Get LoRA training presets
#[tauri::command]
fn get_lora_presets() -> Vec<LoraPreset> {
    ModelManager::get_lora_presets()
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

// ===== Enhanced HuggingFace Commands (OAuth PKCE, GGUF, Local Models) =====

/// Start OAuth PKCE auth flow - returns authorization URL and state for verification
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct AuthFlowResponse {
    url: String,
    state: String,
}

#[tauri::command]
async fn hf_start_auth_flow(state: State<'_, AppState>) -> Result<AuthFlowResponse, String> {
    let (url, auth_state) = state.hf_manager.start_auth_flow().await?;
    Ok(AuthFlowResponse { url, state: auth_state })
}

/// Exchange authorization code for token using PKCE verifier
#[tauri::command]
async fn hf_exchange_code_with_pkce(
    app_state: State<'_, AppState>,
    code: String,
    state: String,
) -> Result<HFAuthState, String> {
    let token = app_state.hf_manager.exchange_code_with_pkce(&code, &state).await?;
    app_state.hf_manager.set_token(token).await?;
    Ok(app_state.hf_manager.get_auth_state().await)
}

/// Search for GGUF-compatible models on HuggingFace
#[tauri::command]
async fn hf_search_gguf_models(
    state: State<'_, AppState>,
    query: String,
    limit: Option<u32>,
) -> Result<Vec<GGUFModelInfo>, String> {
    state.hf_manager.search_gguf_models(&query, limit.unwrap_or(20)).await
}

/// Get detailed GGUF model information
#[tauri::command]
async fn hf_get_gguf_model(
    state: State<'_, AppState>,
    model_id: String,
) -> Result<GGUFModelInfo, String> {
    state.hf_manager.get_gguf_model(&model_id).await
}

/// Scan local models directory and return detailed info
#[tauri::command]
async fn hf_scan_local_models(state: State<'_, AppState>) -> Result<Vec<LocalModelInfo>, String> {
    state.hf_manager.scan_local_models().await
}

/// Auto-detect and return the best available local model for inference
#[tauri::command]
async fn hf_auto_select_model(state: State<'_, AppState>) -> Result<Option<LocalModelInfo>, String> {
    Ok(state.hf_manager.auto_select_model().await)
}

/// Download model file with resume support
#[tauri::command]
async fn hf_download_file_resumable(
    state: State<'_, AppState>,
    model_id: String,
    filename: String,
) -> Result<String, String> {
    state.hf_manager
        .download_file_resumable(&model_id, &filename)
        .await
        .map(|p| p.to_string_lossy().to_string())
}

/// Cancel an active download
#[tauri::command]
async fn hf_cancel_download_resumable(
    state: State<'_, AppState>,
    model_id: String,
    filename: String,
) -> Result<(), String> {
    state.hf_manager.cancel_download_resumable(&model_id, &filename).await;
    Ok(())
}

/// Delete a local model file
#[tauri::command]
async fn hf_delete_local_model(
    state: State<'_, AppState>,
    path: String,
) -> Result<(), String> {
    let path = std::path::PathBuf::from(path);
    state.hf_manager.delete_local_model(&path).await
}

/// Recommended model info for frontend display
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct RecommendedModel {
    model_id: String,
    name: String,
    description: String,
}

/// Get recommended models for first-time users
#[tauri::command]
async fn hf_get_recommended_models(state: State<'_, AppState>) -> Result<Vec<RecommendedModel>, String> {
    let models = state.hf_manager.get_recommended_models().await;
    Ok(models.into_iter().map(|(id, name, desc)| RecommendedModel {
        model_id: id.to_string(),
        name: name.to_string(),
        description: desc.to_string(),
    }).collect())
}

/// Download statistics
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct DownloadStats {
    active: usize,
    completed: usize,
    total_downloaded: u64,
}

/// Get download statistics
#[tauri::command]
async fn hf_get_download_stats(state: State<'_, AppState>) -> Result<DownloadStats, String> {
    let (active, completed, total_downloaded) = state.hf_manager.get_download_stats().await;
    Ok(DownloadStats { active, completed, total_downloaded })
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

// ===== GPU Resource Commands =====

/// Get all detected GPU devices
#[tauri::command]
async fn gpu_get_devices(state: State<'_, AppState>) -> Result<Vec<GPUDevice>, String> {
    Ok(state.gpu_manager.get_devices().await)
}

/// Refresh GPU device detection
#[tauri::command]
async fn gpu_refresh_devices(state: State<'_, AppState>) -> Result<Vec<GPUDevice>, String> {
    Ok(state.gpu_manager.refresh_devices().await)
}

/// Get GPU allocation settings
#[tauri::command]
async fn gpu_get_settings(state: State<'_, AppState>) -> Result<GPUAllocationSettings, String> {
    Ok(state.gpu_manager.get_settings().await)
}

/// Update GPU allocation settings
#[tauri::command]
async fn gpu_update_settings(
    state: State<'_, AppState>,
    settings: GPUAllocationSettings,
) -> Result<(), String> {
    state.gpu_manager.update_settings(settings).await
}

/// Get GPU usage statistics
#[tauri::command]
async fn gpu_get_stats(state: State<'_, AppState>) -> Result<GPUStats, String> {
    Ok(state.gpu_manager.get_stats().await)
}

/// Get provider registration status
#[tauri::command]
async fn gpu_get_provider_status(state: State<'_, AppState>) -> Result<ProviderStatus, String> {
    Ok(state.gpu_manager.get_provider_status().await)
}

/// Submit a compute job to the queue
#[tauri::command]
async fn gpu_submit_job(
    state: State<'_, AppState>,
    job_type: ComputeJobType,
    model_id: String,
    input_hash: String,
    requester: String,
    max_payment: u64,
    memory_required: u64,
    estimated_time: u64,
    priority: u32,
) -> Result<String, String> {
    let job = ComputeJob {
        id: uuid::Uuid::new_v4().to_string(),
        job_type,
        model_id,
        input_hash,
        requester,
        max_payment,
        status: ComputeJobStatus::Queued,
        created_at: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
        memory_required,
        estimated_time,
        priority,
    };
    state.gpu_manager.submit_job(job).await
}

/// Get a specific compute job by ID
#[tauri::command]
async fn gpu_get_job(
    state: State<'_, AppState>,
    job_id: String,
) -> Result<Option<ComputeJob>, String> {
    Ok(state.gpu_manager.get_job(&job_id).await)
}

/// Get all compute jobs (active + queued)
#[tauri::command]
async fn gpu_get_all_jobs(state: State<'_, AppState>) -> Result<Vec<ComputeJob>, String> {
    Ok(state.gpu_manager.get_all_jobs().await)
}

/// Cancel a compute job
#[tauri::command]
async fn gpu_cancel_job(state: State<'_, AppState>, job_id: String) -> Result<(), String> {
    state.gpu_manager.cancel_job(&job_id).await
}

/// Get available GPU memory for compute
#[tauri::command]
async fn gpu_get_available_memory(state: State<'_, AppState>) -> Result<u64, String> {
    Ok(state.gpu_manager.get_available_compute_memory().await)
}

/// Check if GPU compute is within scheduled hours
#[tauri::command]
async fn gpu_is_within_schedule(state: State<'_, AppState>) -> Result<bool, String> {
    Ok(state.gpu_manager.is_within_schedule().await)
}

// ===== Image Model Commands =====

/// Get all image models
#[tauri::command]
async fn image_get_models(state: State<'_, AppState>) -> Result<Vec<ImageModel>, String> {
    Ok(state.image_model_manager.get_models().await)
}

/// Get a specific image model by ID
#[tauri::command]
async fn image_get_model(
    state: State<'_, AppState>,
    model_id: String,
) -> Result<Option<ImageModel>, String> {
    Ok(state.image_model_manager.get_model(&model_id).await)
}

/// Scan for local image models
#[tauri::command]
async fn image_scan_local_models(state: State<'_, AppState>) -> Result<Vec<ImageModel>, String> {
    Ok(state.image_model_manager.scan_local_models().await)
}

/// Create an image generation job
#[tauri::command]
async fn image_create_generation_job(
    state: State<'_, AppState>,
    model_id: String,
    prompt: String,
    negative_prompt: Option<String>,
    width: u32,
    height: u32,
    num_images: u32,
    seed: Option<u64>,
    guidance_scale: f32,
    num_steps: u32,
) -> Result<String, String> {
    let request = ImageGenerationRequest {
        model_id,
        prompt,
        negative_prompt,
        resolution: ImageResolution::new(width, height),
        num_images,
        seed,
        guidance_scale,
        num_steps,
        scheduler: ImageScheduler::EulerAncestral,
        input_image: None,
        strength: None,
        lora_weights: vec![],
    };
    state.image_model_manager.create_generation_job(request).await
}

/// Get generation job by ID
#[tauri::command]
async fn image_get_generation_job(
    state: State<'_, AppState>,
    job_id: String,
) -> Result<Option<GenerationJob>, String> {
    Ok(state.image_model_manager.get_generation_job(&job_id).await)
}

/// Get all generation jobs
#[tauri::command]
async fn image_get_generation_jobs(state: State<'_, AppState>) -> Result<Vec<GenerationJob>, String> {
    Ok(state.image_model_manager.get_generation_jobs().await)
}

/// Cancel a generation job
#[tauri::command]
async fn image_cancel_generation_job(
    state: State<'_, AppState>,
    job_id: String,
) -> Result<(), String> {
    state.image_model_manager.cancel_generation_job(&job_id).await
}

/// Create an image training job
#[tauri::command]
async fn image_create_training_job(
    state: State<'_, AppState>,
    config: ImageTrainingConfig,
) -> Result<String, String> {
    state.image_model_manager.create_training_job(config).await
}

/// Get training job by ID
#[tauri::command]
async fn image_get_training_job(
    state: State<'_, AppState>,
    job_id: String,
) -> Result<Option<ImageTrainingJob>, String> {
    Ok(state.image_model_manager.get_training_job(&job_id).await)
}

/// Get all training jobs
#[tauri::command]
async fn image_get_training_jobs(state: State<'_, AppState>) -> Result<Vec<ImageTrainingJob>, String> {
    Ok(state.image_model_manager.get_training_jobs().await)
}

/// Cancel a training job
#[tauri::command]
async fn image_cancel_training_job(
    state: State<'_, AppState>,
    job_id: String,
) -> Result<(), String> {
    state.image_model_manager.cancel_training_job(&job_id).await
}

/// Get generated images gallery
#[tauri::command]
async fn image_get_gallery(state: State<'_, AppState>) -> Result<Vec<GeneratedImage>, String> {
    Ok(state.image_model_manager.get_gallery().await)
}

/// Delete image from gallery
#[tauri::command]
async fn image_delete_from_gallery(
    state: State<'_, AppState>,
    image_id: String,
) -> Result<(), String> {
    state.image_model_manager.delete_from_gallery(&image_id).await
}

/// Get image models directory
#[tauri::command]
async fn image_get_models_dir(state: State<'_, AppState>) -> Result<String, String> {
    Ok(state.image_model_manager.get_models_dir().to_string_lossy().to_string())
}

/// Get image output directory
#[tauri::command]
async fn image_get_output_dir(state: State<'_, AppState>) -> Result<String, String> {
    Ok(state.image_model_manager.get_output_dir().to_string_lossy().to_string())
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
    let gpu_manager = Arc::new(GPUResourceManager::new());
    let image_model_manager = Arc::new(ImageModelManager::new());

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
            gpu_manager,
            image_model_manager,
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
            // Tracked addresses
            get_tracked_addresses,
            save_tracked_addresses,
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
            validate_password_strength,
            get_account,
            send_transaction,
            eth_call,
            sign_message,
            verify_signature,
            export_private_key,
            update_balance,
            // Session management commands
            get_session_remaining,
            is_session_active,
            lock_wallet,
            lock_all_wallets,
            check_password_required,
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
            // LoRA Training commands
            create_lora_job,
            start_lora_training,
            get_lora_job,
            get_lora_jobs,
            cancel_lora_job,
            delete_lora_job,
            get_lora_adapters,
            delete_lora_adapter,
            run_inference_with_lora,
            validate_dataset,
            get_lora_presets,
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
            // First-run and onboarding commands
            check_first_run,
            setup_bundled_model,
            get_onboarding_questions,
            process_onboarding_answer,
            skip_onboarding,
            // Secure API key management commands
            secure_store_api_key,
            validate_api_key_format,
            validate_stored_api_key,
            has_secure_api_key,
            delete_secure_api_key,
            load_secure_api_keys,
            get_secure_api_key_status,
            // Enhanced model download commands
            download_enhanced_model,
            check_enhanced_model_status,
            cancel_model_download,
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
            // Enhanced HuggingFace commands (OAuth PKCE, GGUF, Local Models)
            hf_start_auth_flow,
            hf_exchange_code_with_pkce,
            hf_search_gguf_models,
            hf_get_gguf_model,
            hf_scan_local_models,
            hf_auto_select_model,
            hf_download_file_resumable,
            hf_cancel_download_resumable,
            hf_delete_local_model,
            hf_get_recommended_models,
            hf_get_download_stats,
            // Foundry/Contract compilation commands
            forge_check_installed,
            forge_build,
            forge_init,
            forge_test,
            // GPU Resource commands
            gpu_get_devices,
            gpu_refresh_devices,
            gpu_get_settings,
            gpu_update_settings,
            gpu_get_stats,
            gpu_get_provider_status,
            gpu_submit_job,
            gpu_get_job,
            gpu_get_all_jobs,
            gpu_cancel_job,
            gpu_get_available_memory,
            gpu_is_within_schedule,
            // Image Model commands
            image_get_models,
            image_get_model,
            image_scan_local_models,
            image_create_generation_job,
            image_get_generation_job,
            image_get_generation_jobs,
            image_cancel_generation_job,
            image_create_training_job,
            image_get_training_job,
            image_get_training_jobs,
            image_cancel_training_job,
            image_get_gallery,
            image_delete_from_gallery,
            image_get_models_dir,
            image_get_output_dir,
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
