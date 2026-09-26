# syntax=docker/dockerfile:1.18@sha256:dabfc0969b935b2080555ace70ee69a5261af8a8f1b4df97b9e7fbcf6722eddf
FROM rust:1.96-bookworm@sha256:a339861ae23e9abb272cea45dfafde21760d2ce6577a70f8a926153677902663 AS build
ARG EXTENSION_NAME
ENV DEBIAN_FRONTEND=noninteractive
RUN apt-get update \
 && apt-get install -y --no-install-recommends postgresql-common gnupg ca-certificates make gcc libcurl4-openssl-dev \
 && /usr/share/postgresql-common/pgdg/apt.postgresql.org.sh -y \
 && apt-get update \
 && apt-get install -y --no-install-recommends postgresql-server-dev-18 \
 && rm -rf /var/lib/apt/lists/*
WORKDIR /src
COPY --from=extension-source . ./
RUN if [ "$EXTENSION_NAME" = "pg_cron" ]; then sed -i 's/^SHLIB_LINK = $(libpq) -lintl/SHLIB_LINK = $(libpq)/' Makefile; fi \
 && make PG_CONFIG=/usr/lib/postgresql/18/bin/pg_config \
 && make install DESTDIR=/out PG_CONFIG=/usr/lib/postgresql/18/bin/pg_config

FROM scratch
COPY --from=build /out/usr/share/postgresql/18/extension/ /share/extension/
COPY --from=build /out/usr/lib/postgresql/18/lib/ /lib/
