pub mod protocol;
pub mod peer;
pub mod discovery;
pub mod sync;
pub mod gossip;
pub mod types;
pub mod ai_handler;
pub mod block_propagation;
pub mod transaction_gossip;

pub use protocol::{NetworkMessage, Protocol, ProtocolVersion, ModelMetadata};
pub use peer::{Peer, PeerManager, PeerId, PeerInfo, PeerManagerConfig};
pub use discovery::{Discovery, DiscoveryConfig};
pub use sync::{SyncManager, SyncState};
pub use gossip::{GossipProtocol, GossipConfig};
pub use types::{NetworkConfig, NetworkError};
pub use ai_handler::AINetworkHandler;
pub use block_propagation::BlockPropagation;
pub use transaction_gossip::{TransactionGossip, GossipConfig as TxGossipConfig};