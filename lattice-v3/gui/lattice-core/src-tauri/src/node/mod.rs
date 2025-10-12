use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info, warn};

// Core blockchain components - use what's actually available
use lattice_consensus::{
    types::{Block, BlockHeader, Hash, PublicKey, Signature, VrfProof},
    DagStore, GhostDag, GhostDagParams,
};
use lattice_execution::{state::StateDB, Executor};
use lattice_network::peer::{Direction as PeerDirection, PeerId, PeerState as NetPeerState};
use lattice_network::NetworkMessage;
use lattice_network::{PeerManager, PeerManagerConfig};
use lattice_sequencer::mempool::{Mempool, MempoolConfig};
use lattice_storage::StorageManager;
// use lattice_api::{RpcServer, RpcConfig}; // TODO: Fix mempool type mismatch
use crate::sync::iterative_sync::{IterativeSyncManager, SyncConfig};
use crate::wallet::WalletManager;
use sha3::{Digest, Sha3_256};
use tokio::task::JoinHandle;

/// Manages the embedded Lattice node
pub struct NodeManager {
    node: Arc<RwLock<Option<LatticeNode>>>,
    config: Arc<RwLock<NodeConfig>>,
    storage: Arc<RwLock<Option<Arc<StorageManager>>>>,
    ghostdag: Arc<RwLock<Option<Arc<GhostDag>>>>,
    sync_manager: Arc<RwLock<Option<Arc<IterativeSyncManager>>>>,
    reward_address: Arc<RwLock<Option<String>>>,
    wallet_manager: Arc<RwLock<Option<Arc<WalletManager>>>>,
}

impl NodeManager {
    pub fn new() -> Result<Self> {
        let config = NodeConfig::load_or_default()?;
        Ok(Self {
            node: Arc::new(RwLock::new(None)),
            config: Arc::new(RwLock::new(config)),
            storage: Arc::new(RwLock::new(None)),
            ghostdag: Arc::new(RwLock::new(None)),
            sync_manager: Arc::new(RwLock::new(None)),
            reward_address: Arc::new(RwLock::new(None)),
            wallet_manager: Arc::new(RwLock::new(None)),
        })
    }

    pub async fn attach_wallet_manager(&self, wallet: Arc<WalletManager>) {
        *self.wallet_manager.write().await = Some(wallet);
    }

    pub async fn start(&self) -> Result<()> {
        info!("Starting Lattice node");
        let mut node_guard = self.node.write().await;
        if node_guard.is_some() {
            return Err(anyhow::anyhow!("Node is already running"));
        }

        let mut config = self.config.read().await.clone();

        // If we're in testnet mode, ensure we have the right configuration
        if config.network == "testnet" {
            info!("Applying testnet configuration override");
            config.configure_for_testnet();
            // Update the shared config
            *self.config.write().await = config.clone();
            // Save the corrected config
            let _ = config.save();
        }

        // Initialize basic components with simplified setup
        let storage_path = PathBuf::from(&config.data_dir).join("chain");
        std::fs::create_dir_all(&storage_path)?;

        // Force clean up any existing lock files before starting
        let lock_file = storage_path.join("LOCK");
        if lock_file.exists() {
            warn!("Found existing LOCK file, removing it");
            match std::fs::remove_file(&lock_file) {
                Ok(_) => info!("Removed old LOCK file"),
                Err(e) => {
                    error!(
                        "Failed to remove LOCK file: {}. Trying to kill any zombie processes...",
                        e
                    );
                    // Try to find and kill any processes holding the lock
                    let _ = std::process::Command::new("lsof").arg(&lock_file).output();
                }
            }
        }

        // Also clean any other lock-related files
        if let Ok(entries) = std::fs::read_dir(&storage_path) {
            for entry in entries.flatten() {
                if let Some(name) = entry.file_name().to_str() {
                    if name.contains("LOCK") || name.contains(".lock") {
                        let _ = std::fs::remove_file(entry.path());
                        info!("Cleaned up lock file: {}", name);
                    }
                }
            }
        }

        let storage = Arc::new(StorageManager::new(
            storage_path.clone(),
            lattice_storage::pruning::PruningConfig {
                keep_blocks: 10000,
                keep_states: 1000,
                auto_prune: true,
                batch_size: 100,
                interval: std::time::Duration::from_secs(3600),
            },
        )?);

        // Create simplified GhostDAG setup
        let ghostdag_params = GhostDagParams {
            k: config.consensus.k_parameter,
            max_parents: 10,
            pruning_window: config.consensus.pruning_window,
            finality_depth: config.consensus.finality_depth,
            max_blue_score_diff: 1000,
        };

        let dag_store = Arc::new(DagStore::new());
        let ghostdag = Arc::new(GhostDag::new(ghostdag_params, dag_store.clone()));

        // Initialize execution environment with simplified setup
        let state_db = Arc::new(StateDB::new());
        let executor = Arc::new(Executor::new(state_db.clone()));

        // Initialize mempool from config
        let cfg_mempool = &config.mempool;
        let mempool_config = MempoolConfig {
            min_gas_price: cfg_mempool.min_gas_price,
            max_per_sender: cfg_mempool.max_per_sender,
            allow_replacement: cfg_mempool.allow_replacement,
            chain_id: cfg_mempool.chain_id,
            max_size: cfg_mempool.max_size,
            replacement_factor: cfg_mempool.replacement_factor,
            require_valid_signature: cfg_mempool.require_valid_signature,
            tx_expiry_secs: cfg_mempool.tx_expiry_secs,
        };
        let mempool = Arc::new(RwLock::new(Mempool::new(mempool_config)));

        // Initialize network manager and start real transport listener
        let peer_config = PeerManagerConfig {
            max_peers: config.max_peers,
            max_inbound: config.max_peers / 2,
            max_outbound: config.max_peers / 2,
            ban_duration: std::time::Duration::from_secs(3600),
            peer_timeout: std::time::Duration::from_secs(30),
            score_threshold: -100,
        };

        let peer_manager = Arc::new(PeerManager::new(peer_config));

        // Initialize iterative sync manager to avoid stack overflow
        let sync_config = SyncConfig {
            batch_size: 50,      // Small batches for GUI
            max_queue_size: 200, // Limit memory usage
            sync_interval: 5,
            max_retries: 3,
            sparse_sync: true, // Use sparse sync initially
            sparse_step: 10,
        };
        let sync_manager = Arc::new(IterativeSyncManager::new(
            storage.clone(),
            peer_manager.clone(),
            Some(sync_config),
        ));

        if config.enable_network {
            // Head info
            let head_height = storage.blocks.get_latest_height().unwrap_or(0);
            let head_hash = if head_height > 0 {
                storage
                    .blocks
                    .get_block_by_height(head_height)
                    .ok()
                    .flatten()
                    .unwrap_or_default()
            } else {
                Hash::default()
            };
            let listen_addr: std::net::SocketAddr =
                format!("0.0.0.0:{}", config.p2p_port).parse().unwrap();
            let network_id = config.mempool.chain_id as u32;
            let genesis_hash = {
                let gh = storage.blocks.get_block_by_height(0).ok().flatten();
                gh.unwrap_or_default()
            };
            // Subscribe to incoming messages and route them
            let (in_tx, mut in_rx) = tokio::sync::mpsc::channel::<(
                lattice_network::peer::PeerId,
                lattice_network::NetworkMessage,
            )>(512);
            peer_manager.set_incoming(in_tx).await;
            let pm_for_listener = peer_manager.clone();
            let storage_for_listener = storage.clone();
            let _ghostdag_for_listener = ghostdag.clone();
            let sync_manager_for_listener = sync_manager.clone();
            let mempool_for_listener = mempool.clone();
            let config_for_listener = Arc::new(RwLock::new(config.clone()));
            tokio::spawn(async move {
                use lattice_network::{protocol::PeerAddress, NetworkMessage};
                use lattice_sequencer::mempool::TxClass;
                while let Some((peer_id, msg)) = in_rx.recv().await {
                    match msg {
                        NetworkMessage::NewBlock { block } => {
                            // Dedup by storage
                            if !storage_for_listener
                                .blocks
                                .has_block(&block.header.block_hash)
                                .unwrap_or(false)
                            {
                                info!(
                                    "Received new block at height {} with hash {:?}",
                                    block.header.height, block.header.block_hash
                                );

                                // Use sync manager to handle the block (avoids recursion)
                                if let Err(e) = sync_manager_for_listener
                                    .handle_blocks(vec![block.clone()])
                                    .await
                                {
                                    tracing::warn!(
                                        "Failed to handle block via sync manager: {}",
                                        e
                                    );
                                }

                                // Re-broadcast to others
                                let _ = pm_for_listener
                                    .broadcast(&NetworkMessage::NewBlock { block })
                                    .await;
                            } else {
                                tracing::debug!("Block already exists, skipping");
                            }
                        }
                        NetworkMessage::GetBlocks { from, count, .. } => {
                            // Serve blocks AFTER the 'from' block
                            let mut blocks = Vec::new();
                            if let Ok(Some(start_block)) =
                                storage_for_listener.blocks.get_block(&from)
                            {
                                // Start from the NEXT block after the requested one
                                let start_h = start_block.header.height + 1;
                                let end_h = start_h.saturating_add(count as u64);
                                let mut h = start_h;
                                while h < end_h {
                                    if let Ok(Some(hash_h)) =
                                        storage_for_listener.blocks.get_block_by_height(h)
                                    {
                                        if let Ok(Some(b)) =
                                            storage_for_listener.blocks.get_block(&hash_h)
                                        {
                                            blocks.push(b);
                                        }
                                    }
                                    if blocks.len() as u32 >= count {
                                        break;
                                    }
                                    h += 1;
                                }
                                info!(
                                    "Serving {} blocks starting from height {}",
                                    blocks.len(),
                                    start_h
                                );
                            } else if from == Hash::new([0u8; 32]) {
                                // Special case: requesting from genesis (zero hash)
                                let mut h = 0u64;
                                while h < count as u64 {
                                    if let Ok(Some(hash_h)) =
                                        storage_for_listener.blocks.get_block_by_height(h)
                                    {
                                        if let Ok(Some(b)) =
                                            storage_for_listener.blocks.get_block(&hash_h)
                                        {
                                            blocks.push(b);
                                        }
                                    }
                                    if blocks.len() as u32 >= count {
                                        break;
                                    }
                                    h += 1;
                                }
                                info!("Serving {} blocks from genesis", blocks.len());
                            }
                            let _ = pm_for_listener
                                .send_to_peers(
                                    &[peer_id.clone()],
                                    &NetworkMessage::Blocks { blocks },
                                )
                                .await;
                        }
                        NetworkMessage::GetHeaders { from, count } => {
                            let mut headers = Vec::new();
                            if let Ok(Some(start_block)) =
                                storage_for_listener.blocks.get_block(&from)
                            {
                                let start_h = start_block.header.height;
                                let end_h = start_h.saturating_add(count as u64);
                                let mut h = start_h;
                                while h <= end_h {
                                    if let Ok(Some(hash_h)) =
                                        storage_for_listener.blocks.get_block_by_height(h)
                                    {
                                        if let Ok(Some(b)) =
                                            storage_for_listener.blocks.get_block(&hash_h)
                                        {
                                            headers.push(b.header.clone());
                                        }
                                    }
                                    if headers.len() as u32 >= count {
                                        break;
                                    }
                                    h += 1;
                                }
                            }
                            let _ = pm_for_listener
                                .send_to_peers(
                                    &[peer_id.clone()],
                                    &NetworkMessage::Headers { headers },
                                )
                                .await;
                        }
                        NetworkMessage::Blocks { blocks } => {
                            info!("Received {} blocks from peer", blocks.len());
                            // Use sync manager to handle blocks iteratively (avoids stack overflow)
                            if let Err(e) = sync_manager_for_listener.handle_blocks(blocks).await {
                                warn!("Failed to handle blocks via sync manager: {}", e);
                            }
                        }
                        NetworkMessage::NewTransaction { transaction } => {
                            let already = mempool_for_listener
                                .read()
                                .await
                                .contains(&transaction.hash)
                                .await;
                            if !already {
                                let _ = mempool_for_listener
                                    .read()
                                    .await
                                    .add_transaction(transaction.clone(), TxClass::Standard)
                                    .await;
                                let _ = pm_for_listener
                                    .broadcast(&NetworkMessage::NewTransaction { transaction })
                                    .await;
                            }
                        }
                        NetworkMessage::GetTransactions { hashes } => {
                            let mut txs = Vec::new();
                            for h in hashes {
                                if let Some(tx) =
                                    mempool_for_listener.read().await.get_transaction(&h).await
                                {
                                    txs.push(tx);
                                }
                            }
                            let _ = pm_for_listener
                                .send_to_peers(
                                    &[peer_id.clone()],
                                    &NetworkMessage::Transactions { transactions: txs },
                                )
                                .await;
                        }
                        NetworkMessage::GetPeers => {
                            let mut peers = Vec::new();
                            for p in pm_for_listener.get_all_peers() {
                                let info = p.info.read().await;
                                peers.push(PeerAddress {
                                    id: info.id.0.clone(),
                                    addr: info.addr.to_string(),
                                    last_seen: 0,
                                    score: info.score,
                                });
                            }
                            let _ = pm_for_listener
                                .send_to_peers(&[peer_id.clone()], &NetworkMessage::Peers { peers })
                                .await;
                        }
                        NetworkMessage::Peers { peers } => {
                            // Only auto-connect to discovered peers in devnet with discovery enabled
                            let cfg = config_for_listener.read().await;
                            if cfg.network == "devnet" && cfg.discovery {
                                // Attempt to connect to new peers
                                for pa in peers {
                                    if let Ok(addr) = pa.addr.parse() {
                                        let _ = pm_for_listener
                                            .clone()
                                            .connect_bootnode_real(
                                                Some(lattice_network::peer::PeerId::new(pa.id)),
                                                addr,
                                                network_id,
                                                genesis_hash,
                                                head_height,
                                                head_hash,
                                            )
                                            .await;
                                    }
                                }
                            } else {
                                info!(
                                    "Ignoring {} discovered peers (discovery disabled)",
                                    peers.len()
                                );
                            }
                        }
                        _ => {}
                    }
                }
            });
            let pm_for_listener = peer_manager.clone();
            pm_for_listener
                .start_listener(
                    listen_addr,
                    network_id,
                    genesis_hash,
                    head_height,
                    head_hash,
                )
                .await
                .ok();
            // Periodic discovery: ask peers for peers (only in devnet with discovery enabled)
            if config.network == "devnet" && config.discovery {
                let pm_for_peers = peer_manager.clone();
                tokio::spawn(async move {
                    loop {
                        let _ = pm_for_peers
                            .broadcast(&lattice_network::NetworkMessage::GetPeers)
                            .await;
                        tokio::time::sleep(std::time::Duration::from_secs(30)).await;
                    }
                });
            } else {
                info!("Peer discovery disabled for network: {}", config.network);
            }
        }

        // Connect to configured bootnodes and start syncing
        if config.enable_network {
            let head_height = storage.blocks.get_latest_height().unwrap_or(0);
            let head_hash = if head_height > 0 {
                storage
                    .blocks
                    .get_block_by_height(head_height)
                    .ok()
                    .flatten()
                    .unwrap_or_default()
            } else {
                Hash::default()
            };
            let network_id = config.mempool.chain_id as u32;
            let genesis_hash = storage
                .blocks
                .get_block_by_height(0)
                .ok()
                .flatten()
                .unwrap_or_default();

            info!("Connecting to {} bootnodes", config.bootnodes.len());
            for entry in &config.bootnodes {
                info!("Processing bootnode entry: {}", entry);
                if let Some((peer_id, addr)) = parse_bootnode(entry) {
                    info!("Attempting to connect to bootnode at {}", addr);
                    let pm = peer_manager.clone();
                    tokio::spawn(async move {
                        match pm
                            .connect_bootnode_real(
                                Some(peer_id),
                                addr,
                                network_id,
                                genesis_hash,
                                head_height,
                                head_hash,
                            )
                            .await
                        {
                            Ok(_) => info!("Successfully connected to bootnode at {}", addr),
                            Err(e) => error!("Failed to connect to bootnode at {}: {}", addr, e),
                        }
                    });
                } else {
                    warn!(
                        "Invalid bootnode entry: {} (expected peerId@ip:port or ip:port)",
                        entry
                    );
                }
            }

            // Start block synchronization task
            let pm_for_sync = peer_manager.clone();
            let storage_for_sync = storage.clone();
            tokio::spawn(async move {
                info!("Starting block sync task");

                // Wait for peers to connect
                tokio::time::sleep(std::time::Duration::from_secs(3)).await;

                loop {
                    // Get current height
                    let current_height = storage_for_sync.blocks.get_latest_height().unwrap_or(0);
                    info!("Sync: Current height is {}", current_height);

                    // Check if we have peers
                    let (peer_count, _, _) = pm_for_sync.get_peer_counts().await;
                    if peer_count == 0 {
                        warn!("Sync: No peers connected, skipping sync");
                        tokio::time::sleep(std::time::Duration::from_secs(10)).await;
                        continue;
                    }

                    info!("Sync: Have {} peers, requesting blocks", peer_count);

                    // Request blocks from peers starting AFTER our current height
                    if current_height > 0 {
                        // We have some blocks, request the next ones
                        if let Ok(Some(current_hash)) =
                            storage_for_sync.blocks.get_block_by_height(current_height)
                        {
                            info!(
                                "Sync: Have blocks up to height {}, requesting next batch",
                                current_height
                            );
                            // GetBlocks with a hash requests blocks AFTER that block
                            let _ = pm_for_sync
                                .broadcast(&NetworkMessage::GetBlocks {
                                    from: current_hash,
                                    count: 100,
                                    step: 1, // Get every block (no sparse download)
                                })
                                .await;
                        }
                    } else {
                        // We don't have any blocks, request from genesis
                        info!("Sync: No blocks found, requesting from genesis");
                        // Use a zero hash to request from the beginning
                        let genesis_hash = Hash::new([0u8; 32]);
                        let _ = pm_for_sync
                            .broadcast(&NetworkMessage::GetBlocks {
                                from: genesis_hash,
                                count: 100,
                                step: 1, // Get every block
                            })
                            .await;
                    }

                    // Wait before next sync attempt
                    tokio::time::sleep(std::time::Duration::from_secs(10)).await;
                }
            });
        }

        // Create genesis block if needed
        let latest_height = storage.blocks.get_latest_height().unwrap_or(0);
        if latest_height == 0 {
            info!("Creating genesis block");
            let genesis = create_genesis_block();
            storage.blocks.put_block(&genesis)?;
            dag_store.store_block(genesis.clone()).await?;
            // Add genesis as initial tip
            ghostdag.add_block(&genesis).await?;
            info!(
                "Genesis block created with hash: {:?}",
                genesis.header.block_hash
            );
        } else {
            // Load existing blocks into DAG
            info!("Loading existing blocks into DAG");
            for height in 0..=latest_height {
                if let Ok(Some(block_hash)) = storage.blocks.get_block_by_height(height) {
                    if let Ok(Some(block)) = storage.blocks.get_block(&block_hash) {
                        let _ = dag_store.store_block(block.clone()).await;
                        let _ = ghostdag.add_block(&block).await;
                    }
                }
            }
        }

        // Store references for DAG manager before moving
        *self.storage.write().await = Some(storage.clone());
        *self.ghostdag.write().await = Some(ghostdag.clone());
        *self.sync_manager.write().await = Some(sync_manager.clone());

        // Start the sync manager
        if config.enable_network {
            info!("Starting iterative sync manager");
            if let Err(e) = sync_manager.start_sync().await {
                warn!("Failed to start sync manager: {}", e);
            }
        }

        // Get reward address for block production
        let reward_address = self.reward_address.read().await.clone();

        // Create node instance
        let running = Arc::new(RwLock::new(true));

        // Only start block producer if:
        // 1. We have a reward address
        // 2. We're NOT in testnet/mainnet mode (only for devnet)
        let should_produce_blocks = reward_address.is_some() && config.network == "devnet";

        let (block_producer_handle, block_producer_running) = if should_produce_blocks {
            if let Some(addr) = reward_address {
                info!(
                    "Starting block producer for devnet with reward address {}",
                    addr
                );
                let wm = self.wallet_manager.read().await.clone();
                let producer = crate::block_producer::BlockProducer::new(
                    ghostdag.clone(),
                    mempool.clone(),
                    executor.clone(),
                    storage.clone(),
                    Arc::new(RwLock::new(Some(addr))),
                    wm,
                    if config.enable_network {
                        Some(peer_manager.clone())
                    } else {
                        None
                    },
                );
                let running_flag = producer.running_flag();
                let handle = producer.start().await.ok();
                (handle, Some(running_flag))
            } else {
                (None, None)
            }
        } else {
            if config.network != "devnet" {
                info!(
                    "Block production disabled - syncing from {} network",
                    config.network
                );
            } else {
                warn!("No reward address set, block production disabled");
            }
            (None, None)
        };

        // TODO: Start RPC server once mempool type mismatch is resolved
        // The RPC server expects Arc<Mempool> but GUI uses Arc<RwLock<Mempool>>
        let rpc_handle = None;

        let node = LatticeNode {
            storage,
            executor,
            mempool,
            ghostdag,
            peer_manager,
            _running: running,
            start_time: std::time::Instant::now(),
            block_producer_handle,
            block_producer_running,
            rpc_handle,
        };

        *node_guard = Some(node);

        info!("Lattice node started");

        Ok(())
    }

    pub async fn stop(&self) -> Result<()> {
        // First clear external references to storage
        *self.storage.write().await = None;
        *self.ghostdag.write().await = None;

        let mut node_guard = self.node.write().await;
        if let Some(mut node) = node_guard.take() {
            info!("Stopping Lattice node");
            *node._running.write().await = false;

            // Stop block producer loop cleanly
            if let Some(rf) = node.block_producer_running.as_ref() {
                *rf.write().await = false;
            }

            // Stop block producer
            if let Some(handle) = node.block_producer_handle.take() {
                handle.abort();
                // Wait a bit for thread to stop
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            }

            // Stop RPC server (when implemented)
            if let Some(handle) = node.rpc_handle.take() {
                handle.abort();
            }

            // Explicitly drop all components to release resources
            drop(node.peer_manager);
            drop(node.mempool);
            drop(node.executor);
            drop(node.ghostdag);
            drop(node.storage); // This should release the database

            // Wait for resources to be released
            tokio::time::sleep(std::time::Duration::from_millis(200)).await;

            // Force clean up lock files
            let config = self.config.read().await;
            let chain_dir = std::path::Path::new(&config.data_dir).join("chain");

            // Remove LOCK file
            let lock_file = chain_dir.join("LOCK");
            if lock_file.exists() {
                match std::fs::remove_file(&lock_file) {
                    Ok(_) => info!("Cleaned up lock file: {:?}", lock_file),
                    Err(e) => warn!("Failed to remove lock file: {}", e),
                }
            }

            // Also try to remove any other lock-related files
            if let Ok(entries) = std::fs::read_dir(&chain_dir) {
                for entry in entries.flatten() {
                    if let Some(name) = entry.file_name().to_str() {
                        if name.contains("LOCK") || name.contains(".lock") {
                            let _ = std::fs::remove_file(entry.path());
                        }
                    }
                }
            }

            info!("Lattice node stopped and cleaned up");
        }

        Ok(())
    }

    pub async fn get_storage(&self) -> Option<Arc<StorageManager>> {
        self.storage.read().await.clone()
    }

    pub async fn get_ghostdag(&self) -> Option<Arc<GhostDag>> {
        self.ghostdag.read().await.clone()
    }

    /// Expose mempool for local submissions
    pub async fn get_mempool(&self) -> Option<Arc<RwLock<Mempool>>> {
        self.node
            .read()
            .await
            .as_ref()
            .map(|node| node.mempool.clone())
    }

    /// Return current peer summaries
    pub async fn get_peers_summary(&self) -> Vec<PeerSummary> {
        if let Some(node) = self.node.read().await.as_ref() {
            let peers = node.peer_manager.get_all_peers();
            let mut out = Vec::with_capacity(peers.len());
            for p in peers {
                let info = p.info.read().await;
                out.push(PeerSummary {
                    id: info.id.0.clone(),
                    addr: info.addr.to_string(),
                    direction: match info.direction {
                        PeerDirection::Inbound => "inbound".into(),
                        PeerDirection::Outbound => "outbound".into(),
                    },
                    state: match info.state {
                        NetPeerState::Connecting => "connecting".into(),
                        NetPeerState::Handshaking => "handshaking".into(),
                        NetPeerState::Connected => "connected".into(),
                        NetPeerState::Disconnecting => "disconnecting".into(),
                        NetPeerState::Disconnected => "disconnected".into(),
                    },
                    score: info.score,
                    last_seen_secs: info.last_seen.elapsed().as_secs(),
                });
            }
            out
        } else {
            vec![]
        }
    }

    /// Connect to all configured bootnodes now (if network is enabled)
    pub async fn connect_bootnodes_now(&self) -> Result<usize> {
        let node_guard = self.node.read().await;
        let node = node_guard
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Node is not running"))?;
        if !self.config.read().await.enable_network {
            return Err(anyhow::anyhow!("Network is disabled in config"));
        }

        let cfg = self.config.read().await.clone();
        let bootnodes = cfg.bootnodes.clone();
        if bootnodes.is_empty() {
            return Ok(0);
        }
        // Use chainId as network id for handshake (must match core node)
        let network_id = cfg.mempool.chain_id as u32;
        let head_height = node.storage.blocks.get_latest_height().unwrap_or(0);
        let head_hash = if head_height > 0 {
            node.storage
                .blocks
                .get_block_by_height(head_height)
                .ok()
                .flatten()
                .unwrap_or_default()
        } else {
            Hash::default()
        };
        let genesis_hash = node
            .storage
            .blocks
            .get_block_by_height(0)
            .ok()
            .flatten()
            .unwrap_or_default();

        let pm = node.peer_manager.clone();
        let mut ok = 0usize;
        for entry in bootnodes {
            if let Some((pid, addr)) = parse_bootnode(&entry) {
                if pm
                    .clone()
                    .connect_bootnode_real(
                        Some(pid),
                        addr,
                        network_id,
                        genesis_hash,
                        head_height,
                        head_hash,
                    )
                    .await
                    .is_ok()
                {
                    ok += 1;
                }
            }
        }
        Ok(ok)
    }

    /// Connect to a single peer specified by bootnode-style entry
    pub async fn connect_peer(&self, entry: &str) -> Result<String> {
        let node_guard = self.node.read().await;
        let node = node_guard
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Node is not running"))?;
        if !self.config.read().await.enable_network {
            return Err(anyhow::anyhow!("Network is disabled in config"));
        }
        let (peer_id, addr) =
            parse_bootnode(entry).ok_or_else(|| anyhow::anyhow!("Invalid peer address format"))?;
        let cfg = self.config.read().await.clone();
        let network_id = cfg.mempool.chain_id as u32;
        let head_height = node.storage.blocks.get_latest_height().unwrap_or(0);
        let head_hash = if head_height > 0 {
            node.storage
                .blocks
                .get_block_by_height(head_height)
                .ok()
                .flatten()
                .unwrap_or_default()
        } else {
            Hash::default()
        };
        let genesis_hash = node
            .storage
            .blocks
            .get_block_by_height(0)
            .ok()
            .flatten()
            .unwrap_or_default();
        node.peer_manager
            .clone()
            .connect_bootnode_real(
                Some(peer_id.clone()),
                addr,
                network_id,
                genesis_hash,
                head_height,
                head_hash,
            )
            .await?;
        Ok(peer_id.0)
    }

    /// Disconnect the specified peer
    pub async fn disconnect_peer(&self, peer_id: &str) -> Result<()> {
        if let Some(node) = self.node.read().await.as_ref() {
            let pid = PeerId(peer_id.to_string());
            node.peer_manager.remove_peer(&pid).await;
            Ok(())
        } else {
            Err(anyhow::anyhow!("Node is not running"))
        }
    }

    /// Read current bootnodes from config
    pub async fn get_bootnodes(&self) -> Vec<String> {
        self.config.read().await.bootnodes.clone()
    }

    /// Add a bootnode entry to config (requires node stopped)
    pub async fn add_bootnode_entry(&self, entry: &str) -> Result<()> {
        if self.node.read().await.is_some() {
            return Err(anyhow::anyhow!(
                "Cannot modify bootnodes while node is running"
            ));
        }
        if parse_bootnode(entry).is_none() {
            return Err(anyhow::anyhow!("Invalid bootnode format"));
        }
        let mut cfg = self.config.read().await.clone();
        if !cfg.bootnodes.contains(&entry.to_string()) {
            cfg.bootnodes.push(entry.to_string());
        }
        self.update_config(cfg).await
    }

    /// Remove a bootnode entry from config (requires node stopped)
    pub async fn remove_bootnode_entry(&self, entry: &str) -> Result<()> {
        if self.node.read().await.is_some() {
            return Err(anyhow::anyhow!(
                "Cannot modify bootnodes while node is running"
            ));
        }
        let mut cfg = self.config.read().await.clone();
        cfg.bootnodes.retain(|e| e != entry);
        self.update_config(cfg).await
    }

    /// Convert consensus PublicKey to wallet-style address (keccak256(pubkey)[12..])
    fn pk_to_address_hex(pk: &PublicKey) -> String {
        use sha3::{Digest, Keccak256};
        let hash = Keccak256::digest(pk.as_bytes());
        format!("0x{}", hex::encode(&hash[12..]))
    }

    /// Try to interpret a tx.to public key as either a raw 20-byte address (left-padded in 32 bytes)
    /// or as a real public key that requires keccak(pubkey)[12..]
    fn to_field_as_address_hex(to: &PublicKey) -> String {
        let bytes = to.as_bytes();
        // If the last 12 bytes are all zero, assume first 20 bytes are the address directly
        if bytes[20..].iter().all(|&b| b == 0) {
            format!("0x{}", hex::encode(&bytes[0..20]))
        } else {
            Self::pk_to_address_hex(to)
        }
    }

    /// Get pending and confirmed transactions for the given account address
    pub async fn get_account_activity(
        &self,
        address: &str,
        block_window: u64,
        limit: usize,
    ) -> Result<Vec<TxActivity>> {
        let mut activity: Vec<TxActivity> = Vec::new();
        let addr_lc = address.to_lowercase();

        // Snapshot handles from node and drop the lock to avoid holding across await
        let (storage, mempool) = {
            let guard = self.node.read().await;
            let node = match guard.as_ref() {
                Some(n) => n,
                None => return Ok(activity),
            };
            (node.storage.clone(), node.mempool.clone())
        };

        // Collect pending from mempool (outgoing and incoming)
        {
            let mempool_guard = mempool.read().await;
            let memtx = mempool_guard.get_transactions(1000).await; // coarse upper bound
            for tx in memtx {
                let from_addr = Self::pk_to_address_hex(&tx.from).to_lowercase();
                let to_addr = tx
                    .to
                    .as_ref()
                    .map(|p| Self::to_field_as_address_hex(p).to_lowercase());
                if from_addr == addr_lc || to_addr.as_deref() == Some(&addr_lc) {
                    let to_hex = tx.to.as_ref().map(Self::to_field_as_address_hex);
                    activity.push(TxActivity {
                        hash: hex::encode(tx.hash.as_bytes()),
                        from: Self::pk_to_address_hex(&tx.from),
                        to: to_hex,
                        value: tx.value.to_string(),
                        nonce: tx.nonce,
                        status: "pending".into(),
                        block_hash: None,
                        block_height: None,
                        timestamp: None,
                    });
                }
            }
        }

        // Collect confirmed from recent blocks (use receipts to surface status)
        let latest = storage.blocks.get_latest_height().unwrap_or(0);
        if latest > 0 {
            let start = latest.saturating_sub(block_window);
            let mut h = latest;
            while h >= start {
                if let Ok(Some(bh)) = storage.blocks.get_block_by_height(h) {
                    if let Ok(Some(block)) = storage.blocks.get_block(&bh) {
                        for tx in &block.transactions {
                            let from_addr = Self::pk_to_address_hex(&tx.from).to_lowercase();
                            let to_addr = tx
                                .to
                                .as_ref()
                                .map(|p| Self::to_field_as_address_hex(p).to_lowercase());
                            if from_addr == addr_lc || to_addr.as_deref() == Some(&addr_lc) {
                                let to_hex = tx.to.as_ref().map(Self::to_field_as_address_hex);
                                let status = match storage.transactions.get_receipt(&tx.hash) {
                                    Ok(Some(r)) => {
                                        if r.status {
                                            "confirmed"
                                        } else {
                                            "failed"
                                        }
                                    }
                                    _ => "confirmed",
                                };
                                activity.push(TxActivity {
                                    hash: hex::encode(tx.hash.as_bytes()),
                                    from: Self::pk_to_address_hex(&tx.from),
                                    to: to_hex,
                                    value: tx.value.to_string(),
                                    nonce: tx.nonce,
                                    status: status.into(),
                                    block_hash: Some(block.header.block_hash.to_hex()),
                                    block_height: Some(block.header.height),
                                    timestamp: Some(block.header.timestamp),
                                });
                            }
                        }
                    }
                }
                if h == 0 {
                    break;
                }
                h -= 1;
            }
        }

        // Sort by (timestamp desc, pending on top if no timestamp)
        activity.sort_by(|a, b| {
            let at = a.timestamp.unwrap_or(u64::MAX);
            let bt = b.timestamp.unwrap_or(u64::MAX);
            bt.cmp(&at)
        });

        // Deduplicate by hash, prefer pending first then confirmed latest
        let mut seen = std::collections::HashSet::new();
        let mut dedup: Vec<TxActivity> = Vec::new();
        for item in activity.into_iter() {
            if seen.insert(item.hash.clone()) {
                dedup.push(item);
            }
            if dedup.len() >= limit {
                break;
            }
        }
        Ok(dedup)
    }

    /// Get global tx overview: pending mempool count and tx count in latest block
    pub async fn get_tx_overview(&self) -> Result<TxOverview> {
        let mut pending = 0usize;
        let mut last_block = 0usize;

        if let Some(node) = self.node.read().await.as_ref() {
            pending = node
                .mempool
                .read()
                .await
                .get_transactions(10_000)
                .await
                .len();
            let latest = node.storage.blocks.get_latest_height().unwrap_or(0);
            if latest > 0 {
                if let Ok(Some(bh)) = node.storage.blocks.get_block_by_height(latest) {
                    if let Ok(Some(block)) = node.storage.blocks.get_block(&bh) {
                        last_block = block.transactions.len();
                    }
                }
            }
        }

        Ok(TxOverview {
            pending,
            last_block,
        })
    }

    /// Snapshot current mempool pending txs (best-effort, limited)
    pub async fn get_mempool_pending(&self, limit: usize) -> Result<Vec<PendingTx>> {
        if let Some(node) = self.node.read().await.as_ref() {
            let txs = node.mempool.read().await.get_transactions(limit).await;
            let mut out = Vec::new();
            for tx in txs {
                let from = Self::pk_to_address_hex(&tx.from);
                let to = tx.to.as_ref().map(Self::to_field_as_address_hex);
                out.push(PendingTx {
                    hash: hex::encode(tx.hash.as_bytes()),
                    from,
                    to,
                    value: tx.value.to_string(),
                    nonce: tx.nonce,
                });
            }
            return Ok(out);
        }
        Ok(vec![])
    }

    /// Compute observed balance over a recent window (incoming - outgoing)
    pub async fn get_observed_balance(&self, address: &str, block_window: u64) -> Result<String> {
        let addr_lc = address.to_lowercase();
        let storage = match self.node.read().await.as_ref() {
            Some(n) => n.storage.clone(),
            None => return Ok("0".to_string()),
        };
        let latest = storage.blocks.get_latest_height().unwrap_or(0);
        let mut incoming: u128 = 0;
        let mut outgoing: u128 = 0;
        if latest > 0 {
            let start = latest.saturating_sub(block_window);
            let mut h = latest;
            while h >= start {
                if let Ok(Some(bh)) = storage.blocks.get_block_by_height(h) {
                    if let Ok(Some(block)) = storage.blocks.get_block(&bh) {
                        for tx in &block.transactions {
                            let from_addr = Self::pk_to_address_hex(&tx.from).to_lowercase();
                            let to_addr = tx
                                .to
                                .as_ref()
                                .map(|p| Self::pk_to_address_hex(p).to_lowercase());
                            if to_addr.as_deref() == Some(&addr_lc) {
                                incoming = incoming.saturating_add(tx.value);
                            }
                            if from_addr == addr_lc {
                                outgoing = outgoing.saturating_add(tx.value);
                            }
                        }
                    }
                }
                if h == 0 {
                    break;
                }
                h -= 1;
            }
        }
        Ok(incoming.saturating_sub(outgoing).to_string())
    }

    pub async fn get_status(&self) -> Result<NodeStatus> {
        let node_guard = self.node.read().await;

        if let Some(node) = node_guard.as_ref() {
            let block_height = node.storage.blocks.get_latest_height().unwrap_or(0);
            let (total, _inbound, _outbound) = node.peer_manager.get_peer_counts().await;
            let peer_count = total;
            let tips = node.ghostdag.get_tips().await;
            let (last_hash, last_ts) = if block_height > 0 {
                match node.storage.blocks.get_block_by_height(block_height) {
                    Ok(Some(h)) => match node.storage.blocks.get_block(&h) {
                        Ok(Some(b)) => (
                            Some(hex::encode(b.header.block_hash.as_bytes())),
                            Some(b.header.timestamp),
                        ),
                        _ => (None, None),
                    },
                    _ => (None, None),
                }
            } else {
                (None, None)
            };

            // Derive a current blue score from the best tip when available
            let blue_score = if !tips.is_empty() {
                if let Ok(hash) = node.ghostdag.select_tip().await {
                    node.ghostdag.get_blue_score(&hash).await.unwrap_or(0)
                } else {
                    0
                }
            } else {
                0
            };

            Ok(NodeStatus {
                running: true,
                syncing: false,
                block_height,
                peer_count,
                network_id: self.config.read().await.network.clone(),
                version: env!("CARGO_PKG_VERSION").to_string(),
                uptime: node.get_uptime(),
                dag_tips: tips.len(),
                blue_score,
                last_block_hash: last_hash,
                last_block_timestamp: last_ts,
            })
        } else {
            Ok(NodeStatus {
                running: false,
                syncing: false,
                block_height: 0,
                peer_count: 0,
                network_id: self.config.read().await.network.clone(),
                version: env!("CARGO_PKG_VERSION").to_string(),
                uptime: 0,
                dag_tips: 0,
                blue_score: 0,
                last_block_hash: None,
                last_block_timestamp: None,
            })
        }
    }

    pub async fn update_config(&self, new_config: NodeConfig) -> Result<()> {
        if self.node.read().await.is_some() {
            return Err(anyhow::anyhow!(
                "Cannot update config while node is running"
            ));
        }

        new_config.validate()?;
        new_config.save()?;
        *self.config.write().await = new_config;

        Ok(())
    }

    pub async fn get_config(&self) -> NodeConfig {
        self.config.read().await.clone()
    }

    /// Set the reward address for block production rewards
    pub async fn set_reward_address(&self, address: String) {
        *self.reward_address.write().await = Some(address.clone());
        info!("Set reward address to: {}", address);
        // If node is already running and producer not started, start it now
        let mut node_guard = self.node.write().await;
        if let Some(node) = node_guard.as_mut() {
            if node.block_producer_handle.is_none() {
                let storage = node.storage.clone();
                let executor = node.executor.clone();
                let mempool = node.mempool.clone();
                let ghostdag = node.ghostdag.clone();
                let wallet_manager = self.wallet_manager.read().await.clone();
                let addr = address.clone();
                let producer = crate::block_producer::BlockProducer::new(
                    ghostdag,
                    mempool,
                    executor,
                    storage,
                    Arc::new(RwLock::new(Some(addr))),
                    wallet_manager,
                    Some(node.peer_manager.clone()),
                );
                node.block_producer_running = Some(producer.running_flag());
                node.block_producer_handle = producer.start().await.ok();
                info!("Block producer started after setting reward address");
            }
        }
    }

    /// Get the current reward address
    pub async fn get_reward_address(&self) -> Option<String> {
        self.reward_address.read().await.clone()
    }

    /// Broadcast a network message if the node is running
    pub async fn broadcast_network(&self, msg: NetworkMessage) -> Result<(), String> {
        if let Some(node) = self.node.read().await.as_ref() {
            node.peer_manager
                .broadcast(&msg)
                .await
                .map_err(|e| e.to_string())
        } else {
            Err("Node is not running".into())
        }
    }
}

/// Parse bootnode strings in formats like:
/// - peer123@203.0.113.10:30303
/// - 203.0.113.10:30303 (peer id will be generated)
fn parse_bootnode(s: &str) -> Option<(lattice_network::peer::PeerId, SocketAddr)> {
    let (peer_part, addr_part) = if let Some((pid, rest)) = s.split_once('@') {
        (Some(pid.trim()), rest.trim())
    } else {
        (None, s.trim())
    };
    let addr: SocketAddr = addr_part.parse().ok()?;
    let peer_id = peer_part
        .map(|p| lattice_network::peer::PeerId::new(p.to_string()))
        .unwrap_or_else(lattice_network::peer::PeerId::random);
    Some((peer_id, addr))
}

/// Start block production in a separate task
#[allow(dead_code)]
async fn start_block_production(
    storage: Arc<StorageManager>,
    executor: Arc<Executor>,
    mempool: Arc<RwLock<Mempool>>,
    ghostdag: Arc<GhostDag>,
    reward_address: String,
    _running: Arc<RwLock<bool>>,
    wallet_manager: Option<Arc<WalletManager>>,
) {
    use crate::block_producer::BlockProducer;

    info!(
        "Starting block producer with reward address: {}",
        reward_address
    );

    let producer = BlockProducer::new(
        ghostdag,
        mempool,
        executor,
        storage,
        Arc::new(RwLock::new(Some(reward_address))),
        wallet_manager,
        None,
    );

    if let Err(e) = producer.start().await {
        error!("Failed to start block producer: {}", e);
    }
}

/// Create the genesis block
fn create_genesis_block() -> Block {
    let genesis_hash = Hash::default();
    let timestamp = 1700000000; // Fixed timestamp for genesis

    let header = BlockHeader {
        version: 1,
        block_hash: genesis_hash,
        selected_parent_hash: genesis_hash,
        merge_parent_hashes: vec![],
        timestamp,
        height: 0,
        blue_score: 0,
        blue_work: 0,
        pruning_point: genesis_hash,
        proposer_pubkey: PublicKey::new([0u8; 32]),
        vrf_reveal: VrfProof {
            proof: vec![0u8; 80],
            output: Hash::default(),
        },
    };

    Block {
        header,
        state_root: Hash::default(),
        tx_root: Hash::default(),
        receipt_root: Hash::default(),
        artifact_root: Hash::default(),
        ghostdag_params: GhostDagParams::default(),
        transactions: vec![],
        signature: Signature::new([0u8; 64]),
    }
}

/// Calculate block hash
#[allow(dead_code)]
fn calculate_block_hash(header: &BlockHeader) -> Hash {
    let mut hasher = Sha3_256::new();
    hasher.update(header.version.to_le_bytes());
    hasher.update(header.selected_parent_hash.as_bytes());
    for parent in &header.merge_parent_hashes {
        hasher.update(parent.as_bytes());
    }
    hasher.update(header.timestamp.to_le_bytes());
    hasher.update(header.height.to_le_bytes());

    let hash_bytes = hasher.finalize();
    let mut hash_array = [0u8; 32];
    hash_array.copy_from_slice(&hash_bytes[..32]);
    Hash::new(hash_array)
}

/// Run the block producer  
#[allow(dead_code)]
async fn run_block_producer(
    storage: Arc<StorageManager>,
    _executor: Arc<Executor>,
    mempool: Arc<RwLock<Mempool>>,
    ghostdag: Arc<GhostDag>,
    dag_store: Arc<DagStore>,
    reward_address: String,
    running: Arc<RwLock<bool>>,
) {
    info!(
        "Starting block producer with reward address: {}",
        reward_address
    );

    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(2));

    while *running.read().await {
        interval.tick().await;

        // Get current tips
        let tips = ghostdag.get_tips().await;
        if tips.is_empty() {
            warn!("No tips available for block production");
            // If no tips, use genesis or latest block
            let latest_height = storage.blocks.get_latest_height().unwrap_or(0);
            if latest_height == 0 {
                continue; // No genesis block yet
            }
            // Continue with a default tip
        }

        // Select parent
        let selected_parent = if !tips.is_empty() {
            tips[0]
        } else {
            // Use genesis or latest block hash as parent
            let latest_height = storage.blocks.get_latest_height().unwrap_or(0);
            if let Ok(Some(block_hash)) = storage.blocks.get_block_by_height(latest_height) {
                block_hash
            } else {
                Hash::default()
            }
        };
        let merge_parents = if tips.len() > 1 {
            tips[1..].to_vec()
        } else {
            vec![]
        };

        // Get transactions from mempool
        let txs = {
            let mempool_guard = mempool.read().await;
            mempool_guard.get_transactions(100).await
        };

        // Get latest height
        let height = storage.blocks.get_latest_height().unwrap_or(0) + 1;

        // Create new block
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let mut header = BlockHeader {
            version: 1,
            block_hash: Hash::new([0u8; 32]), // Will be calculated
            selected_parent_hash: selected_parent,
            merge_parent_hashes: merge_parents,
            timestamp,
            height,
            blue_score: height, // Simplified
            blue_work: height as u128,
            pruning_point: Hash::new([0u8; 32]),
            proposer_pubkey: PublicKey::new([0u8; 32]),
            vrf_reveal: VrfProof {
                proof: vec![0u8; 80],
                output: Hash::new([0u8; 32]),
            },
        };

        // Calculate block hash
        header.block_hash = calculate_block_hash(&header);

        let block = Block {
            header: header.clone(),
            state_root: Hash::new([0u8; 32]),
            tx_root: Hash::new([0u8; 32]),
            receipt_root: Hash::new([0u8; 32]),
            artifact_root: Hash::new([0u8; 32]),
            ghostdag_params: GhostDagParams::default(),
            transactions: txs, // Include transactions from mempool
            signature: Signature::new([0u8; 64]),
        };

        // Store block
        if let Err(e) = storage.blocks.put_block(&block) {
            warn!("Failed to store block: {}", e);
            continue;
        }

        // Add to DAG
        if let Err(e) = dag_store.store_block(block.clone()).await {
            warn!("Failed to add block to DAG: {}", e);
            continue;
        }

        info!(
            "Produced block {} at height {} with hash {:?}",
            height, height, header.block_hash
        );
    }

    info!("Block producer stopped");
}

/// Simplified Lattice node instance
struct LatticeNode {
    storage: Arc<StorageManager>,
    executor: Arc<Executor>,
    mempool: Arc<RwLock<Mempool>>,
    ghostdag: Arc<GhostDag>,
    peer_manager: Arc<PeerManager>,
    _running: Arc<RwLock<bool>>,
    start_time: std::time::Instant,
    block_producer_handle: Option<JoinHandle<()>>,
    block_producer_running: Option<Arc<RwLock<bool>>>,
    rpc_handle: Option<JoinHandle<()>>,
}

impl LatticeNode {
    fn get_uptime(&self) -> u64 {
        self.start_time.elapsed().as_secs()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeStatus {
    pub running: bool,
    pub syncing: bool,
    pub block_height: u64,
    pub peer_count: usize,
    pub network_id: String,
    pub version: String,
    pub uptime: u64,
    pub dag_tips: usize,
    pub blue_score: u64,
    pub last_block_hash: Option<String>,
    pub last_block_timestamp: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerSummary {
    pub id: String,
    pub addr: String,
    pub direction: String,
    pub state: String,
    pub score: i32,
    pub last_seen_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TxActivity {
    pub hash: String,
    pub from: String,
    pub to: Option<String>,
    pub value: String,
    pub nonce: u64,
    pub status: String, // "pending" | "confirmed"
    pub block_hash: Option<String>,
    pub block_height: Option<u64>,
    pub timestamp: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TxOverview {
    pub pending: usize,
    pub last_block: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingTx {
    pub hash: String,
    pub from: String,
    pub to: Option<String>,
    pub value: String,
    pub nonce: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NodeConfig {
    pub data_dir: String,
    pub network: String, // "local", "testnet", or "mainnet"
    pub rpc_port: u16,
    pub ws_port: u16,
    pub p2p_port: u16,
    pub rest_port: u16,
    pub max_peers: usize,
    pub bootnodes: Vec<String>,
    pub reward_address: Option<String>,
    pub external_rpc: Option<String>, // External RPC URL to connect to instead of embedded node
    #[serde(default)]
    pub enable_network: bool,
    #[serde(default)]
    pub discovery: bool,
    #[serde(default)]
    pub mempool: MempoolSettings,
    pub consensus: ConsensusConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConsensusConfig {
    pub k_parameter: u32,
    pub pruning_window: u64,
    pub block_time_seconds: u64,
    pub finality_depth: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MempoolSettings {
    pub min_gas_price: u64,
    pub max_per_sender: usize,
    pub allow_replacement: bool,
    pub chain_id: u64,
    pub max_size: usize,
    pub replacement_factor: u64, // percentage e.g., 125 means 1.25x
    pub require_valid_signature: bool,
    pub tx_expiry_secs: u64,
}

impl Default for MempoolSettings {
    fn default() -> Self {
        Self {
            min_gas_price: 1_000_000_000,
            max_per_sender: 100,
            allow_replacement: true,
            chain_id: 1,
            max_size: 10_000,
            replacement_factor: 125,
            require_valid_signature: true,
            tx_expiry_secs: 3600,
        }
    }
}

impl NodeConfig {
    fn load_or_default() -> Result<Self> {
        let config_path = Self::config_path();
        if config_path.exists() {
            let config_str = std::fs::read_to_string(config_path)?;
            Ok(serde_json::from_str(&config_str)?)
        } else {
            Ok(Self::default())
        }
    }

    fn save(&self) -> Result<()> {
        let config_path = Self::config_path();
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let config_str = serde_json::to_string_pretty(self)?;
        std::fs::write(config_path, config_str)?;
        Ok(())
    }

    fn config_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("lattice-core")
            .join("config.json")
    }

    /// Configure node for testnet connection
    pub fn configure_for_testnet(&mut self) {
        self.network = "testnet".to_string();
        self.mempool.chain_id = 42069;
        self.enable_network = true;
        self.discovery = false; // Don't auto-discover, only connect to specified nodes
        self.max_peers = 100;

        // ONLY connect to localhost testnet node - clear any other bootnodes
        self.bootnodes.clear();
        self.bootnodes.push("127.0.0.1:30303".to_string());
        info!("Testnet mode: Will only connect to localhost:30303");

        // Ensure proper ports for GUI node (different from testnet node)
        self.p2p_port = 30304; // Different from testnet's 30303
        self.rpc_port = 18545; // Different from testnet's 8545
        self.ws_port = 18546; // Different from testnet's 8546
    }

    fn validate(&self) -> Result<()> {
        // Basic port sanity
        if self.rpc_port == 0 || self.ws_port == 0 || self.p2p_port == 0 || self.rest_port == 0 {
            return Err(anyhow::anyhow!(
                "Invalid port configuration: ports must be > 0"
            ));
        }

        // Supported networks for chain selection
        let network_ok = matches!(self.network.as_str(), "devnet" | "testnet" | "mainnet");
        if !network_ok {
            return Err(anyhow::anyhow!(
                "Invalid network: must be one of devnet, testnet, mainnet"
            ));
        }

        // Consensus params
        if self.consensus.k_parameter == 0 {
            return Err(anyhow::anyhow!("Invalid consensus.kParameter: must be > 0"));
        }
        if self.consensus.block_time_seconds == 0 {
            return Err(anyhow::anyhow!(
                "Invalid consensus.blockTimeSeconds: must be > 0"
            ));
        }

        // Mempool settings
        let m = &self.mempool;
        if m.min_gas_price == 0 {
            return Err(anyhow::anyhow!("Invalid mempool.minGasPrice: must be > 0"));
        }
        if m.max_per_sender == 0 {
            return Err(anyhow::anyhow!("Invalid mempool.maxPerSender: must be > 0"));
        }
        if m.chain_id == 0 {
            return Err(anyhow::anyhow!("Invalid mempool.chainId: must be > 0"));
        }
        if m.max_size == 0 {
            return Err(anyhow::anyhow!("Invalid mempool.maxSize: must be > 0"));
        }
        if m.replacement_factor < 100 {
            return Err(anyhow::anyhow!(
                "Invalid mempool.replacementFactor: must be >= 100 (percent)"
            ));
        }
        if m.tx_expiry_secs == 0 {
            return Err(anyhow::anyhow!("Invalid mempool.txExpirySecs: must be > 0"));
        }

        // Bootnodes (when provided) must parse
        for entry in &self.bootnodes {
            if parse_bootnode(entry).is_none() {
                return Err(anyhow::anyhow!(format!(
                    "Invalid bootnode entry '{}': expected peerId@ip:port or ip:port",
                    entry
                )));
            }
        }

        Ok(())
    }
}

impl Default for NodeConfig {
    fn default() -> Self {
        Self {
            data_dir: dirs::data_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("lattice")
                .to_string_lossy()
                .to_string(),
            network: "devnet".to_string(),
            rpc_port: 8545,
            ws_port: 8546,
            p2p_port: 30303,
            rest_port: 3000,
            max_peers: 50,
            bootnodes: vec![],
            reward_address: None,
            external_rpc: None,
            enable_network: false,
            discovery: true,
            mempool: MempoolSettings {
                min_gas_price: 1_000_000_000,
                max_per_sender: 100,
                allow_replacement: true,
                chain_id: 1,
                max_size: 10_000,
                replacement_factor: 125,
                require_valid_signature: true,
                tx_expiry_secs: 3600,
            },
            consensus: ConsensusConfig {
                k_parameter: 18,
                pruning_window: 100_000,
                block_time_seconds: 2,
                finality_depth: 100,
            },
        }
    }
}
