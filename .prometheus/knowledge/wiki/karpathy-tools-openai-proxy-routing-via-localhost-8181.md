---
type: Reference
id: karpathy-tools-openai-proxy-routing-via-localhost-8181
title: Karpathy Tools OpenAI Proxy Routing via localhost:8181
description: "On `2026-07-10`, the Prometheus stack was configured so Karpathy knowledge tools route chat/completion traffic through a local OpenAI-compatible proxy:"
tags:
- openai-proxy
- karpathy-tools
- pk-cli
- pk-cherry
- embeddings
- surreal-memory-native
links:
- surreal-memory-server-latest-binary-stabilization
sources:
- manual-test
timestamp: 2026-07-10T23:42:00.108897+00:00
created_at: 2026-07-10T23:42:00.108897+00:00
updated_at: 2026-07-10T23:42:00.108897+00:00
revision: 0
---

## Configuration Summary

On `2026-07-10`, the Prometheus stack was configured so Karpathy knowledge tools route chat/completion traffic through a local OpenAI-compatible proxy:

- `pk` CLI
- `pk-cherry` MCP server
- Proxy base URL: `http://localhost:8181/v1`

## Routing Behavior

The proxy is used for LLM chat/completion requests only.

- Served models:
  - `gpt-5.4` for compile
  - `gpt-5.4-mini` for lint
  - `gpt-5.4-mini` for focus
  - `gpt-5.4-mini` for fix
- The proxy does **not** implement embeddings:
  - `POST /v1/embeddings` returns `404`

## Embeddings Architecture

Because the local OpenAI-compatible proxy does not provide embeddings, vector embedding generation remains on the local `BAAI/bge-small-en-v1.5` model hosted by `surreal-memory-native`.

This embeddings path is separate from `pk`'s full-text store.

Related operational background for the memory service is documented in [surreal-memory-server Latest Binary Stabilization](/surreal-memory-server-latest-binary-stabilization.md).

## Environment Variables

Routing is controlled through these environment variables:

- `CLOUD_LLM_URL`
- `LOCAL_LLM_URL`
- `CLOUD_LLM_API_KEY`
- `PK_COMPILE_MODEL`
- `PK_LINT_MODEL`
- `PK_FOCUS_MODEL`
- `PK_FIX_MODEL`

## Effective Model Mapping

```text
PK_COMPILE_MODEL -> gpt-5.4
PK_LINT_MODEL    -> gpt-5.4-mini
PK_FOCUS_MODEL   -> gpt-5.4-mini
PK_FIX_MODEL     -> gpt-5.4-mini
Base URL         -> http://localhost:8181/v1
Embeddings       -> not served by proxy (404)
```

## Operational Implication

The configuration splits inference responsibilities:

- Chat/completion inference goes through the local OpenAI-compatible proxy.
- Embeddings stay on the local `surreal-memory-native` service using `BAAI/bge-small-en-v1.5`.

This should be preserved when updating `pk` or `pk-cherry` environment configuration, since redirecting embeddings to the proxy will fail unless `/v1/embeddings` support is added.

# Citations

1. [1] manual-test