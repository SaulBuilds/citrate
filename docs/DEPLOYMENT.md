Citrate v3 Testnet Deployment Guide

Overview

This guide shows how to bring up a local Citrate v3 Testnet stack:

- Core node with RPC and WebSocket
- Explorer (Next.js UI + indexer) backed by Postgres + Redis
- Desktop GUI (Tauri)

Use the provided script for a one‑command setup, or follow the manual steps.

Prerequisites

- Rust toolchain (rustup, cargo)
- Node.js 18+ and npm
- Docker and Docker Compose
- macOS or Linux (tested on macOS)

Quickstart (recommended)

1) From the repo root, run:

   scripts/testnet-up.sh

   The script will:
   - Build and start the core node with RPC on 127.0.0.1:8545 and WS on 127.0.0.1:8546
   - Start Postgres and Redis in Docker
   - Apply Prisma migrations and start the Explorer (http://localhost:3000) and Indexer
   - Launch the GUI (Tauri) once services are healthy

2) Open the Explorer at http://localhost:3000 and the GUI window that appears.

3) Stopping services:
   - Core node, Explorer, and Indexer started by the script can be stopped with:
     - pkill -f "citrate-node"
     - (from citrate/explorer) docker compose down

Manual Steps

1) Core Node + RPC

   - Build and run the node:
     cd citrate
     cargo run -p citrate-node --release

   - By default, RPC listens on 127.0.0.1:8545 and WS on 127.0.0.1:8546.

2) Explorer (Postgres, Redis, Web, Indexer)

   - Start DB and cache:
     cd citrate/explorer
     docker compose up -d postgres redis

   - Wait until Postgres is healthy, then apply migrations and generate Prisma client:
     npm ci
     npx prisma migrate deploy
     npx prisma generate

   - Start Explorer web and indexer (Docker):
     RPC_ENDPOINT=http://host.docker.internal:8545 docker compose up -d explorer indexer

   - Explorer will be available at http://localhost:3000

3) GUI (Tauri)

   - In a separate terminal:
     cd citrate/gui/lattice-core
     npm ci
     npm run tauri

Troubleshooting

- If Explorer can’t connect to RPC from Docker, ensure host.docker.internal resolves. On Linux, you may need to set RPC_ENDPOINT to your host IP.
- If Prisma migration fails, ensure Postgres is running and DATABASE_URL in explorer/.env points to localhost:5432.
- For a fresh start, stop services, then remove the Postgres volume (docker volume rm) and rerun migrations.

