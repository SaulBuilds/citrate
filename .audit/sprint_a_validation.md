# Sprint A Validation (claimed fixes)
Date: 2025-12-03

Scope: Verify engineer summary claims against current working tree (`/citrate`).

## Verified in code
- **Chain ID configuration (API)**
  - `core/api/src/server.rs:821-835`: `net_version` uses the configured `chain_id` (no hardcoded 1337).
  - `core/api/src/eth_rpc_simple.rs:158-183`: `eth_chainId` returns formatted hex from `chain_id` param.
  - `core/api/tests/integration_rpc.rs:983-1038`: `test_eth_chain_id_is_configurable` present; exercises chain IDs 42069, 1, 1337.
- **Gas estimation implemented**
  - `core/api/src/eth_rpc.rs:900-1160`: `eth_estimateGas` performs dry-run execution with 10% buffer; nontrivial logic present.
  - `core/api/src/eth_rpc_simple.rs:334-364`: heuristic estimation per tx type (transfer/call/deploy).
  - `core/api/tests/integration_rpc.rs:1039-...`: `test_eth_estimate_gas_real_execution` present and uses real execution.
- **DAG explorer dead code**
  - Files `core/api/src/dag_explorer.rs` and `core/api/src/explorer_server.rs` are absent; `rg` shows no references. (Previously stubbed.)
  - `core/api/src/lib.rs` exports do not include dag_explorer.
- **Metrics module**
  - `node/src/metrics.rs`: uses `metrics` crate macros (`counter!`, `gauge!`, `histogram!`) consistent with 0.21 API; no deprecated `.increment()` usage.

## Not directly validated
- Test execution counts (e.g., “48/48 pass”) not re-run in this session. The new tests exist, but results are unverified here.
- Sync-module failing/long-running tests cited by engineering were not executed; pre-existing status not confirmed.

## Notes
- Execution layer still defaults to `DEFAULT_CHAIN_ID: 1337` in `core/execution/src/executor.rs` (configurable via `CITRATE_CHAIN_ID`), which is outside Sprint A scope.
- Other open TODOs/stubs (AI handler, IPFS mocks, mempool/RPC GUI issues) remain and were not claimed as Sprint A items.
