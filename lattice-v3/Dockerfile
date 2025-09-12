# Lattice V3 Docker Image
# Multi-stage build for optimal size

# Build stage
FROM rust:1.75 as builder

WORKDIR /usr/src/lattice

# Copy workspace files
COPY Cargo.toml Cargo.lock ./
COPY core ./core
COPY node ./node
COPY cli ./cli

# Build release binary
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create user for running the node
RUN useradd -m -u 1000 -s /bin/bash lattice

# Copy binary from builder
COPY --from=builder /usr/src/lattice/target/release/lattice /usr/local/bin/lattice

# Create data directories
RUN mkdir -p /data/chain /data/state /data/models /data/logs && \
    chown -R lattice:lattice /data

# Switch to non-root user
USER lattice

# Expose ports
# JSON-RPC
EXPOSE 8545
# WebSocket
EXPOSE 8546
# REST API
EXPOSE 3000
# P2P
EXPOSE 30303

# Volume for blockchain data
VOLUME ["/data"]

# Default environment variables
ENV RUST_LOG=info
ENV LATTICE_DATA_DIR=/data
ENV LATTICE_NETWORK=mainnet

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD lattice status || exit 1

# Entry point
ENTRYPOINT ["lattice"]

# Default command (can be overridden)
CMD ["--data-dir", "/data", \
     "--rpc-port", "8545", \
     "--ws-port", "8546", \
     "--rest-port", "3000", \
     "--p2p-port", "30303", \
     "--rpc-host", "0.0.0.0", \
     "--ws-host", "0.0.0.0", \
     "--rest-host", "0.0.0.0"]