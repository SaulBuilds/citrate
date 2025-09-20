use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::path::PathBuf;

/// Node configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeConfig {
    /// Chain configuration
    pub chain: ChainConfig,
    
    /// Network configuration
    pub network: NetworkConfig,
    
    /// RPC configuration
    pub rpc: RpcConfig,
    
    /// Storage configuration
    pub storage: StorageConfig,
    
    /// Mining configuration
    pub mining: MiningConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainConfig {
    /// Chain ID
    pub chain_id: u64,
    
    /// Genesis block hash (empty for new chain)
    pub genesis_hash: Option<String>,
    
    /// Block time in seconds
    pub block_time: u64,
    
    /// GhostDAG K parameter
    pub ghostdag_k: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// P2P listen address
    pub listen_addr: SocketAddr,
    
    /// Bootstrap nodes
    pub bootstrap_nodes: Vec<String>,
    
    /// Max peers
    pub max_peers: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcConfig {
    /// RPC enabled
    pub enabled: bool,
    
    /// RPC listen address
    pub listen_addr: SocketAddr,
    
    /// WebSocket listen address
    pub ws_addr: SocketAddr,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    /// Data directory
    pub data_dir: PathBuf,
    
    /// Prune old blocks
    pub pruning: bool,
    
    /// Blocks to keep if pruning
    pub keep_blocks: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MiningConfig {
    /// Enable mining
    pub enabled: bool,
    
    /// Coinbase address (hex)
    pub coinbase: String,
    
    /// Target block time (seconds)
    pub target_block_time: u64,
    
    /// Min gas price
    pub min_gas_price: u64,
}

impl Default for NodeConfig {
    fn default() -> Self {
        Self {
            chain: ChainConfig {
                chain_id: 1337, // Devnet chain ID
                genesis_hash: None,
                block_time: 5,
                ghostdag_k: 18,
            },
            network: NetworkConfig {
                listen_addr: "127.0.0.1:30303".parse().unwrap(),
                bootstrap_nodes: vec![],
                max_peers: 50,
            },
            rpc: RpcConfig {
                enabled: true,
                listen_addr: "127.0.0.1:8545".parse().unwrap(),
                ws_addr: "127.0.0.1:8546".parse().unwrap(),
            },
            storage: StorageConfig {
                data_dir: dirs::home_dir()
                    .unwrap_or_else(|| PathBuf::from("."))
                    .join(".lattice"),
                pruning: false,
                keep_blocks: 100000,
            },
            mining: MiningConfig {
                enabled: true,
                coinbase: "0x0000000000000000000000000000000000000000".to_string(),
                target_block_time: 5,
                min_gas_price: 1_000_000_000,
            },
        }
    }
}

impl NodeConfig {
    /// Create devnet configuration
    pub fn devnet() -> Self {
        let mut config = Self::default();
        config.chain.chain_id = 1337;
        config.mining.enabled = true;
        config.mining.target_block_time = 2; // Fast blocks for testing
        config
    }
    
    /// Load from file
    pub fn from_file(path: &PathBuf) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: NodeConfig = toml::from_str(&content)?;
        Ok(config)
    }
    
    /// Save to file
    #[allow(dead_code)]
    pub fn save(&self, path: &PathBuf) -> anyhow::Result<()> {
        let content = toml::to_string_pretty(self)?;
        std::fs::create_dir_all(path.parent().unwrap())?;
        std::fs::write(path, content)?;
        Ok(())
    }
}
