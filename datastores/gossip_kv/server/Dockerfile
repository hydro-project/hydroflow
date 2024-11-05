FROM "hydroflow-gossip-kv-base-image:latest" AS builder
WORKDIR /usr/src/gossip-kv-server
COPY . .
RUN find .
RUN cargo build --release --workspace -p gossip_kv

FROM rustlang/rust:nightly-slim
COPY --from=builder /usr/src/gossip-kv-server/target/release/gossip_server /usr/local/bin/gossip_server

RUN  mkdir -p /config/static
# Don't skip the trailing slash in the destination directory
COPY datastores/gossip_kv/server/config/static/*.toml /config/static/
CMD ["gossip_server"]
