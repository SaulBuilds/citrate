#!/usr/bin/env bash
set -euo pipefail

# Lattice Dev Orchestrator
# Starts core node (Rust), Explorer (Next.js + Postgres), Docs (Docusaurus), Marketing (Next.js).
# Optional: WITH_MONITORING=1 spins up Prometheus+Grafana (targets host metrics at 9100).
#
# Usage:
#   scripts/dev_all.sh up     # start everything
#   scripts/dev_all.sh down   # stop everything and cleanup
#   scripts/dev_all.sh status # show ports and pids
#
# Logs are in run-logs/*.log and PIDs in run-logs/*.pid

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
LOG_DIR="$ROOT_DIR/run-logs"
PID_DIR="$LOG_DIR"
mkdir -p "$LOG_DIR"

WITH_MONITORING="${WITH_MONITORING:-}" # 1/true to enable

info() { echo -e "\033[1;34m[info]\033[0m $*"; }
warn() { echo -e "\033[1;33m[warn]\033[0m $*"; }
err()  { echo -e "\033[1;31m[err ]\033[0m $*"; }

have() { command -v "$1" >/dev/null 2>&1; }

ensure_cmds() {
  local missing=()
  for c in cargo rustc node npm docker; do
    have "$c" || missing+=("$c")
  done
  if ((${#missing[@]})); then
    err "Missing required commands: ${missing[*]}"; exit 1;
  fi
}

compose() {
  if have docker; then
    if docker compose version >/dev/null 2>&1; then
      docker compose "$@"
    else
      docker-compose "$@"
    fi
  else
    err "docker not found"; exit 1
  fi
}

wait_http() {
  local url="$1"; shift
  local timeout="${1:-60}"
  local start=$(date +%s)
  until curl -sSf "$url" >/dev/null 2>&1; do
    sleep 1
    local now=$(date +%s)
    if (( now - start > timeout )); then
      return 1
    fi
  done
  return 0
}

ensure_npm_installed() {
  local dir="$1"
  pushd "$dir" >/dev/null
  if [ ! -d node_modules ]; then
    if [ -f package-lock.json ]; then
      info "npm ci in $dir"
      if ! npm ci; then
        warn "npm ci failed; falling back to npm install"
        npm install
      fi
    else
      info "npm install in $dir"
      npm install
    fi
  fi
  popd >/dev/null
}

start_node() {
  info "Starting Lattice node (devnet)"
  pushd "$ROOT_DIR/lattice-v3" >/dev/null
  mkdir -p "$LOG_DIR"
  RUST_LOG="info,lattice_api=debug" LATTICE_METRICS=1 LATTICE_METRICS_ADDR=0.0.0.0:9100 \
    nohup cargo run -p lattice-node -- devnet >"$LOG_DIR/node.log" 2>&1 &
  echo $! > "$PID_DIR/node.pid"
  popd >/dev/null
  info "Waiting for RPC (8545) and metrics (9100)"
  wait_http "http://localhost:9100/health" 60 || { err "node metrics health check failed"; exit 1; }
  # JSON-RPC: do a simple call (eth_blockNumber)
  curl -sS localhost:8545 -H 'content-type: application/json' \
    -d '{"jsonrpc":"2.0","id":1,"method":"eth_blockNumber","params":[]}' >/dev/null || {
      err "node RPC check failed"; exit 1;
    }
}

start_explorer() {
  info "Starting Postgres for Explorer"
  if ! docker ps --format '{{.Names}}' | grep -q '^lattice-explorer-db$'; then
    docker run -d --name lattice-explorer-db -p 5432:5432 \
      -e POSTGRES_PASSWORD=password -e POSTGRES_DB=lattice_explorer postgres:15-alpine >/dev/null
  fi
  info "Waiting for Postgres (localhost:5432)"
  local start=$(date +%s)
  until docker exec lattice-explorer-db pg_isready -U postgres >/dev/null 2>&1; do
    sleep 1
    local now=$(date +%s)
    if (( now - start > 60 )); then err "Postgres health check failed"; exit 1; fi
  done

  info "Preparing Explorer app"
  ensure_npm_installed "$ROOT_DIR/lattice-v3/explorer"
  pushd "$ROOT_DIR/lattice-v3/explorer" >/dev/null
  export DATABASE_URL="postgresql://postgres:password@localhost:5432/lattice_explorer"
  npx prisma generate >/dev/null
  npx prisma db push >/dev/null
  info "Starting Explorer indexer"
  nohup npm run indexer:dev >"$LOG_DIR/explorer-indexer.log" 2>&1 &
  echo $! > "$PID_DIR/explorer-indexer.pid"
  info "Starting Explorer web (3000)"
  nohup npm run dev >"$LOG_DIR/explorer-web.log" 2>&1 &
  echo $! > "$PID_DIR/explorer-web.pid"
  popd >/dev/null
  wait_http "http://localhost:3000" 90 || warn "Explorer web not ready yet"
}

start_docs() {
  info "Starting Docs Portal (Docusaurus on 3002)"
  ensure_npm_installed "$ROOT_DIR/docs-portal"
  pushd "$ROOT_DIR/docs-portal" >/dev/null
  nohup npm run start -- --port 3002 >"$LOG_DIR/docs.log" 2>&1 &
  echo $! > "$PID_DIR/docs.pid"
  popd >/dev/null
  wait_http "http://localhost:3002" 60 || warn "Docs not ready yet"
}

start_marketing() {
  info "Starting Marketing Site (Next.js on 4000)"
  ensure_npm_installed "$ROOT_DIR/marketing-site"
  pushd "$ROOT_DIR/marketing-site" >/dev/null
  nohup npm run dev >"$LOG_DIR/marketing.log" 2>&1 &
  echo $! > "$PID_DIR/marketing.pid"
  popd >/dev/null
  wait_http "http://localhost:4000" 60 || warn "Marketing site not ready yet"
}

start_monitoring() {
  if [[ "${WITH_MONITORING,,}" =~ ^(1|true|yes|on)$ ]]; then
    info "Starting Prometheus + Grafana"
    pushd "$ROOT_DIR/lattice-v3" >/dev/null
    compose up -d prometheus grafana >/dev/null
    popd >/dev/null
    info "Prometheus at http://localhost:9090, Grafana at http://localhost:3001 (admin/admin)"
  fi
}

stop_pid() {
  local name="$1"; local pid_file="$PID_DIR/$name.pid"
  if [ -f "$pid_file" ]; then
    local pid; pid=$(cat "$pid_file" || true)
    if [ -n "$pid" ] && kill -0 "$pid" >/dev/null 2>&1; then
      info "Stopping $name (pid $pid)"
      kill "$pid" || true
      sleep 1
      if kill -0 "$pid" >/dev/null 2>&1; then kill -9 "$pid" || true; fi
    fi
    rm -f "$pid_file"
  fi
}

down_all() {
  info "Stopping background processes"
  stop_pid node
  stop_pid explorer-indexer
  stop_pid explorer-web
  stop_pid docs
  stop_pid marketing

  if docker ps --format '{{.Names}}' | grep -q '^lattice-explorer-db$'; then
    info "Stopping Postgres container"
    docker stop lattice-explorer-db >/dev/null || true
    docker rm lattice-explorer-db >/dev/null || true
  fi

  if [[ "${WITH_MONITORING,,}" =~ ^(1|true|yes|on)$ ]]; then
    pushd "$ROOT_DIR/lattice-v3" >/dev/null
    compose stop prometheus grafana >/dev/null || true
    compose rm -sfv prometheus grafana >/dev/null || true
    popd >/dev/null
  fi
}

status() {
  info "Service status"
  for n in node explorer-indexer explorer-web docs marketing; do
    if [ -f "$PID_DIR/$n.pid" ]; then
      pid=$(cat "$PID_DIR/$n.pid" 2>/dev/null || true)
      if [ -n "$pid" ] && kill -0 "$pid" 2>/dev/null; then
        echo "  $n: running (pid $pid)"
      else
        echo "  $n: not running"
      fi
    else
      echo "  $n: not started"
    fi
  done
  echo "  Ports: RPC 8545, WS 8546, REST 3000, Metrics 9100, Docs 3002, Marketing 4000"
}

cmd_up() {
  ensure_cmds
  start_node
  start_explorer
  start_docs
  start_marketing
  start_monitoring
  info "All services attempted to start. Check $LOG_DIR for logs."
  status
}

case "${1:-}" in
  up) cmd_up ;;
  down) down_all ;;
  status) status ;;
  *) echo "Usage: $0 {up|down|status}"; exit 1 ;;
esac
