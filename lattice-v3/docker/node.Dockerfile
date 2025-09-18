FROM rust:1.75 as builder
WORKDIR /app
COPY lattice-v3 /app/lattice-v3
RUN apt-get update && apt-get install -y build-essential clang cmake pkg-config && \
    cargo build -p lattice-node --release

FROM debian:bookworm-slim
RUN useradd -m lattice
COPY --from=builder /app/target/release/lattice /usr/local/bin/lattice
USER lattice
EXPOSE 8545 30303
ENTRYPOINT ["/usr/local/bin/lattice"]

