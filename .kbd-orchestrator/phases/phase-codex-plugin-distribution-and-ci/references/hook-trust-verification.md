# Hook-Trust Verification — change-cpd-006

_Executed 2026-07-12 against **codex-cli 0.144.1** — done headlessly (no interactive TUI)._

## Verdict: plugin hooks DO fire ✅ (and the real hooks needed a portability fix)

Codex was **not** actually a manual/interactive-only case — it ships a documented
headless bypass: `codex exec --dangerously-bypass-hook-trust` ("Run enabled hooks
without requiring persisted hook trust for this invocation… for automation that
already vets hook sources").

## Method

1. `npm run build:codex` → `codex plugin marketplace add .` → `codex plugin add prometheus-skill-pack@prometheus-skill-pack`.
2. Replaced the **cached** plugin's `hooks/hooks.json` with a clean `SessionStart`
   probe that writes a marker echoing `$PLUGIN_ROOT` / `$PLUGIN_DATA`.
3. `codex exec --dangerously-bypass-approvals-and-sandbox --dangerously-bypass-hook-trust "Reply with exactly the word: done"`.

## Evidence

- Codex log: `hook: SessionStart` → `hook: SessionStart Completed`.
- Marker written: `cpi006-fired at 13:02:01 with PLUGIN_ROOT=/Users/gqadonis/.codex/plugins/cache/prometheus-skill-pack/prometheus-skill-pack/1.6.0 PLUGIN_DATA=/Users/gqadonis/.codex/plugins/data/prometheus-skill-pack-prometheus-skill-pack`.
- Confirmed env: **`PLUGIN_ROOT`** = the install cache dir; **`PLUGIN_DATA`** = `~/.codex/plugins/data/<marketplace>-<plugin>` (created on first fire, not at install).

## Defect found + fixed

The pack's real `hooks/hooks.json` referenced **`${CLAUDE_PLUGIN_ROOT}`** (a Claude
var) in all 39 hook commands. Codex sets `PLUGIN_ROOT`, **not** `CLAUDE_PLUGIN_ROOT`,
so under Codex the real hooks would fire but run `bash /shared/scripts/…` (empty
path) and fail.

**Fix (this change):** `${CLAUDE_PLUGIN_ROOT}` → `${CLAUDE_PLUGIN_ROOT:-$PLUGIN_ROOT}`
across all 39 commands. Backward-compatible — under Claude Code `CLAUDE_PLUGIN_ROOT`
is set so behavior is identical; under Codex it falls back to `PLUGIN_ROOT`. JSON
re-validated.

## Caveats / notes

- The `--dangerously-*` flags are required for headless firing — an *interactive*
  `codex` session instead shows a one-time trust prompt (persisted after accept).
- Unrelated benign noise during the run: an MCP server with a relative URL logged
  `rmcp transport` errors, and a pre-existing `~/.codex/shell_snapshots/*.tmp`
  had a shell syntax error. Neither affected hook firing.
- Cleanup: the plugin + marketplace were removed from `~/.codex` after the test.
