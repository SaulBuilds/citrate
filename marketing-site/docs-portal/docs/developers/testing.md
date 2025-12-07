---
id: testing
title: Testing
---

Run unit/integration tests and bring up local stacks using the unified script:

```
# Rust workspace tests
cd citrate && cargo test --workspace

# Local dev stack (node + explorer + docs + marketing)
scripts/lattice.sh dev up
scripts/lattice.sh dev status
scripts/lattice.sh dev down

# Docker devnet node
scripts/lattice.sh docker up

# Docker testnet node
scripts/lattice.sh docker testnet up

# 5-node cluster and load generator
scripts/lattice.sh docker cluster up
RATE=20 NODE_RPC=http://localhost:8545 node scripts/loadgen.js
```

SDK tests:
```
cd citrate/sdk/javascript && npm install && npm test
```
