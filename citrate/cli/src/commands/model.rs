//citrate/cli/src/commands/model.rs

use anyhow::{Context, Result};
use base64::{engine::general_purpose::STANDARD, Engine};
use clap::Subcommand;
use colored::Colorize;
use serde_json::json;
use sha3::Digest;
use std::fs;
use std::path::PathBuf;
use std::time::Duration;
use tokio::time::sleep;

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

        /// Access policy (public | private | restricted | payPerUse)
        #[arg(long, default_value = "public")]
        access_policy: String,

        /// Inference price (only for payPerUse policy)
        #[arg(long)]
        price: Option<String>,
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

        /// Optional updated model file (ONNX/weights)
        #[arg(long)]
        model: Option<PathBuf>,

        /// Provide existing artifact CID instead of uploading
        #[arg(long)]
        cid: Option<String>,
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
        ModelCommands::Deploy {
            model,
            metadata,
            name,
            version,
            account,
            access_policy,
            price,
        } => {
            deploy_model(
                config,
                model,
                metadata,
                name,
                version,
                account,
                access_policy,
                price,
            )
            .await?;
        }
        ModelCommands::Inference {
            model_id,
            input,
            output,
            with_proof,
        } => {
            run_inference(config, &model_id, input, output, with_proof).await?;
        }
        ModelCommands::List {
            owner,
            model_type,
            limit,
        } => {
            list_models(config, owner, model_type, limit).await?;
        }
        ModelCommands::Info { model_id } => {
            get_model_info(config, &model_id).await?;
        }
        ModelCommands::Update {
            model_id,
            metadata,
            account,
            model,
            cid,
        } => {
            update_model(config, &model_id, metadata, model, cid, account).await?;
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
    access_policy: String,
    price: Option<String>,
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
    let mut metadata = if let Some(path) = metadata_path {
        let content = fs::read_to_string(path).context("Failed to read metadata file")?;
        serde_json::from_str(&content)?
    } else {
        json!({
            "name": name.as_ref().map(|s| s.as_str()).unwrap_or("Unnamed Model"),
            "version": version.as_ref().map(|s| s.as_str()).unwrap_or("1.0.0"),
            "format": model_format,
            "size": model_data.len(),
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "description": format!("{} model", model_format.to_uppercase()),
            "framework": model_format,
            "input_shape": [1],
            "output_shape": [1],
        })
    };

    let metadata_obj = metadata
        .as_object_mut()
        .ok_or_else(|| anyhow::anyhow!("Metadata JSON must be an object"))?;

    if let Some(name) = name {
        metadata_obj.insert("name".to_string(), json!(name));
    } else {
        metadata_obj
            .entry("name".to_string())
            .or_insert(json!("Unnamed Model"));
    }

    if let Some(version) = version {
        metadata_obj.insert("version".to_string(), json!(version));
    } else {
        metadata_obj
            .entry("version".to_string())
            .or_insert(json!("1.0.0"));
    }

    metadata_obj.insert("framework".to_string(), json!(model_format));
    metadata_obj
        .entry("description".to_string())
        .or_insert(json!(format!("{} model", model_format.to_uppercase())));
    metadata_obj
        .entry("input_shape".to_string())
        .or_insert(json!([1]));
    metadata_obj
        .entry("output_shape".to_string())
        .or_insert(json!([1]));
    metadata_obj.insert("size_bytes".to_string(), json!(model_data.len()));

    // Get account address
    let from_account = account
        .or(config.default_account.clone())
        .context("No account specified and no default account configured")?;

    // Prepare deployment transaction
    let model_hash = sha3::Keccak256::digest(&model_data);
    let model_hash_hex = format!("0x{}", hex::encode(model_hash));
    let size_bytes = model_data.len() as u64;
    let model_b64 = base64::engine::general_purpose::STANDARD.encode(&model_data);

    // Make RPC call to deploy
    let client = reqwest::Client::new();
    let mut params = serde_json::Map::new();
    params.insert("from".to_string(), json!(from_account));
    params.insert("model_data".to_string(), json!(model_b64));
    params.insert("metadata".to_string(), metadata);
    params.insert("model_hash".to_string(), json!(model_hash_hex));
    params.insert("size_bytes".to_string(), json!(size_bytes));
    params.insert("access_policy".to_string(), json!(access_policy));
    params.insert("gas_price".to_string(), json!(config.gas_price));
    params.insert("gas_limit".to_string(), json!(config.gas_limit));
    if let Some(price) = price {
        params.insert("inference_price".to_string(), json!(price));
    }

    let response = client
        .post(&config.rpc_endpoint)
        .json(&json!({
            "jsonrpc": "2.0",
            "method": "citrate_deployModel",
            "params": params,
            "id": 1
        }))
        .send()
        .await
        .context("Failed to connect to RPC endpoint")?;

    let result: serde_json::Value = response.json().await?;

    if let Some(res) = result.get("result").and_then(|v| v.as_object()) {
        let tx_hash = res
            .get("tx_hash")
            .and_then(|v| v.as_str())
            .context("RPC response missing tx_hash")?;

        println!("{}", "✓ Transaction submitted".green());
        println!("Transaction: {}", tx_hash.cyan());

        if let Some(model_id) = res.get("model_id").and_then(|v| v.as_str()) {
            println!("Model ID: {}", model_id.cyan());
        }
        if let Some(cid) = res.get("artifact_cid").and_then(|v| v.as_str()) {
            println!("Artifact CID: {}", cid.cyan());
        }
        println!("Model Hash: 0x{}", hex::encode(model_hash));

        println!("Waiting for confirmation...");
        let receipt = wait_for_receipt(config, tx_hash).await?;

        let status = receipt["status"].as_str().unwrap_or_default();
        if status == "0x1" || status == "0x01" {
            println!("{}", "✓ Model deployment confirmed".green().bold());
            if let Some(block) = receipt["blockNumber"].as_str() {
                println!("Included in block: {}", block);
            }
            if let Some(gas_used) = receipt["gasUsed"].as_str() {
                println!("Gas Used: {}", gas_used);
            }
            println!(
                "{}",
                "Model registered on-chain. You can query it via `lattice_getModel`."
                    .italic()
            );
        } else {
            println!("{}", "✗ Model deployment reverted".red().bold());
            println!("{}", serde_json::to_string_pretty(&receipt)?);
            anyhow::bail!("Model deployment transaction failed");
        }
    } else if let Some(error) = result["error"].as_object() {
        anyhow::bail!(
            "Deployment failed: {}",
            error["message"].as_str().unwrap_or("Unknown error")
        );
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

    let input_json: serde_json::Value =
        serde_json::from_str(&input_data).context("Invalid JSON input")?;

    // Make RPC call for inference
    let client = reqwest::Client::new();
    let response = client
        .post(&config.rpc_endpoint)
        .json(&json!({
            "jsonrpc": "2.0",
            "method": "citrate_runInference",
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
        anyhow::bail!(
            "Inference failed: {}",
            error["message"].as_str().unwrap_or("Unknown error")
        );
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
            "method": "citrate_listModels",
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
                println!(
                    "  Deployed: {}",
                    model["timestamp"].as_str().unwrap_or("N/A")
                );
                println!();
            }
        }
    } else if let Some(error) = result["error"].as_object() {
        anyhow::bail!(
            "Query failed: {}",
            error["message"].as_str().unwrap_or("Unknown error")
        );
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
            "method": "citrate_getModel",
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
        anyhow::bail!(
            "Query failed: {}",
            error["message"].as_str().unwrap_or("Unknown error")
        );
    } else {
        anyhow::bail!("Model not found");
    }

    Ok(())
}

async fn update_model(
    config: &Config,
    model_id: &str,
    metadata_path: PathBuf,
    model_path: Option<PathBuf>,
    cid_override: Option<String>,
    account: Option<String>,
) -> Result<()> {
    println!("{}", "Updating model metadata...".cyan());

    // Read metadata
    let metadata_str = fs::read_to_string(&metadata_path)
        .with_context(|| format!("Failed to read metadata file {:?}", metadata_path))?;

    let mut metadata_value: serde_json::Value =
        serde_json::from_str(&metadata_str).context("Invalid JSON metadata")?;
    let metadata_obj = metadata_value
        .as_object_mut()
        .ok_or_else(|| anyhow::anyhow!("Metadata JSON must be an object"))?;

    metadata_obj
        .entry("name".to_string())
        .or_insert(json!("Updated Model"));
    metadata_obj
        .entry("version".to_string())
        .or_insert(json!("1.0.1"));
    metadata_obj
        .entry("description".to_string())
        .or_insert(json!("Updated model"));
    metadata_obj
        .entry("framework".to_string())
        .or_insert(json!("Unknown"));
    metadata_obj
        .entry("input_shape".to_string())
        .or_insert(json!([1]));
    metadata_obj
        .entry("output_shape".to_string())
        .or_insert(json!([1]));

    let mut model_data_b64: Option<String> = None;

    if let Some(path) = model_path {
        let bytes = fs::read(&path)
            .with_context(|| format!("Failed to read updated model file {:?}", path))?;
        metadata_obj.insert("size_bytes".to_string(), json!(bytes.len()));
        model_data_b64 = Some(STANDARD.encode(bytes));
    }

    if !metadata_obj.contains_key("size_bytes") {
        metadata_obj.insert("size_bytes".to_string(), json!(0));
    }

    // Get account
    let from_account = account
        .or(config.default_account.clone())
        .context("No account specified and no default account configured")?;

    let mut params = serde_json::Map::new();
    params.insert("model_id".to_string(), json!(model_id));
    params.insert("metadata".to_string(), serde_json::Value::Object(metadata_obj.clone()));
    params.insert("from".to_string(), json!(from_account));

    if let Some(data_b64) = model_data_b64 {
        params.insert("model_data".to_string(), json!(data_b64));
    }

    if let Some(cid) = cid_override {
        params.insert("ipfs_cid".to_string(), json!(cid));
    }

    params.insert("gas_price".to_string(), json!(config.gas_price));
    params.insert("gas_limit".to_string(), json!(config.gas_limit));

    // Make RPC call to update
    let client = reqwest::Client::new();
    let response = client
        .post(&config.rpc_endpoint)
        .json(&json!({
            "jsonrpc": "2.0",
            "method": "citrate_updateModel",
            "params": params,
            "id": 1
        }))
        .send()
        .await
        .context("Failed to connect to RPC endpoint")?;

    let result: serde_json::Value = response.json().await?;

    if let Some(res) = result.get("result").and_then(|v| v.as_object()) {
        let tx_hash = res
            .get("tx_hash")
            .and_then(|v| v.as_str())
            .context("RPC response missing tx_hash")?;

        println!("{}", "✓ Transaction submitted".green());
        println!("Transaction: {}", tx_hash.cyan());
        if let Some(cid) = res.get("artifact_cid").and_then(|v| v.as_str()) {
            println!("Artifact CID: {}", cid.cyan());
        }
        if let Some(model_hex) = res.get("model_id").and_then(|v| v.as_str()) {
            println!("Model ID: {}", model_hex.cyan());
        }

        println!("Waiting for confirmation...");
        let receipt = wait_for_receipt(config, tx_hash).await?;
        let status = receipt["status"].as_str().unwrap_or_default();
        if status == "0x1" || status == "0x01" {
            println!("{}", "✓ Model update confirmed".green().bold());
            if let Some(block) = receipt["blockNumber"].as_str() {
                println!("Included in block: {}", block);
            }
            if let Some(gas_used) = receipt["gasUsed"].as_str() {
                println!("Gas Used: {}", gas_used);
            }
        } else {
            println!("{}", "✗ Model update reverted".red().bold());
            println!("{}", serde_json::to_string_pretty(&receipt)?);
            anyhow::bail!("Model update transaction failed");
        }
    } else if let Some(error) = result["error"].as_object() {
        anyhow::bail!(
            "Update failed: {}",
            error["message"].as_str().unwrap_or("Unknown error")
        );
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

    let proof_json: serde_json::Value =
        serde_json::from_str(&proof_data).context("Invalid JSON proof")?;

    // Make RPC call to verify
    let client = reqwest::Client::new();

    let mut params = json!({
        "proof": proof_json,
    });

    if let Some(hash) = output_hash.as_ref() {
        params["output_hash"] = json!(hash);
    }

    let response = client
        .post(&config.rpc_endpoint)
        .json(&json!({
            "jsonrpc": "2.0",
            "method": "citrate_verifyProof",
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
                println!(
                    "  Model ID: {}",
                    details["model_id"].as_str().unwrap_or("N/A")
                );
                println!(
                    "  Execution ID: {}",
                    details["execution_id"].as_str().unwrap_or("N/A")
                );
                println!(
                    "  Timestamp: {}",
                    details["timestamp"].as_str().unwrap_or("N/A")
                );

                if let Some(computed_hash) = details["output_hash"].as_str() {
                    println!("  Output Hash: {}", computed_hash);

                    if let Some(expected) = output_hash.as_ref() {
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
        anyhow::bail!(
            "Verification failed: {}",
            error["message"].as_str().unwrap_or("Unknown error")
        );
    } else {
        anyhow::bail!("Unexpected response from RPC");
    }

    Ok(())
}

async fn wait_for_receipt(config: &Config, tx_hash: &str) -> Result<serde_json::Value> {
    let client = reqwest::Client::new();
    let mut attempts = 0;
    const MAX_ATTEMPTS: usize = 30;

    loop {
        attempts += 1;

        let response = client
            .post(&config.rpc_endpoint)
            .json(&json!({
                "jsonrpc": "2.0",
                "method": "eth_getTransactionReceipt",
                "params": [tx_hash],
                "id": 1
            }))
            .send()
            .await
            .context("Failed to connect to RPC endpoint")?;

        let result: serde_json::Value = response.json().await?;

        if let Some(receipt) = result["result"].as_object() {
            return Ok(serde_json::Value::Object(receipt.clone()));
        }

        if attempts >= MAX_ATTEMPTS {
            anyhow::bail!("Transaction receipt not found after {} attempts", MAX_ATTEMPTS);
        }

        sleep(Duration::from_secs(2)).await;
    }
}
