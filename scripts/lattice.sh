#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)

usage() {
  cat <<EOF
Lattice DevOps Script

Usage: scripts/lattice.sh <command>

Commands:
  build              Build all Rust binaries, Docs, and Webapp
  test               Run Rust workspace tests
  docker:build       Build all Docker images (node, node-app, faucet, docs, web)
  up:dev             Start dev stack with docker-compose.dev.yml
  down:dev           Stop dev stack
  up:prod            Start prod stack with docker-compose.prod.yml
  down:prod          Stop prod stack
  docs               Start docs portal locally (docusaurus)
  web                Start marketing site locally (Next.js)
  clean              Clean target and node modules caches

Examples:
  scripts/lattice.sh build
  scripts/lattice.sh docker:build
  scripts/lattice.sh up:dev
EOF
}

ensure_cmd() { command -v "$1" >/dev/null 2>&1 || { echo "Missing required command: $1"; exit 1; }; }

cmd_build() {
  ensure_cmd cargo; ensure_cmd npm
  echo "[+] Building Rust workspace (release)…"
  (cd "$ROOT_DIR/lattice-v3" && cargo build --release -p lattice-node -p node-app -p lattice-faucet -p lattice-cli)

  echo "[+] Building Docs portal…"
  (cd "$ROOT_DIR/docs-portal" && npm ci || npm install && npm run build)

  echo "[+] Building Marketing/Web (Next.js)…"
  (cd "$ROOT_DIR/marketing-site" && npm ci || npm install && npm run build)
}

cmd_test() {
  (cd "$ROOT_DIR/lattice-v3" && cargo test --workspace)
}

cmd_docker_build() {
  ensure_cmd docker
  echo "[+] Building node image…"
  docker build -f "$ROOT_DIR/lattice-v3/docker/node.Dockerfile" -t lattice/node:local "$ROOT_DIR"
  echo "[+] Building node-app image…"
  docker build -f "$ROOT_DIR/lattice-v3/docker/node-app.Dockerfile" -t lattice/node-app:local "$ROOT_DIR"
  echo "[+] Building faucet image…"
  docker build -f "$ROOT_DIR/lattice-v3/docker/faucet.Dockerfile" -t lattice/faucet:local "$ROOT_DIR"
  echo "[+] Building docs portal image…"
  docker build -f "$ROOT_DIR/docs-portal/Dockerfile" -t lattice/docs-portal:local "$ROOT_DIR"
  echo "[+] Building marketing-site image…"
  docker build -f "$ROOT_DIR/marketing-site/Dockerfile" -t lattice/marketing-site:local "$ROOT_DIR"
}

cmd_up_dev() {
  ensure_cmd docker; ensure_cmd docker-compose || true
  (cd "$ROOT_DIR/lattice-v3/docker" && docker compose -f docker-compose.dev.yml up -d)
}

cmd_down_dev() {
  (cd "$ROOT_DIR/lattice-v3/docker" && docker compose -f docker-compose.dev.yml down)
}

cmd_up_prod() {
  ensure_cmd docker; ensure_cmd docker-compose || true
  (cd "$ROOT_DIR/lattice-v3/docker" && docker compose -f docker-compose.prod.yml up -d)
}

cmd_down_prod() {
  (cd "$ROOT_DIR/lattice-v3/docker" && docker compose -f docker-compose.prod.yml down)
}

cmd_docs() {
  (cd "$ROOT_DIR/docs-portal" && npm run start)
}

cmd_web() {
  (cd "$ROOT_DIR/marketing-site" && npm run dev)
}

cmd_clean() {
  echo "[+] Cleaning Rust targets…"; (cd "$ROOT_DIR/lattice-v3" && cargo clean)
  echo "[+] Cleaning node_modules caches (docs/web)…"
  rm -rf "$ROOT_DIR/docs-portal/node_modules" "$ROOT_DIR/marketing-site/node_modules" || true
}

case "${1:-}" in
  build) cmd_build ;;
  test) cmd_test ;;
  docker:build) cmd_docker_build ;;
  up:dev) cmd_up_dev ;;
  down:dev) cmd_down_dev ;;
  up:prod) cmd_up_prod ;;
  down:prod) cmd_down_prod ;;
  docs) cmd_docs ;;
  web) cmd_web ;;
  clean) cmd_clean ;;
  *) usage; exit 1 ;;
esac

