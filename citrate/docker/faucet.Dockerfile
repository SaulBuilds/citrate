FROM rust:1.75 as builder
WORKDIR /app
COPY lattice-v3 /app/lattice-v3
RUN apt-get update && apt-get install -y build-essential clang cmake pkg-config && \
    cargo build -p lattice-faucet --release

FROM debian:bookworm-slim
RUN useradd -m lattice
COPY --from=builder /app/target/release/lattice-faucet /usr/local/bin/lattice-faucet
USER lattice
EXPOSE 3001
ENTRYPOINT ["/usr/local/bin/lattice-faucet"]

