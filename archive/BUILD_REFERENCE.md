# Citrate v3 Build Reference

## CRITICAL: Always Reference These Documents

Before implementing ANY feature, ALWAYS check:
1. `/CLAUDE.md` - Core guidance and architecture overview
2. `/IMPLEMENTATION_STRATEGY.md` - Sprint plan and implementation details  
3. `/lattice-docs-v3/` - Architecture and specifications

## Current Phase: Sprint 1 - Core GhostDAG

### Key Implementation Rules

1. **Consensus**: We use GhostDAG, NOT GHAST or GHOST
   - Selected parent + merge parents structure
   - Blue set/Blue score calculations
   - K-cluster rule for consistency

2. **Block Structure** (from CLAUDE.md:44-61):
   ```rust
   struct Block {
       selected_parent_hash: Hash,
       merge_parent_hashes: Vec<Hash>,
       blue_score: u64,
       // ... see CLAUDE.md for full structure
   }
   ```

3. **Module Organization**:
   - `core/consensus/` - GhostDAG engine ONLY
   - `core/sequencer/` - Mempool and parent selection
   - `core/execution/` - LVM (EVM-compatible)
   - `core/primitives/` - ModelRegistry, LoRAFactory, etc.
   - `core/storage/` - State DB and block store
   - `core/api/` - JSON-RPC and MCP REST

## Sprint Checkpoints

### Sprint 1 (Current)
- [ ] Block structure with selected/merge parents
- [ ] Blue set calculation algorithm
- [ ] Blue score computation
- [ ] DAG storage implementation
- [ ] Basic unit tests

### Sprint 2
- [ ] Tip selection algorithm
- [ ] VRF proposer selection
- [ ] Parent selection logic
- [ ] Integration tests

## Testing Requirements

Every implementation MUST have:
1. Unit tests for core algorithms
2. Property-based tests for invariants
3. Integration tests for module interactions
4. Performance benchmarks

## Common Mistakes to Avoid

1. ❌ Don't use GHAST consensus
2. ❌ Don't use single parent blocks
3. ❌ Don't forget blue/red distinction
4. ❌ Don't skip test coverage
5. ❌ Don't drift from specifications

## Commands for Verification

```bash
# Check implementation matches spec
grep -r "selected_parent" core/consensus/
grep -r "merge_parent" core/consensus/
grep -r "blue_score" core/consensus/

# Run tests
cargo test --all

# Check for spec drift
diff -u CLAUDE.md core/consensus/README.md
```

## Next Action Items

1. Implement `core/consensus/src/types.rs` with Block structure
2. Implement `core/consensus/src/ghostdag.rs` with blue set logic
3. Implement `core/consensus/src/dag_store.rs` for storage
4. Write comprehensive tests

REMEMBER: This is GhostDAG, not GHAST!