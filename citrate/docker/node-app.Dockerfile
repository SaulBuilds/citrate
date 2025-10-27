FROM rust:1.75 as builder
WORKDIR /app
COPY lattice-v3 /app/lattice-v3
RUN apt-get update && apt-get install -y build-essential clang cmake pkg-config && \
    cargo build -p node-app --release

FROM debian:bookworm-slim
RUN useradd -m lattice
COPY --from=builder /app/target/release/node-app /usr/local/bin/node-app
USER lattice
EXPOSE 8545 3000
ENTRYPOINT ["/usr/local/bin/node-app"]

