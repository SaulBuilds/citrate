# Lattice V3 Docker Image - Root Context Build
# Multi-stage build for optimal size

# Build stage
FROM rust:latest as builder

WORKDIR /usr/src/lattice

# System build dependencies
RUN apt-get update && apt-get install -y \
    build-essential \
    clang \
    llvm-dev \
    libclang-dev \
    pkg-config \
    cmake \
    git \
    curl \
    zlib1g-dev \
    libssl-dev \
  && rm -rf /var/lib/apt/lists/*

# Copy workspace files from lattice-v3 subdirectory
COPY lattice-v3/Cargo.toml lattice-v3/Cargo.lock ./
COPY lattice-v3/core ./core
COPY lattice-v3/node ./node
COPY lattice-v3/cli ./cli
COPY lattice-v3/contracts ./contracts
COPY lattice-v3/node-app ./node-app
COPY lattice-v3/wallet ./wallet
COPY lattice-v3/faucet ./faucet
COPY lattice-v3/gui/lattice-core/src-tauri ./gui/lattice-core/src-tauri

# Build release binary
RUN cargo build --release -p lattice-node

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create user
RUN useradd -m -u 1000 -s /bin/bash lattice

# Copy binary
COPY --from=builder /usr/src/lattice/target/release/lattice /usr/local/bin/lattice

# Create directories
RUN mkdir -p /app/data && chown -R lattice:lattice /app

USER lattice
WORKDIR /app

# Expose ports
EXPOSE 8545 8546 30303 9100

VOLUME ["/app/data"]

ENV RUST_LOG=info

# Health check
HEALTHCHECK --interval=30s --timeout=5s --start-period=60s --retries=5 \
    CMD curl -f http://localhost:8545/health || exit 1

ENTRYPOINT ["lattice"]
CMD ["devnet"]
