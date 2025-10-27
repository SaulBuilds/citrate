// citrate/core/api/src/methods/network.rs
use crate::types::{error::ApiError, response::NodeInfo};
use citrate_consensus::types::Hash;
use citrate_network::peer::PeerManager;
use std::sync::Arc;

/// Network-related API methods
pub struct NetworkApi {
    peer_manager: Arc<PeerManager>,
}

impl NetworkApi {
    pub fn new(peer_manager: Arc<PeerManager>) -> Self {
        Self { peer_manager }
    }

    /// Get network version
    pub async fn get_version(&self) -> Result<String, ApiError> {
        Ok("1".to_string())
    }

    /// Get peer count
    pub async fn get_peer_count(&self) -> Result<usize, ApiError> {
        let (total, _, _) = self.peer_manager.get_peer_counts().await;
        Ok(total)
    }

    /// Check if node is listening for connections
    pub async fn is_listening(&self) -> Result<bool, ApiError> {
        // Check if we can accept more peers
        let (total, _, _) = self.peer_manager.get_peer_counts().await;
        Ok(total < self.peer_manager.max_peers())
    }

    /// Get node information
    pub async fn get_node_info(&self) -> Result<NodeInfo, ApiError> {
        let (peer_count, _, _) = self.peer_manager.get_peer_counts().await;

        Ok(NodeInfo {
            version: "citrate/v0.1.0".to_string(),
            network_id: 1,
            chain_id: 1337,
            genesis_hash: Hash::default(),
            head_hash: Hash::default(),
            head_height: 0,
            peer_count,
        })
    }

    /// Get connected peers
    pub async fn get_peers(&self) -> Result<Vec<String>, ApiError> {
        let peers = self.peer_manager.get_all_peers();
        let mut peer_ids = Vec::new();

        for peer in peers {
            let info = peer.info.read().await;
            peer_ids.push(info.id.to_string());
        }

        Ok(peer_ids)
    }
}
