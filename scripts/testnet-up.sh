#!/usr/bin/env bash
set -euo pipefail

# Lattice v3 Testnet bring-up script
# - Core node (with RPC/WS)
# - Explorer stack (Postgres, Redis, web, indexer)
# - GUI (Tauri)

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
LOG_DIR="${REPO_ROOT}/run-logs"
mkdir -p "${LOG_DIR}"

info() { printf "\033[1;32m[INFO]\033[0m %s\n" "$*"; }
warn() { printf "\033[1;33m[WARN]\033[0m %s\n" "$*"; }
err()  { printf "\033[1;31m[ERR ]\033[0m %s\n" "$*"; }

need() { command -v "$1" >/dev/null 2>&1 || { err "Missing dependency: $1"; exit 1; }; }

info "Checking prerequisites"
need cargo
need rustc
need node
need npm
need docker

# Prefer docker compose v2 (docker compose). Fallback to docker-compose.
if docker compose version >/dev/null 2>&1; then
  DOCKER_COMPOSE=(docker compose)
elif command -v docker-compose >/dev/null 2>&1; then
  DOCKER_COMPOSE=(docker-compose)
else
  err "Docker Compose not found (docker compose or docker-compose)."
  exit 1
fi

NODE_VER=$(node -v | sed 's/v//')
NODE_MAJOR=${NODE_VER%%.*}
if [[ ${NODE_MAJOR} -lt 18 ]]; then
  warn "Node.js >= 18 is recommended. Detected ${NODE_VER}."
fi

# 1) Start Core Node (with RPC)
info "Building and starting the core node (RPC on 127.0.0.1:8545)"
(
  cd "${REPO_ROOT}/lattice-v3"
  # Build release for faster runtime
  cargo build -p lattice-node --release >/dev/null
  nohup cargo run -p lattice-node --release >"${LOG_DIR}/node.log" 2>&1 &
  NODE_PID=$!
  echo ${NODE_PID} > "${LOG_DIR}/node.pid"
)

# Wait for RPC to respond
info "Waiting for RPC to become ready on 127.0.0.1:8545"
for i in {1..60}; do
  if curl -s -X POST http://127.0.0.1:8545 \
      -H 'Content-Type: application/json' \
      --data '{"jsonrpc":"2.0","id":1,"method":"eth_blockNumber","params":[]}' \
      >/dev/null 2>&1; then
    READY=1; break
  fi
  sleep 1
done
if [[ -z "${READY:-}" ]]; then
  err "RPC did not become ready in time. See ${LOG_DIR}/node.log"
  exit 1
fi

# 2) Explorer stack (Postgres, Redis, web, indexer)
EXP_DIR="${REPO_ROOT}/lattice-v3/explorer"
info "Starting Postgres and Redis (Docker)"
(
  cd "${EXP_DIR}"
  # Ensure .env exists
  if [[ ! -f .env ]]; then
    cp .env.example .env
  fi
  "${DOCKER_COMPOSE[@]}" up -d postgres redis

  # Wait for Postgres health (compose healthcheck)
  info "Waiting for Postgres health..."
  for i in {1..60}; do
    if "${DOCKER_COMPOSE[@]}" ps --status running | grep -q lattice-explorer-db; then
      # Additional check: pg_isready via exec
      if "${DOCKER_COMPOSE[@]}" exec -T postgres pg_isready -U postgres >/dev/null 2>&1; then
        break
      fi
    fi
    sleep 1
  done

  info "Applying Prisma migrations"
  npm ci >/dev/null
  npx prisma migrate deploy >/dev/null
  npx prisma generate >/dev/null

  info "Starting explorer (web) and indexer"
  RPC_ENDPOINT="http://host.docker.internal:8545" "${DOCKER_COMPOSE[@]}" up -d explorer indexer
)

# 3) GUI (Tauri)
GUI_DIR="${REPO_ROOT}/lattice-v3/gui/lattice-core"
info "Installing GUI dependencies"
(
  cd "${GUI_DIR}"
  npm ci >/dev/null
)

info "Launching GUI (Tauri) — closing this window will stop the GUI"
(
  cd "${GUI_DIR}"
  # Run in foreground so the user can close it with Ctrl+C or the window close button
  npm run tauri 2>&1 | tee "${LOG_DIR}/gui.log"
)

info "Explorer available at: http://localhost:3000"
info "RPC endpoint:          http://127.0.0.1:8545"
info "Logs in:               ${LOG_DIR}"
info "To stop Explorer:      (cd lattice-v3/explorer && ${DOCKER_COMPOSE[*]} down)"
info "To stop Node:          kill \$(cat ${LOG_DIR}/node.pid) || pkill -f lattice-node"

