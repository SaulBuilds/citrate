# Sprint 5: Observability & Polish

## Sprint Info
- **Duration**: Week 6
- **Sprint Goal**: Add production observability, cleanup dev-only code, update docs
- **Phase**: Audit Fixes - Final Polish
- **Depends On**: Sprints 1-4 (all prior work)

---

## Sprint Objectives

1. Add metrics for node lifecycle, RPC, block production, AI, IPFS
2. Implement structured logging with trace IDs
3. Surface user-facing errors in GUI
4. Gate or remove dev-only mock code
5. Update documentation to match implementation

---

## Work Breakdown Structure (WBS)

### WP-5.1: Node Lifecycle Metrics
**Points**: 5 | **Priority**: P1 | **Status**: [ ] Not Started

**Description**:
Add Prometheus-style metrics for node operations.

**Tasks**:
- [ ] Add metrics crate (prometheus or metrics-rs)
- [ ] Track node start/stop events
- [ ] Track peer count and connections
- [ ] Track mempool size and transactions
- [ ] Track block height and sync status
- [ ] Track DAG tips count
- [ ] Expose metrics endpoint (/metrics)
- [ ] Add Grafana dashboard template

**Acceptance Criteria**:
- [ ] Metrics endpoint returns Prometheus format
- [ ] All key node metrics tracked
- [ ] Dashboard shows node health

**Files to Modify**:
- `node/src/metrics.rs` (new)
- `node/src/main.rs`
- `grafana/dashboards/` (new)

**Dependencies**: None

---

### WP-5.2: RPC Error Metrics
**Points**: 3 | **Priority**: P1 | **Status**: [ ] Not Started

**Description**:
Track RPC errors, latency, and throughput.

**Tasks**:
- [ ] Track RPC request count by method
- [ ] Track RPC error count by method and error type
- [ ] Track RPC latency histogram
- [ ] Track transaction submission success/failure
- [ ] Add rate limiting metrics

**Acceptance Criteria**:
- [ ] RPC performance visible in metrics
- [ ] Error rates tracked by type
- [ ] Latency distribution available

**Files to Modify**:
- `core/api/src/metrics.rs` (new)
- `core/api/src/eth_rpc.rs`

**Dependencies**: WP-5.1

---

### WP-5.3: Block Production Metrics
**Points**: 3 | **Priority**: P1 | **Status**: [ ] Not Started

**Description**:
Track block production performance.

**Tasks**:
- [ ] Track blocks produced per minute
- [ ] Track block build time
- [ ] Track transactions per block
- [ ] Track orphan/reorg rate
- [ ] Track blue score progression

**Acceptance Criteria**:
- [ ] Block production rate visible
- [ ] Build time distribution tracked
- [ ] DAG health indicators available

**Files to Modify**:
- `core/sequencer/src/metrics.rs` (new)
- `core/sequencer/src/block_builder.rs`
- `gui/citrate-core/src-tauri/src/block_producer.rs`

**Dependencies**: WP-5.1

---

### WP-5.4: AI Job Metrics
**Points**: 3 | **Priority**: P1 | **Status**: [ ] Not Started

**Description**:
Track AI inference and training job metrics.

**Tasks**:
- [ ] Track inference request count
- [ ] Track inference latency by model
- [ ] Track token throughput (tokens/sec)
- [ ] Track model load time
- [ ] Track training job progress
- [ ] Track GPU/Metal utilization if available

**Acceptance Criteria**:
- [ ] AI performance visible in metrics
- [ ] Per-model breakdown available
- [ ] Resource utilization tracked

**Files to Modify**:
- `core/mcp/src/metrics.rs` (new)
- `gui/citrate-core/src-tauri/src/models/mod.rs`

**Dependencies**: WP-5.1

---

### WP-5.5: IPFS Operation Metrics
**Points**: 2 | **Priority**: P2 | **Status**: [ ] Not Started

**Description**:
Track IPFS operations and connectivity.

**Tasks**:
- [ ] Track IPFS connection status
- [ ] Track upload count and size
- [ ] Track pin/unpin operations
- [ ] Track gateway vs local node usage
- [ ] Track retrieval latency

**Acceptance Criteria**:
- [ ] IPFS health visible
- [ ] Upload/download stats tracked

**Files to Modify**:
- `gui/citrate-core/src-tauri/src/ipfs/mod.rs`

**Dependencies**: WP-5.1

---

### WP-5.6: Structured Logging
**Points**: 5 | **Priority**: P0 | **Status**: [ ] Not Started

**Description**:
Implement structured logging with trace IDs for request correlation.

**Tasks**:
- [ ] Add tracing-subscriber with JSON output
- [ ] Generate trace ID for each RPC request
- [ ] Propagate trace ID through call stack
- [ ] Add trace ID to log fields
- [ ] Configure log levels per module
- [ ] Add log rotation for file output
- [ ] Document log format

**Acceptance Criteria**:
- [ ] Logs are JSON formatted
- [ ] Trace ID correlates related logs
- [ ] Log levels configurable at runtime

**Files to Modify**:
- `node/src/logging.rs` (new)
- `core/api/src/middleware.rs` (trace ID)
- All crates (update log calls)

**Dependencies**: None

---

### WP-5.7: GUI Error Surfaces
**Points**: 5 | **Priority**: P0 | **Status**: [ ] Not Started

**Description**:
Surface backend errors to users with clear, actionable messages.

**Tasks**:
- [ ] Create error notification component
- [ ] Map backend errors to user-friendly messages
- [ ] Add toast notifications for transient errors
- [ ] Add modal for blocking errors
- [ ] Include error codes for support
- [ ] Add "Report Issue" link with context
- [ ] Log frontend errors to backend

**Acceptance Criteria**:
- [ ] Users see clear error messages
- [ ] Errors include actionable next steps
- [ ] Error context available for debugging

**Files to Modify**:
- `gui/citrate-core/src/components/ErrorNotification.tsx` (new)
- `gui/citrate-core/src/contexts/ErrorContext.tsx` (new)
- `gui/citrate-core/src/App.tsx`

**Dependencies**: None

---

### WP-5.8: Dev-Only Code Gating
**Points**: 3 | **Priority**: P0 | **Status**: [ ] Not Started

**Description**:
Gate or remove all dev-only mock code from production builds.

**Tasks**:
- [ ] Audit all mock/demo code paths
- [ ] Add build-time flag for dev mode
- [ ] Gate mock code behind dev flag (or remove)
- [ ] Ensure prod build has no mocks
- [ ] Add build verification test
- [ ] Document dev mode usage

**Acceptance Criteria**:
- [ ] Production build contains no mock paths
- [ ] Dev mode clearly labeled in UI
- [ ] Build test verifies no mocks in prod

**Files to Modify**:
- All files with mock code (from audit list)
- `gui/citrate-core/vite.config.ts`
- `gui/citrate-core/src-tauri/build.rs`

**Dependencies**: Sprints 2-3 (mocks already removed)

---

### WP-5.9: Review Voting Implementation
**Points**: 3 | **Priority**: P2 | **Status**: [ ] Not Started

**Description**:
Review voting/reporting marked as unimplemented (useReviews.ts:112-180).

**Tasks**:
- [ ] Implement vote submission to contract
- [ ] Implement vote tallying
- [ ] Implement report submission
- [ ] Add vote UI components
- [ ] Add report modal

**Acceptance Criteria**:
- [ ] Users can vote on reviews
- [ ] Votes stored on-chain
- [ ] Reports submitted to contract

**Files to Modify**:
- `gui/citrate-core/src/hooks/useReviews.ts:112-180`
- `gui/citrate-core/src/components/ReviewCard.tsx`

**Dependencies**: Sprint 3 (marketplace contracts)

---

### WP-5.10: Documentation Update
**Points**: 5 | **Priority**: P1 | **Status**: [ ] Not Started

**Description**:
Update all documentation to match current implementation.

**Tasks**:
- [ ] Update CLAUDE.md with audit fixes
- [ ] Update marketplace docs (remove "no mocks" claim or verify)
- [ ] Update SDK documentation
- [ ] Add chain ID configuration docs
- [ ] Add metrics/observability docs
- [ ] Add error code reference
- [ ] Update API reference
- [ ] Verify all examples work

**Acceptance Criteria**:
- [ ] Docs match implementation
- [ ] All examples tested and working
- [ ] No misleading claims

**Files to Modify**:
- `CLAUDE.md`
- `gui/citrate-core/docs/marketplace/README.md`
- `sdk/javascript/README.md`
- `docs-portal/docs/`

**Dependencies**: All other WPs complete

---

## Sprint Backlog Summary

| WP | Title | Points | Priority | Status |
|----|-------|--------|----------|--------|
| WP-5.1 | Node Lifecycle Metrics | 5 | P1 | [ ] |
| WP-5.2 | RPC Error Metrics | 3 | P1 | [ ] |
| WP-5.3 | Block Production Metrics | 3 | P1 | [ ] |
| WP-5.4 | AI Job Metrics | 3 | P1 | [ ] |
| WP-5.5 | IPFS Operation Metrics | 2 | P2 | [ ] |
| WP-5.6 | Structured Logging | 5 | P0 | [ ] |
| WP-5.7 | GUI Error Surfaces | 5 | P0 | [ ] |
| WP-5.8 | Dev-Only Code Gating | 3 | P0 | [ ] |
| WP-5.9 | Review Voting Implementation | 3 | P2 | [ ] |
| WP-5.10 | Documentation Update | 5 | P1 | [ ] |

**Total Points**: 37
**Committed Points**: 32 (excluding P2)
**Buffer**: 5 points

---

## Definition of Done

- [ ] Metrics endpoint returns data
- [ ] Logs are structured JSON
- [ ] GUI shows user-friendly errors
- [ ] No mock code in production build
- [ ] Documentation accurate and complete
- [ ] All examples tested

---

## Risks & Blockers

| Risk | Impact | Mitigation |
|------|--------|------------|
| Metrics overhead on performance | Med | Benchmark before/after |
| Documentation drift | Low | Generate from code where possible |
| Review voting contract not ready | Low | Defer to post-launch |

---

## Notes

- WP-5.6 (Structured Logging) and WP-5.7 (GUI Errors) are highest priority
- Metrics can be added incrementally
- Documentation should be done last to capture all changes
- This is the final polish before testnet launch

---

## Launch Readiness Checklist

After Sprint 5 completion, verify:

- [ ] `cargo test --workspace` passes
- [ ] `cargo build --release` succeeds
- [ ] GUI builds without errors
- [ ] No mock code in release build
- [ ] Metrics endpoint returns data
- [ ] Logs are structured and searchable
- [ ] All SDK tests pass
- [ ] Documentation reviewed and accurate
- [ ] Security review completed
- [ ] Performance baseline established

---

*Created: 2025-12-04*
*Last Updated: 2025-12-04*
