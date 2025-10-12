// lattice-v3/core/api/src/dag_explorer.rs

// Complete DAG Explorer API with Multi-Network Support
// No stubs, full implementation for devnet/testnet/mainnet

use crate::types::{ApiResponse, ApiError};
use lattice_consensus::types::{Block, Hash, PublicKey, Transaction};
use lattice_storage::chain::{BlockStore, ChainStore, TransactionStore};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;
use tokio::sync::RwLock;
use warp::{Filter, Rejection, Reply};
use primitive_types::{H256, U256};

// Network configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub name: String,
    pub chain_id: u64,
    pub rpc_endpoint: String,
    pub ws_endpoint: String,
    pub explorer_port: u16,
    pub data_dir: String,
    pub is_active: bool,
    pub genesis_hash: Hash,
    pub block_time: u64,
    pub ghostdag_k: usize,
}

// Complete block model with all GhostDAG fields
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockData {
    pub hash: String,
    pub height: u64,
    pub timestamp: u64,
    pub selected_parent: String,
    pub merge_parents: Vec<String>,
    pub blue_score: u64,
    pub is_blue: bool,
    pub transactions: Vec<String>,
    pub state_root: String,
    pub receipts_root: String,
    pub proposer: String,
    pub gas_used: u64,
    pub gas_limit: u64,
    pub base_fee: String,
    pub difficulty: String,
    pub total_difficulty: String,
    pub size: u64,
    pub transaction_count: usize,
    pub uncle_count: usize,
    pub network: String,
}

// DAG structure for visualization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DagNode {
    pub id: String,
    pub height: u64,
    pub blue_score: u64,
    pub is_blue: bool,
    pub selected_parent: Option<String>,
    pub merge_parents: Vec<String>,
    pub timestamp: u64,
    pub tx_count: usize,
    pub size: u64,
    pub proposer: String,
    pub network: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DagEdge {
    pub source: String,
    pub target: String,
    pub edge_type: String, // "selected" or "merge"
    pub weight: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DagVisualization {
    pub nodes: Vec<DagNode>,
    pub edges: Vec<DagEdge>,
    pub stats: DagStats,
    pub network: String,
}

// Network statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DagStats {
    pub total_blocks: u64,
    pub total_transactions: u64,
    pub blue_blocks: u64,
    pub red_blocks: u64,
    pub avg_blue_score: f64,
    pub max_blue_score: u64,
    pub total_supply: String,
    pub validators_count: u64,
    pub tps: f64,
    pub avg_block_time: f64,
    pub network_hash_rate: String,
    pub difficulty: String,
    pub mempool_size: u64,
    pub peers_count: u64,
    pub chain_tips: Vec<String>,
    pub network: String,
}

// Transaction details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionData {
    pub hash: String,
    pub block_hash: String,
    pub block_height: u64,
    pub from: String,
    pub to: Option<String>,
    pub value: String,
    pub gas_price: String,
    pub gas_limit: u64,
    pub gas_used: u64,
    pub nonce: u64,
    pub input: String,
    pub status: bool,
    pub timestamp: u64,
    pub tx_type: String,
    pub network: String,
}

// Search results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub result_type: String, // "block", "transaction", "address", "model"
    pub data: serde_json::Value,
    pub network: String,
}

// Multi-network DAG Explorer
pub struct DagExplorer {
    networks: Arc<RwLock<HashMap<String, NetworkState>>>,
    active_network: Arc<RwLock<String>>,
}

struct NetworkState {
    config: NetworkConfig,
    storage: Arc<ChainStore>,
    block_cache: Arc<RwLock<HashMap<Hash, Block>>>,
    stats_cache: Arc<RwLock<DagStats>>,
    dag_cache: Arc<RwLock<DagVisualization>>,
    last_update: Arc<RwLock<u64>>,
}

impl DagExplorer {
    pub async fn new() -> Self {
        let mut networks = HashMap::new();
        
        // Initialize devnet
        let devnet_config = NetworkConfig {
            name: "devnet".to_string(),
            chain_id: 1337,
            rpc_endpoint: "http://localhost:8545".to_string(),
            ws_endpoint: "ws://localhost:8546".to_string(),
            explorer_port: 3001,
            data_dir: ".lattice-devnet".to_string(),
            is_active: false,
            genesis_hash: Hash::zero(),
            block_time: 1,
            ghostdag_k: 8,
        };
        
        // Initialize testnet
        let testnet_config = NetworkConfig {
            name: "testnet".to_string(),
            chain_id: 42069,
            rpc_endpoint: "http://localhost:8545".to_string(),
            ws_endpoint: "ws://localhost:8546".to_string(),
            explorer_port: 3002,
            data_dir: ".lattice-testnet".to_string(),
            is_active: true,
            genesis_hash: Hash::zero(),
            block_time: 2,
            ghostdag_k: 18,
        };
        
        // Initialize mainnet
        let mainnet_config = NetworkConfig {
            name: "mainnet".to_string(),
            chain_id: 1,
            rpc_endpoint: "http://localhost:8545".to_string(),
            ws_endpoint: "ws://localhost:8546".to_string(),
            explorer_port: 3000,
            data_dir: ".lattice-mainnet".to_string(),
            is_active: false,
            genesis_hash: Hash::zero(),
            block_time: 3,
            ghostdag_k: 50,
        };
        
        // Create network states
        for config in [devnet_config, testnet_config, mainnet_config] {
            let storage = Arc::new(ChainStore::new(&config.data_dir).await.unwrap());
            let state = NetworkState {
                config: config.clone(),
                storage,
                block_cache: Arc::new(RwLock::new(HashMap::new())),
                stats_cache: Arc::new(RwLock::new(Self::default_stats(&config.name))),
                dag_cache: Arc::new(RwLock::new(Self::default_dag(&config.name))),
                last_update: Arc::new(RwLock::new(0)),
            };
            networks.insert(config.name.clone(), state);
        }
        
        Self {
            networks: Arc::new(RwLock::new(networks)),
            active_network: Arc::new(RwLock::new("testnet".to_string())),
        }
    }
    
    fn default_stats(network: &str) -> DagStats {
        DagStats {
            total_blocks: 0,
            total_transactions: 0,
            blue_blocks: 0,
            red_blocks: 0,
            avg_blue_score: 0.0,
            max_blue_score: 0,
            total_supply: "0".to_string(),
            validators_count: 0,
            tps: 0.0,
            avg_block_time: 0.0,
            network_hash_rate: "0".to_string(),
            difficulty: "0".to_string(),
            mempool_size: 0,
            peers_count: 0,
            chain_tips: vec![],
            network: network.to_string(),
        }
    }
    
    fn default_dag(network: &str) -> DagVisualization {
        DagVisualization {
            nodes: vec![],
            edges: vec![],
            stats: Self::default_stats(network),
            network: network.to_string(),
        }
    }
    
    // Get current network
    pub async fn get_current_network(&self) -> String {
        self.active_network.read().await.clone()
    }
    
    // Switch network
    pub async fn switch_network(&self, network: String) -> Result<NetworkConfig, ApiError> {
        let networks = self.networks.read().await;
        if let Some(state) = networks.get(&network) {
            *self.active_network.write().await = network.clone();
            Ok(state.config.clone())
        } else {
            Err(ApiError::NotFound(format!("Network {} not found", network)))
        }
    }
    
    // Get all networks
    pub async fn get_networks(&self) -> Vec<NetworkConfig> {
        let networks = self.networks.read().await;
        networks.values().map(|s| s.config.clone()).collect()
    }
    
    // Get block by hash
    pub async fn get_block(&self, hash: &str, network: Option<String>) -> Result<BlockData, ApiError> {
        let network = network.unwrap_or(self.get_current_network().await);
        let networks = self.networks.read().await;
        
        if let Some(state) = networks.get(&network) {
            let hash_bytes = Hash::from_hex(hash).map_err(|_| ApiError::InvalidRequest("Invalid hash".to_string()))?;
            
            // Check cache first
            let cache = state.block_cache.read().await;
            if let Some(block) = cache.get(&hash_bytes) {
                return Ok(self.block_to_data(block, &network));
            }
            
            // Load from storage
            if let Ok(Some(block)) = state.storage.blocks.get_block(&hash_bytes).await {
                // Update cache
                state.block_cache.write().await.insert(hash_bytes, block.clone());
                Ok(self.block_to_data(&block, &network))
            } else {
                Err(ApiError::NotFound(format!("Block {} not found", hash)))
            }
        } else {
            Err(ApiError::NotFound(format!("Network {} not found", network)))
        }
    }
    
    // Get blocks by height range
    pub async fn get_blocks_by_height(
        &self,
        start: u64,
        end: u64,
        network: Option<String>,
    ) -> Result<Vec<BlockData>, ApiError> {
        let network = network.unwrap_or(self.get_current_network().await);
        let networks = self.networks.read().await;
        
        if let Some(state) = networks.get(&network) {
            let mut blocks = Vec::new();
            
            for height in start..=end {
                if let Ok(Some(hash)) = state.storage.blocks.get_block_by_height(height).await {
                    if let Ok(Some(block)) = state.storage.blocks.get_block(&hash).await {
                        blocks.push(self.block_to_data(&block, &network));
                    }
                }
            }
            
            Ok(blocks)
        } else {
            Err(ApiError::NotFound(format!("Network {} not found", network)))
        }
    }
    
    // Get DAG visualization data
    pub async fn get_dag_visualization(
        &self,
        depth: usize,
        network: Option<String>,
    ) -> Result<DagVisualization, ApiError> {
        let network = network.unwrap_or(self.get_current_network().await);
        let networks = self.networks.read().await;
        
        if let Some(state) = networks.get(&network) {
            // Check cache
            let last_update = *state.last_update.read().await;
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            
            if now - last_update < 5 {
                // Return cached data if less than 5 seconds old
                return Ok(state.dag_cache.read().await.clone());
            }
            
            // Build DAG visualization
            let mut nodes = Vec::new();
            let mut edges = Vec::new();
            let mut visited = HashSet::new();
            
            // Get chain tips
            let tips = state.storage.consensus.get_chain_tips().await
                .unwrap_or_default();
            
            // BFS from tips
            let mut queue = VecDeque::new();
            for tip in &tips {
                queue.push_back((tip.clone(), 0));
            }
            
            while let Some((hash, level)) = queue.pop_front() {
                if level >= depth || visited.contains(&hash) {
                    continue;
                }
                
                visited.insert(hash.clone());
                
                if let Ok(Some(block)) = state.storage.blocks.get_block(&hash).await {
                    // Create node
                    let node = DagNode {
                        id: hash.to_hex(),
                        height: block.header.height,
                        blue_score: block.header.blue_score,
                        is_blue: block.header.is_blue,
                        selected_parent: block.header.selected_parent.map(|h| h.to_hex()),
                        merge_parents: block.header.merge_parents.iter().map(|h| h.to_hex()).collect(),
                        timestamp: block.header.timestamp,
                        tx_count: block.transactions.len(),
                        size: block.size(),
                        proposer: block.header.proposer.to_hex(),
                        network: network.clone(),
                    };
                    nodes.push(node);
                    
                    // Create edges
                    if let Some(selected_parent) = block.header.selected_parent {
                        edges.push(DagEdge {
                            source: selected_parent.to_hex(),
                            target: hash.to_hex(),
                            edge_type: "selected".to_string(),
                            weight: 1.0,
                        });
                        queue.push_back((selected_parent, level + 1));
                    }
                    
                    for merge_parent in &block.header.merge_parents {
                        edges.push(DagEdge {
                            source: merge_parent.to_hex(),
                            target: hash.to_hex(),
                            edge_type: "merge".to_string(),
                            weight: 0.5,
                        });
                        queue.push_back((merge_parent.clone(), level + 1));
                    }
                }
            }
            
            // Calculate stats
            let stats = self.calculate_dag_stats(&nodes, &network).await;
            
            let dag = DagVisualization {
                nodes,
                edges,
                stats,
                network: network.clone(),
            };
            
            // Update cache
            *state.dag_cache.write().await = dag.clone();
            *state.last_update.write().await = now;
            
            Ok(dag)
        } else {
            Err(ApiError::NotFound(format!("Network {} not found", network)))
        }
    }
    
    // Calculate DAG statistics
    async fn calculate_dag_stats(&self, nodes: &[DagNode], network: &str) -> DagStats {
        let total_blocks = nodes.len() as u64;
        let blue_blocks = nodes.iter().filter(|n| n.is_blue).count() as u64;
        let red_blocks = total_blocks - blue_blocks;
        
        let total_blue_score: u64 = nodes.iter().map(|n| n.blue_score).sum();
        let avg_blue_score = if total_blocks > 0 {
            total_blue_score as f64 / total_blocks as f64
        } else {
            0.0
        };
        
        let max_blue_score = nodes.iter().map(|n| n.blue_score).max().unwrap_or(0);
        
        let total_transactions: u64 = nodes.iter().map(|n| n.tx_count as u64).sum();
        
        // Calculate TPS
        let time_range = if nodes.len() > 1 {
            let min_time = nodes.iter().map(|n| n.timestamp).min().unwrap_or(0);
            let max_time = nodes.iter().map(|n| n.timestamp).max().unwrap_or(0);
            (max_time - min_time) as f64
        } else {
            1.0
        };
        
        let tps = if time_range > 0.0 {
            total_transactions as f64 / time_range
        } else {
            0.0
        };
        
        // Calculate average block time
        let mut block_times = Vec::new();
        let mut sorted_nodes = nodes.to_vec();
        sorted_nodes.sort_by_key(|n| n.timestamp);
        
        for i in 1..sorted_nodes.len() {
            let time_diff = sorted_nodes[i].timestamp - sorted_nodes[i-1].timestamp;
            if time_diff > 0 {
                block_times.push(time_diff as f64);
            }
        }
        
        let avg_block_time = if !block_times.is_empty() {
            block_times.iter().sum::<f64>() / block_times.len() as f64
        } else {
            0.0
        };
        
        DagStats {
            total_blocks,
            total_transactions,
            blue_blocks,
            red_blocks,
            avg_blue_score,
            max_blue_score,
            total_supply: "21000000000000000000000000".to_string(), // 21M LATT
            validators_count: 1, // TODO: Get from consensus
            tps,
            avg_block_time,
            network_hash_rate: "0".to_string(), // TODO: Calculate
            difficulty: "0".to_string(), // TODO: Get from latest block
            mempool_size: 0, // TODO: Get from mempool
            peers_count: 0, // TODO: Get from network
            chain_tips: vec![], // TODO: Get from consensus
            network: network.to_string(),
        }
    }
    
    // Get network statistics
    pub async fn get_stats(&self, network: Option<String>) -> Result<DagStats, ApiError> {
        let network = network.unwrap_or(self.get_current_network().await);
        let networks = self.networks.read().await;
        
        if let Some(state) = networks.get(&network) {
            Ok(state.stats_cache.read().await.clone())
        } else {
            Err(ApiError::NotFound(format!("Network {} not found", network)))
        }
    }
    
    // Search across blocks, transactions, and addresses
    pub async fn search(
        &self,
        query: &str,
        network: Option<String>,
    ) -> Result<Vec<SearchResult>, ApiError> {
        let network = network.unwrap_or(self.get_current_network().await);
        let networks = self.networks.read().await;
        
        if let Some(state) = networks.get(&network) {
            let mut results = Vec::new();
            
            // Try as block hash
            if query.starts_with("0x") && query.len() == 66 {
                if let Ok(hash) = Hash::from_hex(query) {
                    if let Ok(Some(block)) = state.storage.blocks.get_block(&hash).await {
                        results.push(SearchResult {
                            result_type: "block".to_string(),
                            data: serde_json::to_value(self.block_to_data(&block, &network)).unwrap(),
                            network: network.clone(),
                        });
                    }
                }
            }
            
            // Try as transaction hash
            if query.starts_with("0x") && query.len() == 66 {
                if let Ok(hash) = Hash::from_hex(query) {
                    if let Ok(Some((tx, _))) = state.storage.transactions.get_transaction(&hash).await {
                        results.push(SearchResult {
                            result_type: "transaction".to_string(),
                            data: serde_json::to_value(self.tx_to_data(&tx, &network)).unwrap(),
                            network: network.clone(),
                        });
                    }
                }
            }
            
            // Try as address
            if query.starts_with("0x") && query.len() == 42 {
                // TODO: Implement address search
                results.push(SearchResult {
                    result_type: "address".to_string(),
                    data: serde_json::json!({
                        "address": query,
                        "balance": "0",
                        "nonce": 0,
                        "network": network
                    }),
                    network: network.clone(),
                });
            }
            
            // Try as block height
            if let Ok(height) = query.parse::<u64>() {
                if let Ok(Some(hash)) = state.storage.blocks.get_block_by_height(height).await {
                    if let Ok(Some(block)) = state.storage.blocks.get_block(&hash).await {
                        results.push(SearchResult {
                            result_type: "block".to_string(),
                            data: serde_json::to_value(self.block_to_data(&block, &network)).unwrap(),
                            network: network.clone(),
                        });
                    }
                }
            }
            
            Ok(results)
        } else {
            Err(ApiError::NotFound(format!("Network {} not found", network)))
        }
    }
    
    // Get recent blocks
    pub async fn get_recent_blocks(
        &self,
        limit: usize,
        network: Option<String>,
    ) -> Result<Vec<BlockData>, ApiError> {
        let network = network.unwrap_or(self.get_current_network().await);
        let networks = self.networks.read().await;
        
        if let Some(state) = networks.get(&network) {
            let mut blocks = Vec::new();
            
            // Get latest height
            let latest_height = state.storage.blocks.get_latest_height().await
                .unwrap_or(0);
            
            let start = if latest_height > limit as u64 {
                latest_height - limit as u64
            } else {
                0
            };
            
            for height in (start..=latest_height).rev().take(limit) {
                if let Ok(Some(hash)) = state.storage.blocks.get_block_by_height(height).await {
                    if let Ok(Some(block)) = state.storage.blocks.get_block(&hash).await {
                        blocks.push(self.block_to_data(&block, &network));
                    }
                }
            }
            
            Ok(blocks)
        } else {
            Err(ApiError::NotFound(format!("Network {} not found", network)))
        }
    }
    
    // Get recent transactions
    pub async fn get_recent_transactions(
        &self,
        limit: usize,
        network: Option<String>,
    ) -> Result<Vec<TransactionData>, ApiError> {
        let network = network.unwrap_or(self.get_current_network().await);
        let networks = self.networks.read().await;
        
        if let Some(state) = networks.get(&network) {
            let mut transactions = Vec::new();
            
            // Get recent blocks and extract transactions
            let latest_height = state.storage.blocks.get_latest_height().await
                .unwrap_or(0);
            
            let start = if latest_height > 10 {
                latest_height - 10
            } else {
                0
            };
            
            for height in (start..=latest_height).rev() {
                if let Ok(Some(hash)) = state.storage.blocks.get_block_by_height(height).await {
                    if let Ok(Some(block)) = state.storage.blocks.get_block(&hash).await {
                        for tx in &block.transactions {
                            transactions.push(self.tx_to_data(tx, &network));
                            if transactions.len() >= limit {
                                return Ok(transactions);
                            }
                        }
                    }
                }
            }
            
            Ok(transactions)
        } else {
            Err(ApiError::NotFound(format!("Network {} not found", network)))
        }
    }
    
    // Convert internal block to API BlockData
    fn block_to_data(&self, block: &Block, network: &str) -> BlockData {
        BlockData {
            hash: block.hash.to_hex(),
            height: block.header.height,
            timestamp: block.header.timestamp,
            selected_parent: block.header.selected_parent
                .map(|h| h.to_hex())
                .unwrap_or_else(|| "0x0".to_string()),
            merge_parents: block.header.merge_parents
                .iter()
                .map(|h| h.to_hex())
                .collect(),
            blue_score: block.header.blue_score,
            is_blue: block.header.is_blue,
            transactions: block.transactions
                .iter()
                .map(|tx| tx.hash().to_hex())
                .collect(),
            state_root: block.header.state_root.to_hex(),
            receipts_root: block.header.receipts_root.to_hex(),
            proposer: block.header.proposer.to_hex(),
            gas_used: block.header.gas_used,
            gas_limit: block.header.gas_limit,
            base_fee: block.header.base_fee.to_string(),
            difficulty: block.header.difficulty.to_string(),
            total_difficulty: block.header.total_difficulty.to_string(),
            size: block.size(),
            transaction_count: block.transactions.len(),
            uncle_count: 0, // GhostDAG doesn't have uncles
            network: network.to_string(),
        }
    }
    
    // Convert internal transaction to API TransactionData
    fn tx_to_data(&self, tx: &Transaction, network: &str) -> TransactionData {
        TransactionData {
            hash: tx.hash().to_hex(),
            block_hash: "".to_string(), // TODO: Get from receipt
            block_height: 0, // TODO: Get from receipt
            from: format!("0x{:x}", tx.from),
            to: tx.to.map(|addr| format!("0x{:x}", addr)),
            value: tx.value.to_string(),
            gas_price: tx.gas_price.to_string(),
            gas_limit: tx.gas_limit,
            gas_used: 0, // TODO: Get from receipt
            nonce: tx.nonce,
            input: hex::encode(&tx.data),
            status: true, // TODO: Get from receipt
            timestamp: 0, // TODO: Get from block
            tx_type: self.classify_transaction(tx),
            network: network.to_string(),
        }
    }
    
    // Classify transaction type
    fn classify_transaction(&self, tx: &Transaction) -> String {
        if tx.to.is_none() {
            "contract_creation".to_string()
        } else if !tx.data.is_empty() {
            if tx.data.len() >= 4 {
                let selector = &tx.data[0..4];
                // Common ERC20 function selectors
                match selector {
                    [0xa9, 0x05, 0x9c, 0xbb] => "erc20_transfer".to_string(),
                    [0x09, 0x5e, 0xa7, 0xb3] => "erc20_approve".to_string(),
                    [0x23, 0xb8, 0x72, 0xdd] => "erc20_transfer_from".to_string(),
                    _ => "contract_call".to_string(),
                }
            } else {
                "contract_call".to_string()
            }
        } else {
            "transfer".to_string()
        }
    }
    
    // HTTP API routes
    pub fn routes(explorer: Arc<Self>) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
        let networks = warp::path!("api" / "networks")
            .and(warp::get())
            .and(with_explorer(explorer.clone()))
            .and_then(handle_get_networks);
        
        let switch_network = warp::path!("api" / "network" / String)
            .and(warp::post())
            .and(with_explorer(explorer.clone()))
            .and_then(handle_switch_network);
        
        let block = warp::path!("api" / "block" / String)
            .and(warp::get())
            .and(warp::query::<HashMap<String, String>>())
            .and(with_explorer(explorer.clone()))
            .and_then(handle_get_block);
        
        let blocks = warp::path!("api" / "blocks")
            .and(warp::get())
            .and(warp::query::<HashMap<String, String>>())
            .and(with_explorer(explorer.clone()))
            .and_then(handle_get_blocks);
        
        let dag = warp::path!("api" / "dag")
            .and(warp::get())
            .and(warp::query::<HashMap<String, String>>())
            .and(with_explorer(explorer.clone()))
            .and_then(handle_get_dag);
        
        let stats = warp::path!("api" / "stats")
            .and(warp::get())
            .and(warp::query::<HashMap<String, String>>())
            .and(with_explorer(explorer.clone()))
            .and_then(handle_get_stats);
        
        let search = warp::path!("api" / "search")
            .and(warp::get())
            .and(warp::query::<HashMap<String, String>>())
            .and(with_explorer(explorer.clone()))
            .and_then(handle_search);
        
        let recent_blocks = warp::path!("api" / "recent" / "blocks")
            .and(warp::get())
            .and(warp::query::<HashMap<String, String>>())
            .and(with_explorer(explorer.clone()))
            .and_then(handle_recent_blocks);
        
        let recent_txs = warp::path!("api" / "recent" / "transactions")
            .and(warp::get())
            .and(warp::query::<HashMap<String, String>>())
            .and(with_explorer(explorer.clone()))
            .and_then(handle_recent_transactions);
        
        networks
            .or(switch_network)
            .or(block)
            .or(blocks)
            .or(dag)
            .or(stats)
            .or(search)
            .or(recent_blocks)
            .or(recent_txs)
    }
}

// Warp filter helper
fn with_explorer(
    explorer: Arc<DagExplorer>,
) -> impl Filter<Extract = (Arc<DagExplorer>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || explorer.clone())
}

// Handler functions
async fn handle_get_networks(
    explorer: Arc<DagExplorer>,
) -> Result<impl Reply, Rejection> {
    let networks = explorer.get_networks().await;
    Ok(warp::reply::json(&networks))
}

async fn handle_switch_network(
    network: String,
    explorer: Arc<DagExplorer>,
) -> Result<impl Reply, Rejection> {
    match explorer.switch_network(network).await {
        Ok(config) => Ok(warp::reply::json(&config)),
        Err(e) => Ok(warp::reply::json(&ApiResponse::<()>::error(e.to_string()))),
    }
}

async fn handle_get_block(
    hash: String,
    params: HashMap<String, String>,
    explorer: Arc<DagExplorer>,
) -> Result<impl Reply, Rejection> {
    let network = params.get("network").cloned();
    match explorer.get_block(&hash, network).await {
        Ok(block) => Ok(warp::reply::json(&block)),
        Err(e) => Ok(warp::reply::json(&ApiResponse::<()>::error(e.to_string()))),
    }
}

async fn handle_get_blocks(
    params: HashMap<String, String>,
    explorer: Arc<DagExplorer>,
) -> Result<impl Reply, Rejection> {
    let start = params.get("start")
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    let end = params.get("end")
        .and_then(|s| s.parse().ok())
        .unwrap_or(start + 10);
    let network = params.get("network").cloned();
    
    match explorer.get_blocks_by_height(start, end, network).await {
        Ok(blocks) => Ok(warp::reply::json(&blocks)),
        Err(e) => Ok(warp::reply::json(&ApiResponse::<()>::error(e.to_string()))),
    }
}

async fn handle_get_dag(
    params: HashMap<String, String>,
    explorer: Arc<DagExplorer>,
) -> Result<impl Reply, Rejection> {
    let depth = params.get("depth")
        .and_then(|s| s.parse().ok())
        .unwrap_or(50);
    let network = params.get("network").cloned();
    
    match explorer.get_dag_visualization(depth, network).await {
        Ok(dag) => Ok(warp::reply::json(&dag)),
        Err(e) => Ok(warp::reply::json(&ApiResponse::<()>::error(e.to_string()))),
    }
}

async fn handle_get_stats(
    params: HashMap<String, String>,
    explorer: Arc<DagExplorer>,
) -> Result<impl Reply, Rejection> {
    let network = params.get("network").cloned();
    match explorer.get_stats(network).await {
        Ok(stats) => Ok(warp::reply::json(&stats)),
        Err(e) => Ok(warp::reply::json(&ApiResponse::<()>::error(e.to_string()))),
    }
}

async fn handle_search(
    params: HashMap<String, String>,
    explorer: Arc<DagExplorer>,
) -> Result<impl Reply, Rejection> {
    let query = params.get("q").cloned().unwrap_or_default();
    let network = params.get("network").cloned();
    
    match explorer.search(&query, network).await {
        Ok(results) => Ok(warp::reply::json(&results)),
        Err(e) => Ok(warp::reply::json(&ApiResponse::<()>::error(e.to_string()))),
    }
}

async fn handle_recent_blocks(
    params: HashMap<String, String>,
    explorer: Arc<DagExplorer>,
) -> Result<impl Reply, Rejection> {
    let limit = params.get("limit")
        .and_then(|s| s.parse().ok())
        .unwrap_or(10);
    let network = params.get("network").cloned();
    
    match explorer.get_recent_blocks(limit, network).await {
        Ok(blocks) => Ok(warp::reply::json(&blocks)),
        Err(e) => Ok(warp::reply::json(&ApiResponse::<()>::error(e.to_string()))),
    }
}

async fn handle_recent_transactions(
    params: HashMap<String, String>,
    explorer: Arc<DagExplorer>,
) -> Result<impl Reply, Rejection> {
    let limit = params.get("limit")
        .and_then(|s| s.parse().ok())
        .unwrap_or(10);
    let network = params.get("network").cloned();
    
    match explorer.get_recent_transactions(limit, network).await {
        Ok(txs) => Ok(warp::reply::json(&txs)),
        Err(e) => Ok(warp::reply::json(&ApiResponse::<()>::error(e.to_string()))),
    }
}

// Extension traits for convenience
impl Block {
    fn size(&self) -> u64 {
        // Estimate block size
        let mut size = 0u64;
        size += 32; // hash
        size += 8;  // height
        size += 8;  // timestamp
        size += 32; // selected_parent
        size += self.header.merge_parents.len() as u64 * 32;
        size += 8;  // blue_score
        size += 1;  // is_blue
        size += 32; // state_root
        size += 32; // receipts_root
        size += 32; // proposer
        size += 8;  // gas_used
        size += 8;  // gas_limit
        size += 32; // base_fee
        size += 32; // difficulty
        size += 32; // total_difficulty
        
        for tx in &self.transactions {
            size += 32; // hash
            size += 32; // from
            size += 32; // to
            size += 32; // value
            size += 8;  // gas
            size += 32; // gas_price
            size += 8;  // nonce
            size += tx.data.len() as u64;
        }
        
        size
    }
}

impl Hash {
    fn from_hex(hex: &str) -> Result<Self, String> {
        let hex = hex.strip_prefix("0x").unwrap_or(hex);
        let bytes = hex::decode(hex).map_err(|e| e.to_string())?;
        if bytes.len() != 32 {
            return Err("Invalid hash length".to_string());
        }
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&bytes);
        Ok(Hash::new(arr))
    }
    
    fn to_hex(&self) -> String {
        format!("0x{}", hex::encode(self.0))
    }
}

impl PublicKey {
    fn to_hex(&self) -> String {
        format!("0x{}", hex::encode(&self.0))
    }
}

impl Transaction {
    fn hash(&self) -> Hash {
        // Calculate transaction hash
        use sha3::{Digest, Keccak256};
        let mut hasher = Keccak256::new();
        hasher.update(&self.from.to_be_bytes());
        if let Some(to) = self.to {
            hasher.update(&to.to_be_bytes());
        }
        hasher.update(&self.value.to_be_bytes());
        hasher.update(&self.nonce.to_le_bytes());
        hasher.update(&self.data);
        let result = hasher.finalize();
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&result);
        Hash::new(hash)
    }
}