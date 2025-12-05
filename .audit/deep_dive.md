# Deep Dive Inventory (Testnet Readiness)
Date: 2025-12-03

## Core & Node
- Consensus: `core/consensus/src/sync/efficient_sync.rs` still stubbed (`{processed:0}` + unimplemented tests). Tip/ordering tests thin (`consensus/tests/real_tests.rs`); no property/fuzz coverage for mergeset ordering or blue-set determinism.
- Execution: Chain ID fixed to 1337 (`core/execution/src/executor.rs:800`); AI inference timing hardcoded (`inference/metal_runtime.rs:239`); CoreML metadata extraction missing (`coreml_bridge.rs:135`). Dual address derivation needs collision tests and cross-component consistency (wallet/executor/RPC/GUI).
- Tx pipeline: Block builder returns fake roots (`core/sequencer/src/block_builder.rs:306-320`); `eth_tx_decoder` mock tx creation (`core/api/src/eth_tx_decoder.rs:665-710`); mempool DOS limits not documented/enforced.
- Network/P2P: AI handler stubs (`core/network/src/ai_handler.rs:331,415,480`), training deltas not applied; message variant missing; limited rate-limit/evasion tests.
- Storage/Genesis: Genesis DAG tracking TODO (`node/src/genesis.rs:237`); model verifier missing validator list (`node/src/model_verifier.rs:75`); genesis embedding mock (`core/genesis/genesis_model.rs:114-129`).
- API/RPC: DAG explorer stats/tx details stubbed (`core/api/src/dag_explorer.rs:486-493,697-708,550`); `eth_feeHistory` mock (`core/api/src/eth_rpc.rs:923-930`); AI methods return empty (`core/api/src/methods/ai.rs:560-575`).

## GUI (Tauri Backend + React Frontend)
- Embedded node/RPC: RPC server commented out due to mempool type mismatch (`gui/citrate-core/src-tauri/src/node/mod.rs` + Cargo.toml); `eth_call` returns error (`src-tauri/src/lib.rs`); block production relies on sequencer roots stub; sync stub prevents catch-up.
- Agent tools (Tauri): Image generation returns mock on failure (`src-tauri/src/agent/tools/generation.rs:145-171`); IPFS upload generates mock CID (`src-tauri/src/agent/tools/storage.rs:148-174`), using fallback hash-based CID; MCP/LLM fallback to mock model (`gui/citrate-core/src/agent/llm/mod.rs:140-177,207`); ChatBot MCP connection TODO (`gui/citrate-core/src/components/ChatBot.tsx:110-113`) and speech recognition TODOs (241,244).
- Wallet/Signing: `gui/citrate-core/src/services/tauri.ts` falls back to random mock signature on failure; signature verification simplified (length-only), unsafe for production.
- Contract tooling: Compiler is placeholder/mock (`gui/citrate-core/src/utils/contractCompiler.ts`); deployment helpers lack receipt polling/gas estimation (`contractDeployment.ts:113-128,147,172`); marketplace helpers use mock IPFS/metadata (`marketplaceHelpers.ts:99,124,132,170`); search index totalInferences stub (`searchIndexBuilder.ts:252`); marketplace initialization falls back to mock data when contracts absent (`components/Marketplace.tsx`).
- IPFS/frontend: IPFS component fully mocked (`components/IPFS.tsx:80-227`); IPFS uploader metadata update TODO (`utils/metadata/ipfsUploader.ts:288`); performance benchmarks and examples rely on mock data (`components/Marketplace.tsx`, `utils/testing/performanceBenchmarks.ts`, `examples/MetricsDemo.tsx`).
- Reviews/governance: Review voting/reporting unimplemented (`hooks/useReviews.ts:112-180`).

## SDKs & CLI
- JS SDKs (`sdk/javascript`, `sdks/javascript/citrate-js`): No TODO markers, but integration tests against live RPC are absent; ensure chain ID, dual-address handling, and fee-market types are exercised. Dist bundles present but not validated for current API surface.
- Python SDK (`sdks/python`): Lacks visible TODOs; needs RPC integration tests (eth_call/sendRawTx/feeHistory), address derivation parity, and CI coverage. Examples not wired to embedded node.
- CLI/wallet: No explicit TODOs, but should verify chain ID configurability, address derivation consistency, and signing paths for both ed25519/k256 before public testnet.

## Dev-Only/Bloat to Gate or Remove
- Mock compilation and random signatures (contractCompiler, tauri.ts) must be gated behind explicit dev flag or removed before release.
- IPFS mock uploads and generated CIDs (agent tools and React component) should fail loudly when no IPFS connectivity, not pretend success.
- Marketplace “no mocks” docs contradict current mock-heavy implementation (`gui/citrate-core/docs/marketplace/README.md` vs. components using mock data); update docs or implement real flows.
- Metrics/performance demos and mock marketplace data should be clearly labeled dev-only and excluded from prod builds.

## Testnet Release Gates (delta from roadmap)
- Core: enable real block roots, configurable chain ID, efficient sync, DAG tracking, and AI handler message wiring; add property/fuzz tests for GhostDAG ordering and tx decoding.
- RPC/GUI: resolve mempool type mismatch so RPC server starts; fix `eth_call`; add receipt polling/gas estimation; remove signature mocks; ensure IPFS/AI tools error when offline.
- Agent/Codegen: integrate real compiler (solc-js or Foundry via Tauri); real IPFS pinning; deterministic LLM selection with audit logs; MCP connectivity tested.
- SDKs: add live RPC integration tests (eth_call/sendRawTx/feeHistory, dual addresses, EIP-1559/2930), publish with verified chain IDs and network configs.
- Observability: add metrics/logging for embedded-node lifecycle, RPC errors, block production, AI jobs, and IPFS operations; surface user-facing errors in GUI.
