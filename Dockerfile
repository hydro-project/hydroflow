# Compilation stage
FROM --platform=$BUILDPLATFORM rust:slim-buster AS build

ARG TARGETOS TARGETARCH

RUN apt-get update

RUN /bin/bash -c "if [ "${TARGETARCH}" == "arm64" ]; then apt-get install -y gcc-aarch64-linux-gnu ; else apt-get install -y gcc-x86-64-linux-gnu ; fi"

WORKDIR /usr/src/myapp
COPY . .

RUN ./scripts/build_dist_release.sh ${TARGETOS} ${TARGETARCH}

RUN mkdir -p xfer/examples
RUN ls -dR target/*/release/examples/* | grep -vE '^.*/[a-z_]+\-.*$' | grep -vE '^.*\.d$' | xargs -I{} cp {} xfer/examples/
RUN ls xfer/examples

# Runtime stage
FROM rust:slim

WORKDIR /usr/src/myapp
COPY --from=build /usr/src/myapp/xfer/examples/* .
