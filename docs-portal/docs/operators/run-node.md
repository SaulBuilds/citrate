---
title: Run a Node
---

1. Build locally or use Docker.
   - Local: `cd citrate && cargo build -p citrate-node --release`
   - Docker (devnet): `docker compose --profile devnet up -d` (from `citrate/`) or `scripts/lattice.sh docker up`
2. Configure ports (RPC 8545, P2P 30303). See `citrate/gui/citrate-core/config/devnet.json`.
3. Testnet via Docker: `docker compose --profile testnet up -d` or `scripts/lattice.sh docker testnet up` (exposes RPC on 18545).
4. For local dev stack (node + explorer + docs + marketing): `scripts/lattice.sh dev up`
5. Compose file: `citrate/docker-compose.yml` with profiles:
   - `devnet`: `citrate-node-devnet`
   - `testnet`: `citrate-node-testnet`
   - `explorer`: `explorer-db`, `explorer-web` (port 3000), `explorer-indexer`
   - `monitoring`: `prometheus` (9090), `grafana` (3001)
