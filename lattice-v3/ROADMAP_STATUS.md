# Lattice V3 – Phase 3 & 4 Sprint Roadmap

**Scope Window:** Weeks 13–24  
**Revision:** March 2025  
**Source:** Updated after the Phase 3 readiness audit (`COMPREHENSIVE_AUDIT_AND_ROADMAP.md`)

---

## Strategic Context

- Phases 1–2 delivered a stable GhostDAG core, storage layer, and AI-facing APIs, but the inference stack and incentive wiring remain synthetic.
- Phase 3 must convert the existing scaffolding into a provable, distributed compute marketplace while keeping the chain stable.
- Phase 4 will harden, secure, and launch the production environment.

The roadmap below slices the remaining work into six two-week sprints with explicit deliverables and exit criteria.

---

## Phase 3 – Distributed Compute (Weeks 13–20)

### Sprint 13 (Weeks 13–14): **Runtime Convergence**
- **Goals**
  - Replace placeholder inference/training paths with real execution in MCP (`core/mcp/src/execution.rs`) and the executor (`core/execution/src/executor.rs`).
  - Fix Metal runtime/precompile parity (`core/execution/src/inference/metal_runtime.rs`, `core/execution/src/precompiles/inference.rs`).
  - Pipe model deployment and inference through mempool/consensus instead of direct executor calls (`core/api/src/server.rs`).
- **Deliverables**
  - Passing integration tests that invoke `ModelExecutor::execute_inference` end-to-end with deterministic outputs.
  - Updated RPC/CLI flows that submit transactions and observe receipts.
  - Regression tests covering the fixed struct mismatches.
- **Exit Criteria**
  - No `todo`/placeholder paths in inference pipeline.
  - CLI `model deploy` + `model inference` completes using the new RPC path on a devnet.

### Sprint 14 (Weeks 15–16): **Provider & Scheduler Wiring**
- **Goals**
  - Implement provider capability registration, attestation, and staking in MCP (`core/mcp/src/provider.rs`, `core/mcp/src/registry.rs`).
  - Integrate VRF leader election into block production (`core/consensus/src/vrf.rs`, `node/src/producer.rs`) to select proposers/providers.
  - Persist pinning metadata and add retry/backoff in `IPFSService` (`core/storage/src/ipfs/mod.rs`).
- **Deliverables**
  - Provider lifecycle tests (register → bid → complete job).
  - VRF-backed proposer selection exercised in multinode testnet.
  - Metrics emitted for provider participation and IPFS pinning health.
- **Exit Criteria**
  - Testnet run showing provider rotation and VRF selection without manual intervention.
  - Documented provider onboarding flow (CLI + API).

### Sprint 15 (Weeks 17–18): **Proofs, Payments, and Governance**
- **Goals**
  - Generate and verify inference/training proofs, storing artifacts via the artifact service.
  - Enforce payment flows (fee splits, provider payouts, treasury accrual) inside the executor (`core/execution/src/executor.rs:1783-1845`).
  - Surface governance hooks for configurable fees/slashing and expose them via RPC.
- **Deliverables**
  - Proof verifier integrated with precompiles (`core/execution/src/precompiles/inference.rs`) and Solidity contracts.
  - Automated tests covering fee distribution and failure cases.
  - Governance parameter documentation + CLI commands.
- **Exit Criteria**
  - Successful proof submission and verification recorded on-chain.
  - Provider misbehavior path triggers slash/forfeit in simulated scenario.

### Sprint 16 (Weeks 19–20): **Stability & Performance**
- **Goals**
  - Execute scaled load tests (≥10 GPU nodes, sustained inference jobs) using retained scripts (`scripts/load_test.sh`, `scripts/monitor_testnet.sh`).
  - Harden observability: dashboards, alerts, log hygiene.
  - Security review of new surfaces (RPC changes, provider registry, proofs).
- **Deliverables**
  - Load-test report with TPS, latency, and resource graphs.
  - Prometheus/Grafana artifacts checked into `monitoring/`.
  - Security findings backlog with mitigations scheduled.
- **Exit Criteria**
  - Stable 24-hour testnet run with successful job completions and no panics.
  - All P0/P1 security issues closed or waived with sign-off.

---

## Phase 4 – Production Launch (Weeks 21–24)

### Sprint 17 (Weeks 21–22): **Production Readiness**
- **Goals**
  - Harden deployment tooling (Docker images, helm manifests) and document operator runbooks.
  - Run full-chain upgrade rehearsal including provider onboarding, governance parameter changes, and rollback drills.
  - Finalize API/SDK changes and update docs/portal content.
- **Deliverables**
  - Release candidate build + reproducible build artefacts.
  - Operator guide covering provisioning, monitoring, and upgrades.
  - Updated SDK examples exercising new inference marketplace flows.
- **Exit Criteria**
  - Dry-run upgrade completed on staging without human intervention.
  - Documentation sign-off from DevRel/support.

### Sprint 18 (Weeks 23–24): **Launch & Post-Launch Support**
- **Goals**
  - Launch controlled mainnet/testnet with selected providers, enable incentives.
  - Establish incident response playbooks, on-call rotation, and observability dashboards.
  - Capture post-launch metrics and user feedback for backlog grooming.
- **Deliverables**
  - Public launch announcement, explorer overlays, and marketing collateral.
  - Post-launch health report (uptime, job volume, economic flows).
  - Prioritized backlog for Phase 5 (scaling/expansion).
- **Exit Criteria**
  - Mainnet uptime ≥99% during launch window.
  - First production inference jobs settled with verified proofs and payouts.

---

## Cross-Cutting Streams

- **Quality & Security:** add unit/integration tests alongside features; schedule third-party security review during Sprint 17; ensure CI covers GPU-enabled and CPU-fallback paths.
- **Documentation:** update developer docs, CLI help, and portal content sprint-by-sprint; keep `docs/` scoped to production materials (historical docs remain in `archive/`).
- **Stakeholder Demos:** host biweekly demos at the end of Sprints 13, 15, 17, and 18 to validate progress with ecosystem partners.

---

## Risks & Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| Runtime/precompile mismatch blocks inference | High | Address in Sprint 13; add compile-time assertions to keep structs aligned. |
| Provider economics untested at scale | High | Simulate varied provider sets during Sprint 16 load tests; add telemetry for payouts. |
| Proof generation latency on commodity GPUs | Medium | Provide CPU fallback and asynchronous proof submission queue; benchmark with representative hardware. |
| RPC migration breaks existing tooling | Medium | Maintain backward-compatible endpoints with deprecation notices; document migration steps. |
| IPFS availability | Medium | Configure multi-provider pinning and retries; monitor via new metrics dashboards. |

---

## Definition of Done for Phase 4

- Distributed compute jobs execute on heterogeneous GPU nodes, produce verifiable proofs, and settle payments automatically.
- Governance parameters, fee schedules, and provider registry administration are managed on-chain with documented processes.
- Production nodes, explorers, and APIs are monitored, alerting, and supported by operational runbooks.

This roadmap will be refined each sprint, but it provides the framework necessary to take Lattice V3 from the current audit baseline to a production-ready distributed AI compute network.
