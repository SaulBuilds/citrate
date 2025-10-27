# Citrate Network — Technical Whitepaper (v3, 2025-09-06)
## A GhostDAG Layer‑1 for Verifiable AI and Interoperable ML Tooling

### Abstract
We present Citrate, a BlockDAG blockchain using **GhostDAG** consensus and an EVM‑compatible VM. Citrate introduces a chain‑native **Model Context Protocol (MCP)** and a suite of **AI primitives** that make model artifacts, training, and inference verifiable and composable on‑chain. We formalize block structure, ordering, and state commitments to support multi‑parent blocks, define an execution semantics for model workflows, and outline incentives that align compute supply with model demand.

---

## 1. Motivation
Modern AI systems lack verifiable provenance, robust auditability, and open market access to compute. Blockchains lack native abstractions for model lifecycles. Citrate unifies these domains with a GhostDAG consensus, MCP, and contracts that orchestrate models, compute, and data.

## 2. Consensus: GhostDAG
### 2.1 Block Graph
Each block `B` references one **selected_parent** and ≥0 **merge_parents**. Let `Blue(B)` be the maximal set consistent with GhostDAG's k‑cluster rule; `blue_score(B)` totals ancestry‑consistent blue mass. Total order is the selected‑parent chain plus the mergeset, topologically sorted by blue scores.

**Header Fields (overview)**
```
version, block_hash
selected_parent_hash
merge_parent_hashes[]
timestamp (ms), height
state_root, tx_root, receipt_root, artifact_root
blue_score, ghostdag_params (k, window, pruning)
proposer_pubkey, vrf_reveal, difficulty
signature
```
### 2.2 Liveness and Finality
- **Tip selection**: highest blue_score; anti‑past awareness to avoid wide forks.
- **Finality gadget**: committee BFT checkpoints; optimistic confirmation in ≤12s.

## 3. Execution: Citrate VM (LVM)
EVM‑compatible execution with deterministic gas metering and precompiles for model tasks:
- `mcp.hash_context(bytes) -> bytes32`
- `mcp.blob_commit(bytes32 cid, bytes meta) -> commitment`
- `lora.apply(delta, base_model) -> new_model`
- `eval.score(model_id, dataset_id) -> metrics_root`

State is persisted in a Merkle‑Patricia Trie; large artifacts live off‑chain (IPFS/AR, S3 w/ gateways), bound via `artifact_root` and CIDs.

## 4. Model Context Protocol (MCP)
### 4.1 Goals
A canonical schema for **requests**, **context**, **constraints**, **provenance**, and **safety**. MCP normalizes payloads across providers (OpenAI/Anthropic‑style) and on‑chain usage.
### 4.2 Objects
- **Model**: `id`, `version`, `capabilities`, `license`, `artifact_cids[]`, `policy`.
- **Context**: references to prompts, tools, datasets, and prior outputs.
- **Job**: `type=inference|train|eval`, `SLA`, `price`, `verifier`.
- **Attestation**: proofs and signatures linking outputs to inputs and versions.

## 5. Primitives (On‑Chain Contracts)
1. **ModelRegistry** — register, version, deprecate, link artifacts; emit `ModelRegistered`, `VersionAdded`.
2. **LoRAFactory** — mint/update LoRA deltas; merge/compose; on‑chain royalties.
3. **InferenceRouter** — post jobs, match providers, escrow payments, post receipts.
4. **StorageRegistry** — declare CIDs, replication factor, erasure codes, retention.
5. **ComputeMarket** — staking, GPU supply/demand auctions, slashing on SLA failure.
6. **Trainer** — coordinate fine‑tunes/federated rounds; checkpoints & lineage DAG.
7. **Eval/Attest** — standardized benchmarks and Proof‑of‑Training/Evaluation.
8. **Bridge** — ZK light‑client proofs for trust‑minimized interop.

## 6. Sequencing & Block Building
- **Mempool segregation**: tx classes (consensus, model, data, compute).
- **Bundle policy**: model‑updates co‑located with commitments to artifacts & logs.
- **Parent selection**: pick virtual selected‑parent tip; include eligible parallel tips as merge parents; recompute blue set; produce block and broadcast.
- **Timestamping**: wall‑clock for UX; consensus order from blue‑score chain governs state machine time.

## 7. Data & Provenance
- **On‑chain commitments**: `artifact_root` binds to content‑addressed blobs.
- **Off‑chain availability**: IPFS/Arweave/S3 through StorageRegistry providers.
- **Reproducibility**: deterministically rebuildable runs from MCP job manifests.

## 8. Token & Incentives (outline)
- Fees on tx, inference, training, storage; staking and slashing in ComputeMarket.
- Grant programs for open benchmarks and public‑goods datasets.

## 9. Security & Economics
- **Consensus**: GhostDAG k‑cluster security; pruning & anti‑past limits.
- **Bridges**: SNARK‑verified state transitions; no trusted custodians.
- **Model fraud**: require attestations; stake‑weighted challenges; slash misreports.

## 10. Roadmap (high level)
- Testnet α: GhostDAG+LVM, ModelRegistry, InferenceRouter.
- Testnet β: ComputeMarket, LoRAFactory, Eval/Attest, ZK Bridge MVP.
- Mainnet: Finality gadget, MCP 1.0, enterprise features.

## 11. References
The v3 spec supersedes earlier GHAST drafts and expands AI‑native sections with MCP and primitives.
