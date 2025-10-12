use lattice_consensus::types::{Block, BlockHeader, Hash, Transaction};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Model metadata for AI network messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelMetadata {
    pub name: String,
    pub version: String,
    pub description: String,
    pub framework: String,
    pub input_shape: Vec<usize>,
    pub output_shape: Vec<usize>,
    pub size_bytes: u64,
    pub created_at: u64,
}

/// Protocol version for compatibility checking
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProtocolVersion {
    pub major: u16,
    pub minor: u16,
    pub patch: u16,
}

impl ProtocolVersion {
    pub const CURRENT: Self = Self {
        major: 1,
        minor: 0,
        patch: 0,
    };

    pub fn is_compatible(&self, other: &Self) -> bool {
        // Major version must match, minor/patch can differ
        self.major == other.major
    }
}

impl fmt::Display for ProtocolVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

/// Network message types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(clippy::large_enum_variant)]
pub enum NetworkMessage {
    // Handshake messages
    Hello {
        version: ProtocolVersion,
        network_id: u32,
        genesis_hash: Hash,
        head_height: u64,
        head_hash: Hash,
        peer_id: String,
    },

    HelloAck {
        version: ProtocolVersion,
        head_height: u64,
        head_hash: Hash,
    },

    Disconnect {
        reason: String,
    },

    // Ping/Pong for keepalive
    Ping {
        nonce: u64,
    },

    Pong {
        nonce: u64,
    },

    // Block messages
    NewBlock {
        block: Block,
    },

    GetBlocks {
        from: Hash,
        count: u32,
        step: u32, // For sparse download
    },

    Blocks {
        blocks: Vec<Block>,
    },

    GetHeaders {
        from: Hash,
        count: u32,
    },

    Headers {
        headers: Vec<BlockHeader>,
    },

    // Transaction messages
    NewTransaction {
        transaction: Transaction,
    },

    GetTransactions {
        hashes: Vec<Hash>,
    },

    Transactions {
        transactions: Vec<Transaction>,
    },

    // AI-specific messages for model and inference data

    // Model registration and updates
    ModelAnnounce {
        model_id: Hash,
        model_hash: Hash,
        owner: Vec<u8>, // Address bytes
        metadata: ModelMetadata,
        weight_cid: String, // IPFS CID for weights
    },

    GetModel {
        model_id: Hash,
    },

    ModelData {
        model_id: Hash,
        weight_cid: String,
        metadata: ModelMetadata,
    },

    // Inference requests and results
    InferenceRequest {
        request_id: Hash,
        model_id: Hash,
        input_hash: Hash,
        requester: Vec<u8>,
        max_fee: u128,
    },

    InferenceResponse {
        request_id: Hash,
        output_hash: Hash,
        proof: Vec<u8>, // ZK proof of computation
        provider: Vec<u8>,
    },

    // Training coordination
    TrainingJobAnnounce {
        job_id: Hash,
        model_id: Hash,
        dataset_hash: Hash,
        participants_needed: u32,
        reward_per_gradient: u128,
    },

    GradientSubmission {
        job_id: Hash,
        gradient_hash: Hash,
        epoch: u32,
        participant: Vec<u8>,
    },

    // LoRA adapter sharing
    LoraAdapterAnnounce {
        adapter_id: Hash,
        base_model: Hash,
        weight_cid: String,
        rank: u32,
        alpha: f32,
    },

    GetLoraAdapter {
        adapter_id: Hash,
    },

    // Model weight synchronization
    WeightSync {
        model_id: Hash,
        version: u32,
        weight_delta: Vec<u8>, // Compressed weight update
    },

    // AI state synchronization
    GetAIState {
        from_height: u64,
    },

    AIStateUpdate {
        height: u64,
        models_root: Hash,
        training_root: Hash,
        inference_root: Hash,
        lora_root: Hash,
    },

    GetMempool,

    Mempool {
        tx_hashes: Vec<Hash>,
    },

    // Sync messages
    GetBlocksByHeight {
        from_height: u64,
        count: u32,
    },

    GetState {
        root: Hash,
        keys: Vec<Vec<u8>>,
    },

    StateData {
        root: Hash,
        data: Vec<(Vec<u8>, Vec<u8>)>,
    },

    // Discovery messages
    GetPeers,

    Peers {
        peers: Vec<PeerAddress>,
    },

    // Consensus messages (GhostDAG specific)
    GetBlueSet {
        block: Hash,
    },

    BlueSet {
        block: Hash,
        blue_blocks: Vec<Hash>,
        blue_score: u64,
    },

    GetDagInfo {
        blocks: Vec<Hash>,
    },

    DagInfo {
        info: Vec<DagBlockInfo>,
    },
}

/// Peer address information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerAddress {
    pub id: String,
    pub addr: String,
    pub last_seen: u64,
    pub score: i32,
}

/// DAG block information for GhostDAG
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DagBlockInfo {
    pub hash: Hash,
    pub selected_parent: Hash,
    pub merge_parents: Vec<Hash>,
    pub blue_score: u64,
    pub is_blue: bool,
}

/// Protocol handler trait
#[async_trait::async_trait]
pub trait Protocol: Send + Sync {
    /// Handle incoming message
    async fn handle_message(
        &self,
        peer_id: &str,
        message: NetworkMessage,
    ) -> Result<Option<NetworkMessage>, crate::NetworkError>;

    /// Called when a new peer connects
    async fn on_peer_connected(&self, peer_id: &str);

    /// Called when a peer disconnects
    async fn on_peer_disconnected(&self, peer_id: &str);
}

/// Message priority for queue management
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MessagePriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

impl NetworkMessage {
    /// Get the priority of this message
    pub fn priority(&self) -> MessagePriority {
        match self {
            // Critical priority for handshake and sync
            Self::Hello { .. } | Self::HelloAck { .. } => MessagePriority::Critical,
            Self::GetBlocks { .. } | Self::GetHeaders { .. } => MessagePriority::Critical,

            // High priority for new blocks
            Self::NewBlock { .. } => MessagePriority::High,

            // Normal priority for transactions and general messages
            Self::NewTransaction { .. } => MessagePriority::Normal,
            Self::GetTransactions { .. } | Self::Transactions { .. } => MessagePriority::Normal,

            // Low priority for discovery and stats
            Self::GetPeers | Self::Peers { .. } => MessagePriority::Low,
            Self::Ping { .. } | Self::Pong { .. } => MessagePriority::Low,

            _ => MessagePriority::Normal,
        }
    }

    /// Check if this message requires a response
    pub fn requires_response(&self) -> bool {
        matches!(
            self,
            Self::Ping { .. }
                | Self::GetBlocks { .. }
                | Self::GetHeaders { .. }
                | Self::GetTransactions { .. }
                | Self::GetMempool
                | Self::GetPeers
                | Self::GetBlueSet { .. }
                | Self::GetDagInfo { .. }
                | Self::GetState { .. }
                | Self::GetBlocksByHeight { .. }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_protocol_version_compatibility() {
        let v1 = ProtocolVersion {
            major: 1,
            minor: 0,
            patch: 0,
        };
        let v2 = ProtocolVersion {
            major: 1,
            minor: 1,
            patch: 0,
        };
        let v3 = ProtocolVersion {
            major: 2,
            minor: 0,
            patch: 0,
        };

        assert!(v1.is_compatible(&v2));
        assert!(!v1.is_compatible(&v3));
    }

    #[test]
    fn test_message_priority() {
        let hello = NetworkMessage::Hello {
            version: ProtocolVersion::CURRENT,
            network_id: 1,
            genesis_hash: Hash::default(),
            head_height: 0,
            head_hash: Hash::default(),
            peer_id: "test".to_string(),
        };

        assert_eq!(hello.priority(), MessagePriority::Critical);

        // Create a test transaction
        let new_tx = NetworkMessage::NewTransaction {
            transaction: Transaction {
                hash: Hash::default(),
                nonce: 0,
                from: lattice_consensus::types::PublicKey::new([0; 32]),
                to: None,
                value: 0,
                gas_limit: 21000,
                gas_price: 1000000000,
                data: vec![],
                signature: lattice_consensus::types::Signature::new([0; 64]),
                tx_type: None,
            },
        };

        assert_eq!(new_tx.priority(), MessagePriority::Normal);

        // GetBlocks should be critical
        let get_blocks = NetworkMessage::GetBlocks {
            from: Hash::default(),
            count: 10,
            step: 1,
        };
        assert_eq!(get_blocks.priority(), MessagePriority::Critical);

        // NewBlock should be high
        let nb = NetworkMessage::NewBlock {
            block: Block {
                header: BlockHeader {
                    version: 1,
                    block_hash: Hash::default(),
                    selected_parent_hash: Hash::default(),
                    merge_parent_hashes: vec![],
                    timestamp: 0,
                    height: 0,
                    blue_score: 0,
                    blue_work: 0,
                    pruning_point: Hash::default(),
                    proposer_pubkey: lattice_consensus::types::PublicKey::new([0; 32]),
                    vrf_reveal: lattice_consensus::types::VrfProof {
                        proof: vec![],
                        output: Hash::default(),
                    },
                },
                state_root: Hash::default(),
                tx_root: Hash::default(),
                receipt_root: Hash::default(),
                artifact_root: Hash::default(),
                ghostdag_params: Default::default(),
                transactions: vec![],
                signature: lattice_consensus::types::Signature::new([0; 64]),
            },
        };
        assert_eq!(nb.priority(), MessagePriority::High);

        // GetPeers is low
        assert_eq!(NetworkMessage::GetPeers.priority(), MessagePriority::Low);
    }

    #[test]
    fn test_message_requires_response() {
        let ping = NetworkMessage::Ping { nonce: 42 };
        assert!(ping.requires_response());

        // Test a message that doesn't require response
        let pong = NetworkMessage::Pong { nonce: 42 };
        assert!(!pong.requires_response());
    }
}
