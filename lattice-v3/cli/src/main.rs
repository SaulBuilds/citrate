//lattice-v3/cli/src/main.rs

use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::Colorize;
use std::path::PathBuf;

mod commands;
mod config;
mod utils;

use commands::{account, contract, governance, model, network};

#[derive(Parser)]
#[command(
    name = "lattice",
    version,
    about = "Lattice v3 CLI - AI-native Layer-1 BlockDAG",
    long_about = None
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Config file path
    #[arg(short, long, global = true, env = "LATTICE_CONFIG")]
    config: Option<PathBuf>,

    /// RPC endpoint
    #[arg(short, long, global = true, env = "LATTICE_RPC")]
    rpc: Option<String>,

    /// Verbosity level
    #[arg(short, long, global = true, action = clap::ArgAction::Count)]
    verbose: u8,
}

#[derive(Subcommand)]
enum Commands {
    /// Account management commands
    #[command(subcommand)]
    Account(account::AccountCommands),

    /// Model deployment and management
    #[command(subcommand)]
    Model(model::ModelCommands),

    /// Smart contract deployment and interaction
    #[command(subcommand)]
    Contract(contract::ContractCommands),

    /// Network and node operations
    #[command(subcommand)]
    Network(network::NetworkCommands),

    /// Governance parameter management
    #[command(subcommand)]
    Governance(governance::GovernanceCommands),

    /// Initialize configuration
    Init {
        /// Force overwrite existing config
        #[arg(short, long)]
        force: bool,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Setup logging
    let log_level = match cli.verbose {
        0 => "error",
        1 => "warn",
        2 => "info",
        3 => "debug",
        _ => "trace",
    };

    tracing_subscriber::fmt()
        .with_env_filter(log_level)
        .with_target(false)
        .init();

    // Load or create config
    let config = config::Config::load(cli.config.as_deref(), cli.rpc.as_deref())?;

    // Execute command
    match cli.command {
        Commands::Account(cmd) => account::execute(cmd, &config).await?,
        Commands::Model(cmd) => model::execute(cmd, &config).await?,
        Commands::Contract(cmd) => contract::execute(cmd, &config).await?,
        Commands::Network(cmd) => network::execute(cmd, &config).await?,
        Commands::Governance(cmd) => governance::execute(cmd, &config).await?,
        Commands::Init { force } => {
            config::Config::init(force)?;
            println!("{}", "✓ Configuration initialized successfully".green());
        }
    }

    Ok(())
}
