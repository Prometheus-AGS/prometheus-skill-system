variable "REGISTRY" { default = "ghcr.io/prometheus-ags" }
variable "TAG" { default = "dev" }

group "default" {
  targets = ["knowme-docs", "flint-forge-gateway", "flint-realtime-fabric", "flint-gate"]
}

group "extensions" {
  targets = ["flint-llm", "flint-vault", "flint-meta", "flint-auth", "flint-hooks", "pg-net", "pg-cron", "pgvector"]
}

group "validate" {
  targets = ["knowme-docs", "flint-forge-gateway", "flint-realtime-fabric", "flint-gate", "prometheus-postgres18"]
}

group "publish" {
  targets = ["knowme-docs", "flint-forge-gateway", "flint-realtime-fabric", "flint-gate", "prometheus-postgres18", "flint-llm", "flint-vault", "flint-meta", "flint-auth", "flint-hooks", "pg-net", "pg-cron", "pgvector"]
}

target "release" {
  platforms = ["linux/amd64", "linux/arm64"]
  args = { BUILDKIT_SYNTAX = "docker/dockerfile:1.18@sha256:dabfc0969b935b2080555ace70ee69a5261af8a8f1b4df97b9e7fbcf6722eddf" }
  attest = ["type=provenance,mode=max", "type=sbom"]
  cache-from = ["type=gha"]
  cache-to = ["type=gha,mode=max"]
}

target "knowme-docs" {
  inherits = ["release"]
  context = "https://github.com/Know-Me-Tools/hybrid-mobile-architecture-skill.git?ref=6fddffdac56075737f6f114adeddbed56208699b&checksum=6fddffdac56075737f6f114adeddbed56208699b&subdir=site"
  dockerfile = "Dockerfile"
  tags = ["${REGISTRY}/knowme-docs:${TAG}"]
}

target "flint-forge-gateway" {
  inherits = ["release"]
  context = "."
  dockerfile = "deploy/docker/flint-forge-gateway.Dockerfile"
  contexts = { component-source = "https://github.com/Know-Me-Tools/flint-forge.git?ref=4d5f97fc6695f3fa69ae43e720cea3c97ef057a4&checksum=4d5f97fc6695f3fa69ae43e720cea3c97ef057a4" }
  tags = ["${REGISTRY}/flint-forge-gateway:${TAG}"]
}

target "flint-realtime-fabric" {
  inherits = ["release"]
  context = "."
  dockerfile = "deploy/docker/flint-realtime-fabric.Dockerfile"
  contexts = { component-source = "https://github.com/Prometheus-AGS/flint-realtime-fabric.git?ref=edbb21556b0b37e2d7431e3969bcdb0c62fd6b9c&checksum=edbb21556b0b37e2d7431e3969bcdb0c62fd6b9c" }
  tags = ["${REGISTRY}/flint-realtime-fabric:${TAG}"]
}

target "flint-gate" {
  inherits = ["release"]
  context = "."
  dockerfile = "deploy/docker/flint-gate.Dockerfile"
  contexts = { component-source = "https://github.com/Know-Me-Tools/flint-gate.git?ref=057d64d2757f2065443fc05bf6da68aa72e4d4b5&checksum=057d64d2757f2065443fc05bf6da68aa72e4d4b5" }
  tags = ["${REGISTRY}/flint-gate:${TAG}"]
}

target "pgrx-extension" {
  inherits = ["release"]
  context = "."
  dockerfile = "deploy/docker/postgres-pgrx-extension.Dockerfile"
  contexts = {
    forge-source = "https://github.com/Know-Me-Tools/flint-forge.git?ref=4d5f97fc6695f3fa69ae43e720cea3c97ef057a4&checksum=4d5f97fc6695f3fa69ae43e720cea3c97ef057a4"
  }
}

target "flint-llm" {
  inherits = ["pgrx-extension"]
  args = { EXTENSION_PATH = "crates/ext-flint-llm" }
  tags = ["${REGISTRY}/postgres-extension-flint-llm:${TAG}"]
}
target "flint-vault" {
  inherits = ["pgrx-extension"]
  args = { EXTENSION_PATH = "crates/ext-flint-vault" }
  tags = ["${REGISTRY}/postgres-extension-flint-vault:${TAG}"]
}
target "flint-meta" {
  inherits = ["pgrx-extension"]
  args = { EXTENSION_PATH = "crates/ext-flint-meta" }
  tags = ["${REGISTRY}/postgres-extension-flint-meta:${TAG}"]
}
target "flint-auth" {
  inherits = ["pgrx-extension"]
  args = { EXTENSION_PATH = "crates/ext-flint-auth" }
  tags = ["${REGISTRY}/postgres-extension-flint-auth:${TAG}"]
}
target "flint-hooks" {
  inherits = ["pgrx-extension"]
  args = { EXTENSION_PATH = "crates/ext-flint-hooks" }
  tags = ["${REGISTRY}/postgres-extension-flint-hooks:${TAG}"]
}

target "pg-net" {
  inherits = ["release"]
  context = "."
  dockerfile = "deploy/docker/postgres-c-extension.Dockerfile"
  contexts = { extension-source = "https://github.com/supabase/pg_net.git?ref=a8299b11182ea5c974f5e89ae83e70e9e44e9e8f&checksum=a8299b11182ea5c974f5e89ae83e70e9e44e9e8f" }
  args = { EXTENSION_NAME = "pg_net" }
  tags = ["${REGISTRY}/postgres-extension-pg-net:${TAG}"]
}

target "pg-cron" {
  inherits = ["release"]
  context = "."
  dockerfile = "deploy/docker/postgres-c-extension.Dockerfile"
  contexts = { extension-source = "https://github.com/citusdata/pg_cron.git?ref=89d55ced1391bcfabc3c20d5672f14bde5c71925&checksum=89d55ced1391bcfabc3c20d5672f14bde5c71925" }
  args = { EXTENSION_NAME = "pg_cron" }
  tags = ["${REGISTRY}/postgres-extension-pg-cron:${TAG}"]
}

target "pgvector" {
  inherits = ["release"]
  context = "."
  dockerfile = "deploy/docker/postgres-pgvector-extension.Dockerfile"
  contexts = { extension-source = "https://github.com/pgvector/pgvector.git?ref=778dacf20c07caf904557a88705142631818d8cb&checksum=778dacf20c07caf904557a88705142631818d8cb" }
  tags = ["${REGISTRY}/postgres-extension-pgvector:${TAG}"]
}

target "prometheus-postgres18" {
  inherits = ["release"]
  context = "."
  dockerfile = "deploy/docker/prometheus-postgres18.Dockerfile"
  contexts = {
    flint-llm = "target:flint-llm"
    flint-vault = "target:flint-vault"
    flint-meta = "target:flint-meta"
    flint-auth = "target:flint-auth"
    flint-hooks = "target:flint-hooks"
    pg-net = "target:pg-net"
    pg-cron = "target:pg-cron"
    pgvector = "target:pgvector"
  }
  tags = ["${REGISTRY}/prometheus-postgres18:${TAG}"]
}
