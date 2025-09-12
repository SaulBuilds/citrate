# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Lattice is an AI-native Layer-1 BlockDAG blockchain using **GhostDAG consensus**, paired with an EVM-compatible execution environment (LVM) and a standardized Model Context Protocol (MCP) layer. The project makes AI models first-class on-chain assets with registries, weights, training/eval logs, and verifiable provenance.

## Core Architecture

### Consensus: GhostDAG Protocol
- **Block Structure**: Each block has one `selected_parent` and ≥0 `merge_parents`
- **Blue Set**: Maximal set consistent with k-cluster rule
- **Blue Score**: Total ancestry-consistent blue mass for ordering
- **Total Order**: Selected-parent chain + mergeset, topologically sorted by blue scores
- **Finality**: Committee BFT checkpoints with optimistic confirmation ≤12s

### Workspace Structure
```
lattice-v3/
├── cli/                      # CLI tools
├── core/
│   ├── consensus/           # GhostDAG engine, tip selection, finality
│   ├── sequencer/           # Mempool policy, bundling, parent selection
│   ├── execution/           # LVM (EVM-compatible) + precompiles
│   ├── storage/             # State DB (MPT), block store, artifact pinning
│   ├── api/                 # JSON-RPC, REST; OpenAI/Anthropic-compatible
│   ├── network/             # P2P networking, block propagation
│   ├── mcp/                 # Model Context Protocol layer
│   ├── economics/           # Rewards and tokenomics
│   └── primitives/          # Core types and utilities
├── node/                    # Main node binary
├── wallet/                  # CLI wallet (ed25519)
├── faucet/                  # Test token faucet
├── gui/lattice-core/        # Tauri-based GUI wallet
├── contracts/               # Solidity smart contracts
└── scripts/                 # Deployment and testing scripts
```

## Development Commands

### Rust/Cargo Commands
```bash
# Build entire workspace
cargo build --release

# Build specific package
cargo build -p lattice-consensus
cargo build -p lattice-node

# Run tests
cargo test --workspace
cargo test -p lattice-consensus ghostdag
cargo test -p lattice-execution

# Run specific node
cargo run --bin lattice-node -- --data-dir .lattice-devnet

# Run wallet CLI
cargo run --bin lattice-wallet -- --rpc-url http://localhost:8545

# Format code
cargo fmt --all

# Lint
cargo clippy --all-targets --all-features
```

### GUI/Tauri Commands
```bash
# Navigate to GUI directory
cd gui/lattice-core

# Install dependencies
npm install

# Run development server
npm run dev

# Build Tauri app
npm run tauri:build

# Run Tauri in dev mode
npm run tauri dev
```

### Solidity/Foundry Commands
```bash
# Build contracts
forge build

# Run contract tests
forge test

# Deploy contracts
forge script script/Deploy.s.sol --rpc-url http://localhost:8545 --broadcast

# Format Solidity
forge fmt
```

### Testing Scripts
```bash
# Start local testnet (from lattice-v3/)
./scripts/start_testnet.sh --consensus ghostdag

# Deploy test contracts
./scripts/deploy_contracts.sh

# Send test transaction
./scripts/send_test_tx.sh
```

## Critical Transaction Pipeline Issues & Solutions

### Known Issues
The transaction pipeline has several critical issues that prevent transactions from completing:

1. **GUI Producer Issue**: The Tauri GUI's embedded node producer doesn't execute transactions or persist receipts, only simulates wallet balances locally
2. **Address Mismatch**: 20-byte EVM addresses embedded in 32-byte fields cause derivation mismatches
3. **RPC Decoder Gaps**: Limited support for EIP-1559 typed transactions
4. **Nonce Reading**: Uses "latest" instead of "pending", causing sequential transaction blocks
5. **Missing Observability**: No mempool snapshot endpoint or clear transaction status visibility

### Transaction Flow Paths

#### CLI Wallet → RPC → Node
```
1. wallet/src/transaction.rs - Signs with ed25519
2. wallet/src/rpc_client.rs - Sends via eth_sendRawTransaction
3. core/api/src/eth_tx_decoder.rs - Decodes bincode/RLP
4. core/sequencer/src/mempool.rs - Validates and stores
5. node/src/producer.rs - Executes and stores receipts
```

#### GUI Wallet → Embedded Node
```
1. gui/../wallet_manager.rs - Creates and signs transaction
2. gui/../lib.rs:195 - Adds directly to embedded mempool
3. gui/../block_producer.rs - Produces blocks WITHOUT execution
4. Problem: No receipts stored, no state changes
```

### File-Level Fix Points

#### High Priority Fixes
- `gui/lattice-core/src-tauri/src/block_producer.rs` - Align with node producer execution
- `core/api/src/eth_tx_decoder.rs` - Add EIP-1559 support
- `core/execution/src/executor.rs:338` - Fix address mapping for transfers
- `core/api/src/eth_rpc.rs:389` - Handle "pending" nonce correctly

#### Mempool & Verification
- `core/sequencer/src/mempool.rs:374` - Unify signature verification paths
- `core/sequencer/src/mempool.rs:298` - Fix nonce state tracking

#### Storage & Receipts
- `core/storage/src/chain/transaction_store.rs` - Ensure receipt persistence
- `node/src/producer.rs` - Reference implementation for GUI

## Block Structure
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

## API Endpoints

### JSON-RPC (EVM-compatible)
- Standard `eth_*` methods at port 8545
- Custom `lattice_*` for DAG queries
- WebSocket support at ws://localhost:8546

### MCP REST API
```
/v1/models           # Model registry
/v1/chat/completions # OpenAI-compatible
/v1/embeddings       # Embeddings API
/v1/jobs            # Async job management
/v1/messages        # Anthropic-compatible
```

## Testing Strategy

### Unit Tests
```bash
# Test specific module
cargo test -p lattice-consensus
cargo test -p lattice-execution

# Test with output
cargo test -- --nocapture

# Test single function
cargo test test_blue_set_calculation
```

### Integration Tests
```bash
# Full node integration
cargo test --test integration

# DAG scenarios
cargo test -p lattice-consensus --test dag_scenarios
```

### End-to-End Tests
```bash
# Start testnet and run E2E
./scripts/run_e2e_tests.sh
```

## Performance Targets
- **Throughput**: 10,000+ TPS
- **Finality**: ≤12 seconds
- **Block Time**: 1-2 seconds
- **DAG Width**: Support 100+ parallel blocks

## Important Implementation Notes

1. **GhostDAG vs GHOST**: We use GhostDAG (DAG-based) not GHOST (tree-based)
2. **Signature Schemes**: Supporting both ed25519 (native) and ECDSA (EVM compatibility)
3. **Address Format**: 20-byte EVM addresses must be carefully handled when embedded in 32-byte fields
4. **Nonce Management**: Always use "pending" for wallet queries to avoid transaction blocks
5. **Producer Execution**: Both GUI and core node producers must execute transactions and store receipts
6. **MCP Integration**: All AI operations go through MCP standard
7. **Storage**: Artifacts stored off-chain (IPFS/Arweave), referenced by CID

## Debugging Transaction Issues

### Check Transaction Status
```bash
# Get transaction by hash
curl -X POST http://localhost:8545 -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"eth_getTransactionByHash","params":["0xHASH"],"id":1}'

# Get receipt
curl -X POST http://localhost:8545 -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"eth_getTransactionReceipt","params":["0xHASH"],"id":1}'

# Check mempool (custom endpoint needed)
curl http://localhost:8545/mempool
```

### Common Issues
- **Pending Forever**: Check if GUI producer is executing transactions
- **Wrong Balance**: Verify address derivation in executor
- **Invalid Signature**: Check chainId and signature scheme compatibility
- **Nonce Too Low**: Ensure using "pending" nonce, not "latest"

## Sprint Implementation Plan

### Sprint 1: EVM Transaction Support
- Full EIP-1559 decoder implementation
- Unified signature verification
- ChainId alignment

### Sprint 2: Producer Unification
- Fix GUI block producer execution
- Ensure receipt persistence
- Remove wallet manager simulation

### Sprint 3: Address Handling
- Standardize 20-byte address embedding
- Fix transfer address derivation
- Add comprehensive tests

### Sprint 4: Nonce & UX
- Implement "pending" nonce support
- Add mempool visibility endpoint
- Surface transaction status in UI

### Sprint 5: Observability
- Transaction lifecycle tracing
- Mempool metrics
- Error surfacing in UI

### Sprint 6: EVM Wallet Integration
- MetaMask compatibility testing
- Documentation for external wallets
- Example integration scripts