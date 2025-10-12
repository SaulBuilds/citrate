use anyhow::{Context, Result};
use clap::Subcommand;
use colored::Colorize;
use serde_json::json;
use std::fs;
use std::path::PathBuf;

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

        /// Optional runtime bytecode (hex). If not provided, RPC can compile source if configured.
        #[arg(long)]
        runtime_bytecode: Option<String>,

        /// Optional path to runtime bytecode file (hex). Used if --runtime-bytecode not set.
        #[arg(long)]
        runtime_bytecode_file: Option<PathBuf>,

        /// Read verification record for an address instead of submitting (alias: --get <address>)
        #[arg(long)]
        get: Option<String>,
    },

    /// Get a verification record (alias for --get)
    VerifyGet {
        /// Contract address
        address: String,
    },

    /// List all verification records known to the node
    VerifyList,
}

pub async fn execute(cmd: ContractCommands, config: &Config) -> Result<()> {
    match cmd {
        ContractCommands::Deploy {
            contract,
            args,
            account,
            value,
        } => {
            deploy_contract(config, contract, args, account, value).await?;
        }
        ContractCommands::Call {
            address,
            method,
            args,
            account,
            value,
        } => {
            call_contract(config, &address, &method, args, account, value).await?;
        }
        ContractCommands::Read {
            address,
            method,
            args,
        } => {
            read_contract(config, &address, &method, args).await?;
        }
        ContractCommands::Code { address, output } => {
            get_contract_code(config, &address, output).await?;
        }
        ContractCommands::Verify {
            address,
            source,
            compiler,
            optimized,
            runtime_bytecode,
            runtime_bytecode_file,
            get,
        } => {
            if let Some(addr) = get {
                get_verification(config, &addr).await?;
            } else {
                verify_contract(
                    config,
                    &address,
                    source,
                    &compiler,
                    optimized,
                    runtime_bytecode,
                    runtime_bytecode_file,
                )
                .await?;
            }
        }
        ContractCommands::VerifyGet { address } => {
            get_verification(config, &address).await?;
        }
        ContractCommands::VerifyList => {
            list_verifications(config).await?;
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
    let _constructor_args = if let Some(args_str) = args {
        serde_json::from_str(&args_str).context("Invalid constructor arguments JSON")?
    } else {
        json!([])
    };

    // Get account
    let from_account = account
        .or(config.default_account.clone())
        .context("No account specified and no default account configured")?;

    // Parse value
    let value_wei = value.parse::<u128>().context("Invalid value")?;

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
        anyhow::bail!(
            "Deployment failed: {}",
            error["message"].as_str().unwrap_or("Unknown error")
        );
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
    let from_account = account
        .or(config.default_account.clone())
        .context("No account specified and no default account configured")?;

    // Parse value
    let value_wei = value.parse::<u128>().context("Invalid value")?;

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
        anyhow::bail!(
            "Call failed: {}",
            error["message"].as_str().unwrap_or("Unknown error")
        );
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
                    if text
                        .chars()
                        .all(|c| c.is_ascii_graphic() || c.is_ascii_whitespace())
                    {
                        println!("Decoded (string): {}", text.trim());
                    }
                }
            }
        }
    } else if let Some(error) = result["error"].as_object() {
        anyhow::bail!(
            "Call failed: {}",
            error["message"].as_str().unwrap_or("Unknown error")
        );
    } else {
        anyhow::bail!("Unexpected response from RPC");
    }

    Ok(())
}

async fn get_contract_code(config: &Config, address: &str, output: Option<PathBuf>) -> Result<()> {
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
            let code_bytes = hex::decode(&code[2..]).context("Invalid code format")?;

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
        anyhow::bail!(
            "Query failed: {}",
            error["message"].as_str().unwrap_or("Unknown error")
        );
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
    runtime_bytecode: Option<String>,
    runtime_bytecode_file: Option<PathBuf>,
) -> Result<()> {
    println!("{}", "Verifying contract...".cyan());

    // Read source code
    let source_code = fs::read_to_string(&source_path)
        .with_context(|| format!("Failed to read source file {:?}", source_path))?;

    // Determine runtime bytecode input if provided
    let mut runtime_bc_hex: Option<String> = None;
    if let Some(hex_in) = runtime_bytecode {
        let s = hex_in.trim();
        let s = if s.starts_with("0x") {
            s.to_string()
        } else {
            format!("0x{}", s)
        };
        runtime_bc_hex = Some(s);
    } else if let Some(path) = runtime_bytecode_file {
        let content = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read runtime bytecode file {:?}", path))?;
        let s = content.trim();
        let s = if s.starts_with("0x") {
            s.to_string()
        } else {
            format!("0x{}", s)
        };
        runtime_bc_hex = Some(s);
    }

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
                "runtime_bytecode": runtime_bc_hex,
            },
            "id": 1
        }))
        .send()
        .await
        .context("Failed to connect to RPC endpoint")?;

    let result: serde_json::Value = response.json().await?;

    if result["result"]["verified"].as_bool() == Some(true) {
        println!("{}", "✓ Contract verified successfully".green().bold());
        println!(
            "Verification ID: {}",
            result["result"]["verification_id"]
                .as_str()
                .unwrap_or("N/A")
        );
    } else if let Some(error) = result["error"].as_object() {
        anyhow::bail!(
            "Verification failed: {}",
            error["message"].as_str().unwrap_or("Unknown error")
        );
    } else {
        anyhow::bail!("Contract verification failed");
    }

    Ok(())
}

async fn get_verification(config: &Config, address: &str) -> Result<()> {
    let client = reqwest::Client::new();
    let response = client
        .post(&config.rpc_endpoint)
        .json(&json!({
            "jsonrpc": "2.0",
            "method": "lattice_getVerification",
            "params": [address],
            "id": 1
        }))
        .send()
        .await
        .context("Failed to connect to RPC endpoint")?;

    let v: serde_json::Value = response.json().await?;
    if let Some(rec) = v["result"].as_object() {
        println!("{}", "Verification Record:".bold());
        println!(
            "Address: {}",
            rec.get("address").and_then(|s| s.as_str()).unwrap_or("")
        );
        println!(
            "Verified: {}",
            rec.get("verified")
                .and_then(|b| b.as_bool())
                .unwrap_or(false)
        );
        if let Some(id) = rec.get("verification_id").and_then(|s| s.as_str()) {
            println!("ID: {}", id);
        }
        if let Some(ts) = rec.get("timestamp").and_then(|s| s.as_str()) {
            println!("Timestamp: {}", ts);
        }
        Ok(())
    } else {
        println!("No verification record for {}", address);
        Ok(())
    }
}

async fn list_verifications(config: &Config) -> Result<()> {
    let client = reqwest::Client::new();
    let response = client
        .post(&config.rpc_endpoint)
        .json(&json!({
            "jsonrpc": "2.0",
            "method": "lattice_listVerifications",
            "params": [],
            "id": 1
        }))
        .send()
        .await
        .context("Failed to connect to RPC endpoint")?;

    let v: serde_json::Value = response.json().await?;
    if let Some(arr) = v["result"].as_array() {
        println!("{} {}", arr.len(), "verification record(s)".bold());
        for rec in arr {
            if let Some(addr) = rec["address"].as_str() {
                let status = rec["verified"].as_bool().unwrap_or(false);
                println!("- {} (verified: {})", addr, status);
            }
        }
    } else {
        println!("No verification records found");
    }
    Ok(())
}

pub async fn wait_for_receipt(config: &Config, tx_hash: &str) -> Result<serde_json::Value> {
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

fn encode_method_call(method_sig: &str, args: serde_json::Value) -> Result<String> {
    use anyhow::bail;
    use sha3::{Digest, Keccak256};

    // Compute 4-byte selector from full signature
    let mut hasher = Keccak256::new();
    hasher.update(method_sig.as_bytes());
    let hash = hasher.finalize();
    let selector = &hash[..4];

    // Parse types from signature, e.g. "transfer(address,uint256)"
    let open = method_sig.find('(').context(
        "Invalid method signature: missing '
'",
    )?;
    let close = method_sig
        .rfind(')')
        .context("Invalid method signature: missing ')'")?;
    let types_str = &method_sig[open + 1..close];
    let types: Vec<String> = if types_str.trim().is_empty() {
        vec![]
    } else {
        types_str
            .split(',')
            .map(|s| s.trim().to_lowercase())
            .collect()
    };

    // Args must be an array
    let args_arr = match args {
        serde_json::Value::Array(v) => v,
        serde_json::Value::Null => vec![],
        _ => bail!("Method arguments must be a JSON array"),
    };

    if args_arr.len() != types.len() {
        bail!(
            "Argument count mismatch: expected {}, got {}",
            types.len(),
            args_arr.len()
        );
    }

    // Encode static arguments (subset of ABI: address, uint{8,16,32,64,128,256}, bool, bytes32)
    let mut encoded: Vec<u8> = Vec::new();
    for (ty, val) in types.iter().zip(args_arr.into_iter()) {
        match ty.as_str() {
            // address: 20 bytes, left-padded to 32
            "address" => {
                let s = val.as_str().context("address must be a hex string")?;
                let hexstr = s.trim().trim_start_matches("0x");
                let bytes = hex::decode(hexstr).context("invalid address hex")?;
                if bytes.len() != 20 {
                    bail!("address must be 20 bytes (40 hex chars)");
                }
                let mut word = [0u8; 32];
                word[12..32].copy_from_slice(&bytes);
                encoded.extend_from_slice(&word);
            }
            // bool: 0 or 1 in last byte, left-padded
            "bool" => {
                let b = match val {
                    serde_json::Value::Bool(b) => b,
                    serde_json::Value::Number(n) => n.as_u64().unwrap_or(0) != 0,
                    serde_json::Value::String(s) => {
                        let sl = s.to_lowercase();
                        sl == "true" || sl == "1"
                    }
                    _ => bail!("bool must be true/false or 0/1"),
                };
                let mut word = [0u8; 32];
                if b {
                    word[31] = 1u8;
                }
                encoded.extend_from_slice(&word);
            }
            // bytes32: fixed 32 bytes (or shorter and right-padded with zeros)
            "bytes32" => {
                let s = val.as_str().context("bytes32 must be a hex string")?;
                let hexstr = s.trim().trim_start_matches("0x");
                let mut bytes = hex::decode(hexstr).context("invalid bytes32 hex")?;
                if bytes.len() > 32 {
                    bail!("bytes32 must be <= 32 bytes");
                }
                bytes.resize(32, 0u8);
                encoded.extend_from_slice(&bytes);
            }
            // uint types
            ty if ty.starts_with("uint") => {
                // Determine bit size (default 256)
                let bits: u16 = ty[4..].parse().unwrap_or(256);
                if bits == 0 || bits % 8 != 0 || bits > 256 {
                    bail!("unsupported uint size: {}", bits);
                }
                // Accept number or hex string
                let mut word = [0u8; 32];
                match val {
                    serde_json::Value::Number(n) => {
                        // Encode as big-endian; support up to u128 directly
                        if let Some(u) = n.as_u64() {
                            let be = (u as u128).to_be_bytes();
                            word[16..32].copy_from_slice(&be);
                        } else {
                            // Fallback: parse string representation
                            bail!("uint must be a positive integer within u64 for now");
                        }
                    }
                    serde_json::Value::String(s) => {
                        let ss = s.trim();
                        if let Some(stripped) = ss.strip_prefix("0x") {
                            let bytes = hex::decode(stripped).context("invalid uint hex")?;
                            if bytes.len() > 32 {
                                bail!("uint hex too large (max 32 bytes)");
                            }
                            word[32 - bytes.len()..32].copy_from_slice(&bytes);
                        } else {
                            // Decimal string, parse into u128
                            let val = ss.parse::<u128>().context("invalid uint decimal string")?;
                            let be = val.to_be_bytes();
                            word[16..32].copy_from_slice(&be);
                        }
                    }
                    _ => bail!("uint must be a number or string (hex or decimal)"),
                }
                encoded.extend_from_slice(&word);
            }
            // unsupported dynamic types
            ty if ty == "string" || ty == "bytes" => {
                bail!("dynamic types (string, bytes) are not supported in this CLI encoder yet")
            }
            other => bail!("unsupported or unknown type in signature: {}", other),
        }
    }

    let mut out = Vec::with_capacity(4 + encoded.len());
    out.extend_from_slice(selector);
    out.extend_from_slice(&encoded);
    Ok(format!("0x{}", hex::encode(out)))
}

#[cfg(test)]
mod tests {
    use super::encode_method_call;
    use sha3::{Digest, Keccak256};

    #[test]
    fn test_encode_transfer_address_uint256() {
        let addr = "0x1111111111111111111111111111111111111111";
        let json = serde_json::json!([addr, "1000"]);
        let sig = "transfer(address,uint256)";
        let out = encode_method_call(sig, json).expect("encode");

        // selector correctness
        let mut hasher = Keccak256::new();
        hasher.update(sig.as_bytes());
        let selector = hex::encode(&hasher.finalize()[..4]);
        assert!(out.starts_with(&format!("0x{}", selector)));

        // length = 4 bytes selector + 2 words
        // hex length: 2 chars per byte, plus 2 for '0x'
        let expected_hex_len = 2 + (4 + 32 + 32) * 2;
        assert_eq!(out.len(), expected_hex_len);

        // address should be right-padded in last 40 hex of the first word after selector
        // Skip '0x' + 8 selector hex = 10 chars; next 64 hex is first word
        let first_word_hex = &out[10..10 + 64];
        // last 40 of word equals address without 0x
        assert_eq!(&first_word_hex[24..], &addr[2..]);
    }

    #[test]
    fn test_encode_bool_and_bytes32() {
        let bytes32 = "0x".to_string() + &"aa".repeat(32);
        let json = serde_json::json!([true, bytes32]);
        let sig = "setFlagAndHash(bool,bytes32)";
        let out = encode_method_call(sig, json).expect("encode");

        // Expect selector + 2 words
        let expected_hex_len = 2 + (4 + 32 + 32) * 2;
        assert_eq!(out.len(), expected_hex_len);

        // Bool word ends with 01
        let bool_word = &out[10..10 + 64];
        assert!(bool_word.ends_with("01"));
    }
}
