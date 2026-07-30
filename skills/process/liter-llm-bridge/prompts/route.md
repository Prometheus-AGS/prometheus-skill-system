# liter-llm-bridge — Route

You are running the **route** phase. The goal is to document (or optionally activate) the contract by which KBD / iterative-evolver / artifact-refiner phases dispatch to the correct model class via the liter-llm MCP server.

## Model Selection

**Required model class: `small`**

This phase produces documentation and config — no reasoning. Read `project.json → model_policy.phases.liter-bridge-route`.

## The Routing Contract

Every KBD / evolver / refiner phase that emits a routing directive uses this format:

```
[MODEL_ROUTING] phase=<phase-key> class=<small|medium|frontier> model=<concrete-model> env=<environment>
```

External orchestrators parse the directive and dispatch the next phase to the matching model. There are three integration patterns:

### Pattern 1 — Native harness model switching (preferred)

If the harness supports per-dispatch model selection (Claude Code subagent `--model`, prom-lanes, UAR), it parses `[MODEL_ROUTING]` and dispatches directly. liter-llm is not in the loop. This skill's contribution is just the alias table — the harness reads `model_policy.registry.<class>.<env>` and switches.

Nothing to do in this case beyond ensuring the directive is emitted (already handled by KBD/evolver/refiner prompts).

### Pattern 2 — MCP-mediated routing (fallback)

If the harness does not support model switching but does support MCP, route via liter-llm:

1. The phase emits `[MODEL_ROUTING] class=medium ...`.
2. A harness hook intercepts dispatch.
3. The hook resolves the role to a `[[models]]` **name** with `kbd_resolve_role`, then
   POSTs OpenAI REST to the resolved gateway (`kbd_complete`). The MCP path uses the
   `chat` tool — **there is no `complete` tool and no `liter-llm complete` CLI.**
4. The response replaces what the host model would have produced.

Model names come from `~/.config/liter-llm/liter-llm-proxy.toml` (`[[models]]`) and roles
from `~/.prometheus/kbd/models.toml`. The old flat `[aliases]` table of
`small`/`medium`/`frontier` in `~/.config/liter-llm/config.toml` is **retired** — that
shape is not a schema liter-llm can load.

This requires opt-in installation of the routing hook. Do NOT auto-install — the user explicitly invokes `/liter-llm-bridge enable-routing-hook` to consent.

### Pattern 3 — Manual routing (last resort)

The user reads the directive in the phase output and re-invokes the next phase manually with the right model. Useful for one-off frontier-tier work; impractical for full pipelines.

## Hook Template (Pattern 2, Claude Code)

If the user opts in, write the hook to `~/.claude/hooks/liter-llm-route.sh`:

```bash
#!/usr/bin/env bash
# Intercept dispatched subagent prompts that carry a [MODEL_ROUTING] directive
# and route them through liter-llm instead of the host model.

set -euo pipefail

INPUT="$(cat)"
DIRECTIVE="$(echo "$INPUT" | grep -oE '\[MODEL_ROUTING\] class=[a-z]+' | head -1 | awk -F'=' '{print $2}')"

if [[ -z "$DIRECTIVE" ]]; then
  # No directive — pass through unchanged
  echo "$INPUT"
  exit 0
fi

# Strip the directive line so it doesn't pollute the prompt
PROMPT="$(echo "$INPUT" | grep -v '\[MODEL_ROUTING\]')"

# Dispatch over the REST gateway via the shared helper.
# NOT `liter-llm complete` / `liter-llm-cli complete` — neither exists. The binary
# ships only `api` and `mcp`; it is a proxy SERVER. Earlier versions of this template
# called a non-existent command, and because callers only checked that the binary was
# present, the failure read as "liter-llm unavailable".
. "${CLAUDE_PLUGIN_ROOT}/shared/scripts/lib/kbd-model-resolve.sh"

# $DIRECTIVE is a CLASS (small/medium/frontier). Map it to a role, then resolve the
# role to a real [[models]] name — never send the class name as a model id.
case "$DIRECTIVE" in
  frontier) ROLE=judge ;;
  *)        ROLE=critic ;;
esac

kbd_complete "$(kbd_resolve_role "$ROLE")" "" "$PROMPT" 2048 || {
  echo "[route] model call failed (see message above) — falling back to host model" >&2
  printf '%s' "$PROMPT"
}
```

Register the hook in `~/.claude/settings.json` under `hooks.PreToolUse` matching the dispatch tool. Document the exact integration in the harness's hook spec — Claude Code uses JSON over stdin/stdout, opencode and cursor differ.

## When NOT to Use Pattern 2

- The harness's MCP integration is read-only (cannot replace dispatched tool output)
- The phase requires the host model's specific tool/skill bindings (you can't fully replace a Claude Code subagent with a Groq completion)

In those cases, fall back to Pattern 3 or accept the host model.

## Activation Commands

- `/liter-llm-bridge enable-routing-hook` — install the Pattern 2 hook for the active harness, after explicit user confirmation
- `/liter-llm-bridge disable-routing-hook` — remove the hook, restore default dispatch
- `/liter-llm-bridge status` — print the current routing mode (native / MCP-mediated / manual) and the alias table

## Output

For documentation-only invocations (no `enable-routing-hook` flag), output:

```
ROUTE: documented
Mode: <native | mcp-mediated-not-installed | manual>
Aliases:
  small    → <provider/model>
  medium   → <provider/model>
  frontier → <provider/model>
To activate Pattern 2: /liter-llm-bridge enable-routing-hook
```
