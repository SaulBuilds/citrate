# Scripts

This folder contains a single orchestrator script for local development and simple Docker bring‑up. Legacy ad‑hoc scripts have been removed in favor of one entrypoint.

## lattice.sh

Usage:
```
scripts/lattice.sh <command> [args]
```

Common commands:
- `setup` — install JS deps across subprojects (idempotent)
- `build` — build Node/CLI (release), Explorer, Docs, GUI web
- `dev up|down|status` — start/stop local dev stack (node, explorer, docs, marketing)
- `testnet up|down` — start/stop native node in testnet mode (placeholder config)
- `mainnet up|down` — placeholder hooks for mainnet
- `docker up|down` — run devnet node via `citrate/docker-compose.yml`
- `docker cluster up|down` — start/stop a 5-node cluster (profile: cluster)
- `logs` — tail logs from `run-logs/`
- `clean` — clean Rust targets and common JS build outputs

Examples:
```
# First time
scripts/lattice.sh setup

# Build core and sites
scripts/lattice.sh build

# Start local dev stack (node + explorer + docs + marketing)
scripts/lattice.sh dev up
scripts/lattice.sh dev status
scripts/lattice.sh dev down

# Run node in Docker (devnet)
scripts/lattice.sh docker up
scripts/lattice.sh docker down

# 5-node cluster (compose profile)
scripts/lattice.sh docker cluster up
scripts/lattice.sh docker cluster down
```

Notes:
- Explorer DB uses a local Postgres container named `lattice-explorer-db`.
- Logs and PIDs are stored under `run-logs/` at the repo root.
- Monitoring stack (Prometheus+Grafana) available via `scripts/lattice.sh docker monitoring up`.
