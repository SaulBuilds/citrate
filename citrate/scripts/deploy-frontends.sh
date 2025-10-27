#!/bin/bash

# Citrate Frontends Deployment Script
# Deploys all developer tools frontends to Vercel

set -e

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

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

# Check prerequisites
check_prerequisites() {
    log "Checking prerequisites..."

    # Check if Vercel CLI is installed
    if ! command -v vercel &> /dev/null; then
        error "Vercel CLI is required but not installed. Run: npm install -g vercel"
    fi

    # Check if logged in to Vercel
    if ! vercel whoami &> /dev/null; then
        error "Please login to Vercel first: vercel login"
    fi

    log "Prerequisites check passed ✓"
}

# Deploy individual frontend
deploy_frontend() {
    local app_name=$1
    local app_path=$2

    log "Deploying $app_name..."

    cd "$app_path"

    # Install dependencies if needed
    if [ ! -d "node_modules" ]; then
        info "Installing dependencies for $app_name..."
        npm install
    fi

    # Build the application
    info "Building $app_name..."
    npm run build

    # Deploy to Vercel
    info "Deploying $app_name to Vercel..."
    vercel --prod --yes

    log "$app_name deployed successfully ✓"
    cd - > /dev/null
}

# Main deployment function
main() {
    log "🚀 Starting Citrate frontends deployment..."

    check_prerequisites

    # Get the developer-tools directory
    SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
    DEVELOPER_TOOLS_DIR="$(dirname "$SCRIPT_DIR")/developer-tools"

    if [ ! -d "$DEVELOPER_TOOLS_DIR" ]; then
        error "Developer tools directory not found: $DEVELOPER_TOOLS_DIR"
    fi

    # Deploy each frontend
    deploy_frontend "Citrate Studio" "$DEVELOPER_TOOLS_DIR/citrate-studio"
    deploy_frontend "Documentation Portal" "$DEVELOPER_TOOLS_DIR/documentation-portal"
    deploy_frontend "Debug Dashboard" "$DEVELOPER_TOOLS_DIR/debug-dashboard"

    log "🎉 All frontends deployed successfully!"
    log "📄 URLs will be displayed in the Vercel CLI output above"

    info "Next steps:"
    info "1. Update DNS records to point to Vercel deployments"
    info "2. Configure custom domains in Vercel dashboard"
    info "3. Set up monitoring and alerts"
    info "4. Update documentation with new URLs"
}

# Handle script arguments
case "${1:-deploy}" in
    "deploy")
        main
        ;;
    "studio")
        SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
        DEVELOPER_TOOLS_DIR="$(dirname "$SCRIPT_DIR")/developer-tools"
        deploy_frontend "Citrate Studio" "$DEVELOPER_TOOLS_DIR/citrate-studio"
        ;;
    "docs")
        SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
        DEVELOPER_TOOLS_DIR="$(dirname "$SCRIPT_DIR")/developer-tools"
        deploy_frontend "Documentation Portal" "$DEVELOPER_TOOLS_DIR/documentation-portal"
        ;;
    "dashboard")
        SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
        DEVELOPER_TOOLS_DIR="$(dirname "$SCRIPT_DIR")/developer-tools"
        deploy_frontend "Debug Dashboard" "$DEVELOPER_TOOLS_DIR/debug-dashboard"
        ;;
    *)
        echo "Usage: $0 [deploy|studio|docs|dashboard]"
        echo "  deploy:    Deploy all frontends (default)"
        echo "  studio:    Deploy Citrate Studio only"
        echo "  docs:      Deploy Documentation Portal only"
        echo "  dashboard: Deploy Debug Dashboard only"
        exit 1
        ;;
esac