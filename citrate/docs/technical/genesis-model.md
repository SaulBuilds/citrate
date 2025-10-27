# Genesis Model: Generation and Embedding

This document explains how the genesis AI model is generated, embedded, and discovered by nodes.

## Overview

- A small ONNX artifact is embedded into the node binary and registered at chain initialization.
- The model is public and meant to provide a baseline for inference and testing.
- For production, ship the real artifact via Git LFS or a release asset, then rebuild nodes.

## Files

- Generator script: `citrate/assets/genesis_model_generator.py`
- Embedded artifact (placeholder): `citrate/assets/genesis_model.onnx`
- Registration logic: `citrate/node/src/genesis.rs`

## Generate an ONNX Model

Requirements:
- Python 3.10+
- `torch`, `transformers`, `onnx`, `onnxruntime`

```bash
python3 -m venv .venv && source .venv/bin/activate
pip install torch transformers onnx onnxruntime numpy
python citrate/assets/genesis_model_generator.py
```

This creates/validates an ONNX file and emits a simple training script for reference.

## Embed the Artifact

The node includes the artifact at compile time via `include_bytes!("../../assets/genesis_model.onnx")` and registers the model during `initialize_genesis_state`.

Steps:
1) Replace the placeholder `citrate/assets/genesis_model.onnx` with your actual small artifact (or store the real artifact using Git LFS).
2) Rebuild the node:

```bash
cargo build -p citrate-node --release
```

3) Start a fresh devnet (to run the genesis logic):

```bash
scripts/lattice.sh reset devnet
scripts/lattice.sh dev up
```

## Discovering the Model

Use JSON‑RPC to list/get the model:

```bash
# List model IDs
curl -s http://localhost:8545 -H 'Content-Type: application/json' \
  -d '{"jsonrpc":"2.0","id":1,"method":"citrate_listModels","params":[]}'

# Get model info
curl -s http://localhost:8545 -H 'Content-Type: application/json' \
  -d '{"jsonrpc":"2.0","id":1,"method":"citrate_getModel","params":["0x<model_id>"]}'
```

## Artifact Management Notes

- For larger artifacts, store weights via IPFS and register the CID when deploying.
- The node supports multiple IPFS providers via `CITRATE_IPFS_PROVIDERS` or a single provider via `CITRATE_IPFS_API`.
- For production, prefer shipping artifacts via releases or LFS and pin in IPFS on startup.

## Security & Size Guidance

- Keep the genesis artifact small (a few MB max) for fast builds and distribution.
- Avoid embedding private/licensed weights without the appropriate license.

## Updating the Genesis Model

- Changing the embedded artifact changes the model hash used as `ModelId`. Nodes bootstrapped with different artifacts will have diverging state.
- When updating, increment metadata version and coordinate across validator/operator builds.

