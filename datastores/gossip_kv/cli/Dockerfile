FROM "hydroflow-gossip-kv-base-image:latest" AS builder
WORKDIR /usr/src/gossip-kv-server
COPY . .
RUN find .
RUN cargo build --release --workspace -p gossip_kv

FROM rustlang/rust:nightly-slim
COPY --from=builder /usr/src/gossip-kv-server/target/release/gossip_cli /usr/local/bin/gossip_cli

RUN apt-get update && apt-get install -y \
    dnsutils \
    && rm -rf /var/lib/apt/lists/*
