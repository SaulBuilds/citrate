# Citrate — User Stories (v3, 2025-09-06)

## Epic A: MCP & API Compatibility
**As a** developer, **I want** OpenAI/Anthropic‑style endpoints, **so that** I can reuse existing tooling.
- `/v1/chat/completions`, `/v1/embeddings`, `/v1/messages`, `/v1/models`
- SDK examples (TS/Python/Rust); streaming and tool‑use support.
**Acceptance**: parity tests vs. reference providers; 99% spec coverage.

## Epic B: Model Lifecycle Primitives
**As a** model owner, **I want** to version models and attach LoRA deltas, **so that** I can iterate safely.
- `ModelRegistry`, `LoRAFactory`, royalties, provenance (attestations).
**Acceptance**: register → fine‑tune → evaluate → publish → deprecate workflows succeed end‑to‑end.

## Epic C: Inference & Compute Market
**As a** provider, **I want** to accept inference jobs with escrowed payment and SLAs, **so that** I can monetize GPUs.
- `InferenceRouter` + `ComputeMarket`; stake/slashing; audit logs.
**Acceptance**: failure injection shows refunds/slashing; throughput >1k RPS per region.

## Epic D: GhostDAG Explorer & Ops
**As a** participant, **I want** to visualize mergesets/selected‑parent chains, **so that** I can reason about finality and forks.
**Acceptance**: DAG view, fork timelines, blue score overlays; APIs for analytics.

## Definition of Done
- Code coverage ≥80%; integration & E2E tests green; docs + SDK samples; production runbooks present.
