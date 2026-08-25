---
license: MIT
name: kbd-new-phase
version: '1.0.0'
description: >
  Manually create a new top-level KBD phase. Accepts <name> [goals…] and
  initialises the phase directory, waypoint, project.json activePhase, and
  fires phase:before. Use this when no prior reflection exists (e.g. very
  first phase of a new project), when pivoting away from /kbd-next-phase's
  suggestion, or when initialising state by hand.
metadata:
  tags: [process, orchestration, automation]
---

# /kbd-new-phase

Create a fresh top-level KBD phase from scratch — the manual-entry
counterpart to `/kbd-next-phase`.

## What this does

1. Parses arguments — `<name>` plus zero or more `[goals…]`.
2. Validates the name (kebab-case, no path traversal, no slashes).
3. Refuses if `.kbd-orchestrator/phases/<name>/` already exists.
4. In runtime-authority mode, reads canonical status. If the lifecycle is
   `completed`, `cancelled`, or `failed`, starts exactly one operator-signed
   successor run before creating the requested phase.
5. Creates the phase directory and writes `goals.md` + `progress.json`
   atomically (temp file + `mv`).
6. Flips `current-waypoint.json`: `previousPhase ← prior phase`,
   `phase ← <name>`, `status ← assessment_ready`, …; preserves unknown
   fields untouched.
7. Updates `.kbd-orchestrator/project.json` `activePhase` (warns if absent).
8. Sources `shared/lib/hooks.sh` and fires `phase:before` exactly once for
   the new phase (best-effort — phase persists even if hooks subsystem is
   unavailable).
9. Emits the canonical Progress Signals and a confirmation banner.

## When to use

Run when **any** of:

- You're starting the very first phase of a project (no prior reflection
  exists).
- You're pivoting away from the suggestion in the previous phase's
  `reflection.md` (`/kbd-next-phase` is the auto-seed path).
- You're initialising state by hand and want one canonical entry point
  instead of editing `current-waypoint.json` directly.

Compare to `/kbd-next-phase`, which reads `reflection.md → "Recommended
Next Phase"` and auto-seeds the new phase from it.

## Progress Signals (MANDATORY)

Before any other action, emit:

```
Starting kbd-new-phase — <name>
```

When the new phase is ready, emit:

```
Completed kbd-new-phase — <name> ready for /kbd-assess
```

Emit to plain response text — no tool call needed.

## Prerequisites

- The proposed phase name must not already exist as a directory under
  `.kbd-orchestrator/phases/`.
- `current-waypoint.json`, if present, must be valid JSON. (If it is
  malformed, fix it by hand before retrying; the skill refuses to write
  on top of a corrupted waypoint to avoid compounding the corruption.)
- `jq` must be on `PATH`. (It's already a documented orchestrator
  dependency.)

## How to invoke

Either invoke the helper script directly (single entry point, runs the
whole workflow) or follow the step list below from inside an agent
session:

```sh
"$KBD_ORCHESTRATOR_ROOT/skills/kbd-new-phase/kbd-new-phase.sh" <name> [goal-1] [goal-2] …
```

Or, step-by-step:

1. Parse `$ARGUMENTS` → `name` + `goals[]`.
2. Validate `name` against `^[a-z0-9][a-z0-9._-]*$`; refuse `..`, `/`.
3. Refuse if `.kbd-orchestrator/phases/<name>/` exists.
4. When runtime authority is active, run `prometheus kbd status --json`. For
   terminal lifecycle state, run `prometheus kbd run start` with a unique run
   ID, a reason naming the prior run, and `/kbd-new-phase <name>` as exact next
   work. The CLI commits and projects the successor before releasing `PAUSE`.
5. `mkdir -p .kbd-orchestrator/phases/<name>`.
6. Write `phases/<name>/goals.md` atomically (`# Goals` heading + bullets,
   or `# Goals` + TBD stub when no goals supplied).
7. Write `phases/<name>/progress.json` atomically with the canonical field
   set (see `references/schemas/current-waypoint.template.json` and the
   "Nested phases" section of the orchestrator `SKILL.md`).
8. Rewrite `.kbd-orchestrator/current-waypoint.json` atomically. Existing
   unknown fields pass through; if the file is malformed, abort with a
   clear error and **do not** modify any on-disk state.
9. Update `.kbd-orchestrator/project.json` `activePhase` atomically, deleting
   the legacy `active_phase` alias; warn and continue if the file is absent
   (run `/kbd-init` later).
10. Source `shared/lib/waypoint.sh` and `shared/lib/hooks.sh` from
   `$KBD_ORCHESTRATOR_ROOT`, then call
   `kbd_hooks_fire phase before "$name" 1 1`. If the hooks subsystem is
   unavailable, emit one stderr warning and continue.
11. Print the confirmation banner: phase name, `goals.md` path, and
    `Next: /kbd-assess <name>`.

## Examples

```
/kbd-new-phase phase-1-foundation
/kbd-new-phase ux-refresh "polish dashboard" "ship dark mode" "audit a11y"
/kbd-new-phase ssed-followup-fixes
```

## Hook integration

`/kbd-new-phase` is a phase-bracket opening writer. It MUST fire
`phase:before` exactly once for the new phase, after the waypoint and
`project.json` flips, and before the `Completed kbd-new-phase` Progress
Signal — so any hook reading state sees the new phase as authoritative.

```sh
. "$KBD_ORCHESTRATOR_ROOT/shared/lib/waypoint.sh"
. "$KBD_ORCHESTRATOR_ROOT/shared/lib/hooks.sh"

# … validate, write phase dir, flip waypoint + project.json …
kbd_hooks_fire phase before "$name" 1 1
```

The closing `phase:after` is the responsibility of `/kbd-reflect` for
the *previous* phase, which should have run already.

See orchestrator `SKILL.md` → "Hooks" for taxonomy and payload, and
`references/schemas/current-waypoint.template.json` for the canonical
field set written into `progress.json` and `current-waypoint.json`.
