---
title: RPC Methods
---

Ethereum-compatible:
- eth_blockNumber, eth_getBlockByNumber, eth_getTransactionByHash, eth_getTransactionReceipt, eth_call, eth_sendRawTransaction, eth_sendTransaction, eth_estimateGas.

Lattice extensions:
- lattice_deployModel, lattice_requestInference, lattice_getModel, lattice_listModels, lattice_pinArtifact, lattice_getArtifactStatus.
- Verification: lattice_verifyContract, lattice_getVerification, lattice_getVerificationById, lattice_listVerifications, lattice_listVerificationsByStatus, lattice_listVerificationsByAddressPrefix.

See implementations in `lattice-v3/core/api/src/server.rs` and `eth_rpc.rs`.

Notes:
- `lattice_deployModel` now accepts optional `access_policy` ("Public" | "Private" | "Restricted" | "PayPerUse") and `inference_price` (wei, decimal or 0x-hex). When provided, it uses the extended precompile to register policy/pricing alongside the model.
- `lattice_verifyContract` supports `standard_json` (Solc standard JSON input) for multi-file projects, `contract_name` selection, and `constructor_args` (hex-encoded ABI args) to enable full creation bytecode matching. Creation/runtime bytecode hashes are returned with metadata-stripping applied.
- Verification listings support pagination and filtering: `lattice_listVerifications` accepts `{ offset?: u64, limit?: u64, verified?: bool, address_prefix?: string }`. Status/prefix variants accept `{ verified: bool, offset?: u64, limit?: u64 }` and `{ prefix: string, offset?: u64, limit?: u64 }` respectively.
- A GC endpoint `lattice_pruneVerifications` is available with payload `{ max_age_seconds?: u64, max_records?: u64 }` to prune verification records.
