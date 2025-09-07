use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fmt;

/// Hash type for block and transaction identifiers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default, PartialOrd, Ord)]
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
            k: 18,  // Standard k-cluster parameter
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
        self.header.selected_parent_hash == Hash::default() && 
        self.header.merge_parent_hashes.is_empty()
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
        block.header.merge_parent_hashes = vec![
            Hash::new([2; 32]),
            Hash::new([3; 32]),
        ];
        
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
        }
    }
}