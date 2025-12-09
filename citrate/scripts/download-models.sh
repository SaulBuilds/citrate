#!/bin/bash
# Citrate Model Download Script
# Downloads required AI models for enhanced capabilities

set -e

# Platform detection
case "$(uname -s)" in
    Darwin*)    PLATFORM="macos" ;;
    Linux*)     PLATFORM="linux" ;;
    MINGW*|CYGWIN*|MSYS*) PLATFORM="windows" ;;
    *)          PLATFORM="unknown" ;;
esac

# Models directory - OS specific
if [ "$PLATFORM" = "macos" ]; then
    MODELS_DIR="${CITRATE_MODELS_DIR:-$HOME/Library/Application Support/citrate/models}"
elif [ "$PLATFORM" = "windows" ]; then
    MODELS_DIR="${CITRATE_MODELS_DIR:-$APPDATA/citrate/models}"
else
    MODELS_DIR="${CITRATE_MODELS_DIR:-$HOME/.citrate/models}"
fi

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${GREEN}╔════════════════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║       Citrate AI Model Download Script             ║${NC}"
echo -e "${GREEN}╚════════════════════════════════════════════════════╝${NC}"
echo ""

# Create models directory
mkdir -p "$MODELS_DIR"
echo -e "Platform: ${YELLOW}$PLATFORM${NC}"
echo -e "Models directory: ${YELLOW}$MODELS_DIR${NC}"
echo ""

# Model definitions
declare -A MODELS
MODELS["qwen2-0.5b"]="Qwen/Qwen2-0.5B-Instruct-GGUF|qwen2-0_5b-instruct-q4_k_m.gguf|379M|Fast lightweight model (bundled)"
MODELS["qwen2.5-coder-7b"]="Qwen/Qwen2.5-Coder-7B-Instruct-GGUF|qwen2.5-coder-7b-instruct-q4_k_m.gguf|4.4G|Enhanced coding model (recommended)"

# Function to download from HuggingFace with progress
download_model() {
    local key=$1
    local info="${MODELS[$key]}"

    IFS='|' read -r repo file size desc <<< "$info"
    local output="$MODELS_DIR/$file"
    local url="https://huggingface.co/$repo/resolve/main/$file"

    echo -e "${BLUE}[$key]${NC} $desc"
    echo -e "  Size: ${YELLOW}$size${NC}"

    if [[ -f "$output" ]]; then
        local existing_size=$(du -h "$output" 2>/dev/null | cut -f1)
        echo -e "  ${GREEN}✓ Already downloaded${NC} ($existing_size)"
        return 0
    fi

    echo -e "  Downloading from HuggingFace..."
    echo -e "  URL: $url"

    if curl -L --progress-bar --fail "$url" -o "$output"; then
        echo -e "  ${GREEN}✓ Downloaded successfully${NC}"
        return 0
    else
        echo -e "  ${RED}✗ Failed to download${NC}"
        rm -f "$output" 2>/dev/null
        return 1
    fi
}

# Parse arguments
DOWNLOAD_ALL=false
DOWNLOAD_7B=false
DOWNLOAD_05B=false

if [[ $# -eq 0 ]]; then
    echo "Usage: $0 [--all | --7b | --0.5b]"
    echo ""
    echo "Options:"
    echo "  --all   Download all models (~4.8GB total)"
    echo "  --7b    Download Qwen2.5-Coder-7B (~4.4GB) - recommended for best results"
    echo "  --0.5b  Download Qwen2-0.5B (~379MB) - lightweight, already bundled"
    echo ""
    echo "Available models:"
    for key in "${!MODELS[@]}"; do
        IFS='|' read -r repo file size desc <<< "${MODELS[$key]}"
        echo -e "  ${BLUE}$key${NC}: $desc ($size)"
    done
    echo ""
    echo "The 0.5B model is bundled with the app and works immediately."
    echo "For better AI capabilities, download the 7B model with: $0 --7b"
    exit 0
fi

while [[ $# -gt 0 ]]; do
    case $1 in
        --all)
            DOWNLOAD_ALL=true
            shift
            ;;
        --7b)
            DOWNLOAD_7B=true
            shift
            ;;
        --0.5b)
            DOWNLOAD_05B=true
            shift
            ;;
        *)
            echo "Unknown option: $1"
            exit 1
            ;;
    esac
done

echo "Starting downloads..."
echo ""

# Download requested models
if $DOWNLOAD_ALL || $DOWNLOAD_05B; then
    download_model "qwen2-0.5b"
    echo ""
fi

if $DOWNLOAD_ALL || $DOWNLOAD_7B; then
    download_model "qwen2.5-coder-7b"
    echo ""
fi

# Summary
echo -e "${GREEN}╔════════════════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║              Download Summary                       ║${NC}"
echo -e "${GREEN}╚════════════════════════════════════════════════════╝${NC}"
echo ""

for key in "${!MODELS[@]}"; do
    IFS='|' read -r repo file size desc <<< "${MODELS[$key]}"
    local_file="$MODELS_DIR/$file"

    if [[ -f "$local_file" ]]; then
        actual_size=$(du -h "$local_file" 2>/dev/null | cut -f1)
        echo -e "  ${GREEN}✓${NC} $key: $actual_size"
    else
        echo -e "  ${YELLOW}○${NC} $key: Not downloaded"
    fi
done

echo ""
echo -e "Models stored in: ${YELLOW}$MODELS_DIR${NC}"
echo ""
echo -e "${GREEN}Done!${NC}"
echo ""
echo "To use the downloaded model in Citrate:"
echo "1. Open Citrate"
echo "2. Go to Settings > AI Provider"
echo "3. Select 'Local Model' and choose your preferred model"
