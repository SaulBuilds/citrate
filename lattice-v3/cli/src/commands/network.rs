//lattice-v3/cli/src/commands/network.rs

use anyhow::{Context, Result};
use clap::Subcommand;
use colored::Colorize;
use serde_json::json;

use crate::config::Config;

#[derive(Subcommand)]
pub enum NetworkCommands {
    /// Get network status
    Status,

    /// Get current block information
    Block {
        /// Block number or "latest"
        #[arg(default_value = "latest")]
        block: String,
    },

    /// Get transaction details
    Transaction {
        /// Transaction hash
        tx_hash: String,
    },

    /// Get current gas price
    GasPrice,

    /// Get peer information
    Peers,

    /// Get sync status
    Sync,

    /// Get DAG statistics
    DagStats,
}

pub async fn execute(cmd: NetworkCommands, config: &Config) -> Result<()> {
    match cmd {
        NetworkCommands::Status => get_status(config).await?,
        NetworkCommands::Block { block } => get_block(config, &block).await?,
        NetworkCommands::Transaction { tx_hash } => get_transaction(config, &tx_hash).await?,
        NetworkCommands::GasPrice => get_gas_price(config).await?,
        NetworkCommands::Peers => get_peers(config).await?,
        NetworkCommands::Sync => get_sync_status(config).await?,
        NetworkCommands::DagStats => get_dag_stats(config).await?,
    }
    Ok(())
}

async fn get_status(config: &Config) -> Result<()> {
    let client = reqwest::Client::new();

    // Get network ID
    let response = client
        .post(&config.rpc_endpoint)
        .json(&json!({
            "jsonrpc": "2.0",
            "method": "net_version",
            "params": [],
            "id": 1
        }))
        .send()
        .await
        .context("Failed to connect to RPC endpoint")?;

    let result: serde_json::Value = response.json().await?;
    let network_id = result["result"].as_str().unwrap_or("Unknown");

    // Get latest block
    let response = client
        .post(&config.rpc_endpoint)
        .json(&json!({
            "jsonrpc": "2.0",
            "method": "eth_blockNumber",
            "params": [],
            "id": 1
        }))
        .send()
        .await?;

    let result: serde_json::Value = response.json().await?;
    let block_number = if let Some(hex) = result["result"].as_str() {
        u64::from_str_radix(&hex[2..], 16).unwrap_or(0)
    } else {
        0
    };

    // Get syncing status
    let response = client
        .post(&config.rpc_endpoint)
        .json(&json!({
            "jsonrpc": "2.0",
            "method": "eth_syncing",
            "params": [],
            "id": 1
        }))
        .send()
        .await?;

    let result: serde_json::Value = response.json().await?;
    let syncing = !result["result"].is_boolean();

    println!("{}", "Network Status:".bold());
    println!("RPC Endpoint: {}", config.rpc_endpoint.cyan());
    println!("Network ID: {}", network_id);
    println!("Chain ID: {}", config.chain_id);
    println!("Latest Block: {}", block_number);
    println!(
        "Syncing: {}",
        if syncing {
            "Yes".yellow()
        } else {
            "No (synced)".green()
        }
    );

    Ok(())
}

async fn get_block(config: &Config, block: &str) -> Result<()> {
    let client = reqwest::Client::new();

    let block_param = if block == "latest" {
        "latest".to_string()
    } else if block.starts_with("0x") {
        block.to_string()
    } else {
        format!(
            "0x{:x}",
            block.parse::<u64>().context("Invalid block number")?
        )
    };

    let response = client
        .post(&config.rpc_endpoint)
        .json(&json!({
            "jsonrpc": "2.0",
            "method": "eth_getBlockByNumber",
            "params": [block_param, true],
            "id": 1
        }))
        .send()
        .await
        .context("Failed to connect to RPC endpoint")?;

    let result: serde_json::Value = response.json().await?;

    if let Some(block) = result["result"].as_object() {
        println!("{}", "Block Information:".bold());
        println!(
            "Number: {}",
            u64::from_str_radix(
                block["number"]
                    .as_str()
                    .unwrap_or("0x0")
                    .trim_start_matches("0x"),
                16
            )
            .unwrap_or(0)
        );
        println!("Hash: {}", block["hash"].as_str().unwrap_or("N/A").cyan());
        println!("Parent: {}", block["parentHash"].as_str().unwrap_or("N/A"));
        println!(
            "Timestamp: {}",
            u64::from_str_radix(
                block["timestamp"]
                    .as_str()
                    .unwrap_or("0x0")
                    .trim_start_matches("0x"),
                16
            )
            .unwrap_or(0)
        );
        println!("Miner: {}", block["miner"].as_str().unwrap_or("N/A"));

        if let Some(txs) = block["transactions"].as_array() {
            println!("Transactions: {}", txs.len());

            if !txs.is_empty() && txs.len() <= 5 {
                println!("\nTransactions:");
                for tx in txs {
                    if let Some(hash) = tx["hash"].as_str() {
                        println!("  • {}", hash);
                    }
                }
            } else if txs.len() > 5 {
                println!("  (showing first 5)");
                for tx in txs.iter().take(5) {
                    if let Some(hash) = tx["hash"].as_str() {
                        println!("  • {}", hash);
                    }
                }
                println!("  ... and {} more", txs.len() - 5);
            }
        }

        // GhostDAG specific fields
        if let Some(merge_parents) = block["mergeParents"].as_array() {
            if !merge_parents.is_empty() {
                println!("\nGhostDAG Merge Parents: {}", merge_parents.len());
                for parent in merge_parents.iter().take(3) {
                    if let Some(hash) = parent.as_str() {
                        println!("  • {}", hash);
                    }
                }
            }
        }

        if let Some(blue_score) = block["blueScore"].as_u64() {
            println!("Blue Score: {}", blue_score);
        }
    } else if let Some(error) = result["error"].as_object() {
        anyhow::bail!(
            "Query failed: {}",
            error["message"].as_str().unwrap_or("Unknown error")
        );
    } else {
        anyhow::bail!("Block not found");
    }

    Ok(())
}

async fn get_transaction(config: &Config, tx_hash: &str) -> Result<()> {
    let client = reqwest::Client::new();

    let response = client
        .post(&config.rpc_endpoint)
        .json(&json!({
            "jsonrpc": "2.0",
            "method": "eth_getTransactionByHash",
            "params": [tx_hash],
            "id": 1
        }))
        .send()
        .await
        .context("Failed to connect to RPC endpoint")?;

    let result: serde_json::Value = response.json().await?;

    if let Some(tx) = result["result"].as_object() {
        println!("{}", "Transaction Details:".bold());
        println!("Hash: {}", tx["hash"].as_str().unwrap_or("N/A").cyan());
        println!("From: {}", tx["from"].as_str().unwrap_or("N/A"));
        println!("To: {}", tx["to"].as_str().unwrap_or("(Contract Creation)"));

        if let Some(value_hex) = tx["value"].as_str() {
            let value = u128::from_str_radix(value_hex.trim_start_matches("0x"), 16).unwrap_or(0);
            println!("Value: {} wei ({} ETH)", value, value as f64 / 1e18);
        }

        println!(
            "Nonce: {}",
            u64::from_str_radix(
                tx["nonce"]
                    .as_str()
                    .unwrap_or("0x0")
                    .trim_start_matches("0x"),
                16
            )
            .unwrap_or(0)
        );

        println!(
            "Gas Price: {} gwei",
            u64::from_str_radix(
                tx["gasPrice"]
                    .as_str()
                    .unwrap_or("0x0")
                    .trim_start_matches("0x"),
                16
            )
            .unwrap_or(0)
                / 1_000_000_000
        );

        println!(
            "Gas Limit: {}",
            u64::from_str_radix(
                tx["gas"].as_str().unwrap_or("0x0").trim_start_matches("0x"),
                16
            )
            .unwrap_or(0)
        );

        if let Some(block_number) = tx["blockNumber"].as_str() {
            println!(
                "Block: {}",
                u64::from_str_radix(block_number.trim_start_matches("0x"), 16).unwrap_or(0)
            );
        } else {
            println!("Status: {}", "Pending".yellow());
        }

        // Get receipt if transaction is mined
        if tx["blockNumber"].is_string() {
            let receipt_response = client
                .post(&config.rpc_endpoint)
                .json(&json!({
                    "jsonrpc": "2.0",
                    "method": "eth_getTransactionReceipt",
                    "params": [tx_hash],
                    "id": 1
                }))
                .send()
                .await?;

            let receipt_result: serde_json::Value = receipt_response.json().await?;

            if let Some(receipt) = receipt_result["result"].as_object() {
                println!("\nReceipt:");

                let status = receipt["status"].as_str().unwrap_or("0x0");
                if status == "0x1" {
                    println!("  Status: {}", "Success".green());
                } else {
                    println!("  Status: {}", "Failed".red());
                }

                println!(
                    "  Gas Used: {}",
                    u64::from_str_radix(
                        receipt["gasUsed"]
                            .as_str()
                            .unwrap_or("0x0")
                            .trim_start_matches("0x"),
                        16
                    )
                    .unwrap_or(0)
                );

                if let Some(contract_address) = receipt["contractAddress"].as_str() {
                    if contract_address != "null" {
                        println!("  Contract Created: {}", contract_address.cyan());
                    }
                }

                if let Some(logs) = receipt["logs"].as_array() {
                    if !logs.is_empty() {
                        println!("  Events: {} emitted", logs.len());
                    }
                }
            }
        }
    } else if let Some(error) = result["error"].as_object() {
        anyhow::bail!(
            "Query failed: {}",
            error["message"].as_str().unwrap_or("Unknown error")
        );
    } else {
        anyhow::bail!("Transaction not found");
    }

    Ok(())
}

async fn get_gas_price(config: &Config) -> Result<()> {
    let client = reqwest::Client::new();

    let response = client
        .post(&config.rpc_endpoint)
        .json(&json!({
            "jsonrpc": "2.0",
            "method": "eth_gasPrice",
            "params": [],
            "id": 1
        }))
        .send()
        .await
        .context("Failed to connect to RPC endpoint")?;

    let result: serde_json::Value = response.json().await?;

    if let Some(price_hex) = result["result"].as_str() {
        let price = u64::from_str_radix(price_hex.trim_start_matches("0x"), 16).unwrap_or(0);

        println!("{}", "Current Gas Price:".bold());
        println!("  {} wei", price);
        println!("  {} gwei", price as f64 / 1e9);
        println!("  {} ETH", price as f64 / 1e18);
    } else if let Some(error) = result["error"].as_object() {
        anyhow::bail!(
            "Query failed: {}",
            error["message"].as_str().unwrap_or("Unknown error")
        );
    } else {
        anyhow::bail!("Unexpected response");
    }

    Ok(())
}

async fn get_peers(config: &Config) -> Result<()> {
    let client = reqwest::Client::new();

    let response = client
        .post(&config.rpc_endpoint)
        .json(&json!({
            "jsonrpc": "2.0",
            "method": "net_peerCount",
            "params": [],
            "id": 1
        }))
        .send()
        .await
        .context("Failed to connect to RPC endpoint")?;

    let result: serde_json::Value = response.json().await?;

    if let Some(count_hex) = result["result"].as_str() {
        let count = u64::from_str_radix(count_hex.trim_start_matches("0x"), 16).unwrap_or(0);

        println!("{}", "Network Peers:".bold());
        println!("Connected Peers: {}", count);

        // Try to get more detailed peer info
        let response = client
            .post(&config.rpc_endpoint)
            .json(&json!({
                "jsonrpc": "2.0",
                "method": "admin_peers",
                "params": [],
                "id": 1
            }))
            .send()
            .await;

        if let Ok(response) = response {
            let result: serde_json::Value = response.json().await?;

            if let Some(peers) = result["result"].as_array() {
                if !peers.is_empty() {
                    println!("\nPeer Details:");
                    for (i, peer) in peers.iter().take(5).enumerate() {
                        if let Some(enode) = peer["enode"].as_str() {
                            println!("  {}. {}", i + 1, &enode[..enode.len().min(60)]);
                        }
                    }

                    if peers.len() > 5 {
                        println!("  ... and {} more", peers.len() - 5);
                    }
                }
            }
        }
    } else if let Some(error) = result["error"].as_object() {
        anyhow::bail!(
            "Query failed: {}",
            error["message"].as_str().unwrap_or("Unknown error")
        );
    } else {
        anyhow::bail!("Unexpected response");
    }

    Ok(())
}

async fn get_sync_status(config: &Config) -> Result<()> {
    let client = reqwest::Client::new();

    let response = client
        .post(&config.rpc_endpoint)
        .json(&json!({
            "jsonrpc": "2.0",
            "method": "eth_syncing",
            "params": [],
            "id": 1
        }))
        .send()
        .await
        .context("Failed to connect to RPC endpoint")?;

    let result: serde_json::Value = response.json().await?;

    if let Some(sync) = result["result"].as_object() {
        println!("{}", "Sync Status: SYNCING".yellow().bold());

        if let Some(current) = sync["currentBlock"].as_str() {
            let current = u64::from_str_radix(current.trim_start_matches("0x"), 16).unwrap_or(0);
            println!("Current Block: {}", current);
        }

        if let Some(highest) = sync["highestBlock"].as_str() {
            let highest = u64::from_str_radix(highest.trim_start_matches("0x"), 16).unwrap_or(0);
            println!("Highest Block: {}", highest);
        }

        if let (Some(current), Some(highest)) =
            (sync["currentBlock"].as_str(), sync["highestBlock"].as_str())
        {
            let current = u64::from_str_radix(current.trim_start_matches("0x"), 16).unwrap_or(0);
            let highest = u64::from_str_radix(highest.trim_start_matches("0x"), 16).unwrap_or(1);
            let progress = (current as f64 / highest as f64) * 100.0;

            println!("Progress: {:.2}%", progress);

            // Progress bar
            let bar_width = 40;
            let filled = (progress / 100.0 * bar_width as f64) as usize;
            let bar = "█".repeat(filled) + &"░".repeat(bar_width - filled);
            println!("[{}]", bar);
        }
    } else if result["result"].as_bool() == Some(false) {
        println!("{}", "Sync Status: SYNCED".green().bold());
        println!("Node is fully synchronized with the network");
    } else if let Some(error) = result["error"].as_object() {
        anyhow::bail!(
            "Query failed: {}",
            error["message"].as_str().unwrap_or("Unknown error")
        );
    } else {
        anyhow::bail!("Unexpected response");
    }

    Ok(())
}

async fn get_dag_stats(config: &Config) -> Result<()> {
    let client = reqwest::Client::new();

    // Custom Lattice RPC method for DAG statistics
    let response = client
        .post(&config.rpc_endpoint)
        .json(&json!({
            "jsonrpc": "2.0",
            "method": "lattice_getDagStats",
            "params": [],
            "id": 1
        }))
        .send()
        .await
        .context("Failed to connect to RPC endpoint")?;

    let result: serde_json::Value = response.json().await?;

    if let Some(stats) = result["result"].as_object() {
        println!("{}", "DAG Statistics:".bold());
        println!(
            "Total Blocks: {}",
            stats["totalBlocks"].as_u64().unwrap_or(0)
        );
        println!("Blue Blocks: {}", stats["blueBlocks"].as_u64().unwrap_or(0));
        println!("Red Blocks: {}", stats["redBlocks"].as_u64().unwrap_or(0));
        println!("Tips Count: {}", stats["tipsCount"].as_u64().unwrap_or(0));
        println!(
            "Max Blue Score: {}",
            stats["maxBlueScore"].as_u64().unwrap_or(0)
        );

        if let Some(tips) = stats["currentTips"].as_array() {
            if !tips.is_empty() {
                println!("\nCurrent Tips:");
                for (i, tip) in tips.iter().take(3).enumerate() {
                    if let Some(hash) = tip.as_str() {
                        println!("  {}. {}", i + 1, hash);
                    }
                }

                if tips.len() > 3 {
                    println!("  ... and {} more", tips.len() - 3);
                }
            }
        }
    } else if let Some(error) = result["error"].as_object() {
        // Method might not be available
        if error["message"]
            .as_str()
            .unwrap_or("")
            .contains("not found")
        {
            println!(
                "{}",
                "DAG statistics not available (custom RPC method)".yellow()
            );
            println!("This feature requires a Lattice v3 node with GhostDAG support");
        } else {
            anyhow::bail!(
                "Query failed: {}",
                error["message"].as_str().unwrap_or("Unknown error")
            );
        }
    } else {
        anyhow::bail!("Unexpected response");
    }

    Ok(())
}
