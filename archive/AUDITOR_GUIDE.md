# Citrate Security Audit Guide

**Document Version**: 1.0
**Date**: 2025-12-03
**Prepared For**: Security Auditors
**Contact**: Project Maintainers

---

## Executive Summary

Citrate is an AI-native Layer-1 BlockDAG blockchain with an EVM-compatible execution environment. This document provides comprehensive guidance for security auditors to navigate the codebase, understand the architecture, and identify potential vulnerabilities.

### Key Components Under Audit

| Component | Priority | Location | Language |
|-----------|----------|----------|----------|
| Consensus (GhostDAG) | CRITICAL | `core/consensus/` | Rust |
| Execution (EVM) | CRITICAL | `core/execution/` | Rust |
| Smart Contracts | CRITICAL | `contracts/` | Solidity |
| Network/P2P | HIGH | `core/network/` | Rust |
| Transaction Pipeline | HIGH | `core/sequencer/` + `core/api/` | Rust |
| Wallet/Crypto | HIGH | `wallet/` | Rust |
| GUI Agent | MEDIUM | `gui/citrate-core/src-tauri/` | Rust |
| Frontend | LOW | `gui/citrate-core/src/` | TypeScript |

---

## Table of Contents

1. [Architecture Overview](#1-architecture-overview)
2. [Consensus Layer (GhostDAG)](#2-consensus-layer-ghostdag)
3. [Execution Layer (EVM)](#3-execution-layer-evm)
4. [Transaction Pipeline](#4-transaction-pipeline)
5. [Cryptographic Implementations](#5-cryptographic-implementations)
6. [Network Layer](#6-network-layer)
7. [Smart Contracts](#7-smart-contracts)
8. [Known Issues & Technical Debt](#8-known-issues--technical-debt)
9. [Security Concerns from Developers](#9-security-concerns-from-developers)
10. [Testing Coverage](#10-testing-coverage)
11. [Recommended Audit Focus Areas](#11-recommended-audit-focus-areas)

---

## 1. Architecture Overview

### System Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                        Citrate Node                             │
├─────────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────┐  │
│  │   API       │  │  Network    │  │      Storage            │  │
│  │ (JSON-RPC)  │  │   (P2P)     │  │  (RocksDB + MPT)        │  │
│  └──────┬──────┘  └──────┬──────┘  └───────────┬─────────────┘  │
│         │                │                     │                │
│  ┌──────▼────────────────▼─────────────────────▼─────────────┐  │
│  │                    Sequencer                              │  │
│  │              (Mempool + Block Production)                 │  │
│  └──────────────────────────┬────────────────────────────────┘  │
│                             │                                   │
│  ┌──────────────────────────▼────────────────────────────────┐  │
│  │                    Consensus                              │  │
│  │                    (GhostDAG)                             │  │
│  └──────────────────────────┬────────────────────────────────┘  │
│                             │                                   │
│  ┌──────────────────────────▼────────────────────────────────┐  │
│  │                    Execution                              │  │
│  │              (EVM + Precompiles)                          │  │
│  └───────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
```

### Directory Structure

```
citrate/
├── core/
│   ├── consensus/          # GhostDAG implementation
│   │   ├── src/
│   │   │   ├── ghostdag.rs      # Core GHOSTDAG algorithm
│   │   │   ├── dag_store.rs     # DAG storage & queries
│   │   │   ├── tip_selection.rs # Tip selection logic
│   │   │   ├── chain_selection.rs
│   │   │   ├── finality.rs      # Finality checkpoints
│   │   │   ├── ordering.rs      # Total ordering
│   │   │   └── types.rs         # Block/DAG types
│   │   └── tests/
│   │
│   ├── execution/          # EVM execution
│   │   ├── src/
│   │   │   ├── executor.rs      # Transaction executor
│   │   │   ├── vm/              # EVM implementation
│   │   │   │   ├── evm.rs
│   │   │   │   ├── evm_opcodes.rs
│   │   │   │   └── evm_memory.rs
│   │   │   ├── precompiles/     # Precompiled contracts
│   │   │   │   ├── mod.rs
│   │   │   │   ├── ecrecover.rs
│   │   │   │   ├── sha256.rs
│   │   │   │   └── ai_inference.rs  # AI model precompile
│   │   │   └── revm_adapter.rs  # REVM integration
│   │   └── tests/
│   │
│   ├── sequencer/          # Mempool & block production
│   │   ├── src/
│   │   │   ├── mempool.rs
│   │   │   ├── block_builder.rs
│   │   │   └── parent_selection.rs
│   │   └── tests/
│   │
│   ├── network/            # P2P networking
│   │   ├── src/
│   │   │   ├── block_propagation.rs
│   │   │   ├── transaction_gossip.rs
│   │   │   └── transport.rs
│   │   └── tests/
│   │
│   ├── api/                # JSON-RPC API
│   │   ├── src/
│   │   │   ├── eth_rpc.rs       # Ethereum-compatible RPC
│   │   │   └── eth_tx_decoder.rs # Transaction decoding
│   │   └── tests/
│   │
│   ├── storage/            # Persistence layer
│   │   └── src/
│   │
│   └── primitives/         # Core types
│       └── src/
│           ├── types.rs
│           └── crypto.rs
│
├── contracts/              # Solidity smart contracts
│   ├── src/
│   │   ├── CitrateToken.sol
│   │   ├── ModelRegistry.sol
│   │   ├── Marketplace.sol
│   │   └── governance/
│   └── test/
│
├── wallet/                 # CLI wallet
│   └── src/
│       ├── transaction.rs
│       └── rpc_client.rs
│
└── gui/citrate-core/       # Tauri GUI application
    ├── src-tauri/src/
    │   ├── lib.rs           # Main Tauri commands
    │   ├── agent/           # AI agent implementation
    │   ├── node/            # Embedded node
    │   └── wallet_manager.rs
    └── src/                 # React frontend
```

---

## 2. Consensus Layer (GhostDAG)

### Overview

Citrate uses GhostDAG (not GHOST) - a DAG-based consensus protocol where:
- Each block has one `selected_parent` and 0+ `merge_parents`
- The "blue set" contains blocks consistent with the k-cluster rule
- Total ordering is derived from blue scores

### Critical Files

| File | Purpose | Risk Level |
|------|---------|------------|
| `consensus/src/ghostdag.rs` | Core GHOSTDAG algorithm | CRITICAL |
| `consensus/src/dag_store.rs` | DAG storage and queries | HIGH |
| `consensus/src/tip_selection.rs` | Selects parents for new blocks | HIGH |
| `consensus/src/finality.rs` | Finality checkpoints | CRITICAL |
| `consensus/src/ordering.rs` | Total ordering derivation | CRITICAL |

### Key Algorithms

#### Blue Set Calculation (`ghostdag.rs`)
```rust
// Simplified logic
pub async fn calculate_blue_set(&self, block: &Block) -> Result<Vec<Hash>> {
    let mut blue_set = self.get_blue_set(&block.selected_parent_hash).await?;

    for merge_parent in &block.merge_parent_hashes {
        if self.is_k_cluster_consistent(&blue_set, merge_parent).await? {
            blue_set.push(*merge_parent);
        }
    }

    Ok(blue_set)
}
```

#### Tip Selection (`tip_selection.rs`)
```rust
pub async fn select_parents(&self, max_parents: usize) -> Result<(Hash, Vec<Hash>)> {
    let tips = self.dag_store.get_tips().await;

    // Select highest blue-score tip as selected parent
    let selected_parent = tips.iter()
        .max_by_key(|tip| self.get_blue_score(tip))
        .unwrap();

    // Select additional merge parents
    let merge_parents = self.select_merge_parents(&tips, selected_parent, max_parents - 1);

    Ok((*selected_parent, merge_parents))
}
```

### Security Concerns

1. **K-Cluster Rule Enforcement**: Verify that the k-cluster rule cannot be bypassed
2. **Blue Score Manipulation**: Check for blue score inflation attacks
3. **Tip Starvation**: Can an attacker cause legitimate tips to be ignored?
4. **Finality Guarantees**: Are checkpoints immutable once committed?
5. **Ordering Determinism**: Is total ordering deterministic across all nodes?

### Known Issues

- **Sprint 0 Fix**: Total ordering (mergeset topological sort) was recently implemented
- **Test Coverage**: `consensus/tests/real_tests.rs` contains basic tests but edge cases need more coverage

---

## 3. Execution Layer (EVM)

### Overview

Citrate implements an EVM-compatible execution environment with:
- Full EVM opcode support
- Custom precompiles for AI operations
- REVM adapter for complex operations

### Critical Files

| File | Purpose | Risk Level |
|------|---------|------------|
| `execution/src/executor.rs` | Transaction execution | CRITICAL |
| `execution/src/vm/evm.rs` | EVM implementation | CRITICAL |
| `execution/src/vm/evm_opcodes.rs` | Opcode handlers | CRITICAL |
| `execution/src/precompiles/` | Precompiled contracts | HIGH |
| `execution/src/revm_adapter.rs` | REVM integration | HIGH |

### Precompiles

| Address | Name | Purpose |
|---------|------|---------|
| 0x01 | ECRECOVER | Signature recovery |
| 0x02 | SHA256 | SHA-256 hash |
| 0x03 | RIPEMD160 | RIPEMD-160 hash |
| 0x04 | IDENTITY | Data copy |
| 0x05 | MODEXP | Modular exponentiation |
| 0x06 | BN_ADD | BN128 addition |
| 0x07 | BN_MUL | BN128 multiplication |
| 0x08 | BN_PAIRING | BN128 pairing |
| 0x09 | BLAKE2F | BLAKE2 compression |
| 0x100 | AI_INFERENCE | AI model inference (custom) |

### Security Concerns

1. **Gas Metering**: Verify all opcodes consume correct gas
2. **Integer Overflow**: Check for overflow in arithmetic operations
3. **Stack Bounds**: Verify stack depth limits enforced
4. **Memory Expansion**: Check memory cost calculations
5. **ECRECOVER**: Signature malleability handling
6. **Reentrancy**: State changes before external calls
7. **AI Precompile**: Resource exhaustion, model validation

### Address Derivation

**IMPORTANT**: Citrate supports two address formats:

```rust
// From execution/src/types.rs:13-34
pub fn from_public_key(pubkey: &PublicKey) -> Self {
    // Check if this is an embedded EVM address (20 bytes + 12 zero bytes)
    let is_evm_address = pubkey.0[20..].iter().all(|&b| b == 0)
        && !pubkey.0[..20].iter().all(|&b| b == 0);

    if is_evm_address {
        // Direct mapping for embedded EVM addresses
        let mut addr = [0u8; 20];
        addr.copy_from_slice(&pubkey.0[..20]);
        Address(addr)
    } else {
        // Keccak256 derivation for full 32-byte public keys
        let hash = keccak256(&pubkey.0);
        let mut addr = [0u8; 20];
        addr.copy_from_slice(&hash[12..32]);
        Address(addr)
    }
}
```

This dual-format handling should be audited for:
- Collision attacks between the two formats
- Consistent behavior across all components

---

## 4. Transaction Pipeline

### Flow

```
1. User signs transaction (ed25519 or ECDSA)
       │
       ▼
2. Transaction submitted via RPC (eth_sendRawTransaction)
       │
       ▼
3. Decoded in eth_tx_decoder.rs (bincode/RLP/EIP-1559/EIP-2930)
       │
       ▼
4. Validated and added to mempool (sequencer/src/mempool.rs)
       │
       ▼
5. Block producer includes in block
       │
       ▼
6. Executed by executor (execution/src/executor.rs)
       │
       ▼
7. Receipt stored, state committed
```

### Critical Files

| File | Purpose | Risk Level |
|------|---------|------------|
| `api/src/eth_tx_decoder.rs` | Transaction decoding | HIGH |
| `sequencer/src/mempool.rs` | Transaction validation | HIGH |
| `execution/src/executor.rs` | Transaction execution | CRITICAL |
| `gui/.../block_producer.rs` | GUI block production | HIGH |

### Transaction Types Supported

- Legacy transactions (RLP encoded)
- EIP-2930 (Access list transactions)
- EIP-1559 (Fee market transactions)
- Native bincode format

### Security Concerns

1. **Nonce Management**: Verify nonce gaps handled correctly
2. **Signature Verification**: Both ed25519 and ECDSA paths
3. **Fee Calculation**: EIP-1559 fee handling
4. **Mempool Limits**: DOS protection
5. **Transaction Replay**: Chain ID validation
6. **yParity Recovery**: Correct signature recovery for typed transactions

### Recent Fixes (Sprint 0)

- GUI block producer now executes transactions (previously transactions were pending forever)
- Pending nonce support added for `eth_getTransactionCount`
- Address derivation fixed for both EVM and native formats

---

## 5. Cryptographic Implementations

### Signature Schemes

| Scheme | Usage | Library |
|--------|-------|---------|
| Ed25519 | Native transactions | `ed25519-dalek` |
| ECDSA secp256k1 | EVM compatibility | `k256` |

### Key Files

| File | Purpose |
|------|---------|
| `primitives/src/crypto.rs` | Cryptographic primitives |
| `wallet/src/transaction.rs` | Transaction signing |
| `execution/src/precompiles/ecrecover.rs` | ECRECOVER precompile |

### Security Concerns

1. **Key Generation**: Randomness source quality
2. **Signature Verification**: Timing attacks
3. **Hash Functions**: Collision resistance
4. **Nonce Reuse**: Prevention in ECDSA signing

---

## 6. Network Layer

### Overview

P2P networking using libp2p with:
- Block propagation
- Transaction gossip
- Peer discovery

### Critical Files

| File | Purpose | Risk Level |
|------|---------|------------|
| `network/src/block_propagation.rs` | Block broadcasting | HIGH |
| `network/src/transaction_gossip.rs` | Transaction gossip | MEDIUM |
| `network/src/transport.rs` | Network transport | MEDIUM |

### Security Concerns

1. **Eclipse Attacks**: Peer selection diversity
2. **Sybil Attacks**: Peer authentication
3. **Message Validation**: Invalid message handling
4. **DOS Protection**: Rate limiting
5. **Peer ID Handling**: Recent fix for block/tx propagation bugs

---

## 7. Smart Contracts

### Contracts Under Audit

| Contract | Location | Purpose |
|----------|----------|---------|
| `CitrateToken.sol` | `contracts/src/` | Native token |
| `ModelRegistry.sol` | `contracts/src/` | AI model registry |
| `Marketplace.sol` | `contracts/src/` | Model marketplace |
| `Governance.sol` | `contracts/src/governance/` | DAO governance |

### Testing

```bash
cd contracts
forge test -vvv
forge coverage
```

### Security Concerns

1. **Access Control**: Role-based permissions
2. **Reentrancy**: State changes before external calls
3. **Integer Overflow**: SafeMath usage
4. **Front-running**: MEV considerations
5. **Upgrade Patterns**: Proxy implementation safety

---

## 8. Known Issues & Technical Debt

### CRITICAL Stubs/Mocks (8 items - MUST FIX)

| # | Location | Issue | Impact |
|---|----------|-------|--------|
| 1 | `gui/.../node/mod.rs:20,652-654` | RPC server disabled - mempool type mismatch | GUI can't interact with blockchain |
| 2 | `gui/.../lib.rs:785-792` | eth_call returns error instead of executing | Contract reads fail |
| 3 | `core/api/src/eth_tx_decoder.rs:665-710` | Mock transaction creation with hardcoded addresses | Security: invalid txs in mempool |
| 4 | `core/api/src/dag_explorer.rs:486-493,697-708` | Network stats hardcoded to zero/empty | Explorer shows wrong data |
| 5 | `core/consensus/src/sync/efficient_sync.rs:398-404` | Returns `{processed:0}` without processing | Sync is non-functional |
| 6 | `core/sequencer/src/block_builder.rs:306-320` | State/receipt roots return `[1;32]`/`[2;32]` | Block verification broken |
| 7 | `core/network/src/ai_handler.rs:331-336` | AI results return `Ok(None)` | AI network propagation broken |
| 8 | `core/mcp/src/verification.rs:150-164` | ZK verification accepts empty proofs | Security: fake proofs accepted |

### HIGH Priority Stubs (7 items)

| # | Location | Issue |
|---|----------|-------|
| 9 | `core/api/src/methods/ai.rs:560-575` | Training jobs query returns empty vec |
| 10 | `gui/.../src/components/IPFS.tsx:80-152` | IPFS shows mock data (5 TODOs) |
| 11 | `gui/.../src/utils/contractDeployment.ts:113-128` | Receipt polling not implemented |
| 12 | `gui/.../src/utils/marketplaceHelpers.ts:99,124` | Model metadata not fetched |
| 13 | `gui/.../src-tauri/src/agent/tools/generation.rs:145-171` | Image gen returns mock results |
| 14 | `gui/.../src-tauri/src/agent/tools/storage.rs:148-174` | IPFS upload returns fake CID |
| 15 | `gui/.../src/components/ChatBot.tsx:110-113` | MCP service not connected |

### MEDIUM Priority (15 items)

| # | Location | Issue |
|---|----------|-------|
| 16 | `core/execution/src/inference/metal_runtime.rs:239` | Inference time hardcoded 5.0ms |
| 17 | `core/execution/src/inference/coreml_bridge.rs:135` | Model metadata not extracted |
| 18 | `core/execution/src/executor.rs:800` | Chain ID hardcoded to 1337 |
| 19 | `core/marketplace/src/search.rs:216` | Search highlighting empty |
| 20 | `core/network/src/ai_handler.rs:415` | Training job owner hardcoded |
| 21 | `core/network/src/ai_handler.rs:480` | Model weight delta not applied |
| 22 | `node/src/genesis.rs:237` | Genesis DAG tracking not initialized |
| 23 | `node/src/model_verifier.rs:75` | Cannot get validator list |
| 24 | `core/marketplace/src/analytics_engine.rs:318-334` | Analytics all zeros |
| 25 | `core/api/src/eth_rpc.rs:923-930` | eth_feeHistory mock data |
| 26 | `gui/.../src/agent/llm/mod.rs:140-177,207` | Falls back to mock LLM |
| 27 | `gui/.../src/components/ChatBot.tsx:241,244` | Speech recognition TODOs |
| 28 | `core/api/src/dag_explorer.rs:550` | Address search not implemented |
| 29 | `core/consensus/src/sync/efficient_sync.rs:415-427` | Empty test functions |
| 30 | `core/marketplace/src/search.rs:216` | Highlighting missing |

### LOW Priority (5 items)

| # | Location | Issue |
|---|----------|-------|
| 31 | `gui/.../src/hooks/useReviews.ts:112-180` | Review voting TODOs |
| 32 | `gui/.../src/utils/ipfsUploader.ts:288` | Metadata update not implemented |
| 33 | `contracts/MARKETPLACE_TESTING.md:239` | Frontend rate limiting TODO |
| 34 | `contracts/MARKETPLACE_TESTING.md:240` | CID validation TODO |
| 35 | `core/genesis/genesis_model.rs:114-129` | Mock embedding generation |

### Summary

| Severity | Count | Status |
|----------|-------|--------|
| **Critical** | 8 | Needs immediate attention |
| **High** | 7 | Should fix before release |
| **Medium** | 15 | Should fix before mainnet |
| **Low** | 5 | Nice to have |
| **Total** | **35** | |

### Previously Known Issues

| Issue | Location | Status | Description |
|-------|----------|--------|-------------|
| SEARCH_WEB tool | `gui/.../tools/` | Deferred | Requires external API, backlogged |
| Block fetch errors | `gui/.../dag.rs` | Known | Debug errors for missing blocks at height 2747+ (dev environment) |

### Compiler Warnings

The following warnings exist in the codebase:

```
- Unused imports in agent modules
- Unused variables in EVM opcodes (intentionally popped for stack effects)
- Dead code in formatting/orchestrator modules
```

---

## 9. Security Concerns from Developers

### High Priority Concerns

1. **Consensus Safety**: The GhostDAG implementation is relatively new. While based on the academic paper, edge cases may not be fully covered.

2. **EVM Compatibility**: Custom EVM implementation alongside REVM adapter. Discrepancies between the two could lead to issues.

3. **AI Precompile**: The AI inference precompile (`0x100`) is novel and hasn't been battle-tested. Resource exhaustion and model validation are concerns.

4. **Dual Signature Scheme**: Supporting both ed25519 and ECDSA adds complexity. Cross-scheme issues possible.

5. **GUI Embedded Node**: The GUI embeds a full node which runs in-process. Isolation concerns if GUI is compromised.

### Medium Priority Concerns

6. **IPFS Integration**: Model storage relies on IPFS. CID verification and integrity checks should be audited.

7. **Agent Tool Execution**: The AI agent can execute tools (terminal commands, file operations). Sandboxing should be verified.

8. **Mempool Management**: No formal mempool size limits documented. DOS vector possible.

9. **Finality Mechanism**: Checkpoint-based finality is relatively simple. Liveness concerns in network partitions.

### Low Priority Concerns

10. **Frontend Security**: React frontend handles wallet operations. XSS and CSRF protections should be verified.

---

## 10. Testing Coverage

### Unit Tests

```bash
# Run all tests
cargo test --workspace

# Run specific package tests
cargo test -p citrate-consensus
cargo test -p citrate-execution
cargo test -p citrate-network

# Run with output
cargo test -- --nocapture
```

### Integration Tests

```bash
# Consensus integration tests
cargo test -p citrate-consensus --test integration

# E2E tests
./scripts/run_e2e_tests.sh
```

### Contract Tests

```bash
cd contracts
forge test -vvv
forge coverage
```

### Coverage Gaps

| Component | Coverage | Notes |
|-----------|----------|-------|
| Consensus | ~60% | Edge cases need more tests |
| Execution | ~70% | Good opcode coverage |
| Network | ~40% | Integration tests lacking |
| Contracts | ~80% | Good coverage |

---

## 11. Recommended Audit Focus Areas

### CRITICAL (Must Audit)

1. **GhostDAG Blue Set Calculation** (`consensus/src/ghostdag.rs`)
   - K-cluster rule enforcement
   - Blue score computation correctness

2. **Transaction Execution** (`execution/src/executor.rs`)
   - State transition correctness
   - Gas metering accuracy

3. **Signature Verification** (multiple files)
   - Ed25519 and ECDSA paths
   - Address derivation logic

4. **Smart Contracts** (`contracts/src/`)
   - Standard vulnerability checks
   - Access control

### HIGH (Should Audit)

5. **Mempool & Block Production** (`sequencer/`)
   - Validation logic
   - DOS protection

6. **Transaction Decoding** (`api/src/eth_tx_decoder.rs`)
   - All transaction type handling
   - RLP parsing edge cases

7. **Network Message Handling** (`network/`)
   - Message validation
   - Peer management

### MEDIUM (Recommended)

8. **AI Precompiles** (`execution/src/precompiles/ai_inference.rs`)
   - Resource limits
   - Input validation

9. **GUI Agent Sandbox** (`gui/.../agent/`)
   - Tool execution safety
   - Permission model

---

## Appendix A: Build Instructions

```bash
# Clone and build
git clone <repo>
cd citrate

# Build all Rust components
cargo build --release

# Build contracts
cd contracts
forge build

# Run GUI
cd gui/citrate-core
npm install
npm run tauri dev
```

## Appendix B: Environment Setup

```bash
# Required tools
- Rust 1.75+
- Node.js 18+
- Foundry (forge, cast, anvil)
- IPFS (kubo) - optional

# Environment variables
export RUST_LOG=debug
export CITRATE_DATA_DIR=~/.citrate
```

## Appendix C: Useful Commands

```bash
# Check transaction status
curl -X POST http://localhost:8545 -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"eth_getTransactionReceipt","params":["0xHASH"],"id":1}'

# View mempool
curl -X POST http://localhost:8545 -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"citrate_getMempoolSnapshot","params":[],"id":1}'

# Get block by number
curl -X POST http://localhost:8545 -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"eth_getBlockByNumber","params":["latest",true],"id":1}'
```

---

## Contact Information

For questions during the audit:
- Create issues in the repository
- Tag with `audit` label

---

*Document prepared by the Citrate development team*
*Last updated: 2025-12-03*
