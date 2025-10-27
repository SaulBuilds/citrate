---
title: RPC Methods
---

Ethereum-compatible:
- eth_blockNumber, eth_getBlockByNumber, eth_getTransactionByHash, eth_getTransactionReceipt, eth_call, eth_sendRawTransaction, eth_sendTransaction, eth_estimateGas.

Citrate extensions:
- citrate_deployModel, citrate_runInference, citrate_getModel, citrate_listModels (alias: citrate_getModels), citrate_pinArtifact, citrate_getArtifactStatus.
- Verification: citrate_verifyContract, citrate_getVerification, citrate_getVerificationById, citrate_listVerifications, citrate_listVerificationsByStatus, citrate_listVerificationsByAddressPrefix.

See implementations in `citrate/core/api/src/server.rs` and `eth_rpc.rs`.

Notes:
- `citrate_deployModel` accepts optional `access_policy` ("Public" | "Private" | "Restricted" | "PayPerUse") and `inference_price` (wei, decimal or 0x-hex). When provided, it registers policy/pricing alongside the model.
- `citrate_runInference` performs a synchronous preview inference and returns a structured result: `{ output, encoding, execution_time_ms, gas_used, provider, provider_fee, proof? }`. For async flows, `citrate_requestInference`/`citrate_getInferenceResult` may be provided in future releases.
- `citrate_verifyContract` supports `standard_json` (Solc standard JSON input) for multi-file projects, `contract_name` selection, and `constructor_args` (hex-encoded ABI args) to enable full creation bytecode matching. Creation/runtime bytecode hashes are returned with metadata-stripping applied.
- Verification listings support pagination and filtering: `citrate_listVerifications` accepts `{ offset?: u64, limit?: u64, verified?: bool, address_prefix?: string }`. Status/prefix variants accept `{ verified: bool, offset?: u64, limit?: u64 }` and `{ prefix: string, offset?: u64, limit?: u64 }` respectively.
- A GC endpoint `citrate_pruneVerifications` is available with payload `{ max_age_seconds?: u64, max_records?: u64 }` to prune verification records.
