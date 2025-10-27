#!/bin/bash

# Stop development servers started by deploy-dev.sh

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
NC='\033[0m'

log() {
    echo -e "${GREEN}[$(date +'%H:%M:%S')]${NC} $1"
}

error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

log "Stopping Citrate development servers..."

# Stop Citrate node
if [[ -f ".citrate-dev.pid" ]]; then
    PID=$(cat .citrate-dev.pid)
    if kill -0 $PID 2>/dev/null; then
        kill $PID
        log "Stopped Citrate node (PID: $PID)"
    fi
    rm -f .citrate-dev.pid
fi

# Stop Explorer
if [[ -f ".explorer-dev.pid" ]]; then
    PID=$(cat .explorer-dev.pid)
    if kill -0 $PID 2>/dev/null; then
        kill $PID
        log "Stopped Explorer (PID: $PID)"
    fi
    rm -f .explorer-dev.pid
fi

# Kill any remaining processes
pkill -f "lattice.*devnet" 2>/dev/null || true
pkill -f "next.*dev" 2>/dev/null || true

log "✅ All development servers stopped"