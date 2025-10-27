# Citrate V3 – Phase 3 Readiness Audit (March 2025)

This report captures the current production posture of the Citrate V3 stack as we enter Phase 3 (Distributed Compute) and sets the baseline for the forthcoming sprint roadmap.

---

## Executive Summary

- **Foundation in good shape**: GhostDAG consensus, DAG storage, RocksDB-backed state, and the Rust node/CLI toolchain form a stable base. Multi-node networking remains healthy.
- **AI/compute pipeline still synthetic**: MCP execution, inference precompiles, and Metal runtime paths return placeholder data. The RPC layer bypasses consensus for model deployment and does not orchestrate real GPU jobs.
- **Governance & incentives wiring incomplete**: VRF-based proposer selection, provider staking/slashing, and treasury distribution hooks are defined but not wired through block production.
- **Documentation cleanup done**: Historic phase reports, sprint logs, and test harness scripts have been moved to `archive/` to keep only production-facing assets in the workspace.

Phase 3 should focus on replacing the stubs in the inference pipeline with real execution flows, wiring incentives, and delivering verifiable distributed compute.

---

## Platform Readiness Snapshot

| Subsystem | Status | Notes |
|-----------|--------|-------|
| Consensus & DAG | ✅ Stable | GhostDAG/tip selection solid (`core/consensus/src/ghostdag.rs`, `tip_selection.rs`). VRF leader election exists but unused. |
| Execution (EVM + AI) | ⚠️ Partial | Transaction pipeline functional; AI paths short-circuit via stubs. |
| Storage & IPFS | ⚠️ Partial | RocksDB + state trie solid; IPFS client integrates chunking but lacks persistence/backpressure. |
| Networking & Sync | ✅ Ready | Peer manager, gossip, and DAG sync exercised in multi-node scripts. |
| API / RPC | ⚠️ Partial | ETH JSON-RPC implemented; `citrate_*` methods skip mempool/consensus, OpenAI facade returns canned data. |
| MCP & Provider Flow | ❌ Stubbed | ModelExecutor returns placeholder output, no provider attestation, no real scheduling. |
| Tooling (CLI/GUI) | ⚠️ Partial | CLI commands function against RPC stubs; GUI references precompute data. |
| Solidity Contracts | ⚠️ Partial | Registry/router contracts ready, but on-chain precompile counterparts missing real execution. |

---

## Highlights by Component

### Consensus & Core Node
- GhostDAG, DAG storage, and tip selection remain production-ready with existing integration tests (`core/consensus/tests/real_tests.rs`).
- VRF proposer logic is implemented but only exercised in unit tests; block production still uses static leader configuration (VRF structs in `core/consensus/src/vrf.rs` are not consumed by `node/src/producer.rs`).

### Execution & AI Pipeline
- The EVM-style executor normalizes addresses and unifies VM execution (`core/execution/src/executor.rs`), but AI opcodes divert into simulated paths.
- MCP service, inference service, and precompiles exist, yet they still simulate inference/training, returning deterministic filler data instead of calling GPU runtimes.

### Storage & IPFS
- IPFS integration supports chunk manifests and pinning summaries (`core/storage/src/ipfs/{chunking.rs,pinning.rs}`), yet metadata is cached in memory only, and failure recovery/backpressure is minimal.

### API & Tooling
- JSON-RPC coverage is wide, but the bespoke `citrate_*` endpoints mint balances and execute transactions directly via the executor, bypassing mempool validation.
- The OpenAI-compatible REST layer exposes routes but serves placeholder payloads; job orchestration is absent.

### Smart Contracts & Explorer
- Solidity contracts capture the intended compute marketplace semantics, though they assume functioning precompiles. Explorer/GUI components still rely on mocked inference data.

---

## Critical Findings (Requires Phase 3 Attention)

1. **Inference runtime returns synthetic outputs**  
   - `ModelExecutor::execute_in_vm` and `execute_training_in_vm` simply emit zeroed tensors and fixed gas usage (`core/mcp/src/execution.rs:222-250`).  
   - The Metal runtime's backend methods return zero-filled vectors (`core/execution/src/inference/metal_runtime.rs:238-275`), so no real GPU work occurs.

2. **Metal runtime/precompile struct mismatch**  
   - `MetalModel` lacks a `weights_path` field (`core/execution/src/inference/metal_runtime.rs:55-62`), yet the precompile constructs one (`core/execution/src/precompiles/inference.rs:143-156`) and the runtime dereferences it (`core/execution/src/inference/metal_runtime.rs:214-215`), which would panic once exercised.

3. **RPC-side deployment bypasses consensus economics**  
   - `citrate_deployModel` seeds unlimited balance and executes the transaction directly in-process (`core/api/src/server.rs:1473-1513`), skipping signature validation, mempool policy, and block inclusion. This must be replaced with a proper transaction submission path.

4. **OpenAI facade and job orchestration are placeholders**  
   - `AiApi::list_models` returns an empty vector outside of owner-filtered calls (`core/api/src/methods/ai.rs:309-330`).  
   - `request_inference` enqueues empty transactions with no payload encoding (`core/api/src/methods/ai.rs:355-417`), and `get_inference_result` fills metadata with hard-coded markers (`core/api/src/methods/ai.rs:420-439`).

5. **Leader election not wired**  
   - VRF leader election lives entirely in the consensus crate but is unused by the node; block production in `node/src/producer.rs` still chooses proposers deterministically. This undermines incentive design and compute-market fairness.

---

## Phase 3 Implementation Focus

1. **Real inference execution path**
   - Integrate the MCP executor with actual runtimes (Metal/CoreML on macOS, CUDA/ROCm via adapters) and persist artifacts fetched via IPFS.
   - Populate `ModelExecutor::execute_in_vm` with real VM invocation and remove placeholder outputs.
   - Ensure inference results carry concrete proofs or attestations that can be verified on-chain.

2. **Precompile and contract parity**
   - Align Rust precompile structures with Solidity expectations (fix the `MetalModel` shape, persist weights/artifacts, implement benchmarking/proof opcodes).
   - Surface the precompiles through the executor so EVM contracts (e.g., `InferenceRouter`) observe real behavior.

3. **RPC & CLI hardening**
   - Route model deployment/inference through the mempool and consensus instead of direct executor calls.
   - Replace placeholder REST responses with storage-backed queries and asynchronous job tracking hooked into MCP/provider services.

4. **Distributed compute marketplace**
   - Implement provider registration, job scheduling, fee accounting, and slashing in MCP (`core/mcp/src/provider.rs`) with accompanying storage/state updates.
   - Wire VRF leader election and incentive mechanisms so GPU nodes compete fairly for jobs.

5. **Observability & reliability**
   - Persist pinning metadata, add retries/backoff in `IPFSService`, and emit metrics for MCP execution latency/gas usage.
   - Extend existing Prometheus metrics to cover inference throughput and provider participation.

---

## Testing & Operational Posture

- Consensus and state tests exist, but AI/MCP integration lacks automated coverage. Add unit tests around real runtime adapters once implemented.
- Retain `load_test.sh`, `monitor_testnet.sh`, and production scripts in `scripts/`; all historical test harnesses now reside in `archive/scripts/`.
- Ensure CI runs deterministic checks around new compute paths; GPU-dependent tests should be gated or mocked for portability.

---

## Archived Assets (for reference)

Following files were moved to `archive/` because they are historic planning artifacts or ad-hoc validation helpers no longer required for production operations:

- Phase history: `archive/phase-history/PHASE1_COMPLETE.md`, `PHASE1_VERIFICATION.md`, `PHASE2_*`, `WEEK_1_2_*`.
- Legacy planning docs: `archive/docs/CURRENT_STATE_ANALYSIS.md`, `GAP_ANALYSIS_AND_IMPLEMENTATION_PLAN.md`, `SYSTEM_AUDIT_AND_ROADMAP.md`, and related sprint notes.
- Testing harnesses: `archive/scripts/test_*.sh`, `run_e2e_tests.sh`, `chaos_testing.sh`, etc.

---

With the above baseline, Phase 3 sprints should deliver a functional distributed compute layer, credible proofs of execution, and production-caliber APIs. The roadmap in the companion document will decompose these focus areas into sprint-sized workstreams for Phases 3 and 4.
