# Sprint 0: Critical Infrastructure Fixes

## Sprint Info
- **Duration**: 2 weeks (10 working days)
- **Sprint Goal**: Fix all critical blockers preventing reliable blockchain operation
- **Phase**: Phase 0 - Critical Fixes (BLOCKER)
- **Status**: READY TO START

---

## Sprint Objectives

1. **Make the codebase compile** - Fix all compilation errors in consensus tests
2. **Enable deterministic execution** - Implement total ordering for transactions
3. **Fix network propagation** - Blocks and transactions route to correct peers
4. **Establish safety guarantees** - Basic finality mechanism
5. **Prevent crashes** - Convert panic points to proper error handling
6. **Enable signature verification** - Implement ECRECOVER precompile

---

## Work Breakdown Structure (WBS)

### WP-0.1: Fix Consensus Test Compilation
**Points**: 2 | **Priority**: P0 | **Status**: [ ] Not Started

**Description**:
The consensus module tests fail to compile because Block struct initializers are missing the new `embedded_models` and `required_pins` fields added to the Block type.

**Tasks**:
- [ ] Add `embedded_models: vec![]` to Block initializers in `chain_selection.rs:428`
- [ ] Add `required_pins: vec![]` to Block initializers in `chain_selection.rs:428`
- [ ] Fix Block initializer in `dag_store.rs:286`
- [ ] Fix Block initializer in `ghostdag.rs:397`
- [ ] Fix Block initializer in `tip_selection.rs:356`
- [ ] Fix Block initializer in `types.rs:551`
- [ ] Run `cargo test -p citrate-consensus` to verify

**Acceptance Criteria**:
- [ ] `cargo build -p citrate-consensus` succeeds
- [ ] `cargo test -p citrate-consensus` compiles and runs
- [ ] No new warnings introduced

**Files to Modify**:
- `citrate/core/consensus/src/chain_selection.rs`
- `citrate/core/consensus/src/dag_store.rs`
- `citrate/core/consensus/src/ghostdag.rs`
- `citrate/core/consensus/src/tip_selection.rs`
- `citrate/core/consensus/src/types.rs`

**Dependencies**: None (start here)

---

### WP-0.2: Implement Total Ordering (Mergeset Topological Sort)
**Points**: 13 | **Priority**: P0 | **Status**: [ ] Not Started

**Description**:
GhostDAG requires a deterministic total ordering of all blocks/transactions. Currently, the chain selection only follows the selected-parent chain and ignores merge parents. We need to implement:
1. Topological sort of the mergeset
2. Deterministic ordering within each level (by blue score, then hash)
3. Integration with execution layer for transaction ordering

**Tasks**:
- [ ] Create new file `citrate/core/consensus/src/ordering.rs`
- [ ] Implement `TotalOrderIterator` struct that yields blocks in order
- [ ] Implement `get_ordered_blocks(from_hash, to_hash)` function
- [ ] Add mergeset collection (blocks reachable via merge parents)
- [ ] Implement topological sort respecting blue scores
- [ ] Add deterministic tiebreaker (lexicographic hash comparison)
- [ ] Integrate with `chain_selection.rs` to expose ordered blocks
- [ ] Add comprehensive tests for various DAG shapes
- [ ] Document the ordering algorithm

**Acceptance Criteria**:
- [ ] Given any two nodes, they produce identical block orderings
- [ ] All reachable blocks are included (no orphans)
- [ ] Ordering is stable across runs
- [ ] Performance: O(n log n) for n blocks
- [ ] Tests cover: linear chain, simple fork, complex DAG with multiple merges

**Files to Create**:
- `citrate/core/consensus/src/ordering.rs`

**Files to Modify**:
- `citrate/core/consensus/src/lib.rs` (add module)
- `citrate/core/consensus/src/chain_selection.rs` (integrate ordering)

**Dependencies**: WP-0.1

**Algorithm Reference**:
```
TotalOrder(block):
  1. Let S = selected-parent chain from genesis to block
  2. For each block B in S (oldest to newest):
     a. Yield B
     b. Let M = mergeset(B) sorted by (blue_score DESC, hash ASC)
     c. For each merge block MB in M:
        - Recursively yield TotalOrder(MB) (if not already yielded)
  3. Deduplicate: only yield each block once (first occurrence)
```

---

### WP-0.3: Fix Block Propagation Peer IDs
**Points**: 3 | **Priority**: P0 | **Status**: [ ] Not Started

**Description**:
In `block_propagation.rs:114`, the code generates random peer IDs instead of extracting actual peer IDs from the peer object. This breaks all block propagation.

**Tasks**:
- [ ] Identify correct peer ID accessor in peer object
- [ ] Fix `propagate_block` function (lines 103-137)
- [ ] Update `target_peers` collection to use real peer IDs
- [ ] Add logging for propagation debugging
- [ ] Write test to verify blocks reach intended peers

**Acceptance Criteria**:
- [ ] Blocks propagate to connected peers (not random IDs)
- [ ] Block announced to all peers except sender
- [ ] No duplicate announcements
- [ ] Test verifies correct routing

**Files to Modify**:
- `citrate/core/network/src/block_propagation.rs`

**Dependencies**: WP-0.1

**Current Bug Location**:
```rust
// Line 113-121 - WRONG
let target_peers: Vec<PeerId> = all_peers
    .iter()
    .filter_map(|_peer| {
        let peer_id = PeerId::new(format!("peer_{}", rand::random::<u64>())); // BUG!
        ...
    })
```

---

### WP-0.4: Fix Transaction Gossip Peer IDs
**Points**: 2 | **Priority**: P0 | **Status**: [ ] Not Started

**Description**:
Same issue as WP-0.3 but in `transaction_gossip.rs:184`. Random peer IDs prevent transaction relay.

**Tasks**:
- [ ] Fix `relay_transaction` function (lines 177-215)
- [ ] Extract real peer ID from peer object
- [ ] Ensure transaction reaches up to `max_relay_peers` actual peers
- [ ] Add test for transaction relay

**Acceptance Criteria**:
- [ ] Transactions relay to connected peers
- [ ] Respects `max_relay_peers` limit
- [ ] No relay to sender
- [ ] Test verifies correct routing

**Files to Modify**:
- `citrate/core/network/src/transaction_gossip.rs`

**Dependencies**: WP-0.1

---

### WP-0.5: Implement Basic Finality Mechanism
**Points**: 13 | **Priority**: P0 | **Status**: [ ] Not Started

**Description**:
Currently, finality is just a manual flag with no algorithm. We need a depth-based finality mechanism where blocks become final after reaching a certain confirmation depth.

**Tasks**:
- [ ] Create `citrate/core/consensus/src/finality.rs`
- [ ] Define `FinalityConfig` with `confirmation_depth` (default: 100 blocks)
- [ ] Implement `FinalityTracker` struct
- [ ] Add method `check_finality(block_hash)` → bool
- [ ] Add method `get_finalized_tip()` → Option<Hash>
- [ ] Integrate with chain selection to mark blocks final
- [ ] Emit events when blocks become final
- [ ] Add reorg protection (cannot reorg past finalized block)
- [ ] Write tests for finality scenarios

**Acceptance Criteria**:
- [ ] Blocks at depth ≥100 are marked final
- [ ] Finalized blocks cannot be reorganized
- [ ] `is_finalized(hash)` returns correct result
- [ ] Finality events emitted
- [ ] Tests cover: normal finalization, attempted reorg past final

**Files to Create**:
- `citrate/core/consensus/src/finality.rs`

**Files to Modify**:
- `citrate/core/consensus/src/lib.rs`
- `citrate/core/consensus/src/chain_selection.rs`
- `citrate/core/consensus/src/dag_store.rs`

**Dependencies**: WP-0.1, WP-0.2

**Design Notes**:
```rust
pub struct FinalityTracker {
    config: FinalityConfig,
    finalized_tip: RwLock<Option<Hash>>,
    finalized_height: AtomicU64,
}

impl FinalityTracker {
    pub fn update_finality(&self, current_tip: &Hash, current_height: u64) {
        if current_height > self.config.confirmation_depth {
            let final_height = current_height - self.config.confirmation_depth;
            // Find block at final_height in selected-parent chain
            // Mark it and all ancestors as final
        }
    }
}
```

---

### WP-0.6: Fix Executor Panic Points
**Points**: 5 | **Priority**: P1 | **Status**: [ ] Not Started

**Description**:
Multiple `unwrap()` calls in transaction parsing can cause node crashes on malformed input. Convert these to proper error handling.

**Tasks**:
- [ ] Find all unwrap() calls in executor.rs
- [ ] Line 501: `data[0..32].try_into().unwrap()` → `?` with error
- [ ] Line 502: `data[32..36].try_into().unwrap()` → `?` with error
- [ ] Line 558: Similar fix
- [ ] Line 585: Similar fix
- [ ] Line 600-601: Similar fixes
- [ ] Line 631: Similar fix
- [ ] Line 1572, 1601: Similar fixes
- [ ] Create `ExecutionError::InvalidInput` variant if not exists
- [ ] Add test with malformed transaction data

**Acceptance Criteria**:
- [ ] No panic on malformed transactions
- [ ] Proper error returned to caller
- [ ] Test verifies graceful handling
- [ ] Zero unwrap() calls on user-provided data

**Files to Modify**:
- `citrate/core/execution/src/executor.rs`
- `citrate/core/execution/src/types.rs` (if error variant needed)

**Dependencies**: WP-0.1

---

### WP-0.7: Implement ECRECOVER Precompile
**Points**: 5 | **Priority**: P1 | **Status**: [ ] Not Started

**Description**:
The ECRECOVER precompile (0x01) currently returns zeros instead of recovering the signer address. This breaks all signature verification in contracts.

**Tasks**:
- [ ] Add secp256k1 dependency if not present
- [ ] Implement actual ECDSA recovery in `ecrecover` function
- [ ] Parse input: hash (32 bytes), v (32 bytes), r (32 bytes), s (32 bytes)
- [ ] Convert v to recovery ID (27/28 → 0/1)
- [ ] Recover public key from signature
- [ ] Derive address from public key (keccak256 → last 20 bytes)
- [ ] Return zero-padded 32-byte address
- [ ] Add comprehensive tests

**Acceptance Criteria**:
- [ ] Recovers correct address from valid signature
- [ ] Returns zeros for invalid signature (per EVM spec)
- [ ] Gas cost: 3000 (standard)
- [ ] Test with known Ethereum test vectors

**Files to Modify**:
- `citrate/core/execution/src/precompiles/mod.rs`

**Dependencies**: WP-0.1

**Reference Implementation**:
```rust
fn ecrecover(&self, input: &[u8], gas_limit: u64) -> Result<PrecompileResult> {
    const GAS_COST: u64 = 3000;
    if gas_limit < GAS_COST {
        return Err(anyhow::anyhow!("Insufficient gas"));
    }

    // Parse input (128 bytes)
    if input.len() < 128 {
        return Ok(PrecompileResult { output: vec![0u8; 32], gas_used: GAS_COST });
    }

    let hash = &input[0..32];
    let v = input[63]; // Last byte of v field
    let r = &input[64..96];
    let s = &input[96..128];

    // Recovery ID: v should be 27 or 28
    let recovery_id = match v {
        27 => 0,
        28 => 1,
        _ => return Ok(PrecompileResult { output: vec![0u8; 32], gas_used: GAS_COST }),
    };

    // Recover public key and derive address
    // Use secp256k1 crate
    ...
}
```

---

### WP-0.8: Add Integration Tests
**Points**: 8 | **Priority**: P1 | **Status**: [ ] Not Started

**Description**:
Add comprehensive integration tests to verify all fixes work together.

**Tasks**:
- [ ] Create `citrate/core/consensus/tests/integration.rs`
- [ ] Test: Multi-node block propagation
- [ ] Test: Transaction gossip between nodes
- [ ] Test: Total ordering consistency across nodes
- [ ] Test: Finality progression
- [ ] Test: Reorg handling
- [ ] Test: Malformed transaction rejection
- [ ] Test: ECRECOVER in contract

**Acceptance Criteria**:
- [ ] All integration tests pass
- [ ] Tests run in CI
- [ ] Coverage of critical paths

**Files to Create**:
- `citrate/core/consensus/tests/integration.rs`
- `citrate/tests/e2e/` (end-to-end tests)

**Dependencies**: WP-0.1 through WP-0.7

---

## Sprint Backlog Summary

| WP | Title | Points | Priority | Status | Assignee |
|----|-------|--------|----------|--------|----------|
| WP-0.1 | Fix Consensus Test Compilation | 2 | P0 | [ ] | - |
| WP-0.2 | Implement Total Ordering | 13 | P0 | [ ] | - |
| WP-0.3 | Fix Block Propagation Peer IDs | 3 | P0 | [ ] | - |
| WP-0.4 | Fix Transaction Gossip Peer IDs | 2 | P0 | [ ] | - |
| WP-0.5 | Implement Basic Finality | 13 | P0 | [ ] | - |
| WP-0.6 | Fix Executor Panic Points | 5 | P1 | [ ] | - |
| WP-0.7 | Implement ECRECOVER Precompile | 5 | P1 | [ ] | - |
| WP-0.8 | Add Integration Tests | 8 | P1 | [ ] | - |

**Total Points**: 51
**P0 Points**: 33 (must complete)
**P1 Points**: 18 (should complete)

---

## Recommended Execution Order

```
Day 1-2:   WP-0.1 (compilation) → enables all other work
Day 2-3:   WP-0.3, WP-0.4 (network fixes) → quick wins
Day 3-6:   WP-0.2 (total ordering) → critical path
Day 6-8:   WP-0.5 (finality) → depends on ordering
Day 8-9:   WP-0.6 (panic fixes) → security
Day 9-10:  WP-0.7 (ECRECOVER) → contract compatibility
Day 10:    WP-0.8 (integration tests) → verification
```

---

## Definition of Done

- [ ] Code compiles without warnings (`cargo build --workspace`)
- [ ] All tests pass (`cargo test --workspace`)
- [ ] Clippy passes (`cargo clippy --all-targets --all-features`)
- [ ] Code formatted (`cargo fmt --all`)
- [ ] Changes documented in CLAUDE.md if architecture changed
- [ ] Manual testing on devnet completed
- [ ] No critical security issues introduced

---

## Risks & Blockers

| Risk | Impact | Probability | Mitigation |
|------|--------|-------------|------------|
| Total ordering more complex than estimated | High | Medium | Start early, ask for help if stuck >2 days |
| Network refactor needed for peer IDs | Medium | Low | Check peer API first before starting |
| Finality interacts with existing code unexpectedly | Medium | Medium | Keep implementation isolated, integrate carefully |
| secp256k1 dependency conflicts | Low | Low | Check Cargo.toml first |

---

## Success Metrics

After Sprint 0:
- [ ] `cargo test --workspace` passes (0 failures)
- [ ] Multi-node devnet runs for 1 hour without issues
- [ ] Blocks propagate between 3+ nodes
- [ ] Transactions execute in same order on all nodes
- [ ] Finality reached within expected depth
- [ ] Contract with ECRECOVER works

---

## Notes

This sprint is a **BLOCKER** for all subsequent work. The AI-first GUI cannot be built on a broken foundation. Prioritize P0 items ruthlessly.

If any P0 item is at risk of not completing, escalate immediately.

---

*Created: December 2024*
*Last Updated: December 2024*
