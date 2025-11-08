# Building Citrate from Source

This guide explains how to build the Citrate blockchain node from source code.

## Prerequisites

- **Rust 1.75+** - Install via [rustup](https://rustup.rs/)
- **Build essentials** - C compiler, make, etc.
- **Git** - For cloning the repository

### Platform-Specific Requirements

#### macOS
```bash
# Install Xcode Command Line Tools
xcode-select --install

# Install dependencies via Homebrew
brew install rocksdb llvm
```

#### Linux (Ubuntu/Debian)
```bash
sudo apt update
sudo apt install -y build-essential pkg-config libssl-dev librocksdb-dev clang
```

#### Linux (Fedora/RHEL)
```bash
sudo dnf install -y gcc gcc-c++ make openssl-devel rocksdb-devel clang
```

## Quick Build

### Standard Build (Recommended for Contributors)

```bash
# Clone the repository
git clone https://github.com/SaulBuilds/citrate.git
cd citrate/citrate

# Build all workspace packages
cargo build --workspace --release

# Or build specific packages
cargo build -p citrate-node --release
cargo build -p citrate-wallet --release
```

**Build time:** ~5-10 minutes on modern hardware

**Note:** The standard build does **not** embed the genesis AI model. The node will load the genesis block from the blockchain database when it starts.

## Genesis Block Creation (Advanced)

⚠️ **Only needed if you're creating a new genesis block**

If you need to create a fresh genesis block with an embedded AI model:

### Step 1: Download the BGE-M3 Model

```bash
# Create assets directory
mkdir -p node/assets

# Download BGE-M3 Q4 GGUF model (417 MB)
cd node/assets
wget https://huggingface.co/BAAI/bge-m3-gguf/resolve/main/bge-m3-q4.gguf
# Or use curl
curl -L -o bge-m3-q4.gguf https://huggingface.co/BAAI/bge-m3-gguf/resolve/main/bge-m3-q4.gguf

cd ../..
```

### Step 2: Build with Genesis Model Embedding

```bash
# Build with the embed-genesis-model feature flag
cargo build -p citrate-node --release --features embed-genesis-model

# The binary will now include the 417 MB model
ls -lh target/release/citrate
# Should show ~450+ MB binary
```

### Step 3: Create Genesis Block

```bash
# Run genesis creation command
./target/release/citrate create-genesis --output genesis.json

# The genesis block file will contain the embedded model
```

## Build Targets

### All Packages

```bash
# Build entire workspace (recommended)
cargo build --workspace --release
```

### Individual Packages

```bash
# Core blockchain node
cargo build -p citrate-node --release

# CLI wallet
cargo build -p citrate-wallet --release

# Block explorer backend
cargo build -p citrate-explorer --release

# All SDKs
cargo build -p citrate-sdk --release
```

### GUI Application

```bash
# Navigate to GUI directory
cd gui/citrate-core

# Install Node.js dependencies
npm install

# Build Tauri desktop app
npm run tauri build

# Output: gui/citrate-core/src-tauri/target/release/bundle/
```

## Build Profiles

### Development Build
```bash
# Faster compilation, larger binary, no optimizations
cargo build --workspace

# Binary location: target/debug/citrate
```

### Release Build
```bash
# Slower compilation, optimized for performance
cargo build --workspace --release

# Binary location: target/release/citrate
# ~10x faster runtime performance
```

### Production Build
```bash
# Maximum optimizations, LTO, thin binaries
RUSTFLAGS="-C target-cpu=native" cargo build --workspace --release

# Additional size reduction
strip target/release/citrate
```

## Feature Flags

Citrate uses Cargo feature flags for optional functionality:

| Feature | Package | Description | Default |
|---------|---------|-------------|---------|
| `embed-genesis-model` | `citrate-node` | Embed 417 MB AI model in binary | ❌ Off |
| `devnet` | `citrate-node` | Enable devnet-specific features | ❌ Off |

### Using Feature Flags

```bash
# Enable specific feature
cargo build -p citrate-node --features embed-genesis-model

# Enable multiple features
cargo build -p citrate-node --features "devnet,embed-genesis-model"

# Disable default features
cargo build -p citrate-node --no-default-features
```

## Build Configuration

### Cargo.toml Workspace

The project uses a Cargo workspace with the following structure:

```
citrate/
├── Cargo.toml              # Workspace root
├── node/                   # Main blockchain node
├── core/
│   ├── consensus/          # GhostDAG consensus
│   ├── execution/          # LVM execution engine
│   ├── storage/            # State and block storage
│   ├── network/            # P2P networking
│   ├── api/                # JSON-RPC API
│   └── mcp/                # Model Context Protocol
├── gui/citrate-core/       # Tauri GUI application
└── wallet/                 # CLI wallet
```

### Environment Variables

Control build behavior with environment variables:

```bash
# Use native CPU instructions for maximum performance
export RUSTFLAGS="-C target-cpu=native"

# Enable link-time optimization
export CARGO_PROFILE_RELEASE_LTO=true

# Set custom llama.cpp path (for AI inference)
export LLAMA_CPP_PATH="$HOME/llama.cpp"

# Increase parallel build jobs
export CARGO_BUILD_JOBS=16
```

## Troubleshooting

### Common Build Errors

#### Missing RocksDB

```
error: failed to run custom build command for `librocksdb-sys`
```

**Solution:**
```bash
# macOS
brew install rocksdb

# Ubuntu/Debian
sudo apt install librocksdb-dev

# Fedora
sudo dnf install rocksdb-devel
```

#### Missing OpenSSL

```
error: failed to run custom build command for `openssl-sys`
```

**Solution:**
```bash
# macOS
brew install openssl
export OPENSSL_DIR=/usr/local/opt/openssl

# Linux
sudo apt install libssl-dev  # or openssl-devel
```

#### Out of Memory

```
error: could not compile `citrate-node` due to previous error
signal: 9, SIGKILL: kill
```

**Solution:**
```bash
# Reduce parallel jobs
export CARGO_BUILD_JOBS=2

# Or build sequentially
cargo build -p citrate-consensus --release
cargo build -p citrate-execution --release
cargo build -p citrate-node --release
```

#### Linker Errors on Linux

```
error: linking with `cc` failed
```

**Solution:**
```bash
# Install build essentials
sudo apt install build-essential pkg-config

# Or use lld linker (faster)
sudo apt install lld
export RUSTFLAGS="-C link-arg=-fuse-ld=lld"
```

### Build Performance Tips

1. **Use Release Mode for Production**
   ```bash
   cargo build --release  # Much faster at runtime
   ```

2. **Enable Parallel Compilation**
   ```bash
   export CARGO_BUILD_JOBS=$(nproc)  # Linux
   export CARGO_BUILD_JOBS=$(sysctl -n hw.ncpu)  # macOS
   ```

3. **Use Cargo Cache**
   ```bash
   # sccache speeds up rebuilds
   cargo install sccache
   export RUSTC_WRAPPER=sccache
   ```

4. **Incremental Compilation**
   ```bash
   # Already enabled by default for dev builds
   export CARGO_INCREMENTAL=1
   ```

## Cross-Compilation

### Build for Different Platforms

```bash
# Install cross-compilation target
rustup target add x86_64-unknown-linux-gnu
rustup target add aarch64-apple-darwin

# Build for target
cargo build --target x86_64-unknown-linux-gnu --release
cargo build --target aarch64-apple-darwin --release
```

### Using Cross

```bash
# Install cross tool
cargo install cross

# Build for Linux from macOS
cross build --target x86_64-unknown-linux-gnu --release

# Build for ARM
cross build --target aarch64-unknown-linux-gnu --release
```

## Verification

### Verify Build Success

```bash
# Check binary exists and runs
./target/release/citrate --version

# Expected output:
# citrate 0.1.0

# Test node startup
./target/release/citrate --help
```

### Run Tests

```bash
# Run all tests
cargo test --workspace

# Run specific package tests
cargo test -p citrate-consensus
cargo test -p citrate-execution

# Run with output
cargo test -- --nocapture
```

## Next Steps

After building:
- [Setup Development Environment](./installation.md)
- [Configure AI Inference](./ai-inference-setup.md)
- [Run a Local Devnet](../DEVNET_QUICKSTART.md)
- [Deploy to Production](./deployment.md)

## Support

- **Documentation:** [docs.citrate.ai](https://docs.citrate.ai)
- **Issues:** [github.com/SaulBuilds/citrate/issues](https://github.com/SaulBuilds/citrate/issues)
- **Discord:** [discord.gg/citrate](https://discord.gg/citrate)
