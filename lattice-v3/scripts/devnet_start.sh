#!/usr/bin/env bash
set -euo pipefail

# Start a single-node devnet with relaxed signature checks for testing
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

cd "$PROJECT_ROOT"

if [ ! -f target/debug/lattice ]; then
  echo "Building lattice node (debug)..."
  cargo build -p lattice-node
fi

echo "Starting devnet node on http://127.0.0.1:8545 (P2P 127.0.0.1:30303)"
LATTICE_REQUIRE_VALID_SIGNATURE=0 \
RUST_LOG=${RUST_LOG:-info} \
./target/debug/lattice devnet > "$PROJECT_ROOT/devnet-node.log" 2>&1 &
echo $! > "$PROJECT_ROOT/devnet-node.pid"

echo "Devnet PID: $(cat "$PROJECT_ROOT/devnet-node.pid")"
echo "Logs: $PROJECT_ROOT/devnet-node.log"

