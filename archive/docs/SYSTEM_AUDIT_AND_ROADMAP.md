# Citrate v3 — System Audit, Gaps, and Roadmap

This document summarizes the current state of the Citrate v3 codebase, highlights what is complete vs. partial vs. missing, and proposes a concrete roadmap to reach the envisioned AI‑native BlockDAG platform (compute + storage + governance + incentives).

Scope covers consensus, execution/VM, AI primitives, storage/pinning, economics/rewards, governance, API/RPC, CLI/GUI/Explorer, Solidity contracts, and security/testing.

## Executive Summary

Completed (foundation present)

- Consensus: GhostDAG core, DAG store, tip selection, chain selection are implemented and wired. See `citrate/core/consensus/src/`.
- Execution: Transaction executor with gas accounting and account/state management; block production integrates rewards. See `citrate/core/execution/src/executor.rs` and `citrate/node/src/producer.rs`.
- AI Primitives (Rust): Tensor engine, ZKP backend, AI VM extension and opcodes exist as modules. See `citrate/core/execution/src/tensor/*`, `.../zkp/*`, `.../vm/*`.
- Economics: Reward calculator and native token model defined; producer applies rewards (validator + treasury). See `citrate/core/economics/src/*`, `citrate/node/src/producer.rs`.
- Networking + Sync: Peer manager, DAG sync, and block broadcast loop present. See GUI node and core network crates.
- API: JSON‑RPC subset (ETH compatibility + `citrate_*`), OpenAI‑compatible REST scaffolding. See `citrate/core/api/src`.
- Tooling/UI: Tauri GUI node manager, Explorer app, CLI, SDK.

Partial (functional placeholders or not fully integrated)

- VM Integration: A custom VM with AI opcodes exists (`core/execution/src/vm`), but the main executor currently performs its own bytecode scan and does not call the VM. Duplication needs unification.
- AI Flows: Inference, training, and proofs are simulated in executor/API; the MCP (provider marketplace) exists but is not wired into node or RPC.
- OpenAI REST: Endpoints respond but return placeholder data and asynchronous orchestration is not complete.
- Storage Roots: AI state and storage trees are present, but artifact CIDs exist only as strings; no IPFS client/pinning automation implemented.
- Solidity Contracts: ModelRegistry, LoRAFactory, and InferenceRouter exist and reference precompile addresses, but there are no corresponding Rust “precompile” handlers.

Missing (design needed and/or implementation work)

- Governance: No on‑chain governance primitives for protocol parameters/upgrades exist yet.
- IPFS/Arweave Automation: No pinning service, replication policy, or failure/retry pipeline. No artifact manager.
- Rewards for AI Compute/Storage: Block rewards exist, but provider/trainer/storage fee flows, staking, and slashing are not yet enforced end‑to‑end.
- GPU/Compute Scheduling: MCP not connected to block execution nor to RPC scheduling. No provider payments or slashing on failure.
- Formal VM Semantics/EVM/WASM: The custom VM is not yet a drop‑in EVM/WASM replacement; no bytecode loader/runtime integration for precompiles.

---

## Component Audit

### Consensus (BlockDAG)

- Implemented: GhostDAG, DAG store, tip and chain selection.
  - Files: `citrate/core/consensus/src/ghostdag.rs`, `dag_store.rs`, `tip_selection.rs`, `chain_selection.rs`, `types.rs`.
  - Blue score/work and parents are computed; block producer calls these paths.
- Tests: Present, including `citrate/core/consensus/tests/real_tests.rs`.
- Gaps/Risks:
  - VRF proposer logic exists (`vrf.rs`) but is not used in block production for real leader election.
  - Finality/rewind rules and slashing hooks are not connected to governance or economics.

### Execution and VM

- Implemented:
  - Executor handles transfer, deploy, call, and AI‑typed transactions (parse by data prefix). See `citrate/core/execution/src/executor.rs`.
  - AI VM module with opcodes for tensors, model load/exec, and ZK proofs. See `citrate/core/execution/src/vm/mod.rs` and `vm/ai_opcodes.rs`.
  - Tensor engine and ZKP backend abstractions exist and are internally testable.
- Partial:
  - Executor currently scans bytecode to detect “AI opcodes” and simulates work, rather than invoking the VM. The dedicated VM is not wired into transaction execution, leading to duplication and drift.
  - Precompiles referenced by Solidity (e.g., `0x...1000`) are not handled by executor; no precompile dispatch table.
- Gaps/Risks:
  - No formal EVM/WASM runtime; contract execution is simplified, so Solidity/EVM bytecode beyond proofs and markers will not work as expected.
  - Gas schedule is coarse for AI ops; no resource metering for real tensor/model sizes.

### AI Integrations & MCP (Compute Marketplace)

- Implemented (modules):
  - MCP service with provider registry, reputation scoring, model cache, execution/verifier scaffolding. See `citrate/core/mcp/src/*`.
  - AI state tree in storage with model, training, inference cache, and LoRA entries (`citrate/core/storage/src/state/ai_state.rs`).
- Partial:
  - MCP not wired to RPC or block execution paths; no flow from API → MCP → provider → receipts.
  - Executor simulates inference output rather than delegating through MCP and recording proof artifacts.
- Gaps/Risks:
  - Provider payments and slashing (for failed or malicious results) are not enforced.
  - No GPU capability declaration/attestation flows are persisted on‑chain.

### Storage & IPFS/Artifacts

- Implemented:
  - RocksDB‑based state/chain storage; column families for accounts, storage, code, blocks, txs, receipts. See `citrate/core/storage/src/*`.
  - AI state roots are computed; artifacts referenced via string CIDs in state.
- Partial:
  - Contracts and docs use IPFS CIDs but there is no Rust IPFS client/pinning and no artifact replication policy.
- Gaps:
  - No automated pinning on model register/update or LoRA completion; no retry/replication; no garbage collection policy.

### Economics & Rewards

- Implemented:
  - `RewardCalculator` with halving schedule, inference bonus heuristics, and treasury split. See `citrate/core/economics/src/rewards.rs`.
  - Block producer mints rewards to validator and treasury accounts (`citrate/node/src/producer.rs`).
- Partial:
  - Fees for inference/training/storage are not yet fully charged/settled across transactions and providers.
  - No staking/slashing for providers/validators beyond placeholders.
- Gaps:
  - Tokenomics proposal exists in docs but not codified (e.g., emission policy changes via governance, fee burn, redistribution, MCP fee splitting).

### Solidity Contracts

- Implemented:
  - `ModelRegistry.sol`, `LoRAFactory.sol`, `InferenceRouter.sol`, with precompile integrations and ownership/permissions.
  - Foundry tests mock precompiles via `vm.etch`. See `citrate/contracts/test/ModelRegistry.t.sol`.
- Gaps:
  - No corresponding Rust precompile handlers; on‑chain calls into precompile addresses will currently do nothing in the node.
  - Governance contracts do not yet exist; no parameter registry or voting system.

### API / RPC / REST

- Implemented:
  - JSON‑RPC server with ETH compatibility and custom `citrate_*` methods (deploy model, inference, training). See `citrate/core/api/src/server.rs` and `eth_rpc.rs`.
  - OpenAI‑compatible REST router with endpoints for models, chat, embeddings, and lattice‑specific paths. See `citrate/core/api/src/openai_api.rs` and `methods/ai.rs`.
- Partial:
  - Many OpenAI methods return placeholders and do not orchestrate inference end‑to‑end.
  - WebSocket broadcasts for inference results exist but are not fed by real execution.

### GUI (Tauri) and Explorer

- Implemented:
  - Embedded node manager (Tauri) initializes storage, mempool, peers, and iterative sync. See `citrate/gui/lattice-core/src-tauri/src/node/mod.rs`.
  - Explorer has DAG visualization and RPC‑backed indexer.
- Partial:
  - GUI notes a mempool type mismatch workaround and uses dev/test configs.
  - Explorer has AI‑themed views but no end‑to‑end data for model executions.

### Security & Testing

- Present:
  - Signature verification utilities (`core/consensus/src/crypto.rs`), mempool validation with optional devnet lax mode.
  - Unit tests across consensus, storage, economics, sequencer; Foundry tests for contracts.
- Gaps:
  - No end‑to‑end test that exercises deploy → pin → precompile call → MCP execution → proof → reward distribution.
  - No fuzzing/property tests on VM/tensor ops. No ZK circuit tests beyond scaffolding.

---

## Gaps to Vision Mapping

Target: Immutable knowledge DAG + native AI compute/storage network, paid in LAT, accessible via RPC, with governance and tokenomics.

- Immutable knowledge DAG: DAG and state roots exist; artifact root computed, but no enforced artifact availability (IPFS pinning/replication and availability proofs missing).
- Native AI network: MCP exists in modules, but not connected to RPC or block execution; provider staking, payments, and slashing are not implemented.
- Pay by LAT: Fee flow for AI ops is partial; no automated settlement to providers/trainers or storage pinning providers.
- Governance: No on‑chain governance for protocol params/upgrades; no DAO factory for app‑level governance.

---

## Recommendations and Roadmap

Prioritized milestones (incremental, testable):

1) Unify Executor with VM and Add Precompile Dispatch

- Replace manual AI opcode scan in `executor.rs` with a VM call path:
  - Load contract code, execute via `VM::execute` to handle AI opcodes and standard ops.
  - Keep gas accounting in executor; VM reports gas used.
- Add precompile routing table in executor for addresses like `0x000...1000` and `0x000...1002`:
  - `MODEL_PRECOMPILE`: model register/update/inference entry points that forward to MCP/AI state.
  - `ARTIFACT_PRECOMPILE`: artifact publish/pin commands that integrate with the IPFS manager.
- Outcome: Solidity contracts interact with working system precompiles.

2) Wire MCP Into API and Execution

- In API server, route `citrate_requestInference` to MCP selection and execution pipeline, returning request IDs and emitting WebSocket updates.
- In executor, for inference/training tx types, enqueue/coordinate with MCP (async model):
  - Record a pending receipt initially; finalize with result and proof once MCP finishes.
- Persist proof artifacts via artifact manager (see next step) and reference CIDs on‑chain in AI state.

3) Implement Artifact & IPFS Pinning Automation

- Add a Rust IPFS client and an Artifact Manager service:
  - On model register/update or LoRA completion, pin CIDs (primary and N replicas) and record pin status.
  - Retry with backoff; maintain replication factor; surface status via RPC.
- For storage providers, support multiple backends (IPFS HTTP, web3.storage, Pinata, local gateway) behind a trait.
- Expose artifact status APIs to GUI/Explorer.

4) Fees, Settlement, and Provider Staking/Slashing

- Extend economics to include:
  - Per‑inference and per‑training fees from requester to provider; protocol fee to treasury.
  - Provider staking requirement; slash on verified misconduct (bad proofs, timeouts, unavailability).
- Add minimal staking contract and hooks in MCP for reputation + stake weighting.

5) Governance Primitives (Protocol and App‑level)

- Implement `ProtocolGovernance` contract suite:
  - Parameter Registry (block time, min gas price, reward splits, VRF params, etc.).
  - Proposal + Voting + Timelock; dual‑chamber model (technical vs token chamber) per whitepaper.
  - Precompile or privileged call path to apply approved parameter changes in node runtime.
- Implement `GovernanceFactory` to spawn app DAOs with reusable patterns (treasury, voting, timelock).

6) End‑to‑End Flow and Observability

- Add E2E tests to exercise: deploy model → pin → inference request → provider selection → execution → proof → settlement → UI/Explorer display.
- Add metrics for MCP queue sizes, provider performance, pinning status.

---

## Detailed Designs (Synopsis)

### A. Precompile Design

- Address map:
  - `0x....1000` MODEL: selectors for register/update/inference/training; ABI arguments point to hashes/CIDs and fees.
  - `0x....1002` ARTIFACT: selectors for pin(cid, replicas), status(cid), replicate(cid, provider).
- Executor path:
  - If `to` is precompile, decode selector, call the corresponding service (MCP/Artifact Manager), return ABI‑encoded result.

### B. Artifact Manager (IPFS)

- Trait: `ArtifactBackend` with `pin`, `status`, `replicate`, `unpin`.
- Default backends: IPFS HTTP client + optional Pinata/web3.storage adaptors.
- Policy: replication factor (e.g., 3), K runs, backoff strategy, health checks.
- State: map of CID → providers → pin status; tie into AI state roots.

### C. MCP Execution & Settlement

- Flow: request → provider selection → execution → proof → verification → settlement (LAT transfer) → receipt finalization.
- Proofs: use ZK backend abstraction to generate/verify per job type; store proof CID via Artifact Manager.
- Slashing: if proof invalid or timeout exceeded, penalize provider stake and update reputation.

### D. Governance

- Contracts:
  - `ProtocolParams`: mapping of configurable keys to values; only `Governor` can update.
  - `GovernorChambers`: Technical chamber (weighted by contribution/NFT) + Token chamber (weighted by stake); both may be required depending on proposal type.
  - `Timelock`: execution delay and cancelation rules.
- Runtime:
  - Node reads approved parameter updates from chain and applies to config at epoch boundaries.

### E. Tokenomics Updates

- Fees: per‑inference/training fees distributed as: provider x%, treasury y%, burn z%.
- Rewards: maintain halving for base rewards; add AI‑specific bonuses gated by actual usage (counting MCP receipts rather than heuristics).
- Staking: providers must post stake proportional to declared capacity; higher weight with performance reputation.

---

## What’s Done vs. Not (Per Module)

- Consensus: DONE (core) / NEEDS: VRF integration, slashing hooks.
- Execution: PARTIAL (gas/state ok) / NEEDS: VM integration, precompiles, formal runtime.
- VM (AI opcodes): DONE (module) / NEEDS: wire into executor, gas metering by tensor/model size.
- MCP: PARTIAL (module) / NEEDS: RPC wiring, settlement, staking, slashing.
- Storage: PARTIAL / NEEDS: IPFS automation, replication, artifact service.
- Economics: PARTIAL / NEEDS: provider fees, staking, burn, fee split codified.
- Contracts: PARTIAL / NEEDS: governance suite, precompile support in node.
- API/REST: PARTIAL / NEEDS: real orchestration and streaming; integrate MCP + artifacts.
- GUI/Explorer: PARTIAL / NEEDS: status views for models, jobs, artifacts; real‑time inference results.
- Security/Tests: PARTIAL / NEEDS: E2E, fuzzing, ZK circuit tests, provider slashing tests.

---

## Immediate Action Items (2–3 weeks)

1) Add precompile dispatch in executor and stub handlers for MODEL/ARTIFACT.
2) Replace executor bytecode scanning with VM execution path for AI ops.
3) Introduce `ArtifactManager` crate with IPFS client and wire ModelRegistry flows to pin on register/update.
4) Wire MCP to API methods; return request IDs and stream results via WebSocket.
5) Basic provider payment flow: charge requester, pay provider, take protocol fee.
6) Draft `ProtocolGovernance` contracts; add a read‑only runtime bridge for parameter updates.

---

## File References (Key Entry Points)

- Consensus
  - `citrate/core/consensus/src/ghostdag.rs`
  - `citrate/core/consensus/src/dag_store.rs`
  - `citrate/node/src/producer.rs:240`
- Execution/VM
  - `citrate/core/execution/src/executor.rs:440`
  - `citrate/core/execution/src/vm/mod.rs:1`
  - `citrate/core/execution/src/vm/ai_opcodes.rs:31`
- AI/MCP
  - `citrate/core/mcp/src/lib.rs:16`
  - `citrate/core/mcp/src/provider.rs:85`
  - `citrate/core/storage/src/state/ai_state.rs:18`
- Storage
  - `citrate/core/storage/src/state/state_store.rs:112`
  - `citrate/core/storage/src/chain/block_store.rs:7`
- Economics
  - `citrate/core/economics/src/rewards.rs:51`
  - `citrate/node/src/producer.rs:296`
- Contracts
  - `citrate/contracts/src/ModelRegistry.sol:14`
  - `citrate/contracts/src/LoRAFactory.sol:13`
- API / REST
  - `citrate/core/api/src/server.rs:384`
  - `citrate/core/api/src/openai_api.rs:67`
- GUI
  - `citrate/gui/lattice-core/src-tauri/src/node/mod.rs:1`
  - `citrate/gui/LATTICE_GUI_INTEGRATION_PLAN.md`
- Explorer
  - `citrate/explorer/src/indexer/index.ts:380`

---

## Risks & Open Questions

- Security of precompiles and privileged operations requires strict ABI validation and access controls.
- Proof verification costs must be tuned; consider off‑chain proving with on‑chain verification only.
- Storage availability: require minimum replication and introduce storage provider incentives.
- Governance complexity: dual‑chamber weighting and identity for “technical chamber” need a robust attestations model.

---

## Conclusion

The codebase provides solid consensus, storage, execution scaffolding, and early AI primitives. To unlock the full AI‑native, pay‑by‑LAT platform, focus on: (1) VM/precompiles unification, (2) MCP wiring + settlement, (3) automated IPFS artifact management, and (4) governance + tokenomics enforcement. The roadmap above sequences these to deliver visible, testable progress.

