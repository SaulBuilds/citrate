use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub rpc_endpoint: String,
    pub chain_id: u64,
    pub keystore_path: PathBuf,
    pub default_account: Option<String>,
    pub gas_price: u64,
    pub gas_limit: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            rpc_endpoint: "http://localhost:8545".to_string(),
            chain_id: 1337,
            keystore_path: dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".lattice")
                .join("keystore"),
            default_account: None,
            gas_price: 1_000_000_000, // 1 gwei
            gas_limit: 3_000_000,
        }
    }
}

impl Config {
    pub fn load(config_path: Option<&Path>, rpc_override: Option<&str>) -> Result<Self> {
        let config_path = config_path
            .map(PathBuf::from)
            .or_else(Self::default_config_path)
            .context("Unable to determine config path")?;

        let mut config = if config_path.exists() {
            let contents = fs::read_to_string(&config_path)
                .with_context(|| format!("Failed to read config from {:?}", config_path))?;
            serde_json::from_str(&contents)
                .with_context(|| format!("Failed to parse config from {:?}", config_path))?
        } else {
            Self::default()
        };

        // Override RPC endpoint if provided
        if let Some(rpc) = rpc_override {
            config.rpc_endpoint = rpc.to_string();
        }

        Ok(config)
    }

    pub fn save(&self, config_path: Option<&Path>) -> Result<()> {
        let config_path = config_path
            .map(PathBuf::from)
            .or_else(Self::default_config_path)
            .context("Unable to determine config path")?;

        // Create parent directory if it doesn't exist
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory {:?}", parent))?;
        }

        let contents = serde_json::to_string_pretty(self)?;
        fs::write(&config_path, contents)
            .with_context(|| format!("Failed to write config to {:?}", config_path))?;

        Ok(())
    }

    pub fn init(force: bool) -> Result<()> {
        let config_path = Self::default_config_path().context("Unable to determine config path")?;

        if config_path.exists() && !force {
            anyhow::bail!(
                "Config already exists at {:?}. Use --force to overwrite",
                config_path
            );
        }

        let config = Self::default();
        config.save(Some(&config_path))?;

        // Create keystore directory
        fs::create_dir_all(&config.keystore_path)
            .with_context(|| format!("Failed to create keystore at {:?}", config.keystore_path))?;

        Ok(())
    }

    fn default_config_path() -> Option<PathBuf> {
        dirs::home_dir().map(|home| home.join(".lattice").join("config.json"))
    }
}
