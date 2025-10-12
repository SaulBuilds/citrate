#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
ROOT="$(dirname "$SCRIPT_DIR")"

# Start two nodes fresh
bash "$ROOT/scripts/start_multinode.sh"

# Point E2E to our two-node cluster
export BOOTSTRAP_NODE=${BOOTSTRAP_NODE:-http://127.0.0.1:8545}
export VALIDATOR_1=${VALIDATOR_1:-http://127.0.0.1:8547}
export VALIDATOR_2=${VALIDATOR_2:-http://127.0.0.1:8545}
export FULL_NODE=${FULL_NODE:-http://127.0.0.1:8547}
export ARCHIVE_NODE=${ARCHIVE_NODE:-http://127.0.0.1:8545}
export RESULTS_DIR=${RESULTS_DIR:-$ROOT/scripts/test-results}

bash "$ROOT/scripts/run_e2e_tests.sh"

