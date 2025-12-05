# Audit Roadmap (GUI-Embedded Node & Codegen Focus)

## Objectives
- Verify the GUI can run an embedded node end-to-end: start, sync, produce blocks, execute contracts, and expose a working wallet/RPC surface.
- Validate the code-generation tooling (LLM + templates) can create, compile, deploy, and interact with smart contracts from within the GUI.
- Assess consensus safety, execution correctness, and storage integrity required to secure the chain while users earn tokens through block production and AI work.

## Scope & Assumptions
- In-scope components: consensus, execution (EVM + precompiles), sequencer/mempool, API/RPC, wallet, GUI (React + Tauri), agent tools (LLM/codegen/IPFS), contracts, networking/P2P, storage, and genesis configuration.
- Out-of-scope for this phase: external explorer deployments and marketing sites (unless they affect embedded-node flows).
- Target platforms: macOS + Apple Silicon (Metal/CoreML) for inference; Linux nodes for headless deployments.

## Workstreams (suggested order)
1) **Reproduce & Baseline**
   - Build/test in release/debug; capture compiler warnings; confirm feature flags.
   - Validate genesis and config defaults for embedded node vs. standalone node.
2) **Consensus & Sync**
   - GhostDAG blue set, ordering, finality, k-cluster enforcement; sync correctness (`core/consensus`, `node/src/genesis.rs`).
   - Check efficient sync and DAG tracking initialization paths for liveness.
3) **Execution & Precompiles**
   - EVM core and opcode gas metering; dual address derivation; chain ID handling; AI precompile resource bounding.
   - Compare REVM adapter vs. native engine outputs on canonical vectors.
4) **Transaction Pipeline & Mempool**
   - RPC decoding, signature recovery, mempool DOS controls, nonce gaps, fee handling, block builder roots.
5) **Network/P2P**
   - Gossip validation, rate limits, peer selection diversity, Sybil/Eclipse resistance, AI message handling.
6) **Storage & State Integrity**
   - RocksDB/MPT persistence, snapshotting, pruning, and consistency checks across node restarts and GUI-embedded node mode.
7) **GUI Embedded Node (Tauri)**
   - Node lifecycle inside Tauri process; RPC bridge; mempool type compatibility; `eth_call` behavior; block production path; wallet integration.
8) **Agent & Codegen Tooling**
   - LLM prompt/response handling, tool sandboxing, MCP connectivity, template safety, contract deployment helpers, receipt polling.
9) **Smart Contracts**
   - Access control, reentrancy, economics, upgrade patterns; align with GUI codegen defaults.
10) **Observability & Operations**
    - Logging, metrics, alerts for consensus/execution/agent; crash recovery; data directory permissions.
11) **Testing & Verification**
    - Unit/integration/E2E; fuzz/property tests on consensus and execution; determinism checks; cross-client conformance where possible.

## GUI E2E Acceptance Path (must all pass)
1) Launch GUI → embedded node starts with correct genesis/config; RPC server reachable on loopback.
2) Node syncs to tip (or seeds DAG if offline mode) without `{processed:0}`/stubbed responses.
3) Wallet onboarding: key creation/import; dual address derivation consistent across wallet, executor, and RPC.
4) Faucet/test tokens (or local mint) succeed; balances reflect in GUI.
5) Codegen flow: prompt → contract scaffold → lint/compile (Foundry) → deploy via embedded node RPC → address returned and stored.
6) Contract interactions: `eth_call` and `eth_sendRawTransaction` work from GUI, including pending nonce handling and receipt polling.
7) Block production: GUI shows blocks produced, state/receipt roots correct, DAG view updates.
8) AI actions: IPFS upload, model registry updates, inference precompile calls; rewards emitted/credited.
9) Shutdown/restart: node restarts cleanly, retains state, no DB corruption or replay failures.

## Priority Checklists (near-term)
- Clear critical stubs affecting E2E:
  - GUI RPC server/mempool type mismatch (`gui/citrate-core/src-tauri/src/node/mod.rs`).
  - `eth_call` error path (`gui/citrate-core/src-tauri/src/lib.rs`).
  - Block builder root stubs (`core/sequencer/src/block_builder.rs`).
  - Consensus efficient sync returns `{processed:0}` (`core/consensus/src/sync/efficient_sync.rs`).
  - AI network handler returns `Ok(None)` (`core/network/src/ai_handler.rs`).
- Remove mock data that hides failures: DAG explorer stats, fee history, analytics zeros, AI generation/storage mocks, IPFS mocks.
- Validate chain ID, genesis DAG tracking, and address derivation consistency across wallet, executor, RPC, and GUI.
- Add RPC-level regression tests for `eth_call`, `eth_sendRawTransaction`, mempool snapshot, fee history, and GUI-specific paths.
- Instrument embedded node startup/shutdown in Tauri with metrics and user-facing error surfaces.

## Testing & Verification Approach
- **Unit/Integration**: `cargo test --workspace`; targeted packages (`-p citrate-consensus`, `-p citrate-execution`, `-p citrate-network`).
- **Property/Fuzz**: Proptest/bolero on GhostDAG ordering, mergeset topological sort, and opcode gas metering; fuzz `eth_tx_decoder` RLP/Bincode inputs.
- **Conformance**: Cross-check EVM outputs vs. REVM on fixture vectors; Ethereum JSON tests where applicable.
- **E2E**: Scripted GUI + embedded node flows (headless if possible) plus CLI-based local node to compare outputs.
- **Performance/Security**: DOS scenarios (mempool flood, peer flood), resource exhaustion on AI precompile, IPFS timeout handling.

## Deliverables
- Updated `findings.md` with severity, ownership, and fix suggestions.
- Reproduction steps for each failure path.
- Coverage deltas and proposed tests.
- Risk register for remaining unknowns and follow-up actions.
