use crate::{NetworkError, NetworkMessage, ProtocolVersion};
use dashmap::DashMap;
use lattice_consensus::types::Hash;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, RwLock};
use tracing::{debug, info, warn};

/// Unique peer identifier
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct PeerId(pub String);

impl PeerId {
    pub fn new(id: String) -> Self {
        Self(id)
    }
    
    pub fn random() -> Self {
        Self(format!("peer_{}", rand::random::<u64>()))
    }
}

impl std::fmt::Display for PeerId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Peer connection state
#[derive(Debug, Clone, PartialEq)]
pub enum PeerState {
    Connecting,
    Handshaking,
    Connected,
    Disconnecting,
    Disconnected,
}

/// Peer connection direction
#[derive(Debug, Clone, PartialEq)]
pub enum Direction {
    Inbound,
    Outbound,
}

/// Information about a peer
#[derive(Debug, Clone)]
pub struct PeerInfo {
    pub id: PeerId,
    pub addr: SocketAddr,
    pub state: PeerState,
    pub direction: Direction,
    pub version: Option<ProtocolVersion>,
    pub head_height: u64,
    pub head_hash: Hash,
    pub connected_at: Instant,
    pub last_seen: Instant,
    pub messages_sent: u64,
    pub messages_received: u64,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub score: i32,
}

impl PeerInfo {
    pub fn new(id: PeerId, addr: SocketAddr, direction: Direction) -> Self {
        let now = Instant::now();
        Self {
            id,
            addr,
            state: PeerState::Connecting,
            direction,
            version: None,
            head_height: 0,
            head_hash: Hash::default(),
            connected_at: now,
            last_seen: now,
            messages_sent: 0,
            messages_received: 0,
            bytes_sent: 0,
            bytes_received: 0,
            score: 0,
        }
    }
    
    pub fn update_last_seen(&mut self) {
        self.last_seen = Instant::now();
    }
    
    pub fn is_stale(&self, timeout: Duration) -> bool {
        self.last_seen.elapsed() > timeout
    }
}

/// Individual peer connection
pub struct Peer {
    pub info: Arc<RwLock<PeerInfo>>,
    pub send_tx: mpsc::Sender<NetworkMessage>,
    pub recv_tx: mpsc::Receiver<NetworkMessage>,
}

impl Peer {
    pub fn new(
        info: PeerInfo,
        send_tx: mpsc::Sender<NetworkMessage>,
        recv_tx: mpsc::Receiver<NetworkMessage>,
    ) -> Self {
        Self {
            info: Arc::new(RwLock::new(info)),
            send_tx,
            recv_tx,
        }
    }
    
    pub async fn send(&self, message: NetworkMessage) -> Result<(), NetworkError> {
        self.send_tx
            .send(message)
            .await
            .map_err(|_| NetworkError::ConnectionFailed("Channel closed".to_string()))?;
        
        let mut info = self.info.write().await;
        info.messages_sent += 1;
        info.update_last_seen();
        
        Ok(())
    }
    
    pub async fn disconnect(&self, reason: String) -> Result<(), NetworkError> {
        self.send(NetworkMessage::Disconnect { reason }).await?;
        
        let mut info = self.info.write().await;
        info.state = PeerState::Disconnected;
        
        Ok(())
    }
}

/// Peer manager for handling multiple connections
pub struct PeerManager {
    config: PeerManagerConfig,
    peers: Arc<DashMap<PeerId, Arc<Peer>>>,
    banned_peers: Arc<RwLock<Vec<SocketAddr>>>,
    stats: Arc<RwLock<PeerStats>>,
}

#[derive(Debug, Clone)]
pub struct PeerManagerConfig {
    pub max_peers: usize,
    pub max_inbound: usize,
    pub max_outbound: usize,
    pub peer_timeout: Duration,
    pub ban_duration: Duration,
    pub score_threshold: i32,
}

impl Default for PeerManagerConfig {
    fn default() -> Self {
        Self {
            max_peers: 50,
            max_inbound: 30,
            max_outbound: 20,
            peer_timeout: Duration::from_secs(120),
            ban_duration: Duration::from_secs(3600),
            score_threshold: -100,
        }
    }
}

#[derive(Debug, Default)]
struct PeerStats {
    total_connected: usize,
    inbound_count: usize,
    outbound_count: usize,
}

impl PeerManager {
    pub fn new(config: PeerManagerConfig) -> Self {
        Self {
            config,
            peers: Arc::new(DashMap::new()),
            banned_peers: Arc::new(RwLock::new(Vec::new())),
            stats: Arc::new(RwLock::new(PeerStats::default())),
        }
    }
    
    /// Get max peers configuration
    pub fn max_peers(&self) -> usize {
        self.config.max_peers
    }
    
    /// Connect to a peer
    pub async fn connect_to_peer(&self, peer_id: PeerId, addr: SocketAddr) -> Result<(), NetworkError> {
        // Check if already connected
        if self.peers.contains_key(&peer_id) {
            return Ok(());
        }
        
        // Check if banned
        if self.is_banned(&addr).await {
            return Err(NetworkError::ConnectionFailed("Peer is banned".to_string()));
        }
        
        // Create channels for communication
        let (send_tx, _send_rx) = mpsc::channel(100);
        let (_recv_tx, recv_rx) = mpsc::channel(100);
        
        // Create peer info and peer
        let info = PeerInfo::new(peer_id.clone(), addr, Direction::Outbound);
        let peer = Arc::new(Peer::new(info, send_tx, recv_rx));
        
        // Add the peer
        self.add_peer(peer).await?;
        
        info!("Initiated connection to peer: {} at {}", peer_id, addr);
        Ok(())
    }
    
    /// Add a new peer
    pub async fn add_peer(&self, peer: Arc<Peer>) -> Result<(), NetworkError> {
        let info = peer.info.read().await;
        let peer_id = info.id.clone();
        let direction = info.direction.clone();
        
        // Check limits
        let stats = self.stats.read().await;
        if stats.total_connected >= self.config.max_peers {
            return Err(NetworkError::ConnectionFailed("Max peers reached".to_string()));
        }
        
        match direction {
            Direction::Inbound if stats.inbound_count >= self.config.max_inbound => {
                return Err(NetworkError::ConnectionFailed("Max inbound peers reached".to_string()));
            }
            Direction::Outbound if stats.outbound_count >= self.config.max_outbound => {
                return Err(NetworkError::ConnectionFailed("Max outbound peers reached".to_string()));
            }
            _ => {}
        }
        drop(stats);
        drop(info);
        
        // Add peer
        self.peers.insert(peer_id.clone(), peer);
        
        // Update stats
        let mut stats = self.stats.write().await;
        stats.total_connected += 1;
        match direction {
            Direction::Inbound => stats.inbound_count += 1,
            Direction::Outbound => stats.outbound_count += 1,
        }
        
        info!("Added peer: {}", peer_id);
        Ok(())
    }
    
    /// Remove a peer
    pub async fn remove_peer(&self, peer_id: &PeerId) -> Option<Arc<Peer>> {
        let peer = self.peers.remove(peer_id).map(|(_, p)| p);
        
        if let Some(ref p) = peer {
            let info = p.info.read().await;
            let mut stats = self.stats.write().await;
            stats.total_connected = stats.total_connected.saturating_sub(1);
            match info.direction {
                Direction::Inbound => stats.inbound_count = stats.inbound_count.saturating_sub(1),
                Direction::Outbound => stats.outbound_count = stats.outbound_count.saturating_sub(1),
            }
            
            info!("Removed peer: {}", peer_id);
        }
        
        peer
    }
    
    /// Get a peer by ID
    pub fn get_peer(&self, peer_id: &PeerId) -> Option<Arc<Peer>> {
        self.peers.get(peer_id).map(|p| p.clone())
    }
    
    /// Get all connected peers
    pub fn get_all_peers(&self) -> Vec<Arc<Peer>> {
        self.peers.iter().map(|p| p.value().clone()).collect()
    }
    
    /// Get peer count by direction
    pub async fn get_peer_counts(&self) -> (usize, usize, usize) {
        let stats = self.stats.read().await;
        (stats.total_connected, stats.inbound_count, stats.outbound_count)
    }
    
    /// Ban a peer
    pub async fn ban_peer(&self, addr: SocketAddr) {
        let mut banned = self.banned_peers.write().await;
        if !banned.contains(&addr) {
            banned.push(addr);
            warn!("Banned peer: {}", addr);
        }
    }
    
    /// Check if an address is banned
    pub async fn is_banned(&self, addr: &SocketAddr) -> bool {
        self.banned_peers.read().await.contains(addr)
    }
    
    /// Update peer score
    pub async fn update_peer_score(&self, peer_id: &PeerId, delta: i32) {
        if let Some(peer) = self.get_peer(peer_id) {
            let mut info = peer.info.write().await;
            info.score += delta;
            
            // Ban if score too low
            if info.score < self.config.score_threshold {
                drop(info);
                self.ban_peer(peer.info.read().await.addr).await;
                self.remove_peer(peer_id).await;
            }
        }
    }
    
    /// Clean up stale peers
    pub async fn cleanup_stale_peers(&self) {
        let stale_peers: Vec<PeerId> = {
            let mut stale = Vec::new();
            for peer in self.peers.iter() {
                let info = peer.value().info.read().await;
                if info.is_stale(self.config.peer_timeout) {
                    stale.push(info.id.clone());
                }
            }
            stale
        };
        
        for peer_id in stale_peers {
            debug!("Removing stale peer: {}", peer_id);
            self.remove_peer(&peer_id).await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_peer_manager_limits() {
        let mut config = PeerManagerConfig::default();
        config.max_peers = 2;
        config.max_inbound = 1;
        config.max_outbound = 1;
        
        let manager = PeerManager::new(config);
        
        // Create mock peers
        let (send_tx1, recv_rx1) = mpsc::channel(10);
        let (send_tx2, recv_rx2) = mpsc::channel(10);
        let (send_tx3, recv_rx3) = mpsc::channel(10);
        
        let peer1 = Arc::new(Peer::new(
            PeerInfo::new(PeerId::random(), "127.0.0.1:8001".parse().unwrap(), Direction::Inbound),
            send_tx1,
            recv_rx1,
        ));
        
        let peer2 = Arc::new(Peer::new(
            PeerInfo::new(PeerId::random(), "127.0.0.1:8002".parse().unwrap(), Direction::Outbound),
            send_tx2,
            recv_rx2,
        ));
        
        let peer3 = Arc::new(Peer::new(
            PeerInfo::new(PeerId::random(), "127.0.0.1:8003".parse().unwrap(), Direction::Inbound),
            send_tx3,
            recv_rx3,
        ));
        
        // Add first two peers - should succeed
        assert!(manager.add_peer(peer1).await.is_ok());
        assert!(manager.add_peer(peer2).await.is_ok());
        
        // Try to add third peer - should fail (max peers reached)
        assert!(manager.add_peer(peer3).await.is_err());
        
        let (total, inbound, outbound) = manager.get_peer_counts().await;
        assert_eq!(total, 2);
        assert_eq!(inbound, 1);
        assert_eq!(outbound, 1);
    }
    
    #[tokio::test]
    async fn test_peer_scoring_and_ban() {
        let mut config = PeerManagerConfig::default();
        config.score_threshold = -10;
        
        let manager = PeerManager::new(config);
        
        let (send_tx, recv_rx) = mpsc::channel(10);
        let peer_id = PeerId::random();
        let addr = "127.0.0.1:8001".parse().unwrap();
        
        let peer = Arc::new(Peer::new(
            PeerInfo::new(peer_id.clone(), addr, Direction::Inbound),
            send_tx,
            recv_rx,
        ));
        
        manager.add_peer(peer).await.unwrap();
        
        // Decrease score below threshold
        manager.update_peer_score(&peer_id, -15).await;
        
        // Peer should be removed and banned
        assert!(manager.get_peer(&peer_id).is_none());
        assert!(manager.is_banned(&addr).await);
    }
}