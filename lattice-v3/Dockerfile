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
COPY contracts ./contracts

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
# P2P
EXPOSE 30303
# Metrics
EXPOSE 9100

# Volume for blockchain data
VOLUME ["/data"]

# Default environment variables
ENV RUST_LOG=info
ENV LATTICE_DATA_DIR=/data
ENV LATTICE_METRICS=1
ENV LATTICE_METRICS_ADDR=0.0.0.0:9100

# Health check
HEALTHCHECK --interval=30s --timeout=5s --start-period=15s --retries=5 \
    CMD curl -fsS http://localhost:9100/health || exit 1

# Entry point
ENTRYPOINT ["lattice"]

# Default command (can be overridden)
CMD ["devnet"]
