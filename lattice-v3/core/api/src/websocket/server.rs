use crate::types::{request::SubscriptionType, response::BlockResponse};
use anyhow::Result;
use futures::stream::StreamExt;
use jsonrpc_ws_server::{Server, ServerBuilder};
use lattice_consensus::types::{Block, Hash, Transaction};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use tracing::{info, debug, error};

/// WebSocket subscription ID
type SubscriptionId = String;

/// WebSocket server for subscriptions
pub struct WebSocketServer {
    listen_addr: SocketAddr,
    subscriptions: Arc<RwLock<HashMap<SubscriptionId, SubscriptionType>>>,
    block_sender: broadcast::Sender<Block>,
    tx_sender: broadcast::Sender<Transaction>,
}

impl WebSocketServer {
    pub fn new(listen_addr: SocketAddr) -> Self {
        let (block_sender, _) = broadcast::channel(100);
        let (tx_sender, _) = broadcast::channel(1000);
        
        Self {
            listen_addr,
            subscriptions: Arc::new(RwLock::new(HashMap::new())),
            block_sender,
            tx_sender,
        }
    }
    
    /// Start the WebSocket server
    pub async fn start(self) -> Result<Server> {
        let mut io = jsonrpc_core::IoHandler::new();
        
        // Register subscription methods
        self.register_subscriptions(&mut io);
        
        let server = ServerBuilder::new(io)
            .start(&self.listen_addr)?;
        
        info!("WebSocket server listening on {}", self.listen_addr);
        Ok(server)
    }
    
    /// Register subscription methods
    fn register_subscriptions(&self, io: &mut jsonrpc_core::IoHandler) {
        let subscriptions = self.subscriptions.clone();
        let block_sender = self.block_sender.clone();
        
        // eth_subscribe
        io.add_method("eth_subscribe", move |params: jsonrpc_core::Params| {
            let subscriptions = subscriptions.clone();
            let mut block_receiver = block_sender.subscribe();
            
            async move {
                let (sub_type, _): (String, Option<jsonrpc_core::Value>) = params.parse()?;
                let subscription_id = format!("0x{}", hex::encode(&rand::random::<[u8; 16]>()));
                
                let sub_type = match sub_type.as_str() {
                    "newHeads" => SubscriptionType::NewHeads,
                    "newPendingTransactions" => SubscriptionType::NewPendingTransactions,
                    "syncing" => SubscriptionType::Syncing,
                    _ => return Err(jsonrpc_core::Error::invalid_params("Invalid subscription type")),
                };
                
                subscriptions.write().await.insert(subscription_id.clone(), sub_type);
                
                // Start subscription handler
                tokio::spawn(async move {
                    while let Ok(block) = block_receiver.recv().await {
                        // Send block to subscriber
                        debug!("New block for subscription: {}", block.hash());
                    }
                });
                
                Ok(jsonrpc_core::Value::String(subscription_id))
            }
        });
        
        // eth_unsubscribe
        let subscriptions = self.subscriptions.clone();
        io.add_method("eth_unsubscribe", move |params: jsonrpc_core::Params| {
            let subscriptions = subscriptions.clone();
            
            async move {
                let subscription_id: String = params.parse()?;
                let removed = subscriptions.write().await.remove(&subscription_id).is_some();
                Ok(jsonrpc_core::Value::Bool(removed))
            }
        });
    }
    
    /// Broadcast new block to subscribers
    pub async fn broadcast_block(&self, block: Block) {
        if let Err(e) = self.block_sender.send(block) {
            error!("Failed to broadcast block: {}", e);
        }
    }
    
    /// Broadcast new transaction to subscribers
    pub async fn broadcast_transaction(&self, tx: Transaction) {
        if let Err(e) = self.tx_sender.send(tx) {
            error!("Failed to broadcast transaction: {}", e);
        }
    }
    
    /// Get active subscription count
    pub async fn subscription_count(&self) -> usize {
        self.subscriptions.read().await.len()
    }
}