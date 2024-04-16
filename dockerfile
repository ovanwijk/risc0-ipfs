FROM rust:1.72.1 as builder

WORKDIR /ipfs-content-proof

COPY ./ipfs-content-proof/Cargo.lock ./Cargo.lock
COPY ./ipfs-content-proof/Cargo.toml ./Cargo.toml

COPY ./ipfs-content-proof/ipfs_core ./ipfs_core
COPY ./ipfs-content-proof/ipfs_host ./ipfs_host
COPY ./ipfs-content-proof/methods ./methods
COPY ./ipfs-content-proof/src ./src
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    libssl-dev \
    ca-certificates \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*
RUN apt-get update && apt-get install -y protobuf-compiler
RUN cargo install cargo-risczero
RUN cargo risczero install
RUN cargo build && ls /ipfs-content-proof/target/debug/

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y openssl ca-certificates && apt clean && rm -rf /var/lib/apt/lists/*

COPY --from=builder /ipfs-content-proof/target/debug/ipfs_content_proof .

CMD ["./ipfs_content_proof"]
