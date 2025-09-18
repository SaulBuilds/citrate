use anyhow::{Context, Result};
use clap::Subcommand;
use colored::Colorize;
use serde_json::json;

use crate::config::Config;

#[derive(Subcommand)]
pub enum GovernanceCommands {
    /// Set governance admin (address hex)
    SetAdmin { address: String },

    /// Queue parameter update with timelock
    QueueParam { key: String, value: String, eta: u64 },

    /// Execute parameter update after timelock
    ExecuteParam { key: String },

    /// Get parameter value
    GetParam { key: String },
}

pub async fn execute(cmd: GovernanceCommands, config: &Config) -> Result<()> {
    match cmd {
        GovernanceCommands::SetAdmin { address } => {
            call_precompile(config, encode_set_admin(&address)?).await?;
            println!("{}", "✓ Admin updated".green());
        }
        GovernanceCommands::QueueParam { key, value, eta } => {
            call_precompile(config, encode_queue_param(&key, &value, eta)?).await?;
            println!("{}", "✓ Parameter queued".green());
        }
        GovernanceCommands::ExecuteParam { key } => {
            call_precompile(config, encode_execute_param(&key)?).await?;
            println!("{}", "✓ Parameter executed".green());
        }
        GovernanceCommands::GetParam { key } => {
            let out = eth_call_precompile(config, encode_get_param(&key)?).await?;
            println!("{} {}", "Param bytes:".cyan(), out);
        }
    }
    Ok(())
}

fn precompile_address() -> String { "0x0000000000000000000000000000000000001003".to_string() }

fn selector(sig: &str) -> [u8;4] {
    use sha3::{Digest, Keccak256};
    let d = Keccak256::digest(sig.as_bytes());
    [d[0], d[1], d[2], d[3]]
}

fn pad32(mut v: Vec<u8>) -> Vec<u8> {
    let rem = v.len() % 32;
    if rem != 0 { v.extend(vec![0u8; 32 - rem]); }
    v
}

fn encode_address(addr_hex: &str) -> Result<[u8;32]> {
    let s = addr_hex.trim_start_matches("0x");
    let bytes = hex::decode(s).context("Invalid address hex")?;
    let mut out = [0u8;32];
    if bytes.len() == 20 { out[12..32].copy_from_slice(&bytes[..20]); }
    else if bytes.len() == 32 { out.copy_from_slice(&bytes[..32]); }
    else { anyhow::bail!("Address must be 20 or 32 bytes"); }
    Ok(out)
}

fn encode_bytes(b: &[u8]) -> Vec<u8> {
    // ABI dynamic bytes: length(32) + data + padding
    let mut out = Vec::new();
    let mut len = [0u8;32];
    let l = b.len() as u64;
    len[24..32].copy_from_slice(&l.to_be_bytes());
    out.extend_from_slice(&len);
    out.extend_from_slice(b);
    pad32(out)
}

fn encode_set_admin(addr_hex: &str) -> Result<String> {
    let mut data = Vec::new();
    data.extend_from_slice(&selector("setAdmin(address)"));
    data.extend_from_slice(&encode_address(addr_hex)?);
    Ok(format!("0x{}", hex::encode(data)))
}

fn encode_queue_param(key: &str, value: &str, eta: u64) -> Result<String> {
    // key as bytes32 (UTF-8 padded/truncated)
    let mut key32 = [0u8;32];
    let kb = key.as_bytes();
    let n = kb.len().min(32);
    key32[..n].copy_from_slice(&kb[..n]);

    // ABI: selector | key32 | offset(bytes) | eta(32) | dynamic bytes
    let mut data = Vec::new();
    data.extend_from_slice(&selector("queueSetParam(bytes32,bytes,uint64)"));
    data.extend_from_slice(&key32);
    // offset = 0x40
    data.extend_from_slice(&{
        let mut off=[0u8;32]; off[31]=0x40; off
    });
    let mut eta_be = [0u8;32]; eta_be[24..32].copy_from_slice(&eta.to_be_bytes());
    data.extend_from_slice(&eta_be);
    // dynamic bytes encoding
    let vb = encode_bytes(value.as_bytes());
    data.extend_from_slice(&vb);
    Ok(format!("0x{}", hex::encode(data)))
}

fn encode_execute_param(key: &str) -> Result<String> {
    let mut key32 = [0u8;32];
    let kb = key.as_bytes();
    let n = kb.len().min(32);
    key32[..n].copy_from_slice(&kb[..n]);
    let mut data = Vec::new();
    data.extend_from_slice(&selector("executeSetParam(bytes32)"));
    data.extend_from_slice(&key32);
    Ok(format!("0x{}", hex::encode(data)))
}

fn encode_get_param(key: &str) -> Result<String> {
    let mut key32 = [0u8;32];
    let kb = key.as_bytes();
    let n = kb.len().min(32);
    key32[..n].copy_from_slice(&kb[..n]);
    let mut data = Vec::new();
    data.extend_from_slice(&selector("getParam(bytes32)"));
    data.extend_from_slice(&key32);
    Ok(format!("0x{}", hex::encode(data)))
}

async fn call_precompile(config: &Config, data_hex: String) -> Result<()> {
    let to = precompile_address();
    let resp = reqwest::Client::new()
        .post(&config.rpc_endpoint)
        .json(&json!({
            "jsonrpc":"2.0",
            "method":"eth_sendTransaction",
            "params":[{
                "from": config.default_account.as_ref().context("Missing default account")?,
                "to": to,
                "data": data_hex,
                "gas": format!("0x{:x}", config.gas_limit.max(300000)),
                "gasPrice": format!("0x{:x}", config.gas_price.max(1_000_000_000)),
            }],
            "id": 1
        }))
        .send().await?;
    let v: serde_json::Value = resp.json().await?;
    if v.get("error").is_some() { anyhow::bail!(v.to_string()); }
    Ok(())
}

async fn eth_call_precompile(config: &Config, data_hex: String) -> Result<String> {
    let to = precompile_address();
    let resp = reqwest::Client::new()
        .post(&config.rpc_endpoint)
        .json(&json!({
            "jsonrpc":"2.0",
            "method":"eth_call",
            "params":[{
                "from": config.default_account.as_ref().context("Missing default account")?,
                "to": to,
                "data": data_hex
            }, "latest"],
            "id": 1
        }))
        .send().await?;
    let v: serde_json::Value = resp.json().await?;
    if let Some(err) = v.get("error") { anyhow::bail!(err.to_string()); }
    Ok(v["result"].as_str().unwrap_or("").to_string())
}
