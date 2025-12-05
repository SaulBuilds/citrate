# Current Findings (snapshot)
Date: 2025-12-03  
Focus: GUI embedded node + codegen; consensus/execution risks that block end-to-end flows.

## Critical (blockers for GUI E2E)
- GUI RPC disabled due to mempool type mismatch (`gui/citrate-core/src-tauri/src/node/mod.rs` around init). RPC must start with the same mempool type as core to allow the embedded node to accept/serve txs.
- `eth_call` returns an error instead of executing in GUI path (`gui/citrate-core/src-tauri/src/lib.rs` near `eth_call`). Blocks contract reads and codegen verification.
- Block builder returns stubbed state/receipt roots (`core/sequencer/src/block_builder.rs:306-320`), producing invalid blocks and receipts; prevents honest block production and GUI block view accuracy.
- Consensus efficient sync returns `{processed:0}` without processing (`core/consensus/src/sync/efficient_sync.rs:398-404`), so an embedded node cannot catch up or validate peers.
- AI network handler returns `Ok(None)` for results (`core/network/src/ai_handler.rs:331-336`), breaking AI propagation and rewards paths exposed in the GUI.
- ZK verification accepts empty proofs (`core/mcp/src/verification.rs:150-164`), allowing forged proofs to pass and undermining any model verification exposed to users.
- Mock transaction creation with hardcoded addresses (`core/api/src/eth_tx_decoder.rs:665-710`) risks invalid tx injection into the mempool and misleads GUI displays and codegen flows.

## High Priority
- DAG explorer/network stats mocked to zero (`core/api/src/dag_explorer.rs:486-493,697-708`), hiding liveness/safety issues from GUI dashboards.
- AI generation/storage mocks (`gui/citrate-core/src-tauri/src/agent/tools/generation.rs:145-171`, `gui/citrate-core/src-tauri/src/agent/tools/storage.rs:148-174`, `gui/citrate-core/src/components/IPFS.tsx:80-152`) prevent real IPFS upload and AI outputs; users cannot verify artifacts or earn rewards.
- Contract deployment helpers missing receipt polling and metadata fetch (`gui/citrate-core/src/utils/contractDeployment.ts:113-128`, `gui/citrate-core/src/utils/marketplaceHelpers.ts:99,124`), causing silent failures in codegen-to-deploy flow.
- Training jobs query returns empty vec (`core/api/src/methods/ai.rs:560-575`), blocking AI job visibility in GUI.
- `eth_feeHistory` mock data (`core/api/src/eth_rpc.rs:923-930`) can break gas estimation paths used by GUI wallet.
- Chain ID hardcoded to 1337 (`core/execution/src/executor.rs:800`), enabling replay across networks and confusing GUI chain displays; must source from config/genesis.
- Genesis DAG tracking not initialized (`node/src/genesis.rs:237`), risking incorrect DAG bootstrap for embedded node.
- Address derivation dual-mode risks collisions if 32-byte keys with trailing zeros resemble EVM addresses (`core/execution/src/types.rs`); needs collision tests and consistent handling in wallet, executor, and RPC.

## Medium / Observability & Testing Gaps
- GhostDAG ordering/blue-set edge cases lack coverage (`consensus/tests/real_tests.rs`, missing property tests). Determinism must be proven for GUI block explorers.
- Efficient sync tests empty (`core/consensus/src/sync/efficient_sync.rs:415-427`), leaving regressions unguarded.
- AI inference timing hardcoded (`core/execution/src/inference/metal_runtime.rs:239`) and model metadata not extracted (`core/execution/src/inference/coreml_bridge.rs:135`), so GUI performance/reward metrics are unreliable.
- AI handler training job owner/model deltas hardcoded (`core/network/src/ai_handler.rs:415,480`), leading to incorrect reward attribution in GUI.
- Analytics zeros (`core/marketplace/src/analytics_engine.rs:318-334`) and search highlighting missing (`core/marketplace/src/search.rs:216`), reducing visibility into marketplace quality and codegen usage.
- GUI agent falls back to mock LLM (`gui/citrate-core/src/agent/llm/mod.rs:140-177,207`), so generated code may not reflect intended models; enables silent degraded mode.
- Speech recognition TODOs (`gui/citrate-core/src/components/ChatBot.tsx:241,244`) and MCP connection stub (`gui/citrate-core/src/components/ChatBot.tsx:110-113`) limit agent capabilities and testing coverage.
- IPFS uploader metadata update TODO (`gui/citrate-core/src/utils/ipfsUploader.ts:288`) and governance review vote TODOs (`gui/citrate-core/src/hooks/useReviews.ts:112-180`) leave moderation flows unverified.

## Recommended Actions (next sprint)
- Unblock RPC and block production first: fix GUI node mempool type, `eth_call`, block builder roots, efficient sync, and chain ID sourcing; add regression tests for each.
- Remove mocks in DAG explorer, fee history, AI handlers, IPFS, and analytics; surface failures instead of zeros to catch issues early.
- Add property/fuzz tests for GhostDAG ordering and `eth_tx_decoder` inputs; add deterministic replay tests comparing native EVM vs. REVM for sample vectors.
- Harden dual-address derivation with collision tests and explicit format tagging in wallet, executor, and RPC responses.
- Instrument embedded node lifecycle in Tauri: startup/shutdown events, RPC health, database errors; expose user-facing error banners to avoid silent failure.
- Extend codegen/deploy flow with receipt polling, revert reason surfacing, and post-deploy contract ABI storage; add UI tests to ensure generated contracts are callable via `eth_call`.
- Validate AI precompile and network message limits: input size caps, timeout budget, and reward accounting paths; add negative tests for malformed payloads.
