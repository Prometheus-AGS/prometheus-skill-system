---
name: liter-llm-bridge
version: '0.1.0'
description: >
  Harness-agnostic multi-model routing fallback. Builds and configures
  liter-llm (a Rust LLM proxy with a built-in MCP server) so any harness
  with MCP support can dispatch each phase of KBD / iterative-evolver /
  artifact-refiner to the cheapest model that meets the cognitive
  requirement. Use when the host harness (Claude Code, codex, opencode,
  cursor, etc.) does not natively support per-skill model selection.
authors:
  - 'Prometheus AGS'
allowed-tools: file_system code_interpreter
model_routing:
  policy_source: ".kbd-orchestrator/project.json → model_policy"
  phases:
    liter-bridge-install: small
    liter-bridge-configure: small
    liter-bridge-route: small
  routing_reference: "references/mcp-tools.md"
triggers:
  keywords:
    - liter-llm
    - liter-llm-bridge
    - install liter-llm
    - configure liter-llm
    - multi-model routing
    - model proxy
    - mcp model server
  semantic: >
    Install liter-llm, detect provider API keys, register the liter-llm
    MCP server with the active harness, or document how skills should
    invoke its tools to route per-phase model classes.
---

# liter-llm-bridge

A harness-agnostic fallback for multi-model routing across the KBD pipeline. Builds [liter-llm](https://github.com/GQAdonis/liter-llm) from the user's Rust fork, detects which providers are configured, and registers liter-llm's MCP server with the active harness so skills can dispatch each phase to the cheapest viable model.

## When to Use

Use this skill when:

- The host harness does not support per-skill model selection (most don't, today)
- A KBD/evolver/refiner phase emits `[MODEL_ROUTING] class=medium ...` and you want it actually honored
- You want frontier API spend bounded to assess/plan/reflect — everything else routed to local or T4-class models

Skip this skill when:

- The harness already supports model switching per dispatch (e.g., Claude Code with `--model` per subagent, prom-lanes, UAR)
- You don't have any non-frontier providers configured (in which case there's nothing to route to)

## Workflow

The skill has three slash entry points:

1. **`/liter-llm-bridge install`** — clone, build, install the binary. See `prompts/install.md`.
2. **`/liter-llm-bridge configure`** — detect providers, write `~/.config/liter-llm/config.toml`, register the MCP server with the active harness. See `prompts/configure.md`.
3. **`/liter-llm-bridge route`** — document or activate per-phase routing via the liter-llm MCP `complete` tool. See `prompts/route.md`.

Run them in order on first setup. After that, only `configure` needs to be re-run when adding new providers.

## Architecture

```
KBD/evolver/refiner phase
        │
        ▼
[MODEL_ROUTING] class=medium model=...
        │
        ▼
   harness hook  ──→  liter-llm MCP server  ──→  provider (Anthropic / OpenAI / Groq / Ollama / vLLM / ...)
        │                  (stdio transport)
        ▼
    response
```

liter-llm exposes 22 MCP tools (model routing, virtual API keys, rate limits, cost tracking, response caching, OpenAPI spec at `/openapi.json`). The bridge skill only needs the `complete` tool plus the alias resolution table — the rest is reference.

## Fallback Semantics

- **No provider configured for a class** → route falls through to the host model. Emit warning. Do not fail the phase.
- **Class downgrade attempted** (e.g., a `frontier`-required phase tries to run on a `small` model) → emit `MODEL MISMATCH` and stop. The cheap-runs-frontier-work case is the dangerous silent failure and is never allowed.
- **liter-llm not installed but skill invoked** → run `/liter-llm-bridge install` first, do not partial-configure.

## References

- `prompts/install.md` — build + install workflow
- `prompts/configure.md` — provider detection, harness registration
- `prompts/route.md` — invocation contract, hook templates
- `references/provider-env-vars.md` — canonical list of provider env vars (Anthropic, OpenAI, Groq, Together, Mistral, Cohere, Ollama, vLLM, etc.)
- `references/mcp-tools.md` — the 22 tools liter-llm exposes via stdio MCP

## Source

The bridge expects the user's Rust fork: `https://github.com/GQAdonis/liter-llm.git`. The fork's `liter-llm-cli` crate provides the binary, and `liter-llm mcp --transport stdio` starts the MCP server.
