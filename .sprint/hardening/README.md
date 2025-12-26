# Citrate Hardening Initiative

**Start Date**: 2025-12-25
**Target Completion**: 6 Sprints (3 weeks estimated)
**Reference**: [FEATURE_INVENTORY.md](../FEATURE_INVENTORY.md) | [TEST_PLAN.md](../TEST_PLAN.md)

---

## Executive Summary

This hardening initiative addresses all critical gaps identified in the comprehensive project audit. Each sprint focuses on a specific area and must be completed, validated, and tested before proceeding to the next.

## Sprint Overview

| Sprint | Focus Area | Priority | Duration | Status |
|--------|------------|----------|----------|--------|
| [Sprint 01](./sprint-01-session-security/) | Session Security & Frontend Exposure | P0 Critical | 2-3 days | **CURRENT** |
| [Sprint 02](./sprint-02-tracked-addresses/) | Tracked Addresses Backend | P0 Critical | 1-2 days | Blocked |
| [Sprint 03](./sprint-03-missing-rpcs/) | Missing RPC Endpoints | P1 High | 2-3 days | Blocked |
| [Sprint 04](./sprint-04-cli-alignment/) | CLI Signature Alignment | P2 Medium | 1-2 days | Blocked |
| [Sprint 05](./sprint-05-precompiles/) | Precompile Completion | P2 Medium | 2-3 days | Blocked |
| [Sprint 06](./sprint-06-polish/) | Final Polish & Validation | P1 High | 2-3 days | Blocked |

## Critical Rules

### 1. Sequential Execution
- **DO NOT** start a new sprint until the current sprint is complete
- Each sprint has explicit acceptance criteria that MUST pass
- No partial completions - all tasks in a sprint must be done

### 2. Validation Requirements
- Every sprint ends with automated tests passing
- Manual validation using the test plan
- Documentation updated before sprint closes

### 3. Sprint Structure
Each sprint folder contains:
```
sprint-XX-name/
├── PLAN.md          # Detailed implementation plan
├── PROGRESS.md      # Daily progress tracking
├── VALIDATION.md    # Test results and sign-off
└── CHANGES.md       # Summary of all changes made
```

### 4. Completion Checklist
Before marking a sprint complete:
- [ ] All tasks in PLAN.md are done
- [ ] All tests in VALIDATION.md pass
- [ ] CHANGES.md documents all modifications
- [ ] PROGRESS.md has final status
- [ ] Git commit with sprint tag created

---

## Issue Prioritization

### P0 Critical (Must Fix)

| Issue | Sprint | Impact |
|-------|--------|--------|
| Session management not exposed | 01 | Password required every transaction |
| No session UI in frontend | 01 | Users can't see/control session |
| Tracked addresses broken | 02 | Feature completely non-functional |

### P1 High (Should Fix)

| Issue | Sprint | Impact |
|-------|--------|--------|
| Missing model RPC endpoints | 03 | SDK methods fail |
| Missing artifact RPC endpoints | 03 | IPFS integration broken |
| Missing DAG statistics RPC | 03 | SDK incomplete |

### P2 Medium (Nice to Fix)

| Issue | Sprint | Impact |
|-------|--------|--------|
| CLI uses secp256k1 vs ed25519 | 04 | CLI/wallet incompatible |
| CLI stubs not implemented | 04 | CLI partially broken |
| RIPEMD160 stub | 05 | Some contracts fail |
| MODEXP stub | 05 | ZK circuits fail |
| EC-pairings stubs | 05 | BLS signatures fail |

---

## Success Metrics

### Sprint 01 Success
- User can see session countdown in UI
- User can manually lock wallet
- Transactions send without password if session active
- High-value transactions still require password

### Sprint 02 Success
- Tracked addresses persist across sessions
- Balance updates for tracked addresses work
- Activity history for tracked addresses works

### Sprint 03 Success
- JavaScript SDK model methods work
- `citrate_deployModel` RPC responds
- `citrate_getDagStats` RPC responds

### Sprint 04 Success
- CLI and wallet use same signature scheme
- Accounts created in CLI work in wallet
- All CLI commands functional

### Sprint 05 Success
- RIPEMD160 returns correct hashes
- MODEXP calculates correctly
- EC-pairings verify BLS signatures

### Sprint 06 Success
- All 87 test cases pass
- E2E user journeys complete
- Documentation updated

---

## Quick Start

### Begin Sprint 01
```bash
# Read the sprint plan
cat .sprint/hardening/sprint-01-session-security/PLAN.md

# Track progress
# Update PROGRESS.md as you work

# Validate completion
cat .sprint/hardening/sprint-01-session-security/VALIDATION.md
```

### Check Current Sprint
```bash
# Find current sprint
grep -l "CURRENT" .sprint/hardening/*/PLAN.md
```

### Mark Sprint Complete
```bash
# Update status in README.md and sprint PLAN.md
# Create git commit with sprint tag
git add .
git commit -m "Complete Sprint 01: Session Security"
git tag hardening-sprint-01
```

---

## File References

### Key Files to Modify

| Sprint | Primary Files |
|--------|---------------|
| 01 | `gui/citrate-core/src-tauri/src/lib.rs`, `gui/citrate-core/src/components/` |
| 02 | `core/api/src/eth_rpc.rs`, `gui/citrate-core/src/components/Wallet.tsx` |
| 03 | `core/api/src/server.rs`, `sdk/javascript/src/` |
| 04 | `cli/src/`, `wallet/src/` |
| 05 | `core/execution/src/precompiles/mod.rs` |
| 06 | All documentation, tests |

### Test Commands

```bash
# Core tests
cargo test -p citrate-consensus
cargo test -p citrate-execution
cargo test -p citrate-api

# GUI tests
cd gui/citrate-core && npm test

# Contract tests
cd contracts && forge test

# SDK tests
cd sdk/javascript && npm test
```

---

## Governance

### Decision Authority
- Implementation decisions: Developer discretion
- Architecture changes: Require documentation update
- API changes: Require SDK update

### Escalation Path
1. Check existing documentation
2. Review audit findings
3. Consult CLAUDE.md guidelines
4. Document decision in sprint CHANGES.md

---

**Current Sprint**: [Sprint 01 - Session Security](./sprint-01-session-security/PLAN.md)
