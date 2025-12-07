# Audit Validation – 2025-12-07
Scope: validate the “Complete Audit Sprint Report (A-F)” claims against current code across core/gui/sdks plus notes from .audit and .sprint. Static review only (no tests run here).

## Summary
- Confirmed many claimed fixes (chain ID propagation, pending nonce, signature/LLM/IPFS fail-loud, GUI password/export key, RPC server wiring, inference response proofing, solc/compiler helpers).
- Found critical mismatches: block roots are still placeholder/nondeterministic; analytics and training ownership remain stubbed; validator set handling is empty; sdk integration coverage is thin. Account deletion lives in `Wallet` not `Settings`.
- Several “next audit pass” items remain unresolved (analytics zeros, speech recognition TODOs, CoreML metadata, hardcoded inference timing, etc.).

## Checks by Sprint

### Sprint A (Core Infrastructure)
- Chain ID configurable: `core/api/src/server.rs:826-833`, `core/api/src/eth_rpc_simple.rs:161-166` use the configured `chain_id`; tests present.
- Gas estimation: `core/api/src/eth_rpc.rs:918-1179` dry-runs execute and returns gas_used +10% buffer; heuristics in `eth_rpc_simple` present. (Behavior still basic but matches claim.)
- DAG explorer removal: no `dag_explorer` references in `core/api`; files absent.
- Metrics: `node/src/metrics.rs` uses `metrics` 0.21 macros (`counter!`, `gauge!`, `histogram!`).

### Sprint B (Transaction Pipeline)
- Tx decoder: `core/api/src/eth_tx_decoder.rs` implements legacy + EIP-2930/1559 parsing and removes mock construction; no mock fallbacks found.
- Pending nonce: `core/api/src/eth_rpc.rs:518-556` scans mempool for sender when tag=="pending".
- Block roots (issue): `core/sequencer/src/block_builder.rs:295-361` still uses placeholder hashing. `calculate_state_root` hashes tx fields plus `SystemTime::now()` (non-deterministic, no real state), and `calculate_receipt_root` assumes success and uses `gas_limit` as gas used with no receipts. This does not satisfy “real roots” and breaks determinism/verification.

### Sprint C (GUI Security Hardening)
- Signature mocks removed: `gui/citrate-core/src/services/tauri.ts:329-368` hard-fail on bad signatures; no random fallback.
- IPFS/image/LLM fail-loud: `src-tauri/src/agent/tools/storage.rs:148-176`, `generation.rs:145-165`, `agent/llm/mod.rs` uses `UnconfiguredLLMBackend` errors and `MockLLMBackend` under `cfg(test)`.
- Hardcoded password removed: `gui/citrate-core/src/components/FirstTimeSetup.tsx` uses user input/validation only.

### Sprint D (GUI Polish & Visualization)
- DAG visualization present using `react-force-graph-2d`: `gui/citrate-core/src/components/DAGVisualization.tsx`.
- Download weights button calls real IPFS path: `gui/citrate-core/src/components/Models.tsx:15-75` uses `ipfsService.get` then gateway fallback.
- Account deletion: implemented in `gui/citrate-core/src/components/Wallet.tsx:279-321` (confirmation + `walletService.deleteAccount`). Not present in `Settings.tsx` as the report states.

### Sprint E (Wallet & Account Security)
- ChatBot MCP flow wired: `gui/citrate-core/src/components/ChatBot.tsx` creates sessions via `agent_*` commands with direct inference fallback; errors surface to UI.
- Export private key modal: `gui/citrate-core/src/components/Wallet.tsx:1618-1710` requires password, warns, hides key by default.

### Sprint F (Auditor Recommendations)
- Embedded RPC server in GUI node: `gui/citrate-core/src-tauri/src/node/mod.rs:644-700` starts `RpcServer` with `RpcHandles`, graceful shutdown at `:716-744`, default enable via `default_enable_rpc()` and `NodeConfig.enable_rpc`.
- Efficient sync: `node/src/sync/efficient_sync.rs` contains full iterative implementation and tests.
- AI inference response: `core/network/src/ai_handler.rs:320-359` returns `NetworkMessage::InferenceResponse` with commitment-based proof; `sha3` dep present in `core/network/Cargo.toml`.
- ZK verification rejects empty/short proofs: `core/mcp/src/verification.rs:163-178`.
- Dual-address tests exist: `core/execution/src/types.rs` includes the reported tests; chain ID passed into executor in `src-tauri/src/node/mod.rs:137-151`.
- Frontend deploy helpers use real solc + receipt polling: `gui/citrate-core/src/utils/contractCompiler.ts` imports `solc`; `contractDeployment.ts:114-188` polls `getTransactionReceipt`, `:194-228` estimates gas with buffer.

## Discrepancies / Open Risks
- **Block roots still placeholder (critical)**: `core/sequencer/src/block_builder.rs:295-361` uses concatenated hashes plus `SystemTime::now()` for state root and assumes success for receipts. Not a real state/receipt Merkle root; non-deterministic and unverifiable.
- **Training job owner/deltas stubbed**: `core/network/src/ai_handler.rs:436-445` sets `owner` to zero address and does not track participants/weight deltas from messages.
- **Analytics return zeros**: `core/marketplace/src/analytics_engine.rs:316-339` returns placeholder metrics (DAU, retention, market share) with zeros, matching “Additional Concerns” unresolved.
- **Validator list not populated**: `node/src/model_verifier.rs:60-82` uses an in-memory static provider defaulting to empty; no retrieval from config/network, so verification gates cannot enforce any real validator set.
- **SDK integration coverage thin**: `sdks/javascript/citrate-js/tests/integration/client.test.ts` and `sdks/python/tests/test_integration.py` cover chainId/balance/nonce only; no live coverage for `eth_call`, `sendRawTransaction` (typed tx), `feeHistory`, dual-address behavior, or EIP-1559/2930 paths as flagged in .audit concerns.
- **Account deletion location mismatch**: implemented in `Wallet.tsx:279-321` with a simple confirm dialog; no flow in `Settings.tsx` despite claim.
- **Eth fee history and gas data are synthetic**: `core/api/src/eth_rpc.rs:1160-1179` fabricates base fees and uses `gas_limit` minima rather than execution receipts; may mislead clients relying on fee history accuracy.
- **Remaining TODOs noted in code**: speech recognition polish in `ChatBot.tsx:241-270`, hardcoded inference timing in `core/execution/src/inference/metal_runtime.rs:236-242`, CoreML metadata extraction TODO in `core/execution/src/inference/coreml_bridge.rs:129-139`. These were listed for “next audit pass” and remain unresolved.
- **Default chain_id fallback**: `gui/citrate-core/src/components/Settings.tsx` injects mempool defaults (including `chainId: 1337`) when config lacks mempool block, so hardcoded 1337 can reappear via UI defaults.

## Recommendations
1) Replace block root computations with deterministic, state-backed hashing (remove timestamp, derive state/receipt commitments from actual executor state and receipts).
2) Capture training job owner/participant data from network messages and persist deltas; avoid zero-owner placeholder.
3) Implement real analytics data sources or guard the API so callers see “unavailable” instead of zeros.
4) Wire validator lists to configuration or on-chain registry; fail closed when empty.
5) Expand SDK integration tests to cover typed transactions, `eth_call`, `sendRawTransaction`, `feeHistory`, dual-address behaviors on both JS and Python SDKs.
6) Align UI copy/reporting: move account deletion controls into Settings or update documentation to point to Wallet; add stronger safeguards/backups.
7) Address remaining TODOs (speech recognition UX, CoreML metadata, inference timing) and reconsider feeHistory fabrication or clearly mark as stub in RPC responses.

