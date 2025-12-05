// citrate/core/network/src/block_propagation.rs

// Block propagation handler for efficient block distribution
use crate::{NetworkMessage, PeerId, PeerManager};
use anyhow::Result;
use citrate_consensus::types::{Block, BlockHeader, Hash};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};

/// Block propagation handler for efficient block distribution
pub struct BlockPropagation {
    /// Peer manager for network operations
    peer_manager: Arc<PeerManager>,

    /// Track which blocks we've seen from which peers
    block_sources: Arc<RwLock<HashMap<Hash, HashSet<PeerId>>>>,

    /// Recent blocks we've broadcasted (to avoid re-broadcasting)
    recent_broadcasts: Arc<RwLock<HashSet<Hash>>>,

    /// Blocks we're currently downloading
    downloading: Arc<RwLock<HashSet<Hash>>>,

    /// Block header cache
    header_cache: Arc<RwLock<HashMap<Hash, BlockHeader>>>,
}

impl BlockPropagation {
    pub fn new(peer_manager: Arc<PeerManager>) -> Self {
        Self {
            peer_manager,
            block_sources: Arc::new(RwLock::new(HashMap::new())),
            recent_broadcasts: Arc::new(RwLock::new(HashSet::new())),
            downloading: Arc::new(RwLock::new(HashSet::new())),
            header_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Handle new block announcement
    pub async fn handle_new_block(&self, peer_id: &PeerId, block: Block) -> Result<()> {
        let block_hash = block.header.block_hash;

        // Check if we've already seen this block
        let mut sources = self.block_sources.write().await;
        let peers = sources.entry(block_hash).or_insert_with(HashSet::new);

        if peers.contains(peer_id) {
            debug!(
                "Already received block {} from peer {}",
                block_hash, peer_id
            );
            return Ok(());
        }

        peers.insert(peer_id.clone());
        drop(sources);

        info!("Received new block {} from peer {}", block_hash, peer_id);

        // Cache the header
        self.header_cache
            .write()
            .await
            .insert(block_hash, block.header.clone());

        // Propagate to other peers (but not back to sender)
        self.broadcast_block_except(block, peer_id).await?;

        Ok(())
    }

    /// Broadcast a new block to all peers
    pub async fn broadcast_block(&self, block: Block) -> Result<()> {
        let block_hash = block.header.block_hash;

        // Check if we've recently broadcasted this
        let mut recent = self.recent_broadcasts.write().await;
        if recent.contains(&block_hash) {
            debug!("Block {} was recently broadcasted, skipping", block_hash);
            return Ok(());
        }

        recent.insert(block_hash);

        // Clean up old entries if too many
        if recent.len() > 1000 {
            recent.clear();
        }

        drop(recent);

        // Broadcast to all peers
        let message = NetworkMessage::NewBlock { block };
        self.peer_manager.broadcast(&message).await?;

        info!("Broadcasted block {} to all peers", block_hash);
        Ok(())
    }

    /// Broadcast block to all peers except the specified one
    async fn broadcast_block_except(&self, block: Block, except_peer: &PeerId) -> Result<()> {
        let block_hash = block.header.block_hash;

        // Mark as recently broadcasted
        self.recent_broadcasts.write().await.insert(block_hash);

        // Get all peers except the sender
        let all_peers = self.peer_manager.get_all_peers();
        let mut target_peers: Vec<PeerId> = Vec::with_capacity(all_peers.len());

        for peer in all_peers.iter() {
            let peer_id = peer.info.read().await.id.clone();
            if peer_id != *except_peer {
                target_peers.push(peer_id);
            }
        }

        if !target_peers.is_empty() {
            let message = NetworkMessage::NewBlock { block };
            self.peer_manager
                .send_to_peers(&target_peers, &message)
                .await?;

            debug!(
                "Propagated block {} to {} peers",
                block_hash,
                target_peers.len()
            );
        }

        Ok(())
    }

    /// Request specific blocks from peers
    pub async fn request_blocks(&self, from: Hash, count: u32) -> Result<()> {
        // Mark blocks as being downloaded
        self.downloading.write().await.insert(from);

        let message = NetworkMessage::GetBlocks {
            from,
            count,
            step: 1,
        };

        // Request from all peers (could optimize to select best peers)
        self.peer_manager.broadcast(&message).await?;

        info!("Requested {} blocks starting from {}", count, from);
        Ok(())
    }

    /// Handle received blocks response
    pub async fn handle_blocks_response(&self, peer_id: &PeerId, blocks: Vec<Block>) -> Result<()> {
        if blocks.is_empty() {
            return Ok(());
        }

        info!("Received {} blocks from peer {}", blocks.len(), peer_id);

        // Remove from downloading set
        for block in &blocks {
            self.downloading
                .write()
                .await
                .remove(&block.header.block_hash);

            // Cache headers
            self.header_cache
                .write()
                .await
                .insert(block.header.block_hash, block.header.clone());
        }

        // Track sources
        let mut sources = self.block_sources.write().await;
        for block in &blocks {
            sources
                .entry(block.header.block_hash)
                .or_insert_with(HashSet::new)
                .insert(peer_id.clone());
        }

        Ok(())
    }

    /// Request block headers
    pub async fn request_headers(&self, from: Hash, count: u32) -> Result<()> {
        let message = NetworkMessage::GetHeaders { from, count };
        self.peer_manager.broadcast(&message).await?;

        info!("Requested {} headers starting from {}", count, from);
        Ok(())
    }

    /// Handle received headers
    pub async fn handle_headers_response(
        &self,
        peer_id: &PeerId,
        headers: Vec<BlockHeader>,
    ) -> Result<()> {
        if headers.is_empty() {
            return Ok(());
        }

        info!("Received {} headers from peer {}", headers.len(), peer_id);

        // Cache headers
        let mut cache = self.header_cache.write().await;
        for header in headers {
            cache.insert(header.block_hash, header);
        }

        Ok(())
    }

    /// Get cached header
    pub async fn get_cached_header(&self, hash: &Hash) -> Option<BlockHeader> {
        self.header_cache.read().await.get(hash).cloned()
    }

    /// Clean up old data
    pub async fn cleanup(&self) {
        // Clean up old broadcast records
        let mut recent = self.recent_broadcasts.write().await;
        if recent.len() > 10000 {
            recent.clear();
        }

        // Clean up header cache
        let mut cache = self.header_cache.write().await;
        if cache.len() > 50000 {
            // Keep only recent headers (would need timestamp tracking in production)
            let to_remove = cache.len() / 2;
            let keys: Vec<Hash> = cache.keys().take(to_remove).cloned().collect();
            for key in keys {
                cache.remove(&key);
            }
        }

        debug!("Cleaned up block propagation caches");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use citrate_consensus::types::{GhostDagParams, PublicKey, Signature, VrfProof};

    #[tokio::test]
    async fn test_block_propagation() {
        let peer_manager = Arc::new(PeerManager::new(Default::default()));
        let propagation = BlockPropagation::new(peer_manager);

        // Create a test block
        let block = Block {
            header: BlockHeader {
                version: 1,
                block_hash: Hash::new([1; 32]),
                selected_parent_hash: Hash::new([2; 32]),
                merge_parent_hashes: vec![],
                timestamp: 12345,
                height: 100,
                blue_score: 50,
                blue_work: 1000,
                pruning_point: Hash::new([0; 32]),
                proposer_pubkey: PublicKey::new([0; 32]),
                vrf_reveal: VrfProof {
                    proof: vec![],
                    output: Hash::new([0; 32]),
                },
            },
            state_root: Hash::new([3; 32]),
            tx_root: Hash::new([4; 32]),
            receipt_root: Hash::new([5; 32]),
            artifact_root: Hash::new([6; 32]),
            ghostdag_params: GhostDagParams::default(),
            transactions: vec![],
            signature: Signature::new([0; 64]),
        };

        // Test broadcasting
        assert!(propagation.broadcast_block(block.clone()).await.is_ok());

        // Should skip re-broadcasting
        assert!(propagation.broadcast_block(block.clone()).await.is_ok());

        // Test header caching
        let peer_id = PeerId::new("test_peer".to_string());
        assert!(propagation
            .handle_new_block(&peer_id, block.clone())
            .await
            .is_ok());

        let cached = propagation
            .get_cached_header(&block.header.block_hash)
            .await;
        assert!(cached.is_some());
        assert_eq!(cached.unwrap().block_hash, block.header.block_hash);
    }
}
