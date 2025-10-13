# Lattice V3 - Complete Deployment Guide

This guide covers deploying the entire Lattice infrastructure on Render, Vercel, and setting up public SDK/CLI distribution.

## Table of Contents

1. [Infrastructure Overview](#infrastructure-overview)
2. [Render Core Infrastructure](#render-core-infrastructure)
3. [Frontend Deployments](#frontend-deployments)
4. [SDK & CLI Publishing](#sdk--cli-publishing)
5. [Domain & DNS Setup](#domain--dns-setup)
6. [Monitoring & Maintenance](#monitoring--maintenance)
7. [CI/CD Automation](#cicd-automation)

## Infrastructure Overview

### Architecture

```mermaid
graph TB
    subgraph "Frontend (Vercel)"
        Landing[Landing Page]
        Explorer[Block Explorer]
        Docs[Documentation]
        Dashboard[Dashboard]
    end

    subgraph "Core Infrastructure (Render)"
        RPC[RPC Node]
        Validator[Validator Node]
        IPFS[IPFS Node]
        API[REST API]
        Indexer[Block Indexer]
        Faucet[Testnet Faucet]
    end

    subgraph "Databases (Render)"
        PostgreSQL[(PostgreSQL)]
        Redis[(Redis)]
    end

    subgraph "CDN & Distribution"
        NPM[NPM Registry]
        PyPI[PyPI Registry]
        GitHub[GitHub Releases]
        Homebrew[Homebrew Tap]
    end

    Landing --> RPC
    Explorer --> RPC
    Explorer --> API
    Dashboard --> API

    RPC --> Validator
    Validator --> IPFS
    API --> PostgreSQL
    Indexer --> PostgreSQL
    Faucet --> Redis

    NPM --> SDK[JavaScript SDK]
    PyPI --> PythonSDK[Python SDK]
    GitHub --> CLI[CLI Binaries]
    Homebrew --> CLI
```

### Service Dependencies

| Service | Type | Dependencies | Resources |
|---------|------|-------------|-----------|
| **RPC Node** | Web Service | IPFS, PostgreSQL | 2GB RAM, 2 CPU |
| **Validator Node** | Background Worker | RPC Node | 4GB RAM, 2 CPU |
| **IPFS Node** | Web Service | - | 2GB RAM, 1 CPU |
| **REST API** | Web Service | PostgreSQL, Redis | 1GB RAM, 1 CPU |
| **Block Indexer** | Background Worker | PostgreSQL, RPC Node | 2GB RAM, 1 CPU |
| **Testnet Faucet** | Web Service | Redis, RPC Node | 512MB RAM, 1 CPU |

## Render Core Infrastructure

### 1. PostgreSQL Database

Create a new PostgreSQL database on Render:

```yaml
# render-postgres.yaml
name: lattice-postgres
plan: standard
region: oregon
version: "15"
databases:
  - name: lattice_maindb
    user: lattice_user
```

**Environment Variables:**
```env
DATABASE_URL=postgresql://lattice_user:password@hostname/lattice_maindb
POSTGRES_PASSWORD=your_secure_password
POSTGRES_USER=lattice_user
POSTGRES_DB=lattice_maindb
```

### 2. Redis Cache

```yaml
# render-redis.yaml
name: lattice-redis
plan: standard
region: oregon
version: "7"
maxmemory_policy: allkeys-lru
```

### 3. IPFS Node

Create `ipfs.Dockerfile`:

```dockerfile
FROM ipfs/kubo:v0.24.0

# Configure IPFS
RUN ipfs init --profile server
RUN ipfs config Addresses.API /ip4/0.0.0.0/tcp/5001
RUN ipfs config Addresses.Gateway /ip4/0.0.0.0/tcp/8080
RUN ipfs config --json API.HTTPHeaders.Access-Control-Allow-Origin '["*"]'
RUN ipfs config --json API.HTTPHeaders.Access-Control-Allow-Methods '["GET", "POST", "PUT", "DELETE"]'
RUN ipfs config --json API.HTTPHeaders.Access-Control-Allow-Headers '["Authorization", "Content-Type"]'

# Optimize for server deployment
RUN ipfs config Datastore.StorageMax 50GB
RUN ipfs config --json Swarm.ConnMgr.HighWater 2000
RUN ipfs config --json Swarm.ConnMgr.LowWater 500

EXPOSE 4001 5001 8080

CMD ["ipfs", "daemon", "--enable-pubsub-experiment", "--enable-namesys-pubsub"]
```

**Render Service Config:**
```yaml
# render-ipfs.yaml
name: lattice-ipfs
type: web
env: docker
dockerfilePath: ./ipfs.Dockerfile
plan: standard
region: oregon
healthCheckPath: /api/v0/version
envVars:
  - key: IPFS_PROFILE
    value: server
  - key: IPFS_PATH
    value: /data/ipfs
disk:
  name: ipfs-data
  mountPath: /data/ipfs
  sizeGB: 50
```

### 4. RPC Node (Core Blockchain)

Create `node.Dockerfile`:

```dockerfile
FROM rust:1.75-slim as builder

# Install system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    build-essential \
    curl \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY . .

# Build optimized release
ENV CARGO_TARGET_DIR=/app/target
RUN cargo build --release --bin lattice-node

FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create lattice user
RUN useradd -m -s /bin/bash lattice

# Copy binary
COPY --from=builder /app/target/release/lattice-node /usr/local/bin/
RUN chmod +x /usr/local/bin/lattice-node

# Create data directory
RUN mkdir -p /data && chown lattice:lattice /data

USER lattice
WORKDIR /data

EXPOSE 8545 30303

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=60s --retries=3 \
  CMD curl -f http://localhost:8545 -X POST \
    -H "Content-Type: application/json" \
    -d '{"jsonrpc":"2.0","method":"net_version","params":[],"id":1}' || exit 1

CMD ["lattice-node", \
     "--rpc-addr", "0.0.0.0:8545", \
     "--p2p-addr", "0.0.0.0:30303", \
     "--data-dir", "/data", \
     "--chain-id", "1337", \
     "--max-peers", "50", \
     "--cache-size", "2GB"]
```

**Render Service Config:**
```yaml
# render-rpc-node.yaml
name: lattice-rpc-node
type: web
env: docker
dockerfilePath: ./node.Dockerfile
plan: standard_plus
region: oregon
healthCheckPath: /
envVars:
  - key: LATTICE_CHAIN_ID
    value: "1337"
  - key: LATTICE_NETWORK_NAME
    value: "testnet"
  - key: RUST_LOG
    value: "info"
  - key: DATABASE_URL
    fromDatabase:
      name: lattice-postgres
      property: connectionString
  - key: IPFS_API_URL
    fromService:
      name: lattice-ipfs
      type: web
      property: host
      envVarKey: IPFS_API_URL
disk:
  name: blockchain-data
  mountPath: /data
  sizeGB: 100
```

## SDK & CLI Publishing

### 1. JavaScript/TypeScript SDK Setup

Create the package structure and publishing configuration:

```bash
# Create SDK directory structure
mkdir -p sdk/javascript/src
cd sdk/javascript
```

Create `package.json`:

```json
{
  "name": "@lattice-ai/sdk",
  "version": "1.0.0",
  "description": "Official JavaScript SDK for Lattice AI blockchain",
  "main": "dist/index.js",
  "types": "dist/index.d.ts",
  "files": [
    "dist/**/*",
    "README.md",
    "LICENSE"
  ],
  "scripts": {
    "build": "tsc && rollup -c",
    "test": "jest",
    "lint": "eslint src/**/*.ts",
    "prepublishOnly": "npm run build && npm run test"
  },
  "keywords": [
    "lattice",
    "blockchain",
    "ai",
    "sdk",
    "web3",
    "machine-learning"
  ],
  "author": "Lattice Team <developers@lattice.ai>",
  "license": "MIT",
  "homepage": "https://lattice.ai",
  "repository": {
    "type": "git",
    "url": "https://github.com/lattice-ai/lattice-v3.git",
    "directory": "sdk/javascript"
  },
  "publishConfig": {
    "access": "public",
    "registry": "https://registry.npmjs.org/"
  },
  "dependencies": {
    "web3": "^4.0.0",
    "ethers": "^6.0.0",
    "axios": "^1.6.0"
  },
  "devDependencies": {
    "typescript": "^5.0.0",
    "rollup": "^4.0.0",
    "@rollup/plugin-typescript": "^11.0.0",
    "jest": "^29.0.0",
    "eslint": "^8.0.0"
  }
}
```

### 2. Python SDK Setup

Create the Python package structure:

```bash
mkdir -p sdks/python/lattice_sdk
cd sdks/python
```

Create `setup.py`:

```python
from setuptools import setup, find_packages

with open("README.md", "r", encoding="utf-8") as fh:
    long_description = fh.read()

setup(
    name="lattice-sdk",
    version="1.0.0",
    author="Lattice Team",
    author_email="developers@lattice.ai",
    description="Official Python SDK for Lattice AI blockchain",
    long_description=long_description,
    long_description_content_type="text/markdown",
    url="https://github.com/lattice-ai/lattice-v3",
    packages=find_packages(),
    classifiers=[
        "Development Status :: 5 - Production/Stable",
        "Intended Audience :: Developers",
        "License :: OSI Approved :: MIT License",
        "Programming Language :: Python :: 3",
        "Programming Language :: Python :: 3.8",
        "Programming Language :: Python :: 3.9",
        "Programming Language :: Python :: 3.10",
        "Programming Language :: Python :: 3.11",
        "Programming Language :: Python :: 3.12",
    ],
    python_requires=">=3.8",
    install_requires=[
        "web3>=6.0.0",
        "aiohttp>=3.8.0",
        "pydantic>=2.0.0",
        "httpx>=0.24.0",
    ],
    entry_points={
        "console_scripts": [
            "lattice-cli=lattice_sdk.cli:main",
        ],
    },
)
```

### 3. CLI Publishing Setup

Create GitHub Actions workflow for automated releases:

```bash
mkdir -p .github/workflows
```

Create `.github/workflows/publish-releases.yml`:

```yaml
name: Publish SDK and CLI Releases

on:
  push:
    tags:
      - 'v*'
  workflow_dispatch:
    inputs:
      version:
        description: 'Version to publish'
        required: true
        type: string

jobs:
  publish-npm:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '18'
          registry-url: 'https://registry.npmjs.org'

      - name: Install and build JavaScript SDK
        working-directory: sdk/javascript
        run: |
          npm ci
          npm run build
          npm test

      - name: Publish to NPM
        working-directory: sdk/javascript
        run: npm publish
        env:
          NODE_AUTH_TOKEN: ${{ secrets.NPM_TOKEN }}

  publish-pypi:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Set up Python
        uses: actions/setup-python@v4
        with:
          python-version: '3.11'

      - name: Install build dependencies
        run: |
          python -m pip install --upgrade pip
          pip install build twine

      - name: Build Python package
        working-directory: sdks/python
        run: python -m build

      - name: Publish to PyPI
        working-directory: sdks/python
        env:
          TWINE_USERNAME: __token__
          TWINE_PASSWORD: ${{ secrets.PYPI_TOKEN }}
        run: twine upload dist/*

  build-cli:
    strategy:
      matrix:
        include:
          - os: ubuntu-latest
            target: x86_64-unknown-linux-gnu
            name: lattice-cli-linux-amd64
          - os: ubuntu-latest
            target: aarch64-unknown-linux-gnu
            name: lattice-cli-linux-arm64
          - os: macos-latest
            target: x86_64-apple-darwin
            name: lattice-cli-macos-amd64
          - os: macos-latest
            target: aarch64-apple-darwin
            name: lattice-cli-macos-arm64
          - os: windows-latest
            target: x86_64-pc-windows-msvc
            name: lattice-cli-windows-amd64.exe

    runs-on: ${{ matrix.os }}

    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}

      - name: Build CLI
        run: |
          cargo build --release --target ${{ matrix.target }} --bin lattice-cli

      - name: Prepare binary
        shell: bash
        run: |
          if [[ "${{ matrix.os }}" == "windows-latest" ]]; then
            cp target/${{ matrix.target }}/release/lattice-cli.exe ${{ matrix.name }}
          else
            cp target/${{ matrix.target }}/release/lattice-cli ${{ matrix.name }}
            chmod +x ${{ matrix.name }}
          fi

      - name: Upload artifact
        uses: actions/upload-artifact@v4
        with:
          name: ${{ matrix.name }}
          path: ${{ matrix.name }}

  release-cli:
    needs: build-cli
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Download all artifacts
        uses: actions/download-artifact@v4

      - name: Create checksums
        run: |
          find . -name "lattice-cli-*" -type f -exec sha256sum {} \; > SHA256SUMS

      - name: Create GitHub Release
        uses: softprops/action-gh-release@v1
        with:
          files: |
            lattice-cli-*/lattice-cli-*
            SHA256SUMS
          body: |
            ## Lattice CLI and SDK Release

            ### JavaScript SDK
            ```bash
            npm install -g @lattice-ai/sdk
            ```

            ### Python SDK
            ```bash
            pip install lattice-sdk
            ```

            ### CLI Installation
            Download the appropriate binary for your platform and add to PATH.

            **macOS/Linux:**
            ```bash
            curl -L https://github.com/lattice-ai/lattice-v3/releases/latest/download/lattice-cli-{platform}-{arch} -o lattice-cli
            chmod +x lattice-cli
            sudo mv lattice-cli /usr/local/bin/
            ```
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
```

### 4. Homebrew Tap Setup

Create a separate repository for Homebrew tap:

1. Create repository: `lattice-ai/homebrew-tap`
2. Add formula file `Formula/lattice-cli.rb`:

```ruby
class LatticeCli < Formula
  desc "Command-line interface for Lattice AI blockchain"
  homepage "https://lattice.ai"
  version "1.0.0"
  license "MIT"

  if OS.mac? && Hardware::CPU.intel?
    url "https://github.com/lattice-ai/lattice-v3/releases/download/v#{version}/lattice-cli-macos-amd64"
    sha256 "REPLACE_WITH_ACTUAL_SHA256"
  elsif OS.mac? && Hardware::CPU.arm?
    url "https://github.com/lattice-ai/lattice-v3/releases/download/v#{version}/lattice-cli-macos-arm64"
    sha256 "REPLACE_WITH_ACTUAL_SHA256"
  elsif OS.linux? && Hardware::CPU.intel?
    url "https://github.com/lattice-ai/lattice-v3/releases/download/v#{version}/lattice-cli-linux-amd64"
    sha256 "REPLACE_WITH_ACTUAL_SHA256"
  elsif OS.linux? && Hardware::CPU.arm?
    url "https://github.com/lattice-ai/lattice-v3/releases/download/v#{version}/lattice-cli-linux-arm64"
    sha256 "REPLACE_WITH_ACTUAL_SHA256"
  end

  def install
    bin.install "lattice-cli-#{OS.mac? ? "macos" : "linux"}-#{Hardware::CPU.intel? ? "amd64" : "arm64"}" => "lattice-cli"
  end

  test do
    system "#{bin}/lattice-cli", "--version"
  end
end
```

## Frontend Deployments

### 1. Block Explorer (Vercel)

Create `explorer/vercel.json`:

```json
{
  "framework": "nextjs",
  "buildCommand": "npm run build",
  "outputDirectory": ".next",
  "env": {
    "NEXT_PUBLIC_RPC_URL": "https://lattice-rpc-node.onrender.com",
    "NEXT_PUBLIC_API_URL": "https://lattice-api.onrender.com",
    "NEXT_PUBLIC_IPFS_GATEWAY": "https://lattice-ipfs.onrender.com",
    "NEXT_PUBLIC_CHAIN_ID": "1337"
  },
  "rewrites": [
    {
      "source": "/api/proxy/:path*",
      "destination": "https://lattice-api.onrender.com/:path*"
    }
  ]
}
```

Deploy command:
```bash
cd explorer
npm ci
npm run build
vercel --prod
```

### 2. Landing Page (Vercel)

Create landing page with proper environment configuration and deploy similarly to the explorer.

## Quick Deployment Script

Create `scripts/deploy-all.sh`:

```bash
#!/bin/bash

set -e

echo "🚀 Starting Lattice V3 Complete Deployment"

# Check prerequisites
command -v docker >/dev/null 2>&1 || { echo "Docker required but not installed. Aborting." >&2; exit 1; }
command -v npm >/dev/null 2>&1 || { echo "Node.js/npm required but not installed. Aborting." >&2; exit 1; }
command -v cargo >/dev/null 2>&1 || { echo "Rust/Cargo required but not installed. Aborting." >&2; exit 1; }

echo "✅ Prerequisites check passed"

# Build all components
echo "🔨 Building Rust components..."
cargo build --release

echo "🔨 Building JavaScript SDK..."
cd sdk/javascript
npm ci && npm run build
cd ../..

echo "🔨 Building Python SDK..."
cd sdks/python
pip install build
python -m build
cd ../..

echo "🔨 Building frontend applications..."
cd explorer
npm ci && npm run build
cd ../gui/lattice-core
npm ci && npm run tauri build
cd ../..

echo "📦 Creating deployment packages..."

# Create Docker images for Render
docker build -f node.Dockerfile -t lattice-node .
docker build -f api.Dockerfile -t lattice-api .
docker build -f ipfs.Dockerfile -t lattice-ipfs .

echo "✅ All components built successfully!"
echo ""
echo "Next steps:"
echo "1. Push Docker images to a registry (Docker Hub, etc.)"
echo "2. Deploy services on Render using the provided YAML configs"
echo "3. Deploy frontends on Vercel"
echo "4. Publish SDKs using the GitHub Actions workflow"
echo ""
echo "See DEPLOYMENT.md for detailed instructions."
```

Make it executable:
```bash
chmod +x scripts/deploy-all.sh
```

## Publishing Instructions

### 1. Set up NPM Token
1. Create account on npmjs.com
2. Generate access token: `npm token create`
3. Add `NPM_TOKEN` to GitHub repository secrets

### 2. Set up PyPI Token
1. Create account on pypi.org
2. Generate API token in account settings
3. Add `PYPI_TOKEN` to GitHub repository secrets

### 3. Create First Release
```bash
# Tag and push release
git tag v1.0.0
git push origin v1.0.0

# This will trigger the GitHub Actions workflow
# which will automatically publish to NPM, PyPI, and GitHub Releases
```

### 4. Install Published Packages

Users can then install your packages:

```bash
# JavaScript SDK
npm install -g @lattice-ai/sdk

# Python SDK
pip install lattice-sdk

# CLI via Homebrew (macOS)
brew tap lattice-ai/homebrew-tap
brew install lattice-cli

# CLI direct download
curl -L https://github.com/lattice-ai/lattice-v3/releases/latest/download/lattice-cli-linux-amd64 -o lattice-cli
chmod +x lattice-cli
sudo mv lattice-cli /usr/local/bin/
```

This complete deployment guide provides everything needed to deploy the Lattice infrastructure on Render and distribute the SDKs/CLI publicly with automated publishing workflows.