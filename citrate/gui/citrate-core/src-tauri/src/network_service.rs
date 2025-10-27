use std::sync::Arc;
use anyhow::Result;
use tokio::sync::RwLock;
use tracing::{info, warn, error, debug};

use citrate_consensus::{
    GhostDag,
    types::{Block, Hash, Transaction},
};
use citrate_storage::StorageManager;
use citrate_network::{
    PeerManager, NetworkConfig, PeerManagerConfig,
    message::{NetworkMessage, BlockAnnouncement, TransactionBroadcast},
    peer::{PeerId, PeerInfo},
};
use citrate_sequencer::mempool::Mempool;

/// Manages P2P networking for the blockchain
pub struct NetworkService {
    peer_manager: Arc<PeerManager>,
    ghostdag: Arc<GhostDag>,
    mempool: Arc<RwLock<Mempool>>,
    storage: Arc<StorageManager>,
    running: Arc<RwLock<bool>>,
}

impl NetworkService {
    pub async fn new(
        config: NetworkConfig,
        ghostdag: Arc<GhostDag>,
        mempool: Arc<RwLock<Mempool>>,
        storage: Arc<StorageManager>,
    ) -> Result<Self> {
        info!("Initializing network service with config: {:?}", config);
        
        let peer_config = PeerManagerConfig {
            max_peers: config.max_peers,
            enable_discovery: config.enable_discovery,
            enable_mdns: config.enable_mdns,
            peer_score_threshold: config.peer_score_threshold,
            bootstrap_nodes: config.bootstrap_nodes.clone(),
        };
        
        let peer_manager = Arc::new(PeerManager::new(
            config.listen_addr,
            peer_config,
        ).await?);
        
        Ok(Self {
            peer_manager,
            ghostdag,
            mempool,
            storage,
            running: Arc::new(RwLock::new(false)),
        })
    }

    pub async fn start(&self) -> Result<()> {
        info!("Starting network service");
        *self.running.write().await = true;
        
        // Start peer manager
        self.peer_manager.start().await?;
        
        // Start message handler
        let service = self.clone_service();
        tokio::spawn(async move {
            service.handle_messages().await;
        });
        
        // Start peer discovery
        let service = self.clone_service();
        tokio::spawn(async move {
            service.discover_peers().await;
        });
        
        // Start heartbeat
        let service = self.clone_service();
        tokio::spawn(async move {
            service.send_heartbeats().await;
        });
        
        info!("Network service started successfully");
        Ok(())
    }

    pub async fn stop(&self) -> Result<()> {
        info!("Stopping network service");
        *self.running.write().await = false;
        self.peer_manager.stop().await?;
        info!("Network service stopped");
        Ok(())
    }

    pub async fn get_peer_count(&self) -> usize {
        self.peer_manager.get_peer_count().await
    }

    pub async fn get_peers(&self) -> Vec<PeerInfo> {
        self.peer_manager.get_peers().await
    }

    pub async fn broadcast_block(&self, block: &Block) -> Result<()> {
        let announcement = BlockAnnouncement {
            block_hash: block.hash,
            height: block.header.height,
            selected_parent: block.header.selected_parent_hash,
            merge_parents: block.header.merge_parent_hashes.clone(),
            blue_score: block.header.blue_score,
        };
        
        let message = NetworkMessage::BlockAnnouncement(announcement);
        self.peer_manager.broadcast(&message).await?;
        
        debug!("Broadcasted block {} at height {}", 
               hex::encode(&block.hash.as_bytes()[..8]), block.header.height);
        Ok(())
    }

    pub async fn broadcast_transaction(&self, tx: &Transaction) -> Result<()> {
        let broadcast = TransactionBroadcast {
            transaction: tx.clone(),
        };
        
        let message = NetworkMessage::TransactionBroadcast(broadcast);
        self.peer_manager.broadcast(&message).await?;
        
        debug!("Broadcasted transaction {}", hex::encode(&tx.hash.as_bytes()[..8]));
        Ok(())
    }

    pub async fn request_block(&self, hash: &Hash, peer: &PeerId) -> Result<Block> {
        self.peer_manager.request_block(hash, peer).await
    }

    pub async fn request_blocks(&self, from_height: u64, to_height: u64) -> Result<Vec<Block>> {
        // Request blocks from peers for syncing
        let peers = self.get_peers().await;
        if peers.is_empty() {
            return Err(anyhow::anyhow!("No peers available for block request"));
        }
        
        // Pick the best peer (highest height)
        let best_peer = peers.iter()
            .max_by_key(|p| p.best_height)
            .ok_or_else(|| anyhow::anyhow!("No suitable peer found"))?;
        
        self.peer_manager.request_block_range(
            from_height,
            to_height,
            &best_peer.peer_id,
        ).await
    }

    async fn handle_messages(&self) {
        info!("Message handler started");
        
        while *self.running.read().await {
            // Get next message from peer manager
            match self.peer_manager.recv_message().await {
                Some((_peer, message)) => {
                    if let Err(e) = self.handle_message(message).await {
                        warn!("Failed to handle message: {}", e);
                    }
                }
                None => {
                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                }
            }
        }
        
        info!("Message handler stopped");
    }

    async fn handle_message(&self, message: NetworkMessage) -> Result<()> {
        match message {
            NetworkMessage::BlockAnnouncement(announcement) => {
                debug!("Received block announcement for height {}", announcement.height);
                
                // Check if we already have this block
                if self.storage.blocks.has_block(&announcement.block_hash)? {
                    return Ok(());
                }
                
                // Request the full block from peers
                let peers = self.get_peers().await;
                if let Some(peer) = peers.first() {
                    match self.request_block(&announcement.block_hash, &peer.peer_id).await {
                        Ok(block) => {
                            // Validate and add to DAG
                            if let Err(e) = self.ghostdag.add_block(&block).await {
                                warn!("Failed to add announced block to DAG: {}", e);
                            } else {
                                // Store the block
                                self.storage.blocks.insert_block(&block)?;
                                info!("Added announced block {} at height {}", 
                                      hex::encode(&block.hash.as_bytes()[..8]), block.header.height);
                            }
                        }
                        Err(e) => {
                            warn!("Failed to request announced block: {}", e);
                        }
                    }
                }
            }
            
            NetworkMessage::TransactionBroadcast(broadcast) => {
                debug!("Received transaction broadcast");
                
                // Add to mempool
                let mut pool = self.mempool.write().await;
                if let Err(e) = pool.add_transaction(broadcast.transaction) {
                    debug!("Failed to add broadcasted transaction to mempool: {}", e);
                }
            }
            
            _ => {
                debug!("Received unhandled message type");
            }
        }
        
        Ok(())
    }

    async fn discover_peers(&self) {
        info!("Peer discovery started");
        
        while *self.running.read().await {
            // Trigger peer discovery
            if let Err(e) = self.peer_manager.discover_peers().await {
                warn!("Peer discovery failed: {}", e);
            }
            
            // Wait before next discovery round
            tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
        }
        
        info!("Peer discovery stopped");
    }

    async fn send_heartbeats(&self) {
        info!("Heartbeat sender started");
        
        while *self.running.read().await {
            // Get current chain state
            let height = self.storage.blocks.get_latest_height().unwrap_or(0);
            let tips = self.ghostdag.get_tips().await;
            
            // Send heartbeat to all peers
            if let Err(e) = self.peer_manager.send_heartbeat(height, tips).await {
                debug!("Failed to send heartbeat: {}", e);
            }
            
            // Wait before next heartbeat
            tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
        }
        
        info!("Heartbeat sender stopped");
    }

    fn clone_service(&self) -> NetworkService {
        NetworkService {
            peer_manager: self.peer_manager.clone(),
            ghostdag: self.ghostdag.clone(),
            mempool: self.mempool.clone(),
            storage: self.storage.clone(),
            running: self.running.clone(),
        }
    }
}