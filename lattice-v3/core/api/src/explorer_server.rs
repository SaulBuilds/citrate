// lattice-v3/core/api/src/explorer_server.rs

// DAG Explorer Server with WebSocket support
// Complete implementation for multi-network explorer

use crate::dag_explorer::{DagExplorer, NetworkConfig};
use futures_util::{StreamExt, SinkExt};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio_stream::wrappers::UnboundedReceiverStream;
use warp::{Filter, Rejection, Reply};
use warp::ws::{Message, WebSocket};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExplorerConfig {
    pub host: String,
    pub port: u16,
    pub networks: Vec<NetworkConfig>,
    pub enable_ws: bool,
    pub cors_origins: Vec<String>,
}

impl Default for ExplorerConfig {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: 3030,
            networks: vec![
                NetworkConfig {
                    name: "devnet".to_string(),
                    chain_id: 1337,
                    rpc_endpoint: "http://localhost:8545".to_string(),
                    ws_endpoint: "ws://localhost:8546".to_string(),
                    explorer_port: 3001,
                    data_dir: ".lattice-devnet".to_string(),
                    is_active: false,
                    genesis_hash: Default::default(),
                    block_time: 1,
                    ghostdag_k: 8,
                },
                NetworkConfig {
                    name: "testnet".to_string(),
                    chain_id: 42069,
                    rpc_endpoint: "http://localhost:8545".to_string(),
                    ws_endpoint: "ws://localhost:8546".to_string(),
                    explorer_port: 3002,
                    data_dir: ".lattice-testnet".to_string(),
                    is_active: true,
                    genesis_hash: Default::default(),
                    block_time: 2,
                    ghostdag_k: 18,
                },
                NetworkConfig {
                    name: "mainnet".to_string(),
                    chain_id: 1,
                    rpc_endpoint: "http://localhost:8545".to_string(),
                    ws_endpoint: "ws://localhost:8546".to_string(),
                    explorer_port: 3000,
                    data_dir: ".lattice-mainnet".to_string(),
                    is_active: false,
                    genesis_hash: Default::default(),
                    block_time: 3,
                    ghostdag_k: 50,
                },
            ],
            enable_ws: true,
            cors_origins: vec!["http://localhost:3000".to_string()],
        }
    }
}

pub struct ExplorerServer {
    config: ExplorerConfig,
    explorer: Arc<DagExplorer>,
    subscribers: Arc<RwLock<Vec<tokio::sync::mpsc::UnboundedSender<WsMessage>>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WsMessage {
    // Client -> Server
    Subscribe { network: String },
    Unsubscribe { network: String },
    SwitchNetwork { network: String },
    
    // Server -> Client
    NewBlock { network: String, block: serde_json::Value },
    NewTransaction { network: String, tx: serde_json::Value },
    StatsUpdate { network: String, stats: serde_json::Value },
    DagUpdate { network: String, dag: serde_json::Value },
    NetworkSwitched { network: String },
    Error { message: String },
}

impl ExplorerServer {
    pub async fn new(config: ExplorerConfig) -> Self {
        let explorer = Arc::new(DagExplorer::new().await);
        
        Self {
            config,
            explorer,
            subscribers: Arc::new(RwLock::new(Vec::new())),
        }
    }
    
    pub async fn start(self: Arc<Self>) -> Result<(), Box<dyn std::error::Error>> {
        let addr = format!("{}:{}", self.config.host, self.config.port)
            .parse::<std::net::SocketAddr>()?;
        
        tracing::info!("Starting DAG Explorer server on {}", addr);
        
        // Start background update task
        let update_task = self.clone();
        tokio::spawn(async move {
            update_task.background_updates().await;
        });
        
        // Build routes
        let routes = self.build_routes();
        
        // Start server
        warp::serve(routes)
            .run(addr)
            .await;
        
        Ok(())
    }
    
    fn build_routes(self: &Arc<Self>) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
        let cors = warp::cors()
            .allow_any_origin()
            .allow_headers(vec!["content-type", "authorization"])
            .allow_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"]);
        
        // Static files (if we have a web UI)
        let static_files = warp::fs::dir("./explorer/public")
            .map(|reply| warp::reply::with_header(reply, "cache-control", "public, max-age=3600"));
        
        // API routes from dag_explorer
        let api_routes = crate::dag_explorer::DagExplorer::routes(self.explorer.clone());
        
        // WebSocket endpoint
        let ws_route = warp::path("ws")
            .and(warp::ws())
            .and(warp::any().map({
                let server = self.clone();
                move || server.clone()
            }))
            .map(|ws: warp::ws::Ws, server: Arc<ExplorerServer>| {
                ws.on_upgrade(move |socket| handle_websocket(socket, server))
            });
        
        // Health check
        let health = warp::path!("health")
            .and(warp::get())
            .map(|| warp::reply::json(&serde_json::json!({
                "status": "ok",
                "service": "dag-explorer"
            })));
        
        // Combine all routes
        health
            .or(ws_route)
            .or(api_routes)
            .or(static_files)
            .with(cors)
    }
    
    async fn background_updates(&self) {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(5));
        
        loop {
            interval.tick().await;
            
            // Get current network
            let network = self.explorer.get_current_network().await;
            
            // Get latest stats
            if let Ok(stats) = self.explorer.get_stats(Some(network.clone())).await {
                self.broadcast(WsMessage::StatsUpdate {
                    network: network.clone(),
                    stats: serde_json::to_value(stats).unwrap(),
                }).await;
            }
            
            // Get latest DAG visualization (limited depth for updates)
            if let Ok(dag) = self.explorer.get_dag_visualization(20, Some(network.clone())).await {
                self.broadcast(WsMessage::DagUpdate {
                    network: network.clone(),
                    dag: serde_json::to_value(dag).unwrap(),
                }).await;
            }
            
            // Get recent blocks
            if let Ok(blocks) = self.explorer.get_recent_blocks(1, Some(network.clone())).await {
                for block in blocks {
                    self.broadcast(WsMessage::NewBlock {
                        network: network.clone(),
                        block: serde_json::to_value(block).unwrap(),
                    }).await;
                }
            }
        }
    }
    
    async fn broadcast(&self, message: WsMessage) {
        let subscribers = self.subscribers.read().await;
        for tx in subscribers.iter() {
            let _ = tx.send(message.clone());
        }
    }
    
    async fn add_subscriber(&self, tx: tokio::sync::mpsc::UnboundedSender<WsMessage>) {
        let mut subscribers = self.subscribers.write().await;
        subscribers.push(tx);
    }
    
    async fn remove_subscriber(&self, tx: &tokio::sync::mpsc::UnboundedSender<WsMessage>) {
        let mut subscribers = self.subscribers.write().await;
        subscribers.retain(|s| !std::ptr::eq(s, tx));
    }
}

async fn handle_websocket(ws: WebSocket, server: Arc<ExplorerServer>) {
    let (mut ws_tx, mut ws_rx) = ws.split();
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
    let mut rx = UnboundedReceiverStream::new(rx);
    
    // Add subscriber
    server.add_subscriber(tx.clone()).await;
    
    // Spawn task to forward messages to WebSocket
    tokio::spawn(async move {
        while let Some(msg) = rx.next().await {
            let json = serde_json::to_string(&msg).unwrap();
            if ws_tx.send(Message::text(json)).await.is_err() {
                break;
            }
        }
    });
    
    // Handle incoming messages
    while let Some(result) = ws_rx.next().await {
        match result {
            Ok(msg) => {
                if let Ok(text) = msg.to_str() {
                    if let Ok(ws_msg) = serde_json::from_str::<WsMessage>(text) {
                        handle_ws_message(ws_msg, &server, &tx).await;
                    }
                }
            }
            Err(_) => break,
        }
    }
    
    // Remove subscriber on disconnect
    server.remove_subscriber(&tx).await;
}

async fn handle_ws_message(
    msg: WsMessage,
    server: &Arc<ExplorerServer>,
    tx: &tokio::sync::mpsc::UnboundedSender<WsMessage>,
) {
    match msg {
        WsMessage::Subscribe { network } => {
            tracing::info!("Client subscribed to network: {}", network);
            // Send initial data
            if let Ok(stats) = server.explorer.get_stats(Some(network.clone())).await {
                let _ = tx.send(WsMessage::StatsUpdate {
                    network: network.clone(),
                    stats: serde_json::to_value(stats).unwrap(),
                });
            }
        }
        
        WsMessage::Unsubscribe { network } => {
            tracing::info!("Client unsubscribed from network: {}", network);
        }
        
        WsMessage::SwitchNetwork { network } => {
            if let Ok(_) = server.explorer.switch_network(network.clone()).await {
                let _ = tx.send(WsMessage::NetworkSwitched { network });
            } else {
                let _ = tx.send(WsMessage::Error {
                    message: format!("Failed to switch to network: {}", network),
                });
            }
        }
        
        _ => {
            // Ignore server->client messages
        }
    }
}

// CLI command to start explorer
pub async fn run_explorer_server(config: Option<ExplorerConfig>) -> Result<(), Box<dyn std::error::Error>> {
    let config = config.unwrap_or_default();
    let server = Arc::new(ExplorerServer::new(config).await);
    server.start().await
}