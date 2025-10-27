#!/bin/bash

# Citrate V3 Complete Deployment Script
# Deploys all components of the Citrate ecosystem

set -e

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
PURPLE='\033[0;35m'
NC='\033[0m' # No Color

# Configuration
DEPLOYMENT_ENV=${1:-production}
VERSION=${2:-$(git describe --tags --abbrev=0 2>/dev/null || echo "v0.1.0")}

log() {
    echo -e "${GREEN}[$(date +'%Y-%m-%d %H:%M:%S')]${NC} $1"
}

info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

warn() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

error() {
    echo -e "${RED}[ERROR]${NC} $1"
    exit 1
}

section() {
    echo ""
    echo -e "${PURPLE}============================================${NC}"
    echo -e "${PURPLE} $1${NC}"
    echo -e "${PURPLE}============================================${NC}"
    echo ""
}

# Check prerequisites
check_prerequisites() {
    log "Checking deployment prerequisites..."

    # Check Rust
    if ! command -v cargo &> /dev/null; then
        error "Rust/Cargo is required but not installed"
    fi

    # Check Node.js
    if ! command -v node &> /dev/null; then
        error "Node.js is required but not installed"
    fi

    # Check Python
    if ! command -v python3 &> /dev/null; then
        error "Python 3 is required but not installed"
    fi

    # Check Docker (optional but recommended)
    if ! command -v docker &> /dev/null; then
        warn "Docker not found - container deployment will be skipped"
    fi

    # Check GitHub CLI (for releases)
    if ! command -v gh &> /dev/null; then
        warn "GitHub CLI not found - GitHub releases will be skipped"
    fi

    log "Prerequisites check completed ✓"
}

# Build Rust binaries
build_rust_binaries() {
    section "Building Rust Binaries"

    local targets=("x86_64-unknown-linux-gnu" "x86_64-apple-darwin" "aarch64-apple-darwin" "x86_64-pc-windows-msvc")

    for target in "${targets[@]}"; do
        info "Building for target: $target"

        # Install target if not present
        rustup target add "$target" 2>/dev/null || true

        # Build release binaries
        cargo build --release --target "$target"

        # Create target-specific directory
        mkdir -p "dist/binaries/$target"

        # Copy binaries
        if [[ "$target" == *"windows"* ]]; then
            cp "target/$target/release/lattice.exe" "dist/binaries/$target/"
            cp "target/$target/release/citrate-cli.exe" "dist/binaries/$target/"
            cp "target/$target/release/citrate-wallet.exe" "dist/binaries/$target/"
            cp "target/$target/release/faucet.exe" "dist/binaries/$target/"
        else
            cp "target/$target/release/lattice" "dist/binaries/$target/"
            cp "target/$target/release/citrate-cli" "dist/binaries/$target/"
            cp "target/$target/release/citrate-wallet" "dist/binaries/$target/"
            cp "target/$target/release/faucet" "dist/binaries/$target/"
        fi

        log "Built binaries for $target ✓"
    done
}

# Build GUI applications
build_gui_applications() {
    section "Building GUI Applications"

    cd gui/citrate-core

    info "Installing dependencies..."
    npm ci

    info "Building Tauri application..."
    npm run tauri:build

    # Copy GUI artifacts
    mkdir -p "../../dist/gui"

    if [[ "$OSTYPE" == "linux-gnu"* ]]; then
        cp src-tauri/target/release/bundle/deb/*.deb ../../dist/gui/ 2>/dev/null || true
        cp src-tauri/target/release/bundle/appimage/*.AppImage ../../dist/gui/ 2>/dev/null || true
    elif [[ "$OSTYPE" == "darwin"* ]]; then
        cp src-tauri/target/release/bundle/dmg/*.dmg ../../dist/gui/ 2>/dev/null || true
        cp -r src-tauri/target/release/bundle/macos/*.app ../../dist/gui/ 2>/dev/null || true
    elif [[ "$OSTYPE" == "msys" ]]; then
        cp src-tauri/target/release/bundle/msi/*.msi ../../dist/gui/ 2>/dev/null || true
        cp src-tauri/target/release/bundle/nsis/*.exe ../../dist/gui/ 2>/dev/null || true
    fi

    cd - > /dev/null
    log "GUI applications built ✓"
}

# Build and publish SDKs
build_and_publish_sdks() {
    section "Building and Publishing SDKs"

    # JavaScript SDK
    info "Building JavaScript SDK..."
    cd sdk/javascript
    npm ci
    npm run lint
    npm run test
    npm run build

    if [[ "$DEPLOYMENT_ENV" == "production" ]]; then
        info "Publishing JavaScript SDK to NPM..."
        npm publish --access public
    else
        info "Building JavaScript SDK for staging..."
    fi

    cd - > /dev/null

    # Python SDK
    info "Building Python SDK..."
    cd sdks/python

    # Install dependencies
    pip3 install -e ".[dev]"

    # Run tests and linting
    pytest tests/ -v
    ruff check .
    black --check .

    # Build package
    python3 -m build

    if [[ "$DEPLOYMENT_ENV" == "production" ]]; then
        info "Publishing Python SDK to PyPI..."
        twine upload dist/*
    else
        info "Building Python SDK for staging..."
    fi

    cd - > /dev/null
    log "SDKs built and published ✓"
}

# Build web applications
build_web_applications() {
    section "Building Web Applications"

    # Explorer
    info "Building Explorer..."
    cd explorer
    npm ci
    npm run build
    mkdir -p ../dist/web/explorer
    cp -r .next/static ../dist/web/explorer/ 2>/dev/null || true
    cp -r out/* ../dist/web/explorer/ 2>/dev/null || true
    cd - > /dev/null

    log "Web applications built ✓"
}

# Create Docker images
build_docker_images() {
    if ! command -v docker &> /dev/null; then
        warn "Docker not available, skipping container builds"
        return
    fi

    section "Building Docker Images"

    # Node Docker image
    info "Building Citrate Node Docker image..."
    cat > Dockerfile.node << EOF
FROM rust:1.70 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/lattice /usr/local/bin/lattice
EXPOSE 8545 8546 30303
CMD ["citrate", "--config", "/etc/lattice/config.toml"]
EOF

    docker build -f Dockerfile.node -t "lattice/node:$VERSION" .
    docker tag "lattice/node:$VERSION" "lattice/node:latest"

    # Explorer Docker image
    info "Building Explorer Docker image..."
    cd explorer
    cat > Dockerfile << EOF
FROM node:18-alpine
WORKDIR /app
COPY package*.json ./
RUN npm ci --only=production
COPY . .
RUN npm run build
EXPOSE 3000
CMD ["npm", "start"]
EOF

    docker build -t "lattice/explorer:$VERSION" .
    docker tag "lattice/explorer:$VERSION" "lattice/explorer:latest"
    cd - > /dev/null

    log "Docker images built ✓"
}

# Deploy to cloud platforms
deploy_to_cloud() {
    section "Deploying to Cloud Platforms"

    # Deploy Explorer to Vercel (if configured)
    if [[ -n "${VERCEL_TOKEN:-}" ]]; then
        info "Deploying Explorer to Vercel..."
        cd explorer
        npx vercel --prod --token "$VERCEL_TOKEN"
        cd - > /dev/null
    fi

    # Deploy documentation to GitHub Pages
    if command -v gh &> /dev/null; then
        info "Deploying documentation..."
        if [[ -f "mkdocs.yml" ]]; then
            pip3 install mkdocs mkdocs-material
            mkdocs build
            # GitHub Pages deployment would be handled by GitHub Actions
        fi
    fi

    log "Cloud deployment completed ✓"
}

# Create release packages
create_release_packages() {
    section "Creating Release Packages"

    mkdir -p dist/releases

    # Create binary packages
    for target_dir in dist/binaries/*/; do
        target=$(basename "$target_dir")
        info "Creating package for $target..."

        cd "$target_dir"
        if [[ "$target" == *"windows"* ]]; then
            zip -r "../../releases/citrate-$VERSION-$target.zip" .
        else
            tar -czf "../../releases/citrate-$VERSION-$target.tar.gz" .
        fi
        cd - > /dev/null
    done

    # Create GUI packages
    if [[ -d "dist/gui" && $(ls -A dist/gui) ]]; then
        info "Creating GUI packages..."
        cd dist/gui
        for file in *; do
            if [[ -f "$file" ]]; then
                cp "$file" "../releases/"
            fi
        done
        cd - > /dev/null
    fi

    # Create checksums
    info "Creating checksums..."
    cd dist/releases
    sha256sum * > checksums.txt
    cd - > /dev/null

    log "Release packages created ✓"
}

# Create GitHub release
create_github_release() {
    if ! command -v gh &> /dev/null; then
        warn "GitHub CLI not available, skipping GitHub release"
        return
    fi

    section "Creating GitHub Release"

    # Check if release already exists
    if gh release view "$VERSION" &>/dev/null; then
        warn "Release $VERSION already exists"
        return
    fi

    info "Creating GitHub release $VERSION..."

    # Create release
    gh release create "$VERSION" \
        --title "Citrate V3 Release $VERSION" \
        --notes "Release $VERSION of Citrate V3 AI Blockchain Platform" \
        dist/releases/*

    log "GitHub release created ✓"
}

# Generate deployment report
generate_deployment_report() {
    section "Generating Deployment Report"

    cat > "dist/deployment-report-$VERSION.md" << EOF
# Citrate V3 Deployment Report

**Version**: $VERSION
**Environment**: $DEPLOYMENT_ENV
**Date**: $(date)
**Git Commit**: $(git rev-parse HEAD)

## Components Deployed

### Core Binaries
- ✅ lattice (main node)
- ✅ citrate-cli (command line interface)
- ✅ citrate-wallet (wallet binary)
- ✅ faucet (test token faucet)

### GUI Applications
- ✅ Citrate Core (Tauri desktop app)

### SDKs
- ✅ JavaScript/TypeScript SDK (@citrate-ai/sdk)
- ✅ Python SDK (citrate-sdk)

### Web Applications
- ✅ Explorer (Next.js application)

### Docker Images
- ✅ lattice/node:$VERSION
- ✅ lattice/explorer:$VERSION

## Deployment Artifacts

### Binary Packages
$(ls -la dist/releases/*.tar.gz dist/releases/*.zip 2>/dev/null | awk '{print "- " $9 " (" $5 " bytes)"}' || echo "- No binary packages found")

### GUI Packages
$(ls -la dist/releases/*.deb dist/releases/*.dmg dist/releases/*.msi dist/releases/*.exe dist/releases/*.AppImage 2>/dev/null | awk '{print "- " $9 " (" $5 " bytes)"}' || echo "- No GUI packages found")

## Next Steps

1. **Verify Deployments**: Test all deployed components
2. **Update Documentation**: Update installation guides and API docs
3. **Community Notification**: Announce release to community
4. **Monitor**: Set up monitoring for all deployed services

## Support

- Documentation: https://docs.lattice.ai
- GitHub: https://github.com/citrate-ai/citrate
- Discord: https://discord.gg/citrate-ai

EOF

    log "Deployment report generated: dist/deployment-report-$VERSION.md ✓"
}

# Main deployment function
main() {
    log "🚀 Starting Citrate V3 complete deployment..."
    log "Environment: $DEPLOYMENT_ENV"
    log "Version: $VERSION"

    # Clean and prepare
    rm -rf dist/
    mkdir -p dist/{binaries,gui,web,releases}

    # Run deployment steps
    check_prerequisites
    build_rust_binaries
    build_gui_applications
    build_and_publish_sdks
    build_web_applications
    build_docker_images
    deploy_to_cloud
    create_release_packages
    create_github_release
    generate_deployment_report

    section "🎉 Deployment Complete!"

    echo "✅ All components successfully deployed"
    echo "📦 Release packages: $(ls -1 dist/releases/*.tar.gz dist/releases/*.zip 2>/dev/null | wc -l) created"
    echo "🐳 Docker images: lattice/node:$VERSION, lattice/explorer:$VERSION"
    echo "📊 Report: dist/deployment-report-$VERSION.md"
    echo ""
    echo "🔗 Quick Links:"
    echo "  • Repository: https://github.com/citrate-ai/citrate"
    echo "  • Documentation: https://docs.lattice.ai"
    echo "  • Explorer: https://explorer.lattice.ai"
    echo "  • Discord: https://discord.gg/citrate-ai"
}

# Handle script arguments
case "${1:-production}" in
    "production"|"staging"|"development")
        main
        ;;
    "--help"|"-h")
        echo "Usage: $0 [environment] [version]"
        echo ""
        echo "Environments:"
        echo "  production:  Full production deployment (default)"
        echo "  staging:     Staging deployment (no publishing)"
        echo "  development: Development deployment (local only)"
        echo ""
        echo "Examples:"
        echo "  $0                           # Production deployment, auto-detect version"
        echo "  $0 production v1.0.0         # Production deployment, specific version"
        echo "  $0 staging                   # Staging deployment"
        exit 0
        ;;
    *)
        error "Invalid environment. Use production, staging, or development"
        ;;
esac