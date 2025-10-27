FROM rust:1.75 as builder
WORKDIR /app
COPY citrate /app/citrate
WORKDIR /app/citrate
RUN cargo build --release -p node-app

FROM debian:bookworm-slim
RUN useradd -m lattice && mkdir -p /data && chown lattice:lattice /data
USER lattice
WORKDIR /home/lattice
COPY --from=builder /app/citrate/target/release/node-app /usr/local/bin/citrate-node
ENV CITRATE_DATA_DIR=/data
EXPOSE 8545
CMD ["/usr/local/bin/citrate-node"]
