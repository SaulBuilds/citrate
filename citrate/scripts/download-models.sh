#!/bin/bash
# Citrate Model Download Script
# Downloads required AI models for first-run setup

set -e

MODELS_DIR="${CITRATE_MODELS_DIR:-$HOME/.citrate/models}"
IPFS_GATEWAY="${IPFS_GATEWAY:-https://ipfs.io/ipfs}"

# Model CIDs
MISTRAL_7B_CID="QmUsYyxg71bV8USRQ6Ccm3SdMqeWgEEVnCYkgNDaxvBTZB"
QWEN2_05B_CID="QmZj4ZaG9v6nXKnT5yqwi8YaH5bm48zooNdh9ff4CHGTY4"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}╔════════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║     Citrate AI Model Download Script       ║${NC}"
echo -e "${GREEN}╚════════════════════════════════════════════╝${NC}"
echo ""

# Create models directory
mkdir -p "$MODELS_DIR"
echo -e "Models directory: ${YELLOW}$MODELS_DIR${NC}"
echo ""

# Function to download from IPFS with fallback gateways
download_from_ipfs() {
    local cid=$1
    local output=$2
    local name=$3

    local gateways=(
        "https://ipfs.io/ipfs"
        "https://cloudflare-ipfs.com/ipfs"
        "https://gateway.pinata.cloud/ipfs"
        "https://dweb.link/ipfs"
    )

    echo -e "Downloading ${YELLOW}$name${NC}..."

    for gateway in "${gateways[@]}"; do
        echo -e "  Trying gateway: $gateway"
        if curl -L --progress-bar --fail "$gateway/$cid" -o "$output" 2>/dev/null; then
            echo -e "  ${GREEN}✓ Downloaded successfully${NC}"
            return 0
        fi
    done

    echo -e "  ${RED}✗ Failed to download from all gateways${NC}"
    return 1
}

# Function to download from HuggingFace
download_from_huggingface() {
    local repo=$1
    local file=$2
    local output=$3
    local name=$4

    local url="https://huggingface.co/$repo/resolve/main/$file"

    echo -e "Downloading ${YELLOW}$name${NC} from HuggingFace..."
    echo -e "  URL: $url"

    if curl -L --progress-bar --fail "$url" -o "$output"; then
        echo -e "  ${GREEN}✓ Downloaded successfully${NC}"
        return 0
    else
        echo -e "  ${RED}✗ Failed to download${NC}"
        return 1
    fi
}

# Check what's already downloaded
check_model() {
    local path=$1
    local min_size=$2  # Minimum size in bytes

    if [[ -f "$path" ]]; then
        local size=$(stat -f%z "$path" 2>/dev/null || stat -c%s "$path" 2>/dev/null)
        if [[ $size -ge $min_size ]]; then
            return 0
        fi
    fi
    return 1
}

echo "Checking existing models..."
echo ""

# Download Mistral 7B Instruct (4.1GB)
MISTRAL_PATH="$MODELS_DIR/mistral-7b-instruct-v0.3.gguf"
if check_model "$MISTRAL_PATH" 4000000000; then
    echo -e "${GREEN}✓ Mistral 7B already downloaded${NC}"
else
    echo -e "${YELLOW}Mistral 7B Instruct v0.3 (~4.1GB)${NC}"
    download_from_ipfs "$MISTRAL_7B_CID" "$MISTRAL_PATH" "Mistral 7B Instruct"
fi
echo ""

# Download Qwen2 0.5B (fast inference, ~400MB)
QWEN2_PATH="$MODELS_DIR/qwen2-0.5b-instruct-q4.gguf"
if check_model "$QWEN2_PATH" 300000000; then
    echo -e "${GREEN}✓ Qwen2 0.5B already downloaded${NC}"
else
    echo -e "${YELLOW}Qwen2 0.5B Instruct Q4 (~400MB)${NC}"
    if [[ -n "$QWEN2_05B_CID" ]]; then
        download_from_ipfs "$QWEN2_05B_CID" "$QWEN2_PATH" "Qwen2 0.5B"
    else
        # Fallback to HuggingFace if no IPFS CID
        download_from_huggingface "Qwen/Qwen2-0.5B-Instruct-GGUF" \
            "qwen2-0_5b-instruct-q4_k_m.gguf" \
            "$QWEN2_PATH" \
            "Qwen2 0.5B"
    fi
fi
echo ""

# Verify downloads
echo -e "${GREEN}╔════════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║           Download Summary                 ║${NC}"
echo -e "${GREEN}╚════════════════════════════════════════════╝${NC}"
echo ""

if [[ -f "$MISTRAL_PATH" ]]; then
    size=$(du -h "$MISTRAL_PATH" | cut -f1)
    echo -e "  ${GREEN}✓${NC} Mistral 7B:  $size"
else
    echo -e "  ${RED}✗${NC} Mistral 7B:  Not downloaded"
fi

if [[ -f "$QWEN2_PATH" ]]; then
    size=$(du -h "$QWEN2_PATH" | cut -f1)
    echo -e "  ${GREEN}✓${NC} Qwen2 0.5B: $size"
else
    echo -e "  ${RED}✗${NC} Qwen2 0.5B: Not downloaded"
fi

echo ""
echo -e "Models stored in: ${YELLOW}$MODELS_DIR${NC}"
echo ""
echo -e "${GREEN}Done!${NC}"
