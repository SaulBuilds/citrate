use lattice_consensus::types::Hash;
use serde::{Deserialize, Serialize};
use sha3::{Digest, Keccak256};
use std::collections::HashMap;

/// Merkle Patricia Trie node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrieNode {
    Empty,
    Leaf {
        key: Vec<u8>,
        value: Vec<u8>,
    },
    Branch {
        children: [Box<TrieNode>; 16],
        value: Option<Vec<u8>>,
    },
    Extension {
        prefix: Vec<u8>,
        node: Box<TrieNode>,
    },
}

impl Default for TrieNode {
    fn default() -> Self {
        TrieNode::Empty
    }
}

/// Merkle Patricia Trie
#[derive(Clone)]
pub struct Trie {
    root: TrieNode,
    cache: HashMap<Vec<u8>, Vec<u8>>,
}

impl Trie {
    pub fn new() -> Self {
        Self {
            root: TrieNode::Empty,
            cache: HashMap::new(),
        }
    }
    
    /// Insert a key-value pair
    pub fn insert(&mut self, key: Vec<u8>, value: Vec<u8>) {
        let nibbles = to_nibbles(&key);
        self.root = Self::insert_node(self.root.clone(), &nibbles, value.clone());
        self.cache.insert(key, value);
    }
    
    fn insert_node(node: TrieNode, key: &[u8], value: Vec<u8>) -> TrieNode {
        match node {
            TrieNode::Empty => TrieNode::Leaf {
                key: key.to_vec(),
                value,
            },
            
            TrieNode::Leaf { key: leaf_key, value: leaf_value } => {
                if leaf_key == key {
                    // Update existing leaf
                    TrieNode::Leaf {
                        key: key.to_vec(),
                        value,
                    }
                } else {
                    // Convert to branch
                    Self::create_branch(leaf_key, leaf_value, key.to_vec(), value)
                }
            }
            
            TrieNode::Branch { mut children, value: branch_value } => {
                if key.is_empty() {
                    // Update branch value
                    TrieNode::Branch {
                        children,
                        value: Some(value),
                    }
                } else {
                    let index = key[0] as usize;
                    children[index] = Box::new(Self::insert_node(
                        *children[index].clone(),
                        &key[1..],
                        value,
                    ));
                    TrieNode::Branch {
                        children,
                        value: branch_value,
                    }
                }
            }
            
            TrieNode::Extension { prefix, node } => {
                let common = common_prefix(&prefix, key);
                
                if common.len() == prefix.len() {
                    // Entire prefix matches
                    TrieNode::Extension {
                        prefix: prefix.clone(),
                        node: Box::new(Self::insert_node(*node, &key[common.len()..], value)),
                    }
                } else {
                    // Partial match - split extension
                    Self::split_extension(prefix, *node, key.to_vec(), value, common.len())
                }
            }
        }
    }
    
    /// Get a value by key
    pub fn get(&self, key: &[u8]) -> Option<Vec<u8>> {
        // Check cache first
        if let Some(value) = self.cache.get(key) {
            return Some(value.clone());
        }
        
        let nibbles = to_nibbles(key);
        Self::get_node(&self.root, &nibbles)
    }
    
    fn get_node(node: &TrieNode, key: &[u8]) -> Option<Vec<u8>> {
        match node {
            TrieNode::Empty => None,
            
            TrieNode::Leaf { key: leaf_key, value } => {
                if leaf_key == key {
                    Some(value.clone())
                } else {
                    None
                }
            }
            
            TrieNode::Branch { children, value } => {
                if key.is_empty() {
                    value.clone()
                } else {
                    let index = key[0] as usize;
                    Self::get_node(&children[index], &key[1..])
                }
            }
            
            TrieNode::Extension { prefix, node } => {
                if key.starts_with(prefix) {
                    Self::get_node(node, &key[prefix.len()..])
                } else {
                    None
                }
            }
        }
    }
    
    /// Remove a key
    pub fn remove(&mut self, key: &[u8]) {
        let nibbles = to_nibbles(key);
        self.root = Self::remove_node(self.root.clone(), &nibbles);
        self.cache.remove(key);
    }
    
    fn remove_node(node: TrieNode, key: &[u8]) -> TrieNode {
        match node {
            TrieNode::Empty => TrieNode::Empty,
            
            TrieNode::Leaf { key: ref leaf_key, .. } => {
                if leaf_key == key {
                    TrieNode::Empty
                } else {
                    node
                }
            }
            
            TrieNode::Branch { mut children, value } => {
                if key.is_empty() {
                    // Remove branch value
                    TrieNode::Branch {
                        children,
                        value: None,
                    }
                } else {
                    let index = key[0] as usize;
                    children[index] = Box::new(Self::remove_node(
                        *children[index].clone(),
                        &key[1..],
                    ));
                    
                    // Check if branch can be simplified
                    Self::simplify_branch(children, value)
                }
            }
            
            TrieNode::Extension { prefix, node } => {
                if key.starts_with(&prefix) {
                    let new_node = Self::remove_node(*node, &key[prefix.len()..]);
                    if matches!(new_node, TrieNode::Empty) {
                        TrieNode::Empty
                    } else {
                        TrieNode::Extension {
                            prefix,
                            node: Box::new(new_node),
                        }
                    }
                } else {
                    TrieNode::Extension { prefix, node }
                }
            }
        }
    }
    
    /// Calculate the root hash
    pub fn root_hash(&self) -> Hash {
        let encoded = self.encode_node(&self.root);
        let mut hasher = Keccak256::new();
        hasher.update(&encoded);
        Hash::new(hasher.finalize().into())
    }
    
    fn encode_node(&self, node: &TrieNode) -> Vec<u8> {
        match node {
            TrieNode::Empty => vec![],
            
            TrieNode::Leaf { key, value } => {
                let items: [&[u8]; 2] = [key.as_slice(), value.as_slice()];
                rlp::encode_list::<&[u8], _>(&items).to_vec()
            }
            
            TrieNode::Branch { children, value } => {
                let mut items: Vec<Vec<u8>> = Vec::new();
                for child in children.iter() {
                    items.push(self.encode_node(child));
                }
                if let Some(v) = value {
                    items.push(v.clone());
                } else {
                    items.push(vec![]);
                }
                let items_refs: Vec<&[u8]> = items.iter().map(|v| v.as_slice()).collect();
                rlp::encode_list::<&[u8], _>(&items_refs).to_vec()
            }
            
            TrieNode::Extension { prefix, node } => {
                let node_encoded = self.encode_node(node);
                let items: [&[u8]; 2] = [prefix.as_slice(), node_encoded.as_slice()];
                rlp::encode_list::<&[u8], _>(&items).to_vec()
            }
        }
    }
    
    // Helper functions
    
    fn create_branch(key1: Vec<u8>, value1: Vec<u8>, key2: Vec<u8>, value2: Vec<u8>) -> TrieNode {
        let mut children: [Box<TrieNode>; 16] = default_children();
        
        if key1.is_empty() {
            // key1 goes to branch value
            let index = key2[0] as usize;
            children[index] = Box::new(TrieNode::Leaf {
                key: key2[1..].to_vec(),
                value: value2,
            });
            TrieNode::Branch {
                children,
                value: Some(value1),
            }
        } else if key2.is_empty() {
            // key2 goes to branch value
            let index = key1[0] as usize;
            children[index] = Box::new(TrieNode::Leaf {
                key: key1[1..].to_vec(),
                value: value1,
            });
            TrieNode::Branch {
                children,
                value: Some(value2),
            }
        } else {
            // Both go to children
            let index1 = key1[0] as usize;
            let index2 = key2[0] as usize;
            
            if index1 == index2 {
                children[index1] = Box::new(Self::create_branch(
                    key1[1..].to_vec(),
                    value1,
                    key2[1..].to_vec(),
                    value2,
                ));
            } else {
                children[index1] = Box::new(TrieNode::Leaf {
                    key: key1[1..].to_vec(),
                    value: value1,
                });
                children[index2] = Box::new(TrieNode::Leaf {
                    key: key2[1..].to_vec(),
                    value: value2,
                });
            }
            
            TrieNode::Branch {
                children,
                value: None,
            }
        }
    }
    
    fn split_extension(
        prefix: Vec<u8>,
        node: TrieNode,
        key: Vec<u8>,
        value: Vec<u8>,
        common_len: usize,
    ) -> TrieNode {
        let common = &prefix[..common_len];
        let remaining_prefix = &prefix[common_len..];
        let remaining_key = &key[common_len..];
        
        let mut children: [Box<TrieNode>; 16] = default_children();
        
        if !remaining_prefix.is_empty() {
            let index = remaining_prefix[0] as usize;
            if remaining_prefix.len() == 1 {
                children[index] = Box::new(node);
            } else {
                children[index] = Box::new(TrieNode::Extension {
                    prefix: remaining_prefix[1..].to_vec(),
                    node: Box::new(node),
                });
            }
        }
        
        let branch_value = if remaining_key.is_empty() {
            Some(value)
        } else {
            let index = remaining_key[0] as usize;
            children[index] = Box::new(TrieNode::Leaf {
                key: remaining_key[1..].to_vec(),
                value,
            });
            None
        };
        
        let branch = TrieNode::Branch {
            children,
            value: branch_value,
        };
        
        if common.is_empty() {
            branch
        } else {
            TrieNode::Extension {
                prefix: common.to_vec(),
                node: Box::new(branch),
            }
        }
    }
    
    fn simplify_branch(children: [Box<TrieNode>; 16], value: Option<Vec<u8>>) -> TrieNode {
        let non_empty: Vec<_> = children
            .iter()
            .enumerate()
            .filter(|(_, child)| !matches!(child.as_ref(), TrieNode::Empty))
            .collect();
        
        if non_empty.len() == 1 && value.is_none() {
            // Only one child - convert to extension or leaf
            let (index, child) = non_empty[0];
            match child.as_ref() {
                TrieNode::Leaf { key, value } => {
                    let mut new_key = vec![index as u8];
                    new_key.extend(key);
                    TrieNode::Leaf {
                        key: new_key,
                        value: value.clone(),
                    }
                }
                _ => TrieNode::Extension {
                    prefix: vec![index as u8],
                    node: child.clone(),
                },
            }
        } else {
            TrieNode::Branch { children, value }
        }
    }
}

/// Convert bytes to nibbles (4-bit values)
fn to_nibbles(bytes: &[u8]) -> Vec<u8> {
    let mut nibbles = Vec::with_capacity(bytes.len() * 2);
    for byte in bytes {
        nibbles.push(byte >> 4);
        nibbles.push(byte & 0x0f);
    }
    nibbles
}

/// Find common prefix length
fn common_prefix(a: &[u8], b: &[u8]) -> Vec<u8> {
    a.iter()
        .zip(b.iter())
        .take_while(|(x, y)| x == y)
        .map(|(x, _)| *x)
        .collect()
}

fn default_children() -> [Box<TrieNode>; 16] {
    [
        Box::new(TrieNode::Empty),
        Box::new(TrieNode::Empty),
        Box::new(TrieNode::Empty),
        Box::new(TrieNode::Empty),
        Box::new(TrieNode::Empty),
        Box::new(TrieNode::Empty),
        Box::new(TrieNode::Empty),
        Box::new(TrieNode::Empty),
        Box::new(TrieNode::Empty),
        Box::new(TrieNode::Empty),
        Box::new(TrieNode::Empty),
        Box::new(TrieNode::Empty),
        Box::new(TrieNode::Empty),
        Box::new(TrieNode::Empty),
        Box::new(TrieNode::Empty),
        Box::new(TrieNode::Empty),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_trie_insert_get() {
        let mut trie = Trie::new();
        
        trie.insert(b"key1".to_vec(), b"value1".to_vec());
        trie.insert(b"key2".to_vec(), b"value2".to_vec());
        trie.insert(b"key3".to_vec(), b"value3".to_vec());
        
        assert_eq!(trie.get(b"key1"), Some(b"value1".to_vec()));
        assert_eq!(trie.get(b"key2"), Some(b"value2".to_vec()));
        assert_eq!(trie.get(b"key3"), Some(b"value3".to_vec()));
        assert_eq!(trie.get(b"key4"), None);
    }
    
    #[test]
    fn test_trie_remove() {
        let mut trie = Trie::new();
        
        trie.insert(b"key1".to_vec(), b"value1".to_vec());
        trie.insert(b"key2".to_vec(), b"value2".to_vec());
        
        trie.remove(b"key1");
        
        assert_eq!(trie.get(b"key1"), None);
        assert_eq!(trie.get(b"key2"), Some(b"value2".to_vec()));
    }
    
    #[test]
    fn test_trie_root_hash() {
        let mut trie1 = Trie::new();
        let mut trie2 = Trie::new();
        
        // Same data should produce same root
        trie1.insert(b"key".to_vec(), b"value".to_vec());
        trie2.insert(b"key".to_vec(), b"value".to_vec());
        
        assert_eq!(trie1.root_hash(), trie2.root_hash());
        
        // Different data should produce different root
        trie2.insert(b"key2".to_vec(), b"value2".to_vec());
        assert_ne!(trie1.root_hash(), trie2.root_hash());
    }
}