# syntax=docker/dockerfile:1.18@sha256:dabfc0969b935b2080555ace70ee69a5261af8a8f1b4df97b9e7fbcf6722eddf
FROM rust:1.96-bookworm@sha256:a339861ae23e9abb272cea45dfafde21760d2ce6577a70f8a926153677902663 AS build
ARG EXTENSION_PATH
ENV DEBIAN_FRONTEND=noninteractive
RUN apt-get update \
 && apt-get install -y --no-install-recommends postgresql-common gnupg ca-certificates clang pkg-config libssl-dev \
 && /usr/share/postgresql-common/pgdg/apt.postgresql.org.sh -y \
 && apt-get update \
 && apt-get install -y --no-install-recommends postgresql-server-dev-18 \
 && rm -rf /var/lib/apt/lists/*
RUN cargo install cargo-pgrx --version 0.18.1 --locked \
 && cargo pgrx init --pg18 /usr/lib/postgresql/18/bin/pg_config
WORKDIR /build/extension
COPY --from=forge-source ${EXTENSION_PATH}/ ./
RUN cargo pgrx package --pg-config /usr/lib/postgresql/18/bin/pg_config --out-dir /out

FROM scratch
COPY --from=build /out/usr/share/postgresql/18/extension/ /share/extension/
COPY --from=build /out/usr/lib/postgresql/18/lib/ /lib/
