# CLAUDE.md - Lattice v3 with GhostDAG

This file provides guidance to Claude Code (claude.ai/code) when working with the Lattice v3 blockchain implementation.

## Project Overview

Lattice is an AI-native Layer-1 BlockDAG blockchain using **GhostDAG consensus**, paired with an EVM-compatible execution environment (LVM) and a standardized Model Context Protocol (MCP) layer. The project makes AI models first-class on-chain assets with registries, weights, training/eval logs, and verifiable provenance.

## Core Architecture

### Consensus: GhostDAG Protocol
- **Block Structure**: Each block has one `selected_parent` and ≥0 `merge_parents`
- **Blue Set**: Maximal set consistent with k-cluster rule
- **Blue Score**: Total ancestry-consistent blue mass for ordering
- **Total Order**: Selected-parent chain + mergeset, topologically sorted by blue scores
- **Finality**: Committee BFT checkpoints with optimistic confirmation ≤12s

### Key Components
```
lattice/
├─ consensus/         # GhostDAG engine, tip selection, finality
├─ sequencer/         # Mempool policy, bundling, parent selection
├─ execution/         # LVM (EVM-compatible) + precompiles
├─ primitives/        # ModelRegistry, LoRAFactory, InferenceRouter, etc.
├─ bridge/            # ZK light clients, proof verifier
├─ storage/           # State DB (MPT), block store, artifact pinning
├─ api/               # JSON-RPC, REST; OpenAI/Anthropic-compatible
├─ observability/     # Logs, metrics, tracing, DAG visualizer
└─ sdk/               # TS/Python/Rust SDKs (MCP native)
```

## Implementation Priorities

### Phase 1: Core GhostDAG (Sprints 1-4)
1. **Consensus Engine**
   - Selected/merge parent logic
   - Blue set calculation
   - Blue score computation
   - Tip selection algorithm
   - VRF-based proposer selection

2. **Block Structure**
   ```rust
   struct Block {
       version: u32,
       block_hash: Hash,
       selected_parent_hash: Hash,
       merge_parent_hashes: Vec<Hash>,
       timestamp: u64,
       height: u64,
       state_root: Hash,
       tx_root: Hash,
       receipt_root: Hash,
       artifact_root: Hash,
       blue_score: u64,
       ghostdag_params: GhostDagParams,
       proposer_pubkey: PublicKey,
       vrf_reveal: VrfProof,
       signature: Signature,
   }
   ```

3. **DAG Management**
   - Efficient storage of DAG structure
   - Parent/child relationships
   - Anti-past awareness
   - Pruning strategy

### Phase 2: Execution & MCP (Sprints 5-8)
1. **LVM (EVM-compatible)**
   - Transaction execution
   - State transitions
   - Gas metering

2. **MCP Precompiles**
   - Model operations
   - LoRA apply/merge
   - Artifact commits
   - Inference routing

### Phase 3: AI Primitives (Sprints 9-14)
1. **Core Contracts**
   - ModelRegistry
   - LoRAFactory
   - InferenceRouter
   - StorageRegistry
   - ComputeMarket

## Development Guidelines

### Consensus Implementation
- Use Rust for core consensus
- Prioritize correctness over optimization initially
- Extensive unit tests for blue set/score calculations
- Property-based testing for DAG invariants

### Code Organization
```rust
// consensus/src/ghostdag.rs
pub struct GhostDag {
    k: u32,  // k-cluster parameter
    window: u64,  // pruning window
    blue_cache: HashMap<Hash, BlueSet>,
}

impl GhostDag {
    pub fn calculate_blue_set(&self, block: &Block) -> BlueSet;
    pub fn calculate_blue_score(&self, block: &Block) -> u64;
    pub fn select_tip(&self, tips: &[Hash]) -> Hash;
}
```

### Testing Strategy
1. **Unit Tests**: Blue set/score algorithms
2. **Integration Tests**: Full DAG scenarios
3. **Simulation Tests**: Network partitions, reorgs
4. **Performance Tests**: DAG with 10k+ blocks

## Key Algorithms

### Blue Set Calculation
```
1. Start with selected parent's blue set
2. For each merge parent:
   - If consistent with k-cluster rule, add to blue set
   - Otherwise, mark as red
3. Cache results for efficiency
```

### Tip Selection
```
1. Find all current tips (blocks with no children)
2. Calculate blue score for each
3. Select highest blue score
4. Break ties deterministically (by hash)
```

## Sprint Plan Overview

### Sprints 1-4: Core Network (GhostDAG)
- Tip selection, mergeset, blue score
- Finality gadget (alpha version)
- DAG explorer basics

### Sprints 5-6: Sequencer & Mempool
- Transaction classes
- Bundle policy for model updates
- Timestamping & logging

### Sprints 7-8: LVM & Precompiles
- MCP helpers
- LoRA apply/merge
- Gas schedule

### Sprints 9-14: AI Primitives
- ModelRegistry (S9-10)
- InferenceRouter v1 (S11-12)
- LoRAFactory v1 (S13-14)

## API Compatibility

### JSON-RPC (EVM-compatible)
- Standard `eth_*` methods
- Custom `lattice_*` for DAG queries

### MCP REST API
```
/v1/models           # Model registry
/v1/chat/completions # OpenAI-compatible
/v1/embeddings       # Embeddings API
/v1/jobs            # Async job management
/v1/messages        # Anthropic-compatible
```

## Performance Targets

- **Throughput**: 10,000+ TPS
- **Finality**: ≤12 seconds
- **Block Time**: 1-2 seconds
- **DAG Width**: Support 100+ parallel blocks

## Security Considerations

- Gas limits for model operations
- Sandboxed precompile execution
- Attestation & challenge flows
- VRF security for leader election
- Anti-spam measures in mempool

## Common Commands

```bash
# Build consensus module
cargo build -p lattice-consensus

# Run GhostDAG tests
cargo test -p lattice-consensus ghostdag

# Start local testnet
./scripts/start_testnet.sh --consensus ghostdag

# Query DAG state
lattice-cli dag tips
lattice-cli dag blue-score <block-hash>

# Deploy AI primitives
forge script scripts/DeployPrimitives.s.sol --broadcast
```

## Important Notes

1. **GhostDAG vs GHOST**: We use GhostDAG (DAG-based) not GHOST (tree-based)
2. **Blue/Red Distinction**: Blue blocks are in main history, red blocks are not
3. **Parent Types**: Selected parent defines chain, merge parents enable parallelism
4. **MCP Integration**: All AI operations go through MCP standard
5. **Storage**: Artifacts stored off-chain (IPFS/Arweave), referenced by CID

## Resources

- Whitepaper: `lattice-docs-v3/lattice_whitepaper_v3.md`
- Architecture: `lattice-docs-v3/lattice_architecture_v3.md`
- Sprint Plan: `lattice-docs-v3/lattice_sprint_plan_expanded_v3.txt`