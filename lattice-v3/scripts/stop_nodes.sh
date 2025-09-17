#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )/.."

kill_pid_file() {
  local f="$1"
  if [ -f "$f" ]; then
    PID=$(cat "$f" 2>/dev/null || echo "")
    if [ -n "${PID}" ]; then
      kill -9 "$PID" 2>/dev/null || true
    fi
    rm -f "$f"
  fi
}

echo "Stopping devnet/multinode processes..."
kill_pid_file "$ROOT_DIR/devnet-node.pid"
kill_pid_file "$ROOT_DIR/mn-node1.pid"
kill_pid_file "$ROOT_DIR/mn-node2.pid"

echo "Killing any lingering lattice processes..."
pkill -f "target/debug/lattice" 2>/dev/null || true
pkill -f "target/release/lattice" 2>/dev/null || true

echo "Done."

