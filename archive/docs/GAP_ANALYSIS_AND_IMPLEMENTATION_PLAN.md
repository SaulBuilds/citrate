# Gap Analysis and Implementation Plan (v1)

Purpose: identify concrete gaps between the current codebase and the initial Lattice vision, and define an incremental, test‑driven plan to deliver a credible v1 that aligns with stated goals without breaking existing behavior.

## 0) Vision → Deliverables Map

- BlockDAG performance: demonstrate parallel block processing with multi‑node stability and published metrics.
- Cross‑chain verification: ship minimal ETH and BTC proof verification flows (read‑only) with on‑chain/engine verifiers.
- AI‑native substrate: functional Model Registry + provenance; evaluation tiers T0–T1 (attestation + deterministic recomputation).
- EVM compatibility: conformance against a curated Ethereum tests set; stable RPC surface.
- ZK integration: in‑node precompiles for Groth16/Plonk verification; sample circuits and end‑to‑end RPC.
- Multi‑VM runway: WASM PoC (sandboxed), cross‑VM call boundary; docs + SDK note.
- Operator UX: docker profiles, monitoring dashboards, downloads, setup guides.

## 1) Current State (concise)

- Node, CLI, GUI, Explorer, Docs build; devnet runs; RPC works; CI and releases exist.
- No published performance numbers; multi‑node soak not exercised.
- Cross‑chain/ZK/Multi‑VM are not implemented beyond stubs.
- Model registry and AI flows exist partially; need end‑to‑end proof.
- SDK/GUI stabilized; explorer containers added.

## 2) Gaps (by domain)

- Consensus/Performance: no multi‑node harness, no steady‑state TPS/finality metrics, no GC/pruning stress tests.
- EVM Conformance: edge‑case semantics unverified; missing canonical test vectors; gas/revert coverage light.
- Cross‑chain: no ETH beacon client, no BTC SPV, no proof verifiers.
- ZK: no verifier precompiles; no circuits; no ceremony/process docs.
- AI Registry/Semantics: contracts and RPC exist but lack lineage/evaluation semantics + T0/T1 workflows and explorer views.
- WASM: no runtime or bridge; no ABI or call boundary.
- Networking: limited peer discovery/handshake observability; no NAT/UPnP/STUN; no churn tests.
- Storage: RocksDB tuning, snapshots, pruning boundary tests, state rent policy.
- SDK: TS fixes pending for a v0.1 release; samples incomplete.
- Docs: placeholders for operators/providers/security need content; no published benchmark reports.

## 3) Release Epics, Acceptance Criteria, and Tests

E1: Multi‑node Stability + Performance
- Deliverables: 5‑node local cluster (compose + k8s optional), perf harness, dashboards.
- Acceptance:
  - Sustained ≥1,000 TPS for 10 min on 5 nodes with ≤5% tx loss; 2–3s block interval; finality ≤15s.
  - Metrics exported (blocks/s, mempool depth, peer counts, gossip latency) and Grafana dashboards.
- Tests: load generator, soak tests, regression perf snapshot stored in CI artifacts.

E2: EVM Conformance & RPC Completeness
- Deliverables: curated ethereum/tests suite subset, gas/revert tests, RPC parity matrix.
- Acceptance:
  - ≥90% pass on selected GeneralStateTests/BlockchainTests subset; documented exclusions.
  - RPC spec doc with implemented/unsupported; error codes normalized.
- Tests: integrate test vectors; CI job with results summary.

E3: Minimal Cross‑Chain Verification (ETH + BTC)
- Deliverables: 
  - BTC: SPV header chain + Merkle proof verify; RPC `bridge_verifyBtcTx`.
  - ETH: beacon header light client (sync committee) or checkpoint verifier; RPC `bridge_verifyEthProof`.
- Acceptance:
  - End‑to‑end: given proof input, verifier returns verified=true with reason codes on failure.
  - Precompile interface or native engine call defined and tested.
- Tests: golden proofs from known networks; property tests for proof parsing/validation.

E4: ZK Verifier Precompiles (Groth16/Plonk)
- Deliverables: in‑node verifier precompiles; sample circuits repo; RPC to submit/verify proof.
- Acceptance:
  - Verify example proofs (hash preimage / Merkle branch) in ≤100ms on dev hardware.
  - Negative tests fail deterministically; gas/fee model documented (even if off‑chain now).
- Tests: circuit CI; verifier unit + integration tests; micro‑benchmarks.

E5: Model Registry + T0/T1 Evaluation
- Deliverables: registry contracts finalized; provenance metadata schema; deterministic recomputation harness.
- Acceptance:
  - Register → publish artifacts → attest T0 (env/seed/package digests) → T1 (deterministic small split) verified; explorer shows lineage.
  - Stake/permission checks enforced; SDK samples for deploy/evaluate.
- Tests: contract unit tests; RPC integration; explorer indexing tests.

E6: WASM PoC + Cross‑VM Boundary
- Deliverables: wasmtime sandbox; simple wasm contract calling a precompile; ABI draft for cross‑VM.
- Acceptance:
  - Run demo wasm contract; EVM↔WASM call boundary tested in a small flow.
- Tests: unit + e2e demo; security sandbox constraints verified.

E7: SDK v0.1 + Samples
- Deliverables: fixed TS types; published NPM prerelease; examples for wallet, contracts, model flows.
- Acceptance:
  - `npm i @lattice/sdk@0.1.0-alpha` works; examples run against devnet; typed API.
- Tests: unit tests in SDK; smoke in CI.

E8: Operator UX + Monitoring
- Deliverables: compose profiles; grafana dashboards; install docs; downloads wired.
- Acceptance:
  - One‑page operator guide; successful install on macOS/Windows/Linux; monitoring verified.
- Tests: CI build of installers; link checks; optionally scripted smoke.

E9: Security & Docs
- Deliverables: security overview, threat model, audits plan placeholder; SBOM and code scanning.
- Acceptance:
  - Docs pages written; GitHub Advanced Security or CodeQL enabled; dependency audit passes.

## 4) Sequencing & Timeline (indicative, parallelizable)

- Month 1: E1, E2 groundwork; E7 SDK fixes; E8 dashboards/docs.
- Month 2: E3 (BTC SPV), E4 (Groth16 verifier), continue E2.
- Month 3: E3 (ETH light client minimal), E5 T0/T1, publish benchmarks.
- Month 4: E6 WASM PoC, governance/economics scaffolding, doc polish.

## 5) Key Design Choices (recommended)

- ZK: start with `arkworks` Groth16; later add Plonkish via `halo2`.
- BTC SPV: headers only + Merkle proof, no script validation beyond P2PKH for demo; store in node DB.
- ETH light client: sync‑committee checkpoint verification (minimal), not full beacon spec; `rust-ssz` + `kzg` as needed.
- Precompiles: define stable ABI for verifiers and bridge queries; keep as node‑native precompile layer initially, document gas surrogate.
- WASM: use wasmtime with WASI disabled by default; explicit host calls to precompiles only.

## 6) CI/CD & Quality Gates

- Perf jobs (nightly): run multi‑node; collect metrics; compare to baselines.
- Conformance job: run ethereum test subset; publish report.
- Verifier job: install `solc`; run ZK verifier tests; upload proof vectors.
- Lints: `cargo clippy -D warnings`; TS strict; deny warnings in critical crates.
- Release gates: pass E2 subset, perf threshold (e.g., ≥500 TPS initially), and ZK/BTC demos.

## 7) Risks & Mitigations

- Perf shortfall: prioritize batching, RocksDB options, concurrency tuning; backpressure in mempool.
- ZK complexity: start small (Groth16), defer recursion; reuse known circuits.
- Light client complexity: limit to proven minimal paths; mock where needed but deliver one real proof path.
- Schedule pressure: parallelize epics; stage demos (BTC first, ETH later).

## 8) Work Backlog (initial)

- Harness: 5‑node compose; load generator crate; Grafana dashboards.
- EVM tests integration; error normalization in RPC; precompile ABI spec doc.
- BTC SPV store + proof verify; RPC + tests; explorer view.
- Groth16 verifier precompile; sample circuits; CI wiring.
- Registry contract finalize; SDK flows; explorer lineage page.
- WASM PoC crate; cross‑VM ABI; demo contract + tests.
- SDK publish; examples repo; docs.
- Operator guide; downloads + notarization/signing follow‑ups.

---
This plan is designed to be executed incrementally with measurable outcomes after each epic, without breaking existing behavior. Acceptance criteria and tests are defined to gate progress and align claims with working code.
