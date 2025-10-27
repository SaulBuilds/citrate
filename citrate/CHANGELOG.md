# Changelog

All notable changes in this PR are documented here. This follows a Keep‑a‑Changelog‑style summary grouped by impact. Please add an entry in every substantive PR going forward.

## Unreleased

### Highlights
- Added a real TCP transport (length‑delimited, bincode) with a simple handshake and basic DoS protections.
- Implemented synchronous inference RPC `citrate_runInference` and aligned GUI/SDK to current RPCs.
- Registered a public genesis model at chain initialization; added generation/embedding docs.
- Integrated a progressive SyncManager header walk with retry/backoff and peer demotion.
- Added single‑node and multi‑node (5‑node docker) smoke tests.

### Added
- Transport
  - New `NetworkTransport` with Hello/HelloAck handshake (version + network/genesis checks).
  - Length‑delimited frames (1 MB cap), per‑connection message rate limit (200 msgs/sec).
  - Files: `core/network/src/transport.rs`, `core/network/src/lib.rs` (export).
- Sync
  - Progressive header walk via `SyncManager` (tracks `last_received_header`, `last_requested_header`, and pending counts).
  - Retry/backoff (2/4/8/16/32s) and peer demotion on timeouts; ban after repeated failures.
  - Files: `core/network/src/sync.rs`, `node/src/main.rs`.
- API/RPC
  - `citrate_runInference` (sync preview via `Executor.run_inference_preview`) with typed output (json or base64 + metadata).
  - Files: `core/api/src/server.rs`.
- Genesis
  - Embedded genesis model artifact and state registration at init.
  - Files: `node/src/genesis.rs`, `assets/genesis_model.onnx`.
- Test tooling
  - Single‑node inference smoke: `scripts/smoke_inference.sh`.
  - Multi‑node cluster smoke: `scripts/cluster_smoke.sh`, teardown `scripts/cluster_down.sh`.
- Docs & Roadmap
  - P0 roadmap doc: `citrate/ROADMAP_P0.md`.
  - Genesis model doc: `docs/GENESIS_MODEL.md`.
  - RPC docs: documented `citrate_runInference`, `citrate_listModels` (alias: `citrate_getModels`).
  - README: inference example, signing guidance, IPFS env vars, smoke test instructions.

### Changed
- RPC model listing
  - Unified `citrate_listModels`/`citrate_getModels` to return IDs from in‑memory state DB; `citrate_getModel` returns full metadata.
  - Files: `core/api/src/server.rs`.
- GUI/SDK alignment
  - GUI prefers `citrate_listModels` (with fallback), fetches full info via `citrate_getModel`.
  - JS SDK README aligned to `rpcEndpoint`, `sdk.accounts`, list/get/runInference flows; signing best practices.
  - Stubbed out permission RPCs in SDK with clear “Not implemented” errors.
  - Files: `gui/citrate-core/src/services/rpc-client.ts`, `gui/citrate-core/src/services/tauri.ts`, `sdk/javascript/README.md`, `sdk/javascript/src/model.ts`.
- Node networking
  - Node starts listener, dials bootstrap nodes (peer@host:port, ip:port, hostname:port), integrates Discovery peer exchange.
  - Persistent local peer id (stored at `<data_dir>/peer.id`) used in handshake; outbound connections use remote `peer_id` from HelloAck.
  - Gossip used for tx/block propagation; basic sync triggers on Hello/HelloAck.
  - Files: `node/src/main.rs`, `node/Cargo.toml` (added `rand`).

### Security / Robustness
- Transport frame cap: 1 MB.
- Per‑connection message rate limit: 200 msgs/sec.
- Timeout handling in SyncManager with exponential backoff; demote and ban peers after repeated failures.

### Protocol
- HelloAck extends to include `peer_id` (stable remote ID for outbound connections).
  - Backward compatibility: if `peer_id` is missing, the transport falls back to a deterministic placeholder based on the socket address.
  - Files: `core/network/src/protocol.rs`, `core/network/src/transport.rs`.

### Upgrade Notes
- Mixed versions: Nodes that do not send `peer_id` in HelloAck remain compatible (placeholder IDs will be used).
- If you ship a different embedded genesis artifact, ensure all validators/operators rebuild from the same artifact to avoid diverging state.
- For public/testnet RPCs use `eth_sendRawTransaction`; `eth_sendTransaction` is only recommended for local dev with `CITRATE_REQUIRE_VALID_SIGNATURE=false`.

### Testing
- Single node: `scripts/smoke_inference.sh` (set `RPC_URL` to your node if not default).
- Cluster: `scripts/cluster_smoke.sh` (brings up 5‑node docker profile, validates peerCount, block production, and inference), then `scripts/cluster_down.sh`.

