---
title: Run a Node
---

1. Build locally or use Docker.
   - Local: `cd lattice-v3 && cargo build -p lattice-node --release`
   - Docker (devnet): `docker compose --profile devnet up -d` (from `lattice-v3/`) or `scripts/lattice.sh docker up`
2. Configure ports (RPC 8545, P2P 30303). See `lattice-v3/gui/lattice-core/config/devnet.json`.
3. Testnet via Docker: `docker compose --profile testnet up -d` or `scripts/lattice.sh docker testnet up` (exposes RPC on 18545).
4. For local dev stack (node + explorer + docs + marketing): `scripts/lattice.sh dev up`
5. Compose file: `lattice-v3/docker-compose.yml` with profiles:
   - `devnet`: `lattice-node-devnet`
   - `testnet`: `lattice-node-testnet`
   - `explorer`: `explorer-db`, `explorer-web` (port 3000), `explorer-indexer`
   - `monitoring`: `prometheus` (9090), `grafana` (3001)
