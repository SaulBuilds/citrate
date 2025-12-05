# Audit Fixes Sprint Plan

## Overview
This plan addresses all issues identified in the security/completeness audit documented in `.audit/deep_dive.md`. The goal is to achieve testnet readiness by resolving all mocks, stubs, security issues, and missing integrations.

**Timeline**: 6 weeks (5 sprints of ~1 week each, plus buffer)
**Created**: 2025-12-04
**Source**: `.audit/deep_dive.md`

---

## Sprint Summary

| Sprint | Focus Area | Duration | Key Deliverables |
|--------|------------|----------|------------------|
| 1 | Core Infrastructure | Week 1 | Block roots, chain ID, sync, DAG tracking, AI handler |
| 2 | GUI Security & Mocks | Week 2 | Remove signature mocks, fix RPC, eth_call, agent tools |
| 3 | Contract & IPFS | Week 3 | Real compiler (solc-js + Foundry), IPFS pinning, deployment |
| 4 | SDK Integration | Week 4-5 | Integration tests, address derivation, EIP-1559/2930 |
| 5 | Observability & Polish | Week 6 | Metrics, logging, error handling, docs cleanup |

---

## Issue Tracking Matrix

### Critical (Blocking Testnet)
| Issue | Location | Sprint | Status |
|-------|----------|--------|--------|
| Block builder fake roots | `core/sequencer/src/block_builder.rs:306-320` | 1 | FIXED |
| Mock tx creation in decoder | `core/api/src/eth_tx_decoder.rs:665-710` | 1 | FIXED |
| RPC mempool type mismatch | `gui/citrate-core/src-tauri/src/node/mod.rs` | 2 | [ ] |
| eth_call returns error | `src-tauri/src/lib.rs` | 1 | FIXED |
| Mock signatures in tauri.ts | `gui/citrate-core/src/services/tauri.ts` | 2 | [ ] |
| Chain ID hardcoded | `core/execution/src/executor.rs:800` | 1 | [ ] |

### High Priority (Functionality)
| Issue | Location | Sprint | Status |
|-------|----------|--------|--------|
| Efficient sync stubbed | `core/consensus/src/sync/efficient_sync.rs` | 1 | [ ] |
| Genesis DAG tracking TODO | `node/src/genesis.rs:237` | 1 | [ ] |
| AI handler stubs | `core/network/src/ai_handler.rs:331,415,480` | 1 | [ ] |
| Contract compiler placeholder | `gui/citrate-core/src/utils/contractCompiler.ts` | 3 | [ ] |
| IPFS mock uploads | `src-tauri/src/agent/tools/storage.rs:148-174` | 3 | [ ] |
| Image gen mock fallback | `src-tauri/src/agent/tools/generation.rs:145-171` | 2 | [ ] |

### Medium Priority (Quality)
| Issue | Location | Sprint | Status |
|-------|----------|--------|--------|
| eth_feeHistory mock | `core/api/src/eth_rpc.rs:923-930` | 1 | [ ] |
| DAG explorer stubs | `core/api/src/dag_explorer.rs:486-493,697-708` | 1 | [ ] |
| Marketplace mock data | `components/Marketplace.tsx` | 3 | [ ] |
| Search index stubs | `utils/search/searchIndexBuilder.ts:252` | 3 | [ ] |
| Review voting TODO | `hooks/useReviews.ts:112-180` | 5 | [ ] |

### Low Priority (Polish)
| Issue | Location | Sprint | Status |
|-------|----------|--------|--------|
| SDK dist validation | `sdk/javascript/`, `sdks/` | 4 | [ ] |
| Genesis mock embedding | `core/genesis/genesis_model.rs:114-129` | 1 | [ ] |
| Model verifier validators | `node/src/model_verifier.rs:75` | 1 | [ ] |
| Speech recognition TODOs | `ChatBot.tsx:241,244` | 5 | [ ] |

---

## Testnet Gate Checklist

### Core (Sprint 1)
- [ ] Real block roots (state_root, receipt_root) based on transaction execution
- [ ] Configurable chain ID (env var or config file)
- [ ] Efficient sync implementation with real block catch-up
- [ ] DAG tracking in genesis initialization
- [ ] AI handler message wiring (training deltas, inference routing)
- [ ] eth_feeHistory returns real fee data
- [ ] Property/fuzz tests for GhostDAG ordering

### RPC/GUI (Sprint 2)
- [ ] Mempool type mismatch resolved, RPC server starts
- [ ] eth_call executes against real state
- [ ] Signature failures hard-fail (no mock fallback)
- [ ] IPFS/AI tools error when offline (no silent mock)
- [ ] Receipt polling implemented
- [ ] Gas estimation working

### Agent/Codegen (Sprint 3)
- [ ] Real Solidity compiler (solc-js WASM + Foundry CLI option)
- [ ] Real IPFS pinning with connectivity check
- [ ] Deterministic LLM selection with audit logs
- [ ] MCP connectivity tested and validated
- [ ] Marketplace uses real contract data

### SDKs (Sprint 4)
- [ ] Integration tests: eth_call, sendRawTx, feeHistory
- [ ] Dual address derivation tested (EVM + native)
- [ ] EIP-1559 and EIP-2930 transaction types tested
- [ ] Chain ID from config matches network
- [ ] Both embedded node and testnet endpoint tests

### Observability (Sprint 5)
- [ ] Metrics for: node lifecycle, RPC errors, block production, AI jobs, IPFS ops
- [ ] Structured logging with trace IDs
- [ ] User-facing errors surfaced in GUI
- [ ] Docs updated to match implementation
- [ ] Dev-only code gated or removed

---

## Dependencies & Order

```
Sprint 1 (Core) ─┬─> Sprint 2 (GUI Security)
                 │
                 └─> Sprint 3 (Contract/IPFS) ──> Sprint 4 (SDK) ──> Sprint 5 (Polish)
```

Sprint 1 must complete first as it provides the foundation for RPC and SDK work.
Sprints 2 and 3 can run in parallel after Sprint 1.
Sprint 4 depends on Sprints 1-3 completion.
Sprint 5 is cleanup and can overlap with Sprint 4 tail end.

---

## Risk Register

| Risk | Impact | Probability | Mitigation |
|------|--------|-------------|------------|
| solc-js WASM size bloats bundle | Med | Med | Lazy-load compiler, offer Foundry-only mode |
| IPFS connectivity unreliable | High | Med | Graceful degradation with clear user feedback |
| SDK breaking changes | High | Low | Version lock, deprecation warnings |
| Parallel sprint conflicts | Med | Med | Clear code ownership boundaries |

---

## Success Criteria

The audit fix sprints are complete when:
1. All critical and high priority items are resolved
2. No mock/stub code paths are exercised in production builds
3. All testnet gate checklist items pass
4. cargo test --workspace passes with no failures
5. GUI builds and runs without mock fallbacks
6. SDKs have >80% integration test coverage
7. Metrics/observability dashboards show real data

---

*Last Updated: 2025-12-04*
