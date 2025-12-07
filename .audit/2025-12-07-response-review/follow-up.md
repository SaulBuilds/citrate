# Follow-Up on Final Audit Response (Sprint G)
Date: 2025-12-07
Context: Clarifies residual gaps after validating `.audit/2025-12-07-final-response.md`.

## What’s Still Outstanding
- **Receipt root is synthetic:** `core/sequencer/src/block_builder.rs` still hashes `tx.hash + gas_limit + success=1` (no real receipts). Divergence from executed receipts could break verification/consensus.
- **State root fallback is weak:** Without executor, fallback hashes tx fields only (no account/storage commitment). Light clients cannot verify state in this path.
- **Fee history is heuristic:** `eth_feeHistory` now reads receipts when present but still computes base fee from block fullness with a fixed 15M limit; no persisted base_fee per block. Clients expecting EIP-1559 accuracy may be misled.
- **Analytics depend on in-memory windows:** New fail-loud behavior is good, but metrics come from in-process performance windows; real ingestion/persistence remains unspecified.
- **Validator config operational risk:** Empty validator set now logs a warning, but still runs without enforcement unless operators configure validators.

## Recommended Actions
1) Build receipt-root commitment over stored receipts (or Merkle) and make it consensus-critical; fail if receipts missing.
2) For block building without executor, derive state root from committed state or refuse to build; avoid tx-hash-only fallback.
3) Persist base_fee and gas_used per block; surface accurate `eth_feeHistory` (and block gas limits if variable).
4) Document and implement validator-set provisioning (config or on-chain registry); consider fail-closed when empty in production mode.
5) Define the analytics data pipeline (collection, persistence, retention) or explicitly return “analytics unavailable” until live data exists.

## Files to Revisit
- `core/sequencer/src/block_builder.rs` (state_root/receipt_root fallbacks)
- `core/api/src/eth_rpc.rs` (`eth_feeHistory` fullness heuristics)
- `node/src/model_verifier.rs` (validator enforcement defaults)
- `core/marketplace/src/analytics_engine.rs`, `core/marketplace/src/performance_tracker.rs` (data source clarity)
