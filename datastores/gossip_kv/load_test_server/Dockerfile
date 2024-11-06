FROM "hydroflow-gossip-kv-base-image:latest" AS builder
WORKDIR /usr/src/gossip-kv-server
COPY . .
RUN find .
RUN cargo build --release --workspace -p gossip_kv

FROM rustlang/rust:nightly-slim
COPY --from=builder /usr/src/gossip-kv-server/target/release/load_test_server /usr/local/bin/load_test_server

# Don't skip the trailing slash in the destination directory
CMD ["load_test_server"]
