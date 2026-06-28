# Model Discovery Reference

How to programmatically discover available liter-llm models, select the best fit for a task class, and track cost usage.

---

## Config file path

liter-llm reads its provider configuration from (in order of precedence):

1. `LITELLM_CONFIG` environment variable (explicit path)
2. `~/.litellm/config.yaml`
3. `./litellm_config.yaml` (project root)

```bash
CONFIG_PATH="${LITELLM_CONFIG:-${HOME}/.litellm/config.yaml}"
[ -f "${CONFIG_PATH}" ] && echo "Config found: ${CONFIG_PATH}"
```

---

## Listing available models

```bash
# List all configured models
liter-llm list_models 2>/dev/null || echo "[]"

# JSON output
liter-llm list_models --json 2>/dev/null | python3 -c "
import json, sys
models = json.load(sys.stdin)
for m in models:
    print(f\"{m.get('model_name','?')} | provider={m.get('litellm_provider','?')}\")
"
```

---

## Health check per provider

```bash
# Check if a specific model is reachable
liter-llm health --model <model-name> 2>/dev/null
# Exit 0 = healthy, non-0 = unreachable

# Batch health check
for model in claude-3-haiku gpt-4o-mini llama-3-8b; do
  if liter-llm health --model "${model}" 2>/dev/null; then
    echo "${model}: AVAILABLE"
  else
    echo "${model}: UNAVAILABLE"
  fi
done
```

---

## Decision protocol (5 rules)

1. **Discover before routing** — call `list_models` at session start; cache the result for the session.
2. **Match class to task** — use the class table in `model-routing.md`. Never upgrade unless a lower class fails.
3. **Never silently upgrade small to frontier** — if `class=small` has no available model, fail loudly (`exit 1`) or fall back to the harness-native model and log the downgrade.
4. **Prefer the cheapest model that meets the class requirement** — within a class, pick the lowest cost-per-token model that is healthy.
5. **Log model used** — always set `model_used` in the LearningSignal output so cost can be tracked.

---

## Provider capability reference

| Provider | Small class | Medium class | Frontier class | Notes |
|----------|-------------|--------------|----------------|-------|
| Anthropic | claude-haiku-4-5 | claude-sonnet-4-6 | claude-opus-4-8 | Requires `ANTHROPIC_API_KEY` |
| OpenAI | gpt-4o-mini | gpt-4o | o3 | Requires `OPENAI_API_KEY` |
| Groq | llama-3-8b-groq | llama-3-70b-groq | — | Free tier available; no frontier |
| Ollama | llama3.2:3b | llama3.1:8b | llama3.1:70b | Local; requires running service |
| vLLM | model-specific | model-specific | model-specific | Self-hosted; no API key needed |

**Detection:**
```bash
# Anthropic available?
[ -n "${ANTHROPIC_API_KEY:-}" ] && echo "anthropic: yes"

# Groq available?
[ -n "${GROQ_API_KEY:-}" ] && echo "groq: yes"

# Ollama available?
curl -s http://localhost:11434/api/tags >/dev/null 2>&1 && echo "ollama: yes"
```

---

## Cost tracking with get_cost

```bash
# Estimate cost before calling
COST=$(liter-llm get_cost --model <model> --prompt-tokens 1000 --completion-tokens 200 2>/dev/null || echo "unknown")

# After calling, log actual cost
RESPONSE=$(echo "${PROMPT}" | liter-llm complete --model <model> --json-output 2>/dev/null)
ACTUAL_COST=$(echo "${RESPONSE}" | python3 -c "
import json, sys
d = json.load(sys.stdin)
usage = d.get('usage', {})
print(f\"{usage.get('prompt_tokens',0)} in, {usage.get('completion_tokens',0)} out\")
" 2>/dev/null || echo "unknown")
```

---

## Fallback behavior

When `liter-llm` is unavailable or a model is unreachable:

1. All scripts that call `liter-llm` check `command -v liter-llm` first.
2. If absent, emit a structured fallback result and `exit 0` (never fail the evolver loop).
3. If a specific model class is unavailable, the script logs a downgrade warning and uses the next available class.
4. In harness-native fallback mode, the main session model (`claude-sonnet-4-6`) handles the call without class isolation.

```bash
# Standard guard pattern used in all evolver scripts
if ! command -v liter-llm > /dev/null 2>&1; then
  echo "[liter-llm] Not available — using harness-native model" >&2
  # Call host model inline (only when the operation is small enough)
  # For large operations: emit placeholder and exit 0
fi
```
