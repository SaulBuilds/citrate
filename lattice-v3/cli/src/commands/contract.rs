use anyhow::{Context, Result};
use clap::Subcommand;
use colored::Colorize;
use std::fs;
use std::path::PathBuf;
use serde_json::json;

use crate::config::Config;

#[derive(Subcommand)]
pub enum ContractCommands {
    /// Deploy a smart contract
    Deploy {
        /// Path to contract bytecode or WASM file
        contract: PathBuf,
        
        /// Constructor arguments (JSON)
        #[arg(short, long)]
        args: Option<String>,
        
        /// Account to deploy from
        #[arg(short, long)]
        account: Option<String>,
        
        /// Contract value in wei
        #[arg(short, long, default_value = "0")]
        value: String,
    },

    /// Call a contract method
    Call {
        /// Contract address
        address: String,
        
        /// Method signature (e.g., "transfer(address,uint256)")
        method: String,
        
        /// Method arguments (JSON array)
        #[arg(short, long)]
        args: Option<String>,
        
        /// Account to call from
        #[arg(short, long)]
        account: Option<String>,
        
        /// Value to send in wei
        #[arg(short, long, default_value = "0")]
        value: String,
    },

    /// Read contract state (call without transaction)
    Read {
        /// Contract address
        address: String,
        
        /// Method signature
        method: String,
        
        /// Method arguments (JSON array)
        #[arg(short, long)]
        args: Option<String>,
    },

    /// Get contract code
    Code {
        /// Contract address
        address: String,
        
        /// Output file path (optional)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Verify contract source code
    Verify {
        /// Contract address
        address: String,
        
        /// Source code file
        source: PathBuf,
        
        /// Compiler version
        #[arg(short, long)]
        compiler: String,
        
        /// Optimization enabled
        #[arg(short, long)]
        optimized: bool,
    },
}

pub async fn execute(cmd: ContractCommands, config: &Config) -> Result<()> {
    match cmd {
        ContractCommands::Deploy { contract, args, account, value } => {
            deploy_contract(config, contract, args, account, value).await?;
        }
        ContractCommands::Call { address, method, args, account, value } => {
            call_contract(config, &address, &method, args, account, value).await?;
        }
        ContractCommands::Read { address, method, args } => {
            read_contract(config, &address, &method, args).await?;
        }
        ContractCommands::Code { address, output } => {
            get_contract_code(config, &address, output).await?;
        }
        ContractCommands::Verify { address, source, compiler, optimized } => {
            verify_contract(config, &address, source, &compiler, optimized).await?;
        }
    }
    Ok(())
}

async fn deploy_contract(
    config: &Config,
    contract_path: PathBuf,
    args: Option<String>,
    account: Option<String>,
    value: String,
) -> Result<()> {
    println!("{}", "Deploying contract...".cyan());
    
    // Read contract bytecode
    let bytecode = if contract_path.extension().and_then(|s| s.to_str()) == Some("wasm") {
        // WASM contract
        let wasm_bytes = fs::read(&contract_path)
            .with_context(|| format!("Failed to read WASM file {:?}", contract_path))?;
        hex::encode(wasm_bytes)
    } else {
        // Assume hex bytecode
        let content = fs::read_to_string(&contract_path)
            .with_context(|| format!("Failed to read contract file {:?}", contract_path))?;
        content.trim().trim_start_matches("0x").to_string()
    };
    
    // Parse constructor arguments
    let constructor_args = if let Some(args_str) = args {
        serde_json::from_str(&args_str).context("Invalid constructor arguments JSON")?
    } else {
        json!([])
    };
    
    // Get account
    let from_account = account.or(config.default_account.clone())
        .context("No account specified and no default account configured")?;
    
    // Parse value
    let value_wei = value.parse::<u128>()
        .context("Invalid value")?;
    
    // Make RPC call to deploy
    let client = reqwest::Client::new();
    let response = client
        .post(&config.rpc_endpoint)
        .json(&json!({
            "jsonrpc": "2.0",
            "method": "eth_sendTransaction",
            "params": [{
                "from": from_account,
                "data": format!("0x{}", bytecode),
                "value": format!("0x{:x}", value_wei),
                "gas": format!("0x{:x}", config.gas_limit),
                "gasPrice": format!("0x{:x}", config.gas_price),
            }],
            "id": 1
        }))
        .send()
        .await
        .context("Failed to connect to RPC endpoint")?;
    
    let result: serde_json::Value = response.json().await?;
    
    if let Some(tx_hash) = result["result"].as_str() {
        println!("{}", "✓ Contract deployment initiated".green());
        println!("Transaction: {}", tx_hash.cyan());
        
        // Wait for receipt
        println!("Waiting for confirmation...");
        
        let receipt = wait_for_receipt(config, tx_hash).await?;
        
        if let Some(contract_address) = receipt["contractAddress"].as_str() {
            println!("{}", "✓ Contract deployed successfully".green().bold());
            println!("Contract Address: {}", contract_address.cyan().bold());
            println!("Gas Used: {}", receipt["gasUsed"].as_str().unwrap_or("N/A"));
        } else {
            anyhow::bail!("Contract deployment failed");
        }
    } else if let Some(error) = result["error"].as_object() {
        anyhow::bail!("Deployment failed: {}", error["message"].as_str().unwrap_or("Unknown error"));
    } else {
        anyhow::bail!("Unexpected response from RPC");
    }
    
    Ok(())
}

async fn call_contract(
    config: &Config,
    address: &str,
    method: &str,
    args: Option<String>,
    account: Option<String>,
    value: String,
) -> Result<()> {
    println!("{}", format!("Calling {}...", method).cyan());
    
    // Parse method arguments
    let method_args = if let Some(args_str) = args {
        serde_json::from_str(&args_str).context("Invalid method arguments JSON")?
    } else {
        json!([])
    };
    
    // Encode method call
    let call_data = encode_method_call(method, method_args)?;
    
    // Get account
    let from_account = account.or(config.default_account.clone())
        .context("No account specified and no default account configured")?;
    
    // Parse value
    let value_wei = value.parse::<u128>()
        .context("Invalid value")?;
    
    // Make RPC call
    let client = reqwest::Client::new();
    let response = client
        .post(&config.rpc_endpoint)
        .json(&json!({
            "jsonrpc": "2.0",
            "method": "eth_sendTransaction",
            "params": [{
                "from": from_account,
                "to": address,
                "data": call_data,
                "value": format!("0x{:x}", value_wei),
                "gas": format!("0x{:x}", config.gas_limit),
                "gasPrice": format!("0x{:x}", config.gas_price),
            }],
            "id": 1
        }))
        .send()
        .await
        .context("Failed to connect to RPC endpoint")?;
    
    let result: serde_json::Value = response.json().await?;
    
    if let Some(tx_hash) = result["result"].as_str() {
        println!("{}", "✓ Transaction sent".green());
        println!("Transaction: {}", tx_hash.cyan());
        
        // Wait for receipt
        println!("Waiting for confirmation...");
        
        let receipt = wait_for_receipt(config, tx_hash).await?;
        
        if receipt["status"].as_str() == Some("0x1") {
            println!("{}", "✓ Transaction successful".green().bold());
            println!("Gas Used: {}", receipt["gasUsed"].as_str().unwrap_or("N/A"));
            
            // Decode logs if any
            if let Some(logs) = receipt["logs"].as_array() {
                if !logs.is_empty() {
                    println!("\nEvents emitted:");
                    for log in logs {
                        if let Some(topics) = log["topics"].as_array() {
                            if !topics.is_empty() {
                                println!("  Event: {}", topics[0].as_str().unwrap_or("Unknown"));
                            }
                        }
                    }
                }
            }
        } else {
            anyhow::bail!("Transaction failed");
        }
    } else if let Some(error) = result["error"].as_object() {
        anyhow::bail!("Call failed: {}", error["message"].as_str().unwrap_or("Unknown error"));
    } else {
        anyhow::bail!("Unexpected response from RPC");
    }
    
    Ok(())
}

async fn read_contract(
    config: &Config,
    address: &str,
    method: &str,
    args: Option<String>,
) -> Result<()> {
    // Parse method arguments
    let method_args = if let Some(args_str) = args {
        serde_json::from_str(&args_str).context("Invalid method arguments JSON")?
    } else {
        json!([])
    };
    
    // Encode method call
    let call_data = encode_method_call(method, method_args)?;
    
    // Make RPC call (eth_call doesn't require from address)
    let client = reqwest::Client::new();
    let response = client
        .post(&config.rpc_endpoint)
        .json(&json!({
            "jsonrpc": "2.0",
            "method": "eth_call",
            "params": [{
                "to": address,
                "data": call_data,
            }, "latest"],
            "id": 1
        }))
        .send()
        .await
        .context("Failed to connect to RPC endpoint")?;
    
    let result: serde_json::Value = response.json().await?;
    
    if let Some(data) = result["result"].as_str() {
        println!("{}", "Result:".bold());
        println!("Raw: {}", data);
        
        // Try to decode as common types
        if data.len() == 66 {
            // Possibly uint256
            if let Ok(value) = u128::from_str_radix(&data[2..], 16) {
                println!("Decoded (uint): {}", value);
            }
        } else if data.len() == 42 {
            // Possibly address
            println!("Decoded (address): {}", data);
        } else if data.len() > 2 {
            // Try as string
            if let Ok(bytes) = hex::decode(&data[2..]) {
                if let Ok(text) = String::from_utf8(bytes.clone()) {
                    if text.chars().all(|c| c.is_ascii_graphic() || c.is_ascii_whitespace()) {
                        println!("Decoded (string): {}", text.trim());
                    }
                }
            }
        }
    } else if let Some(error) = result["error"].as_object() {
        anyhow::bail!("Call failed: {}", error["message"].as_str().unwrap_or("Unknown error"));
    } else {
        anyhow::bail!("Unexpected response from RPC");
    }
    
    Ok(())
}

async fn get_contract_code(
    config: &Config,
    address: &str,
    output: Option<PathBuf>,
) -> Result<()> {
    // Make RPC call to get code
    let client = reqwest::Client::new();
    let response = client
        .post(&config.rpc_endpoint)
        .json(&json!({
            "jsonrpc": "2.0",
            "method": "eth_getCode",
            "params": [address, "latest"],
            "id": 1
        }))
        .send()
        .await
        .context("Failed to connect to RPC endpoint")?;
    
    let result: serde_json::Value = response.json().await?;
    
    if let Some(code) = result["result"].as_str() {
        if code == "0x" {
            println!("{}", "No contract code at this address".yellow());
        } else {
            let code_bytes = hex::decode(&code[2..])
                .context("Invalid code format")?;
            
            println!("{}", "Contract code retrieved".green());
            println!("Size: {} bytes", code_bytes.len());
            
            if let Some(path) = output {
                fs::write(&path, &code_bytes)
                    .with_context(|| format!("Failed to write code to {:?}", path))?;
                println!("Code saved to: {:?}", path);
            } else {
                println!("\nBytecode (first 100 bytes):");
                println!("{}", &code[..code.len().min(202)]);
                if code.len() > 202 {
                    println!("... ({} more bytes)", (code.len() - 202) / 2);
                }
            }
        }
    } else if let Some(error) = result["error"].as_object() {
        anyhow::bail!("Query failed: {}", error["message"].as_str().unwrap_or("Unknown error"));
    } else {
        anyhow::bail!("Unexpected response from RPC");
    }
    
    Ok(())
}

async fn verify_contract(
    config: &Config,
    address: &str,
    source_path: PathBuf,
    compiler: &str,
    optimized: bool,
) -> Result<()> {
    println!("{}", "Verifying contract...".cyan());
    
    // Read source code
    let source_code = fs::read_to_string(&source_path)
        .with_context(|| format!("Failed to read source file {:?}", source_path))?;
    
    // Make RPC call to verify
    let client = reqwest::Client::new();
    let response = client
        .post(&config.rpc_endpoint)
        .json(&json!({
            "jsonrpc": "2.0",
            "method": "lattice_verifyContract",
            "params": {
                "address": address,
                "source_code": source_code,
                "compiler_version": compiler,
                "optimization_enabled": optimized,
            },
            "id": 1
        }))
        .send()
        .await
        .context("Failed to connect to RPC endpoint")?;
    
    let result: serde_json::Value = response.json().await?;
    
    if result["result"]["verified"].as_bool() == Some(true) {
        println!("{}", "✓ Contract verified successfully".green().bold());
        println!("Verification ID: {}", result["result"]["verification_id"].as_str().unwrap_or("N/A"));
    } else if let Some(error) = result["error"].as_object() {
        anyhow::bail!("Verification failed: {}", error["message"].as_str().unwrap_or("Unknown error"));
    } else {
        anyhow::bail!("Contract verification failed");
    }
    
    Ok(())
}

async fn wait_for_receipt(config: &Config, tx_hash: &str) -> Result<serde_json::Value> {
    let client = reqwest::Client::new();
    
    for _ in 0..30 {
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        
        let response = client
            .post(&config.rpc_endpoint)
            .json(&json!({
                "jsonrpc": "2.0",
                "method": "eth_getTransactionReceipt",
                "params": [tx_hash],
                "id": 1
            }))
            .send()
            .await?;
        
        let result: serde_json::Value = response.json().await?;
        
        if let Some(receipt) = result["result"].as_object() {
            return Ok(json!(receipt));
        }
    }
    
    anyhow::bail!("Transaction receipt not found after 60 seconds");
}

fn encode_method_call(method_sig: &str, _args: serde_json::Value) -> Result<String> {
    // Simple method signature encoding (would use ethers-rs in production)
    use sha3::{Digest, Keccak256};
    
    let mut hasher = Keccak256::new();
    hasher.update(method_sig.as_bytes());
    let hash = hasher.finalize();
    
    // Take first 4 bytes as method selector
    let selector = &hash[..4];
    
    // TODO: Properly encode arguments based on ABI
    // For now, just return the selector
    Ok(format!("0x{}", hex::encode(selector)))
}