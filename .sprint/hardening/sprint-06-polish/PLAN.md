# Sprint 06: Final Polish & Validation

**Status**: Blocked (waiting for Sprint 05)
**Priority**: P1 High
**Duration**: 2-3 days
**Depends On**: Sprint 05

---

## Problem Statement

After completing all hardening sprints, we need to validate that everything works end-to-end, update documentation, and ensure the system is production-ready.

---

## Work Breakdown

### Task 1: Run Full Test Suite

Execute all automated tests:

```bash
# Core tests
cargo test --workspace

# Consensus tests
cargo test -p citrate-consensus

# Execution tests
cargo test -p citrate-execution

# API tests
cargo test -p citrate-api

# Contract tests
cd contracts && forge test

# SDK tests
cd sdk/javascript && npm test
cd sdks/python && pytest

# GUI tests
cd gui/citrate-core && npm test
```

**Acceptance Criteria**:
- [ ] All cargo tests pass
- [ ] All Forge tests pass
- [ ] All SDK tests pass
- [ ] All GUI tests pass

---

### Task 2: Execute TEST_PLAN.md Manual Tests

Run through all 87 test cases from TEST_PLAN.md:

| Category | Tests | Status |
|----------|-------|--------|
| Core Blockchain (CB-001 to CB-015) | 15 | |
| Wallet Operations (WA-001 to WA-012) | 12 | |
| Smart Contracts (SC-001 to SC-010) | 10 | |
| Agent & Tools (AG-001 to AG-014) | 14 | |
| AI/Model Operations (AI-001 to AI-008) | 8 | |
| SDK Integration (JS-001 to JS-005, PY-001 to PY-005) | 10 | |
| CLI Validation (CLI-001 to CLI-008) | 8 | |
| Security (SEC-001 to SEC-010) | 10 | |

**Acceptance Criteria**:
- [ ] All P0 tests pass
- [ ] All P1 tests pass
- [ ] P2 tests documented (pass/fail with notes)

---

### Task 3: E2E User Journey Validation

Execute the three user journeys from TEST_PLAN.md:

#### Journey 1: First-Time Setup
```
1. Install Citrate GUI
2. Complete onboarding
3. Wait for model download
4. Create first account
5. Request testnet tokens
6. Check balance
7. Send transaction
8. Verify in history
9. Use agent to check balance
10. Deploy contract via agent
```

#### Journey 2: Developer Workflow
```
1. Start node via GUI
2. Connect with SDK
3. Create account via SDK
4. Fund from faucet
5. Write Solidity contract
6. Compile with Forge
7. Deploy via SDK
8. Call contract functions
9. Verify events
10. Query via agent
```

#### Journey 3: AI Agent Workflow
```
1. Open ChatDashboard
2. Check balance via chat
3. Show DAG status
4. Deploy contract
5. Approve deployment
6. Call contract
7. Transfer tokens
8. Approve transfer
9. Show history
10. Verify operations
```

**Acceptance Criteria**:
- [ ] All three journeys complete successfully
- [ ] No errors or unexpected behavior

---

### Task 4: Update Documentation

#### 4.1 Update CLAUDE.md
- [ ] Add new Tauri commands (session management)
- [ ] Update RPC endpoint list
- [ ] Add hardening sprint reference

#### 4.2 Update README.md
- [ ] Update feature list
- [ ] Add session management section
- [ ] Update CLI documentation

#### 4.3 Update FEATURE_INVENTORY.md
- [ ] Mark all completed items as "Working"
- [ ] Remove "NOT IMPLEMENTED" notes where fixed
- [ ] Update status percentages

#### 4.4 Update TEST_PLAN.md
- [ ] Fill in all test results
- [ ] Add any new test cases discovered
- [ ] Document known issues

**Acceptance Criteria**:
- [ ] All docs reflect current state
- [ ] No stale information

---

### Task 5: Performance Validation

Run performance benchmarks:

```bash
# Transaction throughput
./scripts/benchmark_throughput.sh

# Block production
./scripts/benchmark_blocks.sh

# Precompile performance
cargo bench -p citrate-execution
```

| Metric | Target | Actual | Status |
|--------|--------|--------|--------|
| TPS | > 1000 | | |
| Block Time | < 2s | | |
| Finality | < 15s | | |
| ECRECOVER | < 1ms | | |
| MODEXP (256-bit) | < 10ms | | |

**Acceptance Criteria**:
- [ ] All metrics meet targets
- [ ] No performance regressions

---

### Task 6: Create Release Notes

Compile release notes for hardening initiative:

```markdown
# Citrate Hardening Release

## New Features

### Session Management
- Session status indicator in wallet
- Smart password prompting
- Manual wallet locking
- High-value transaction protection

### Tracked Addresses
- Backend persistence
- Balance monitoring for external addresses
- Activity history

### RPC Endpoints
- citrate_deployModel
- citrate_runInference
- citrate_getDagStats
- citrate_pinArtifact
- citrate_verifyContract

### CLI Improvements
- ed25519 signature alignment
- Full command implementation
- Cross-compatible with wallet

### Precompiles
- Complete RIPEMD160
- Complete MODEXP
- Complete BN256 curve operations
- Complete BLAKE2F

## Breaking Changes
- CLI signature scheme changed (re-import accounts)

## Bug Fixes
- Session management exposed to frontend
- Tracked addresses now persist
- SDK model methods now work

## Performance
- TPS: X (was Y)
- Block time: X (was Y)
```

**Acceptance Criteria**:
- [ ] Release notes complete
- [ ] All changes documented

---

### Task 7: Final Git Tags and Commits

```bash
# Ensure all sprint tags exist
git tag -l "hardening-*"

# Create final release tag
git tag -a v0.2.0-hardening -m "Hardening Initiative Complete"

# Push tags
git push origin --tags
```

**Acceptance Criteria**:
- [ ] All sprint tags created
- [ ] Release tag created
- [ ] All pushed to origin

---

## Testing Checklist

### Automated Test Results

| Suite | Passed | Failed | Total |
|-------|--------|--------|-------|
| cargo test | | | |
| forge test | | | |
| npm test (SDK) | | | |
| npm test (GUI) | | | |
| pytest | | | |

### Manual Test Results

| Category | Passed | Failed | Blocked |
|----------|--------|--------|---------|
| Core Blockchain | | | |
| Wallet Operations | | | |
| Smart Contracts | | | |
| Agent & Tools | | | |
| AI/Model | | | |
| SDK | | | |
| CLI | | | |
| Security | | | |

---

## Files Modified

| File | Changes |
|------|---------|
| `CLAUDE.md` | Update commands and RPCs |
| `README.md` | Update features |
| `.sprint/FEATURE_INVENTORY.md` | Update status |
| `.sprint/TEST_PLAN.md` | Fill results |
| `CHANGELOG.md` | Add release notes |

---

## Definition of Done

- [ ] All automated tests pass
- [ ] All manual tests documented
- [ ] All user journeys complete
- [ ] Documentation updated
- [ ] Performance validated
- [ ] Release notes created
- [ ] Git tags created
- [ ] Git commit: "Hardening Initiative Complete"
