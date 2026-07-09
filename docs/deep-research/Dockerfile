# syntax=docker/dockerfile:1.7
# Multi-stage build for the prometheus-research daemon (deep-research UI host).
#
# Build context expected at the repo root (see docker-compose.yml), because
# the crate uses include_bytes! to embed both the UI HTML from
# docs/deep-research/ and vendored assets from substrate/prometheus-research/.

# ---------- Stage 1: build ----------
FROM rust:1.88-bookworm AS build

WORKDIR /src

# Copy only what the daemon crate + its embedded assets need. We deliberately
# do NOT copy the whole workspace to keep the layer small and cache friendly.
COPY substrate/prometheus-research /src/substrate/prometheus-research
COPY docs/deep-research/deep-research-ui.html /src/docs/deep-research/deep-research-ui.html

WORKDIR /src/substrate/prometheus-research

# Build in release mode. Cargo will resolve deps against crates.io.
RUN cargo build --release --locked

# ---------- Stage 2: runtime ----------
FROM debian:bookworm-slim AS runtime

# ca-certificates for HTTPS calls the research daemon makes to search APIs.
# tini as PID 1 to reap zombies from spawned child jobs.
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates tini \
    && rm -rf /var/lib/apt/lists/*

# Non-root runtime user.
RUN useradd --system --create-home --shell /usr/sbin/nologin prometheus

COPY --from=build /src/substrate/prometheus-research/target/release/prometheus-research /usr/local/bin/prometheus-research

USER prometheus
WORKDIR /home/prometheus

EXPOSE 7891

# Health hint for compose (compose has its own healthcheck block; this is a
# fallback for `docker inspect`).
HEALTHCHECK --interval=30s --timeout=3s --start-period=10s --retries=3 \
    CMD ["/usr/local/bin/prometheus-research", "--mode", "status"]

ENTRYPOINT ["/usr/bin/tini", "--", "/usr/local/bin/prometheus-research"]
CMD ["--mode", "server"]
