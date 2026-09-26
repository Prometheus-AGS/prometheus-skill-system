# syntax=docker/dockerfile:1.18@sha256:dabfc0969b935b2080555ace70ee69a5261af8a8f1b4df97b9e7fbcf6722eddf
FROM rust:1.96-bookworm@sha256:a339861ae23e9abb272cea45dfafde21760d2ce6577a70f8a926153677902663 AS build
RUN apt-get update && apt-get install -y --no-install-recommends pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*
WORKDIR /src
COPY --from=component-source . ./
RUN cargo build --locked --release
FROM debian:bookworm-slim@sha256:7b140f374b289a7c2befc338f42ebe6441b7ea838a042bbd5acbfca6ec875818
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates curl libssl3 && rm -rf /var/lib/apt/lists/* && useradd --create-home --uid 10001 --shell /usr/sbin/nologin flintgate
COPY --from=build /src/target/release/flint-gate /usr/local/bin/flint-gate
USER 10001:10001
ENV RUST_LOG=info FLINT_GATE_CONFIG=/app/config/config.yaml
EXPOSE 4456 4457
HEALTHCHECK CMD ["curl", "--fail", "--silent", "http://127.0.0.1:4457/health"]
ENTRYPOINT ["/usr/local/bin/flint-gate"]
