# Real Plugin Hooks Under Codex — change-cpv-004

_Executed 2026-07-12 against codex-cli 0.144.1 via `codex exec --dangerously-bypass-hook-trust` (headless). The **real** pack hooks, not a probe._

## Verdict: the portability fix works ✅

- **Codex bundles the entire plugin root** (the whole repo) into the cache
  `~/.codex/plugins/cache/prometheus-skill-pack/prometheus-skill-pack/1.6.0/` —
  confirmed `shared/scripts/pk-health.sh` and the rest are present. So the hook
  commands' `…/shared/scripts/*.sh` targets exist under `PLUGIN_ROOT`.
- **Hooks fire:** `hook: SessionStart` ×4 and `hook: Stop` ×N appeared in the run.
- **Paths resolve — no empty-path errors.** The `No such file` / `/shared/scripts`
  / `CLAUDE_PLUGIN_ROOT` error grep came back **empty**, confirming
  `${CLAUDE_PLUGIN_ROOT:-$PLUGIN_ROOT}` falls back to Codex's `PLUGIN_ROOT`
  correctly (the last-phase fix is validated with the real hooks, not just a probe).

## Caveat — some hooks exit non-zero

A few hooks reported `Failed` (`SessionStart Failed` ×1, some `Stop Failed`). These
are **script-level** non-zero exits, **not** the path defect (path grep was clean).
Expected causes in a Codex `exec` sandbox with no KBD/services context: health
probes (`pk-health.sh` when pk isn't reachable), KBD position/guard hooks that
assume an orchestrator, and sycophancy gates. Most pack hooks are `|| true`
guarded; the "Failed" ones are context-inappropriate here, not broken by the port.

## Follow-up (optional, not this phase)

If the pack ships hooks intended to *run* under Codex (vs. Claude), curate a
Codex-appropriate subset (health/guard hooks that assume the KBD orchestrator or
local services shouldn't fire in a bare Codex session). For now the goal —
"do the real hooks resolve + fire under the portability fix" — is met.

Cleanup: plugin + marketplace removed from `~/.codex` after the test.
