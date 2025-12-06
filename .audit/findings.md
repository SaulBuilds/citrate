# Current Findings (snapshot)
Date: 2025-12-03  
Focus: GUI embedded node + codegen; consensus/execution risks that block end-to-end flows.

## Critical (blockers for GUI E2E)
- Embedded node RPC still disabled/commented out (`gui/citrate-core/src-tauri/src/node/mod.rs:20,650`); GUI cannot serve external JSON-RPC.
- Consensus efficient sync returns `{processed:0}` with no implementation/tests (`core/consensus/src/sync/efficient_sync.rs:398-404,415-427`), blocking safe catch-up.
- AI network handler returns `Ok(None)` for results (`core/network/src/ai_handler.rs:331-336`), breaking AI propagation/rewards.
- ZK verification accepts empty proofs (`core/mcp/src/verification.rs:150-164`), allowing forged proofs.
- Mock transaction creation with hardcoded addresses (`core/api/src/eth_tx_decoder.rs:665-710`) risks invalid tx injection.

## High Priority
- AI generation/storage mocks (`gui/citrate-core/src-tauri/src/agent/tools/generation.rs:145-171`, `gui/citrate-core/src-tauri/src/agent/tools/storage.rs:148-174`, `gui/citrate-core/src/components/IPFS.tsx:80-152`) prevent real IPFS upload and AI outputs.
- Contract deployment helpers missing receipt polling/gas estimation (`gui/citrate-core/src/utils/contractDeployment.ts:116,147,172`) and metadata fetch (`gui/citrate-core/src/utils/marketplaceHelpers.ts:99,124,132,170`), causing silent failures.
- Training jobs query returns empty vec (`core/api/src/methods/ai.rs:560-575`), blocking AI job visibility.
- Chain ID defaults to 1337 in executor unless configured (`core/execution/src/executor.rs:65,166`); misconfig/replay risk.
- Genesis DAG tracking not initialized (`node/src/genesis.rs:237`), risking incorrect bootstrap.
- Address derivation dual-mode collision risk untested (`core/execution/src/types.rs`).
- LLM fallback/MCP TODOs (`gui/citrate-core/src/agent/llm/mod.rs:140-177,207`; `gui/citrate-core/src/components/ChatBot.tsx:110-113`) leave degraded mode.

## Medium / Observability & Testing Gaps
- GhostDAG ordering/blue-set edge cases lack coverage (`consensus/tests/real_tests.rs`, missing property tests).
- Efficient sync tests empty (`core/consensus/src/sync/efficient_sync.rs:415-427`).
- AI inference timing hardcoded (`core/execution/src/inference/metal_runtime.rs:239`) and model metadata not extracted (`core/execution/src/inference/coreml_bridge.rs:135`).
- AI handler training job owner/model deltas hardcoded (`core/network/src/ai_handler.rs:415,480`).
- Analytics zeros (`core/marketplace/src/analytics_engine.rs:318-334`) and search highlighting missing (`core/marketplace/src/search.rs:216`).
- Speech recognition TODOs (`gui/citrate-core/src/components/ChatBot.tsx:241,244`) limit agent coverage.
- IPFS uploader metadata update TODO (`gui/citrate-core/src/utils/ipfsUploader.ts:288`) and review vote/report TODOs (`gui/citrate-core/src/hooks/useReviews.ts:112-180`) leave moderation flows unverified.

## Recommended Actions (next sprint)
- Unblock RPC and block production first: fix GUI node mempool type, `eth_call`, block builder roots, efficient sync, and chain ID sourcing; add regression tests for each.
- Remove mocks in DAG explorer, fee history, AI handlers, IPFS, and analytics; surface failures instead of zeros to catch issues early.
- Add property/fuzz tests for GhostDAG ordering and `eth_tx_decoder` inputs; add deterministic replay tests comparing native EVM vs. REVM for sample vectors.
- Harden dual-address derivation with collision tests and explicit format tagging in wallet, executor, and RPC responses.
- Instrument embedded node lifecycle in Tauri: startup/shutdown events, RPC health, database errors; expose user-facing error banners to avoid silent failure.
- Extend codegen/deploy flow with receipt polling, revert reason surfacing, and post-deploy contract ABI storage; add UI tests to ensure generated contracts are callable via `eth_call`.
- Validate AI precompile and network message limits: input size caps, timeout budget, and reward accounting paths; add negative tests for malformed payloads.
