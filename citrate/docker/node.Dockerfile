# Dockerfile for Lattice Node
FROM rust:1.75-slim as builder

# Install system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    ca-certificates \
    build-essential \
    clang \
    cmake \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy workspace files
COPY . .

# Build the node binary
RUN cargo build --release --bin lattice-node

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    jq \
    && rm -rf /var/lib/apt/lists/*

# Create lattice user
RUN useradd -m -u 1000 lattice

# Copy binary from builder
COPY --from=builder /app/target/release/lattice-node /usr/local/bin/lattice-node

# Create data directory
RUN mkdir -p /data /config && chown -R lattice:lattice /data /config

# Copy default configuration if it exists
COPY docker/config/node.toml /config/node.toml

# Switch to lattice user
USER lattice

# Expose ports (JSON-RPC, WebSocket, P2P)
EXPOSE 8545 8546 30303

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8545 || exit 1

# Set default command
CMD ["lattice-node", "--config", "/config/node.toml", "--data-dir", "/data"]

