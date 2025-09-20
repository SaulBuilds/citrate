---
id: testing
title: Testing
---

Run unit/integration tests and bring up local stacks using the unified script:

```
# Rust workspace tests
cd lattice-v3 && cargo test --workspace

# Local dev stack (node + explorer + docs + marketing)
scripts/lattice.sh dev up
scripts/lattice.sh dev status
scripts/lattice.sh dev down

# Docker devnet node
scripts/lattice.sh docker up

# Docker testnet node
scripts/lattice.sh docker testnet up
```

SDK tests:
```
cd lattice-v3/sdk/javascript && npm install && npm test
```
