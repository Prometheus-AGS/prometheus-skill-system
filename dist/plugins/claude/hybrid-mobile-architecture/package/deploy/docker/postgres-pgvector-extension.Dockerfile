# syntax=docker/dockerfile:1.18@sha256:dabfc0969b935b2080555ace70ee69a5261af8a8f1b4df97b9e7fbcf6722eddf
FROM postgres:18@sha256:32ca0af8e77bfb8c6610c488e4691f83f972a3e9e64d3b02facf3ab111ad5500 AS build
RUN apt-get update \
 && apt-get install -y --no-install-recommends postgresql-server-dev-18 make gcc \
 && rm -rf /var/lib/apt/lists/*
WORKDIR /src
COPY --from=extension-source . ./
RUN make PG_CONFIG=/usr/lib/postgresql/18/bin/pg_config \
 && make install DESTDIR=/out PG_CONFIG=/usr/lib/postgresql/18/bin/pg_config

FROM scratch
COPY --from=build /out/usr/share/postgresql/18/extension/ /share/extension/
COPY --from=build /out/usr/lib/postgresql/18/lib/ /lib/
