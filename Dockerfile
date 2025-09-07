FROM rust:1.75 as builder
WORKDIR /app
COPY lattice-v3 /app/lattice-v3
WORKDIR /app/lattice-v3
RUN cargo build --release -p node-app

FROM debian:bookworm-slim
RUN useradd -m lattice && mkdir -p /data && chown lattice:lattice /data
USER lattice
WORKDIR /home/lattice
COPY --from=builder /app/lattice-v3/target/release/node-app /usr/local/bin/lattice-node
ENV LATTICE_DATA_DIR=/data
EXPOSE 8545
CMD ["/usr/local/bin/lattice-node"]
