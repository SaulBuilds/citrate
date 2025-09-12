#!/bin/bash

# Start Lattice v3 testnet

set -e

echo "Building Lattice node..."
cargo build --release -p lattice-node

echo "Starting Lattice testnet..."
RUST_LOG=info ./target/release/lattice devnet