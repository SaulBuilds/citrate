#!/bin/bash

# Lattice V3 Development Deployment Script
# Quick deployment for development and testing environments

set -e

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

log() {
    echo -e "${GREEN}[$(date +'%H:%M:%S')]${NC} $1"
}

info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

error() {
    echo -e "${RED}[ERROR]${NC} $1"
    exit 1
}

# Configuration
COMPONENTS=${1:-all}
BUILD_MODE=${2:-debug}

show_help() {
    echo "Usage: $0 [components] [build_mode]"
    echo ""
    echo "Components:"
    echo "  all       - Build all components (default)"
    echo "  node      - Core node binary only"
    echo "  cli       - CLI tools only"
    echo "  gui       - GUI application only"
    echo "  explorer  - Explorer web app only"
    echo "  sdks      - JavaScript and Python SDKs only"
    echo ""
    echo "Build modes:"
    echo "  debug     - Debug build (default, faster)"
    echo "  release   - Release build (optimized)"
    echo ""
    echo "Examples:"
    echo "  $0                    # Build all components in debug mode"
    echo "  $0 node release       # Build only node in release mode"
    echo "  $0 gui                # Build only GUI in debug mode"
}

# Build Rust components
build_rust() {
    local component=$1
    local mode=$2

    if [[ "$mode" == "release" ]]; then
        local flag="--release"
        local target_dir="target/release"
    else
        local flag=""
        local target_dir="target/debug"
    fi

    case $component in
        "node")
            log "Building Lattice node ($mode)..."
            cargo build $flag --bin lattice
            info "Node binary: $target_dir/lattice"
            ;;
        "cli")
            log "Building CLI tools ($mode)..."
            cargo build $flag --bin lattice-cli
            cargo build $flag --bin lattice-wallet
            cargo build $flag --bin faucet
            info "CLI binaries: $target_dir/{lattice-cli,lattice-wallet,faucet}"
            ;;
        "all")
            log "Building all Rust components ($mode)..."
            cargo build $flag
            info "All binaries built in $target_dir/"
            ;;
    esac
}

# Build GUI application
build_gui() {
    log "Building GUI application..."

    cd gui/lattice-core

    if [ ! -d "node_modules" ]; then
        info "Installing dependencies..."
        npm install
    fi

    info "Building Tauri app..."
    npm run tauri:build

    cd - > /dev/null

    info "GUI application built successfully"
}

# Build Explorer
build_explorer() {
    log "Building Explorer..."

    cd explorer

    if [ ! -d "node_modules" ]; then
        info "Installing dependencies..."
        npm install
    fi

    info "Building Next.js app..."
    npm run build

    cd - > /dev/null

    info "Explorer built successfully"
}

# Build SDKs
build_sdks() {
    log "Building SDKs..."

    # JavaScript SDK
    info "Building JavaScript SDK..."
    cd sdk/javascript

    if [ ! -d "node_modules" ]; then
        npm install
    fi

    npm run build
    cd - > /dev/null

    # Python SDK
    info "Building Python SDK..."
    cd sdks/python

    if [ ! -d "venv" ]; then
        python3 -m venv venv
        source venv/bin/activate
        pip install -e ".[dev]"
    else
        source venv/bin/activate
    fi

    python -m build
    deactivate
    cd - > /dev/null

    info "SDKs built successfully"
}

# Quick test function
run_quick_tests() {
    log "Running quick tests..."

    # Rust tests (fast ones only)
    info "Running Rust unit tests..."
    cargo test --lib --bins

    # JavaScript SDK tests
    if [[ -d "sdk/javascript/node_modules" ]]; then
        info "Running JavaScript SDK tests..."
        cd sdk/javascript
        npm test
        cd - > /dev/null
    fi

    log "Quick tests completed ✓"
}

# Development server startup
start_dev_servers() {
    log "Starting development servers..."

    # Start node in background
    if [[ -f "target/debug/lattice" ]]; then
        info "Starting Lattice node..."
        ./target/debug/lattice --network devnet --data-dir .lattice-dev &
        echo $! > .lattice-dev.pid
        sleep 2
    fi

    # Start explorer
    if [[ -d "explorer/node_modules" ]]; then
        info "Starting Explorer dev server..."
        cd explorer
        npm run dev &
        echo $! > ../.explorer-dev.pid
        cd - > /dev/null
    fi

    info "Development servers started:"
    info "  • Node: http://localhost:8545"
    info "  • Explorer: http://localhost:3000"
    info ""
    info "To stop servers: ./scripts/stop-dev.sh"
}

# Main function
main() {
    echo "🚀 Lattice V3 Development Deployment"
    echo "Components: $COMPONENTS"
    echo "Build mode: $BUILD_MODE"
    echo ""

    case $COMPONENTS in
        "node")
            build_rust "node" "$BUILD_MODE"
            ;;
        "cli")
            build_rust "cli" "$BUILD_MODE"
            ;;
        "gui")
            build_gui
            ;;
        "explorer")
            build_explorer
            ;;
        "sdks")
            build_sdks
            ;;
        "all")
            build_rust "all" "$BUILD_MODE"
            build_gui
            build_explorer
            build_sdks
            ;;
        "test")
            run_quick_tests
            ;;
        "serve"|"start")
            start_dev_servers
            ;;
        *)
            error "Unknown component: $COMPONENTS"
            ;;
    esac

    echo ""
    log "✅ Development deployment completed!"

    if [[ "$COMPONENTS" == "all" || "$COMPONENTS" == "node" ]]; then
        echo ""
        echo "Next steps:"
        echo "  • Start dev environment: ./scripts/deploy-dev.sh serve"
        echo "  • Run tests: ./scripts/deploy-dev.sh test"
        echo "  • Build GUI: ./scripts/deploy-dev.sh gui"
    fi
}

# Handle arguments
case "${1:-all}" in
    "all"|"node"|"cli"|"gui"|"explorer"|"sdks"|"test"|"serve"|"start")
        main
        ;;
    "--help"|"-h")
        show_help
        ;;
    *)
        show_help
        exit 1
        ;;
esac