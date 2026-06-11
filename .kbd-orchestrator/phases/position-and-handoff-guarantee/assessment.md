# Assessment — position-and-handoff-guarantee

Source: deep framework-evolution assessment (2026-06-11), sycophancy-gated at `strict`
(score 0.0). Full plan: `~/.claude/plans/i-want-to-do-staged-pinwheel.md` (approved).

## Gaps this phase addresses

| ID | Gap | Evidence |
|----|-----|----------|
| F1 | Progress signaling is prose-only; nothing injects or verifies position per turn. User loses place in long sessions. | `references/per-turn-position-hook.md` documents an *optional* settings hook; `hooks/hooks.json` has no position hook. |
| F2 | No machine-readable unified position model. Waypoint mixes legacy snake_case and camelCase keys; `.evolver/`/`.zeespec/` state invisible to KBD status. | `.kbd-orchestrator/current-waypoint.json` carries both `exact_next_command` (stale) and `exactNextCommand` (current). |
| F3 | Stage transitions have no handoff artifacts or precondition gates; pipeline-enforce.sh covers only execute/reflect ordering for Bash invocations. | `shared/scripts/pipeline-enforce.sh`; no `handoffs/` anywhere under `.kbd-orchestrator/phases/`. |
| F4 | No CI verification that process skills declare progress signals. | `scripts/` has validate-skills.js only; signal rule lives in SKILL.md prose. |

## Constraints

- Hook scripts follow house conventions: source `shared/scripts/lib/hook-log.sh`,
  graceful degradation (exit 0 when state absent), JSONL logging.
- UserPromptSubmit injection must never block a prompt (always exit 0).
- Stop gate must respect `stop_hook_active` and soft-cap at 1 block per turn.
- Waypoint readers must handle camelCase-first with snake_case fallback.
- Honest ceiling: hooks cannot force prose; guarantee = inject-every-turn + one
  enforced retry at Stop. Documented, not oversold.

## Verdict

GO — all four gaps are addressable with new scripts + mechanical skill edits;
no schema-breaking changes required (position.json is additive and derived).
