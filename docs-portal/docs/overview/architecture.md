---
title: Architecture
---

High-level components:
- Consensus + DAG (GhostDAG)
- Sequencer + Mempool (conflict-aware scheduling)
- Execution (precompiles for models, governance, artifacts; AI VM opcodes)
- Storage (RocksDB)
- Network (P2P gossip, discovery)
- API (JSON-RPC + Lattice methods)

See repository modules under `lattice-v3/core/*` for details.

