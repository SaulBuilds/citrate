#!/bin/bash

# Citrate SDKs Publishing Script
# Publishes Python and JavaScript SDKs to PyPI and NPM

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

    # Check if Python/pip is installed
    if ! command -v python3 &> /dev/null; then
        error "Python 3 is required but not installed"
    fi

    if ! command -v pip3 &> /dev/null; then
        error "pip3 is required but not installed"
    fi

    # Check if Node.js/npm is installed
    if ! command -v node &> /dev/null; then
        error "Node.js is required but not installed"
    fi

    if ! command -v npm &> /dev/null; then
        error "npm is required but not installed"
    fi

    # Check if build tools are installed
    if ! python3 -c "import build" 2>/dev/null; then
        info "Installing Python build tools..."
        pip3 install build twine
    fi

    log "Prerequisites check passed ✓"
}

# Publish Python SDK
publish_python_sdk() {
    log "Publishing Python SDK to PyPI..."

    SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
    PYTHON_SDK_DIR="$(dirname "$SCRIPT_DIR")/sdks/python"

    cd "$PYTHON_SDK_DIR"

    # Clean previous builds
    rm -rf dist/ build/ *.egg-info/

    # Install dependencies
    info "Installing Python SDK dependencies..."
    pip3 install -e ".[dev]"

    # Run tests
    info "Running Python SDK tests..."
    if [ -d "tests" ]; then
        pytest tests/ -v
    else
        warn "No tests directory found for Python SDK"
    fi

    # Build package
    info "Building Python package..."
    python3 -m build

    # Check package
    info "Checking Python package..."
    twine check dist/*

    # Ask for confirmation before publishing
    echo ""
    read -p "Do you want to publish the Python SDK to PyPI? (y/N): " -n 1 -r
    echo ""

    if [[ $REPLY =~ ^[Yy]$ ]]; then
        # Upload to PyPI
        info "Uploading to PyPI..."
        if [ "$1" = "--test" ]; then
            twine upload --repository testpypi dist/*
        else
            twine upload dist/*
        fi
        log "Python SDK published successfully ✓"
    else
        info "Python SDK publishing skipped"
    fi

    cd - > /dev/null
}

# Publish JavaScript SDK
publish_javascript_sdk() {
    log "Publishing JavaScript SDK to NPM..."

    SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
    JS_SDK_DIR="$(dirname "$SCRIPT_DIR")/sdk/javascript"

    cd "$JS_SDK_DIR"

    # Install dependencies
    info "Installing JavaScript SDK dependencies..."
    npm install

    # Run linting
    info "Running JavaScript SDK linting..."
    npm run lint

    # Run tests
    info "Running JavaScript SDK tests..."
    npm run test

    # Build package
    info "Building JavaScript package..."
    npm run build

    # Ask for confirmation before publishing
    echo ""
    read -p "Do you want to publish the JavaScript SDK to NPM? (y/N): " -n 1 -r
    echo ""

    if [[ $REPLY =~ ^[Yy]$ ]]; then
        # Publish to NPM
        info "Publishing to NPM..."
        if [ "$1" = "--test" ]; then
            npm publish --tag beta --access public
        else
            npm publish --access public
        fi
        log "JavaScript SDK published successfully ✓"
    else
        info "JavaScript SDK publishing skipped"
    fi

    cd - > /dev/null
}

# Main publishing function
main() {
    local mode="${1:-both}"

    log "🚀 Starting Citrate SDKs publishing..."

    check_prerequisites

    case "$mode" in
        "python")
            publish_python_sdk "$2"
            ;;
        "javascript"|"js")
            publish_javascript_sdk "$2"
            ;;
        "both")
            publish_python_sdk "$2"
            echo ""
            publish_javascript_sdk "$2"
            ;;
        *)
            error "Invalid mode: $mode. Use 'python', 'javascript', or 'both'"
            ;;
    esac

    log "🎉 SDK publishing completed!"

    info "Next steps:"
    info "1. Verify packages are available on PyPI/NPM"
    info "2. Update documentation with installation instructions"
    info "3. Create release notes and changelog entries"
    info "4. Notify community about new releases"
}

# Handle script arguments
case "${1:-both}" in
    "python"|"javascript"|"js"|"both")
        main "$1" "$2"
        ;;
    "--help"|"-h")
        echo "Usage: $0 [python|javascript|both] [--test]"
        echo ""
        echo "Options:"
        echo "  python:     Publish Python SDK only"
        echo "  javascript: Publish JavaScript SDK only"
        echo "  both:       Publish both SDKs (default)"
        echo "  --test:     Publish to test repositories (TestPyPI/beta tag)"
        echo ""
        echo "Examples:"
        echo "  $0                      # Publish both SDKs to production"
        echo "  $0 python              # Publish Python SDK only"
        echo "  $0 javascript --test    # Publish JS SDK to beta"
        exit 0
        ;;
    *)
        error "Invalid argument. Use --help for usage information"
        ;;
esac