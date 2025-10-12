// lattice-v3/core/storage/src/chain/block_store.rs

use crate::db::{column_families::*, RocksDB};
use anyhow::Result;
use lattice_consensus::types::{Block, BlockHeader, Hash};
use std::collections::HashSet;
use std::sync::Arc;
use tracing::{debug, info};

/// Block storage manager
pub struct BlockStore {
    db: Arc<RocksDB>,
}

impl BlockStore {
    pub fn new(db: Arc<RocksDB>) -> Self {
        Self { db }
    }

    /// Store a complete block
    pub fn put_block(&self, block: &Block) -> Result<()> {
        let hash = block.hash();
        let block_bytes = bincode::serialize(block)?;

        let mut batch = self.db.batch();

        // Store full block
        self.db
            .batch_put_cf(&mut batch, CF_BLOCKS, hash.as_bytes(), &block_bytes)?;

        // Store header separately for quick access
        let header_bytes = bincode::serialize(&block.header)?;
        self.db
            .batch_put_cf(&mut batch, CF_HEADERS, hash.as_bytes(), &header_bytes)?;

        // Store height -> hash mapping
        let height_key = height_to_key(block.header.height);
        self.db
            .batch_put_cf(&mut batch, CF_METADATA, &height_key, hash.as_bytes())?;

        // Store parent -> children mappings for DAG
        for parent in block.parents() {
            let parent_children_key = parent_children_key(&parent);
            let mut children = self.get_children(&parent)?;
            children.push(hash);
            let children_bytes = bincode::serialize(&children)?;
            self.db.batch_put_cf(
                &mut batch,
                CF_DAG_RELATIONS,
                &parent_children_key,
                &children_bytes,
            )?;
        }

        // Store blue set information
        if block.header.blue_score > 0 {
            let blue_score_key = blue_score_key(block.header.blue_score);
            self.db
                .batch_put_cf(&mut batch, CF_BLUE_SET, &blue_score_key, hash.as_bytes())?;
        }

        self.db.write_batch(batch)?;

        debug!("Stored block {} at height {}", hash, block.header.height);
        Ok(())
    }

    /// Get a block by hash
    pub fn get_block(&self, hash: &Hash) -> Result<Option<Block>> {
        match self.db.get_cf(CF_BLOCKS, hash.as_bytes())? {
            Some(bytes) => Ok(Some(bincode::deserialize(&bytes)?)),
            None => Ok(None),
        }
    }

    /// Get block header by hash
    pub fn get_header(&self, hash: &Hash) -> Result<Option<BlockHeader>> {
        match self.db.get_cf(CF_HEADERS, hash.as_bytes())? {
            Some(bytes) => Ok(Some(bincode::deserialize(&bytes)?)),
            None => Ok(None),
        }
    }

    /// Check if block exists
    pub fn has_block(&self, hash: &Hash) -> Result<bool> {
        self.db.exists_cf(CF_BLOCKS, hash.as_bytes())
    }

    /// Get block hash by height
    pub fn get_block_by_height(&self, height: u64) -> Result<Option<Hash>> {
        let height_key = height_to_key(height);
        match self.db.get_cf(CF_METADATA, &height_key)? {
            Some(bytes) => Ok(Some(Hash::from_bytes(&bytes))),
            None => Ok(None),
        }
    }

    /// Get children of a block
    pub fn get_children(&self, parent: &Hash) -> Result<Vec<Hash>> {
        let key = parent_children_key(parent);
        match self.db.get_cf(CF_DAG_RELATIONS, &key)? {
            Some(bytes) => Ok(bincode::deserialize(&bytes)?),
            None => Ok(Vec::new()),
        }
    }

    /// Get latest block height
    pub fn get_latest_height(&self) -> Result<u64> {
        // Iterate through height mappings to find the highest
        let mut max_height = 0u64;
        for (key, _) in self.db.iter_cf(CF_METADATA)? {
            if key.len() == 9 && key[0] == b'h' {
                let height = u64::from_be_bytes(key[1..9].try_into()?);
                max_height = max_height.max(height);
            }
        }
        Ok(max_height)
    }

    /// Get blocks by blue score range
    pub fn get_blocks_by_blue_score(&self, start: u64, end: u64) -> Result<Vec<Hash>> {
        let mut blocks = Vec::new();
        let start_key = blue_score_key(start);
        let end_key = blue_score_key(end);

        for (key, value) in self.db.iter_cf(CF_BLUE_SET)? {
            if key.as_ref() >= start_key.as_slice() && key.as_ref() <= end_key.as_slice() {
                blocks.push(Hash::from_bytes(&value));
            }
        }

        Ok(blocks)
    }

    /// Return the current DAG tips (blocks without known children), sorted by height descending.
    pub fn get_tips(&self) -> Result<Vec<Hash>> {
        let mut all_blocks = HashSet::new();
        for (key, _) in self.db.iter_cf(CF_BLOCKS)? {
            let key_bytes = key.as_ref();
            if key_bytes.len() == 32 {
                all_blocks.insert(Hash::from_bytes(key_bytes));
            }
        }

        let mut parents_with_children = HashSet::new();
        for (key, value) in self.db.iter_cf(CF_DAG_RELATIONS)? {
            let key_bytes = key.as_ref();
            if key_bytes.len() == 33 && key_bytes[0] == b'c' && !value.is_empty() {
                let parent_hash = Hash::from_bytes(&key_bytes[1..]);
                parents_with_children.insert(parent_hash);
            }
        }

        let mut tips: Vec<(u64, Hash)> = Vec::new();

        for hash in all_blocks.into_iter().filter(|h| !parents_with_children.contains(h)) {
            let height = self
                .get_header(&hash)?
                .map(|header| header.height)
                .unwrap_or_default();
            tips.push((height, hash));
        }

        tips.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| b.1.cmp(&a.1)));
        Ok(tips.into_iter().map(|(_, hash)| hash).collect())
    }

    /// Delete a block and its associated data
    pub fn delete_block(&self, hash: &Hash) -> Result<()> {
        // Get block first to clean up relationships
        if let Some(block) = self.get_block(hash)? {
            let mut batch = self.db.batch();

            // Delete block
            self.db
                .batch_delete_cf(&mut batch, CF_BLOCKS, hash.as_bytes())?;

            // Delete header
            self.db
                .batch_delete_cf(&mut batch, CF_HEADERS, hash.as_bytes())?;

            // Delete height mapping
            let height_key = height_to_key(block.header.height);
            self.db
                .batch_delete_cf(&mut batch, CF_METADATA, &height_key)?;

            // Clean up parent relationships
            for parent in block.parents() {
                let parent_children_key = parent_children_key(&parent);
                if let Ok(mut children) = self.get_children(&parent) {
                    children.retain(|&child| child != *hash);
                    let children_bytes = bincode::serialize(&children)?;
                    self.db.batch_put_cf(
                        &mut batch,
                        CF_DAG_RELATIONS,
                        &parent_children_key,
                        &children_bytes,
                    )?;
                }
            }

            // Delete blue score mapping
            let blue_score_key = blue_score_key(block.header.blue_score);
            self.db
                .batch_delete_cf(&mut batch, CF_BLUE_SET, &blue_score_key)?;

            self.db.write_batch(batch)?;
            info!("Deleted block {}", hash);
        }

        Ok(())
    }

    /// Compact the block storage
    pub fn compact(&self) -> Result<()> {
        self.db.compact_cf(CF_BLOCKS)?;
        self.db.compact_cf(CF_HEADERS)?;
        self.db.compact_cf(CF_DAG_RELATIONS)?;
        self.db.compact_cf(CF_BLUE_SET)?;
        Ok(())
    }
}

// Key generation helpers
fn height_to_key(height: u64) -> Vec<u8> {
    let mut key = vec![b'h'];
    key.extend_from_slice(&height.to_be_bytes());
    key
}

fn parent_children_key(parent: &Hash) -> Vec<u8> {
    let mut key = vec![b'c'];
    key.extend_from_slice(parent.as_bytes());
    key
}

fn blue_score_key(score: u64) -> Vec<u8> {
    let mut key = vec![b'b'];
    key.extend_from_slice(&score.to_be_bytes());
    key
}

#[cfg(test)]
mod tests {
    use super::*;
    use lattice_consensus::types::{PublicKey, Signature, VrfProof};
    use tempfile::TempDir;

    fn create_test_block(height: u64, parent: Hash) -> Block {
        Block {
            header: BlockHeader {
                version: 1,
                block_hash: Hash::new([height as u8; 32]),
                selected_parent_hash: parent,
                merge_parent_hashes: vec![],
                timestamp: 1000000 + height,
                height,
                blue_score: height * 10,
                blue_work: height as u128 * 100,
                pruning_point: Hash::default(),
                proposer_pubkey: PublicKey::new([1; 32]),
                vrf_reveal: VrfProof {
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
            signature: Signature::new([0; 64]),
        }
    }

    #[test]
    fn test_block_storage() {
        let temp_dir = TempDir::new().unwrap();
        let db = Arc::new(RocksDB::open(temp_dir.path()).unwrap());
        let store = BlockStore::new(db);

        // Store blocks
        let block1 = create_test_block(1, Hash::default());
        let block2 = create_test_block(2, block1.hash());

        store.put_block(&block1).unwrap();
        store.put_block(&block2).unwrap();

        // Retrieve blocks
        let retrieved = store.get_block(&block1.hash()).unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().header.height, 1);

        // Check existence
        assert!(store.has_block(&block1.hash()).unwrap());
        assert!(store.has_block(&block2.hash()).unwrap());

        // Get by height
        let hash_at_2 = store.get_block_by_height(2).unwrap();
        assert_eq!(hash_at_2, Some(block2.hash()));

        // Check children
        let children = store.get_children(&block1.hash()).unwrap();
        assert_eq!(children.len(), 1);
        assert_eq!(children[0], block2.hash());
    }
}
