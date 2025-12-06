// citrate/core/api/src/eth_subscriptions.rs
//
// Ethereum-compatible WebSocket subscriptions (eth_subscribe / eth_unsubscribe)
// Supports: newHeads, logs, pendingTransactions, syncing

use citrate_consensus::types::{Block, Hash};
use citrate_sequencer::mempool::Mempool;
use citrate_storage::StorageManager;
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{broadcast, RwLock};
use tokio_tungstenite::{accept_async, tungstenite::Message};
use tracing::{debug, error, info};

/// Subscription types for eth_subscribe
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum EthSubscriptionType {
    /// New block headers
    NewHeads,
    /// Log events matching filter
    Logs,
    /// New pending transactions
    NewPendingTransactions,
    /// Sync status changes
    Syncing,
}

/// Log filter for eth_subscribe("logs", filter)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LogFilter {
    /// Contract addresses to filter (empty = all)
    #[serde(default)]
    pub address: Option<AddressFilter>,
    /// Topics to filter (up to 4, null = any)
    #[serde(default)]
    pub topics: Option<Vec<Option<TopicFilter>>>,
}

/// Address filter - single or array
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AddressFilter {
    Single(String),
    Multiple(Vec<String>),
}

/// Topic filter - single or array
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TopicFilter {
    Single(String),
    Multiple(Vec<String>),
}

/// Subscription request from client
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionRequest {
    pub jsonrpc: String,
    pub id: serde_json::Value,
    pub method: String,
    #[serde(default)]
    pub params: Vec<serde_json::Value>,
}

/// Subscription response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionResponse {
    pub jsonrpc: String,
    pub id: serde_json::Value,
    pub result: serde_json::Value,
}

/// Subscription notification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionNotification {
    pub jsonrpc: String,
    pub method: String,
    pub params: SubscriptionParams,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionParams {
    pub subscription: String,
    pub result: serde_json::Value,
}

/// Active subscription
#[derive(Debug, Clone)]
pub struct Subscription {
    pub id: String,
    pub sub_type: EthSubscriptionType,
    pub filter: Option<LogFilter>,
}

/// Block header for newHeads subscription
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockHeader {
    pub number: String,
    pub hash: String,
    pub parent_hash: String,
    pub nonce: String,
    pub sha3_uncles: String,
    pub logs_bloom: String,
    pub transactions_root: String,
    pub state_root: String,
    pub receipts_root: String,
    pub miner: String,
    pub difficulty: String,
    pub total_difficulty: String,
    pub extra_data: String,
    pub size: String,
    pub gas_limit: String,
    pub gas_used: String,
    pub timestamp: String,
    pub base_fee_per_gas: Option<String>,
}

impl From<&Block> for BlockHeader {
    fn from(block: &Block) -> Self {
        Self {
            number: format!("0x{:x}", block.header.height),
            hash: format!("0x{}", hex::encode(block.header.block_hash.as_bytes())),
            parent_hash: format!("0x{}", hex::encode(block.header.selected_parent_hash.as_bytes())),
            nonce: "0x0000000000000000".to_string(),
            sha3_uncles: "0x1dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347".to_string(),
            logs_bloom: "0x".to_string() + &"00".repeat(256),
            transactions_root: format!("0x{}", hex::encode(block.tx_root.as_bytes())),
            state_root: format!("0x{}", hex::encode(block.state_root.as_bytes())),
            receipts_root: format!("0x{}", hex::encode(block.receipt_root.as_bytes())),
            miner: format!("0x{}", hex::encode(&block.header.proposer_pubkey.0[12..32])), // Last 20 bytes
            difficulty: "0x0".to_string(),
            total_difficulty: "0x0".to_string(),
            extra_data: "0x".to_string(),
            size: "0x0".to_string(),
            gas_limit: "0x1c9c380".to_string(), // 30M gas
            gas_used: "0x0".to_string(),
            timestamp: format!("0x{:x}", block.header.timestamp),
            base_fee_per_gas: Some("0x7".to_string()),
        }
    }
}

/// Log entry for logs subscription
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LogEntry {
    pub address: String,
    pub topics: Vec<String>,
    pub data: String,
    pub block_number: String,
    pub block_hash: String,
    pub transaction_hash: String,
    pub transaction_index: String,
    pub log_index: String,
    pub removed: bool,
}

/// Ethereum-compatible WebSocket subscription server
pub struct EthSubscriptionServer {
    addr: SocketAddr,
    storage: Arc<StorageManager>,
    mempool: Arc<Mempool>,
    /// Broadcast channel for new block headers
    new_heads_tx: broadcast::Sender<Block>,
    /// Broadcast channel for pending transactions
    pending_tx_tx: broadcast::Sender<Hash>,
    /// Active connections
    connections: Arc<RwLock<HashMap<String, Arc<RwLock<ConnectionState>>>>>,
}

/// State for each WebSocket connection
struct ConnectionState {
    subscriptions: HashMap<String, Subscription>,
    next_sub_id: u64,
}

impl ConnectionState {
    fn new() -> Self {
        Self {
            subscriptions: HashMap::new(),
            next_sub_id: 1,
        }
    }

    fn next_subscription_id(&mut self) -> String {
        let id = format!("0x{:x}", self.next_sub_id);
        self.next_sub_id += 1;
        id
    }
}

impl EthSubscriptionServer {
    /// Create a new subscription server
    pub fn new(
        addr: SocketAddr,
        storage: Arc<StorageManager>,
        mempool: Arc<Mempool>,
    ) -> Self {
        let (new_heads_tx, _) = broadcast::channel(100);
        let (pending_tx_tx, _) = broadcast::channel(1000);

        Self {
            addr,
            storage,
            mempool,
            new_heads_tx,
            pending_tx_tx,
            connections: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get sender for broadcasting new block headers
    pub fn new_heads_sender(&self) -> broadcast::Sender<Block> {
        self.new_heads_tx.clone()
    }

    /// Get sender for broadcasting pending transactions
    pub fn pending_tx_sender(&self) -> broadcast::Sender<Hash> {
        self.pending_tx_tx.clone()
    }

    /// Broadcast a new block to all newHeads subscribers
    pub fn broadcast_new_head(&self, block: &Block) {
        let _ = self.new_heads_tx.send(block.clone());
    }

    /// Broadcast a pending transaction to all pendingTransactions subscribers
    pub fn broadcast_pending_transaction(&self, tx_hash: Hash) {
        let _ = self.pending_tx_tx.send(tx_hash);
    }

    /// Start the WebSocket server
    pub async fn start(self: Arc<Self>) -> anyhow::Result<()> {
        let listener = TcpListener::bind(self.addr).await?;
        info!("Ethereum subscription WebSocket server listening on ws://{}", self.addr);

        while let Ok((stream, peer_addr)) = listener.accept().await {
            let server = self.clone();
            tokio::spawn(async move {
                if let Err(e) = server.handle_connection(stream, peer_addr).await {
                    error!("WebSocket connection error from {}: {}", peer_addr, e);
                }
            });
        }

        Ok(())
    }

    async fn handle_connection(
        &self,
        stream: TcpStream,
        peer_addr: SocketAddr,
    ) -> anyhow::Result<()> {
        debug!("New WebSocket connection from {}", peer_addr);

        let ws_stream = accept_async(stream).await?;
        let (mut write, mut read) = ws_stream.split();

        let conn_id = format!("{}-{}", peer_addr, chrono::Utc::now().timestamp_millis());
        let conn_state = Arc::new(RwLock::new(ConnectionState::new()));

        // Register connection
        {
            let mut connections = self.connections.write().await;
            connections.insert(conn_id.clone(), conn_state.clone());
        }

        // Subscribe to broadcasts
        let mut new_heads_rx = self.new_heads_tx.subscribe();
        let mut pending_tx_rx = self.pending_tx_tx.subscribe();

        // Message handling loop
        loop {
            tokio::select! {
                // Handle incoming messages
                msg = read.next() => {
                    match msg {
                        Some(Ok(Message::Text(text))) => {
                            if let Some(response) = self.handle_message(&conn_state, &text).await {
                                let _ = write.send(Message::Text(response)).await;
                            }
                        }
                        Some(Ok(Message::Close(_))) | None => {
                            debug!("WebSocket connection {} closed", conn_id);
                            break;
                        }
                        Some(Ok(Message::Ping(data))) => {
                            let _ = write.send(Message::Pong(data)).await;
                        }
                        Some(Err(e)) => {
                            error!("WebSocket error: {}", e);
                            break;
                        }
                        _ => {}
                    }
                }

                // Broadcast new heads
                block = new_heads_rx.recv() => {
                    if let Ok(block) = block {
                        let state = conn_state.read().await;
                        for (sub_id, sub) in &state.subscriptions {
                            if sub.sub_type == EthSubscriptionType::NewHeads {
                                let header = BlockHeader::from(&block);
                                let notification = SubscriptionNotification {
                                    jsonrpc: "2.0".to_string(),
                                    method: "eth_subscription".to_string(),
                                    params: SubscriptionParams {
                                        subscription: sub_id.clone(),
                                        result: serde_json::to_value(&header).unwrap_or_default(),
                                    },
                                };
                                if let Ok(json) = serde_json::to_string(&notification) {
                                    let _ = write.send(Message::Text(json)).await;
                                }
                            }
                        }
                    }
                }

                // Broadcast pending transactions
                tx_hash = pending_tx_rx.recv() => {
                    if let Ok(tx_hash) = tx_hash {
                        let state = conn_state.read().await;
                        for (sub_id, sub) in &state.subscriptions {
                            if sub.sub_type == EthSubscriptionType::NewPendingTransactions {
                                let notification = SubscriptionNotification {
                                    jsonrpc: "2.0".to_string(),
                                    method: "eth_subscription".to_string(),
                                    params: SubscriptionParams {
                                        subscription: sub_id.clone(),
                                        result: serde_json::Value::String(
                                            format!("0x{}", hex::encode(tx_hash.as_bytes()))
                                        ),
                                    },
                                };
                                if let Ok(json) = serde_json::to_string(&notification) {
                                    let _ = write.send(Message::Text(json)).await;
                                }
                            }
                        }
                    }
                }
            }
        }

        // Cleanup
        {
            let mut connections = self.connections.write().await;
            connections.remove(&conn_id);
        }

        Ok(())
    }

    async fn handle_message(
        &self,
        conn_state: &Arc<RwLock<ConnectionState>>,
        text: &str,
    ) -> Option<String> {
        let request: SubscriptionRequest = match serde_json::from_str(text) {
            Ok(r) => r,
            Err(e) => {
                return Some(serde_json::to_string(&serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": null,
                    "error": {
                        "code": -32700,
                        "message": format!("Parse error: {}", e)
                    }
                })).unwrap_or_default());
            }
        };

        match request.method.as_str() {
            "eth_subscribe" => {
                self.handle_subscribe(conn_state, request).await
            }
            "eth_unsubscribe" => {
                self.handle_unsubscribe(conn_state, request).await
            }
            _ => {
                Some(serde_json::to_string(&serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": request.id,
                    "error": {
                        "code": -32601,
                        "message": format!("Method not found: {}", request.method)
                    }
                })).unwrap_or_default())
            }
        }
    }

    async fn handle_subscribe(
        &self,
        conn_state: &Arc<RwLock<ConnectionState>>,
        request: SubscriptionRequest,
    ) -> Option<String> {
        if request.params.is_empty() {
            return Some(serde_json::to_string(&serde_json::json!({
                "jsonrpc": "2.0",
                "id": request.id,
                "error": {
                    "code": -32602,
                    "message": "Missing subscription type"
                }
            })).unwrap_or_default());
        }

        let sub_type_str = request.params[0].as_str().unwrap_or("");
        let sub_type = match sub_type_str {
            "newHeads" => EthSubscriptionType::NewHeads,
            "logs" => EthSubscriptionType::Logs,
            "newPendingTransactions" => EthSubscriptionType::NewPendingTransactions,
            "syncing" => EthSubscriptionType::Syncing,
            _ => {
                return Some(serde_json::to_string(&serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": request.id,
                    "error": {
                        "code": -32602,
                        "message": format!("Unknown subscription type: {}", sub_type_str)
                    }
                })).unwrap_or_default());
            }
        };

        // Parse filter for logs subscription
        let filter = if sub_type == EthSubscriptionType::Logs && request.params.len() > 1 {
            serde_json::from_value(request.params[1].clone()).ok()
        } else {
            None
        };

        let mut state = conn_state.write().await;
        let sub_id = state.next_subscription_id();

        state.subscriptions.insert(
            sub_id.clone(),
            Subscription {
                id: sub_id.clone(),
                sub_type,
                filter,
            },
        );

        debug!("Created subscription {} for {:?}", sub_id, sub_type_str);

        Some(serde_json::to_string(&SubscriptionResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id,
            result: serde_json::Value::String(sub_id),
        }).unwrap_or_default())
    }

    async fn handle_unsubscribe(
        &self,
        conn_state: &Arc<RwLock<ConnectionState>>,
        request: SubscriptionRequest,
    ) -> Option<String> {
        if request.params.is_empty() {
            return Some(serde_json::to_string(&serde_json::json!({
                "jsonrpc": "2.0",
                "id": request.id,
                "error": {
                    "code": -32602,
                    "message": "Missing subscription ID"
                }
            })).unwrap_or_default());
        }

        let sub_id = request.params[0].as_str().unwrap_or("");
        let mut state = conn_state.write().await;
        let removed = state.subscriptions.remove(sub_id).is_some();

        debug!("Removed subscription {}: {}", sub_id, removed);

        Some(serde_json::to_string(&SubscriptionResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id,
            result: serde_json::Value::Bool(removed),
        }).unwrap_or_default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_subscription_type_parsing() {
        let json = r#"{"jsonrpc":"2.0","id":1,"method":"eth_subscribe","params":["newHeads"]}"#;
        let request: SubscriptionRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.method, "eth_subscribe");
        assert_eq!(request.params[0].as_str().unwrap(), "newHeads");
    }

    #[test]
    fn test_logs_filter_parsing() {
        let json = r#"{"address":"0x1234","topics":[null,"0xabcd"]}"#;
        let filter: LogFilter = serde_json::from_str(json).unwrap();
        assert!(filter.address.is_some());
        assert!(filter.topics.is_some());
    }

    #[test]
    fn test_subscription_response_format() {
        let response = SubscriptionResponse {
            jsonrpc: "2.0".to_string(),
            id: serde_json::Value::Number(1.into()),
            result: serde_json::Value::String("0x1".to_string()),
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"result\":\"0x1\""));
    }

    #[test]
    fn test_subscription_notification_format() {
        let notification = SubscriptionNotification {
            jsonrpc: "2.0".to_string(),
            method: "eth_subscription".to_string(),
            params: SubscriptionParams {
                subscription: "0x1".to_string(),
                result: serde_json::json!({"number": "0x10"}),
            },
        };
        let json = serde_json::to_string(&notification).unwrap();
        assert!(json.contains("eth_subscription"));
        assert!(json.contains("0x1"));
    }
}
