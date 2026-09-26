# syntax=docker/dockerfile:1.18@sha256:dabfc0969b935b2080555ace70ee69a5261af8a8f1b4df97b9e7fbcf6722eddf
FROM postgres:18@sha256:32ca0af8e77bfb8c6610c488e4691f83f972a3e9e64d3b02facf3ab111ad5500
ARG TARGETARCH
ARG WALG_VERSION=v3.0.8
ARG WALG_AMD64_SHA256=f30544c5ce93cf83b87578e3c4a2e9c0e0ffc3d160ef89ecddaf75f397d98deb
ARG WALG_ARM64_SHA256=794d1a81f0c27825a1603bd39c0f2cf5dd8bed7cc36b598ca05d8d963c3d5fcf

RUN apt-get update \
 && apt-get install -y --no-install-recommends curl ca-certificates libcurl4 \
 && rm -rf /var/lib/apt/lists/*

COPY --from=flint-llm /share/extension/ /usr/share/postgresql/18/extension/
COPY --from=flint-llm /lib/ /usr/lib/postgresql/18/lib/
COPY --from=flint-vault /share/extension/ /usr/share/postgresql/18/extension/
COPY --from=flint-vault /lib/ /usr/lib/postgresql/18/lib/
COPY --from=flint-meta /share/extension/ /usr/share/postgresql/18/extension/
COPY --from=flint-meta /lib/ /usr/lib/postgresql/18/lib/
COPY --from=flint-auth /share/extension/ /usr/share/postgresql/18/extension/
COPY --from=flint-auth /lib/ /usr/lib/postgresql/18/lib/
COPY --from=flint-hooks /share/extension/ /usr/share/postgresql/18/extension/
COPY --from=flint-hooks /lib/ /usr/lib/postgresql/18/lib/
COPY --from=pg-net /share/extension/ /usr/share/postgresql/18/extension/
COPY --from=pg-net /lib/ /usr/lib/postgresql/18/lib/
COPY --from=pg-cron /share/extension/ /usr/share/postgresql/18/extension/
COPY --from=pg-cron /lib/ /usr/lib/postgresql/18/lib/
COPY --from=pgvector /share/extension/ /usr/share/postgresql/18/extension/
COPY --from=pgvector /lib/ /usr/lib/postgresql/18/lib/

RUN set -eux; \
    case "$TARGETARCH" in \
      amd64) artifact="wal-g-pg-22.04-amd64"; expected="$WALG_AMD64_SHA256" ;; \
      arm64) artifact="wal-g-pg-22.04-aarch64"; expected="$WALG_ARM64_SHA256" ;; \
      *) echo "unsupported WAL-G architecture: $TARGETARCH" >&2; exit 1 ;; \
    esac; \
    curl -fsSL -o /tmp/wal-g "https://github.com/wal-g/wal-g/releases/download/${WALG_VERSION}/${artifact}"; \
    echo "${expected}  /tmp/wal-g" | sha256sum -c -; \
    install -m 0755 /tmp/wal-g /usr/local/bin/wal-g; \
    rm /tmp/wal-g

COPY deploy/postgres/init/ /docker-entrypoint-initdb.d/
CMD ["postgres", "-c", "wal_level=logical", "-c", "shared_preload_libraries=pg_net,pg_cron,flint_llm", "-c", "cron.database_name=flint"]
