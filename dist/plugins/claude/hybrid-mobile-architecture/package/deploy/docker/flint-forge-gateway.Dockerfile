# syntax=docker/dockerfile:1.18@sha256:dabfc0969b935b2080555ace70ee69a5261af8a8f1b4df97b9e7fbcf6722eddf
FROM rust:1.96-bookworm@sha256:a339861ae23e9abb272cea45dfafde21760d2ce6577a70f8a926153677902663 AS chef
RUN cargo install cargo-chef --locked
WORKDIR /src
FROM chef AS planner
COPY --from=component-source . ./
RUN cargo chef prepare --recipe-path recipe.json
FROM chef AS build
COPY --from=planner /src/recipe.json ./recipe.json
RUN apt-get update && apt-get install -y --no-install-recommends pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*
RUN cargo chef cook --release --recipe-path recipe.json
COPY --from=component-source . ./
RUN cargo build --locked --release -p fdb-gateway
FROM debian:bookworm-slim@sha256:7b140f374b289a7c2befc338f42ebe6441b7ea838a042bbd5acbfca6ec875818
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates curl libssl3 && rm -rf /var/lib/apt/lists/* && useradd --create-home --uid 10001 --shell /usr/sbin/nologin forge
COPY --from=build /src/target/release/fdb-gateway /usr/local/bin/fdb-gateway
COPY --from=component-source docker/fdb-gateway/entrypoint.sh /entrypoint.sh
RUN chmod 0555 /entrypoint.sh
USER 10001:10001
ENV RUST_LOG=info
EXPOSE 8080
HEALTHCHECK CMD ["curl", "--fail", "--silent", "http://127.0.0.1:8080/health"]
ENTRYPOINT ["/entrypoint.sh"]
