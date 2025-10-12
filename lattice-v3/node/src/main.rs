use anyhow::Result;
use clap::{Parser, Subcommand};
use lattice_api::{RpcConfig, RpcServer};
use lattice_consensus::crypto;
use lattice_execution::{Executor, StateDB};
use lattice_network::peer::PeerId;
use lattice_network::peer::{PeerManager, PeerManagerConfig};
use lattice_sequencer::mempool::{Mempool, MempoolConfig};
use lattice_storage::{pruning::PruningConfig, StorageManager};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tracing::{error, info, warn};
use tracing_subscriber::EnvFilter;

mod adapters;
mod artifact;
mod config;
mod genesis;
mod inference;
mod producer;

use config::NodeConfig;
use genesis::{initialize_genesis_state, GenesisConfig};
use producer::BlockProducer;

#[derive(Parser)]
#[command(name = "lattice")]
#[command(about = "Lattice blockchain node")]
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
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("lattice=info".parse()?))
        .init();

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
        None => {
            // Run normal node
        }
    }

    // Load or create config
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
    config.chain.chain_id = cli.chain_id;

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

    // Start node
    start_node(config).await
}

async fn init_chain(chain_id: u64) -> Result<()> {
    info!("Initializing new chain with ID {}", chain_id);

    let temp_dir = PathBuf::from(".lattice");
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
    config.storage.data_dir = PathBuf::from(".lattice-devnet");

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

async fn start_node(config: NodeConfig) -> Result<()> {
    info!("Starting Lattice node...");
    info!("Chain ID: {}", config.chain.chain_id);
    info!("Data directory: {:?}", config.storage.data_dir);

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
    let state_manager = Arc::new(lattice_storage::state_manager::StateManager::new(storage.db.clone()));
    // MCP + inference service
    let vm_for_mcp = Arc::new(lattice_execution::vm::VM::new(10_000_000));
    let mcp = Arc::new(lattice_mcp::MCPService::new(
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
        lattice_execution::types::Address(a)
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
        lattice_execution::types::Address(a)
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
        let ipfs_api = std::env::var("LATTICE_IPFS_API").ok();
        Arc::new(crate::artifact::NodeArtifactService::new(ipfs_api))
    };

    let storage_bridge: Arc<dyn lattice_execution::executor::AIModelStorage> =
        Arc::new(adapters::StorageAdapter::new(state_manager.clone()));
    let registry_bridge: Arc<dyn lattice_execution::executor::ModelRegistryAdapter> =
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
        lattice_execution::types::Address(a)
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
    let require_valid_signature = std::env::var("LATTICE_REQUIRE_VALID_SIGNATURE")
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
    let metrics_enabled = std::env::var("LATTICE_METRICS")
        .map(|v| matches!(v.to_lowercase().as_str(), "1" | "true" | "yes" | "on"))
        .unwrap_or(false);
    if metrics_enabled {
        let addr_str =
            std::env::var("LATTICE_METRICS_ADDR").unwrap_or_else(|_| "0.0.0.0:9100".to_string());
        let addr: std::net::SocketAddr = addr_str.parse().unwrap();
        tokio::spawn(async move {
            if let Err(e) = lattice_api::metrics_server::MetricsServer::new(addr)
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
            lattice_consensus::types::Hash::default()
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
            tokio::sync::mpsc::channel::<(PeerId, lattice_network::NetworkMessage)>(512);
        peer_manager.set_incoming(in_tx).await;
        let pm_for_rx = peer_manager.clone();
        let storage_for_handler = storage.clone();
        let mempool_for_handler = mempool.clone();
        tokio::spawn(async move {
            use lattice_consensus::types::Hash;
            use lattice_network::NetworkMessage;
            use lattice_sequencer::mempool::TxClass;
            while let Some((pid, msg)) = in_rx.recv().await {
                tracing::debug!("[P2P] from={} msg={:?}", pid.0, msg);
                // Handle protocol messages
                match msg {
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
                            // Re-broadcast to other peers
                            let _ = pm_for_rx
                                .broadcast(&NetworkMessage::NewTransaction { transaction })
                                .await;
                        }
                    }
                    NetworkMessage::NewBlock { block } => {
                        // Store block if we don't have it
                        if !storage_for_handler
                            .blocks
                            .has_block(&block.header.block_hash)
                            .unwrap_or(false)
                        {
                            let _ = storage_for_handler.blocks.put_block(&block);
                            // Re-broadcast to other peers
                            let _ = pm_for_rx
                                .broadcast(&NetworkMessage::NewBlock { block })
                                .await;
                        }
                    }
                    _ => {
                        // Other messages not handled yet
                    }
                }
            }
        });

        // Start TCP listener
        let listen_addr = config.network.listen_addr;
        if let Err(e) = peer_manager
            .start_listener(
                listen_addr,
                network_id,
                genesis_hash,
                head_height,
                head_hash,
            )
            .await
        {
            warn!("Failed to start P2P listener on {}: {}", listen_addr, e);
        }

        // Connect to bootstrap nodes
        for entry in &config.network.bootstrap_nodes {
            if let Some((pid, addr)) = parse_bootnode(entry) {
                let pm = peer_manager.clone();
                let g = genesis_hash;
                tokio::spawn(async move {
                    let _ = pm
                        .connect_bootnode_real(
                            Some(pid),
                            addr,
                            network_id,
                            g,
                            head_height,
                            head_hash,
                        )
                        .await;
                });
            } else {
                tracing::warn!("Invalid bootstrap node: {}", entry);
            }
        }
    }

    // Start RPC server if enabled
    let rpc_handle = if config.rpc.enabled {
        info!("Starting RPC server on {}", config.rpc.listen_addr);

        let rpc_config = RpcConfig {
            listen_addr: config.rpc.listen_addr,
            max_connections: 100,
            cors_domains: vec!["*".to_string()],
            threads: 4,
        };

        let rpc_server = RpcServer::new(
            rpc_config,
            storage.clone(),
            mempool.clone(),
            peer_manager.clone(),
            executor.clone(),
            config.chain.chain_id,
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
        let mut treasury_percentage = 10u8;
        if let Some(bytes) = executor
            .state_db()
            .get_storage(&governance_addr, b"PARAM:treasury_percentage")
        {
            if !bytes.is_empty() {
                treasury_percentage = bytes[0];
            }
        }

        let reward_config = lattice_economics::rewards::RewardConfig {
            block_reward: 10,
            halving_interval: 2_100_000,
            inference_bonus: 1,
            model_deployment_bonus: 1,
            treasury_percentage,
            treasury_address: lattice_execution::types::Address([0x11; 20]),
        };

        let producer = Arc::new(BlockProducer::with_peer_manager_and_rewards(
            storage.clone(),
            executor.clone(),
            mempool.clone(),
            producer_peer_manager,
            lattice_consensus::PublicKey::new(coinbase),
            config.mining.target_block_time,
            reward_config,
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
