# Dockerfile for IPFS Node Integration
FROM ipfs/go-ipfs:latest

# Create custom IPFS user and data directory
USER root
RUN mkdir -p /data/ipfs && chown -R ipfs:ipfs /data/ipfs

# Install additional tools for Lattice integration
RUN apk add --no-cache \
    curl \
    jq \
    wget

# Copy IPFS configuration
COPY docker/config/ipfs-config.json /data/ipfs/config

# Copy custom init script
COPY docker/scripts/ipfs-init.sh /usr/local/bin/ipfs-init.sh
RUN chmod +x /usr/local/bin/ipfs-init.sh

# Switch back to ipfs user
USER ipfs

# Expose IPFS ports
EXPOSE 4001 5001 8080

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=10s --retries=3 \
    CMD wget --no-verbose --tries=1 --spider http://localhost:5001/api/v0/version || exit 1

# Use custom init script
ENTRYPOINT ["/usr/local/bin/ipfs-init.sh"]