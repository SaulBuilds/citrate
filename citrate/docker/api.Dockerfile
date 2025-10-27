# Dockerfile for Lattice API Server
FROM rust:1.75-slim as builder

# Install system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    ca-certificates \
    build-essential \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy workspace files
COPY . .

# Build the API server binary
RUN cargo build --release --bin lattice-api

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
COPY --from=builder /app/target/release/lattice-api /usr/local/bin/lattice-api

# Create data directory
RUN mkdir -p /data /config && chown -R lattice:lattice /data /config

# Copy default configuration
COPY docker/config/api.toml /config/api.toml

# Switch to lattice user
USER lattice

# Expose ports (REST API, MCP API)
EXPOSE 3000 3001

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:3000/health || exit 1

# Set default command
CMD ["lattice-api", "--config", "/config/api.toml"]