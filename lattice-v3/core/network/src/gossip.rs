// lattice-v3/core/network/src/gossip.rs

// Gossip protocol implementation
use crate::{
    peer::{Peer, PeerId, PeerManager},
    NetworkError, NetworkMessage,
};
use dashmap::DashMap;
use lattice_consensus::types::{Block, Hash, Transaction};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, warn};

#[derive(Debug, Clone)]
pub struct GossipConfig {
    /// Maximum items in seen cache
    pub max_seen_cache: usize,

    /// Seen cache TTL
    pub seen_cache_ttl: Duration,

    /// Gossip fanout (number of peers to propagate to)
    pub fanout: usize,

    /// Maximum message size
    pub max_message_size: usize,

    /// Validation timeout
    pub validation_timeout: Duration,
}

impl Default for GossipConfig {
    fn default() -> Self {
        Self {
            max_seen_cache: 10000,
            seen_cache_ttl: Duration::from_secs(600),
            fanout: 8,
            max_message_size: 1024 * 1024, // 1MB
            validation_timeout: Duration::from_millis(100),
        }
    }
}

/// Seen item tracking
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct SeenItem {
    hash: Hash,
    first_seen: Instant,
    propagated: bool,
}

/// Gossip protocol implementation
pub struct GossipProtocol {
    config: GossipConfig,
    peer_manager: Arc<PeerManager>,

    // Seen caches for deduplication
    seen_blocks: Arc<DashMap<Hash, SeenItem>>,
    seen_transactions: Arc<DashMap<Hash, SeenItem>>,

    // Statistics
    stats: Arc<RwLock<GossipStats>>,
}

#[derive(Debug, Default)]
struct GossipStats {
    blocks_received: u64,
    blocks_propagated: u64,
    transactions_received: u64,
    transactions_propagated: u64,
    duplicates_filtered: u64,
}

impl GossipProtocol {
    pub fn new(config: GossipConfig, peer_manager: Arc<PeerManager>) -> Self {
        Self {
            config,
            peer_manager,
            seen_blocks: Arc::new(DashMap::new()),
            seen_transactions: Arc::new(DashMap::new()),
            stats: Arc::new(RwLock::new(GossipStats::default())),
        }
    }

    /// Handle new block announcement
    pub async fn handle_new_block(
        &self,
        block: Block,
        from_peer: &PeerId,
    ) -> Result<(), NetworkError> {
        let hash = block.hash();

        // Check if already seen
        if let Some(seen) = self.seen_blocks.get(&hash) {
            if seen.propagated {
                self.stats.write().await.duplicates_filtered += 1;
                return Ok(());
            }
        }

        // Mark as seen
        self.seen_blocks.insert(
            hash,
            SeenItem {
                hash,
                first_seen: Instant::now(),
                propagated: false,
            },
        );

        self.stats.write().await.blocks_received += 1;

        // Validate block (basic checks)
        if !self.validate_block(&block).await {
            warn!("Invalid block received from {}: {}", from_peer, hash);
            return Err(NetworkError::InvalidMessage("Invalid block".to_string()));
        }

        // Propagate to other peers
        self.propagate_block(block.clone(), from_peer).await?;

        Ok(())
    }

    /// Handle new transaction announcement
    pub async fn handle_new_transaction(
        &self,
        tx: Transaction,
        from_peer: &PeerId,
    ) -> Result<(), NetworkError> {
        let hash = tx.hash;

        // Check if already seen
        if let Some(seen) = self.seen_transactions.get(&hash) {
            if seen.propagated {
                self.stats.write().await.duplicates_filtered += 1;
                return Ok(());
            }
        }

        // Mark as seen
        self.seen_transactions.insert(
            hash,
            SeenItem {
                hash,
                first_seen: Instant::now(),
                propagated: false,
            },
        );

        self.stats.write().await.transactions_received += 1;

        // Validate transaction (basic checks)
        if !self.validate_transaction(&tx).await {
            warn!("Invalid transaction received from {}: {}", from_peer, hash);
            return Err(NetworkError::InvalidMessage(
                "Invalid transaction".to_string(),
            ));
        }

        // Propagate to other peers
        self.propagate_transaction(tx.clone(), from_peer).await?;

        Ok(())
    }

    /// Propagate block to peers
    async fn propagate_block(
        &self,
        block: Block,
        exclude_peer: &PeerId,
    ) -> Result<(), NetworkError> {
        let peers = self.select_gossip_peers(exclude_peer).await;

        if peers.is_empty() {
            return Ok(());
        }

        let block_hash = block.hash();
        let message = NetworkMessage::NewBlock { block };

        for peer in peers {
            if let Err(e) = peer.send(message.clone()).await {
                debug!("Failed to propagate block to peer: {}", e);
            }
        }

        // Mark as propagated
        if let Some(mut seen) = self.seen_blocks.get_mut(&block_hash) {
            seen.propagated = true;
        }

        self.stats.write().await.blocks_propagated += 1;

        Ok(())
    }

    /// Propagate transaction to peers
    async fn propagate_transaction(
        &self,
        tx: Transaction,
        exclude_peer: &PeerId,
    ) -> Result<(), NetworkError> {
        let peers = self.select_gossip_peers(exclude_peer).await;

        if peers.is_empty() {
            return Ok(());
        }

        let tx_hash = tx.hash;
        let message = NetworkMessage::NewTransaction { transaction: tx };

        for peer in peers {
            if let Err(e) = peer.send(message.clone()).await {
                debug!("Failed to propagate transaction to peer: {}", e);
            }
        }

        // Mark as propagated
        if let Some(mut seen) = self.seen_transactions.get_mut(&tx_hash) {
            seen.propagated = true;
        }

        self.stats.write().await.transactions_propagated += 1;

        Ok(())
    }

    /// Select peers for gossip propagation
    async fn select_gossip_peers(&self, exclude: &PeerId) -> Vec<Arc<Peer>> {
        let all_peers = self.peer_manager.get_all_peers();

        let mut eligible: Vec<Arc<Peer>> = Vec::new();

        for peer in all_peers {
            let info = peer.info.read().await;
            if info.id != *exclude && info.state == crate::peer::PeerState::Connected {
                eligible.push(peer.clone());
            }
        }

        // Randomly select up to fanout peers
        use rand::seq::SliceRandom;
        let mut rng = rand::thread_rng();
        eligible.shuffle(&mut rng);
        eligible.truncate(self.config.fanout);

        eligible
    }

    /// Validate block (basic checks)
    async fn validate_block(&self, block: &Block) -> bool {
        // Check block size
        let size = bincode::serialize(block).unwrap_or_default().len();
        if size > self.config.max_message_size {
            return false;
        }

        // Additional validation
        // Check block header validity
        if block.header.height == 0 && !block.is_genesis() {
            return false;
        }

        // Check timestamp is reasonable (not too far in future)
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        if block.header.timestamp > now + 900 {
            // Allow 15 minutes clock drift
            return false;
        }

        // Check blue score is not decreasing
        if block.header.blue_score == 0 && !block.is_genesis() {
            return false;
        }

        // VRF proof must be present for non-genesis blocks
        if !block.is_genesis() && block.header.vrf_reveal.proof.is_empty() {
            return false;
        }

        true
    }

    /// Validate transaction (basic checks)
    async fn validate_transaction(&self, tx: &Transaction) -> bool {
        // Check transaction size
        let size = bincode::serialize(tx).unwrap_or_default().len();
        if size > self.config.max_message_size {
            return false;
        }

        // Basic validation
        if tx.gas_price == 0 || tx.gas_limit == 0 {
            return false;
        }

        // Additional validation
        // Check gas price meets minimum
        const MIN_GAS_PRICE: u64 = 1_000_000_000; // 1 Gwei
        if tx.gas_price < MIN_GAS_PRICE {
            return false;
        }

        // Check gas limit is reasonable
        const MAX_GAS_LIMIT: u64 = 30_000_000;
        if tx.gas_limit > MAX_GAS_LIMIT {
            return false;
        }

        // Validate value doesn't overflow
        if tx.value > u128::MAX / 2 {
            return false;
        }

        // Check data size
        const MAX_TX_DATA_SIZE: usize = 128 * 1024; // 128KB
        if tx.data.len() > MAX_TX_DATA_SIZE {
            return false;
        }

        // Signature must be present (64 bytes for Ed25519)
        // Note: Actual signature verification would be done by the mempool

        true
    }

    /// Clean up old seen items
    pub async fn cleanup_seen_cache(&self) {
        let now = Instant::now();
        let ttl = self.config.seen_cache_ttl;

        // Clean blocks
        self.seen_blocks
            .retain(|_, item| now.duration_since(item.first_seen) < ttl);

        // Clean transactions
        self.seen_transactions
            .retain(|_, item| now.duration_since(item.first_seen) < ttl);

        // Enforce max size
        if self.seen_blocks.len() > self.config.max_seen_cache {
            // Remove oldest entries
            let mut items: Vec<_> = self
                .seen_blocks
                .iter()
                .map(|e| (*e.key(), e.value().first_seen))
                .collect();

            items.sort_by_key(|&(_, time)| time);

            let to_remove = items.len() - self.config.max_seen_cache;
            for (hash, _) in items.into_iter().take(to_remove) {
                self.seen_blocks.remove(&hash);
            }
        }

        if self.seen_transactions.len() > self.config.max_seen_cache {
            let mut items: Vec<_> = self
                .seen_transactions
                .iter()
                .map(|e| (*e.key(), e.value().first_seen))
                .collect();

            items.sort_by_key(|&(_, time)| time);

            let to_remove = items.len() - self.config.max_seen_cache;
            for (hash, _) in items.into_iter().take(to_remove) {
                self.seen_transactions.remove(&hash);
            }
        }
    }

    /// Get gossip statistics
    pub async fn get_stats(&self) -> (u64, u64, u64, u64, u64) {
        let stats = self.stats.read().await;
        (
            stats.blocks_received,
            stats.blocks_propagated,
            stats.transactions_received,
            stats.transactions_propagated,
            stats.duplicates_filtered,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::peer::PeerManagerConfig;

    #[tokio::test]
    async fn test_seen_cache() {
        let peer_manager = Arc::new(PeerManager::new(PeerManagerConfig::default()));
        let gossip = GossipProtocol::new(GossipConfig::default(), peer_manager);

        // Use a simple hash for testing
        let hash = Hash::new([1; 32]);

        // First time seeing block
        assert!(gossip.seen_blocks.get(&hash).is_none());

        // Mark as seen
        gossip.seen_blocks.insert(
            hash,
            SeenItem {
                hash,
                first_seen: Instant::now(),
                propagated: false,
            },
        );

        // Should now be in cache
        assert!(gossip.seen_blocks.get(&hash).is_some());
    }

    #[tokio::test]
    async fn test_cache_cleanup() {
        let config = GossipConfig {
            max_seen_cache: 3,
            ..Default::default()
        };

        let peer_manager = Arc::new(PeerManager::new(PeerManagerConfig::default()));
        let gossip = GossipProtocol::new(config, peer_manager);

        // Add more items than max
        for i in 0..5 {
            let hash = Hash::new([i; 32]);
            gossip.seen_blocks.insert(
                hash,
                SeenItem {
                    hash,
                    first_seen: Instant::now() - Duration::from_secs(i as u64),
                    propagated: false,
                },
            );
        }

        gossip.cleanup_seen_cache().await;

        // Should only keep max_seen_cache items
        assert!(gossip.seen_blocks.len() <= 3);
    }
}
