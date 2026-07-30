# Model configuration for adversarial review

How the judge finds a model that is **not** the producer, and how to point it at any
provider you have — a local `openai-proxy`, or a Kimi/MiniMax/Qwen/GLM coding plan.

## The rule this exists to enforce

The critic must not share the producer's blind spots. A same-family judge is a
**failure**, not a fallback.

This was not enforced for a long time. All 8 reviews stored under
`.kbd-orchestrator/phases/docusaurus-github-pages-site/review/` record
`isolation_mode: harness-native` and `judge_model: "harness-subagent (claude,
parent-session family)"` — Claude reviewing Claude, every one `PASS`. Treat those
artifacts as unreliable.

## Two files, one gateway

```
~/.prometheus/kbd/models.toml              KBD owns:       role -> model NAME
~/.config/liter-llm/liter-llm-proxy.toml   liter-llm owns: NAME -> provider + base_url + ${KEY}
```

Adding a provider edits liter-llm's file. Repointing a role edits `models.toml`.
**Neither requires editing a script** — that is the entire point of the split.

```toml
# ~/.prometheus/kbd/models.toml
[gateway]
candidates = ["http://localhost:8181/v1", "http://localhost:4000/v1"]

[roles]
generator = "kbd-frontier"   # the producer; not dispatched, only compared against
critic    = "kbd-critic"     # MUST differ from generator
judge     = "kbd-judge"
```

## Resolution order

AWS-CLI convention, highest wins. Implemented once in
`shared/scripts/lib/kbd-model-resolve.sh`:

```
explicit argument
  > PROMETHEUS_KBD_<ROLE>_MODEL        e.g. PROMETHEUS_KBD_JUDGE_MODEL
  > ~/.prometheus/kbd/models.toml      [roles]
  > .kbd-orchestrator/project.json     model_policy   (repo-local)
  > built-in default                   kbd-judge / kbd-critic / kbd-frontier
```

`preflight-models.sh` prints which layer supplied each role, so this is never a guess.

## Three contracts that will bite you

All three were verified against `tools/liter-llm`, and all three were live defects
in the shipped configuration.

### 1. `/v1/*` requires a Bearer token unconditionally

A config with no `[general] master_key` and no `[[keys]]` answers **401 to
everything**, `/v1/models` included. The template shipped before 2026-07-30 omitted
it, so the config could not serve a single request.

```toml
[general]
master_key = "${LITER_LLM_MASTER_KEY}"
```

### 2. `deny_private` blocks loopback

`[security].outbound_policy` defaults to `deny_private`, which **refuses** any
`localhost` / `127.0.0.1` `base_url` with `OutboundForbidden`.

```toml
[security]
outbound_policy = "off"
# or, if this host also reaches untrusted networks:
#   outbound_policy = "allowlist"
#   outbound_allowlist = ["http://localhost:8181"]
```

### 3. liter-llm never searches `$HOME`

`ProxyConfig::discover()` walks the **CWD upward** for `liter-llm-proxy.toml`.
Always pass `--config <abs path>`:

```bash
liter-llm api --config ~/.config/liter-llm/liter-llm-proxy.toml
liter-llm mcp --transport stdio --config ~/.config/liter-llm/liter-llm-proxy.toml
```

Without it the MCP server does not merely load zero models — it **fails to start**,
because `[mcp] stdio_trust_local` lives in the config it was never given.

## Providers

liter-llm has first-class providers for all four; use the real prefixes.

| Provider | `provider_model` | `base_url` | key var |
|---|---|---|---|
| openai-proxy (local) | `openai/gpt-5.6-sol` | `http://localhost:8181/v1` | none (`sk-proxy-local`) |
| Kimi / Moonshot | `moonshot/kimi-k2.5` | `https://api.moonshot.ai/v1` | `MOONSHOT_API_KEY` |
| MiniMax | `minimax/MiniMax-M2.5` | `https://api.minimax.io/v1` | `MINIMAX_API_KEY` |
| Qwen / DashScope | `dashscope/qwen3-coder-plus` | `https://dashscope-intl.aliyuncs.com/compatible-mode/v1` | `DASHSCOPE_API_KEY` |
| Z.ai GLM | `zai/glm-4.7` | `https://api.z.ai/api/paas/v4` | `ZAI_API_KEY` |

### Coding plans

The `*-coding-plan` entries in liter-llm's `schemas/catalog.json` are **metadata,
not routable providers** — `provider_model = "kimi-for-coding/k3"` will not route.
Keep a routable prefix and override the endpoint:

```toml
[[models]]
name = "kbd-glm"
provider_model = "zai/glm-5.2"
base_url = "https://api.z.ai/api/coding/paas/v4"   # coding path, NOT /api/paas/v4
api_key  = "${ZAI_API_KEY}"
```

Z.ai's docs are explicit: a Coding Plan **must** use `/api/coding/paas/v4`, or the
subscription quota is not drawn. `/liter-llm-bridge configure add-provider glm-coding`
writes this for you.

**OAuth-only plans** (MiniMax/Qwen subscription portals, some Kimi coding plans)
expose *Anthropic*-shaped endpoints an OpenAI-REST caller cannot use. The wizard
refuses these rather than writing config that 404s later. (`openai-proxy`'s OAuth is
OpenAI/ChatGPT-only and unrelated.)

## Secrets

Never in the TOML. Keys live in `~/.prometheus/kbd/secrets.env` (`0600`, outside any
repo) and are referenced as `${VAR}`:

```bash
set -a; . ~/.prometheus/kbd/secrets.env; set +a
```

liter-llm supports `${VAR}` **only** — no `${VAR:-default}` — and expands an **unset**
var to `""`, which surfaces much later as an unexplained 401. Both
`/liter-llm-bridge configure check` and `scripts/check-model-config.sh` verify every
referenced var is actually set.

## Commands

```bash
S="${CLAUDE_PLUGIN_ROOT}/skills/process/liter-llm-bridge/scripts/configure-models.sh"

bash "$S" check                    # report state; change nothing
bash "$S" repair                   # add ONLY the missing mandatory pieces
bash "$S" add-provider glm-coding   # local-proxy|kimi|minimax|qwen|glm|glm-coding|kimi-coding
bash "$S" verify                   # live 1-token completion per role
bash "$S" migrate                  # retire the legacy config.toml

bash scripts/check-model-config.sh  # routing + cache-drift audit (exit 2 = drift)
```

## Reading the artifact

`findings.json` now records what actually answered:

| Field | Meaning |
|---|---|
| `isolation_mode` | `rest-gateway:<url>` — the endpoint that served the review |
| `cross_model_check` | `verified-distinct` · `same-model-collision` · `unverified-producer-unknown` |
| `judge_model` / `producer_model` | the compared pair |

`isolation_mode` used to be the hardcoded literal `"liter-llm"` regardless of what
answered, which made a self-grade indistinguishable from a real cross-model review.

`unverified-producer-unknown` means the packet carried no `producer_model`, so the
collision check passed **trivially** — export `KBD_PRODUCER_MODEL` to fix it.

## Never edit a plugin cache

Files under `~/.claude/plugins/cache/...` (and the Codex equivalent) are overwritten
by the next install, and edits there are invisible to git. Caches are keyed by plugin
**version**, so a same-version edit is not picked up from the repo either — which is
how a previous "fix" both appeared to work and then silently vanished.

Change the repo, then:

```bash
bash scripts/update-skill-pack.sh --force   # also refreshes the plugin caches
bash scripts/check-model-config.sh          # exit 2 if any cache still diverges
```
