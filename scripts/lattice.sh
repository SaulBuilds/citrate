#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
LOG_DIR="$ROOT_DIR/run-logs"
mkdir -p "$LOG_DIR"

have() { command -v "$1" >/dev/null 2>&1; }
need() { have "$1" || { echo "Missing required command: $1" >&2; exit 1; }; }

compose() {
  if have docker; then
    if docker compose version >/dev/null 2>&1; then
      docker compose "$@"
    else
      docker-compose "$@"
    fi
  else
    echo "docker not found" >&2; exit 1
  fi
}

usage() {
  cat <<EOF
Lattice Orchestrator

Usage: scripts/lattice.sh <command> [args]

Common commands:
  setup                          Install JS deps where needed
  build                          Build Node/CLI (release), GUI web, Explorer, Docs
  dev up|down|status             Start/stop dev stack (node, explorer, docs, marketing)
  testnet up|down                Start/stop node in testnet mode (native)
  mainnet up|down                Placeholder (to be released)
  docker up|down                 Run devnet node via docker compose (lattice-v3/docker-compose.yml)
  logs                           Tail logs in run-logs/
  clean                          Clean Rust targets and common caches

Examples:
  scripts/lattice.sh setup
  scripts/lattice.sh build
  scripts/lattice.sh dev up
  scripts/lattice.sh docker up
EOF
}

ensure_npm_installed() {
  local dir="$1"
  pushd "$dir" >/dev/null
  if [ ! -d node_modules ]; then
    if [ -f package-lock.json ]; then
      npm ci || npm install
    else
      npm install
    fi
  fi
  popd >/dev/null
}

start_node_devnet() {
  need cargo
  (cd "$ROOT_DIR/lattice-v3" && \
    RUST_LOG="info,lattice_api=debug" \
    nohup cargo run -p lattice-node -- devnet >"$LOG_DIR/node.log" 2>&1 & echo $! > "$LOG_DIR/node.pid")
}

start_node_testnet() {
  need cargo
  (cd "$ROOT_DIR/lattice-v3" && \
    RUST_LOG="info" nohup cargo run -p lattice-node --release >"$LOG_DIR/node.log" 2>&1 & echo $! > "$LOG_DIR/node.pid")
}

stop_pid() {
  local name="$1"; local pid_file="$LOG_DIR/$name.pid"
  if [ -f "$pid_file" ]; then
    local pid; pid=$(cat "$pid_file" || true)
    if [ -n "$pid" ] && kill -0 "$pid" >/dev/null 2>&1; then
      kill "$pid" || true; sleep 1; kill -9 "$pid" || true
    fi
    rm -f "$pid_file"
  fi
}

start_explorer() {
  need node; need npm; need docker
  # Start Postgres via docker-compose
  (cd "$ROOT_DIR/lattice-v3" && compose up -d explorer-db)
  # Prepare and start the Explorer processes locally
  ensure_npm_installed "$ROOT_DIR/lattice-v3/explorer"
  (
    cd "$ROOT_DIR/lattice-v3/explorer"
    export DATABASE_URL="postgresql://postgres:password@localhost:5432/lattice_explorer"
    npx prisma generate >/dev/null || true
    npx prisma db push >/dev/null || true
    nohup npm run indexer:dev >"$LOG_DIR/explorer-indexer.log" 2>&1 & echo $! > "$LOG_DIR/explorer-indexer.pid"
    nohup npm run dev >"$LOG_DIR/explorer-web.log" 2>&1 & echo $! > "$LOG_DIR/explorer-web.pid"
  )
}

start_docs() {
  ensure_npm_installed "$ROOT_DIR/docs-portal"
  (cd "$ROOT_DIR/docs-portal" && nohup npm run start -- --port 3002 >"$LOG_DIR/docs.log" 2>&1 & echo $! > "$LOG_DIR/docs.pid")
}

start_marketing() {
  ensure_npm_installed "$ROOT_DIR/marketing-site"
  (cd "$ROOT_DIR/marketing-site" && nohup npm run dev >"$LOG_DIR/marketing.log" 2>&1 & echo $! > "$LOG_DIR/marketing.pid")
}

cmd_setup() {
  need node; need npm
  ensure_npm_installed "$ROOT_DIR/lattice-v3/explorer"
  ensure_npm_installed "$ROOT_DIR/docs-portal"
  ensure_npm_installed "$ROOT_DIR/marketing-site"
  ensure_npm_installed "$ROOT_DIR/lattice-v3/gui/lattice-core"
  echo "Setup complete."
}

cmd_build() {
  need cargo; need node; need npm
  echo "[+] Building Node/CLI (release)…"
  (cd "$ROOT_DIR/lattice-v3" && cargo build --release -p lattice-node -p lattice-cli)
  echo "[+] Building Explorer (Next.js)…"
  (cd "$ROOT_DIR/lattice-v3/explorer" && npm run build)
  echo "[+] Building Docs (Docusaurus)…"
  (cd "$ROOT_DIR/docs-portal" && npm run build)
  echo "[+] Building GUI web (Vite)…"
  (cd "$ROOT_DIR/lattice-v3/gui/lattice-core" && npm run build)
}

cmd_dev_up() {
  start_node_devnet
  start_explorer
  start_docs
  start_marketing
  echo "Dev stack started. See $LOG_DIR/*.log"
}

cmd_dev_down() {
  stop_pid explorer-web; stop_pid explorer-indexer; stop_pid docs; stop_pid marketing; stop_pid node
  if have docker; then (cd "$ROOT_DIR/lattice-v3" && compose stop explorer-db >/dev/null || true); fi
  echo "Dev stack stopped."
}

cmd_dev_status() {
  for n in node explorer-indexer explorer-web docs marketing; do
    if [ -f "$LOG_DIR/$n.pid" ]; then
      pid=$(cat "$LOG_DIR/$n.pid" 2>/dev/null || true)
      if [ -n "$pid" ] && kill -0 "$pid" 2>/dev/null; then
        echo "  $n: running (pid $pid)"
      else
        echo "  $n: not running"
      fi
    else
      echo "  $n: not started"
    fi
  done
}

cmd_testnet_up() { start_node_testnet; echo "Testnet node started (native)."; }
cmd_testnet_down() { stop_pid node; echo "Testnet node stopped."; }

cmd_mainnet_up() { echo "Mainnet start: placeholder (to be released)."; }
cmd_mainnet_down() { echo "Mainnet stop: placeholder (to be released)."; }

cmd_docker_up() {
  need docker
  (cd "$ROOT_DIR/lattice-v3" && compose up -d lattice-node-devnet)
  echo "Docker devnet node started via compose."
}

cmd_docker_down() {
  need docker
  (cd "$ROOT_DIR/lattice-v3" && compose stop lattice-node-devnet || true && compose rm -sf lattice-node-devnet || true)
  echo "Docker node stopped."
}

cmd_docker_testnet_up() {
  need docker
  (cd "$ROOT_DIR/lattice-v3" && compose up -d lattice-node-testnet)
  echo "Docker testnet node started via compose."
}

cmd_docker_testnet_down() {
  need docker
  (cd "$ROOT_DIR/lattice-v3" && compose stop lattice-node-testnet || true && compose rm -sf lattice-node-testnet || true)
  echo "Docker testnet node stopped."
}

cmd_logs() { tail -n 200 -F "$LOG_DIR"/*.log 2>/dev/null || echo "No logs in $LOG_DIR"; }

cmd_clean() {
  (cd "$ROOT_DIR/lattice-v3" && cargo clean || true)
  rm -rf "$ROOT_DIR"/**/node_modules "$ROOT_DIR/docs-portal/build" "$ROOT_DIR/lattice-v3/gui/lattice-core/dist" || true
  echo "Cleaned build artifacts and caches."
}

case "${1:-}" in
  setup) cmd_setup ;;
  build) cmd_build ;;
  dev)
    case "${2:-}" in
      up) cmd_dev_up ;;
      down) cmd_dev_down ;;
      status) cmd_dev_status ;;
      *) echo "Usage: $0 dev {up|down|status}"; exit 1 ;;
    esac ;;
  testnet)
    case "${2:-}" in
      up) cmd_testnet_up ;;
      down) cmd_testnet_down ;;
      *) echo "Usage: $0 testnet {up|down}"; exit 1 ;;
    esac ;;
  mainnet)
    case "${2:-}" in
      up) cmd_mainnet_up ;;
      down) cmd_mainnet_down ;;
      *) echo "Usage: $0 mainnet {up|down}"; exit 1 ;;
    esac ;;
  docker)
    case "${2:-}" in
      up) cmd_docker_up ;;
      down) cmd_docker_down ;;
      cluster)
        case "${3:-}" in
          up)
            need docker
            (cd "$ROOT_DIR/lattice-v3" && compose up -d --profile cluster lattice-node-1 lattice-node-2 lattice-node-3 lattice-node-4 lattice-node-5)
            echo "Cluster (5 nodes) started." ;;
          down)
            need docker
            (cd "$ROOT_DIR/lattice-v3" && compose stop lattice-node-1 lattice-node-2 lattice-node-3 lattice-node-4 lattice-node-5 || true && compose rm -sf lattice-node-1 lattice-node-2 lattice-node-3 lattice-node-4 lattice-node-5 || true)
            echo "Cluster stopped." ;;
          *) echo "Usage: $0 docker cluster {up|down}"; exit 1 ;;
        esac ;;
      testnet)
        case "${3:-}" in
          up) cmd_docker_testnet_up ;;
          down) cmd_docker_testnet_down ;;
          *) echo "Usage: $0 docker testnet {up|down}"; exit 1 ;;
        esac ;;
      explorer)
        case "${3:-}" in
          up)
            need docker
            (cd "$ROOT_DIR/lattice-v3" && compose up -d --profile explorer explorer-db explorer-web explorer-indexer)
            echo "Explorer services started (db, web:3000, indexer)." ;;
          down)
            need docker
            (cd "$ROOT_DIR/lattice-v3" && compose stop explorer-web explorer-indexer explorer-db || true && compose rm -sf explorer-web explorer-indexer explorer-db || true)
            echo "Explorer services stopped." ;;
          *) echo "Usage: $0 docker explorer {up|down}"; exit 1 ;;
        esac ;;
      monitoring)
        case "${3:-}" in
          up)
            need docker
            (cd "$ROOT_DIR/lattice-v3" && compose up -d --profile monitoring prometheus grafana)
            echo "Monitoring started (Prometheus:9090, Grafana:3001)." ;;
          down)
            need docker
            (cd "$ROOT_DIR/lattice-v3" && compose stop prometheus grafana || true && compose rm -sf prometheus grafana || true)
            echo "Monitoring stopped." ;;
          *) echo "Usage: $0 docker monitoring {up|down}"; exit 1 ;;
        esac ;;
      *) echo "Usage: $0 docker {up|down|testnet up|testnet down}"; exit 1 ;;
    esac ;;
  reset)
    case "${2:-}" in
      devnet)
        rm -rf "$ROOT_DIR/lattice-v3/.lattice-devnet" "$ROOT_DIR/lattice-v3/gui-data/devnet/chain" || true
        echo "Reset devnet data directories." ;;
      testnet)
        rm -rf "$HOME/.lattice" "$ROOT_DIR/lattice-v3/gui-data/testnet/chain" || true
        echo "Reset testnet data directories." ;;
      *) echo "Usage: $0 reset {devnet|testnet}"; exit 1 ;;
    esac ;;
  logs) cmd_logs ;;
  clean) cmd_clean ;;
  *) usage; exit 1 ;;
esac
