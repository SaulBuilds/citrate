#!/usr/bin/env bash
set -euo pipefail

# Simple devnet runner for Lattice v3
# - Builds the node
# - Starts a local devnet with mining and RPC enabled
# - Optional flags:
#     --no-sig-verify    Disable signature verification in mempool
#     --chain-id N       Override chain ID (default: 1337)
#     --release          Build and run in release mode

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

RELEASE=""
CHAIN_ID="1337"
REQUIRE_VALID_SIGNATURE="true"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --no-sig-verify)
      REQUIRE_VALID_SIGNATURE="false"; shift ;;
    --chain-id)
      CHAIN_ID="$2"; shift 2 ;;
    --release)
      RELEASE="--release"; shift ;;
    *)
      echo "Unknown option: $1" >&2; exit 1 ;;
  esac
done

if ! command -v cargo >/dev/null 2>&1; then
  echo "cargo not found. Install Rust via: curl https://sh.rustup.rs -sSf | sh" >&2
  exit 1
fi

export RUST_LOG=${RUST_LOG:-lattice=info}
export LATTICE_REQUIRE_VALID_SIGNATURE="$REQUIRE_VALID_SIGNATURE"

echo "Building lattice-node $RELEASE …"
cargo build $RELEASE -p lattice-node

echo "Starting devnet (chain_id=$CHAIN_ID, sig_verify=$REQUIRE_VALID_SIGNATURE)…"
echo "Data dir: .lattice-devnet | RPC: http://127.0.0.1:8545"

# Note: chain_id is applied at genesis for new devnet; on existing data dirs it won't change.
RUST_LOG="$RUST_LOG" \
  LATTICE_REQUIRE_VALID_SIGNATURE="$LATTICE_REQUIRE_VALID_SIGNATURE" \
  cargo run $RELEASE -p lattice-node -- devnet

