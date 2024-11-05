FROM rustlang/rust:nightly AS builder
WORKDIR /usr/src/gossip-kv-server-base-image
COPY . .

RUN apt-get update && apt-get install -y \
    python3 \
    python3.11-dev \
    libpython3.11 \
    build-essential \
    && rm -rf /var/lib/apt/lists/*

## Build everything, including dependencies. The built dependencies will be cached, so only changing the server
## code requires lesser build time.
RUN cargo build --release --workspace -p gossip_kv

## Copy the built file to where the actual app will be built subsequently.
RUN mkdir -p /usr/src/gossip-kv-server/target/release
RUN mv /usr/src/gossip-kv-server-base-image/target/release/build/ /usr/src/gossip-kv-server/target/release/build/
RUN mv /usr/src/gossip-kv-server-base-image/target/release/deps/ /usr/src/gossip-kv-server/target/release/deps/
RUN mv /usr/src/gossip-kv-server-base-image/target/release/.fingerprint/ /usr/src/gossip-kv-server/target/release/.fingerprint/

## Delete all the source code
RUN rm -rf /usr/src/gossip-kv-server-base-image