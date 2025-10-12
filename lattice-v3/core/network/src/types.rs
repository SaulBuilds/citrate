// lattice-v3/core/network/src/types.rs

// Network types and constants
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::time::Duration;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum NetworkError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Protocol error: {0}")]
    ProtocolError(String),

    #[error("Peer not found: {0}")]
    PeerNotFound(String),

    #[error("Sync error: {0}")]
    SyncError(String),

    #[error("Message decode error: {0}")]
    DecodeError(String),

    #[error("Timeout: {0}")]
    Timeout(String),

    #[error("Invalid message: {0}")]
    InvalidMessage(String),

    #[error("Network is shutting down")]
    Shutdown,

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// Address to listen on
    pub listen_addr: SocketAddr,

    /// Bootstrap nodes to connect to initially
    pub bootstrap_nodes: Vec<String>,

    /// Maximum number of peers to maintain
    pub max_peers: usize,

    /// Maximum inbound connections
    pub max_inbound: usize,

    /// Maximum outbound connections
    pub max_outbound: usize,

    /// Enable peer discovery
    pub enable_discovery: bool,

    /// Interval for gossip rounds
    pub gossip_interval: Duration,

    /// Connection timeout
    pub connection_timeout: Duration,

    /// Handshake timeout
    pub handshake_timeout: Duration,

    /// Request timeout
    pub request_timeout: Duration,

    /// Ping interval for keepalive
    pub ping_interval: Duration,

    /// Network ID/chain ID
    pub network_id: u32,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            listen_addr: "0.0.0.0:30303".parse().unwrap(),
            bootstrap_nodes: Vec::new(),
            max_peers: 50,
            max_inbound: 30,
            max_outbound: 20,
            enable_discovery: true,
            gossip_interval: Duration::from_secs(1),
            connection_timeout: Duration::from_secs(10),
            handshake_timeout: Duration::from_secs(5),
            request_timeout: Duration::from_secs(30),
            ping_interval: Duration::from_secs(30),
            network_id: 1,
        }
    }
}

/// Network statistics
#[derive(Debug, Clone, Default)]
pub struct NetworkStats {
    pub peers_connected: usize,
    pub peers_inbound: usize,
    pub peers_outbound: usize,
    pub messages_sent: u64,
    pub messages_received: u64,
    pub bytes_sent: u64,
    pub bytes_received: u64,
}
