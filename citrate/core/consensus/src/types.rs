// citrate/core/consensus/src/types.rs

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fmt;

/// Hash type for block and transaction identifiers
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default, PartialOrd, Ord,
)]
pub struct Hash([u8; 32]);

impl Hash {
    pub fn new(data: [u8; 32]) -> Self {
        Self(data)
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&bytes[..32]);
        Self(hash)
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    pub fn to_hex(&self) -> String {
        hex::encode(self.0)
    }
}

impl fmt::Display for Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", &self.to_hex()[..8])
    }
}

/// Public key type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PublicKey(pub [u8; 32]);

impl PublicKey {
    pub fn new(data: [u8; 32]) -> Self {
        Self(data)
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

/// Signature type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Signature([u8; 64]);

impl Signature {
    pub fn new(data: [u8; 64]) -> Self {
        Self(data)
    }

    pub fn as_bytes(&self) -> &[u8; 64] {
        &self.0
    }
}

impl Default for Signature {
    fn default() -> Self {
        Self([0u8; 64])
    }
}

impl Serialize for Signature {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_bytes(&self.0)
    }
}

impl<'de> Deserialize<'de> for Signature {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let bytes = <Vec<u8>>::deserialize(deserializer)?;
        if bytes.len() != 64 {
            return Err(serde::de::Error::custom("Invalid signature length"));
        }
        let mut data = [0u8; 64];
        data.copy_from_slice(&bytes);
        Ok(Signature(data))
    }
}

/// VRF proof for proposer selection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VrfProof {
    pub proof: Vec<u8>,
    pub output: Hash,
}

/// GhostDAG consensus parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GhostDagParams {
    /// K-cluster parameter for blue set calculation
    pub k: u32,

    /// Maximum number of parents a block can have
    pub max_parents: usize,

    /// Maximum allowed blue score difference for reorg
    pub max_blue_score_diff: u64,

    /// Pruning window size
    pub pruning_window: u64,

    /// Finality depth
    pub finality_depth: u64,
}

impl Default for GhostDagParams {
    fn default() -> Self {
        Self {
            k: 18,           // Standard k-cluster parameter
            max_parents: 10, // Maximum 10 parents per block
            max_blue_score_diff: 1000,
            pruning_window: 100000,
            finality_depth: 100,
        }
    }
}

/// Block header containing consensus-critical fields
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockHeader {
    pub version: u32,
    pub block_hash: Hash,
    pub selected_parent_hash: Hash,
    pub merge_parent_hashes: Vec<Hash>,
    pub timestamp: u64,
    pub height: u64,
    pub blue_score: u64,
    pub blue_work: u128,
    pub pruning_point: Hash,
    pub proposer_pubkey: PublicKey,
    pub vrf_reveal: VrfProof,
}

/// Full block structure as specified in CLAUDE.md
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub header: BlockHeader,
    pub state_root: Hash,
    pub tx_root: Hash,
    pub receipt_root: Hash,
    pub artifact_root: Hash,
    pub ghostdag_params: GhostDagParams,
    pub transactions: Vec<Transaction>,
    pub signature: Signature,

    /// AI models embedded in genesis block only (empty for all other blocks)
    #[serde(default)]
    pub embedded_models: Vec<EmbeddedModel>,

    /// Required model pins (genesis block only, empty for all other blocks)
    #[serde(default)]
    pub required_pins: Vec<RequiredModel>,
}

impl Block {
    /// Get the block hash
    pub fn hash(&self) -> Hash {
        self.header.block_hash
    }

    /// Get selected parent
    pub fn selected_parent(&self) -> Hash {
        self.header.selected_parent_hash
    }

    /// Get all parent hashes (selected + merge)
    pub fn parents(&self) -> Vec<Hash> {
        let mut parents = vec![self.header.selected_parent_hash];
        parents.extend(self.header.merge_parent_hashes.clone());
        parents
    }

    /// Get blue score
    pub fn blue_score(&self) -> u64 {
        self.header.blue_score
    }

    /// Check if this is a genesis block
    pub fn is_genesis(&self) -> bool {
        self.header.selected_parent_hash == Hash::default()
            && self.header.merge_parent_hashes.is_empty()
    }
}

/// AI Transaction Types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransactionType {
    Standard = 0,
    ModelDeploy = 1,
    ModelUpdate = 2,
    InferenceRequest = 3,
    TrainingJob = 4,
    LoraAdapter = 5,
}

impl TransactionType {
    pub fn from_data(data: &[u8]) -> Self {
        if data.len() >= 4 {
            match &data[0..4] {
                [0x01, 0x00, 0x00, 0x00] => TransactionType::ModelDeploy,
                [0x02, 0x00, 0x00, 0x00] => TransactionType::ModelUpdate,
                [0x03, 0x00, 0x00, 0x00] => TransactionType::InferenceRequest,
                [0x04, 0x00, 0x00, 0x00] => TransactionType::TrainingJob,
                [0x05, 0x00, 0x00, 0x00] => TransactionType::LoraAdapter,
                _ => TransactionType::Standard,
            }
        } else {
            TransactionType::Standard
        }
    }

    /// Get priority weight for mempool ordering
    pub fn priority_weight(&self) -> u32 {
        match self {
            TransactionType::ModelDeploy => 100, // Highest priority
            TransactionType::TrainingJob => 90,
            TransactionType::ModelUpdate => 80,
            TransactionType::LoraAdapter => 70,
            TransactionType::InferenceRequest => 60,
            TransactionType::Standard => 10, // Lowest priority
        }
    }
}

/// Transaction structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub hash: Hash,
    pub nonce: u64,
    pub from: PublicKey,
    pub to: Option<PublicKey>,
    pub value: u128,
    pub gas_limit: u64,
    pub gas_price: u64,
    pub data: Vec<u8>,
    pub signature: Signature,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tx_type: Option<TransactionType>,
}

impl Transaction {
    /// Determine transaction type from data
    pub fn determine_type(&mut self) {
        self.tx_type = Some(TransactionType::from_data(&self.data));
    }

    /// Get transaction priority for mempool
    pub fn priority(&self) -> u64 {
        let type_weight = self
            .tx_type
            .unwrap_or(TransactionType::Standard)
            .priority_weight() as u64;

        // Combine type weight with gas price for final priority
        (type_weight * 1_000_000) + self.gas_price
    }
}

// ============================================================================
// AI Model Types for Genesis Block (Hybrid Architecture)
// ============================================================================

/// Unique identifier for AI models
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ModelId(pub String);

impl ModelId {
    pub fn from_name(name: &str) -> Self {
        Self(name.to_string())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ModelId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Type of AI model
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModelType {
    /// Embedding model for semantic search
    Embeddings,
    /// Small language model (< 1B params)
    TinyLLM,
    /// General purpose LLM
    GeneralLLM,
    /// Code-specialized model
    CodeLLM,
    /// Vision-language model
    VisionLLM,
    /// Diffusion model for image generation
    Diffusion,
}

/// Metadata about an AI model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelMetadata {
    /// Human-readable name
    pub name: String,
    /// Version string (e.g., "1.0.0")
    pub version: String,
    /// Context length in tokens
    pub context_length: u32,
    /// Embedding dimension (for embedding models)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embedding_dim: Option<u32>,
    /// License (MIT, Apache 2.0, Llama 3.1, etc.)
    pub license: String,
    /// Model framework (GGUF, SafeTensors, etc.)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub framework: Option<String>,
}

/// Model embedded directly in genesis block
/// These models are stored in-block and always available
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddedModel {
    /// Unique model identifier
    pub model_id: ModelId,
    /// Type of model
    pub model_type: ModelType,
    /// Raw model weights (GGUF format)
    pub weights: Vec<u8>,
    /// Model metadata
    pub metadata: ModelMetadata,
}

impl EmbeddedModel {
    /// Get the size of the model in bytes
    pub fn size_bytes(&self) -> usize {
        self.weights.len()
    }

    /// Calculate SHA256 hash of model weights
    pub fn weights_hash(&self) -> Hash {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(&self.weights);
        let result = hasher.finalize();
        Hash::from_bytes(&result)
    }
}

/// Model required to be pinned on IPFS by validators
/// These models are verified via consensus but stored off-chain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequiredModel {
    /// Unique model identifier
    pub model_id: ModelId,
    /// IPFS content identifier
    pub ipfs_cid: String,
    /// SHA256 hash for verification
    pub sha256_hash: Hash,
    /// Size in bytes
    pub size_bytes: u64,
    /// Whether validators MUST pin this model
    pub must_pin: bool,
    /// Penalty in LATT tokens for not pinning
    pub slash_penalty: u128,
    /// Grace period in hours for new validators
    #[serde(default = "default_grace_period")]
    pub grace_period_hours: u64,
}

fn default_grace_period() -> u64 {
    24 // 24 hours default
}

impl RequiredModel {
    /// Create a new required model entry
    pub fn new(
        model_id: ModelId,
        ipfs_cid: String,
        sha256_hash: Hash,
        size_bytes: u64,
        slash_penalty: u128,
    ) -> Self {
        Self {
            model_id,
            ipfs_cid,
            sha256_hash,
            size_bytes,
            must_pin: true,
            slash_penalty,
            grace_period_hours: default_grace_period(),
        }
    }
}

/// Status of a validator's model pin
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PinStatus {
    /// Model is pinned and verified
    Pinned,
    /// Model is not pinned
    Unpinned,
    /// Pin status not yet verified
    Unverified,
}

/// Record of a validator's pin check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorPinCheck {
    /// Validator's public key
    pub validator: PublicKey,
    /// Model CID being checked
    pub model_cid: String,
    /// Last check timestamp
    pub last_check: u64,
    /// Current status
    pub status: PinStatus,
    /// Last challenge-response proof
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_proof: Option<Vec<u8>>,
}

/// Blue set information for a block
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlueSet {
    /// Set of blue block hashes
    pub blocks: HashSet<Hash>,

    /// Blue score (cumulative blue blocks in ancestry)
    pub score: u64,

    /// Blue work (cumulative difficulty)
    pub work: u128,
}

impl BlueSet {
    pub fn new() -> Self {
        Self {
            blocks: HashSet::new(),
            score: 0,
            work: 0,
        }
    }

    pub fn contains(&self, hash: &Hash) -> bool {
        self.blocks.contains(hash)
    }

    pub fn insert(&mut self, hash: Hash) {
        self.blocks.insert(hash);
        self.score += 1;
    }

    pub fn size(&self) -> usize {
        self.blocks.len()
    }
}

impl Default for BlueSet {
    fn default() -> Self {
        Self::new()
    }
}

/// DAG relationship between blocks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DagRelation {
    pub block: Hash,
    pub selected_parent: Hash,
    pub merge_parents: Vec<Hash>,
    pub children: Vec<Hash>,
    pub blue_set: BlueSet,
    pub is_chain_block: bool,
}

/// Represents a tip in the DAG
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tip {
    pub hash: Hash,
    pub blue_score: u64,
    pub height: u64,
    pub timestamp: u64,
}

impl Tip {
    pub fn new(block: &Block) -> Self {
        Self {
            hash: block.hash(),
            blue_score: block.header.blue_score,
            height: block.header.height,
            timestamp: block.header.timestamp,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_display() {
        let hash = Hash::new([0x12; 32]);
        assert_eq!(hash.to_hex().len(), 64);
        assert_eq!(format!("{}", hash), "12121212");
    }

    #[test]
    fn test_block_parents() {
        let mut block = create_test_block();
        block.header.selected_parent_hash = Hash::new([1; 32]);
        block.header.merge_parent_hashes = vec![Hash::new([2; 32]), Hash::new([3; 32])];

        let parents = block.parents();
        assert_eq!(parents.len(), 3);
        assert_eq!(parents[0], Hash::new([1; 32]));
        assert_eq!(parents[1], Hash::new([2; 32]));
        assert_eq!(parents[2], Hash::new([3; 32]));
    }

    #[test]
    fn test_blue_set() {
        let mut blue_set = BlueSet::new();
        assert_eq!(blue_set.score, 0);

        blue_set.insert(Hash::new([1; 32]));
        blue_set.insert(Hash::new([2; 32]));

        assert_eq!(blue_set.score, 2);
        assert_eq!(blue_set.size(), 2);
        assert!(blue_set.contains(&Hash::new([1; 32])));
        assert!(!blue_set.contains(&Hash::new([3; 32])));
    }

    fn create_test_block() -> Block {
        Block {
            header: BlockHeader {
                version: 1,
                block_hash: Hash::new([0; 32]),
                selected_parent_hash: Hash::default(),
                merge_parent_hashes: vec![],
                timestamp: 0,
                height: 0,
                blue_score: 0,
                blue_work: 0,
                pruning_point: Hash::default(),
                proposer_pubkey: PublicKey::new([0; 32]),
                vrf_reveal: VrfProof {
                    proof: vec![],
                    output: Hash::default(),
                },
            },
            state_root: Hash::default(),
            tx_root: Hash::default(),
            receipt_root: Hash::default(),
            artifact_root: Hash::default(),
            ghostdag_params: GhostDagParams::default(),
            transactions: vec![],
            signature: Signature::new([0; 64]),
            embedded_models: vec![],
            required_pins: vec![],
        }
    }
}
