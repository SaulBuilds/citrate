use crate::{NetworkMessage, PeerManager, PeerId};
use lattice_consensus::types::{Transaction, Hash, TransactionType};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{info, debug, warn};
use anyhow::Result;

/// Transaction seen info
#[derive(Clone, Debug)]
struct TxSeenInfo {
    first_seen: Instant,
    peers: HashSet<PeerId>,
    broadcast_count: u32,
}

/// Transaction gossip configuration
#[derive(Clone, Debug)]
pub struct GossipConfig {
    /// Maximum transactions to keep in seen cache
    pub max_seen_txs: usize,
    /// Time to keep transactions in seen cache
    pub tx_ttl: Duration,
    /// Maximum peers to relay to at once
    pub max_relay_peers: usize,
    /// Delay before relaying AI transactions (for bundling)
    pub ai_tx_delay: Duration,
}

impl Default for GossipConfig {
    fn default() -> Self {
        Self {
            max_seen_txs: 100_000,
            tx_ttl: Duration::from_secs(600), // 10 minutes
            max_relay_peers: 10,
            ai_tx_delay: Duration::from_millis(100), // 100ms delay for AI tx bundling
        }
    }
}

/// Transaction gossip handler for efficient mempool synchronization
pub struct TransactionGossip {
    /// Configuration
    config: GossipConfig,
    
    /// Peer manager
    peer_manager: Arc<PeerManager>,
    
    /// Transactions we've seen (hash -> info)
    seen_txs: Arc<RwLock<HashMap<Hash, TxSeenInfo>>>,
    
    /// AI transactions pending relay (for bundling)
    pending_ai_txs: Arc<RwLock<Vec<Transaction>>>,
    
    /// Transaction inventory by peer (peer -> set of tx hashes)
    peer_inventory: Arc<RwLock<HashMap<PeerId, HashSet<Hash>>>>,
}

impl TransactionGossip {
    pub fn new(peer_manager: Arc<PeerManager>, config: GossipConfig) -> Self {
        Self {
            config,
            peer_manager,
            seen_txs: Arc::new(RwLock::new(HashMap::new())),
            pending_ai_txs: Arc::new(RwLock::new(Vec::new())),
            peer_inventory: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Handle new transaction from peer
    pub async fn handle_new_transaction(
        &self,
        peer_id: &PeerId,
        tx: Transaction,
    ) -> Result<bool> {
        let tx_hash = tx.hash;
        
        // Check if we've seen this transaction
        let mut seen = self.seen_txs.write().await;
        let is_new = if let Some(info) = seen.get_mut(&tx_hash) {
            if info.peers.contains(peer_id) {
                debug!("Already received tx {} from peer {}", tx_hash, peer_id);
                return Ok(false);
            }
            info.peers.insert(peer_id.clone());
            info.broadcast_count += 1;
            let broadcast_count = info.broadcast_count;
            drop(seen);
            
            // Don't relay if we've already broadcasted enough
            if broadcast_count > 3 {
                return Ok(false);
            }
            false
        } else {
            // New transaction
            let mut new_info = TxSeenInfo {
                first_seen: Instant::now(),
                peers: HashSet::new(),
                broadcast_count: 0,
            };
            new_info.peers.insert(peer_id.clone());
            seen.insert(tx_hash, new_info);
            drop(seen);
            
            info!("New transaction {} from peer {} (type: {:?})", 
                tx_hash, peer_id, tx.tx_type);
            true
        };
        
        // Update peer inventory
        self.peer_inventory
            .write()
            .await
            .entry(peer_id.clone())
            .or_insert_with(HashSet::new)
            .insert(tx_hash);
        
        // Handle based on transaction type
        match tx.tx_type {
            Some(TransactionType::ModelDeploy) |
            Some(TransactionType::ModelUpdate) |
            Some(TransactionType::InferenceRequest) |
            Some(TransactionType::TrainingJob) |
            Some(TransactionType::LoraAdapter) => {
                // AI transaction - add to pending for bundled relay
                self.pending_ai_txs.write().await.push(tx.clone());
                
                // Schedule bundled relay
                let gossip = self.clone();
                tokio::spawn(async move {
                    tokio::time::sleep(gossip.config.ai_tx_delay).await;
                    let _ = gossip.relay_pending_ai_txs().await;
                });
            }
            _ => {
                // Standard transaction - relay immediately
                self.relay_transaction(tx, Some(peer_id)).await?;
            }
        }
        
        Ok(true)
    }
    
    /// Broadcast a new local transaction
    pub async fn broadcast_transaction(&self, tx: Transaction) -> Result<()> {
        let tx_hash = tx.hash;
        
        // Mark as seen
        let mut seen = self.seen_txs.write().await;
        if seen.contains_key(&tx_hash) {
            debug!("Transaction {} already broadcasted", tx_hash);
            return Ok(());
        }
        
        seen.insert(tx_hash, TxSeenInfo {
            first_seen: Instant::now(),
            peers: HashSet::new(),
            broadcast_count: 1,
        });
        drop(seen);
        
        info!("Broadcasting new transaction {} (type: {:?})", tx_hash, tx.tx_type);
        
        // Relay to peers
        self.relay_transaction(tx, None).await
    }
    
    /// Relay transaction to peers
    async fn relay_transaction(
        &self,
        tx: Transaction,
        except_peer: Option<&PeerId>,
    ) -> Result<()> {
        // Get target peers
        let all_peers = self.peer_manager.get_all_peers();
        let mut target_peers: Vec<PeerId> = Vec::new();
        
        for _peer in all_peers.iter().take(self.config.max_relay_peers) {
            // TODO: Get actual peer ID from peer object
            let peer_id = PeerId::new(format!("peer_{}", rand::random::<u64>()));
            
            if let Some(except) = except_peer {
                if peer_id == *except {
                    continue;
                }
            }
            
            // Check if peer already has this tx
            let inventory = self.peer_inventory.read().await;
            if let Some(peer_txs) = inventory.get(&peer_id) {
                if peer_txs.contains(&tx.hash) {
                    continue;
                }
            }
            drop(inventory);
            
            target_peers.push(peer_id);
        }
        
        if !target_peers.is_empty() {
            let message = NetworkMessage::NewTransaction { transaction: tx };
            self.peer_manager.send_to_peers(&target_peers, &message).await?;
            
            debug!("Relayed transaction to {} peers", target_peers.len());
        }
        
        Ok(())
    }
    
    /// Relay pending AI transactions as a bundle
    async fn relay_pending_ai_txs(&self) -> Result<()> {
        let txs: Vec<Transaction> = {
            let mut pending = self.pending_ai_txs.write().await;
            if pending.is_empty() {
                return Ok(());
            }
            pending.drain(..).collect()
        };
        
        info!("Relaying bundle of {} AI transactions", txs.len());
        
        // Send each transaction (could optimize with batch message)
        for tx in txs {
            self.relay_transaction(tx, None).await?;
        }
        
        Ok(())
    }
    
    /// Request specific transactions from peers
    pub async fn request_transactions(&self, hashes: Vec<Hash>) -> Result<()> {
        if hashes.is_empty() {
            return Ok(());
        }
        
        let message = NetworkMessage::GetTransactions { hashes: hashes.clone() };
        self.peer_manager.broadcast(&message).await?;
        
        info!("Requested {} transactions from peers", hashes.len());
        Ok(())
    }
    
    /// Handle transaction response
    pub async fn handle_transactions_response(
        &self,
        peer_id: &PeerId,
        transactions: Vec<Transaction>,
    ) -> Result<()> {
        if transactions.is_empty() {
            return Ok(());
        }
        
        info!("Received {} transactions from peer {}", transactions.len(), peer_id);
        
        // Process each transaction
        for tx in transactions {
            self.handle_new_transaction(peer_id, tx).await?;
        }
        
        Ok(())
    }
    
    /// Get mempool summary for peer
    pub async fn get_mempool_summary(&self) -> Vec<Hash> {
        let seen = self.seen_txs.read().await;
        seen.keys().cloned().collect()
    }
    
    /// Handle mempool request
    pub async fn handle_mempool_request(&self, peer_id: &PeerId) -> Result<()> {
        let tx_hashes = self.get_mempool_summary().await;
        
        let message = NetworkMessage::Mempool { tx_hashes };
        
        if let Some(peer) = self.peer_manager.get_peer(peer_id) {
            peer.send(message).await?;
            info!("Sent mempool summary to peer {}", peer_id);
        }
        
        Ok(())
    }
    
    /// Clean up old transactions
    pub async fn cleanup(&self) {
        let mut seen = self.seen_txs.write().await;
        let now = Instant::now();
        
        // Remove old transactions
        seen.retain(|hash, info| {
            let age = now.duration_since(info.first_seen);
            if age > self.config.tx_ttl {
                debug!("Removing old transaction {} from cache", hash);
                false
            } else {
                true
            }
        });
        
        // Limit cache size
        if seen.len() > self.config.max_seen_txs {
            let to_remove = seen.len() - self.config.max_seen_txs;
            let mut oldest: Vec<(Hash, Instant)> = seen
                .iter()
                .map(|(h, i)| (*h, i.first_seen))
                .collect();
            oldest.sort_by_key(|(_h, t)| *t);
            
            for (hash, _) in oldest.iter().take(to_remove) {
                seen.remove(hash);
            }
        }
        
        debug!("Cleaned up transaction cache, {} entries remaining", seen.len());
    }
}

impl Clone for TransactionGossip {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            peer_manager: self.peer_manager.clone(),
            seen_txs: self.seen_txs.clone(),
            pending_ai_txs: self.pending_ai_txs.clone(),
            peer_inventory: self.peer_inventory.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lattice_consensus::types::{PublicKey, Signature};
    
    #[tokio::test]
    async fn test_transaction_gossip() {
        let peer_manager = Arc::new(PeerManager::new(Default::default()));
        let gossip = TransactionGossip::new(peer_manager, Default::default());
        
        // Create test transaction
        let tx = Transaction {
            hash: Hash::new([1; 32]),
            nonce: 1,
            from: PublicKey::new([2; 32]),
            to: Some(PublicKey::new([3; 32])),
            value: 1000,
            gas_limit: 21000,
            gas_price: 100,
            data: vec![],
            signature: Signature::new([0; 64]),
            tx_type: Some(TransactionType::Standard),
        };
        
        // Test broadcasting
        assert!(gossip.broadcast_transaction(tx.clone()).await.is_ok());
        
        // Should be marked as seen
        let seen = gossip.seen_txs.read().await;
        assert!(seen.contains_key(&tx.hash));
        drop(seen);
        
        // Test handling from peer
        let peer_id = PeerId::new("test_peer".to_string());
        let is_new = gossip.handle_new_transaction(&peer_id, tx.clone()).await.unwrap();
        assert!(!is_new); // Should not be new since we already have it
        
        // Test AI transaction bundling
        let ai_tx = Transaction {
            hash: Hash::new([10; 32]),
            tx_type: Some(TransactionType::ModelDeploy),
            ..tx
        };
        
        assert!(gossip.handle_new_transaction(&peer_id, ai_tx).await.unwrap());
        
        // AI tx should be in pending
        let pending = gossip.pending_ai_txs.read().await;
        assert_eq!(pending.len(), 1);
    }
}