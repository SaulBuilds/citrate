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

    /// Validator configuration
    #[serde(default)]
    pub validator: ValidatorConfig,
}

/// Validator and production mode configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorConfig {
    /// Production mode: enforces fail-closed behavior for validators
    /// When true, the node will refuse to start without validators configured
    /// Default: false (permissive for development)
    #[serde(default)]
    pub production_mode: bool,

    /// Initial validator public keys (hex-encoded 32-byte keys)
    /// In production mode, at least one validator must be configured
    #[serde(default)]
    pub validators: Vec<String>,

    /// IPFS API endpoint for model pin verification
    #[serde(default = "default_ipfs_url")]
    pub ipfs_api_url: String,

    /// Pin check interval in seconds
    #[serde(default = "default_check_interval")]
    pub check_interval_secs: u64,

    /// Grace period before slashing in hours
    #[serde(default = "default_grace_period")]
    pub grace_period_hours: u64,
}

fn default_ipfs_url() -> String {
    "http://127.0.0.1:5001".to_string()
}

fn default_check_interval() -> u64 {
    3600 // 1 hour
}

fn default_grace_period() -> u64 {
    24 // 24 hours
}

impl Default for ValidatorConfig {
    fn default() -> Self {
        Self {
            production_mode: false,
            validators: vec![],
            ipfs_api_url: default_ipfs_url(),
            check_interval_secs: default_check_interval(),
            grace_period_hours: default_grace_period(),
        }
    }
}

impl ValidatorConfig {
    /// Create a production configuration that enforces validator presence
    pub fn production(validators: Vec<String>) -> Self {
        Self {
            production_mode: true,
            validators,
            ipfs_api_url: default_ipfs_url(),
            check_interval_secs: default_check_interval(),
            grace_period_hours: default_grace_period(),
        }
    }

    /// Validate configuration, returning error if production mode constraints are violated
    pub fn validate(&self) -> Result<(), String> {
        if self.production_mode && self.validators.is_empty() {
            return Err(
                "FAIL-CLOSED: production_mode=true requires at least one validator. \
                 Configure validators in [validator] section or set production_mode=false for development.".to_string()
            );
        }
        Ok(())
    }
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
        // Check for chain ID from environment variable, default to 1337 (devnet)
        let chain_id = std::env::var("CITRATE_CHAIN_ID")
            .ok()
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(1337);

        Self {
            chain: ChainConfig {
                chain_id,
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
                    .join(".citrate"),
                pruning: false,
                keep_blocks: 100000,
            },
            mining: MiningConfig {
                enabled: true,
                coinbase: "0x0000000000000000000000000000000000000000".to_string(),
                target_block_time: 5,
                min_gas_price: 1_000_000_000,
            },
            validator: ValidatorConfig::default(),
        }
    }
}

impl NodeConfig {
    /// Validate the entire configuration
    /// Returns error if any subsystem constraints are violated
    pub fn validate(&self) -> Result<(), String> {
        // Validate validator configuration (fail-closed in production)
        self.validator.validate()?;
        Ok(())
    }

    /// Create devnet configuration
    /// Chain ID can be overridden via CITRATE_CHAIN_ID environment variable
    pub fn devnet() -> Self {
        let mut config = Self::default();
        // Chain ID already set from env var in default(), only override if not set
        if std::env::var("CITRATE_CHAIN_ID").is_err() {
            config.chain.chain_id = 1337;
        }
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
