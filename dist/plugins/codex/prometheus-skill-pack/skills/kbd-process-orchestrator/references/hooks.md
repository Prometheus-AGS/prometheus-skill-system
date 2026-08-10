# KBD Hooks

> Extracted from the orchestrator SKILL.md. The hook surface fired around every KBD lifecycle boundary — event taxonomy, discovery order, per-fire context, wiring, and debugging.

KBD ships an extensible hook surface fired around every lifecycle boundary.
Each KBD skill emits a hook event before its work and another after, and any
project can plug in either *augment* (adds behavior) or *override* (replaces
the default) entries.

**Canonical event form**: `<kind>:<edge>`, where

- `kind` ∈ `phase` | `child` | `plan` | `execute` | `reflect` | `task` | `assess` | `*`
- `edge` ∈ `before` | `after` | `*`

**Legacy alias compatibility** (kept working — no migration required):

| Legacy event | Canonical event |
|---|---|
| `on_phase_complete` | `phase:after` |
| `on_plan_complete` | `plan:after` |
| `on_reflection_complete` | `reflect:after` |
| `on_assessment_complete` | `assess:after` |
| `on_change_complete` | last `task:after` of the change (sentinel — fires when `index == total`) |
| `<kind>:begin` | `<kind>:before` |
| `<kind>:end` | `<kind>:after` |
| `on_blocker_detected`, `on_cross_tool_handoff` | unchanged — situational, not lifecycle |

**Discovery order** (highest precedence wins on override conflicts):

1. **builtin** — `~/.claude/skills/kbd-process-orchestrator/hooks/hooks.json`
2. **user** — `~/.claude/skills/kbd-process-orchestrator/hooks/user.json` (optional)
3. **project** — `.kbd-orchestrator/hooks-config.json` (optional)

**Mode field** — each entry declares `mode: augment` (default) or `mode: override`. Multiple overrides on the same `(kind, edge)` resolve to the entry from the highest layer; within a layer, last-loaded wins. A single warning names winner and losers; dispatch never aborts.

**Default reporter** — the built-in `report-progress` hook fires on every `*:*` and writes to stderr:

```
starting <kind> <name> [<index>/<total>]
ending <kind> <name> [<index>/<total>]
```

A project can replace this wholesale with a single `mode: "override"` entry covering `"*:*"`.

> **Guarantee vs. extension (read this).** The stderr reporter is **not** what
> the user sees each turn — Claude Code does not surface hook stderr into the
> conversation. The user-facing guarantee is the **plain-text Progress Signals**
> that every skill emits and that `/kbd-apply` emits per task. The
> `report-progress` hook is the *extension point* (telemetry, memory mirror,
> custom reporters). For the full design — including an opt-in `Stop` settings
> hook that injects the phase chain regardless of which skill is active — see
> `references/per-turn-position-hook.md`.

**Per-fire context** — every hook command receives these environment variables (in addition to existing `${PHASE}`, `${STEP}`, `${EVENT}` substitutions):

| Variable | Meaning |
|---|---|
| `KBD_HOOK_KIND` | phase / child / plan / execute / reflect / task / assess |
| `KBD_HOOK_EDGE` | before / after |
| `KBD_HOOK_NAME` | active item's canonical name |
| `KBD_HOOK_INDEX` | 1-based index in the containing loop (default 1) |
| `KBD_HOOK_TOTAL` | total count in the containing loop (default 1) |
| `KBD_HOOK_PHASE_PATH` | rendered chain via `chain_separator` |
| `KBD_HOOK_CHILD_PATH` | active child name, or empty |
| `KBD_HOOK_SOURCE_TOOL` | sourceTool from waypoint, or `"unknown"` |
| `KBD_HOOK_STARTED_AT` | ISO-8601 UTC timestamp |

**Hook log** — every fire appends one JSON object to `.kbd-orchestrator/phases/<phase>/hooks.log.jsonl` (or `.kbd-orchestrator/hooks.log.jsonl` when no phase is active yet). Schema:

```json
{"ts":"…","kind":"task","edge":"after","name":"1.1 …","index":1,"total":7,
 "phasePath":"parent › child","sourceTool":"claude-code",
 "hookId":"project/on-task-done","layer":"project","mode":"augment","status":0}
```

Failure entries include `"stderrSnippet"` truncated to 200 chars.

### Wiring stanza (paste into each KBD skill)

```sh
. "$KBD_ORCHESTRATOR_ROOT/shared/lib/waypoint.sh"
. "$KBD_ORCHESTRATOR_ROOT/shared/lib/hooks.sh"

kbd_hooks_fire <kind> before "$item_name" "$index" "$total"
# … do the work …
kbd_hooks_fire <kind> after  "$item_name" "$index" "$total"
```

The existing `Starting/Completed kbd-<skill> — <phase>` Progress Signals
(documented in each skill's "Progress Signals (MANDATORY)" section)
**continue to fire** alongside hook events; the two are complementary.
Progress Signals are agent-facing structured lines; hook output is
operator-facing observability.

### Debugging

Set `KBD_HOOK_DEBUG=1` in the environment to log every event-name
normalisation to stderr (`[hooks] normalised <orig> → <canonical>`).

