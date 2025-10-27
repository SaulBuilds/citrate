use anyhow::Result;
use citrate_consensus::{
    types::{Block, BlockHeader, GhostDagParams, Hash, PublicKey, Signature, VrfProof},
    GhostDag,
};
use citrate_storage::StorageManager;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::Arc;
use tracing::{debug, info};

/// Manages DAG data for visualization and analysis
pub struct DAGManager {
    storage: Arc<StorageManager>,
    ghostdag: Arc<GhostDag>,
}

impl DAGManager {
    pub fn new(storage: Arc<StorageManager>, ghostdag: Arc<GhostDag>) -> Self {
        Self { storage, ghostdag }
    }

    /// Get DAG data for visualization
    pub async fn get_dag_data(&self, limit: usize, start_height: Option<u64>) -> Result<DAGData> {
        let mut nodes: Vec<DAGNode> = Vec::new();
        let mut links = Vec::new();

        // Get the latest height from storage
        let latest_height = self.storage.blocks.get_latest_height().unwrap_or(0);
        info!("DAG: Latest height in storage: {}", latest_height);

        if latest_height == 0 {
            info!("DAG: No blocks in storage, returning empty data");
            // Return empty data if no blocks exist
            return Ok(DAGData {
                nodes: vec![],
                links: vec![],
                tips: vec![],
                statistics: DAGStatistics {
                    total_blocks: 0,
                    blue_blocks: 0,
                    red_blocks: 0,
                    current_tips: 0,
                    average_blue_score: 0.0,
                    max_height: 0,
                },
            });
        }

        // Calculate the range of blocks to fetch
        let base_height = start_height.unwrap_or(latest_height.saturating_sub(limit as u64 - 1));
        let end_height = (base_height + limit as u64).min(latest_height + 1);

        info!(
            "DAG: Fetching blocks from height {} to {} (limit: {})",
            base_height, end_height, limit
        );

        // Fetch blocks from storage (build nodes, links)
        let mut nodes_tmp: Vec<(Hash, DAGNode)> = Vec::new();
        for height in base_height..end_height {
            // Get block hash at this height
            match self.storage.blocks.get_block_by_height(height) {
                Ok(Some(block_hash)) => {
                    // Get the actual block using the hash
                    match self.storage.blocks.get_block(&block_hash) {
                        Ok(Some(block)) => {
                            let block_hash_str = block.header.block_hash.to_hex();

                            // Create node for visualization; is_blue set after computing best tip blue set
                            let node = DAGNode {
                                id: block_hash_str.clone(),
                                hash: block_hash_str.clone(),
                                height: block.header.height,
                                timestamp: block.header.timestamp,
                                is_blue: false,
                                blue_score: match self
                                    .ghostdag
                                    .get_blue_score(&block.header.block_hash)
                                    .await
                                {
                                    Ok(s) => s,
                                    Err(_) => block.header.blue_score,
                                },
                                selected_parent: block.header.selected_parent_hash.to_hex(),
                                merge_parents: block
                                    .header
                                    .merge_parent_hashes
                                    .iter()
                                    .map(|h| h.to_hex())
                                    .collect(),
                                transactions: block.transactions.len(),
                                proposer: hex::encode(block.header.proposer_pubkey.as_bytes()),
                                size: 1000 + (height as usize * 100), // Approximate size
                            };

                            // Create links for visualization
                            let selected_parent = block.header.selected_parent_hash.to_hex();
                            if !selected_parent.starts_with("00000000000000000000000000000000") {
                                links.push(DAGLink {
                                    source: selected_parent,
                                    target: block_hash_str.clone(),
                                    is_selected: true,
                                    link_type: LinkType::SelectedParent,
                                });
                            }

                            // Add merge parent links
                            for merge_parent in &block.header.merge_parent_hashes {
                                let merge_parent_str = merge_parent.to_hex();
                                if !merge_parent_str.starts_with("00000000000000000000000000000000")
                                {
                                    links.push(DAGLink {
                                        source: merge_parent_str,
                                        target: block_hash_str.clone(),
                                        is_selected: false,
                                        link_type: LinkType::MergeParent,
                                    });
                                }
                            }

                            nodes_tmp.push((block.header.block_hash, node));
                        }
                        Ok(None) => {
                            debug!("Block not found for hash at height {}", height);
                        }
                        Err(e) => {
                            debug!("Error fetching block at height {}: {}", height, e);
                        }
                    }
                }
                Ok(None) => {
                    debug!("No block hash found at height {}", height);
                }
                Err(e) => {
                    debug!("Error fetching block hash at height {}: {}", height, e);
                }
            }
        }

        // Determine blue set for the best tip and finalize nodes
        let mut blue_hashes: HashSet<Hash> = HashSet::new();
        if let Ok(best_tip_hash) = self.ghostdag.select_tip().await {
            if let Ok(Some(best_tip_block)) = self.storage.blocks.get_block(&best_tip_hash) {
                if let Ok(blue_set) = self.ghostdag.calculate_blue_set(&best_tip_block).await {
                    blue_hashes = blue_set.blocks;
                }
            }
        }
        for (h, mut node) in nodes_tmp {
            node.is_blue = blue_hashes.contains(&h);
            nodes.push(node);
        }

        // Real tips from ghostdag
        let mut tips: Vec<TipInfo> = Vec::new();
        for tip in self.ghostdag.get_tips().await {
            if let Ok(Some(block)) = self.storage.blocks.get_block(&tip) {
                let blue_score = self
                    .ghostdag
                    .get_blue_score(&tip)
                    .await
                    .unwrap_or(block.header.blue_score);
                tips.push(TipInfo {
                    hash: tip.to_hex(),
                    height: block.header.height,
                    timestamp: block.header.timestamp,
                    blue_score,
                    cumulative_weight: blue_score * 10,
                });
            }
        }

        // Calculate statistics after nodes and tips are ready
        let stats = if !nodes.is_empty() {
            DAGStatistics {
                total_blocks: nodes.len(),
                blue_blocks: nodes.iter().filter(|n| n.is_blue).count(),
                red_blocks: nodes.iter().filter(|n| !n.is_blue).count(),
                current_tips: tips.len(),
                average_blue_score: if nodes.is_empty() {
                    0.0
                } else {
                    nodes.iter().map(|n| n.blue_score).sum::<u64>() as f64 / nodes.len() as f64
                },
                max_height: nodes.iter().map(|n| n.height).max().unwrap_or(0),
            }
        } else {
            DAGStatistics {
                total_blocks: 0,
                blue_blocks: 0,
                red_blocks: 0,
                current_tips: tips.len(),
                average_blue_score: 0.0,
                max_height: 0,
            }
        };

        Ok(DAGData {
            nodes,
            links,
            tips,
            statistics: stats,
        })
    }

    /// Get detailed block information
    pub async fn get_block_details(&self, hash: &str) -> Result<BlockDetails> {
        let h = Hash::from_bytes(&hex::decode(hash).unwrap_or_default());
        let block = self
            .storage
            .blocks
            .get_block(&h)?
            .ok_or_else(|| anyhow::anyhow!("Block not found"))?;
        let blue_score = self
            .ghostdag
            .get_blue_score(&block.header.block_hash)
            .await
            .unwrap_or(block.header.blue_score);
        let children = self
            .storage
            .blocks
            .get_children(&block.header.block_hash)
            .unwrap_or_default();

        // Determine if this block is blue using the current best tip's blue set
        let mut is_blue = false;
        if let Ok(best_tip_hash) = self.ghostdag.select_tip().await {
            if let Ok(Some(best_tip_block)) = self.storage.blocks.get_block(&best_tip_hash) {
                if let Ok(blue_set) = self.ghostdag.calculate_blue_set(&best_tip_block).await {
                    is_blue = blue_set.contains(&block.header.block_hash);
                }
            }
        }

        Ok(BlockDetails {
            hash: block.header.block_hash.to_hex(),
            height: block.header.height,
            timestamp: block.header.timestamp,
            is_blue,
            blue_score,
            selected_parent: block.header.selected_parent_hash.to_hex(),
            merge_parents: block
                .header
                .merge_parent_hashes
                .iter()
                .map(|h| h.to_hex())
                .collect(),
            transactions: block
                .transactions
                .iter()
                .map(|tx| {
                    // Derive wallet-style addresses from public keys (keccak256(pubkey)[12..])
                    let from_addr = {
                        use sha3::{Digest, Keccak256};
                        let mut hasher = Keccak256::new();
                        hasher.update(tx.from.as_bytes());
                        let digest = hasher.finalize();
                        format!("0x{}", hex::encode(&digest[12..]))
                    };
                    let to_addr_opt = tx.to.as_ref().map(|to| {
                        let bytes = to.as_bytes();
                        if bytes[20..].iter().all(|&b| b == 0) {
                            format!("0x{}", hex::encode(&bytes[0..20]))
                        } else {
                            use sha3::{Digest, Keccak256};
                            let mut hasher = Keccak256::new();
                            hasher.update(bytes);
                            let digest = hasher.finalize();
                            format!("0x{}", hex::encode(&digest[12..]))
                        }
                    });
                    TransactionInfo {
                        hash: tx.hash.to_hex(),
                        from: hex::encode(tx.from.as_bytes()),
                        to: tx.to.as_ref().map(|addr| hex::encode(addr.as_bytes())),
                        from_addr,
                        to_addr: to_addr_opt,
                        value: tx.value.to_string(),
                        gas_used: tx.gas_limit,
                        status: true,
                    }
                })
                .collect(),
            proposer: hex::encode(block.header.proposer_pubkey.as_bytes()),
            size: 0,
            state_root: block.state_root.to_hex(),
            tx_root: block.tx_root.to_hex(),
            receipt_root: block.receipt_root.to_hex(),
            children: children.iter().map(|c| c.to_hex()).collect(),
        })
    }

    /// Get the blue set for a given block
    pub async fn get_blue_set(&self, block_hash: &str) -> Result<Vec<String>> {
        let hash = Hash::from_bytes(&hex::decode(block_hash).unwrap_or_default());
        let block = self
            .storage
            .blocks
            .get_block(&hash)?
            .ok_or_else(|| anyhow::anyhow!("Block not found"))?;
        let blue = self.ghostdag.calculate_blue_set(&block).await?;
        Ok(blue.blocks.iter().map(|h| h.to_hex()).collect())
    }

    /// Get current DAG tips
    pub async fn get_current_tips(&self) -> Result<Vec<TipInfo>> {
        let mut tips = Vec::new();
        for tip in self.ghostdag.get_tips().await {
            if let Ok(Some(block)) = self.storage.blocks.get_block(&tip) {
                let blue = self
                    .ghostdag
                    .get_blue_score(&tip)
                    .await
                    .unwrap_or(block.header.blue_score);
                tips.push(TipInfo {
                    hash: tip.to_hex(),
                    height: block.header.height,
                    timestamp: block.header.timestamp,
                    blue_score: blue,
                    cumulative_weight: blue * 10,
                });
            }
        }
        Ok(tips)
    }

    /// Calculate the blue score for a block
    pub async fn calculate_blue_score(&self, block_hash: &str) -> Result<u64> {
        let h = Hash::from_bytes(&hex::decode(block_hash).unwrap_or_default());
        self.ghostdag
            .get_blue_score(&h)
            .await
            .map_err(|e| anyhow::anyhow!(e))
    }

    /// Get the path from genesis to a specific block
    pub async fn get_block_path(&self, block_hash: &str) -> Result<Vec<String>> {
        let mut path = Vec::new();
        let mut current = Hash::from_bytes(&hex::decode(block_hash).unwrap_or_default());
        while let Some(block) = self.storage.blocks.get_block(&current)? {
            path.push(block.hash().to_hex());
            if block.header.selected_parent_hash == Hash::default() {
                break;
            }
            current = block.header.selected_parent_hash;
        }
        path.reverse();
        Ok(path)
    }

    /// Create a sample block for testing/visualization
    #[allow(dead_code)]
    fn create_sample_block(height: u64, parent_hash: &str) -> Block {
        Block {
            header: BlockHeader {
                version: 1,
                block_hash: Hash::new([
                    (height as u8),
                    (height as u8 + 1),
                    (height as u8 + 2),
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                ]),
                selected_parent_hash: if parent_hash.is_empty() {
                    Hash::new([0u8; 32])
                } else {
                    let mut bytes = [0u8; 32];
                    hex::decode(parent_hash)
                        .unwrap_or_default()
                        .iter()
                        .take(32)
                        .enumerate()
                        .for_each(|(i, b)| bytes[i] = *b);
                    Hash::new(bytes)
                },
                merge_parent_hashes: vec![],
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                height,
                blue_score: height * 10,
                blue_work: height as u128 * 1000,
                pruning_point: Hash::new([0u8; 32]),
                proposer_pubkey: PublicKey::new([1u8; 32]),
                vrf_reveal: VrfProof {
                    proof: vec![0u8; 80],
                    output: Hash::new([0u8; 32]),
                },
            },
            state_root: Hash::new([(height as u8 + 10); 32]),
            tx_root: Hash::new([(height as u8 + 20); 32]),
            receipt_root: Hash::new([(height as u8 + 30); 32]),
            artifact_root: Hash::new([(height as u8 + 40); 32]),
            ghostdag_params: GhostDagParams::default(),
            transactions: vec![],
            signature: Signature::new([0u8; 64]),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DAGData {
    pub nodes: Vec<DAGNode>,
    pub links: Vec<DAGLink>,
    pub tips: Vec<TipInfo>,
    pub statistics: DAGStatistics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DAGNode {
    pub id: String,
    pub hash: String,
    pub height: u64,
    pub timestamp: u64,
    pub is_blue: bool,
    pub blue_score: u64,
    pub selected_parent: String,
    pub merge_parents: Vec<String>,
    pub transactions: usize,
    pub proposer: String,
    pub size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DAGLink {
    pub source: String,
    pub target: String,
    pub is_selected: bool,
    pub link_type: LinkType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LinkType {
    SelectedParent,
    MergeParent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DAGStatistics {
    pub total_blocks: usize,
    pub blue_blocks: usize,
    pub red_blocks: usize,
    pub current_tips: usize,
    pub average_blue_score: f64,
    pub max_height: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TipInfo {
    pub hash: String,
    pub height: u64,
    pub timestamp: u64,
    pub blue_score: u64,
    pub cumulative_weight: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockDetails {
    pub hash: String,
    pub height: u64,
    pub timestamp: u64,
    pub is_blue: bool,
    pub blue_score: u64,
    pub selected_parent: String,
    pub merge_parents: Vec<String>,
    pub transactions: Vec<TransactionInfo>,
    pub proposer: String,
    pub size: usize,
    pub state_root: String,
    pub tx_root: String,
    pub receipt_root: String,
    pub children: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionInfo {
    pub hash: String,
    pub from: String,
    pub to: Option<String>,
    pub from_addr: String,
    pub to_addr: Option<String>,
    pub value: String,
    pub gas_used: u64,
    pub status: bool,
}
