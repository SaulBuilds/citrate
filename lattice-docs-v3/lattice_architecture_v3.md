# Lattice Architecture (v3, 2025-09-06)

## Topology
- **Consensus plane**: GhostDAG engine (selected‑parent, mergeset, blue set), VRF leader election, finality gadget.
- **Execution plane**: EVM‑compatible LVM, precompiles for MCP/LoRA/Eval, async job receipts.
- **Data plane**: StorageRegistry providers (IPFS/Arweave/S3), content addressing, erasure‑coded redundancy.
- **Control plane**: Telemetry, metrics, tracing, and audit logs; DAG explorer with mergeset visualization.

## Service Map
```
lattice/
├─ consensus/         # GhostDAG engine, tip selection, finality
├─ sequencer/         # Mempool policy, bundling, parent selection
├─ execution/         # LVM (EVM-compatible) + precompiles
├─ primitives/        # ModelRegistry, LoRAFactory, InferenceRouter, StorageRegistry, ComputeMarket, Trainer, Eval
├─ bridge/            # ZK light clients, proof verifier
├─ storage/           # State DB (MPT), block store, artifact pinning
├─ api/               # JSON-RPC, REST; OpenAI/Anthropic-compatible adapters
├─ observability/     # Logs, metrics, tracing, DAG visualizer
└─ sdk/               # TS/Python/Rust SDKs (MCP native)
```

## Block Lifecycle
1. **Ingest**: Sequencer accepts transactions (classed by type).  
2. **Assemble**: Choose selected parent, add merge parents, snapshot mempool.  
3. **Execute**: LVM executes txs; state transitions; artifact roots computed.  
4. **Commit**: Block persisted; gossip; finality checkpoints emitted.  
5. **Index**: Explorer updates DAG, mergeset, and MCP job views.

## API Surface
- **JSON‑RPC/EVM**: `eth_*` compatible.  
- **MCP REST**: `/v1/models`, `/v1/chat/completions`, `/v1/embeddings`, `/v1/jobs` (OpenAI‑like); `/v1/messages` (Anthropic‑like).

## Security Hardening
- Gas & resource limits for model ops; sandboxed precompiles; attestation & challenge flows.
