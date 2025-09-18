# Lattice DevOps Scripts

Use `scripts/lattice.sh` for building and running the full stack locally and in production.

Examples:

```
# Build Rust workspace, docs, and webapp
scripts/lattice.sh build

# Run dev Docker stack (two nodes, node-app, faucet)
scripts/lattice.sh up:dev

# Build images and run prod stack (node, node-app, faucet, docs, web)
scripts/lattice.sh docker:build
scripts/lattice.sh up:prod
```

Legacy scripts in this folder (`run-devnet.sh`, `test_network.sh`, etc.) are deprecated in favor of `lattice.sh`.

