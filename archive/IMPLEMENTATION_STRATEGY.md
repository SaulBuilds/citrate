# Citrate v3 Implementation Strategy - GhostDAG BlockDAG

## Executive Summary

Complete rewrite of Citrate blockchain using GhostDAG consensus protocol for AI-native Layer-1 with MCP (Model Context Protocol) integration. This strategy provides a granular, sprint-by-sprint implementation plan.

## Architecture Overview

### Core Components
- **GhostDAG Consensus**: Selected parent + merge parents, blue set/score calculation
- **LVM (Citrate VM)**: EVM-compatible with AI-specific precompiles
- **MCP Layer**: Standardized AI model interaction protocol
- **AI Primitives**: On-chain contracts for model registry, inference, training

## Project Structure

```
citrate/
├── consensus/
│   ├── src/
│   │   ├── ghostdag/
│   │   │   ├── mod.rs           # GhostDAG main module
│   │   │   ├── blue_set.rs      # Blue set calculation
│   │   │   ├── blue_score.rs    # Blue score computation
│   │   │   ├── tip_selection.rs # Tip selection algorithm
│   │   │   └── pruning.rs       # DAG pruning logic
│   │   ├── block/
│   │   │   ├── mod.rs           # Block structure
│   │   │   ├── header.rs        # Block header
│   │   │   └── validation.rs    # Block validation
│   │   ├── dag/
│   │   │   ├── mod.rs           # DAG management
│   │   │   ├── storage.rs       # DAG persistence
│   │   │   └── traversal.rs     # DAG traversal algorithms
│   │   ├── vrf/
│   │   │   ├── mod.rs           # VRF implementation
│   │   │   └── proposer.rs      # Proposer selection
│   │   └── finality/
│   │       ├── mod.rs           # Finality gadget
│   │       └── checkpoint.rs    # BFT checkpoints
│   └── Cargo.toml
├── sequencer/
│   ├── src/
│   │   ├── mempool.rs          # Transaction pool
│   │   ├── bundler.rs          # Transaction bundling
│   │   └── parent_selector.rs   # Parent selection logic
│   └── Cargo.toml
├── execution/
│   ├── src/
│   │   ├── lvm/                # Citrate VM
│   │   └── precompiles/        # MCP precompiles
│   └── Cargo.toml
├── primitives/
│   ├── contracts/
│   │   ├── ModelRegistry.sol
│   │   ├── LoRAFactory.sol
│   │   ├── InferenceRouter.sol
│   │   └── StorageRegistry.sol
│   └── foundry.toml
├── storage/
│   ├── src/
│   │   ├── state_db.rs         # State database
│   │   ├── block_store.rs      # Block storage
│   │   └── artifact_store.rs   # Model artifact storage
│   └── Cargo.toml
├── api/
│   ├── src/
│   │   ├── json_rpc.rs         # EVM-compatible RPC
│   │   └── mcp_rest.rs         # MCP REST API
│   └── Cargo.toml
└── tests/
    ├── integration/
    └── simulation/
```

---

## Sprint-by-Sprint Implementation

## Phase 1: Core GhostDAG Consensus (Sprints 1-4)

### Sprint 1: GhostDAG Foundation
**Week 1-2** | **Focus**: Core data structures and blue set algorithm

#### 1.1 Setup Project Structure
```bash
# Initialize workspace
mkdir citrate && cd citrate
cat > Cargo.toml << 'EOF'
[workspace]
members = [
    "consensus",
    "sequencer",
    "execution",
    "storage",
    "api",
]
resolver = "2"

[workspace.package]
version = "0.1.0"
edition = "2021"
authors = ["Citrate Team"]

[workspace.dependencies]
tokio = { version = "1.35", features = ["full"] }
async-trait = "0.1"
serde = { version = "1.0", features = ["derive"] }
blake3 = "1.5"
rocksdb = "0.21"
EOF

cargo new --lib consensus
```

#### 1.2 Block Structure Implementation
```rust
// consensus/src/block/mod.rs
use blake3::Hash;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub header: BlockHeader,
    pub transactions: Vec<Transaction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockHeader {
    pub version: u32,
    pub block_hash: Hash,
    pub selected_parent_hash: Hash,
    pub merge_parent_hashes: Vec<Hash>,
    pub timestamp: u64,
    pub height: u64,
    pub state_root: Hash,
    pub tx_root: Hash,
    pub receipt_root: Hash,
    pub artifact_root: Hash,
    pub blue_score: u64,
    pub ghostdag_params: GhostDagParams,
    pub proposer_pubkey: PublicKey,
    pub vrf_reveal: VrfProof,
    pub signature: Signature,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GhostDagParams {
    pub k: u32,           // k-cluster parameter
    pub window: u64,      // pruning window
    pub difficulty: u64,  // mining difficulty
}
```

#### 1.3 Blue Set Calculation
```rust
// consensus/src/ghostdag/blue_set.rs
use std::collections::{HashMap, HashSet};

pub struct BlueSetCalculator {
    k: u32,
    dag: Arc<DAG>,
}

impl BlueSetCalculator {
    pub fn calculate_blue_set(&self, block: &Block) -> BlueSet {
        let mut blue_set = HashSet::new();
        let mut red_set = HashSet::new();
        
        // Start with selected parent's blue set
        if let Some(parent_blue) = self.dag.get_blue_set(&block.header.selected_parent_hash) {
            blue_set.extend(parent_blue);
        }
        
        // Process merge parents
        for merge_parent in &block.header.merge_parent_hashes {
            if self.is_blue_compatible(&blue_set, merge_parent) {
                blue_set.insert(*merge_parent);
                // Add ancestors to blue set
                self.add_blue_ancestors(&mut blue_set, merge_parent);
            } else {
                red_set.insert(*merge_parent);
            }
        }
        
        BlueSet { blue: blue_set, red: red_set }
    }
    
    fn is_blue_compatible(&self, current_blue: &HashSet<Hash>, candidate: &Hash) -> bool {
        // Check k-cluster rule
        let anti_cone = self.dag.get_anticone(candidate);
        let blue_anticone: HashSet<_> = anti_cone.intersection(current_blue).collect();
        blue_anticone.len() <= self.k as usize
    }
}
```

#### 1.4 Testing Framework
```rust
// consensus/src/ghostdag/tests.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blue_set_single_parent() {
        let calc = BlueSetCalculator::new(3);
        // Test with single selected parent
    }

    #[test]
    fn test_blue_set_multiple_parents() {
        // Test with merge parents
    }

    #[test]
    fn test_k_cluster_violation() {
        // Test red set marking
    }
}
```

### Sprint 2: Blue Score & Tip Selection
**Week 3-4** | **Focus**: Blue score computation and tip selection

#### 2.1 Blue Score Calculation
```rust
// consensus/src/ghostdag/blue_score.rs
pub struct BlueScoreCalculator {
    dag: Arc<DAG>,
    blue_cache: RwLock<HashMap<Hash, u64>>,
}

impl BlueScoreCalculator {
    pub async fn calculate_blue_score(&self, block: &Block) -> u64 {
        // Check cache
        if let Some(score) = self.blue_cache.read().await.get(&block.header.block_hash) {
            return *score;
        }
        
        let blue_set = self.dag.get_blue_set(&block.header.block_hash);
        let mut score = 0u64;
        
        // Sum blue scores of all blue parents
        for parent in &blue_set.blue {
            if let Some(parent_score) = self.get_blue_score(parent).await {
                score += parent_score;
            }
        }
        
        // Add self
        score += 1;
        
        // Cache result
        self.blue_cache.write().await.insert(block.header.block_hash, score);
        score
    }
}
```

#### 2.2 Tip Selection Algorithm
```rust
// consensus/src/ghostdag/tip_selection.rs
pub struct TipSelector {
    dag: Arc<DAG>,
    blue_calculator: Arc<BlueScoreCalculator>,
}

impl TipSelector {
    pub async fn select_tip(&self) -> Result<Hash, ConsensusError> {
        let tips = self.dag.get_tips();
        
        if tips.is_empty() {
            return Err(ConsensusError::NoTips);
        }
        
        let mut best_tip = tips[0];
        let mut best_score = 0u64;
        
        for tip in tips {
            let score = self.blue_calculator.calculate_blue_score(&tip).await;
            if score > best_score || (score == best_score && tip < best_tip) {
                best_tip = tip;
                best_score = score;
            }
        }
        
        Ok(best_tip)
    }
    
    pub async fn select_parents(&self, max_parents: usize) -> Vec<Hash> {
        let tips = self.dag.get_tips();
        let selected_parent = self.select_tip().await.unwrap();
        
        let mut parents = vec![selected_parent];
        
        // Add merge parents
        for tip in tips {
            if tip != selected_parent && parents.len() < max_parents {
                parents.push(tip);
            }
        }
        
        parents
    }
}
```

### Sprint 3: DAG Management & Storage
**Week 5-6** | **Focus**: Efficient DAG storage and traversal

#### 3.1 DAG Storage
```rust
// consensus/src/dag/storage.rs
use rocksdb::{DB, Options};

pub struct DAGStore {
    db: Arc<DB>,
    relations: Arc<RwLock<Relations>>,
}

struct Relations {
    children: HashMap<Hash, Vec<Hash>>,
    parents: HashMap<Hash, ParentSet>,
    tips: HashSet<Hash>,
    blue_sets: HashMap<Hash, BlueSet>,
    blue_scores: HashMap<Hash, u64>,
}

impl DAGStore {
    pub fn new(path: &str) -> Result<Self, StorageError> {
        let mut opts = Options::default();
        opts.create_if_missing(true);
        opts.set_compression_type(rocksdb::DBCompressionType::Lz4);
        
        let db = DB::open(&opts, path)?;
        
        Ok(Self {
            db: Arc::new(db),
            relations: Arc::new(RwLock::new(Relations::default())),
        })
    }
    
    pub async fn add_block(&self, block: Block) -> Result<(), StorageError> {
        // Store block
        let key = block.header.block_hash.as_bytes();
        let value = bincode::serialize(&block)?;
        self.db.put(key, value)?;
        
        // Update relations
        let mut rels = self.relations.write().await;
        
        // Update parent-child relationships
        rels.parents.insert(
            block.header.block_hash,
            ParentSet {
                selected: block.header.selected_parent_hash,
                merge: block.header.merge_parent_hashes.clone(),
            }
        );
        
        // Update children
        for parent in block.all_parents() {
            rels.children.entry(parent)
                .or_insert_with(Vec::new)
                .push(block.header.block_hash);
        }
        
        // Update tips
        rels.tips.remove(&block.header.selected_parent_hash);
        for merge_parent in &block.header.merge_parent_hashes {
            rels.tips.remove(merge_parent);
        }
        rels.tips.insert(block.header.block_hash);
        
        Ok(())
    }
}
```

### Sprint 4: VRF & Finality Gadget
**Week 7-8** | **Focus**: VRF proposer selection and finality

#### 4.1 VRF Implementation
```rust
// consensus/src/vrf/mod.rs
use vrf::{VRF, ProofVerifier};

pub struct VRFProposerSelector {
    vrf: VRF,
    validator_set: Arc<RwLock<ValidatorSet>>,
}

impl VRFProposerSelector {
    pub async fn is_proposer(
        &self,
        validator: &PublicKey,
        slot: u64,
        prev_vrf: &VrfOutput,
    ) -> Result<(bool, VrfProof), VrfError> {
        let input = self.compute_vrf_input(slot, prev_vrf);
        let (output, proof) = self.vrf.prove(&input, validator)?;
        
        let threshold = self.compute_threshold(validator).await;
        let is_leader = output.as_u64() < threshold;
        
        Ok((is_leader, proof))
    }
}
```

#### 4.2 Finality Gadget (Alpha)
```rust
// consensus/src/finality/mod.rs
pub struct FinalityGadget {
    checkpoints: Arc<RwLock<Vec<Checkpoint>>>,
    committee: Arc<Committee>,
}

impl FinalityGadget {
    pub async fn create_checkpoint(&self, block: &Block) -> Option<Checkpoint> {
        if block.header.blue_score % CHECKPOINT_INTERVAL == 0 {
            Some(Checkpoint {
                block_hash: block.header.block_hash,
                height: block.header.height,
                timestamp: block.header.timestamp,
                signatures: Vec::new(),
            })
        } else {
            None
        }
    }
    
    pub async fn sign_checkpoint(&self, checkpoint: &mut Checkpoint, validator: &Validator) {
        let signature = validator.sign(&checkpoint.hash());
        checkpoint.signatures.push((validator.pubkey, signature));
    }
    
    pub fn is_finalized(&self, checkpoint: &Checkpoint) -> bool {
        checkpoint.signatures.len() >= self.committee.threshold()
    }
}
```

---

## Phase 2: Sequencer & Execution (Sprints 5-8)

### Sprint 5-6: Sequencer & Mempool
**Week 9-12** | **Focus**: Transaction sequencing and mempool management

#### 5.1 Mempool Implementation
```rust
// sequencer/src/mempool.rs
pub struct Mempool {
    pending: Arc<RwLock<BTreeMap<TxPriority, Transaction>>>,
    by_sender: Arc<RwLock<HashMap<Address, Vec<TxHash>>>>,
    config: MempoolConfig,
}

pub struct MempoolConfig {
    pub max_size: usize,
    pub max_per_sender: usize,
    pub min_gas_price: u64,
    pub tx_classes: Vec<TxClass>,
}

#[derive(Debug, Clone)]
pub enum TxClass {
    Standard,
    ModelUpdate,
    Inference,
    Training,
    Storage,
}
```

### Sprint 7-8: LVM & Precompiles
**Week 13-16** | **Focus**: EVM compatibility and MCP precompiles

#### 7.1 MCP Precompiles
```rust
// execution/src/precompiles/mcp.rs
pub struct MCPPrecompile;

impl Precompile for MCPPrecompile {
    fn execute(&self, input: &[u8]) -> Result<Vec<u8>, PrecompileError> {
        let call = MCPCall::decode(input)?;
        
        match call {
            MCPCall::RegisterModel { cid, metadata } => {
                self.register_model(cid, metadata)
            }
            MCPCall::ApplyLoRA { model_id, lora_id } => {
                self.apply_lora(model_id, lora_id)
            }
            MCPCall::SubmitInference { model_id, input } => {
                self.submit_inference(model_id, input)
            }
        }
    }
}
```

---

## Phase 3: AI Primitives (Sprints 9-14)

### Sprint 9-10: ModelRegistry
```solidity
// primitives/contracts/ModelRegistry.sol
contract ModelRegistry {
    struct Model {
        string cid;
        address owner;
        uint256 version;
        bytes32 attestation;
        mapping(address => bool) authorized;
    }
    
    mapping(uint256 => Model) public models;
    
    function registerModel(string memory cid, bytes32 attestation) external returns (uint256);
    function updateModel(uint256 modelId, string memory newCid) external;
    function authorizeUser(uint256 modelId, address user) external;
}
```

### Sprint 11-12: InferenceRouter
```solidity
contract InferenceRouter {
    struct Job {
        uint256 modelId;
        bytes input;
        address requester;
        address provider;
        uint256 escrow;
        JobStatus status;
    }
    
    function submitJob(uint256 modelId, bytes calldata input) external payable returns (uint256);
    function claimJob(uint256 jobId) external;
    function submitResult(uint256 jobId, bytes calldata result) external;
}
```

---

## Testing Strategy

### Unit Tests
```bash
# Test GhostDAG algorithms
cargo test -p consensus ghostdag::

# Test blue set calculation
cargo test -p consensus blue_set::

# Test tip selection
cargo test -p consensus tip_selection::
```

### Integration Tests
```rust
// tests/integration/ghostdag_test.rs
#[tokio::test]
async fn test_dag_convergence() {
    let mut dag = create_test_dag();
    
    // Simulate parallel block creation
    for _ in 0..100 {
        let parents = dag.select_parents(5).await;
        let block = create_block(parents);
        dag.add_block(block).await.unwrap();
    }
    
    // Verify convergence
    assert!(dag.has_single_tip_convergence());
}
```

### Simulation Tests
```rust
// tests/simulation/network_partition.rs
#[tokio::test]
async fn test_partition_recovery() {
    let network = SimulatedNetwork::new(10); // 10 nodes
    
    // Create partition
    network.partition(vec![0..5, 5..10]);
    
    // Generate blocks on both sides
    network.run_for(Duration::from_secs(60)).await;
    
    // Heal partition
    network.heal();
    network.run_for(Duration::from_secs(30)).await;
    
    // Check convergence
    assert!(network.has_consensus());
}
```

---

## Performance Benchmarks

### Target Metrics
- **Blue Set Calculation**: <10ms for 1000-block DAG
- **Blue Score Computation**: <5ms with caching
- **Tip Selection**: <1ms for 100 tips
- **Block Validation**: <50ms
- **Transaction Throughput**: 10,000+ TPS

### Benchmark Suite
```rust
// benches/ghostdag_bench.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_blue_set(c: &mut Criterion) {
    let dag = create_large_dag(10000);
    
    c.bench_function("blue_set_1000", |b| {
        b.iter(|| {
            dag.calculate_blue_set(black_box(&random_block()))
        })
    });
}
```

---

## Deployment Pipeline

### Local Development
```bash
# Start single node
./scripts/start_node.sh --consensus ghostdag --k 3

# Start local testnet
./scripts/start_testnet.sh --nodes 5 --consensus ghostdag

# Deploy primitives
forge script scripts/DeployAll.s.sol --rpc-url localhost:8545
```

### Testnet Configuration
```yaml
# config/testnet.yaml
consensus:
  type: ghostdag
  k: 8
  window: 10000
  checkpoint_interval: 100
  
network:
  bootstrap_nodes:
    - /ip4/testnet.lattice.xyz/tcp/30303/p2p/...
  
execution:
  vm: lvm
  precompiles:
    - mcp: 0x100
    - lora: 0x101
    - inference: 0x102
```

---

## Critical Path

### Must-Have for MVP
1. ✅ GhostDAG consensus with blue set/score
2. ✅ Parent selection (selected + merge)
3. ✅ Basic finality (optimistic)
4. ✅ EVM compatibility
5. ✅ ModelRegistry contract
6. ✅ Basic MCP precompiles

### Nice-to-Have
1. ⏳ Full BFT finality
2. ⏳ Advanced pruning
3. ⏳ GPU acceleration for blue set
4. ⏳ Full MCP compliance

---

## Success Criteria

### Sprint 1-4 (Core Consensus)
- [ ] Blue set calculation working correctly
- [ ] Blue score properly computed
- [ ] Tip selection converges
- [ ] 1000+ blocks DAG handling
- [ ] VRF proposer selection

### Sprint 5-8 (Execution)
- [ ] EVM compatibility tests pass
- [ ] MCP precompiles functional
- [ ] Transaction classes working
- [ ] Gas metering accurate

### Sprint 9-14 (AI Primitives)
- [ ] ModelRegistry deployed
- [ ] InferenceRouter handling jobs
- [ ] LoRA operations working
- [ ] Storage integration complete

---

## Next Steps

1. **Immediate**: Set up fresh project structure
2. **Week 1**: Implement core GhostDAG data structures
3. **Week 2**: Blue set algorithm and tests
4. **Week 3**: Blue score and tip selection
5. **Week 4**: DAG storage and persistence

This implementation strategy provides a complete roadmap for building Citrate v3 with GhostDAG consensus and MCP integration.