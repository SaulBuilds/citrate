# Sprint 1: Core Infrastructure Fixes

## Sprint Info
- **Duration**: Week 1
- **Sprint Goal**: Fix all core blockchain infrastructure issues blocking testnet
- **Phase**: Audit Fixes - Foundation

---

## Sprint Objectives

1. Implement real block root calculations (state_root, receipt_root)
2. Make chain ID configurable across the stack
3. Implement efficient sync and DAG tracking
4. Wire AI handler message processing
5. Fix RPC stubs (eth_feeHistory, DAG explorer)

---

## Work Breakdown Structure (WBS)

### WP-1.1: Chain ID Configuration
**Points**: 3 | **Priority**: P0 | **Status**: [ ] Not Started

**Description**:
Chain ID is currently hardcoded to 1337 in the executor. Need to make it configurable via environment variable or config file, and ensure consistency across executor, RPC, wallet, and GUI.

**Tasks**:
- [ ] Add `chain_id` field to node config struct
- [ ] Read chain ID from CITRATE_CHAIN_ID env var or config.toml
- [ ] Update executor to use configured chain ID
- [ ] Update RPC `eth_chainId` to return configured value
- [ ] Update wallet signing to use configured chain ID
- [ ] Update GUI node config to pass chain ID
- [ ] Add chain ID to genesis block metadata

**Acceptance Criteria**:
- [ ] Chain ID can be set via env var CITRATE_CHAIN_ID
- [ ] Chain ID can be set via config.toml
- [ ] All components return consistent chain ID
- [ ] Default remains 1337 for devnet compatibility

**Files to Modify**:
- `core/execution/src/executor.rs:800`
- `core/api/src/eth_rpc.rs` (eth_chainId)
- `node/src/config.rs`
- `wallet/src/lib.rs`
- `gui/citrate-core/src-tauri/src/node/mod.rs`

**Dependencies**: None

---

### WP-1.2: Efficient Sync Implementation
**Points**: 8 | **Priority**: P0 | **Status**: [ ] Not Started

**Description**:
The efficient_sync.rs module is stubbed with `{processed:0}` and no real implementation. Need to implement block catch-up, header sync, and state reconstruction.

**Tasks**:
- [ ] Implement `sync_headers()` to fetch headers from peers
- [ ] Implement `sync_blocks()` to download full blocks
- [ ] Implement `reconstruct_state()` for state catch-up
- [ ] Add sync progress tracking and metrics
- [ ] Implement sync mode detection (full vs fast sync)
- [ ] Add sync timeout and retry logic
- [ ] Write integration tests for sync scenarios
- [ ] Add property tests for sync edge cases

**Acceptance Criteria**:
- [ ] Node can sync from genesis to tip
- [ ] Sync progress reported via metrics/logs
- [ ] Handles peer disconnection gracefully
- [ ] Tests cover: empty chain, single block, multi-block, reorg during sync

**Files to Modify**:
- `core/consensus/src/sync/efficient_sync.rs`
- `core/consensus/tests/real_tests.rs`
- `core/network/src/sync_protocol.rs` (if exists)

**Dependencies**: None

---

### WP-1.3: Genesis DAG Tracking
**Points**: 3 | **Priority**: P1 | **Status**: [ ] Not Started

**Description**:
Genesis initialization has a TODO for DAG tracking at node/src/genesis.rs:237. Need to properly initialize the DAG store with genesis block and set up tracking.

**Tasks**:
- [ ] Initialize DAG store with genesis block
- [ ] Set genesis as initial tip
- [ ] Track genesis blue score (0)
- [ ] Initialize GhostDAG params from genesis
- [ ] Add genesis block to block store
- [ ] Verify genesis hash matches expected

**Acceptance Criteria**:
- [ ] DAG store has genesis block after init
- [ ] get_tips() returns genesis initially
- [ ] Blue score calculation works from genesis

**Files to Modify**:
- `node/src/genesis.rs:237`
- `core/consensus/src/dag_store.rs`

**Dependencies**: None

---

### WP-1.4: AI Handler Message Wiring
**Points**: 5 | **Priority**: P1 | **Status**: [ ] Not Started

**Description**:
AI handler in network has stubs at lines 331, 415, 480. Training deltas aren't applied, message variants may be missing. Need full implementation.

**Tasks**:
- [ ] Implement training delta application (line 331)
- [ ] Implement inference result routing (line 415)
- [ ] Implement model update propagation (line 480)
- [ ] Add missing message variants for AI ops
- [ ] Implement rate limiting for AI messages
- [ ] Add validation for AI payloads
- [ ] Write tests for AI message handling

**Acceptance Criteria**:
- [ ] Training deltas are applied to local model state
- [ ] Inference requests are routed to appropriate handlers
- [ ] Model updates propagate through network
- [ ] Invalid AI messages are rejected

**Files to Modify**:
- `core/network/src/ai_handler.rs:331,415,480`
- `core/network/src/message.rs` (if message variants needed)

**Dependencies**: None

---

### WP-1.5: eth_feeHistory Implementation
**Points**: 3 | **Priority**: P1 | **Status**: [ ] Not Started

**Description**:
eth_feeHistory returns mock data. Need to implement real fee history based on block history.

**Tasks**:
- [ ] Collect gas usage from recent blocks
- [ ] Calculate base fee per block
- [ ] Calculate gas used ratio per block
- [ ] Implement reward percentile calculations
- [ ] Cache recent fee data for performance
- [ ] Handle edge case: fewer blocks than requested

**Acceptance Criteria**:
- [ ] Returns accurate fee history for last N blocks
- [ ] Percentile calculations match spec
- [ ] Works with EIP-1559 transactions

**Files to Modify**:
- `core/api/src/eth_rpc.rs:923-930`
- `core/storage/src/chain_store.rs` (for block fee queries)

**Dependencies**: None

---

### WP-1.6: DAG Explorer API Fixes
**Points**: 3 | **Priority**: P2 | **Status**: [ ] Not Started

**Description**:
DAG explorer has stubbed stats and transaction details at lines 486-493, 697-708, 550. Need real implementations.

**Tasks**:
- [ ] Implement `get_dag_stats()` with real metrics
- [ ] Implement `get_transaction_details()` with full tx info
- [ ] Implement `get_block_details()` with merge parents
- [ ] Add blue set information to block details
- [ ] Cache frequently requested stats

**Acceptance Criteria**:
- [ ] DAG stats show real block count, tip count, blue score range
- [ ] Transaction details include receipt data
- [ ] Block details show full DAG structure

**Files to Modify**:
- `core/api/src/dag_explorer.rs:486-493,697-708,550`

**Dependencies**: WP-1.2 (sync must work for accurate stats)

---

### WP-1.7: Genesis Model Embedding Fix
**Points**: 2 | **Priority**: P2 | **Status**: [ ] Not Started

**Description**:
Genesis model embedding at lines 114-129 uses mock data. Should either embed real minimal model or clearly mark as optional.

**Tasks**:
- [ ] Determine if genesis model is required for testnet
- [ ] If required: embed minimal working model (Qwen or similar)
- [ ] If optional: add flag to skip model embedding
- [ ] Update model verifier to handle missing genesis model
- [ ] Document genesis model requirements

**Acceptance Criteria**:
- [ ] Genesis block can be created without errors
- [ ] Model verifier handles genesis model correctly
- [ ] Clear documentation on model embedding

**Files to Modify**:
- `core/genesis/genesis_model.rs:114-129`
- `node/src/model_verifier.rs:75`

**Dependencies**: None

---

### WP-1.8: GhostDAG Property Tests
**Points**: 5 | **Priority**: P1 | **Status**: [ ] Not Started

**Description**:
Tip/ordering tests are thin. Need property tests and fuzz coverage for mergeset ordering and blue-set determinism.

**Tasks**:
- [ ] Add property tests for blue set calculation determinism
- [ ] Add fuzz tests for mergeset ordering
- [ ] Test edge cases: single parent, max parents, conflicting tips
- [ ] Add tests for reorg scenarios
- [ ] Verify ordering consistency across node restarts

**Acceptance Criteria**:
- [ ] Property tests verify deterministic blue set
- [ ] Fuzz tests find no ordering inconsistencies
- [ ] All edge cases covered

**Files to Modify**:
- `core/consensus/tests/real_tests.rs`
- `core/consensus/tests/property_tests.rs` (new)
- `core/consensus/tests/fuzz_tests.rs` (new)

**Dependencies**: WP-1.2

---

## Sprint Backlog Summary

| WP | Title | Points | Priority | Status |
|----|-------|--------|----------|--------|
| WP-1.1 | Chain ID Configuration | 3 | P0 | [ ] |
| WP-1.2 | Efficient Sync Implementation | 8 | P0 | [ ] |
| WP-1.3 | Genesis DAG Tracking | 3 | P1 | [ ] |
| WP-1.4 | AI Handler Message Wiring | 5 | P1 | [ ] |
| WP-1.5 | eth_feeHistory Implementation | 3 | P1 | [ ] |
| WP-1.6 | DAG Explorer API Fixes | 3 | P2 | [ ] |
| WP-1.7 | Genesis Model Embedding Fix | 2 | P2 | [ ] |
| WP-1.8 | GhostDAG Property Tests | 5 | P1 | [ ] |

**Total Points**: 32
**Committed Points**: 27 (excluding P2)
**Buffer**: 5 points (P2 items)

---

## Definition of Done

- [ ] Code compiles without warnings
- [ ] All tests pass (`cargo test --workspace`)
- [ ] Code reviewed (self-review minimum)
- [ ] Documentation updated
- [ ] Changes tested manually
- [ ] No critical security issues
- [ ] Chain ID is configurable and consistent
- [ ] Sync can catch up from genesis

---

## Risks & Blockers

| Risk | Impact | Mitigation |
|------|--------|------------|
| Sync implementation complexity | High | Start with simple header-first sync, iterate |
| AI handler message format unclear | Med | Review existing message types first |
| Genesis model size constraints | Low | Use minimal quantized model |

---

## Notes

- WP-1.1 (Chain ID) and WP-1.3 (Genesis DAG) should be tackled first as they're foundational
- WP-1.2 (Sync) is the largest work package and may need to be broken down further
- Block roots (state_root, receipt_root) were already fixed in previous session

---

*Created: 2025-12-04*
*Last Updated: 2025-12-04*
