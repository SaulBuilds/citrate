# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## 🚨 CRITICAL IMPLEMENTATION GUIDELINES

### No Mocks, Stubs, or Incomplete Implementations
**MANDATORY:** All code delivered MUST be fully functional and production-ready. Do NOT create:
- Mock implementations or placeholder functions
- TODO comments or stub methods
- Partial implementations that require "future completion"
- Test-only or demonstration code unless explicitly requested

**REQUIREMENTS for all implementations:**
- **Complete functionality** - Every feature must work end-to-end
- **Proper error handling** - Handle all edge cases and error conditions
- **Production security** - Input validation, access controls, and secure patterns
- **Comprehensive testing** - Unit tests, integration tests, and validation
- **Full documentation** - API docs, usage examples, and clear explanations

### Current Development Phase
**Phase 4, Week 3: Model Marketplace & Discovery** (See GLOBAL_ROADMAP.md for complete roadmap)

## Project Overview

Citrate is an AI-native Layer-1 BlockDAG blockchain using **GhostDAG consensus**, paired with an EVM-compatible execution environment (LVM) and a standardized Model Context Protocol (MCP) layer. The project makes AI models first-class on-chain assets with registries, weights, training/eval logs, and verifiable provenance.

## Core Architecture

### Consensus: GhostDAG Protocol
- **Block Structure**: Each block has one `selected_parent` and ≥0 `merge_parents`
- **Blue Set**: Maximal set consistent with k-cluster rule
- **Blue Score**: Total ancestry-consistent blue mass for ordering
- **Total Order**: Selected-parent chain + mergeset, topologically sorted by blue scores
- **Finality**: Committee BFT checkpoints with optimistic confirmation ≤12s

### Workspace Structure
```
citrate/
├── Core Components
│   ├── core/
│   │   ├── consensus/           # GhostDAG engine, tip selection, finality
│   │   ├── sequencer/           # Mempool policy, bundling, parent selection
│   │   ├── execution/           # LVM (EVM-compatible) + precompiles
│   │   ├── storage/             # State DB (MPT), block store, artifact pinning
│   │   ├── api/                 # JSON-RPC, REST; OpenAI/Anthropic-compatible
│   │   ├── network/             # P2P networking, block propagation
│   │   ├── mcp/                 # Model Context Protocol layer
│   │   ├── economics/           # Rewards and tokenomics
│   │   ├── marketplace/         # Marketplace contracts integration
│   │   └── primitives/          # Core types and utilities
│   ├── node/                    # Main node binary
│   └── node-app/                # Node application wrapper
│
├── Client Applications
│   ├── gui/lattice-core/        # Tauri-based GUI wallet (React + Vite)
│   ├── explorer/                # Web explorer (Next.js)
│   ├── wallet/                  # CLI wallet (ed25519)
│   ├── cli/                     # CLI tools
│   └── faucet/                  # Test token faucet
│
├── Developer Tools
│   ├── developer-tools/
│   │   ├── lattice-studio/      # Visual IDE for Citrate development
│   │   ├── debug-dashboard/     # Debug dashboard UI
│   │   ├── documentation-portal/ # Docs generation tools
│   │   └── vscode-extension/    # VS Code language support
│   └── contracts/               # Solidity smart contracts (Foundry)
│
├── SDKs
│   ├── sdk/javascript/          # Official TypeScript SDK (@citrate-ai/sdk)
│   └── sdks/
│       ├── javascript/citrate-js/ # Alternative JS SDK
│       └── python/              # Python SDK
│
├── Documentation & Sites
│   ├── docs-portal/             # Docusaurus documentation site
│   └── marketing-site/          # Next.js marketing site
│
└── Utilities
    └── scripts/                 # Deployment, testing, orchestration
```

## Development Commands

### Orchestration Script (Recommended)
```bash
# Central orchestration script for common tasks
scripts/lattice.sh setup          # Install all dependencies
scripts/lattice.sh build          # Build Node/CLI (release), GUI, Explorer, Docs
scripts/lattice.sh dev up         # Start dev stack (node, explorer, docs, marketing)
scripts/lattice.sh dev down       # Stop dev stack
scripts/lattice.sh dev status     # Check dev stack status
scripts/lattice.sh testnet up     # Start testnet node
scripts/lattice.sh docker up      # Run devnet via docker-compose
scripts/lattice.sh docker down    # Stop docker devnet
scripts/lattice.sh logs           # Tail logs in run-logs/
scripts/lattice.sh clean          # Clean Rust targets and caches
```

### Rust/Cargo Commands
```bash
# Build entire workspace
cargo build --release

# Build specific package
cargo build -p citrate-consensus
cargo build -p citrate-node
cargo build -p citrate-execution

# Run tests
cargo test --workspace
cargo test -p citrate-consensus ghostdag
cargo test -p citrate-execution

# Run specific test with output
cargo test test_blue_set_calculation -- --nocapture

# Run specific node
cargo run --bin citrate-node -- --data-dir .citrate-devnet
cargo run --bin citrate-node -- devnet

# Run wallet CLI
cargo run --bin citrate-wallet -- --rpc-url http://localhost:8545

# Format code
cargo fmt --all

# Lint
cargo clippy --all-targets --all-features
cargo clippy --all-targets --all-features -D warnings  # Fail on warnings
```

### GUI/Tauri Commands
```bash
# Navigate to GUI directory
cd gui/lattice-core

# Install dependencies
npm install

# Run development server (web only)
npm run dev

# Build web version
npm run build

# Run Tauri desktop app (dev mode)
npm run tauri dev

# Build Tauri app (production)
npm run tauri:build

# Lint and format
npm run lint
npm run format
```

### Solidity/Foundry Commands
```bash
# Navigate to contracts directory
cd contracts

# Build contracts
forge build

# Run contract tests
forge test
forge test -vv           # Verbose output
forge test -vvv          # Very verbose (includes traces)
forge test --match-test testModelRegistry  # Run specific test

# Gas snapshots
forge snapshot

# Deploy contracts to local node
forge script script/Deploy.s.sol --rpc-url http://localhost:8545 --broadcast

# Deploy to testnet (with private key)
forge script script/Deploy.s.sol --rpc-url $TESTNET_RPC --private-key $PRIVATE_KEY --broadcast

# Format Solidity
forge fmt

# Interact with contracts using cast
cast call <CONTRACT_ADDR> "symbol()" --rpc-url http://localhost:8545
cast send <CONTRACT_ADDR> "transfer(address,uint256)" <TO> <AMOUNT> --private-key $PRIVATE_KEY

# Check contract storage
cast storage <CONTRACT_ADDR> <SLOT> --rpc-url http://localhost:8545
```

### SDK Commands
```bash
# Official TypeScript SDK (@citrate-ai/sdk)
cd sdk/javascript
npm install
npm run build
npm test
npm run lint

# Alternative citrate-js SDK
cd sdks/javascript/citrate-js
npm install
npm run build
npm test

# Python SDK
cd sdks/python
pip install -e .
pytest
```

### Developer Tools
```bash
# Citrate Studio (Visual IDE)
cd developer-tools/lattice-studio
npm install
npm start                # Runs on port 3001
npm run build

# Debug Dashboard
cd developer-tools/debug-dashboard
npm install
npm run dev

# Documentation Portal
cd developer-tools/documentation-portal
npm install
npm run build
```

### Testing Scripts
```bash
# Start local testnet (from citrate/)
./scripts/start_testnet.sh --consensus ghostdag

# Deploy test contracts
./scripts/deploy_contracts.sh

# Send test transaction
./scripts/send_test_tx.sh

# Cluster management
./scripts/cluster_smoke.sh
./scripts/cluster_down.sh

# Smoke tests
./scripts/smoke_inference.sh
```

## Module APIs

### Consensus (`citrate-consensus`)
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

### Storage (`citrate-storage`)
- StateDB: Merkle Patricia Trie for account state
- ChainStore: Block and transaction storage with RocksDB
- TransactionStore: Receipt storage and indexing

### Execution (`citrate-execution`)
- Executor: EVM-compatible transaction execution
- Precompiles: AI model operations and ZKP verification
- Address utilities: Handle 20-byte EVM addresses in 32-byte fields

### API (`citrate-api`)
```rust
// JSON-RPC server
use citrate_api::EthRpc;

// Start RPC server
let rpc = EthRpc::new(executor, storage, network);
rpc.serve("127.0.0.1:8545").await?;
```

### Network (`citrate-network`)
```rust
// P2P networking
use citrate_network::{Network, PeerConfig};

// Initialize network
let network = Network::new(config).await?;
network.start().await?;

// Broadcast block
network.broadcast_block(&block).await?;
```

## Transaction Pipeline Architecture

### ✅ Recent Fixes (Post-Audit)
All critical transaction pipeline issues have been resolved as of the latest audit:

1. ✅ **GUI Producer Execution**: GUI producer now executes transactions and persists receipts (committed at line 374)
2. ✅ **EIP-1559 Support**: Full support for typed transactions (EIP-1559 and EIP-2930) implemented
3. ✅ **Address Derivation**: Smart handling of both embedded EVM addresses and full 32-byte public keys
4. ✅ **Pending Nonce**: `eth_getTransactionCount` supports "pending" tag with mempool transaction inclusion
5. ✅ **Mempool Visibility**: `citrate_getMempoolSnapshot` RPC endpoint provides full mempool observability

### Transaction Flow Paths

#### CLI Wallet → RPC → Node
```
1. wallet/src/transaction.rs - Signs with ed25519
2. wallet/src/rpc_client.rs - Sends via eth_sendRawTransaction
3. core/api/src/eth_tx_decoder.rs - Decodes bincode/RLP/EIP-1559/EIP-2930
4. core/sequencer/src/mempool.rs - Validates and stores
5. node/src/producer.rs - Executes and stores receipts
```

#### GUI Wallet → Embedded Node
```
1. gui/../wallet_manager.rs - Creates and signs transaction
2. gui/../lib.rs:195 - Adds directly to embedded mempool
3. gui/../block_producer.rs:140 - Executes transactions via executor
4. gui/../block_producer.rs:374 - Commits state changes (CRITICAL FIX)
5. gui/../block_producer.rs:204-223 - Stores receipts
```

### Key Implementation Details

#### Address Handling (`core/execution/src/types.rs:13-34`)
```rust
// Smart address derivation supporting both formats:
// 1. Embedded EVM (20 bytes + 12 zeros) → use directly
// 2. Full 32-byte pubkey → Keccak256 hash last 20 bytes
pub fn from_public_key(pubkey: &PublicKey) -> Self {
    let is_evm_address = pubkey.0[20..].iter().all(|&b| b == 0)
        && !pubkey.0[..20].iter().all(|&b| b == 0);

    if is_evm_address {
        // Direct mapping
    } else {
        // Keccak256 derivation
    }
}
```

#### EIP-1559 Support (`core/api/src/eth_tx_decoder.rs:78-630`)
- Detects transaction type byte (0x01, 0x02)
- Full RLP decoding for typed transactions
- Access list parsing
- yParity-based signature recovery

#### Pending Nonce (`core/api/src/eth_rpc.rs:522-544`)
```rust
// Supports both "latest" and "pending" tags
if tag.eq_ignore_ascii_case("pending") {
    // Scans mempool for sender's pending transactions
    // Returns max(mempool_nonce + 1, base_nonce)
}
```

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
- Custom `citrate_*` for DAG queries
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
cargo test -p citrate-consensus
cargo test -p citrate-execution

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
cargo test -p citrate-consensus --test dag_scenarios
```

### End-to-End Tests
```bash
# Start testnet and run E2E
./scripts/run_e2e_tests.sh

# Smoke tests
./scripts/smoke_inference.sh
```

### Contract Tests
```bash
# Run all Foundry tests
cd contracts
forge test

# Run with gas reporting
forge test --gas-report

# Coverage
forge coverage
```

## Performance Targets
- **Throughput**: 10,000+ TPS
- **Finality**: ≤12 seconds
- **Block Time**: 1-2 seconds
- **DAG Width**: Support 100+ parallel blocks

## Important Implementation Notes

1. **GhostDAG vs GHOST**: We use GhostDAG (DAG-based) not GHOST (tree-based)
2. **Signature Schemes**: Supporting both ed25519 (native) and ECDSA (EVM compatibility)
3. **Address Format**: Smart handling of addresses - embedded 20-byte EVM addresses (with trailing zeros) are used directly, full 32-byte public keys are Keccak256-hashed
4. **Nonce Management**: RPC supports both "latest" and "pending" tags; use "pending" for sequential transactions
5. **Producer Execution**: ✅ Both GUI and core node producers execute transactions and store receipts (fixed in recent audit)
6. **MCP Integration**: All AI operations go through MCP standard
7. **Storage**: Artifacts stored off-chain (IPFS/Arweave), referenced by CID
8. **SDK Paths**:
   - Use `sdk/javascript/` for the official `@citrate-ai/sdk` (published to npm)
   - Alternative `sdks/javascript/citrate-js/` and `sdks/python/` are available
9. **Developer Tools**: Citrate Studio provides visual IDE at port 3001, Debug Dashboard for monitoring
10. **Transaction Types**: Full support for legacy, EIP-2930, and EIP-1559 transactions via RLP decoding
11. **Mempool Visibility**: Use `citrate_getMempoolSnapshot` RPC method for debugging transaction status

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

# Get current block number
curl -X POST http://localhost:8545 -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}'
```

### Using Citrate Studio for Debugging
```bash
# Start Citrate Studio
cd developer-tools/lattice-studio
npm start

# Access at http://localhost:3001
# Features:
# - DAG visualization
# - Block explorer
# - Transaction inspector
# - Mempool monitor
# - Smart contract interaction
# - Monaco code editor for contracts
```

### Common Issues (Post-Audit Resolution)

#### ✅ Previously Known Issues (Now Fixed)
- ~~**Pending Forever**~~ - GUI producer now executes transactions correctly
- ~~**Wrong Balance**~~ - Address derivation fixed for both EVM and native formats
- ~~**Nonce Too Low**~~ - Pending nonce support implemented

#### Current Troubleshooting
- **Invalid Signature**: Check chainId and signature scheme compatibility (ed25519 vs ECDSA)
- **Contract Deployment Fails**: Verify gas limits and contract bytecode size
- **Transaction Not Found**: Use `citrate_getMempoolSnapshot` to check if transaction is in mempool
- **Gas Estimation**: Use `eth_estimateGas` before sending transactions

---

## Observability & Metrics

### Structured Logging

The node uses structured JSON logging with trace ID correlation for production observability.

#### Configuration

```bash
# Log format: json, pretty, or compact
export LOG_FORMAT=json

# Log level (trace, debug, info, warn, error)
export RUST_LOG=info,citrate_api=debug

# Log file output
export LOG_FILE=/var/log/citrate/node.log

# ANSI colors (for terminal)
export LOG_ANSI=true
```

#### Trace ID Correlation

Each request generates a unique trace ID (format: `{timestamp_hex}-{counter_hex}-{random_hex}`) that propagates through the call stack for easy log aggregation.

```rust
// Example log output (JSON format)
{"timestamp":"2025-12-04T...","level":"INFO","trace_id":"18c3f2a-1-a3b4","method":"eth_sendTransaction",...}
```

### Prometheus Metrics

The node exposes Prometheus-compatible metrics at `/metrics` endpoint.

#### Configuration

```bash
# Enable metrics server
export CITRATE_METRICS_ADDR=127.0.0.1:9090

# Start node (metrics auto-enabled)
cargo run --bin citrate -- devnet
```

#### Key Metrics

| Metric | Type | Description |
|--------|------|-------------|
| `citrate_node_uptime_seconds` | Gauge | Node uptime |
| `citrate_peer_count` | Gauge | Connected peers |
| `citrate_block_height` | Gauge | Current block height |
| `citrate_mempool_size` | Gauge | Transactions in mempool |
| `citrate_dag_tips_count` | Gauge | DAG tip count |
| `citrate_rpc_requests_total` | Counter | RPC requests by method |
| `citrate_rpc_latency_seconds` | Histogram | RPC latency distribution |
| `citrate_ai_requests_total` | Counter | AI inference requests |
| `citrate_ipfs_uploads_total` | Counter | IPFS uploads |

### GUI Error Handling

The GUI provides user-friendly error surfaces with:

- **Toast notifications** for transient errors (auto-dismiss after 8 seconds)
- **Modal dialogs** for blocking/critical errors
- **Error codes** for support reference (e.g., `NETWORK_OFFLINE`, `TX_FAILED`)
- **Report issue** functionality with error context

#### Error Categories

| Category | Description | Examples |
|----------|-------------|----------|
| `network` | Network connectivity issues | NETWORK_OFFLINE, NETWORK_TIMEOUT |
| `blockchain` | Blockchain/RPC errors | RPC_UNAVAILABLE, TX_REJECTED |
| `wallet` | Wallet operations | WALLET_LOCKED, INSUFFICIENT_FUNDS |
| `contract` | Smart contract errors | CONTRACT_REVERTED, CONTRACT_NOT_FOUND |
| `ipfs` | IPFS operations | IPFS_UNAVAILABLE, IPFS_UPLOAD_FAILED |
| `ai` | AI model operations | MODEL_NOT_FOUND, INFERENCE_FAILED |
| `validation` | Input validation | INVALID_INPUT, MISSING_REQUIRED |

### Dev Mode

Development builds include additional tooling:

```bash
# Enable dev mode (frontend)
export CITRATE_DEV_MODE=true

# Enable dev mode (Rust)
cargo build --features dev-mode
```

Dev mode features:
- Dev mode indicator badge (bottom-left of GUI)
- Console logging (stripped in production)
- Debug assertions
- Mock data capabilities (when enabled)

---

## 📋 Sprint Planning & Development Roadmap

### 🚨 ALWAYS CHECK SPRINT FOLDER FIRST

**Before starting any work, check the `.sprint/` folder for current sprint planning:**

```bash
# Check current sprint status
cat .sprint/CURRENT_SPRINT.md

# View full roadmap
cat .sprint/ROADMAP.md

# View product backlog
cat .sprint/BACKLOG.md

# View detailed sprint plan
cat .sprint/sprints/sprint-XX-name/SPRINT.md
```

### Sprint Folder Structure
```
.sprint/
├── README.md                    # How to use sprint planning
├── ROADMAP.md                   # Master roadmap with all phases
├── CURRENT_SPRINT.md            # Active sprint quick reference
├── BACKLOG.md                   # Product backlog (prioritized)
├── sprints/
│   └── sprint-XX-name/
│       ├── SPRINT.md            # Detailed sprint plan with WBS
│       ├── DAILY.md             # Daily progress tracking
│       └── RETRO.md             # Sprint retrospective
└── templates/                   # Templates for new sprints
```

### Current Initiative: AI-First GUI Transformation

**Goal**: Transform Citrate GUI into an AI-first conversational interface

**Timeline**: 14 weeks (7 sprints)

| Phase | Sprint | Focus | Status |
|-------|--------|-------|--------|
| Phase 0 | Sprint 0 | Critical Infrastructure Fixes | **ACTIVE** |
| Phase 1 | Sprint 1 | Agent Foundation | Blocked |
| Phase 2 | Sprint 2 | Frontend Redesign | Blocked |
| Phase 3 | Sprint 3-4 | Tool Implementation | Blocked |
| Phase 4 | Sprint 5 | Multi-Window & Terminal | Blocked |
| Phase 5 | Sprint 6 | Polish & Release | Blocked |

### Current Sprint: Sprint 0 - Critical Fixes

**Sprint Goal**: Fix all critical blockers preventing reliable blockchain operation

**Key Work Packages**:
1. Fix consensus test compilation
2. Implement total ordering (mergeset topological sort)
3. Fix network peer ID bugs (block/tx propagation)
4. Implement basic finality mechanism
5. Fix executor panic points
6. Implement ECRECOVER precompile

**Full Details**: See `.sprint/CURRENT_SPRINT.md`

## CI/CD & Release Process

### GitHub Actions Workflows
```bash
# Rust CI - runs on every PR and push to main
.github/workflows/rust-ci.yml

# Solidity CI - Foundry tests and Slither analysis
.github/workflows/solidity-ci.yml

# GUI Tauri - cross-platform builds
.github/workflows/gui-tauri.yml

# Release - triggered on version tags
.github/workflows/release.yml
```

### Creating a Release
```bash
# Tag and push (triggers automated release)
git tag v0.1.0
git push origin v0.1.0

# CI will automatically:
# - Build Node/CLI binaries for Linux/macOS/Windows
# - Build Tauri GUI app for all platforms
# - Upload to GitHub Releases
# - Generate release notes
```

## Network Configuration

### Config Files
- Node TOML configs: `node/config/` (devnet/testnet samples)
- GUI JSON configs: `gui/lattice-core/config/devnet.json`, `testnet.json`
- Switching networks in GUI updates ports, discovery, and `chainId` automatically

### Starting Different Networks
```bash
# Devnet (local single node)
cargo run --bin citrate-node -- devnet

# Testnet
cargo run --bin citrate-node -- --config node/config/testnet.toml

# Using orchestrator
scripts/lattice.sh testnet up
```

---

## 📚 Documentation Protocol & Single Sources of Truth

**CRITICAL**: This codebase follows strict documentation governance to prevent confusion and sprawl.

### Core Rules for AI Assistants & Contributors

1. **One Source of Truth Per Topic** - Never create duplicate documentation
2. **Check Before Creating** - Always consult `DOCUMENTATION_MATRIX.md` before writing new docs
3. **Link, Don't Duplicate** - Reference the authoritative doc, don't copy content
4. **Archive Historical Docs** - Completed work goes to `/archive/` with dates

### Key Reference Documents

| What You Need | File | Path |
|---------------|------|------|
| Documentation governance | DOCUMENTATION.md | `/DOCUMENTATION.md` |
| Quick doc lookup | DOCUMENTATION_MATRIX.md | `/DOCUMENTATION_MATRIX.md` |
| Current P0 roadmap | roadmap-p0.md | `/citrate/docs/roadmap-p0.md` |
| Main README | README.md | `/citrate/README.md` |
| Quick start | DEVNET_QUICKSTART.md | `/DEVNET_QUICKSTART.md` |
| Installation | installation.md | `/citrate/docs/guides/installation.md` |
| Deployment guide | deployment.md | `/citrate/docs/guides/deployment.md` |
| Genesis setup | genesis-startup.md | `/citrate/docs/guides/genesis-startup.md` |
| Wallet guide | wallet-and-rewards.md | `/citrate/docs/guides/wallet-and-rewards.md` |
| User docs portal | docs-portal/docs/ | `/docs-portal/docs/` |
| Whitepaper | lattice-whitepaper-final.md | `/citrate-docs-v3/lattice-whitepaper-final.md` |

### Prohibited Practices

❌ **Do NOT Create**:
- `*_PROGRESS.md` files (use `docs/roadmap-p0.md` with checkboxes)
- `*_COMPLETION.md` files (archive these immediately)
- `*_SUMMARY.md` files (archive these immediately)
- `*_PLAN.md` files for active work (archive old ones)
- `*_v2.md` version files (use git for versioning)
- README files in every subdirectory (only where they add unique value)
- Guides at root level (use `docs/guides/` for all operational guides)

✅ **Do Instead**:
- Update `docs/roadmap-p0.md` for current work status
- Update existing source of truth documents in `docs/guides/` or `docs/technical/`
- Archive completed documents to `/archive/` with dates
- Link to authoritative docs instead of copying content

### Archive Structure
```
archive/
├── audits/              # Dated audit reports (YYYY-MM-name.md)
├── deployment-guides/   # Old deployment docs
├── gui-docs/            # GUI-specific archived docs
├── implementations/     # Implementation plans
├── phase-history/       # Phase completion reports
├── roadmaps/            # Superseded roadmaps (YYYY-MM-name.md)
├── testing/             # Test reports
└── whitepapers/         # Old whitepaper versions
```

### Before Writing Any Documentation

**Mandatory Checklist**:
1. ✅ Check `DOCUMENTATION_MATRIX.md` - Does this topic exist?
2. ✅ Check `DOCUMENTATION.md` - What's the governance rule?
3. ✅ Check `/archive/` - Is there historical context to review?
4. ✅ Ask: Should this be in CLAUDE.md, README, or docs-portal?
5. ✅ Plan: When will this doc be archived? Who maintains it?

**For more details**: See `/DOCUMENTATION.md` for full governance protocol.
