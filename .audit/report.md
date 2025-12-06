# Comprehensive Audit Report (Refreshed)
Date: 2025-12-03  
Focus: Embedded GUI node, code generation workflow, and core protocol readiness for public testnet. Updated after Sprint A fixes (chain ID, gas estimation, DAG explorer removal, metrics, Tauri signing/eth_call improvements).

## Scope & Method
- Reviewed repository structure, audit guide, sprint notes, and code for TODO/FIXME/unimplemented/mocks across core, GUI, SDKs.
- Emphasis on correctness, security, determinism, and user-facing E2E flows (codegen → compile → deploy → interact → earn via blocks/AI).
- No code changes performed; documentation only.

## Severity Key
- **Critical**: Blocks safe testnet or permits consensus/state compromise.
- **High**: Major feature broken or exploitable; must fix before public testnet.
- **Medium**: Functional gaps, observability gaps, or degradation risk.
- **Low**: Non-blocking polish or clarity issues.

## Findings by Area

### 1) Consensus & Sync
- **Critical**: Efficient sync still stubbed (`core/consensus/src/sync/efficient_sync.rs:398-404,415-427`); no processing/tests. Liveness/catch-up remains unproven.
- **High**: Mergeset/blue-set determinism lacks property/fuzz coverage (`core/consensus/tests/real_tests.rs` minimal). Divergence risk under adversarial DAGs.
- **Medium**: Tip selection/ordering tests use panic-based assertions (`core/consensus/tests/real_tests.rs:106,299`); limited adversarial scenarios.

### 2) Execution / EVM
- **High**: Chain ID default remains 1337 in executor (`core/execution/src/executor.rs:65,166`) unless `CITRATE_CHAIN_ID` set. Risk of replay/mismatch if not configured consistently.
- **High**: Dual address derivation lacks collision tests and cross-component parity (`core/execution/src/types.rs`). Ambiguity risk across wallet/executor/RPC/GUI.
- **High**: AI precompile/resource accounting unverified; inference timing stub (`core/execution/src/inference/metal_runtime.rs:239`) and CoreML metadata extraction missing (`core/execution/src/inference/coreml_bridge.rs:135`). Potential DOS or mispriced gas.
- **Medium**: Opcode/gas correctness lacks cross-conformance vs. REVM. Tensor ZKP test still panics on invalid opcode classification (`core/execution/tests/tensor_zkp_test.rs:64`).

### 3) Transaction Pipeline & Block Production
- **High**: Block builder now computes deterministic roots (no stub constants), but uses simplified hashing rather than state trie; needs validation vs. execution/state DB for correctness (`core/sequencer/src/block_builder.rs`).
- **High**: Mock transaction creation with hardcoded addresses persists (`core/api/src/eth_tx_decoder.rs:665-710`).
- **High**: Mempool DOS bounds/eviction still undocumented; embedded RPC remains disabled in node manager (see GUI/Tauri).  
- **Medium**: Nonce/replay handling across dual signature schemes still lightly tested.

### 4) Network / P2P
- **Critical**: AI handler still returns `Ok(None)` for inference results due to missing message variant (`core/network/src/ai_handler.rs:331`); breaks AI propagation/rewards.
- **High**: Training job owner hardcoded; model deltas not applied (`core/network/src/ai_handler.rs:415,480`). Incorrect attribution/state.
- **Medium**: Limited rate-limit/peer-diversity controls; panic-based assertions remain (`core/network/src/sync.rs:465`).

### 5) Storage & Genesis
- **High**: Genesis DAG tracking TODO remains (`node/src/genesis.rs:237`); bootstrap consistency risk.
- **Medium**: Model verifier still lacks validator list retrieval (`node/src/model_verifier.rs:75`).
- **Medium**: Genesis embedding still mock-based (`core/genesis/genesis_model.rs:114-129`).
- **Medium**: Restart/replay integrity in embedded-node mode still undocumented.

### 6) RPC / API
- **Resolved**: DAG explorer dead code removed; production RPC lives in `server.rs`/`eth_rpc.rs`.
- **Resolved**: Chain ID now configurable in API (`server.rs`, `eth_rpc_simple.rs`); tests added.
- **Resolved**: Gas estimation implemented (`eth_rpc.rs:900-1160`, `eth_rpc_simple.rs:334-364`).
- **High**: AI training jobs still return empty (`core/api/src/methods/ai.rs:560-575`).
- **Medium**: Decoder integration test uses mock data (expected), websocket tests still panic-based (`core/api/src/websocket.rs:433`). Fee history now implemented (no mock observed).

### 7) GUI — Tauri Backend (Embedded Node)
- **Critical**: RPC server still disabled/commented out (`gui/citrate-core/src-tauri/src/node/mod.rs:20,650`); no external JSON-RPC from embedded node despite mempool sync fix claim.
- **Resolved**: `eth_call` now implemented (`gui/citrate-core/src-tauri/src/lib.rs:783-842`).
- **Resolved**: Wallet signing no longer falls back to mock; errors propagate (`gui/citrate-core/src/services/tauri.ts:329-364`).
- **High**: Agent tools still return mock/simulated outputs: image gen fallback (`src-tauri/src/agent/tools/generation.rs:145-171`), IPFS upload mock CID (`src-tauri/src/agent/tools/storage.rs:148-174`).
- **High**: LLM fallback to mock and MCP connection TODO persist (`gui/citrate-core/src/agent/llm/mod.rs:140-177,207`; `gui/citrate-core/src/components/ChatBot.tsx:110-113`).
- **Medium**: Speech recognition TODOs (`ChatBot.tsx:241,244`); limited observability surfaced to users.

### 8) GUI — React Frontend
- **High**: IPFS component remains mocked (`gui/citrate-core/src/components/IPFS.tsx:80-227`).
- **High**: Contract compiler still placeholder (`gui/citrate-core/src/utils/contractCompiler.ts`); deployment helpers lack receipt polling/gas estimation (`utils/contractDeployment.ts:116,147,172`).
- **High**: Marketplace/search fall back to mock data when contracts absent; inference count stub (`utils/search/searchIndexBuilder.ts:252`); metadata fetching mocked (`utils/marketplaceHelpers.ts:99,124,132,170`).
- **Medium**: Review voting/reporting TODOs (`gui/citrate-core/src/hooks/useReviews.ts:112-180`); performance demos use mock data (`utils/testing/performanceBenchmarks.ts`, `examples/MetricsDemo.tsx`).
- **Medium**: DAG fallback in `services/tauri.ts` may mask errors.

### 9) Agent / Codegen & IPFS
- **High**: IPFS tools still simulate success; no real pinning integration (`src-tauri/src/agent/tools/storage.rs:148-174`).
- **High**: LLM fallback persists; MCP connection TODO; degraded mode not clearly surfaced.
- **Medium**: Tool provenance/audit logging not evident.

### 10) SDKs (JavaScript/Python) & CLI
- **High**: Integration tests vs. live RPC still absent for eth_call/sendRawTx/feeHistory/dual addresses/EIP-1559/2930.
- **Medium**: Chain ID/dual-address parity with node not demonstrated; embedded-node mode not exercised.

### 11) Smart Contracts
- **High**: No updated audit evidence for `contracts/src/`; need reentrancy/access-control/economics review and alignment with GUI codegen defaults.
- **Medium**: Gas/behavior conformance vs. execution layer/GUI flows unverified.

### 12) Observability & Operations
- **High**: Embedded node still lacks user-facing health/metrics (RPC availability, block production, mempool, AI, IPFS). Mocks hide failures.
- **Medium**: Crash-recovery/data-dir permission handling in Tauri embedded mode still undocumented.

## Recommended Remediations (Prioritized)
1) **Unblock embedded node + sync**: Implement efficient sync; initialize genesis DAG tracking; enable RPC server in Tauri node (now that mempool sync issue is addressed); validate block roots against state DB.
2) **Execution correctness**: Enforce configured chain ID end-to-end; add dual-address collision tests; add AI precompile limits/metadata extraction; cross-test gas/opcode behavior vs. REVM.
3) **Remove mocks**: IPFS (agent + React), AI image gen fallback, LLM mock, eth_tx_decoder mock txs. Fail loudly on missing deps instead of simulating success.
4) **Complete codegen pipeline**: Integrate real compiler (solc-js/Foundry via Tauri); implement receipt polling/gas estimation in frontend helpers; persist ABI/address; add UI regressions for `eth_call`/send flows.
5) **Network/AI correctness**: Add AI message variant; populate owners; apply weight deltas; add rate-limit/validation tests.
6) **SDK & tests**: Add live RPC integration tests (eth_call/sendRawTx/feeHistory/dual addresses/EIP-1559/2930); verify chain ID parity; exercise embedded-node mode.
7) **Observability**: Surface embedded-node health/metrics; expose errors instead of silent mocks; add crash-recovery/data-dir checks.
8) **Contracts audit**: Re-review on-chain contracts; add Foundry tests aligned with GUI flows.

## Coverage & Testing Gaps (Summary)
- Consensus property/fuzz tests absent for mergeset/blue-set determinism.
- Execution vs. REVM conformance not documented; AI precompile bounds untested.
- RPC regression tests missing for eth_call/eth_sendRawTransaction/feeHistory/mempool snapshot.
- SDKs lack live integration suites; GUI lacks end-to-end tests for codegen → deploy → interact.
- Network rate-limit/Sybil/evasion tests not present; AI message paths untested.

## Risk Register (Top Blockers for Public Testnet)
- Embedded RPC still disabled; embedded node cannot serve external JSON-RPC.
- Efficient sync unimplemented; risk of nodes failing to catch up.
- AI handler message/owner/delta stubs; AI rewards/state unreliable.
- IPFS/LLM/image-gen mocks; user-facing actions may silently simulate success.
- Chain ID must be configured everywhere; default 1337 in executor risks replay if misconfigured.
