# Citrate V3 – P0 Engineering Roadmap

Purpose: single source of truth for immediate, granular tasks to make the devnet usable end‑to‑end (GUI/CLI/SDK + RPC) and unblock multi‑node beta work. Keep this file updated as items land.

## Scope (P0)
- RPC parity for GUI/CLI/SDK (aliases + inference API)
- Sane default signing path (prefer raw tx)
- Genesis model artifact plan and registration path
- Minimal docs alignment for setup and usage

## Workstreams and Tasks

1) RPC Parity and Inference
- [x] Add JSON‑RPC alias `citrate_getModels` (GUI calls it today)
- [x] Implement `citrate_runInference` using Executor `run_inference_preview`
- [x] Return stable result shape: output (JSON if possible; fallback b64), execution_time_ms, gas_used, optional proof
- [ ] Keep existing `citrate_requestInference`/`citrate_getInferenceResult` for async flow (later P1)

2) Client Alignment (GUI/SDK/CLI)
- [x] GUI: prefer `citrate_listModels` (fallback to alias) and fetch model info per ID
- [x] SDK: avoid permission RPCs not yet implemented; rely on deploy/list/get/inference only
- [x] CLI/docs: document and prefer `eth_sendRawTransaction` for state‑changing ops; keep `citrate_*` endpoints for models

3) Signing Path (Dev vs Prod)
- [x] Default guidance: raw‑tx for public RPCs
- [x] Devnet convenience: `CITRATE_REQUIRE_VALID_SIGNATURE=false` documented
- [x] Update README quick‑start with explicit guidance

4) Genesis Model
- [x] Commit tiny ONNX placeholder at `citrate/assets/genesis_model.onnx`
- [x] Register model at genesis (node init) so it’s visible via `citrate_getModel`
- [x] Document generation pipeline (`assets/genesis_model_generator.py`) and artifact management

## Milestones
- M1 (Dev Alpha Usability): RPC alias + `citrate_runInference` + docs updates; CLI/GUI/SDK can list models and run inference locally
- M2 (Genesis Model Ready): ONNX artifact present; model registered at init; basic inference against genesis model works
- M3 (Pre‑Beta Prep): clarify client signing behavior; stabilize JSON‑RPC responses; basic smoke tests

## Risks / Mitigations
- Output format variability: parse bytes as JSON, else return base64 alongside metadata
- Large artifacts in git: prefer LFS or releases for ONNX
- Security: Keep CORS wide only for dev; lock down in prod configs later

## Validation
- CLI: deploy (optional) and run inference → structured output
- GUI: connects, lists models (even if mock), runs inference via `citrate_runInference`
- SDK: `deployModel`, `runInference`, `getModel` succeed against devnet

## Update Notes
- Keep this file in PRs touching RPC/clients/genesis
- Use checkboxes and timestamps as items land
