# syntax=docker/dockerfile:1.18@sha256:dabfc0969b935b2080555ace70ee69a5261af8a8f1b4df97b9e7fbcf6722eddf
FROM node:24-bookworm-slim@sha256:6f7b03f7c2c8e2e784dcf9295400527b9b1270fd37b7e9a7285cf83b6951452d AS ui
RUN corepack enable
WORKDIR /src
COPY --from=component-source pnpm-lock.yaml pnpm-workspace.yaml ./
COPY --from=component-source admin-ui/package.json admin-ui/package.json
COPY --from=component-source sdks/ts/package.json sdks/ts/package.json
COPY --from=component-source sdks/entity-management/package.json sdks/entity-management/package.json
RUN pnpm install --frozen-lockfile
COPY --from=component-source admin-ui/ admin-ui/
COPY --from=component-source sdks/ts/ sdks/ts/
COPY --from=component-source sdks/entity-management/ sdks/entity-management/
RUN pnpm --dir sdks/ts build && pnpm --dir sdks/entity-management build && pnpm --dir admin-ui build
FROM rust:1.96-bookworm@sha256:a339861ae23e9abb272cea45dfafde21760d2ce6577a70f8a926153677902663 AS build
RUN apt-get update && apt-get install -y --no-install-recommends clang libclang-dev libprotobuf-dev protobuf-compiler pkg-config && rm -rf /var/lib/apt/lists/*
WORKDIR /src
COPY --from=component-source Cargo.toml Cargo.lock ./
COPY --from=component-source crates/ crates/
COPY --from=component-source proto/ proto/
RUN mkdir -p proto/google \
 && cp -a /usr/include/google/protobuf proto/google/
COPY --from=ui /src/admin-ui/dist ./admin-ui/dist
RUN cargo build --locked --release -p frf-gateway
FROM debian:trixie-slim@sha256:020c0d20b9880058cbe785a9db107156c3c75c2ac944a6aa7ab59f2add76a7bd
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates curl libpq5 && rm -rf /var/lib/apt/lists/* && useradd --create-home --uid 10001 --shell /usr/sbin/nologin fabric
COPY --from=build /src/target/release/frf-gateway /usr/local/bin/frf-gateway
USER 10001:10001
EXPOSE 8080 9090
HEALTHCHECK CMD ["curl", "--fail", "--silent", "http://127.0.0.1:8080/health"]
ENTRYPOINT ["/usr/local/bin/frf-gateway"]
