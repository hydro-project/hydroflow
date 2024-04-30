FROM "hydroflow/gossip-kv-server-base-image:latest" as builder
WORKDIR /usr/src/gossip-kv-server
COPY . .
RUN find .
RUN cargo build --release --workspace -p gossip_kv

FROM rustlang/rust:nightly-slim
# RUN apt-get update && apt-get install -y extra-runtime-dependencies && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/src/gossip-kv-server/target/release/gossip_server /usr/local/bin/gossip_server
CMD ["gossip_server"]