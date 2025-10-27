#!/bin/bash

# Citrate SDK Installation Script
# Installs Citrate SDKs for multiple programming languages

set -e

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Configuration
DEFAULT_VERSION="latest"
INSTALL_JS=${INSTALL_JS:-true}
INSTALL_PYTHON=${INSTALL_PYTHON:-true}
INSTALL_CLI=${INSTALL_CLI:-true}

print_header() {
    echo ""
    echo -e "${BLUE}============================================${NC}"
    echo -e "${BLUE} Citrate AI Blockchain SDK Installer${NC}"
    echo -e "${BLUE}============================================${NC}"
    echo ""
}

log() {
    echo -e "${GREEN}[$(date +'%Y-%m-%d %H:%M:%S')]${NC} $1"
}

warn() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

error() {
    echo -e "${RED}[ERROR]${NC} $1"
    exit 1
}

check_command() {
    if ! command -v "$1" &> /dev/null; then
        return 1
    fi
    return 0
}

install_nodejs_sdk() {
    log "Installing JavaScript/TypeScript SDK..."

    if ! check_command npm; then
        error "npm is required but not installed. Please install Node.js first."
    fi

    # Check if we're in a project directory
    if [[ -f "package.json" ]]; then
        log "Installing Citrate SDK in current project..."
        npm install @citrate-ai/sdk
    else
        log "Installing Citrate SDK globally..."
        npm install -g @citrate-ai/sdk
    fi

    log "JavaScript/TypeScript SDK installed successfully! ✓"
    echo ""
    echo "Usage example:"
    echo "  import { CitrateSDK } from '@citrate-ai/sdk';"
    echo "  const sdk = new CitrateSDK({ nodeUrl: 'http://localhost:8545' });"
}

install_python_sdk() {
    log "Installing Python SDK..."

    if ! check_command pip3 && ! check_command pip; then
        error "pip is required but not installed. Please install Python 3 first."
    fi

    # Use pip3 if available, otherwise pip
    local pip_cmd="pip3"
    if ! check_command pip3; then
        pip_cmd="pip"
    fi

    # Check if we're in a virtual environment
    if [[ -n "$VIRTUAL_ENV" ]]; then
        log "Installing in virtual environment: $VIRTUAL_ENV"
        $pip_cmd install citrate-sdk
    else
        log "Installing globally (consider using a virtual environment)"
        $pip_cmd install --user citrate-sdk
    fi

    log "Python SDK installed successfully! ✓"
    echo ""
    echo "Usage example:"
    echo "  from citrate_sdk import CitrateClient"
    echo "  client = CitrateClient(node_url='http://localhost:8545')"
}

install_cli_tools() {
    log "Installing CLI tools..."

    # Detect OS and architecture
    local os=""
    local arch=""

    case "$(uname -s)" in
        Darwin) os="macos" ;;
        Linux) os="linux" ;;
        MINGW*|CYGWIN*|MSYS*) os="windows" ;;
        *) error "Unsupported operating system: $(uname -s)" ;;
    esac

    case "$(uname -m)" in
        x86_64|amd64) arch="x86_64" ;;
        arm64|aarch64) arch="arm64" ;;
        *) error "Unsupported architecture: $(uname -m)" ;;
    esac

    local binary_suffix=""
    if [[ "$os" == "windows" ]]; then
        binary_suffix=".exe"
    fi

    # Try Homebrew first on macOS
    if [[ "$os" == "macos" ]] && check_command brew; then
        log "Installing via Homebrew..."
        if brew tap citrate-ai/tap 2>/dev/null || true; then
            if brew install lattice 2>/dev/null; then
                log "CLI tools installed via Homebrew! ✓"
                return
            else
                warn "Homebrew installation failed, falling back to direct download"
            fi
        else
            warn "Homebrew tap not available, falling back to direct download"
        fi
    fi

    # Direct download and installation
    local version="${VERSION:-$DEFAULT_VERSION}"
    if [[ "$version" == "latest" ]]; then
        log "Fetching latest version..."
        version=$(curl -s https://api.github.com/repos/citrate-ai/citrate/releases/latest | grep -o '"tag_name": "[^"]*' | cut -d'"' -f4)
        if [[ -z "$version" ]]; then
            version="v0.1.0"  # fallback version
        fi
    fi

    local filename="citrate-${version#v}-${os}-${arch}.tar.gz"
    local url="https://github.com/citrate-ai/citrate/releases/download/${version}/${filename}"

    log "Downloading from: $url"

    # Create temporary directory
    local temp_dir=$(mktemp -d)
    trap "rm -rf $temp_dir" EXIT

    # Download and extract
    if check_command wget; then
        wget -q -O "$temp_dir/$filename" "$url"
    elif check_command curl; then
        curl -sL -o "$temp_dir/$filename" "$url"
    else
        error "wget or curl is required for downloading"
    fi

    # Extract
    tar -xzf "$temp_dir/$filename" -C "$temp_dir"

    # Install to appropriate location
    local install_dir="/usr/local/bin"
    if [[ ! -w "$install_dir" ]]; then
        install_dir="$HOME/.local/bin"
        mkdir -p "$install_dir"
        if [[ ":$PATH:" != *":$install_dir:"* ]]; then
            warn "Add $install_dir to your PATH to use the CLI tools"
            echo "  export PATH=\"\$PATH:$install_dir\""
        fi
    fi

    # Copy binaries
    for binary in lattice citrate-cli citrate-wallet faucet; do
        if [[ -f "$temp_dir/${binary}${binary_suffix}" ]]; then
            if [[ -w "$install_dir" ]]; then
                cp "$temp_dir/${binary}${binary_suffix}" "$install_dir/"
                chmod +x "$install_dir/${binary}${binary_suffix}"
            else
                sudo cp "$temp_dir/${binary}${binary_suffix}" "$install_dir/"
                sudo chmod +x "$install_dir/${binary}${binary_suffix}"
            fi
            log "Installed ${binary}${binary_suffix} to $install_dir"
        fi
    done

    log "CLI tools installed successfully! ✓"
}

show_usage() {
    echo "Citrate SDK Installer"
    echo ""
    echo "Usage: $0 [options]"
    echo ""
    echo "Options:"
    echo "  --js-only          Install only JavaScript/TypeScript SDK"
    echo "  --python-only      Install only Python SDK"
    echo "  --cli-only         Install only CLI tools"
    echo "  --no-js           Skip JavaScript/TypeScript SDK"
    echo "  --no-python       Skip Python SDK"
    echo "  --no-cli          Skip CLI tools"
    echo "  --version VERSION  Install specific version (default: latest)"
    echo "  --help            Show this help message"
    echo ""
    echo "Environment variables:"
    echo "  VERSION           Version to install (default: latest)"
    echo "  INSTALL_JS        Install JS SDK (default: true)"
    echo "  INSTALL_PYTHON    Install Python SDK (default: true)"
    echo "  INSTALL_CLI       Install CLI tools (default: true)"
}

main() {
    print_header

    # Parse command line arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            --js-only)
                INSTALL_JS=true
                INSTALL_PYTHON=false
                INSTALL_CLI=false
                shift
                ;;
            --python-only)
                INSTALL_JS=false
                INSTALL_PYTHON=true
                INSTALL_CLI=false
                shift
                ;;
            --cli-only)
                INSTALL_JS=false
                INSTALL_PYTHON=false
                INSTALL_CLI=true
                shift
                ;;
            --no-js)
                INSTALL_JS=false
                shift
                ;;
            --no-python)
                INSTALL_PYTHON=false
                shift
                ;;
            --no-cli)
                INSTALL_CLI=false
                shift
                ;;
            --version)
                VERSION="$2"
                shift 2
                ;;
            --help)
                show_usage
                exit 0
                ;;
            *)
                error "Unknown option: $1"
                ;;
        esac
    done

    log "Starting Citrate SDK installation..."
    echo ""

    # Install components based on configuration
    if [[ "$INSTALL_JS" == "true" ]]; then
        install_nodejs_sdk
        echo ""
    fi

    if [[ "$INSTALL_PYTHON" == "true" ]]; then
        install_python_sdk
        echo ""
    fi

    if [[ "$INSTALL_CLI" == "true" ]]; then
        install_cli_tools
        echo ""
    fi

    log "Installation completed successfully! 🎉"
    echo ""
    echo "Next steps:"
    echo "  1. Read the documentation: https://docs.lattice.ai"
    echo "  2. Join our Discord: https://discord.gg/citrate-ai"
    echo "  3. Start building with Citrate!"
}

# Run main function
main "$@"