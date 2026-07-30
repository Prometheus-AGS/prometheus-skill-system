# 09b · liter-llm — The Model Gateway

`liter-llm` is the multi-provider gateway the pack routes model traffic through. It is a
vendored Rust workspace at `tools/liter-llm`, installed as a single binary at
`~/.local/bin/liter-llm`.

Everything on this page was verified against the installed binary and its source. Where
earlier documentation in this repository described commands that do not exist, that is
called out explicitly — those errors caused real, silent failures.

## It is a server, not a completion CLI

This is the single most important fact about it, and the one earlier docs got wrong.

```console
$ liter-llm --help
LiterLLM proxy server and MCP tool server

Commands:
  api   Start the OpenAI-compatible proxy server
  mcp   Start the MCP server exposing LLM operations as tools
  help  Print this message or the help of the given subcommand(s)
```

**Two subcommands. That is the entire CLI.**

There is no `liter-llm complete`, no `mcp-call`, no `list_models` subcommand. Earlier
revisions of several skills called `liter-llm complete --model medium` and paired it with
`2>/dev/null || echo "{}"`. Because the guard only checked that the *binary* existed, the
call failed, the error was swallowed, and the caller silently produced empty results —
for the entire life of those scripts.

Shell callers should speak OpenAI REST to the gateway. The pack provides a helper that
reports failures instead of swallowing them:

```bash
. "${CLAUDE_PLUGIN_ROOT}/shared/scripts/lib/kbd-model-resolve.sh"
out="$(kbd_complete "$(kbd_resolve_role critic)" "$SYSTEM" "$USER" 2048)" || {
  echo "model call failed (see message above)" >&2
}
```

`kbd_complete` captures the HTTP status explicitly (`curl -s` without `-f` exits 0 on
4xx/5xx), distinguishes 401/403 from other failures, and fails loudly on a non-JSON body
or an empty message — because a silent empty completion reads as "the model had nothing
to say".

## `liter-llm api` — the OpenAI-compatible proxy

```console
$ liter-llm api --help
Start the OpenAI-compatible proxy server

Options:
  -c, --config <CONFIG>   Path to config file (default: auto-discover liter-llm-proxy.toml)
      --host <HOST>       Override bind host [default: 0.0.0.0]
  -p, --port <PORT>       Override bind port
      --tokio-worker-threads <N>
      --tokio-max-blocking-threads <N>
```

### Routes

The proxy exposes 25 routes. Every `/v1/*` route sits behind an **unconditional Bearer
check**.

| Group | Routes |
|---|---|
| Chat & completion | `/v1/chat/completions`, `/v1/responses`, `/v1/responses/{id}`, `/v1/responses/{id}/cancel` |
| Models | `/v1/models` |
| Embeddings & rerank | `/v1/embeddings`, `/v1/rerank` |
| Audio | `/v1/audio/speech`, `/v1/audio/transcriptions` |
| Images | `/v1/images/generations` |
| Documents | `/v1/ocr` |
| Moderation | `/v1/moderations` |
| Search | `/v1/search` |
| Files | `/v1/files`, `/v1/files/{id}/content` |
| Batches | `/v1/batches`, `/v1/batches/{id}`, `/v1/batches/{id}/cancel` |
| Realtime | `/v1/realtime` |
| Health (unauthenticated) | `/health`, `/healthz`, `/readyz`, `/health/liveness`, `/health/readiness` |
| Spec | `/openapi.json` |

Because it is OpenAI-shaped, any OpenAI SDK works against it by setting the base URL.

## `liter-llm mcp` — the MCP tool server

```console
$ liter-llm mcp --help
Start the MCP server exposing LLM operations as tools

Options:
  -c, --config <CONFIG>   Path to config file
      --transport <TRANSPORT>  Transport mode: stdio or http [default: stdio]
      --host <HOST>       Host for HTTP transport [default: 127.0.0.1]
      --port <PORT>       Port for HTTP transport [default: 3001]
```

### The 22 tools

Enumerated live from `tools/list` on 2026-07-30:

```
cancel_batch     cancel_response   chat              create_batch      create_file
create_response  delete_file       embed             file_content      generate_image
list_batches     list_files        list_models       moderate          ocr
rerank           retrieve_batch    retrieve_file     retrieve_response
search           speech            transcribe
```

The chat tool is **`chat`**. Earlier documentation in this repository described four tools
that do not exist:

| Documented before | Reality |
|---|---|
| `complete` | Does not exist. Use `chat`. |
| `health` | Does not exist. Probe `GET /v1/models`. |
| `stream` | Does not exist. Streaming is a `chat` parameter. |
| `get_cost`, `create_api_key`, `set_rate_limit`, `set_budget`, `cache_*` | Not tools. These are **config sections** (`[budget]`, `[rate_limit]`, `[cache]`, `[[keys]]`). |

`list_models` is a genuine MCP tool — but **not** a CLI subcommand. From a shell, use
`GET /v1/models`.

## Configuration

`liter-llm-proxy.toml`. Every struct uses `serde(deny_unknown_fields)`, so a typo is a
hard parse error rather than a silently ignored key.

### Sections

| Section | Purpose |
|---|---|
| `[server]` | bind host/port |
| `[general]` | `master_key`, `default_timeout_secs`, `max_retries`, cost tracking, tracing |
| `[[models]]` | named model → provider + endpoint + key |
| `[[aliases]]` | pattern-matched routing (`pattern`, `api_key`, `base_url`) |
| `[[keys]]` | virtual API keys with per-key model allowlists and RPM |
| `[rate_limit]` | `rpm`, `tpm` |
| `[budget]` | `global_limit`, per-model limits, enforcement mode |
| `[cache]` | response caching — `max_entries`, `ttl_seconds`, backend |
| `[files]` | file storage |
| `[health]` | probe interval and probe model |
| `[cooldown]` | failure cooldown duration |
| `[mcp]` | `stdio_trust_local` and stdio key binding |
| `[security]` | `outbound_policy`, `outbound_allowlist` |

### `[[models]]` — the exact schema

Fields are exactly these. Anything else is a parse error:

```toml
[[models]]
name = "kbd-judge"                       # what callers send as "model"
provider_model = "openai/gpt-5.6-sol"    # registry prefix + model id
api_key = "${OPENAI_API_KEY}"            # literal or ${VAR}
base_url = "http://localhost:8181/v1"    # optional endpoint override
timeout_secs = 60                        # optional
fallbacks = ["kbd-critic"]               # optional, tried in order
```

`fallbacks` is worth knowing: it gives a role automatic resilience without any caller-side
retry logic.

### A minimal working config

Three things are **mandatory** and were all missing from the config this repo shipped
before 2026-07-30 — the result being a gateway that could not serve a single request:

```toml
[general]
# REQUIRED. Every /v1/* route is behind an unconditional Bearer check, so a config with
# no master_key and no [[keys]] answers 401 to EVERYTHING, /v1/models included.
master_key = "${LITER_LLM_MASTER_KEY}"

[security]
# REQUIRED for any localhost base_url. The default deny_private REFUSES loopback.
outbound_policy = "off"

[[models]]
name = "kbd-judge"
provider_model = "openai/gpt-5.6-sol"
api_key = "sk-proxy-local"
base_url = "http://localhost:8181/v1"
```

### Config discovery — pass `--config`

`ProxyConfig::discover()` walks the **current directory upward** looking for
`liter-llm-proxy.toml`. It **never searches `$HOME`**. Always pass an absolute path:

```bash
liter-llm api --config ~/.config/liter-llm/liter-llm-proxy.toml
liter-llm mcp --transport stdio --config ~/.config/liter-llm/liter-llm-proxy.toml
```

Without it, `liter-llm mcp` does not load zero models — it **fails to start**, because
`[mcp] stdio_trust_local` lives in the config it was never given:

```console
$ liter-llm mcp --transport stdio
Error: stdio MCP transport requires authentication configuration; set either
`mcp.stdio_key_id` or `mcp.stdio_trust_local = true`
```

### Environment interpolation

`${VAR}` only. There is **no** `${VAR:-default}`, and an **unset** variable expands to the
empty string rather than erroring — which surfaces much later as an unexplained 401. The
pack's tooling verifies every referenced variable is set:

```bash
bash skills/process/liter-llm-bridge/scripts/configure-models.sh check
bash scripts/check-model-config.sh
```

## Providers

The registry (`tools/liter-llm/schemas/providers.json`) carries **143 providers**. Model
references are `prefix/model-id`:

```toml
provider_model = "moonshot/kimi-k2.5"
provider_model = "minimax/MiniMax-M2.5"
provider_model = "dashscope/qwen3-coder-plus"
provider_model = "zai/glm-4.7"
provider_model = "openai/gpt-5.6-sol"
```

Any OpenAI-compatible endpoint that is not in the registry is reachable by keeping a
routable prefix and overriding `base_url` — which is also how subscription **coding plans**
are reached. See [09a · Adversarial Review](09a-adversarial-review.md#coding-plans-need-a-different-endpoint).

## What we expose, and why

The pack does not use all 22 tools. What it actually depends on:

| Capability | Used by |
|---|---|
| `chat` / `/v1/chat/completions` | adversarial-review judge, pmpo-evolver extraction & dreaming, learn-grade |
| `/v1/models` | gateway reachability probe, `configure-models.sh verify` |
| `[[models]]` + `base_url` | pointing a logical role at any provider without touching a script |
| `fallbacks` | role resilience |
| `[security] outbound_policy` | permitting the local `openai-proxy` on loopback |
| `[[keys]]`, `[rate_limit]`, `[budget]`, `[cache]` | available, not currently wired by the pack |

The last row matters: virtual keys, rate limits, budgets, and response caching are real
features of the gateway you can enable yourself in `liter-llm-proxy.toml`. No pack skill
configures them today.

## Managing it

```bash
S="${CLAUDE_PLUGIN_ROOT}/skills/process/liter-llm-bridge/scripts/configure-models.sh"

bash "$S" check                    # state; changes nothing
bash "$S" repair                   # add ONLY missing mandatory pieces (merges, never clobbers)
bash "$S" add-provider kimi        # local-proxy|kimi|minimax|qwen|glm|glm-coding|kimi-coding
bash "$S" verify                   # GET /v1/models must be 200, then one real completion per role
bash "$S" migrate                  # retire the legacy config.toml
```

There is **no** `liter-llm config`, `doctor`, or `validate` subcommand. The only way to
validate a config is to start the server and watch it fail — which is exactly what
`verify` automates.

## Relationship to openai-proxy

Two different things that both speak OpenAI:

- **`openai-proxy`** (`:8181`) — bridges OpenAI-compatible clients to a ChatGPT
  subscription via `~/.codex/auth.json`. Needs **no inbound key**. Serves `gpt-5.6-sol`,
  `gpt-5.5`, `gpt-5.4-mini`, and others.
- **`liter-llm api`** (`:4000` by default) — the multi-provider gateway. **Requires** a
  Bearer token on every `/v1/*` route.

They compose: a `[[models]]` entry can point at `openai-proxy` as just another
OpenAI-compatible `base_url`, which is what the default config does.

## See also

- [09a · Adversarial Review](09a-adversarial-review.md) — the primary consumer
- [05 · MCP Substrate](05-mcp-substrate.md) — how MCP servers are registered
- [13 · Tools Reference](13-tools-reference.md) — binary inventory
