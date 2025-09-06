use anyhow::Result;
use clap::{Parser, Subcommand};
use lattice_api::{RpcServer, RpcConfig};
use lattice_consensus::crypto;
use lattice_execution::{Executor, StateDB};
use lattice_network::peer::{PeerManager, PeerManagerConfig};
use lattice_sequencer::mempool::{Mempool, MempoolConfig};
use lattice_storage::{StorageManager, pruning::PruningConfig};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tracing::{info, error};
use tracing_subscriber::EnvFilter;

mod config;
mod genesis;
mod producer;

use config::NodeConfig;
use genesis::{GenesisConfig, initialize_genesis_state};
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
        .with_env_filter(
            EnvFilter::from_default_env()
                .add_directive("lattice=info".parse()?)
        )
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
    
    // Start node
    start_node(config).await
}

async fn init_chain(chain_id: u64) -> Result<()> {
    info!("Initializing new chain with ID {}", chain_id);
    
    let temp_dir = PathBuf::from(".lattice");
    std::fs::create_dir_all(&temp_dir)?;
    
    // Create storage
    let storage = Arc::new(StorageManager::new(
        &temp_dir,
        PruningConfig::default(),
    )?);
    
    // Create executor
    let state_db = Arc::new(StateDB::new());
    let executor = Arc::new(Executor::new(state_db));
    
    // Initialize genesis
    let genesis_config = GenesisConfig {
        chain_id,
        ..Default::default()
    };
    
    let genesis_hash = initialize_genesis_state(
        storage.clone(),
        executor,
        &genesis_config,
    ).await?;
    
    info!("Genesis block created: {:?}", hex::encode(&genesis_hash.as_bytes()[..8]));
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
        let executor = Arc::new(Executor::new(state_db));
        
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
    
    // Create state DB and executor
    let state_db = Arc::new(StateDB::new());
    let executor = Arc::new(Executor::new(state_db));
    
    // Create mempool
    let mempool = Arc::new(Mempool::new(MempoolConfig {
        max_size: 10000,
        max_per_sender: 100,
        min_gas_price: config.mining.min_gas_price,
        tx_expiry_secs: 3600,
        allow_replacement: true,
        replacement_factor: 110,
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
        );
        
        Some(tokio::spawn(async move {
            match rpc_server.start().await {
                Ok(_server) => {
                    info!("RPC server started");
                    // Keep server alive
                    tokio::signal::ctrl_c().await.ok();
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
        let coinbase_bytes = hex::decode(&config.mining.coinbase)
            .unwrap_or_else(|_| vec![0; 32]);
        let mut coinbase = [0u8; 32];
        coinbase.copy_from_slice(&coinbase_bytes[..32.min(coinbase_bytes.len())]);
        
        let producer = Arc::new(BlockProducer::new(
            storage.clone(),
            executor.clone(),
            mempool.clone(),
            lattice_consensus::PublicKey::new(coinbase),
            config.mining.target_block_time,
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