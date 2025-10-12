// lattice-v3/core/network/src/peer.rs

// Peer connection and management
use crate::{NetworkError, NetworkMessage, ProtocolVersion};
use dashmap::DashMap;
use futures::{SinkExt, StreamExt};
use lattice_consensus::types::Hash;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, RwLock};
use tokio_util::codec::{Framed, LengthDelimitedCodec};
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
type IncomingTx = mpsc::Sender<(PeerId, NetworkMessage)>;

pub struct PeerManager {
    config: PeerManagerConfig,
    peers: Arc<DashMap<PeerId, Arc<Peer>>>,
    banned_peers: Arc<RwLock<Vec<SocketAddr>>>,
    stats: Arc<RwLock<PeerStats>>,
    incoming: Arc<RwLock<Option<IncomingTx>>>,
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
            incoming: Arc::new(RwLock::new(None)),
        }
    }

    /// Set incoming message sink
    pub async fn set_incoming(&self, tx: mpsc::Sender<(PeerId, NetworkMessage)>) {
        *self.incoming.write().await = Some(tx);
    }

    /// Get max peers configuration
    pub fn max_peers(&self) -> usize {
        self.config.max_peers
    }

    /// Connect to a peer
    pub async fn connect_to_peer(
        &self,
        peer_id: PeerId,
        addr: SocketAddr,
    ) -> Result<(), NetworkError> {
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
            return Err(NetworkError::ConnectionFailed(
                "Max peers reached".to_string(),
            ));
        }

        match direction {
            Direction::Inbound if stats.inbound_count >= self.config.max_inbound => {
                return Err(NetworkError::ConnectionFailed(
                    "Max inbound peers reached".to_string(),
                ));
            }
            Direction::Outbound if stats.outbound_count >= self.config.max_outbound => {
                return Err(NetworkError::ConnectionFailed(
                    "Max outbound peers reached".to_string(),
                ));
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
                Direction::Outbound => {
                    stats.outbound_count = stats.outbound_count.saturating_sub(1)
                }
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
        (
            stats.total_connected,
            stats.inbound_count,
            stats.outbound_count,
        )
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

    /// Broadcast a message to all connected peers
    pub async fn broadcast(&self, message: &NetworkMessage) -> Result<(), NetworkError> {
        let peers = self.get_all_peers();
        let mut send_count = 0;

        for peer in peers {
            if (peer.send(message.clone()).await).is_ok() {
                send_count += 1;
            }
        }

        debug!("Broadcasted message to {} peers", send_count);
        Ok(())
    }

    /// Send message to specific peers
    pub async fn send_to_peers(
        &self,
        peer_ids: &[PeerId],
        message: &NetworkMessage,
    ) -> Result<(), NetworkError> {
        for peer_id in peer_ids {
            if let Some(peer) = self.get_peer(peer_id) {
                peer.send(message.clone()).await?;
            }
        }
        Ok(())
    }

    /// Start a TCP listener for inbound peer connections
    pub async fn start_listener(
        self: &Arc<Self>,
        listen_addr: SocketAddr,
        network_id: u32,
        genesis_hash: Hash,
        head_height: u64,
        head_hash: Hash,
    ) -> Result<(), NetworkError> {
        let listener = TcpListener::bind(listen_addr)
            .await
            .map_err(NetworkError::Io)?;
        let this = self.clone();
        tokio::spawn(async move {
            loop {
                match listener.accept().await {
                    Ok((stream, addr)) => {
                        let pm = this.clone();
                        let g = genesis_hash;
                        tokio::spawn(async move {
                            if let Err(e) = handle_incoming(
                                stream,
                                addr,
                                pm,
                                network_id,
                                g,
                                head_height,
                                head_hash,
                            )
                            .await
                            {
                                warn!("Inbound connection error from {}: {}", addr, e);
                            }
                        });
                    }
                    Err(e) => {
                        warn!("Accept error: {}", e);
                        break;
                    }
                }
            }
        });
        info!("Listening for peers on {}", listen_addr);
        Ok(())
    }

    /// Dial a remote peer and perform handshake
    pub async fn connect_bootnode_real(
        self: Arc<Self>,
        peer_id_hint: Option<PeerId>,
        addr: SocketAddr,
        network_id: u32,
        genesis_hash: Hash,
        head_height: u64,
        head_hash: Hash,
    ) -> Result<(), NetworkError> {
        let stream = TcpStream::connect(addr).await.map_err(NetworkError::Io)?;
        let peer_id = peer_id_hint.unwrap_or_else(PeerId::random);
        perform_handshake_outbound(
            self,
            stream,
            addr,
            peer_id,
            network_id,
            genesis_hash,
            head_height,
            head_hash,
        )
        .await
    }
}

async fn handle_incoming(
    stream: TcpStream,
    addr: SocketAddr,
    pm: Arc<PeerManager>,
    network_id: u32,
    _genesis_hash: Hash,
    head_height: u64,
    head_hash: Hash,
) -> Result<(), NetworkError> {
    let mut framed = Framed::new(stream, LengthDelimitedCodec::new());
    // Expect Hello
    let bytes = framed
        .next()
        .await
        .ok_or_else(|| NetworkError::ProtocolError("EOF before hello".into()))
        .map_err(|_| NetworkError::ProtocolError("Stream closed".into()))??;
    let hello: NetworkMessage = bincode::deserialize(&bytes)
        .map_err(|e| NetworkError::DecodeError(format!("handshake decode: {}", e)))?;
    let (peer_id_str, ver, net_ok) = match hello {
        NetworkMessage::Hello {
            version,
            network_id: nid,
            peer_id,
            ..
        } => (peer_id, version, nid == network_id),
        _ => return Err(NetworkError::ProtocolError("Expected Hello".into())),
    };
    if !ver.is_compatible(&ProtocolVersion::CURRENT) || !net_ok {
        // send disconnect
        let _ = send_msg(
            &mut framed,
            &NetworkMessage::Disconnect {
                reason: "incompatible".into(),
            },
        )
        .await;
        return Err(NetworkError::ProtocolError(
            "incompatible version or network".into(),
        ));
    }
    // Register peer
    let peer_id = PeerId::new(peer_id_str);
    // Channels for app-level messaging
    let (send_tx, mut send_rx) = mpsc::channel(256);
    let (recv_tx_app, recv_rx) = mpsc::channel(256);
    let info = PeerInfo::new(peer_id.clone(), addr, Direction::Inbound);
    let peer = Arc::new(Peer::new(info, send_tx.clone(), recv_rx));
    pm.add_peer(peer.clone()).await?;
    // Reply HelloAck
    let ack = NetworkMessage::HelloAck {
        version: ProtocolVersion::CURRENT,
        head_height,
        head_hash,
    };
    send_msg(&mut framed, &ack).await?;
    // Split framed into sink and stream
    let (mut sink, mut stream) = framed.split();
    let writer = tokio::spawn(async move {
        while let Some(msg) = send_rx.recv().await {
            if send_msg_sink(&mut sink, &msg).await.is_err() {
                break;
            }
        }
    });
    // Reader
    while let Some(frame) = stream.next().await {
        let bytes = match frame {
            Ok(b) => b,
            Err(_) => break,
        };
        if let Ok(msg) = bincode::deserialize::<NetworkMessage>(&bytes) {
            // Basic responses
            match msg {
                NetworkMessage::Ping { nonce } => {
                    let _ = send_tx.send(NetworkMessage::Pong { nonce }).await;
                }
                other => {
                    // publish to global incoming
                    if let Some(tx) = pm.incoming.read().await.clone() {
                        let _ = tx.send((peer_id.clone(), other.clone())).await;
                    }
                    let _ = recv_tx_app.send(other).await;
                }
            }
            let mut inf = peer.info.write().await;
            inf.messages_received += 1;
            inf.update_last_seen();
        } else {
            break;
        }
    }
    writer.abort();
    pm.remove_peer(&peer_id).await;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn perform_handshake_outbound(
    pm: Arc<PeerManager>,
    stream: TcpStream,
    addr: SocketAddr,
    peer_id: PeerId,
    network_id: u32,
    genesis_hash: Hash,
    head_height: u64,
    head_hash: Hash,
) -> Result<(), NetworkError> {
    let mut framed = Framed::new(stream, LengthDelimitedCodec::new());
    // Send Hello
    let hello = NetworkMessage::Hello {
        version: ProtocolVersion::CURRENT,
        network_id,
        genesis_hash,
        head_height,
        head_hash,
        peer_id: peer_id.0.clone(),
    };
    send_msg(&mut framed, &hello).await?;
    // Expect Ack
    let bytes = framed
        .next()
        .await
        .ok_or_else(|| NetworkError::ProtocolError("EOF before ack".into()))
        .map_err(|_| NetworkError::ProtocolError("Stream closed".into()))??;
    let ack: NetworkMessage = bincode::deserialize(&bytes)
        .map_err(|e| NetworkError::DecodeError(format!("ack decode: {}", e)))?;
    match ack {
        NetworkMessage::HelloAck { version, .. }
            if version.is_compatible(&ProtocolVersion::CURRENT) => {}
        _ => {
            return Err(NetworkError::ProtocolError("invalid ack".into()));
        }
    }
    // Register peer and spawn IO
    let (send_tx, mut send_rx) = mpsc::channel(256);
    let (_recv_tx, recv_rx) = mpsc::channel(256);
    let info = PeerInfo::new(peer_id.clone(), addr, Direction::Outbound);
    let peer = Arc::new(Peer::new(info, send_tx.clone(), recv_rx));
    pm.add_peer(peer.clone()).await?;
    let (mut sink, mut stream) = framed.split();
    let writer = tokio::spawn(async move {
        while let Some(msg) = send_rx.recv().await {
            if let Err(_e) = send_msg_sink(&mut sink, &msg).await {
                break;
            }
        }
    });
    let pm2 = pm.clone();
    tokio::spawn(async move {
        while let Some(frame) = stream.next().await {
            if let Ok(bytes) = frame {
                if let Ok(msg) = bincode::deserialize::<NetworkMessage>(&bytes) {
                    if let Some(tx) = pm2.incoming.read().await.clone() {
                        let _ = tx.send((peer_id.clone(), msg)).await;
                    }
                }
            } else {
                break;
            }
        }
        writer.abort();
    });
    info!("Connected to bootnode {}", addr);
    Ok(())
}

async fn send_msg(
    framed: &mut Framed<TcpStream, LengthDelimitedCodec>,
    msg: &NetworkMessage,
) -> Result<(), NetworkError> {
    let bytes = bincode::serialize(msg).map_err(|e| NetworkError::DecodeError(e.to_string()))?;
    framed.send(bytes.into()).await.map_err(NetworkError::Io)
}

async fn send_msg_sink<S>(sink: &mut S, msg: &NetworkMessage) -> Result<(), NetworkError>
where
    S: futures::Sink<bytes::Bytes, Error = std::io::Error> + Unpin,
{
    let bytes = bincode::serialize(msg).map_err(|e| NetworkError::DecodeError(e.to_string()))?;
    sink.send(bytes.into()).await.map_err(NetworkError::Io)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_peer_manager_limits() {
        let config = PeerManagerConfig {
            max_peers: 2,
            max_inbound: 1,
            max_outbound: 1,
            ..Default::default()
        };

        let manager = PeerManager::new(config);

        // Create mock peers
        let (send_tx1, recv_rx1) = mpsc::channel(10);
        let (send_tx2, recv_rx2) = mpsc::channel(10);
        let (send_tx3, recv_rx3) = mpsc::channel(10);

        let peer1 = Arc::new(Peer::new(
            PeerInfo::new(
                PeerId::random(),
                "127.0.0.1:8001".parse().unwrap(),
                Direction::Inbound,
            ),
            send_tx1,
            recv_rx1,
        ));

        let peer2 = Arc::new(Peer::new(
            PeerInfo::new(
                PeerId::random(),
                "127.0.0.1:8002".parse().unwrap(),
                Direction::Outbound,
            ),
            send_tx2,
            recv_rx2,
        ));

        let peer3 = Arc::new(Peer::new(
            PeerInfo::new(
                PeerId::random(),
                "127.0.0.1:8003".parse().unwrap(),
                Direction::Inbound,
            ),
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
        let config = PeerManagerConfig {
            score_threshold: -10,
            ..Default::default()
        };

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
