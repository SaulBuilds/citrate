// citrate/core/network/src/transport.rs

use crate::peer::{Direction, Peer, PeerId, PeerInfo, PeerManager};
use crate::protocol::{NetworkMessage, ProtocolVersion};
use crate::NetworkError;
use bincode;
use bytes::BytesMut;
use futures::{SinkExt, StreamExt};
use citrate_consensus::types::Hash;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;
use tokio_util::codec::{Framed, LengthDelimitedCodec};
use tracing::{debug, error, info, warn};

/// Parameters sent during handshake
#[derive(Debug, Clone)]
pub struct HandshakeParams {
    pub network_id: u32,
    pub genesis_hash: Hash,
    pub head_height: u64,
    pub head_hash: Hash,
}

/// Simple TCP-based transport with length-delimited frames (bincode payloads)
pub struct NetworkTransport {
    peer_manager: Arc<PeerManager>,
    local_id: PeerId,
    params: HandshakeParams,
}

const MAX_FRAME_LEN: usize = 1024 * 1024; // 1MB

impl NetworkTransport {
    pub fn new(peer_manager: Arc<PeerManager>, local_id: PeerId, params: HandshakeParams) -> Self {
        Self { peer_manager, local_id, params }
    }

    /// Start an async TCP listener and accept inbound peers
    pub async fn start_listener(&self, addr: SocketAddr) -> Result<(), NetworkError> {
        let listener = TcpListener::bind(addr)
            .await
            .map_err(|e| NetworkError::TransportError(format!("bind {}: {}", addr, e)))?;
        info!("P2P listener on {}", addr);

        let pm = self.peer_manager.clone();
        let local_id = self.local_id.clone();
        let params = self.params.clone();

        tokio::spawn(async move {
            loop {
                match listener.accept().await {
                    Ok((stream, remote)) => {
                        let pm = pm.clone();
                        let local_id = local_id.clone();
                        let params = params.clone();
                        tokio::spawn(async move {
                            if let Err(e) = handle_inbound(stream, remote, pm, local_id, params).await {
                                warn!("inbound error from {}: {}", remote, e);
                            }
                        });
                    }
                    Err(e) => {
                        warn!("listener accept error: {}", e);
                    }
                }
            }
        });

        Ok(())
    }

    /// Dial an outbound peer
    pub async fn connect_to(&self, addr: SocketAddr) -> Result<(), NetworkError> {
        let stream = TcpStream::connect(addr)
            .await
            .map_err(|e| NetworkError::TransportError(format!("connect {}: {}", addr, e)))?;
        let pm = self.peer_manager.clone();
        let local_id = self.local_id.clone();
        let params = self.params.clone();
        tokio::spawn(async move {
            if let Err(e) = handle_outbound(stream, addr, pm, local_id, params).await {
                warn!("outbound error to {}: {}", addr, e);
            }
        });
        Ok(())
    }
}

async fn handle_inbound(
    stream: TcpStream,
    addr: SocketAddr,
    peer_manager: Arc<PeerManager>,
    local_id: PeerId,
    params: HandshakeParams,
) -> Result<(), NetworkError> {
    let codec = LengthDelimitedCodec::builder()
        .max_frame_length(MAX_FRAME_LEN)
        .new_length_delimited();
    let framed = Framed::new(stream, codec);
    // Expect Hello from remote
    let (mut sink, mut stream) = framed.split();
    let hello = match stream.next().await {
        Some(Ok(bytes)) => bincode::deserialize::<NetworkMessage>(&bytes)
            .map_err(|e| NetworkError::ProtocolError(format!("decode: {}", e)))?,
        Some(Err(e)) => return Err(NetworkError::TransportError(format!("read: {}", e))),
        None => return Err(NetworkError::TransportError("eof".into())),
    };
    let (remote_id, remote_head_height, remote_head_hash) = match hello {
        NetworkMessage::Hello {
            version,
            network_id,
            genesis_hash,
            head_height,
            head_hash,
            peer_id,
        } => {
            if !version.is_compatible(&ProtocolVersion::CURRENT) {
                return Err(NetworkError::ProtocolError("incompatible version".into()));
            }
            if network_id != params.network_id || genesis_hash != params.genesis_hash {
                return Err(NetworkError::ProtocolError("network mismatch".into()));
            }
            (PeerId::new(peer_id), head_height, head_hash)
        }
        _ => return Err(NetworkError::ProtocolError("expected Hello".into())),
    };

    // Send HelloAck
    let ack = NetworkMessage::HelloAck {
        version: ProtocolVersion::CURRENT,
        head_height: params.head_height,
        head_hash: params.head_hash,
        peer_id: local_id.0.clone(),
    };
    {
        let ser = bincode::serialize(&ack)
            .map_err(|e| NetworkError::ProtocolError(format!("encode: {}", e)))?;
        sink.send(bytes::Bytes::from(ser))
            .await
            .map_err(|e| NetworkError::TransportError(format!("write: {}", e)))?;
    }

    // Create peer channels
    let (to_wire_tx, mut to_wire_rx) = mpsc::channel::<NetworkMessage>(256);
    let (_from_wire_tx, from_wire_rx) = mpsc::channel::<NetworkMessage>(256);

    let mut info = PeerInfo::new(remote_id.clone(), addr, Direction::Inbound);
    info.state = super::peer::PeerState::Connected;
    info.head_height = remote_head_height;
    info.head_hash = remote_head_hash;
    let peer = Arc::new(Peer::new(info, to_wire_tx.clone(), from_wire_rx));
    peer_manager.add_peer(peer.clone()).await?;
    info!("Inbound peer connected: {} from {}", remote_id, addr);

    // Writer: forward messages from send queue to wire
    tokio::spawn(async move {
        while let Some(msg) = to_wire_rx.recv().await {
            match bincode::serialize(&msg) {
                Ok(ser) => {
                    if let Err(e) = sink.send(bytes::Bytes::from(ser)).await {
                        warn!("send to {} failed: {}", addr, e);
                        break;
                    }
                }
                Err(e) => {
                    warn!("encode failed: {}", e);
                    break;
                }
            }
        }
    });

    // Reader loop with simple rate limit
    let mut msg_count = 0u32;
    let mut window_start = std::time::Instant::now();
    const MAX_MSGS_PER_SEC: u32 = 200;
    while let Some(frame) = stream.next().await {
        if window_start.elapsed() > std::time::Duration::from_secs(1) {
            window_start = std::time::Instant::now();
            msg_count = 0;
        }
        msg_count += 1;
        if msg_count > MAX_MSGS_PER_SEC {
            warn!("rate limit exceeded from {} — closing", addr);
            peer_manager.remove_peer(&remote_id).await;
            break;
        }
        match frame {
            Ok(bytes) => match bincode::deserialize::<NetworkMessage>(&bytes) {
                Ok(msg) => {
                    peer_manager
                        .forward_incoming(remote_id.clone(), msg)
                        .await;
                }
                Err(e) => {
                    warn!("decode failed from {}: {}", addr, e);
                    break;
                }
            },
            Err(e) => {
                debug!("peer {} closed: {}", remote_id, e);
                peer_manager.remove_peer(&remote_id).await;
                break;
            }
        }
    }
    Ok(())
}

async fn handle_outbound(
    stream: TcpStream,
    addr: SocketAddr,
    peer_manager: Arc<PeerManager>,
    local_id: PeerId,
    params: HandshakeParams,
) -> Result<(), NetworkError> {
    let codec = LengthDelimitedCodec::builder()
        .max_frame_length(MAX_FRAME_LEN)
        .new_length_delimited();
    let framed = Framed::new(stream, codec);
    // Send Hello
    let hello = NetworkMessage::Hello {
        version: ProtocolVersion::CURRENT,
        network_id: params.network_id,
        genesis_hash: params.genesis_hash,
        head_height: params.head_height,
        head_hash: params.head_hash,
        peer_id: local_id.0.clone(),
    };
    let (mut sink, mut stream) = framed.split();
    {
        let ser = bincode::serialize(&hello)
            .map_err(|e| NetworkError::ProtocolError(format!("encode: {}", e)))?;
        sink.send(bytes::Bytes::from(ser))
            .await
            .map_err(|e| NetworkError::TransportError(format!("write: {}", e)))?;
    }

    // Expect HelloAck
    let ack = match stream.next().await {
        Some(Ok(bytes)) => bincode::deserialize::<NetworkMessage>(&bytes)
            .map_err(|e| NetworkError::ProtocolError(format!("decode: {}", e)))?,
        Some(Err(e)) => return Err(NetworkError::TransportError(format!("read: {}", e))),
        None => return Err(NetworkError::TransportError("eof".into())),
    };
    if let NetworkMessage::HelloAck { version, peer_id, head_height, head_hash } = ack {
        if !version.is_compatible(&ProtocolVersion::CURRENT) {
            return Err(NetworkError::ProtocolError("incompatible ack".into()));
        }
        let rid = if peer_id.is_empty() {
            format!("tcp_{}", addr)
        } else {
            peer_id
        };
        let remote_id = PeerId::new(rid);
        // Create peer channels
        let (to_wire_tx, mut to_wire_rx) = mpsc::channel::<NetworkMessage>(256);
        let (_from_wire_tx, from_wire_rx) = mpsc::channel::<NetworkMessage>(256);
        let mut info = PeerInfo::new(remote_id.clone(), addr, Direction::Outbound);
        info.state = super::peer::PeerState::Connected;
        info.head_height = head_height;
        info.head_hash = head_hash;
        let peer = Arc::new(Peer::new(info, to_wire_tx.clone(), from_wire_rx));
        peer_manager.add_peer(peer.clone()).await?;
        info!("Outbound peer connected: {} at {}", remote_id, addr);

        // Writer task
        tokio::spawn(async move {
            while let Some(msg) = to_wire_rx.recv().await {
                match bincode::serialize(&msg) {
                    Ok(ser) => {
                        if let Err(e) = sink.send(bytes::Bytes::from(ser)).await {
                            warn!("send to {} failed: {}", addr, e);
                            break;
                        }
                    }
                    Err(e) => {
                        warn!("encode failed: {}", e);
                        break;
                    }
                }
            }
        });

        // Reader loop with rate limiting
        let mut msg_count = 0u32;
        let mut window_start = std::time::Instant::now();
        const MAX_MSGS_PER_SEC: u32 = 200;
        while let Some(frame) = stream.next().await {
            if window_start.elapsed() > std::time::Duration::from_secs(1) {
                window_start = std::time::Instant::now();
                msg_count = 0;
            }
            msg_count += 1;
            if msg_count > MAX_MSGS_PER_SEC {
                warn!("rate limit exceeded from {} — closing", addr);
                peer_manager.remove_peer(&remote_id).await;
                break;
            }
            match frame {
                Ok(bytes) => match bincode::deserialize::<NetworkMessage>(&bytes) {
                    Ok(msg) => {
                        peer_manager
                            .forward_incoming(remote_id.clone(), msg)
                            .await;
                    }
                    Err(e) => {
                        warn!("decode failed from {}: {}", addr, e);
                        break;
                    }
                },
                Err(e) => {
                    debug!("peer {} closed: {}", remote_id, e);
                    peer_manager.remove_peer(&remote_id).await;
                    break;
                }
            }
        }
        Ok(())
    } else {
        Err(NetworkError::ProtocolError("expected HelloAck".into()))
    }
}

// helper functions removed in favor of split-based loops
