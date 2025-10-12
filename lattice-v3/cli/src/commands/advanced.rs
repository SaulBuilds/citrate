//lattice-v3/cli/src/commands/advanced.rs

use anyhow::{Context, Result};
use chrono::Utc;
use clap::Subcommand;
use colored::Colorize;
use rand::Rng;
use serde_json::json;
use std::collections::HashMap;
use std::fs;
use std::time::Duration;
use tokio::time::{interval, sleep};

use crate::config::Config;

/// Generate a valid test transaction for benchmarking
fn generate_test_transaction(nonce: u64, to: Option<&str>, value: u64) -> String {
    // Create a simple transaction JSON that can be sent via eth_sendTransaction
    // This will be signed by the node if a default account is configured
    let tx = json!({
        "nonce": format!("0x{:x}", nonce),
        "to": to.unwrap_or("0x0000000000000000000000000000000000000000"),
        "value": format!("0x{:x}", value),
        "gas": "0x5208", // 21000 gas (minimum for transfer)
        "gasPrice": "0x3b9aca00", // 1 gwei
        "data": "0x", // Empty data
        "chainId": "0x539" // Local testnet chain ID (1337)
    });

    serde_json::to_string(&tx).unwrap_or_default()
}

/// Generate random test transaction data for stress testing
fn generate_random_transaction() -> String {
    let mut rng = rand::thread_rng();
    let nonce = rng.gen::<u32>() as u64;
    let value = rng.gen_range(1..1000); // Random value between 1-1000 wei

    // Generate random recipient address
    let mut addr_bytes = [0u8; 20];
    rng.fill(&mut addr_bytes);
    let to_addr = format!("0x{}", hex::encode(addr_bytes));

    generate_test_transaction(nonce, Some(&to_addr), value)
}

#[derive(Subcommand)]
pub enum AdvancedCommands {
    /// Monitor network health and status
    Monitor {
        /// Update interval in seconds
        #[arg(short, long, default_value = "5")]
        interval: u64,

        /// Number of updates to display (0 = infinite)
        #[arg(short, long, default_value = "0")]
        count: usize,

        /// Include DAG metrics
        #[arg(long)]
        dag: bool,

        /// Include transaction pool status
        #[arg(long)]
        mempool: bool,
    },

    /// Benchmark network performance
    Benchmark {
        /// Number of transactions to send
        #[arg(short, long, default_value = "100")]
        txs: usize,

        /// Concurrency level
        #[arg(short, long, default_value = "10")]
        concurrency: usize,

        /// Transaction size in bytes
        #[arg(long, default_value = "256")]
        size: usize,
    },

    /// Analyze network topology
    Topology {
        /// Show peer connections
        #[arg(long)]
        peers: bool,

        /// Export graph format (dot, json)
        #[arg(long)]
        export: Option<String>,
    },

    /// Run network stress test
    StressTest {
        /// Duration in seconds
        #[arg(short, long, default_value = "60")]
        duration: u64,

        /// Transactions per second target
        #[arg(short, long, default_value = "100")]
        tps: u64,

        /// Number of parallel workers
        #[arg(short, long, default_value = "4")]
        workers: usize,
    },

    /// Debug transaction pipeline
    TxDebug {
        /// Transaction hash to trace
        tx_hash: String,

        /// Show detailed execution trace
        #[arg(long)]
        trace: bool,
    },

    /// Model analytics and insights
    ModelStats {
        /// Model ID to analyze
        model_id: Option<String>,

        /// Time range (24h, 7d, 30d)
        #[arg(short, long, default_value = "24h")]
        range: String,

        /// Export to CSV
        #[arg(long)]
        csv: Option<String>,
    },
}

pub async fn execute(cmd: AdvancedCommands, config: &Config) -> Result<()> {
    match cmd {
        AdvancedCommands::Monitor { interval, count, dag, mempool } => {
            monitor_network(config, interval, count, dag, mempool).await?;
        }
        AdvancedCommands::Benchmark { txs, concurrency, size } => {
            benchmark_network(config, txs, concurrency, size).await?;
        }
        AdvancedCommands::Topology { peers, export } => {
            analyze_topology(config, peers, export).await?;
        }
        AdvancedCommands::StressTest { duration, tps, workers } => {
            stress_test(config, duration, tps, workers).await?;
        }
        AdvancedCommands::TxDebug { tx_hash, trace } => {
            debug_transaction(config, &tx_hash, trace).await?;
        }
        AdvancedCommands::ModelStats { model_id, range, csv } => {
            model_analytics(config, model_id, &range, csv).await?;
        }
    }
    Ok(())
}

async fn monitor_network(
    config: &Config,
    interval_secs: u64,
    count: usize,
    show_dag: bool,
    show_mempool: bool,
) -> Result<()> {
    println!("{}", "🔍 Network Monitor".cyan().bold());
    println!("Press Ctrl+C to stop");
    println!();

    let mut ticker = interval(Duration::from_secs(interval_secs));
    let mut iterations = 0;

    loop {
        if count > 0 && iterations >= count {
            break;
        }

        let client = reqwest::Client::new();

        // Get basic network stats
        let mut stats = HashMap::new();

        // Chain height
        if let Ok(response) = client
            .post(&config.rpc_endpoint)
            .json(&json!({
                "jsonrpc": "2.0",
                "method": "eth_blockNumber",
                "params": [],
                "id": 1
            }))
            .send()
            .await
        {
            if let Ok(result) = response.json::<serde_json::Value>().await {
                if let Some(height) = result["result"].as_str() {
                    stats.insert("height", height.to_string());
                }
            }
        }

        // Peer count
        if let Ok(response) = client
            .post(&config.rpc_endpoint)
            .json(&json!({
                "jsonrpc": "2.0",
                "method": "net_peerCount",
                "params": [],
                "id": 2
            }))
            .send()
            .await
        {
            if let Ok(result) = response.json::<serde_json::Value>().await {
                if let Some(peers) = result["result"].as_str() {
                    stats.insert("peers", peers.to_string());
                }
            }
        }

        // Gas price
        if let Ok(response) = client
            .post(&config.rpc_endpoint)
            .json(&json!({
                "jsonrpc": "2.0",
                "method": "eth_gasPrice",
                "params": [],
                "id": 3
            }))
            .send()
            .await
        {
            if let Ok(result) = response.json::<serde_json::Value>().await {
                if let Some(gas_price) = result["result"].as_str() {
                    stats.insert("gas_price", gas_price.to_string());
                }
            }
        }

        // Display stats
        print!("\x1B[2J\x1B[1;1H"); // Clear screen
        println!("{}", "🔍 Lattice Network Monitor".cyan().bold());
        println!("Time: {}", chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC"));
        println!();

        println!("{}", "Basic Metrics:".bold());
        if let Some(height) = stats.get("height") {
            println!("  Block Height: {}", height.cyan());
        }
        if let Some(peers) = stats.get("peers") {
            println!("  Connected Peers: {}", peers.cyan());
        }
        if let Some(gas_price) = stats.get("gas_price") {
            println!("  Gas Price: {} wei", gas_price.cyan());
        }

        if show_dag {
            println!();
            println!("{}", "DAG Metrics:".bold());

            // Get DAG-specific metrics
            if let Ok(response) = client
                .post(&config.rpc_endpoint)
                .json(&json!({
                    "jsonrpc": "2.0",
                    "method": "lattice_getDAGInfo",
                    "params": [],
                    "id": 4
                }))
                .send()
                .await
            {
                if let Ok(result) = response.json::<serde_json::Value>().await {
                    if let Some(dag_info) = result["result"].as_object() {
                        println!("  Blue Score: {}",
                            dag_info["blue_score"].as_str().unwrap_or("0").cyan());
                        println!("  DAG Width: {}",
                            dag_info["dag_width"].as_u64().unwrap_or(1).to_string().cyan());
                    } else {
                        println!("  Blue Score: {}", "N/A".yellow());
                        println!("  DAG Width: {}", "N/A".yellow());
                    }
                }
            } else {
                println!("  Blue Score: {}", "N/A".yellow());
                println!("  DAG Width: {}", "N/A".yellow());
            }
        }

        if show_mempool {
            println!();
            println!("{}", "Mempool Status:".bold());

            // Get mempool metrics
            if let Ok(response) = client
                .post(&config.rpc_endpoint)
                .json(&json!({
                    "jsonrpc": "2.0",
                    "method": "lattice_getMempoolInfo",
                    "params": [],
                    "id": 5
                }))
                .send()
                .await
            {
                if let Ok(result) = response.json::<serde_json::Value>().await {
                    if let Some(mempool_info) = result["result"].as_object() {
                        println!("  Pending Txs: {}",
                            mempool_info["pending_count"].as_u64().unwrap_or(0).to_string().cyan());
                        println!("  Queue Size: {}",
                            mempool_info["queue_size"].as_u64().unwrap_or(0).to_string().cyan());
                    } else {
                        println!("  Pending Txs: {}", "N/A".yellow());
                        println!("  Queue Size: {}", "N/A".yellow());
                    }
                }
            } else {
                println!("  Pending Txs: {}", "N/A".yellow());
                println!("  Queue Size: {}", "N/A".yellow());
            }
        }

        iterations += 1;
        ticker.tick().await;
    }

    Ok(())
}

async fn benchmark_network(
    config: &Config,
    tx_count: usize,
    concurrency: usize,
    _tx_size: usize, // Size parameter kept for API compatibility but not used
) -> Result<()> {
    println!("{}", "⚡ Network Benchmark".cyan().bold());
    println!("Transactions: {}", tx_count);
    println!("Concurrency: {}", concurrency);
    println!("Note: Using standard transaction format for testing");
    println!();

    let start_time = std::time::Instant::now();
    let client = reqwest::Client::new();
    let mut handles = Vec::new();

    println!("🚀 Starting benchmark...");

    // Send transactions in batches
    for batch in 0..(tx_count / concurrency) {
        for tx_index in 0..concurrency {
            let client = client.clone();
            let endpoint = config.rpc_endpoint.clone();
            let nonce = (batch * concurrency + tx_index) as u64;

            let handle = tokio::spawn(async move {
                // Generate a unique test transaction for each request
                let tx_data = generate_test_transaction(nonce, None, 1);

                client
                    .post(&endpoint)
                    .json(&json!({
                        "jsonrpc": "2.0",
                        "method": "eth_sendTransaction",
                        "params": [serde_json::from_str::<serde_json::Value>(&tx_data).unwrap_or_default()],
                        "id": nonce
                    }))
                    .send()
                    .await
            });

            handles.push(handle);
        }

        // Progress indicator
        let progress = ((batch + 1) * concurrency * 100) / tx_count;
        print!("\rProgress: {}% ", progress);
        use std::io::{self, Write};
        io::stdout().flush().unwrap();

        // Small delay between batches to avoid overwhelming the node
        tokio::time::sleep(Duration::from_millis(10)).await;
    }

    // Wait for all transactions
    let mut successful = 0;
    let mut failed = 0;

    for handle in handles {
        match handle.await {
            Ok(Ok(_)) => successful += 1,
            _ => failed += 1,
        }
    }

    let duration = start_time.elapsed();
    let tps = tx_count as f64 / duration.as_secs_f64();

    println!();
    println!("{}", "Benchmark Results:".bold());
    println!("  Total time: {:.2}s", duration.as_secs_f64());
    println!("  Successful: {}", successful.to_string().green());
    println!("  Failed: {}", failed.to_string().red());
    println!("  TPS: {:.2}", tps.to_string().cyan());

    Ok(())
}

async fn analyze_topology(
    config: &Config,
    show_peers: bool,
    export_format: Option<String>,
) -> Result<()> {
    println!("{}", "🌐 Network Topology Analysis".cyan().bold());

    let client = reqwest::Client::new();
    let mut peers_data = Vec::new();
    let mut connections = Vec::new();

    // Get peer information
    if let Ok(response) = client
        .post(&config.rpc_endpoint)
        .json(&json!({
            "jsonrpc": "2.0",
            "method": "lattice_getPeers",
            "params": [],
            "id": 1
        }))
        .send()
        .await
    {
        if let Ok(result) = response.json::<serde_json::Value>().await {
            if let Some(peers) = result["result"].as_array() {
                peers_data = peers.clone();

                if show_peers {
                    println!("Connected Peers: {}", peers.len());
                    for (i, peer) in peers.iter().enumerate() {
                        println!("  Peer {}: {}", i + 1, peer["address"].as_str().unwrap_or("Unknown"));
                    }
                }

                // Collect connection information
                for peer in peers {
                    if let Some(peer_connections) = peer["connections"].as_array() {
                        connections.extend(peer_connections.iter().cloned());
                    }
                }
            }
        }
    } else {
        println!("{}", "⚠️  Could not fetch peer information".yellow());
    }

    // Get additional network information
    let network_info = if let Ok(response) = client
        .post(&config.rpc_endpoint)
        .json(&json!({
            "jsonrpc": "2.0",
            "method": "lattice_getNetworkInfo",
            "params": [],
            "id": 2
        }))
        .send()
        .await
    {
        response.json::<serde_json::Value>().await.ok()
            .and_then(|r| r["result"].clone().into())
    } else {
        None
    };

    if let Some(format) = export_format {
        match format.as_str() {
            "json" => {
                println!("Exporting topology to JSON...");
                let topology_data = json!({
                    "timestamp": chrono::Utc::now().to_rfc3339(),
                    "peers": peers_data,
                    "connections": connections,
                    "network_info": network_info,
                    "total_peers": peers_data.len(),
                    "total_connections": connections.len()
                });

                let filename = format!("topology_{}.json", chrono::Utc::now().format("%Y%m%d_%H%M%S"));
                fs::write(&filename, serde_json::to_string_pretty(&topology_data)?)?;
                println!("✅ Topology exported to {}", filename.green());
            }
            "dot" => {
                println!("Exporting topology to DOT format...");
                let mut dot_content = String::from("digraph network_topology {\n");
                dot_content.push_str("  rankdir=LR;\n");
                dot_content.push_str("  node [shape=circle, style=filled, fillcolor=lightblue];\n\n");

                // Add nodes (peers)
                for (i, peer) in peers_data.iter().enumerate() {
                    let default_id = format!("peer_{}", i);
                    let peer_id = peer["id"].as_str().unwrap_or(&default_id);
                    let peer_addr = peer["address"].as_str().unwrap_or("unknown");
                    dot_content.push_str(&format!("  \"{}\" [label=\"{}\\n{}\"];\n",
                        peer_id, peer_id, peer_addr));
                }

                dot_content.push_str("\n");

                // Add edges (connections)
                for connection in &connections {
                    if let (Some(from), Some(to)) = (
                        connection["from"].as_str(),
                        connection["to"].as_str()
                    ) {
                        dot_content.push_str(&format!("  \"{}\" -> \"{}\";\n", from, to));
                    }
                }

                dot_content.push_str("}\n");

                let filename = format!("topology_{}.dot", chrono::Utc::now().format("%Y%m%d_%H%M%S"));
                fs::write(&filename, dot_content)?;
                println!("✅ Topology exported to {}", filename.green());
                println!("💡 Use 'dot -Tpng {} -o topology.png' to generate visualization", filename);
            }
            _ => {
                anyhow::bail!("Unsupported export format: {}", format);
            }
        }
    }

    Ok(())
}

async fn stress_test(
    config: &Config,
    duration: u64,
    target_tps: u64,
    workers: usize,
) -> Result<()> {
    println!("{}", "💥 Network Stress Test".cyan().bold());
    println!("Duration: {}s", duration);
    println!("Target TPS: {}", target_tps);
    println!("Workers: {}", workers);
    println!();

    let start_time = std::time::Instant::now();
    let end_time = start_time + Duration::from_secs(duration);

    let mut handles = Vec::new();
    let client = reqwest::Client::new();

    // Start worker tasks
    for worker_id in 0..workers {
        let client = client.clone();
        let endpoint = config.rpc_endpoint.clone();
        let worker_tps = target_tps / workers as u64;

        let handle = tokio::spawn(async move {
            let mut tx_count = 0;
            let mut interval = interval(Duration::from_millis(1000 / worker_tps));

            while std::time::Instant::now() < end_time {
                interval.tick().await;

                // Generate a valid test transaction
                let tx_data = generate_random_transaction();

                // Send the transaction
                let _result = client
                    .post(&endpoint)
                    .json(&json!({
                        "jsonrpc": "2.0",
                        "method": "eth_sendTransaction",
                        "params": [serde_json::from_str::<serde_json::Value>(&tx_data).unwrap_or_default()],
                        "id": format!("worker-{}-tx-{}", worker_id, tx_count)
                    }))
                    .send()
                    .await;

                tx_count += 1;
            }

            tx_count
        });

        handles.push(handle);
    }

    // Monitor progress
    while std::time::Instant::now() < end_time {
        let elapsed = start_time.elapsed().as_secs();
        let remaining = duration - elapsed;
        print!("\rTime remaining: {}s ", remaining);
        use std::io::{self, Write};
        io::stdout().flush().unwrap();
        sleep(Duration::from_secs(1)).await;
    }

    // Collect results
    let mut total_txs = 0;
    for handle in handles {
        if let Ok(count) = handle.await {
            total_txs += count;
        }
    }

    let actual_duration = start_time.elapsed().as_secs_f64();
    let actual_tps = total_txs as f64 / actual_duration;

    println!();
    println!("{}", "Stress Test Results:".bold());
    println!("  Duration: {:.2}s", actual_duration);
    println!("  Total transactions: {}", total_txs);
    println!("  Actual TPS: {:.2}", actual_tps.to_string().cyan());
    println!("  Target TPS: {}", target_tps);

    let efficiency = (actual_tps / target_tps as f64) * 100.0;
    println!("  Efficiency: {:.1}%", efficiency.to_string().cyan());

    Ok(())
}

async fn debug_transaction(
    config: &Config,
    tx_hash: &str,
    show_trace: bool,
) -> Result<()> {
    println!("{}", "🔧 Transaction Debug".cyan().bold());
    println!("Hash: {}", tx_hash.cyan());
    println!();

    let client = reqwest::Client::new();

    // Get transaction details
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
        println!("  From: {}", tx["from"].as_str().unwrap_or("N/A"));
        println!("  To: {}", tx["to"].as_str().unwrap_or("N/A"));
        println!("  Value: {} wei", tx["value"].as_str().unwrap_or("0"));
        println!("  Gas: {}", tx["gas"].as_str().unwrap_or("N/A"));
        println!("  Gas Price: {} wei", tx["gasPrice"].as_str().unwrap_or("N/A"));
        println!("  Nonce: {}", tx["nonce"].as_str().unwrap_or("N/A"));

        if show_trace {
            println!();
            println!("{}", "Execution Trace:".bold());

            // Get transaction receipt for execution details
            let receipt_response = client
                .post(&config.rpc_endpoint)
                .json(&json!({
                    "jsonrpc": "2.0",
                    "method": "eth_getTransactionReceipt",
                    "params": [tx_hash],
                    "id": 2
                }))
                .send()
                .await;

            if let Ok(receipt_response) = receipt_response {
                if let Ok(receipt_result) = receipt_response.json::<serde_json::Value>().await {
                    if let Some(receipt) = receipt_result["result"].as_object() {
                        println!("  Status: {}",
                            if receipt["status"].as_str().unwrap_or("0x0") == "0x1" {
                                "Success".green()
                            } else {
                                "Failed".red()
                            });
                        println!("  Gas Used: {}", receipt["gasUsed"].as_str().unwrap_or("N/A"));
                        println!("  Block Number: {}", receipt["blockNumber"].as_str().unwrap_or("N/A"));

                        // Show logs if any
                        if let Some(logs) = receipt["logs"].as_array() {
                            if !logs.is_empty() {
                                println!("  Logs:");
                                for (i, log) in logs.iter().take(5).enumerate() {
                                    println!("    Log {}: {}", i + 1,
                                        log["topics"].as_array()
                                            .and_then(|topics| topics.get(0))
                                            .and_then(|topic| topic.as_str())
                                            .unwrap_or("Unknown topic"));
                                }
                                if logs.len() > 5 {
                                    println!("    ... and {} more logs", logs.len() - 5);
                                }
                            }
                        }

                        // Try to get detailed trace if supported
                        let trace_response = client
                            .post(&config.rpc_endpoint)
                            .json(&json!({
                                "jsonrpc": "2.0",
                                "method": "debug_traceTransaction",
                                "params": [tx_hash, {"tracer": "callTracer"}],
                                "id": 3
                            }))
                            .send()
                            .await;

                        if let Ok(trace_response) = trace_response {
                            if let Ok(trace_result) = trace_response.json::<serde_json::Value>().await {
                                if let Some(trace) = trace_result["result"].as_object() {
                                    println!("  Call Type: {}", trace["type"].as_str().unwrap_or("UNKNOWN"));
                                    if let Some(calls) = trace["calls"].as_array() {
                                        println!("  Subcalls: {}", calls.len());
                                    }
                                }
                            }
                        } else {
                            println!("  {} Detailed tracing not available on this node", "ℹ️".blue());
                        }
                    }
                }
            } else {
                println!("  {} Could not fetch transaction receipt", "⚠️".yellow());
            }
        }
    } else {
        println!("{}", "Transaction not found".red());
    }

    Ok(())
}

async fn model_analytics(
    config: &Config,
    model_id: Option<String>,
    range: &str,
    csv_output: Option<String>,
) -> Result<()> {
    println!("{}", "📊 Model Analytics".cyan().bold());

    if let Some(ref id) = model_id {
        println!("Model ID: {}", id.cyan());
    } else {
        println!("Analyzing all models");
    }

    println!("Time range: {}", range);
    println!();

    let client = reqwest::Client::new();

    // Get model statistics
    let response = client
        .post(&config.rpc_endpoint)
        .json(&json!({
            "jsonrpc": "2.0",
            "method": "lattice_getModelStats",
            "params": {
                "model_id": model_id,
                "range": range
            },
            "id": 1
        }))
        .send()
        .await
        .context("Failed to connect to RPC endpoint")?;

    let result: serde_json::Value = response.json().await?;

    if let Some(stats) = result["result"].as_object() {
        println!("{}", "Model Statistics:".bold());
        println!("  Total Inferences: {}", stats["total_inferences"].as_u64().unwrap_or(0));
        println!("  Average Execution Time: {}ms", stats["avg_execution_time"].as_f64().unwrap_or(0.0));
        println!("  Total Revenue: {} wei", stats["total_revenue"].as_str().unwrap_or("0"));
        println!("  Success Rate: {}%", stats["success_rate"].as_f64().unwrap_or(0.0));

        if let Some(csv_path) = csv_output {
            println!();
            println!("Exporting to CSV: {}", csv_path);

            // Create CSV content with headers
            let mut csv_content = String::from("timestamp,model_id,total_inferences,avg_execution_time_ms,total_revenue_wei,success_rate_percent\n");

            // Add the data row
            let timestamp = Utc::now().to_rfc3339();
            let model_id_str = model_id.as_deref().unwrap_or("all_models");
            let total_inferences = stats["total_inferences"].as_u64().unwrap_or(0);
            let avg_execution_time = stats["avg_execution_time"].as_f64().unwrap_or(0.0);
            let total_revenue = stats["total_revenue"].as_str().unwrap_or("0");
            let success_rate = stats["success_rate"].as_f64().unwrap_or(0.0);

            csv_content.push_str(&format!(
                "{},{},{},{},{},{}\n",
                timestamp, model_id_str, total_inferences, avg_execution_time, total_revenue, success_rate
            ));

            // Try to get detailed inference data if available
            if let Some(inference_history) = stats["inference_history"].as_array() {
                csv_content.push_str("\n# Detailed inference history\n");
                csv_content.push_str("inference_timestamp,model_id,execution_time_ms,gas_used,revenue_wei,status\n");

                for inference in inference_history {
                    let inf_timestamp = inference["timestamp"].as_str().unwrap_or("");
                    let inf_model_id = inference["model_id"].as_str().unwrap_or(model_id_str);
                    let inf_exec_time = inference["execution_time"].as_f64().unwrap_or(0.0);
                    let inf_gas_used = inference["gas_used"].as_u64().unwrap_or(0);
                    let inf_revenue = inference["revenue"].as_str().unwrap_or("0");
                    let inf_status = inference["status"].as_str().unwrap_or("unknown");

                    csv_content.push_str(&format!(
                        "{},{},{},{},{},{}\n",
                        inf_timestamp, inf_model_id, inf_exec_time, inf_gas_used, inf_revenue, inf_status
                    ));
                }
            }

            fs::write(&csv_path, csv_content)
                .with_context(|| format!("Failed to write CSV file: {}", csv_path))?;

            println!("✅ Analytics exported to {}", csv_path.green());
        }
    } else {
        println!("{}", "No statistics available".yellow());
    }

    Ok(())
}