#!/bin/bash
# ============================================================================
# DEEP CLEAN SCRIPT - Aggressive System Cleanup
# ============================================================================
# WARNING: This script aggressively removes caches, build artifacts, and
# temporary files. Everything it removes can be rebuilt/redownloaded.
#
# Usage: ./deep-clean.sh [--dry-run] [--skip-docker] [--skip-homebrew]
#
# Options:
#   --dry-run       Show what would be deleted without deleting
#   --skip-docker   Skip Docker cleanup
#   --skip-homebrew Skip Homebrew cleanup
# ============================================================================

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

DRY_RUN=false
SKIP_DOCKER=false
SKIP_HOMEBREW=false
TOTAL_FREED=0

# Parse arguments
for arg in "$@"; do
    case $arg in
        --dry-run)
            DRY_RUN=true
            echo -e "${YELLOW}=== DRY RUN MODE - No files will be deleted ===${NC}"
            ;;
        --skip-docker)
            SKIP_DOCKER=true
            ;;
        --skip-homebrew)
            SKIP_HOMEBREW=true
            ;;
    esac
done

# Function to get directory size in bytes
get_size() {
    if [ -e "$1" ]; then
        du -sk "$1" 2>/dev/null | cut -f1 || echo 0
    else
        echo 0
    fi
}

# Function to format size
format_size() {
    local size=$1
    if [ $size -ge 1048576 ]; then
        echo "$(echo "scale=2; $size / 1048576" | bc)GB"
    elif [ $size -ge 1024 ]; then
        echo "$(echo "scale=2; $size / 1024" | bc)MB"
    else
        echo "${size}KB"
    fi
}

# Function to safely remove with size tracking
safe_remove() {
    local path="$1"
    local desc="$2"

    if [ -e "$path" ]; then
        local size=$(get_size "$path")
        TOTAL_FREED=$((TOTAL_FREED + size))

        if [ "$DRY_RUN" = true ]; then
            echo -e "${BLUE}[DRY RUN]${NC} Would remove: $path ($(format_size $size))"
        else
            echo -e "${GREEN}Removing:${NC} $path ($(format_size $size))"
            rm -rf "$path" 2>/dev/null || sudo rm -rf "$path" 2>/dev/null || true
        fi
    fi
}

# Function to clean directory contents but keep the directory
safe_clean_dir() {
    local path="$1"
    local desc="$2"

    if [ -d "$path" ]; then
        local size=$(get_size "$path")
        TOTAL_FREED=$((TOTAL_FREED + size))

        if [ "$DRY_RUN" = true ]; then
            echo -e "${BLUE}[DRY RUN]${NC} Would clean: $path/* ($(format_size $size))"
        else
            echo -e "${GREEN}Cleaning:${NC} $path/* ($(format_size $size))"
            rm -rf "$path"/* 2>/dev/null || sudo rm -rf "$path"/* 2>/dev/null || true
        fi
    fi
}

echo -e "${YELLOW}============================================${NC}"
echo -e "${YELLOW}         DEEP SYSTEM CLEANUP               ${NC}"
echo -e "${YELLOW}============================================${NC}"
echo ""

# ============================================================================
# 1. RUST / CARGO CLEANUP
# ============================================================================
echo -e "${YELLOW}=== Rust/Cargo Cleanup ===${NC}"

# Cargo registry cache
safe_remove "$HOME/.cargo/registry/cache" "Cargo registry cache"
safe_remove "$HOME/.cargo/registry/src" "Cargo registry source"
safe_remove "$HOME/.cargo/git/db" "Cargo git db"
safe_remove "$HOME/.cargo/git/checkouts" "Cargo git checkouts"

# Global Rust target directories (find all target dirs)
echo "Searching for Rust target directories..."
if [ "$DRY_RUN" = true ]; then
    find "$HOME" -maxdepth 5 -type d -name "target" -path "*/target" 2>/dev/null | while read dir; do
        if [ -f "$(dirname "$dir")/Cargo.toml" ]; then
            local size=$(get_size "$dir")
            echo -e "${BLUE}[DRY RUN]${NC} Would remove: $dir ($(format_size $size))"
        fi
    done
else
    find "$HOME" -maxdepth 5 -type d -name "target" -path "*/target" 2>/dev/null | while read dir; do
        if [ -f "$(dirname "$dir")/Cargo.toml" ]; then
            safe_remove "$dir" "Rust target"
        fi
    done
fi

# RocksDB libraries specifically - only check known locations
echo "Cleaning known RocksDB locations..."
for rocks_dir in "$HOME/.cargo/registry" "$HOME/Downloads/citrate/citrate/target" "/usr/local/lib" "/opt/homebrew/lib"; do
    if [ -d "$rocks_dir" ]; then
        find "$rocks_dir" -maxdepth 5 -name "librocksdb*" -type f 2>/dev/null | head -20 | while read file; do
            safe_remove "$file" "RocksDB library"
        done
    fi
done

# ============================================================================
# 2. NODE.JS / NPM CLEANUP
# ============================================================================
echo ""
echo -e "${YELLOW}=== Node.js/npm Cleanup ===${NC}"

# npm cache
safe_remove "$HOME/.npm/_cacache" "npm cache"
safe_remove "$HOME/.npm/_logs" "npm logs"

# Yarn cache
safe_remove "$HOME/.yarn/cache" "Yarn cache"
safe_remove "$HOME/.cache/yarn" "Yarn cache (alt)"

# pnpm cache
safe_remove "$HOME/.pnpm-store" "pnpm store"
safe_remove "$HOME/.local/share/pnpm" "pnpm data"

# Bun cache
safe_remove "$HOME/.bun/install/cache" "Bun cache"

# node_modules in Downloads and common project locations
echo "Searching for node_modules directories..."
for dir in "$HOME/Downloads" "$HOME/Projects" "$HOME/Code" "$HOME/Development" "$HOME/dev"; do
    if [ -d "$dir" ]; then
        find "$dir" -maxdepth 4 -type d -name "node_modules" 2>/dev/null | while read nm; do
            safe_remove "$nm" "node_modules"
        done
    fi
done

# ============================================================================
# 3. PYTHON CLEANUP
# ============================================================================
echo ""
echo -e "${YELLOW}=== Python Cleanup ===${NC}"

# pip cache
safe_remove "$HOME/.cache/pip" "pip cache"
safe_remove "$HOME/Library/Caches/pip" "pip cache (macOS)"

# __pycache__ directories
echo "Searching for __pycache__ directories..."
find "$HOME" -maxdepth 6 -type d -name "__pycache__" 2>/dev/null | head -100 | while read dir; do
    safe_remove "$dir" "Python cache"
done

# .pyc files
find "$HOME" -maxdepth 6 -name "*.pyc" -type f 2>/dev/null | head -100 | while read file; do
    safe_remove "$file" "Python bytecode"
done

# Virtual environments (optional - be careful)
# Uncomment to enable:
# safe_remove "$HOME/.virtualenvs" "virtualenvs"

# ============================================================================
# 4. DOCKER CLEANUP
# ============================================================================
if [ "$SKIP_DOCKER" = false ]; then
    echo ""
    echo -e "${YELLOW}=== Docker Cleanup ===${NC}"

    if command -v docker &> /dev/null; then
        if [ "$DRY_RUN" = true ]; then
            echo -e "${BLUE}[DRY RUN]${NC} Would run: docker system prune -a --volumes -f"
            docker system df 2>/dev/null || true
        else
            echo "Stopping all containers..."
            docker stop $(docker ps -aq) 2>/dev/null || true

            echo "Removing all containers..."
            docker rm $(docker ps -aq) 2>/dev/null || true

            echo "Removing all images..."
            docker rmi -f $(docker images -aq) 2>/dev/null || true

            echo "Removing all volumes..."
            docker volume rm $(docker volume ls -q) 2>/dev/null || true

            echo "Docker system prune..."
            docker system prune -a --volumes -f 2>/dev/null || true

            echo "Clearing Docker Desktop data..."
            safe_remove "$HOME/Library/Containers/com.docker.docker/Data/vms" "Docker VMs"
            safe_remove "$HOME/.docker/buildx" "Docker buildx"
        fi
    else
        echo "Docker not installed, skipping..."
    fi
fi

# ============================================================================
# 5. HOMEBREW CLEANUP
# ============================================================================
if [ "$SKIP_HOMEBREW" = false ]; then
    echo ""
    echo -e "${YELLOW}=== Homebrew Cleanup ===${NC}"

    if command -v brew &> /dev/null; then
        if [ "$DRY_RUN" = true ]; then
            echo -e "${BLUE}[DRY RUN]${NC} Would run: brew cleanup --prune=all"
        else
            echo "Cleaning up Homebrew..."
            brew cleanup --prune=all -s 2>/dev/null || true
            brew autoremove 2>/dev/null || true

            # Remove old versions
            safe_remove "$(brew --cache)" "Homebrew cache"
        fi
    else
        echo "Homebrew not installed, skipping..."
    fi
fi

# ============================================================================
# 6. XCODE / iOS CLEANUP (macOS specific)
# ============================================================================
echo ""
echo -e "${YELLOW}=== Xcode/iOS Cleanup ===${NC}"

# DerivedData
safe_remove "$HOME/Library/Developer/Xcode/DerivedData" "Xcode DerivedData"

# iOS Device Support (old simulators)
safe_remove "$HOME/Library/Developer/Xcode/iOS DeviceSupport" "iOS Device Support"

# watchOS Device Support
safe_remove "$HOME/Library/Developer/Xcode/watchOS DeviceSupport" "watchOS Device Support"

# Xcode Archives (old builds)
safe_remove "$HOME/Library/Developer/Xcode/Archives" "Xcode Archives"

# Simulator caches
safe_remove "$HOME/Library/Developer/CoreSimulator/Caches" "Simulator Caches"

# Old simulators
if command -v xcrun &> /dev/null; then
    if [ "$DRY_RUN" = false ]; then
        echo "Deleting unavailable simulators..."
        xcrun simctl delete unavailable 2>/dev/null || true
    fi
fi

# ============================================================================
# 7. GENERAL CACHES
# ============================================================================
echo ""
echo -e "${YELLOW}=== General Caches ===${NC}"

# macOS system caches (safe ones)
safe_remove "$HOME/Library/Caches/com.apple.dt.Xcode" "Xcode cache"
safe_remove "$HOME/Library/Caches/org.swift.swiftpm" "SwiftPM cache"
safe_remove "$HOME/Library/Caches/Google" "Google cache"
safe_remove "$HOME/Library/Caches/com.google.SoftwareUpdate" "Google update cache"
safe_remove "$HOME/Library/Caches/com.spotify.client" "Spotify cache"
safe_remove "$HOME/Library/Caches/com.microsoft.VSCode" "VS Code cache"
safe_remove "$HOME/Library/Caches/com.microsoft.VSCode.ShipIt" "VS Code updates"
safe_remove "$HOME/Library/Caches/com.apple.Safari" "Safari cache"
safe_remove "$HOME/Library/Caches/Firefox" "Firefox cache"

# Electron apps caches
safe_remove "$HOME/Library/Application Support/Code/Cache" "VS Code cache"
safe_remove "$HOME/Library/Application Support/Code/CachedData" "VS Code cached data"
safe_remove "$HOME/Library/Application Support/Code/CachedExtensions" "VS Code cached extensions"
safe_remove "$HOME/Library/Application Support/Slack/Cache" "Slack cache"
safe_remove "$HOME/Library/Application Support/discord/Cache" "Discord cache"

# ============================================================================
# 8. TRASH
# ============================================================================
echo ""
echo -e "${YELLOW}=== Trash Cleanup ===${NC}"

if [ "$DRY_RUN" = true ]; then
    local trash_size=$(get_size "$HOME/.Trash")
    echo -e "${BLUE}[DRY RUN]${NC} Would empty trash ($(format_size $trash_size))"
else
    echo "Emptying trash..."
    rm -rf "$HOME/.Trash"/* 2>/dev/null || true
    # Also clear other volume trashes
    sudo rm -rf /Volumes/*/.Trashes/* 2>/dev/null || true
fi

# ============================================================================
# 9. LOGS
# ============================================================================
echo ""
echo -e "${YELLOW}=== Log Cleanup ===${NC}"

safe_remove "$HOME/Library/Logs" "User logs"
safe_clean_dir "/var/log" "System logs"

# ============================================================================
# 10. AI/ML MODEL CACHES
# ============================================================================
echo ""
echo -e "${YELLOW}=== AI/ML Model Cleanup ===${NC}"

# Hugging Face cache
safe_remove "$HOME/.cache/huggingface" "Hugging Face cache"

# Ollama models
safe_remove "$HOME/.ollama/models" "Ollama models"

# LM Studio models
safe_remove "$HOME/.cache/lm-studio" "LM Studio cache"

# PyTorch cache
safe_remove "$HOME/.cache/torch" "PyTorch cache"

# Transformers cache
safe_remove "$HOME/.cache/transformers" "Transformers cache"

# ============================================================================
# 11. MISC BUILD ARTIFACTS
# ============================================================================
echo ""
echo -e "${YELLOW}=== Build Artifacts ===${NC}"

# Gradle
safe_remove "$HOME/.gradle/caches" "Gradle caches"
safe_remove "$HOME/.gradle/wrapper/dists" "Gradle wrapper"

# Maven
safe_remove "$HOME/.m2/repository" "Maven repository"

# Go
safe_remove "$HOME/go/pkg/mod/cache" "Go mod cache"

# CocoaPods
safe_remove "$HOME/Library/Caches/CocoaPods" "CocoaPods cache"
safe_remove "$HOME/.cocoapods/repos" "CocoaPods repos"

# Foundry (Solidity)
safe_remove "$HOME/.foundry/cache" "Foundry cache"

# ============================================================================
# 12. CITRATE-SPECIFIC CLEANUP
# ============================================================================
echo ""
echo -e "${YELLOW}=== Citrate Project Cleanup ===${NC}"

CITRATE_ROOT="$(dirname "$(dirname "$(realpath "$0")")")"

if [ -d "$CITRATE_ROOT" ]; then
    # Target directories in citrate
    safe_remove "$CITRATE_ROOT/target" "Citrate main target"
    safe_remove "$CITRATE_ROOT/citrate/target" "Citrate workspace target"

    # GUI node_modules
    safe_remove "$CITRATE_ROOT/citrate/gui/citrate-core/node_modules" "GUI node_modules"

    # Explorer node_modules
    safe_remove "$CITRATE_ROOT/citrate/explorer/node_modules" "Explorer node_modules"

    # SDK node_modules
    safe_remove "$CITRATE_ROOT/citrate/sdk/javascript/node_modules" "SDK node_modules"

    # Dev tools
    safe_remove "$CITRATE_ROOT/citrate/developer-tools/lattice-studio/node_modules" "Studio node_modules"
fi

# ============================================================================
# 13. TEMP FILES
# ============================================================================
echo ""
echo -e "${YELLOW}=== Temp Files ===${NC}"

safe_clean_dir "/tmp" "System temp"
safe_clean_dir "$TMPDIR" "User temp"
safe_remove "$HOME/Library/Application Support/CrashReporter" "Crash reports"
safe_remove "$HOME/Library/Caches/CrashReporter" "Crash reporter cache"

# ============================================================================
# SUMMARY
# ============================================================================
echo ""
echo -e "${YELLOW}============================================${NC}"
echo -e "${GREEN}         CLEANUP COMPLETE!                 ${NC}"
echo -e "${YELLOW}============================================${NC}"
echo ""
echo -e "Total space freed: ${GREEN}$(format_size $TOTAL_FREED)${NC}"

if [ "$DRY_RUN" = true ]; then
    echo ""
    echo -e "${YELLOW}This was a dry run. Run without --dry-run to actually delete files.${NC}"
fi

echo ""
echo -e "${YELLOW}Recommended follow-up:${NC}"
echo "  1. Restart your computer to release any locked caches"
echo "  2. Run 'cargo build' in Citrate to rebuild Rust dependencies"
echo "  3. Run 'npm install' in GUI directories to restore node_modules"
echo ""
