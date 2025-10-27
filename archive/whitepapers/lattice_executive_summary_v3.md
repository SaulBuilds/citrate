# Citrate Network — Executive Summary (v3, 2025-09-06)

## What We Are
Citrate is an AI‑native Layer‑1 **BlockDAG** chain using **GhostDAG** consensus, paired with an **EVM‑compatible** execution environment and a standardized **Model Context Protocol (MCP)** layer. We make AI models first‑class on-chain assets: registries, weights, training/eval logs, and verifiable provenance are committed on-chain with off‑chain artifacts referenced via CIDs.

## Why Now
- **AI × Crypto convergence**: demand for verifiable AI and safe cross‑chain coordination.
- **Performance & resilience**: GhostDAG tolerates high concurrency via multi‑parent blocks with deterministic total order.
- **Developer familiarity**: OpenAI/Anthropic‑style endpoints and EVM tooling reduce switching costs.

## Core Technical Choices
- **Consensus**: GhostDAG with selected‑parent chain, merge‑parents, blue set/blue score ordering; VRF‑based proposer selection; fast finality gadget.
- **VM**: EVM‑compatible (Citrate VM, “LVM”) with model‑centric precompiles; WASM roadmap.
- **MCP**: chain‑native standard defining request/response schemas, context packing, provenance, and safety policies.
- **Primitives (on‑chain contracts)**:
  1) **ModelRegistry** (IDs, versions, artifacts, attestation),  
  2) **LoRAFactory** (creation, merge, compose),  
  3) **InferenceRouter** (job spec, SLA, payment, result),  
  4) **StorageRegistry** (CIDs, redundancy policies, retention/SLA),  
  5) **ComputeMarket** (GPU supply/demand, scheduling, slashing),  
  6) **Trainer** (federated/fine‑tune jobs, checkpoints),  
  7) **Eval/Attest** (benchmarks, Proof‑of‑Training/Evaluation),  
  8) **Bridge** (ZK attestations for cross‑chain data/asset flows).

## Business & Impact
- **Developers**: one‑line migration via MCP adapters; JSON‑RPC + REST endpoints compatible with OpenAI/Anthropic conventions.
- **Enterprises**: verifiable AI, data residency controls, permissioned validators, audit trails.
- **Token**: fees for tx/inference/training/storage; staking for security; slashing for SLA violations in ComputeMarket.

## Targets (initial)
- ≥10k TPS testnet, ≤12s finality, 99.5% uptime; API compatibility coverage ≥90%; 500+ developers by mainnet.

## Call to Action
Run a validator, deploy a model with `ModelRegistry`, fine‑tune via `LoRAFactory`, and serve predictions through `InferenceRouter`. The future is parallel, verifiable, and interoperable — the future is **Lattice**.
