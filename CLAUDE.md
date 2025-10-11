# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Lattice is a Layer‑1, AI‑native BlockDAG blockchain with Ethereum‑compatible JSON‑RPC, model registration/inference flows, and a desktop GUI. Uses GhostDAG consensus for high-throughput parallel block processing.

## Core Architecture

### Consensus Layer (GhostDAG)
- **DagStore**: Manages block storage with parent/child relationships and tip tracking
- **GhostDag**: Implements blue set calculation and k-cluster validation
- **TipSelector**: Selects tips based on highest blue score strategy
- **ChainSelector**: Manages chain state and reorganization events

Key types:
- `Hash`: 32-byte identifier, created with `Hash::new([u8; 32])`
- `PublicKey`: 32-byte key, created with `PublicKey::new([u8; 32])`
- `Signature`: 64-byte signature, created with `Signature::new([u8; 64])`
- `Block`: Contains header, state roots, transactions, and GhostDAG params

### Transaction Pipeline
**Working path**: CLI Wallet → RPC → Node → Execution → State
**Broken path**: GUI Wallet → Embedded Node (doesn't execute transactions)

## Development Commands

### Building
```bash
# Full workspace build
cargo build --workspace --release

# Specific module
cargo build -p lattice-consensus
cargo build -p lattice-node
```

### Testing
```bash
# Run all tests
cargo test --workspace

# Run specific module tests
cargo test -p lattice-consensus
cargo test -p lattice-execution

# Run specific test file
cargo test -p lattice-consensus --test real_tests

# Run with output
cargo test -- --nocapture

# Run single test
cargo test test_dag_store_new
```

### Running
```bash
# Start devnet node
cargo run -p lattice-node -- devnet

# Run wallet CLI
cargo run -p lattice-wallet -- --rpc-url http://localhost:8545

# Start GUI (from gui/lattice-core)
npm run tauri dev
```

### Code Quality
```bash
# Format
cargo fmt --all

# Lint
cargo clippy --all-targets --all-features

# Fix warnings
cargo fix --workspace
```

## Module APIs

### Consensus (`lattice-consensus`)
```rust
// Create DAG components
let dag_store = Arc::new(DagStore::new());
let params = GhostDagParams::default(); // k=18, max_parents=10
let ghostdag = GhostDag::new(params, dag_store.clone());

// Store blocks
dag_store.store_block(block).await?;

// Query DAG
let tips = dag_store.get_tips().await;
let children = dag_store.get_children(&parent_hash).await;
let blue_set = ghostdag.calculate_blue_set(&block).await?;
```

### Storage (`lattice-storage`)
- StateDB: Merkle Patricia Trie for account state
- ChainStore: Block and transaction storage with RocksDB
- TransactionStore: Receipt storage and indexing

### Execution (`lattice-execution`)
- Executor: EVM-compatible transaction execution
- Precompiles: AI model operations and ZKP verification
- Address utilities: Handle 20-byte EVM addresses in 32-byte fields

## Critical Issues & Fixes

### GUI Transaction Execution
**Problem**: GUI producer doesn't execute transactions
**Fix location**: `gui/lattice-core/src-tauri/src/block_producer.rs`
**Solution**: Add `executor.execute()` and `state_db.commit()`

### Address Format Mismatch
**Problem**: Mixing 20-byte and 32-byte addresses
**Fix location**: `core/execution/src/address_utils.rs`
**Solution**: Use `normalize_address()` for conversions

### Test Infrastructure
**Status**: Most tests are stubs that don't compile
**Working tests**:
- `core/consensus/tests/real_tests.rs` (comprehensive)
- `core/*/tests/simple_tests.rs` (smoke tests only)

## Testing Best Practices

1. **Always verify compilation**: `cargo test --no-run` before claiming tests work
2. **Use actual APIs**: Check module exports with `cargo doc --open`
3. **Test helpers**: Create realistic test data matching actual structs
4. **Async tests**: Use `#[tokio::test]` for async functions
5. **Integration tests**: Test actual workflows, not just unit functions

## Common Pitfalls

1. **Don't assume APIs**: The actual API often differs from expectations
2. **Hash construction**: Use `Hash::new([u8; 32])` not `Hash([u8; 32])`
3. **Module names**: Use hyphens `lattice-consensus` not underscores
4. **Default values**: Check actual defaults (e.g., `GhostDagParams::default()`)
5. **Async everywhere**: Most consensus/storage operations are async

## Performance Targets
- Throughput: 10,000+ TPS
- Finality: ≤12 seconds
- Block time: 1-2 seconds
- DAG width: 100+ parallel blocks

## Debugging Commands

```bash
# Check RPC status
curl -s http://localhost:8545 -H 'content-type: application/json' \
  -d '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}' | jq

# Get transaction receipt
curl -X POST http://localhost:8545 -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"eth_getTransactionReceipt","params":["0xHASH"],"id":1}'

# Check mempool (if endpoint exists)
curl http://localhost:8545/mempool
```