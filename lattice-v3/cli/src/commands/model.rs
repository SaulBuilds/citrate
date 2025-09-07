use anyhow::{Context, Result};
use clap::Subcommand;
use colored::Colorize;
use std::fs;
use std::path::PathBuf;
use serde_json::json;

use crate::config::Config;

#[derive(Subcommand)]
pub enum ModelCommands {
    /// Deploy a model to the network
    Deploy {
        /// Path to model file (ONNX, PyTorch, TensorFlow)
        model: PathBuf,
        
        /// Model metadata JSON file
        #[arg(short, long)]
        metadata: Option<PathBuf>,
        
        /// Model name
        #[arg(short, long)]
        name: Option<String>,
        
        /// Model version
        #[arg(short, long)]
        version: Option<String>,
        
        /// Account to deploy from
        #[arg(short, long)]
        account: Option<String>,
    },

    /// Run inference on a deployed model
    Inference {
        /// Model ID (hex)
        #[arg(long)]
        model_id: String,
        
        /// Input data file (JSON)
        #[arg(short, long)]
        input: PathBuf,
        
        /// Output file path
        #[arg(short, long)]
        output: Option<PathBuf>,
        
        /// Request proof of inference
        #[arg(long)]
        with_proof: bool,
    },

    /// List deployed models
    List {
        /// Filter by owner address
        #[arg(short, long)]
        owner: Option<String>,
        
        /// Filter by model type
        #[arg(short, long)]
        model_type: Option<String>,
        
        /// Maximum number of results
        #[arg(short, long, default_value = "10")]
        limit: usize,
    },

    /// Get model details
    Info {
        /// Model ID (hex)
        model_id: String,
    },

    /// Update model metadata
    Update {
        /// Model ID (hex)
        model_id: String,
        
        /// New metadata JSON file
        metadata: PathBuf,
        
        /// Account to update from
        #[arg(short, long)]
        account: Option<String>,
    },

    /// Verify a model proof
    Verify {
        /// Proof file path
        proof: PathBuf,
        
        /// Expected output hash (optional)
        #[arg(long)]
        output_hash: Option<String>,
    },
}

pub async fn execute(cmd: ModelCommands, config: &Config) -> Result<()> {
    match cmd {
        ModelCommands::Deploy { model, metadata, name, version, account } => {
            deploy_model(config, model, metadata, name, version, account).await?;
        }
        ModelCommands::Inference { model_id, input, output, with_proof } => {
            run_inference(config, &model_id, input, output, with_proof).await?;
        }
        ModelCommands::List { owner, model_type, limit } => {
            list_models(config, owner, model_type, limit).await?;
        }
        ModelCommands::Info { model_id } => {
            get_model_info(config, &model_id).await?;
        }
        ModelCommands::Update { model_id, metadata, account } => {
            update_model(config, &model_id, metadata, account).await?;
        }
        ModelCommands::Verify { proof, output_hash } => {
            verify_proof(config, proof, output_hash).await?;
        }
    }
    Ok(())
}

async fn deploy_model(
    config: &Config,
    model_path: PathBuf,
    metadata_path: Option<PathBuf>,
    name: Option<String>,
    version: Option<String>,
    account: Option<String>,
) -> Result<()> {
    println!("{}", "Deploying model...".cyan());
    
    // Read model file
    let model_data = fs::read(&model_path)
        .with_context(|| format!("Failed to read model file {:?}", model_path))?;
    
    // Determine model format from extension
    let model_format = model_path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("unknown");
    
    // Read or generate metadata
    let metadata = if let Some(path) = metadata_path {
        let content = fs::read_to_string(path)
            .context("Failed to read metadata file")?;
        serde_json::from_str(&content)?
    } else {
        json!({
            "name": name.unwrap_or_else(|| "Unnamed Model".to_string()),
            "version": version.unwrap_or_else(|| "1.0.0".to_string()),
            "format": model_format,
            "size": model_data.len(),
            "timestamp": chrono::Utc::now().to_rfc3339(),
        })
    };
    
    // Get account address
    let from_account = account.or(config.default_account.clone())
        .context("No account specified and no default account configured")?;
    
    // Prepare deployment transaction
    let model_hash = sha3::Keccak256::digest(&model_data);
    
    // Make RPC call to deploy
    let client = reqwest::Client::new();
    let response = client
        .post(&config.rpc_endpoint)
        .json(&json!({
            "jsonrpc": "2.0",
            "method": "lattice_deployModel",
            "params": {
                "from": from_account,
                "model_data": base64::encode(&model_data),
                "metadata": metadata,
                "gas_price": config.gas_price,
                "gas_limit": config.gas_limit,
            },
            "id": 1
        }))
        .send()
        .await
        .context("Failed to connect to RPC endpoint")?;
    
    let result: serde_json::Value = response.json().await?;
    
    if let Some(model_id) = result["result"]["model_id"].as_str() {
        println!("{}", "✓ Model deployed successfully".green());
        println!("Model ID: {}", model_id.cyan());
        println!("Model Hash: 0x{}", hex::encode(model_hash));
        println!("Transaction: {}", result["result"]["tx_hash"].as_str().unwrap_or("N/A"));
    } else if let Some(error) = result["error"].as_object() {
        anyhow::bail!("Deployment failed: {}", error["message"].as_str().unwrap_or("Unknown error"));
    } else {
        anyhow::bail!("Unexpected response from RPC");
    }
    
    Ok(())
}

async fn run_inference(
    config: &Config,
    model_id: &str,
    input_path: PathBuf,
    output_path: Option<PathBuf>,
    with_proof: bool,
) -> Result<()> {
    println!("{}", "Running inference...".cyan());
    
    // Read input data
    let input_data = fs::read_to_string(&input_path)
        .with_context(|| format!("Failed to read input file {:?}", input_path))?;
    
    let input_json: serde_json::Value = serde_json::from_str(&input_data)
        .context("Invalid JSON input")?;
    
    // Make RPC call for inference
    let client = reqwest::Client::new();
    let response = client
        .post(&config.rpc_endpoint)
        .json(&json!({
            "jsonrpc": "2.0",
            "method": "lattice_runInference",
            "params": {
                "model_id": model_id,
                "input": input_json,
                "with_proof": with_proof,
            },
            "id": 1
        }))
        .send()
        .await
        .context("Failed to connect to RPC endpoint")?;
    
    let result: serde_json::Value = response.json().await?;
    
    if let Some(output) = result["result"]["output"].as_object() {
        println!("{}", "✓ Inference completed successfully".green());
        
        // Save output if path specified
        if let Some(path) = output_path {
            let output_str = serde_json::to_string_pretty(output)?;
            fs::write(&path, output_str)
                .with_context(|| format!("Failed to write output to {:?}", path))?;
            println!("Output saved to: {:?}", path);
        } else {
            println!("Output:");
            println!("{}", serde_json::to_string_pretty(output)?);
        }
        
        if with_proof {
            if let Some(proof) = result["result"]["proof"].as_str() {
                println!("\nProof ID: {}", proof.cyan());
            }
        }
        
        if let Some(exec_time) = result["result"]["execution_time_ms"].as_u64() {
            println!("Execution time: {}ms", exec_time);
        }
    } else if let Some(error) = result["error"].as_object() {
        anyhow::bail!("Inference failed: {}", error["message"].as_str().unwrap_or("Unknown error"));
    } else {
        anyhow::bail!("Unexpected response from RPC");
    }
    
    Ok(())
}

async fn list_models(
    config: &Config,
    owner: Option<String>,
    model_type: Option<String>,
    limit: usize,
) -> Result<()> {
    // Make RPC call to list models
    let client = reqwest::Client::new();
    
    let mut params = json!({
        "limit": limit,
    });
    
    if let Some(owner) = owner {
        params["owner"] = json!(owner);
    }
    if let Some(model_type) = model_type {
        params["type"] = json!(model_type);
    }
    
    let response = client
        .post(&config.rpc_endpoint)
        .json(&json!({
            "jsonrpc": "2.0",
            "method": "lattice_listModels",
            "params": params,
            "id": 1
        }))
        .send()
        .await
        .context("Failed to connect to RPC endpoint")?;
    
    let result: serde_json::Value = response.json().await?;
    
    if let Some(models) = result["result"]["models"].as_array() {
        if models.is_empty() {
            println!("{}", "No models found".yellow());
        } else {
            println!("{}", format!("Found {} model(s):", models.len()).bold());
            println!();
            
            for model in models {
                println!("Model ID: {}", model["id"].as_str().unwrap_or("N/A").cyan());
                println!("  Name: {}", model["name"].as_str().unwrap_or("Unnamed"));
                println!("  Version: {}", model["version"].as_str().unwrap_or("N/A"));
                println!("  Owner: {}", model["owner"].as_str().unwrap_or("N/A"));
                println!("  Type: {}", model["type"].as_str().unwrap_or("Unknown"));
                println!("  Deployed: {}", model["timestamp"].as_str().unwrap_or("N/A"));
                println!();
            }
        }
    } else if let Some(error) = result["error"].as_object() {
        anyhow::bail!("Query failed: {}", error["message"].as_str().unwrap_or("Unknown error"));
    } else {
        anyhow::bail!("Unexpected response from RPC");
    }
    
    Ok(())
}

async fn get_model_info(config: &Config, model_id: &str) -> Result<()> {
    // Make RPC call to get model info
    let client = reqwest::Client::new();
    let response = client
        .post(&config.rpc_endpoint)
        .json(&json!({
            "jsonrpc": "2.0",
            "method": "lattice_getModel",
            "params": {
                "model_id": model_id,
            },
            "id": 1
        }))
        .send()
        .await
        .context("Failed to connect to RPC endpoint")?;
    
    let result: serde_json::Value = response.json().await?;
    
    if let Some(model) = result["result"]["model"].as_object() {
        println!("{}", "Model Information:".bold());
        println!("{}", serde_json::to_string_pretty(model)?);
    } else if let Some(error) = result["error"].as_object() {
        anyhow::bail!("Query failed: {}", error["message"].as_str().unwrap_or("Unknown error"));
    } else {
        anyhow::bail!("Model not found");
    }
    
    Ok(())
}

async fn update_model(
    config: &Config,
    model_id: &str,
    metadata_path: PathBuf,
    account: Option<String>,
) -> Result<()> {
    println!("{}", "Updating model metadata...".cyan());
    
    // Read metadata
    let metadata_str = fs::read_to_string(&metadata_path)
        .with_context(|| format!("Failed to read metadata file {:?}", metadata_path))?;
    
    let metadata: serde_json::Value = serde_json::from_str(&metadata_str)
        .context("Invalid JSON metadata")?;
    
    // Get account
    let from_account = account.or(config.default_account.clone())
        .context("No account specified and no default account configured")?;
    
    // Make RPC call to update
    let client = reqwest::Client::new();
    let response = client
        .post(&config.rpc_endpoint)
        .json(&json!({
            "jsonrpc": "2.0",
            "method": "lattice_updateModel",
            "params": {
                "model_id": model_id,
                "metadata": metadata,
                "from": from_account,
            },
            "id": 1
        }))
        .send()
        .await
        .context("Failed to connect to RPC endpoint")?;
    
    let result: serde_json::Value = response.json().await?;
    
    if result["result"]["success"].as_bool() == Some(true) {
        println!("{}", "✓ Model updated successfully".green());
        println!("Transaction: {}", result["result"]["tx_hash"].as_str().unwrap_or("N/A"));
    } else if let Some(error) = result["error"].as_object() {
        anyhow::bail!("Update failed: {}", error["message"].as_str().unwrap_or("Unknown error"));
    } else {
        anyhow::bail!("Unexpected response from RPC");
    }
    
    Ok(())
}

async fn verify_proof(
    config: &Config,
    proof_path: PathBuf,
    output_hash: Option<String>,
) -> Result<()> {
    println!("{}", "Verifying proof...".cyan());
    
    // Read proof file
    let proof_data = fs::read_to_string(&proof_path)
        .with_context(|| format!("Failed to read proof file {:?}", proof_path))?;
    
    let proof_json: serde_json::Value = serde_json::from_str(&proof_data)
        .context("Invalid JSON proof")?;
    
    // Make RPC call to verify
    let client = reqwest::Client::new();
    
    let mut params = json!({
        "proof": proof_json,
    });
    
    if let Some(hash) = output_hash {
        params["output_hash"] = json!(hash);
    }
    
    let response = client
        .post(&config.rpc_endpoint)
        .json(&json!({
            "jsonrpc": "2.0",
            "method": "lattice_verifyProof",
            "params": params,
            "id": 1
        }))
        .send()
        .await
        .context("Failed to connect to RPC endpoint")?;
    
    let result: serde_json::Value = response.json().await?;
    
    if let Some(valid) = result["result"]["valid"].as_bool() {
        if valid {
            println!("{}", "✓ Proof is VALID".green().bold());
            
            if let Some(details) = result["result"]["details"].as_object() {
                println!("\nProof Details:");
                println!("  Model ID: {}", details["model_id"].as_str().unwrap_or("N/A"));
                println!("  Execution ID: {}", details["execution_id"].as_str().unwrap_or("N/A"));
                println!("  Timestamp: {}", details["timestamp"].as_str().unwrap_or("N/A"));
                
                if let Some(computed_hash) = details["output_hash"].as_str() {
                    println!("  Output Hash: {}", computed_hash);
                    
                    if let Some(expected) = output_hash {
                        if computed_hash == expected {
                            println!("  {}", "✓ Output hash matches expected value".green());
                        } else {
                            println!("  {}", "✗ Output hash does not match expected value".red());
                        }
                    }
                }
            }
        } else {
            println!("{}", "✗ Proof is INVALID".red().bold());
            
            if let Some(reason) = result["result"]["reason"].as_str() {
                println!("Reason: {}", reason);
            }
        }
    } else if let Some(error) = result["error"].as_object() {
        anyhow::bail!("Verification failed: {}", error["message"].as_str().unwrap_or("Unknown error"));
    } else {
        anyhow::bail!("Unexpected response from RPC");
    }
    
    Ok(())
}