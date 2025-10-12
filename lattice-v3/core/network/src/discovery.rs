use crate::{
    peer::{PeerId, PeerManager},
    protocol::PeerAddress,
    NetworkError,
};
use dashmap::DashMap;
use std::collections::HashSet;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use tokio::time;
use tracing::{debug, info};

#[derive(Debug, Clone)]
pub struct DiscoveryConfig {
    /// Bootstrap nodes
    pub bootstrap_nodes: Vec<String>,

    /// Maximum peers to discover
    pub max_peers: usize,

    /// Discovery interval
    pub discovery_interval: Duration,

    /// Peer exchange size
    pub peer_exchange_size: usize,

    /// Peer expiry time
    pub peer_expiry: Duration,
}

impl Default for DiscoveryConfig {
    fn default() -> Self {
        Self {
            bootstrap_nodes: Vec::new(),
            max_peers: 100,
            discovery_interval: Duration::from_secs(30),
            peer_exchange_size: 10,
            peer_expiry: Duration::from_secs(3600),
        }
    }
}

/// Known peer information
#[derive(Debug, Clone)]
struct KnownPeer {
    id: String,
    addr: SocketAddr,
    last_seen: u64,
    score: i32,
    attempts: u32,
}

/// Peer discovery service
pub struct Discovery {
    config: DiscoveryConfig,
    known_peers: Arc<DashMap<String, KnownPeer>>,
    connected_peers: Arc<RwLock<HashSet<String>>>,
    peer_manager: Arc<PeerManager>,
}

impl Discovery {
    pub fn new(config: DiscoveryConfig, peer_manager: Arc<PeerManager>) -> Self {
        Self {
            config,
            known_peers: Arc::new(DashMap::new()),
            connected_peers: Arc::new(RwLock::new(HashSet::new())),
            peer_manager,
        }
    }

    /// Initialize with bootstrap nodes
    pub async fn init(&self) -> Result<(), NetworkError> {
        for node in &self.config.bootstrap_nodes {
            if let Ok(addr) = node.parse::<SocketAddr>() {
                self.add_peer(
                    format!("bootstrap_{}", node),
                    addr,
                    100, // High score for bootstrap nodes
                )
                .await;
            }
        }

        info!(
            "Initialized discovery with {} bootstrap nodes",
            self.config.bootstrap_nodes.len()
        );
        Ok(())
    }

    /// Add a discovered peer
    pub async fn add_peer(&self, id: String, addr: SocketAddr, score: i32) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let peer = KnownPeer {
            id: id.clone(),
            addr,
            last_seen: now,
            score,
            attempts: 0,
        };

        self.known_peers.insert(id, peer);
        debug!("Added peer to discovery: {}", addr);
    }

    /// Mark peer as connected
    pub async fn mark_connected(&self, peer_id: &str) {
        self.connected_peers
            .write()
            .await
            .insert(peer_id.to_string());
    }

    /// Mark peer as disconnected
    pub async fn mark_disconnected(&self, peer_id: &str) {
        self.connected_peers.write().await.remove(peer_id);
    }

    /// Get peers for exchange
    pub async fn get_peers_for_exchange(&self) -> Vec<PeerAddress> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let connected = self.connected_peers.read().await;

        let mut peers: Vec<PeerAddress> = self
            .known_peers
            .iter()
            .filter(|p| {
                !connected.contains(&p.value().id)
                    && (now - p.value().last_seen) < self.config.peer_expiry.as_secs()
            })
            .take(self.config.peer_exchange_size)
            .map(|p| PeerAddress {
                id: p.value().id.clone(),
                addr: p.value().addr.to_string(),
                last_seen: p.value().last_seen,
                score: p.value().score,
            })
            .collect();

        peers.sort_by(|a, b| b.score.cmp(&a.score));
        peers.truncate(self.config.peer_exchange_size);

        peers
    }

    /// Handle peer exchange
    pub async fn handle_peer_exchange(&self, peers: Vec<PeerAddress>) {
        for peer in peers {
            if let Ok(addr) = peer.addr.parse::<SocketAddr>() {
                // Skip if already connected or banned
                if self.connected_peers.read().await.contains(&peer.id) {
                    continue;
                }

                if self.peer_manager.is_banned(&addr).await {
                    continue;
                }

                self.add_peer(peer.id, addr, peer.score).await;
            }
        }
    }

    /// Find new peers to connect to
    pub async fn find_peers(&self) -> Vec<(String, SocketAddr)> {
        let connected = self.connected_peers.read().await;
        let (current_peers, _, _) = self.peer_manager.get_peer_counts().await;

        if current_peers >= self.config.max_peers {
            return Vec::new();
        }

        let needed = self.config.max_peers - current_peers;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let mut candidates: Vec<_> = self
            .known_peers
            .iter()
            .filter(|p| {
                !connected.contains(&p.value().id)
                    && p.value().attempts < 3
                    && (now - p.value().last_seen) < self.config.peer_expiry.as_secs()
            })
            .map(|p| p.value().clone())
            .collect();

        // Sort by score and attempts
        candidates.sort_by(|a, b| b.score.cmp(&a.score).then(a.attempts.cmp(&b.attempts)));

        candidates
            .into_iter()
            .take(needed)
            .map(|p| (p.id, p.addr))
            .collect()
    }

    /// Update peer attempts
    pub async fn update_attempts(&self, peer_id: &str, success: bool) {
        if let Some(mut peer) = self.known_peers.get_mut(peer_id) {
            if success {
                peer.attempts = 0;
                peer.score = (peer.score + 10).min(100);
            } else {
                peer.attempts += 1;
                peer.score = (peer.score - 5).max(-100);
            }
        }
    }

    /// Clean up expired peers
    pub async fn cleanup_expired(&self) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let expired: Vec<String> = self
            .known_peers
            .iter()
            .filter(|p| (now - p.value().last_seen) > self.config.peer_expiry.as_secs())
            .map(|p| p.key().clone())
            .collect();

        for id in expired {
            self.known_peers.remove(&id);
            debug!("Removed expired peer: {}", id);
        }
    }

    /// Run discovery loop
    pub async fn run(&self) {
        let mut interval = time::interval(self.config.discovery_interval);

        loop {
            interval.tick().await;

            // Clean up expired peers
            self.cleanup_expired().await;

            // Find new peers to connect to
            let candidates = self.find_peers().await;

            if !candidates.is_empty() {
                info!("Discovery found {} potential peers", candidates.len());

                // Initiate connections to candidates
                for (id, addr) in candidates {
                    debug!("Attempting to connect to: {} ({})", id, addr);

                    match self
                        .peer_manager
                        .connect_to_peer(PeerId::new(id.clone()), addr)
                        .await
                    {
                        Ok(_) => {
                            info!("Successfully initiated connection to {}", id);
                            self.mark_connected(&id).await;
                            self.update_attempts(&id, true).await;
                        }
                        Err(e) => {
                            debug!("Failed to connect to {}: {}", id, e);
                            self.update_attempts(&id, false).await;
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::peer::PeerManagerConfig;

    #[tokio::test]
    async fn test_discovery_bootstrap() {
        let config = DiscoveryConfig {
            bootstrap_nodes: vec!["127.0.0.1:8001".to_string(), "127.0.0.1:8002".to_string()],
            ..Default::default()
        };

        let peer_manager = Arc::new(PeerManager::new(PeerManagerConfig::default()));
        let discovery = Discovery::new(config, peer_manager);

        discovery.init().await.unwrap();

        assert_eq!(discovery.known_peers.len(), 2);
    }

    #[tokio::test]
    async fn test_peer_exchange() {
        let config = DiscoveryConfig::default();
        let peer_manager = Arc::new(PeerManager::new(PeerManagerConfig::default()));
        let discovery = Discovery::new(config, peer_manager);

        // Add some peers
        discovery
            .add_peer("peer1".to_string(), "127.0.0.1:8001".parse().unwrap(), 50)
            .await;
        discovery
            .add_peer("peer2".to_string(), "127.0.0.1:8002".parse().unwrap(), 75)
            .await;
        discovery
            .add_peer("peer3".to_string(), "127.0.0.1:8003".parse().unwrap(), 25)
            .await;

        // Mark one as connected
        discovery.mark_connected("peer1").await;

        let peers = discovery.get_peers_for_exchange().await;

        // Should only return non-connected peers, sorted by score
        assert_eq!(peers.len(), 2);
        assert_eq!(peers[0].id, "peer2"); // Highest score
        assert_eq!(peers[1].id, "peer3");
    }
}
