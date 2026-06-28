# Model Routing — pmpo-evolver

Reference for liter-llm integration in all pmpo-evolver phases. All `[MODEL_ROUTING]` directives emitted during an evolver run follow the format defined here.

## Directive format

```
[MODEL_ROUTING] phase=evolver-<key> class=<class> model=<resolved-model> env=<env>
```

Logged to `.evolver/<name>/model-routing.log` for each run.

Fields:
- `phase` — evolver phase key (e.g., `evolver-competitive-scan`)
- `class` — requested capability tier (`small | medium | frontier`)
- `model` — the actual model ID resolved by liter-llm (or `harness-native` if liter-llm absent)
- `env` — deployment environment (local, ci, prod)

## Provider discovery

### Step 1: Check configuration

```bash
CONFIG_PATH="${LITER_LLM_CONFIG:-$HOME/.config/liter-llm/config.toml}"
[ -f "${CONFIG_PATH}" ] && cat "${CONFIG_PATH}"
```

### Step 2: Query liter-llm MCP `list_models`

Returns configured aliases with resolved `{provider, model_id, class}`:

```json
[
  {"alias": "small",    "provider": "anthropic", "model_id": "claude-haiku-4-5-20251001", "class": "small"},
  {"alias": "medium",   "provider": "groq",      "model_id": "llama-3.3-70b-versatile",  "class": "medium"},
  {"alias": "frontier", "provider": "anthropic", "model_id": "claude-sonnet-4-6",         "class": "frontier"}
]
```

### Step 3: Verify provider health

Call liter-llm MCP `health` → per-provider status:

```json
{
  "anthropic": {"status": "ok", "latency_ms": 245},
  "groq":      {"status": "ok", "latency_ms": 89},
  "ollama":    {"status": "unreachable", "error": "connection refused"}
}
```

## Decision protocol

1. If `small` class configured AND provider healthy → use for cheap tasks
2. If `medium` class configured AND provider healthy → use for NLP tasks
3. If neither → fall through to `frontier` with a stderr warning
4. **Never** silently upgrade `small` to `frontier` — defeats cost optimization
5. **Never** silently downgrade `frontier` to `medium` for strategic synthesis tasks

## Phase class assignments

| Phase key | Class | Tool call |
|-----------|-------|-----------|
| `evolver-route` | small | routing selection |
| `evolver-competitive-scan` | frontier | `complete(model=frontier)` |
| `evolver-competitive-parity` | frontier | `complete(model=frontier)` |
| `evolver-changelog-extract` | medium | `complete(model=medium)` |
| `evolver-trend-synthesis` | frontier | `complete(model=frontier)` |
| `evolver-carry-forward` | small | bash grep + python3 |
| `evolver-idea-gate1` | small | `complete(model=small)` |
| `evolver-idea-gate2` | medium | `complete(model=medium)` |
| `evolver-idea-spec` | frontier | `complete(model=frontier)` |
| `evolver-signal-gh-issues` | medium | `complete(model=medium)` |
| `evolver-signal-commits` | small | bash + python3 |
| `evolver-signal-sentiment` | medium | `complete(model=medium)` |
| `evolver-signal-telemetry` | small | bash + python3 |
| `evolver-signal-synthesis` | medium | `complete(model=medium)` |
| `evolver-strategic-dream` | frontier | `complete(model=frontier)` |
| `evolver-reflect` | frontier | `complete(model=frontier)` |

## Provider capability reference (2026-06-28)

| Provider | small option | medium option | frontier option |
|----------|-------------|---------------|----------------|
| Anthropic | claude-haiku-4-5-20251001 | claude-sonnet-4-6 | claude-opus-4-8 |
| Groq | llama-3.1-8b-instant | llama-3.3-70b-versatile | (use Anthropic) |
| Ollama (local) | qwen3:4b, phi4-mini | qwen3:14b | qwen3:32b (32GB RAM) |
| vLLM (self-hosted) | depends on model | depends | depends |

## Cost tracking

```bash
# Poll cost after each complete call during development
COST=$(liter-llm mcp-call get_cost --session-id <id> 2>/dev/null || echo "unavailable")
echo "[model-routing] Session cost: $COST"
```

**Target:** feedback collection + changelog ingestion phases should cost ≤10% of what a `frontier-all` run would cost. Verify with `get_cost` during initial tuning.

## Fallback behavior

When liter-llm is absent or a class has no configured provider:
1. Log: `[MODEL_ROUTING] phase=<key> class=<class> model=harness-native fallback=true`
2. Use the session's host model (e.g., claude-sonnet-4-6 in Claude Code)
3. Continue without error — full functionality is preserved, just without cost optimization

## Example invocation (bash)

```bash
# Competitive landscape scan
# [MODEL_ROUTING] phase=evolver-competitive-scan class=frontier
if command -v liter-llm > /dev/null 2>&1; then
  RESULT=$(echo "${COMPETITOR_DATA}" | liter-llm complete \
    --model frontier \
    --system "Analyze these competitor features and produce a parity gap analysis." \
    2>/dev/null)
else
  echo "[model-routing] liter-llm not available; use host model for competitive scan"
  RESULT=""
fi
```
