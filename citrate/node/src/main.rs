use anyhow::Result;
use clap::{Parser, Subcommand};
use citrate_api::{RpcConfig, RpcServer};
use citrate_consensus::crypto;
use citrate_execution::{Executor, StateDB};
use citrate_economics::{UnifiedEconomicsManager, UnifiedEconomicsConfig, StakeholderType};
use citrate_network::peer::PeerId;
use citrate_network::peer::{PeerManager, PeerManagerConfig};
use citrate_network::{NetworkTransport, GossipProtocol, GossipConfig, Discovery, DiscoveryConfig, SyncManager, SyncConfig};
use citrate_sequencer::mempool::{Mempool, MempoolConfig};
use citrate_storage::{pruning::PruningConfig, StorageManager};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, error, info, warn};
use tracing_subscriber::EnvFilter;

mod adapters;
mod artifact;
mod config;
mod genesis;
mod inference;
pub mod logging;
pub mod metrics;
mod model_manager;
mod model_verifier;
mod producer;
mod sync;

use config::NodeConfig;
use genesis::{initialize_genesis_state, GenesisConfig};
use producer::BlockProducer;

#[derive(Parser)]
#[command(name = "citrate")]
#[command(about = "Citrate blockchain node")]
struct Cli {
    /// Configuration file path
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,

    /// Data directory
    #[arg(short, long, value_name = "DIR")]
    data_dir: Option<PathBuf>,

    /// Enable mining
    #[arg(long)]
    mine: bool,

    /// P2P listen address (e.g., 0.0.0.0:30303)
    #[arg(long, value_name = "ADDR")]
    p2p_addr: Option<String>,

    /// Bootstrap nodes (can be specified multiple times)
    /// Format: peer_id@ip:port or ip:port
    #[arg(long, value_name = "NODE")]
    bootstrap_nodes: Vec<String>,

    /// RPC listen address (e.g., 127.0.0.1:8545)
    #[arg(long, value_name = "ADDR")]
    rpc_addr: Option<String>,

    /// Maximum number of peers
    #[arg(long, default_value = "50")]
    max_peers: usize,

    /// Chain ID
    #[arg(long, default_value = "1337")]
    chain_id: u64,

    /// Coinbase address for mining rewards (hex)
    #[arg(long)]
    coinbase: Option<String>,

    /// Disable RPC server
    #[arg(long)]
    no_rpc: bool,

    /// Run as bootstrap node (no active connections)
    #[arg(long)]
    bootstrap: bool,

    /// Subcommands
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new chain with genesis block
    Init {
        /// Chain ID
        #[arg(long, default_value = "1337")]
        chain_id: u64,
    },

    /// Run devnet with default configuration
    Devnet,

    /// Generate a new keypair for signing
    Keygen,

    /// Manage AI models (download, pin, list)
    Model {
        #[command(subcommand)]
        command: ModelCommands,
    },

    /// Show genesis block information
    GenesisInfo,
}

#[derive(Subcommand)]
enum ModelCommands {
    /// List all pinned models
    List,

    /// Show status of a specific model
    Status {
        /// IPFS CID of the model
        cid: String,
    },

    /// Manually pin a model by CID
    Pin {
        /// IPFS CID of the model to pin
        cid: String,
    },

    /// Unpin a model by CID
    Unpin {
        /// IPFS CID of the model to unpin
        cid: String,
    },

    /// Automatically pin all required models from genesis
    AutoPin {
        /// Data directory
        #[arg(short, long, value_name = "DIR")]
        data_dir: Option<PathBuf>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize structured logging
    // Uses LOG_FORMAT env var (json, pretty, compact) and RUST_LOG for levels
    let log_config = if std::env::var("LOG_FORMAT").map(|f| f == "json").unwrap_or(false) {
        logging::LogConfig::production()
    } else {
        logging::LogConfig::from_env()
    };

    if let Err(e) = logging::init_logging(&log_config) {
        // Fallback to basic logging if structured logging fails
        eprintln!("Warning: Failed to initialize structured logging: {}", e);
        tracing_subscriber::fmt()
            .with_env_filter(EnvFilter::from_default_env().add_directive("citrate=info".parse()?))
            .init();
    }

    let cli = Cli::parse();

    // Handle subcommands
    match cli.command {
        Some(Commands::Init { chain_id }) => {
            init_chain(chain_id).await?;
            return Ok(());
        }
        Some(Commands::Devnet) => {
            run_devnet().await?;
            return Ok(());
        }
        Some(Commands::Keygen) => {
            generate_keypair();
            return Ok(());
        }
        Some(Commands::Model { command }) => {
            handle_model_command(command, cli.data_dir.clone()).await?;
            return Ok(());
        }
        Some(Commands::GenesisInfo) => {
            show_genesis_info()?;
            return Ok(());
        }
        None => {
            // Run normal node
        }
    }

    // Load or create config
    let has_config_file = cli.config.is_some();
    let config = if let Some(config_path) = cli.config {
        NodeConfig::from_file(&config_path)?
    } else {
        NodeConfig::default()
    };

    // Override with CLI args
    let mut config = config;
    if let Some(data_dir) = cli.data_dir {
        config.storage.data_dir = data_dir;
    }
    if cli.mine {
        config.mining.enabled = true;
    }

    // Network configuration overrides
    if let Some(p2p_addr) = cli.p2p_addr {
        config.network.listen_addr = p2p_addr
            .parse()
            .map_err(|e| anyhow::anyhow!("Invalid P2P address: {}", e))?;
    }
    if !cli.bootstrap_nodes.is_empty() {
        config.network.bootstrap_nodes = cli.bootstrap_nodes;
    }
    if let Some(rpc_addr) = cli.rpc_addr {
        config.rpc.listen_addr = rpc_addr
            .parse()
            .map_err(|e| anyhow::anyhow!("Invalid RPC address: {}", e))?;
    }
    config.network.max_peers = cli.max_peers;

    // Only override chain_id if no config file was provided
    // This allows config file to set chain_id when using --config flag
    if !has_config_file {
        // No config file provided, use CLI arg (or its default)
        config.chain.chain_id = cli.chain_id;
    }

    if let Some(coinbase) = cli.coinbase {
        config.mining.coinbase = coinbase;
    }
    if cli.no_rpc {
        config.rpc.enabled = false;
    }

    // If running as bootstrap node, clear bootstrap_nodes list
    if cli.bootstrap {
        config.network.bootstrap_nodes.clear();
        info!(
            "Running as bootstrap node on {}",
            config.network.listen_addr
        );
    }

    // Initialize chain if data directory doesn't exist (first run)
    if !config.storage.data_dir.exists() {
        info!("Data directory doesn't exist, initializing genesis...");
        std::fs::create_dir_all(&config.storage.data_dir)?;

        let storage = Arc::new(StorageManager::new(
            &config.storage.data_dir,
            PruningConfig::default(),
        )?);

        let state_db = Arc::new(StateDB::new());
        let executor = Arc::new(Executor::with_storage(
            state_db,
            Some(storage.state.clone()),
        ));

        let genesis_config = genesis::GenesisConfig {
            chain_id: config.chain.chain_id,
            ..Default::default()
        };

        genesis::initialize_genesis_state(storage, executor, &genesis_config).await?;
        info!("Genesis state initialized for chain ID {}", config.chain.chain_id);
    }

    // Start node
    start_node(config).await
}

async fn handle_model_command(command: ModelCommands, data_dir: Option<PathBuf>) -> Result<()> {
    use model_manager::{ModelManager, ModelManagerConfig};

    let models_dir = data_dir.clone()
        .unwrap_or_else(|| dirs::home_dir().unwrap().join(".citrate"))
        .join("models");

    let config = ModelManagerConfig {
        models_dir: models_dir.clone(),
        ..Default::default()
    };

    let manager = ModelManager::new(config).await
        .map_err(|e| anyhow::anyhow!("Failed to create model manager: {}", e))?;

    match command {
        ModelCommands::List => {
            info!("Fetching list of pinned models...");
            let models = manager.list_pinned_models().await;

            if models.is_empty() {
                println!("No models currently pinned.");
                println!("Run 'citrate model auto-pin' to automatically pin required models.");
            } else {
                println!("\nPinned Models:");
                println!("{:-<100}", "");
                for model in models {
                    println!("Model ID: {}", model.model_id);
                    println!("CID:      {}", model.cid);
                    println!("Size:     {} MB", model.size_bytes / 1_000_000);
                    println!("Path:     {}", model.file_path.display());
                    println!("Status:   {:?}", model.status);
                    println!("{:-<100}", "");
                }
            }
        }

        ModelCommands::Status { cid } => {
            info!("Checking status of model: {}", cid);
            match manager.get_model_status(&cid).await {
                Some(status) => {
                    println!("Model CID: {}", cid);
                    println!("Status: {:?}", status);

                    if let Some(path) = manager.get_model_path(&cid).await {
                        println!("Path: {}", path.display());
                    }
                }
                None => {
                    println!("Model {} is not pinned.", cid);
                    println!("Run 'citrate model pin {}' to pin it.", cid);
                }
            }
        }

        ModelCommands::Pin { cid } => {
            println!("Manually pinning model {} is not yet implemented.", cid);
            println!("Please use 'citrate model auto-pin' to pin required models from genesis.");
        }

        ModelCommands::Unpin { cid } => {
            info!("Unpinning model: {}", cid);
            manager.unpin_model(&cid).await
                .map_err(|e| anyhow::anyhow!("Failed to unpin model: {}", e))?;
            println!("Successfully unpinned model {}", cid);
        }

        ModelCommands::AutoPin { data_dir: cmd_data_dir } => {
            let data_dir = cmd_data_dir
                .or(data_dir)
                .unwrap_or_else(|| dirs::home_dir().unwrap().join(".citrate"));

            info!("Initializing genesis to get required models...");

            // Load genesis config to get required models
            let genesis_config = GenesisConfig {
                timestamp: 0,
                chain_id: 1337,
                initial_accounts: vec![],
            };

            let genesis_block = genesis::create_genesis_block(&genesis_config);

            if genesis_block.required_pins.is_empty() {
                println!("No required models found in genesis block.");
                return Ok(());
            }

            println!("\nRequired Models from Genesis:");
            for model in &genesis_block.required_pins {
                println!("  - {} (CID: {}, Size: {} MB)",
                    model.model_id.0,
                    model.ipfs_cid,
                    model.size_bytes / 1_000_000
                );
            }

            println!("\nChecking IPFS daemon...");
            if let Err(e) = manager.check_ipfs_daemon().await {
                eprintln!("Error: {}", e);
                println!("\nPlease ensure IPFS is installed and running:");
                println!("  1. Install IPFS: https://docs.ipfs.tech/install/");
                println!("  2. Start daemon: ipfs daemon");
                return Err(anyhow::anyhow!("IPFS daemon not available"));
            }

            println!("IPFS daemon is running ✓\n");
            println!("Starting automatic model pinning...");
            println!("This may take a while for large models (up to {} MB total)\n",
                genesis_block.required_pins.iter().map(|m| m.size_bytes).sum::<u64>() / 1_000_000
            );

            manager.auto_pin_required_models(&genesis_block.required_pins).await
                .map_err(|e| anyhow::anyhow!("Failed to auto-pin models: {}", e))?;

            println!("\n✓ All required models have been pinned successfully!");
            println!("Models stored in: {}", models_dir.display());
        }
    }

    Ok(())
}

async fn init_chain(chain_id: u64) -> Result<()> {
    info!("Initializing new chain with ID {}", chain_id);

    let temp_dir = PathBuf::from(".citrate");
    std::fs::create_dir_all(&temp_dir)?;

    // Create storage
    let storage = Arc::new(StorageManager::new(&temp_dir, PruningConfig::default())?);

    // Create executor with persistent storage
    let state_db = Arc::new(StateDB::new());
    let executor = Arc::new(Executor::with_storage(
        state_db,
        Some(storage.state.clone()),
    ));

    // Initialize genesis
    let genesis_config = GenesisConfig {
        chain_id,
        ..Default::default()
    };

    let genesis_hash = initialize_genesis_state(storage.clone(), executor, &genesis_config).await?;

    info!(
        "Genesis block created: {:?}",
        hex::encode(&genesis_hash.as_bytes()[..8])
    );
    info!("Chain initialized in {:?}", temp_dir);

    Ok(())
}

async fn run_devnet() -> Result<()> {
    info!("Starting devnet...");

    let mut config = NodeConfig::devnet();
    config.storage.data_dir = PathBuf::from(".citrate-devnet");

    // Initialize chain if needed
    if !config.storage.data_dir.exists() {
        std::fs::create_dir_all(&config.storage.data_dir)?;

        let storage = Arc::new(StorageManager::new(
            &config.storage.data_dir,
            PruningConfig::default(),
        )?);

        let state_db = Arc::new(StateDB::new());
        let executor = Arc::new(Executor::with_storage(
            state_db,
            Some(storage.state.clone()),
        ));

        let genesis_config = GenesisConfig {
            chain_id: config.chain.chain_id,
            ..Default::default()
        };

        initialize_genesis_state(storage, executor, &genesis_config).await?;
        info!("Devnet chain initialized");
    }

    // Start node with devnet config
    start_node(config).await
}

fn generate_keypair() {
    let signing_key = crypto::generate_keypair();
    let verifying_key = signing_key.verifying_key();

    println!("New keypair generated:");
    println!("Private key: {}", hex::encode(signing_key.to_bytes()));
    println!("Public key:  {}", hex::encode(verifying_key.to_bytes()));
}

fn show_genesis_info() -> Result<()> {
    println!("=========================================");
    println!("Genesis Block Information");
    println!("=========================================");
    println!();

    info!("Creating genesis block...");
    let genesis_config = genesis::GenesisConfig {
        timestamp: 0,
        chain_id: 1337,
        initial_accounts: vec![],
    };

    let genesis = genesis::create_genesis_block(&genesis_config);

    println!("Block Details:");
    println!("  Height: {}", genesis.header.height);
    println!("  Timestamp: {}", genesis.header.timestamp);
    println!("  Chain ID: {}", genesis_config.chain_id);
    println!("  Block Hash: {}", hex::encode(genesis.header.block_hash.as_bytes()));
    println!();

    // Embedded models
    println!("Embedded Models ({}):", genesis.embedded_models.len());
    let mut total_embedded_size = 0u64;
    for model in &genesis.embedded_models {
        let size_bytes = model.size_bytes();
        let size_mb = size_bytes as f64 / (1024.0 * 1024.0);
        total_embedded_size += size_bytes as u64;
        println!("  - Model ID: {}", model.model_id);
        println!("    Type: {:?}", model.model_type);
        println!("    Size: {:.2} MB ({} bytes)", size_mb, size_bytes);
        println!("    Metadata: {} v{}", model.metadata.name, model.metadata.version);
        println!();
    }

    let total_embedded_mb = total_embedded_size as f64 / (1024.0 * 1024.0);
    println!("Total Embedded Size: {:.2} MB ({} bytes)", total_embedded_mb, total_embedded_size);
    println!();

    // Required pins (IPFS models)
    println!("Required IPFS Pins ({}):", genesis.required_pins.len());
    let mut total_ipfs_size = 0u64;
    for pin in &genesis.required_pins {
        let size_mb = pin.size_bytes as f64 / (1024.0 * 1024.0);
        let size_gb = size_mb / 1024.0;
        total_ipfs_size += pin.size_bytes;

        println!("  - Model ID: {}", pin.model_id);
        println!("    IPFS CID: {}", pin.ipfs_cid);
        println!("    Size: {:.2} GB ({:.2} MB)", size_gb, size_mb);
        println!("    SHA256: {}", hex::encode(pin.sha256_hash.as_bytes()));
        println!();
    }

    let total_ipfs_gb = total_ipfs_size as f64 / (1024.0 * 1024.0 * 1024.0);
    println!("Total IPFS Required: {:.2} GB", total_ipfs_gb);
    println!();

    // Overall summary
    println!("=========================================");
    println!("Summary:");
    println!("  Embedded in genesis: {:.2} MB", total_embedded_mb);
    println!("  Required to pin: {:.2} GB", total_ipfs_gb);
    println!("  Total AI models: {} embedded + {} IPFS", genesis.embedded_models.len(), genesis.required_pins.len());
    println!("=========================================");

    Ok(())
}

async fn start_node(config: NodeConfig) -> Result<()> {
    info!("Starting Citrate node...");
    info!("Chain ID: {}", config.chain.chain_id);
    info!("Data directory: {:?}", config.storage.data_dir);

    // Initialize metrics server
    let metrics_addr = std::env::var("CITRATE_METRICS_ADDR")
        .unwrap_or_else(|_| "127.0.0.1:9090".to_string());
    if let Err(e) = metrics::init_metrics(&metrics_addr) {
        warn!("Failed to initialize metrics server: {}", e);
    } else {
        info!("Metrics server started on http://{}/metrics", metrics_addr);
    }

    // Record node start time for uptime tracking
    let node_start_time = std::time::Instant::now();

    // Create storage
    let storage = Arc::new(StorageManager::new(
        &config.storage.data_dir,
        PruningConfig {
            keep_blocks: config.storage.keep_blocks,
            keep_states: config.storage.keep_blocks,
            interval: Duration::from_secs(3600),
            batch_size: 1000,
            auto_prune: config.storage.pruning,
        },
    )?);

    // Create state DB and executor with persistent storage
    let state_db = Arc::new(StateDB::new());
    let state_manager = Arc::new(citrate_storage::state_manager::StateManager::new(storage.db.clone()));

    // Load existing state from storage into memory
    info!("Loading state from storage...");
    match storage.state.get_all_accounts() {
        Ok(accounts) => {
            info!("Found {} accounts in storage, loading into memory...", accounts.len());
            for (address, account) in accounts {
                debug!("Loaded account: 0x{} with balance {}", hex::encode(address.0), account.balance);
                state_db.accounts.set_account(address, account);
            }
            info!("State loaded successfully");
        }
        Err(e) => {
            warn!("Failed to load accounts from storage: {}", e);
        }
    }
    // MCP + inference service
    let vm_for_mcp = Arc::new(citrate_execution::vm::VM::new(10_000_000));
    let mcp = Arc::new(citrate_mcp::MCPService::new(
        storage.clone(),
        vm_for_mcp.clone(),
    ));
    // Provider address from config.mining.coinbase (hex 0x...)
    let provider_addr = {
        let mut a = [0u8; 20];
        let s = config.mining.coinbase.trim_start_matches("0x");
        if let Ok(bytes) = hex::decode(s) {
            if bytes.len() >= 20 {
                a.copy_from_slice(&bytes[..20]);
            }
        }
        citrate_execution::types::Address(a)
    };
    // Flat provider fee = 0.01 LATT (1e16 wei)
    let provider_fee = primitive_types::U256::from(10u128.pow(16));
    let inf_svc = Arc::new(crate::inference::NodeInferenceService::new(
        mcp.clone(),
        provider_addr,
        provider_fee,
    ));

    // Artifact service with governance provider list override
    let gov_addr = {
        let mut a = [0u8; 20];
        a[18] = 0x10;
        a[19] = 0x03;
        citrate_execution::types::Address(a)
    };
    let providers_from_gov: Option<Vec<String>> = state_db
        .get_storage(&gov_addr, b"PARAM:ipfs_providers")
        .and_then(|bytes| String::from_utf8(bytes).ok())
        .map(|s| {
            s.split(',')
                .map(|x| x.trim().to_string())
                .filter(|x| !x.is_empty())
                .collect()
        });
    let art_svc = if let Some(providers) = providers_from_gov {
        Arc::new(crate::artifact::NodeArtifactService::new_with_providers(
            providers,
        ))
    } else {
        let ipfs_api = std::env::var("CITRATE_IPFS_API").ok();
        Arc::new(crate::artifact::NodeArtifactService::new(ipfs_api))
    };

    let storage_bridge: Arc<dyn citrate_execution::executor::AIModelStorage> =
        Arc::new(adapters::StorageAdapter::new(state_manager.clone()));
    let registry_bridge: Arc<dyn citrate_execution::executor::ModelRegistryAdapter> =
        Arc::new(adapters::MCPRegistryBridge::new(mcp.clone()));

    let exec_base = Executor::with_storage(state_db, Some(storage.state.clone()));
    let executor = Arc::new(
        exec_base
            .with_ai_storage_adapter(storage_bridge)
            .with_model_registry_adapter(registry_bridge)
            .with_inference_service(inf_svc)
            .with_artifact_service(art_svc),
    );

    // Governance params: read min_gas_price override
    let governance_addr = {
        let mut a = [0u8; 20];
        a[18] = 0x10;
        a[19] = 0x03;
        citrate_execution::types::Address(a)
    };
    let mut min_gas_price_override: Option<u64> = None;
    if let Some(bytes) = executor
        .state_db()
        .get_storage(&governance_addr, b"PARAM:min_gas_price")
    {
        if bytes.len() >= 8 {
            let mut arr = [0u8; 8];
            arr.copy_from_slice(&bytes[..8]);
            min_gas_price_override = Some(u64::from_le_bytes(arr));
        } else if bytes.len() >= 4 {
            // support 32-bit little endian as fallback
            let mut arr = [0u8; 4];
            arr.copy_from_slice(&bytes[..4]);
            min_gas_price_override = Some(u32::from_le_bytes(arr) as u64);
        }
    }

    // Mempool config from env overrides
    let require_valid_signature = std::env::var("CITRATE_REQUIRE_VALID_SIGNATURE")
        .ok()
        .and_then(|v| {
            let s = v.to_lowercase();
            match s.as_str() {
                "1" | "true" | "yes" | "on" => Some(true),
                "0" | "false" | "no" | "off" => Some(false),
                _ => None,
            }
        })
        .unwrap_or({
            // Default to false in devnet mode for easier testing
            #[cfg(feature = "devnet")]
            {
                false
            }
            #[cfg(not(feature = "devnet"))]
            {
                true
            }
        });

    // Create mempool
    let mempool = Arc::new(Mempool::new(MempoolConfig {
        max_size: 10000,
        max_per_sender: 100,
        min_gas_price: min_gas_price_override.unwrap_or(config.mining.min_gas_price),
        tx_expiry_secs: 3600,
        allow_replacement: true,
        replacement_factor: 110,
        require_valid_signature,
        chain_id: config.chain.chain_id,
    }));

    // Create peer manager
    let peer_manager = Arc::new(PeerManager::new(PeerManagerConfig {
        max_peers: config.network.max_peers,
        max_inbound: config.network.max_peers / 2,
        max_outbound: config.network.max_peers / 2,
        peer_timeout: std::time::Duration::from_secs(30),
        ban_duration: std::time::Duration::from_secs(3600),
        score_threshold: -100,
    }));

    // Optionally start Prometheus metrics server
    let metrics_enabled = std::env::var("CITRATE_METRICS")
        .map(|v| matches!(v.to_lowercase().as_str(), "1" | "true" | "yes" | "on"))
        .unwrap_or(false);
    if metrics_enabled {
        let addr_str =
            std::env::var("CITRATE_METRICS_ADDR").unwrap_or_else(|_| "0.0.0.0:9100".to_string());
        let addr: std::net::SocketAddr = addr_str.parse().unwrap();
        tokio::spawn(async move {
            if let Err(e) = citrate_api::metrics_server::MetricsServer::new(addr)
                .start()
                .await
            {
                tracing::warn!("Metrics server failed: {}", e);
            }
        });
        info!("Metrics server enabled at {}", addr);
    }

    // Start P2P listener and connect to bootstrap nodes
    {
        // Prepare head info
        let head_height = storage.blocks.get_latest_height().unwrap_or(0);
        let head_hash = if head_height > 0 {
            storage
                .blocks
                .get_block_by_height(head_height)
                .ok()
                .flatten()
                .unwrap_or_default()
        } else {
            citrate_consensus::types::Hash::default()
        };
        let genesis_hash = storage
            .blocks
            .get_block_by_height(0)
            .ok()
            .flatten()
            .unwrap_or_default();
        let network_id: u32 = config.chain.chain_id as u32;

        // Incoming message channel (log-only for now)
        let (in_tx, mut in_rx) =
            tokio::sync::mpsc::channel::<(PeerId, citrate_network::NetworkMessage)>(512);
        peer_manager.set_incoming(in_tx).await;
        let pm_for_rx = peer_manager.clone();
        let storage_for_handler = storage.clone();
        let mempool_for_handler = mempool.clone();
        let gossip = Arc::new(GossipProtocol::new(GossipConfig::default(), peer_manager.clone()));
        let gossip_for_rx = gossip.clone();
        // Sync manager (basic integration)
        let sync = Arc::new(SyncManager::new(SyncConfig::default()));
        let sync_for_rx = sync.clone();

        // Start transport listener and connect to bootstrap nodes
        let local_peer_id = load_or_create_peer_id(&config.storage.data_dir)?;
        let transport = NetworkTransport::new(
            peer_manager.clone(),
            local_peer_id,
            citrate_network::transport::HandshakeParams {
                network_id,
                genesis_hash,
                head_height,
                head_hash,
            },
        );
        let listen_addr = config.network.listen_addr;
        transport
            .start_listener(listen_addr)
            .await
            .map_err(|e| anyhow::anyhow!(format!("Failed to start P2P listener: {}", e)))?;

        // Dial configured bootstrap nodes (ip:port or peer@ip:port)
        for s in &config.network.bootstrap_nodes {
            if let Some((_pid, addr)) = parse_bootnode(s) {
                let _ = transport.connect_to(addr).await;
                continue;
            }
            // Try numeric IP:port
            if let Ok(addr) = s.parse() {
                let _ = transport.connect_to(addr).await;
                continue;
            }
            // Resolve hostname:port
            if let Ok(mut addrs) = tokio::net::lookup_host(s).await {
                if let Some(addr) = addrs.next() {
                    let _ = transport.connect_to(addr).await;
                }
            } else {
                tracing::warn!("Could not resolve bootstrap node: {}", s);
            }
        }

        // Start discovery and periodic dialer
        let discovery = Arc::new(Discovery::new(
            DiscoveryConfig {
                bootstrap_nodes: config.network.bootstrap_nodes.clone(),
                max_peers: config.network.max_peers,
                ..Default::default()
            },
            peer_manager.clone(),
        ));
        discovery.init().await.ok();

        let discovery_for_loop = discovery.clone();
        let transport_for_loop = transport;
        let pm_for_discovery = pm_for_rx.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(10));
            loop {
                interval.tick().await;
                let candidates = discovery_for_loop.find_peers().await;
                for (id, addr) in candidates {
                    match transport_for_loop.connect_to(addr).await {
                        Ok(_) => {
                            discovery_for_loop.mark_connected(&id).await;
                            discovery_for_loop.update_attempts(&id, true).await;
                        }
                        Err(_) => {
                            discovery_for_loop.update_attempts(&id, false).await;
                        }
                    }
                }
                // Periodically request peer lists
                let _ = pm_for_discovery
                    .broadcast(&citrate_network::NetworkMessage::GetPeers)
                    .await;
            }
        });

        // Periodic sync tick: request headers/blocks and check timeouts
        let pm_for_sync = pm_for_rx.clone();
        let sync_for_loop = sync.clone();
        let storage_for_sync = storage.clone();
        tokio::spawn(async move {
            use std::collections::HashMap;
            use std::time::{Duration, Instant};
            let mut attempt_counts: HashMap<citrate_consensus::types::Hash, u32> = HashMap::new();
            let mut pending_retries: Vec<(Instant, citrate_consensus::types::Hash)> = Vec::new();
            let mut peer_failures: HashMap<String, u32> = HashMap::new();
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(2));
            loop {
                interval.tick().await;
                let peers = pm_for_sync.get_all_peers();
                // Pick best peer by head height
                let mut best: Option<Arc<citrate_network::peer::Peer>> = None;
                let mut best_h: u64 = 0;
                for p in peers {
                    let info = p.info.read().await;
                    if info.state == citrate_network::peer::PeerState::Connected
                        && info.head_height > best_h
                        && peer_failures.get(&info.id.0).cloned().unwrap_or(0) < 3
                    {
                        best_h = info.head_height;
                        best = Some(p.clone());
                    }
                }
                if let Some(peer) = best {
                    // Determine current local head hash
                    let start_from = if let Some(h) = sync_for_loop.last_requested_header().await {
                        h
                    } else if let Some(h) = sync_for_loop.last_received_header().await {
                        h
                    } else {
                        let local_h = storage_for_sync.blocks.get_latest_height().unwrap_or(0);
                        if local_h > 0 {
                            storage_for_sync
                                .blocks
                                .get_block_by_height(local_h)
                                .ok()
                                .flatten()
                                .unwrap_or_else(|| citrate_consensus::types::Hash::new([0u8; 32]))
                        } else {
                            citrate_consensus::types::Hash::new([0u8; 32])
                        }
                    };
                    // Request next headers and blocks from our last known point only if not saturated
                    let (ph, pb) = sync_for_loop.pending_counts().await;
                    if ph < 8 {
                        let _ = sync_for_loop.request_headers(&peer, start_from).await;
                    }
                    if pb < 8 {
                        let _ = sync_for_loop.request_blocks(&peer, start_from).await;
                    }
                }
                // Requeue timed-out requests with exponential backoff
                for (h, pid) in sync_for_loop.check_timeouts().await {
                    let entry = attempt_counts.entry(h).or_insert(0);
                    *entry = entry.saturating_add(1);
                    let backoff = (*entry).min(5); // cap exponent at 5
                    let delay_secs = 1u64 << backoff; // 2,4,8,16,32
                    pending_retries.push((Instant::now() + Duration::from_secs(delay_secs), h));
                    // Penalize the peer that timed out
                    let key = pid.0.clone();
                    let pf = peer_failures.entry(key.clone()).or_insert(0);
                    *pf = pf.saturating_add(1);
                    // Lower peer score
                    pm_for_sync.update_peer_score(&pid, -5).await;
                    // Remove peer if too many failures
                    if *pf >= 5 {
                        if let Some(p) = pm_for_sync.get_peer(&pid) {
                            let addr = p.info.read().await.addr;
                            pm_for_sync.remove_peer(&pid).await;
                            pm_for_sync.ban_peer(addr).await;
                            tracing::warn!("Banned peer {} due to repeated sync timeouts", pid.0);
                        }
                    }
                }
                // Issue any due retries
                let now = Instant::now();
                let mut remaining: Vec<(Instant, citrate_consensus::types::Hash)> = Vec::new();
                for (when, h) in pending_retries.drain(..) {
                    if when <= now {
                        if let Some(peer) = pm_for_sync.get_all_peers().first() {
                            let _ = sync_for_loop.request_headers(peer, h).await;
                            let _ = sync_for_loop.request_blocks(peer, h).await;
                        }
                    } else {
                        remaining.push((when, h));
                    }
                }
                pending_retries = remaining;
            }
        });
        tokio::spawn(async move {
            use citrate_consensus::types::Hash;
            use citrate_network::NetworkMessage;
            use citrate_sequencer::mempool::TxClass;
            while let Some((pid, msg)) = in_rx.recv().await {
                tracing::debug!("[P2P] from={} msg={:?}", pid.0, msg);
                // Handle protocol messages
                match msg {
                    NetworkMessage::Hello { head_height, head_hash, .. } => {
                        // Kick off naive sync: request blocks from genesis if behind
                        let local_h = storage_for_handler
                            .blocks
                            .get_latest_height()
                            .unwrap_or(0);
                        if head_height > local_h {
                            let _ = sync_for_rx.start_sync(head_height, head_hash).await;
                        }
                        // Also request headers
                        // Sync manager will request in periodic loop
                    }
                    NetworkMessage::HelloAck { head_height, head_hash, .. } => {
                        let local_h = storage_for_handler
                            .blocks
                            .get_latest_height()
                            .unwrap_or(0);
                        if head_height > local_h {
                            let _ = sync_for_rx.start_sync(head_height, head_hash).await;
                        }
                        // Requests are driven by periodic sync loop
                    }
                    NetworkMessage::GetBlocks { from, count, .. } => {
                        tracing::info!("Received GetBlocks request from peer {} for {} blocks starting from {:?}", 
                                     pid.0, count, from);
                        let mut blocks = Vec::new();

                        // Handle genesis request (zero hash)
                        if from == Hash::new([0u8; 32]) {
                            tracing::info!("Serving blocks from genesis");
                            let mut h = 0u64;
                            let end_h = count as u64;
                            while h < end_h && blocks.len() < count as usize {
                                if let Ok(Some(hash)) =
                                    storage_for_handler.blocks.get_block_by_height(h)
                                {
                                    if let Ok(Some(block)) =
                                        storage_for_handler.blocks.get_block(&hash)
                                    {
                                        blocks.push(block);
                                    }
                                }
                                h += 1;
                            }
                        } else {
                            // Get blocks after the specified hash
                            if let Ok(Some(start_block)) =
                                storage_for_handler.blocks.get_block(&from)
                            {
                                let start_h = start_block.header.height + 1;
                                let end_h = start_h.saturating_add(count as u64);
                                let mut h = start_h;
                                while h < end_h && blocks.len() < count as usize {
                                    if let Ok(Some(hash)) =
                                        storage_for_handler.blocks.get_block_by_height(h)
                                    {
                                        if let Ok(Some(block)) =
                                            storage_for_handler.blocks.get_block(&hash)
                                        {
                                            blocks.push(block);
                                        }
                                    }
                                    h += 1;
                                }
                            }
                        }

                        tracing::info!("Sending {} blocks to peer {}", blocks.len(), pid.0);
                        let _ = pm_for_rx
                            .send_to_peers(&[pid.clone()], &NetworkMessage::Blocks { blocks })
                            .await;
                    }
                    NetworkMessage::GetPeers => {
                        // Serve a small list of peers from discovery
                        let peers = discovery.get_peers_for_exchange().await;
                        let _ = pm_for_rx
                            .send_to_peers(&[pid.clone()], &NetworkMessage::Peers { peers })
                            .await;
                    }
                    NetworkMessage::Peers { peers } => {
                        discovery.handle_peer_exchange(peers).await;
                    }
                    NetworkMessage::GetHeaders { from, count } => {
                        tracing::info!(
                            "Received GetHeaders request from peer {} starting {:?} count {}",
                            pid.0, from, count
                        );
                        let mut headers = Vec::new();
                        if from == Hash::new([0u8; 32]) {
                            let mut h = 0u64;
                            while headers.len() < count as usize {
                                if let Ok(Some(hash)) =
                                    storage_for_handler.blocks.get_block_by_height(h)
                                {
                                    if let Ok(Some(block)) =
                                        storage_for_handler.blocks.get_block(&hash)
                                    {
                                        headers.push(block.header);
                                    }
                                }
                                h += 1;
                            }
                        } else if let Ok(Some(start_block)) =
                            storage_for_handler.blocks.get_block(&from)
                        {
                            let mut h = start_block.header.height + 1;
                            while headers.len() < count as usize {
                                if let Ok(Some(hash)) =
                                    storage_for_handler.blocks.get_block_by_height(h)
                                {
                                    if let Ok(Some(block)) =
                                        storage_for_handler.blocks.get_block(&hash)
                                    {
                                        headers.push(block.header);
                                    }
                                }
                                h += 1;
                            }
                        }
                        let _ = pm_for_rx
                            .send_to_peers(
                                &[pid.clone()],
                                &NetworkMessage::Headers { headers },
                            )
                            .await;
                    }
                    NetworkMessage::Headers { headers } => {
                        let _ = sync_for_rx.handle_headers(headers).await;
                    }
                    NetworkMessage::GetTransactions { hashes } => {
                        let mut txs = Vec::new();
                        for h in hashes {
                            if let Some(tx) = mempool_for_handler.get_transaction(&h).await {
                                txs.push(tx);
                            }
                        }
                        let _ = pm_for_rx
                            .send_to_peers(
                                &[pid.clone()],
                                &NetworkMessage::Transactions { transactions: txs },
                            )
                            .await;
                    }
                    NetworkMessage::NewTransaction { transaction } => {
                        // Add to mempool if not already present
                        if !mempool_for_handler.contains(&transaction.hash).await {
                            let _ = mempool_for_handler
                                .add_transaction(transaction.clone(), TxClass::Standard)
                                .await;
                        }
                        // Let gossip handle validation + propagation
                        let _ = gossip_for_rx
                            .handle_new_transaction(transaction, &pid)
                            .await;
                    }
                    NetworkMessage::NewBlock { block } => {
                        // Store block if we don't have it
                        let have = storage_for_handler
                            .blocks
                            .has_block(&block.header.block_hash)
                            .unwrap_or(false);
                        if !have {
                            let _ = storage_for_handler.blocks.put_block(&block);
                        }
                        // Let gossip propagate
                        let _ = gossip_for_rx.handle_new_block(block, &pid).await;
                    }
                    NetworkMessage::Blocks { blocks } => {
                        let _ = sync_for_rx.handle_blocks(blocks).await;
                    }
                    NetworkMessage::Transactions { transactions } => {
                        for tx in transactions {
                            let _ = mempool_for_handler
                                .add_transaction(tx, TxClass::Standard)
                                .await;
                        }
                    }
                    _ => {
                        // Other messages not handled yet
                    }
                }
            }
        });

        // (legacy direct socket bootstrap removed; handled by NetworkTransport)
    }

    // Create unified economics manager for RPC and mining
    let economics_config = UnifiedEconomicsConfig::default();
    let mut economics_manager_temp = UnifiedEconomicsManager::new(economics_config);

    // Register initial stakeholders
    let coinbase_bytes = hex::decode(&config.mining.coinbase).unwrap_or_else(|_| vec![0; 32]);
    let mut coinbase = [0u8; 32];
    coinbase.copy_from_slice(&coinbase_bytes[..32.min(coinbase_bytes.len())]);
    let validator_address = citrate_execution::types::Address(coinbase[0..20].try_into().unwrap_or([0; 20]));
    let _ = economics_manager_temp.register_stakeholder(validator_address, StakeholderType::Validator);

    let economics_manager = Arc::new(economics_manager_temp);

    // Start RPC server if enabled
    let rpc_handle = if config.rpc.enabled {
        info!("Starting RPC server on {}", config.rpc.listen_addr);

        let rpc_config = RpcConfig {
            listen_addr: config.rpc.listen_addr,
            max_connections: 100,
            cors_domains: vec!["*".to_string()],
            threads: 4,
        };

        let rpc_server = RpcServer::with_economics(
            rpc_config,
            storage.clone(),
            mempool.clone(),
            peer_manager.clone(),
            executor.clone(),
            config.chain.chain_id,
            Some(economics_manager.clone()),
        );

        Some(tokio::spawn(async move {
            match rpc_server.spawn() {
                Ok((close_handle, join_handle)) => {
                    info!("RPC server started");
                    // Keep server alive
                    tokio::signal::ctrl_c().await.ok();
                    // Signal server to close and join its OS thread
                    close_handle.close();
                    tokio::task::spawn_blocking(move || {
                        let _ = join_handle.join();
                    })
                    .await
                    .ok();
                }
                Err(e) => {
                    error!("Failed to start RPC server: {}", e);
                }
            }
        }))
    } else {
        None
    };

    // Start block producer if mining is enabled
    if config.mining.enabled {
        info!("Starting block producer...");

        // Parse coinbase address
        let coinbase_bytes = hex::decode(&config.mining.coinbase).unwrap_or_else(|_| vec![0; 32]);
        let mut coinbase = [0u8; 32];
        coinbase.copy_from_slice(&coinbase_bytes[..32.min(coinbase_bytes.len())]);

        // Always use peer manager if we have one (network is already setup above)
        let producer_peer_manager = Some(peer_manager.clone());

        // Treasury percentage from governance
        let mut _treasury_percentage = 10u8;
        if let Some(bytes) = executor
            .state_db()
            .get_storage(&governance_addr, b"PARAM:treasury_percentage")
        {
            if !bytes.is_empty() {
                _treasury_percentage = bytes[0];
            }
        }

        // Use the economics manager created earlier
        let producer = Arc::new(BlockProducer::with_economics(
            storage.clone(),
            executor.clone(),
            mempool.clone(),
            producer_peer_manager,
            citrate_consensus::PublicKey::new(coinbase),
            config.mining.target_block_time,
            economics_manager,
        ));

        tokio::spawn(async move {
            producer.start().await;
        });

        info!("Block producer started");
    }

    // Wait for shutdown signal
    tokio::signal::ctrl_c().await?;
    info!("Shutting down...");

    // Wait for RPC to shut down
    if let Some(handle) = rpc_handle {
        handle.abort();
    }

    Ok(())
}

fn load_or_create_peer_id(data_dir: &std::path::Path) -> anyhow::Result<citrate_network::peer::PeerId> {
    use std::fs;
    use std::io::Write;
    let path = data_dir.join("peer.id");
    if let Ok(s) = fs::read_to_string(&path) {
        let id = s.trim().to_string();
        if !id.is_empty() {
            return Ok(citrate_network::peer::PeerId::new(id));
        }
    }
    let id = format!("peer_{}", rand::random::<u64>());
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let mut f = fs::File::create(&path)?;
    writeln!(f, "{}", id)?;
    Ok(citrate_network::peer::PeerId::new(id))
}

/// Parse bootnode strings in formats like:
/// - peer123@203.0.113.10:30303
/// - 203.0.113.10:30303 (peer id will be generated)
fn parse_bootnode(s: &str) -> Option<(PeerId, std::net::SocketAddr)> {
    let (peer_part, addr_part) = if let Some((pid, rest)) = s.split_once('@') {
        (Some(pid.trim()), rest.trim())
    } else {
        (None, s.trim())
    };
    let addr: std::net::SocketAddr = addr_part.parse().ok()?;
    let peer_id = peer_part
        .map(|p| PeerId::new(p.to_string()))
        .unwrap_or_else(PeerId::random);
    Some((peer_id, addr))
}
