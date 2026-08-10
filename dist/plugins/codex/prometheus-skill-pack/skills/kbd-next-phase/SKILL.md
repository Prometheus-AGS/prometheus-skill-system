---
license: MIT
name: kbd-next-phase
version: '1.0.0'
description: >
  Continue to the next KBD phase, automatically seeded from the previous
  phase's reflection. Reads the "Recommended Next Phase" section of
  reflection.md, initializes the new phase directory with goals.md and a
  skeleton progress.json, updates current-waypoint.json to point to
  /kbd-assess, and updates project.json active_phase.
metadata:
  tags: [process, orchestration, automation]
---

# /kbd-next-phase

Seed and initialize the next KBD phase from the previous phase's reflection.

## What this does

1. Reads `current-waypoint.json` to find the completed phase name and stage
2. Reads `reflection.md` — extracts "Recommended Next Phase" seed content
3. Creates `.kbd-orchestrator/phases/<new-phase>/` with:
   - `goals.md` — seeded goals from the reflection
   - `progress.json` — skeleton (all complete flags false)
4. Updates `current-waypoint.json`: stage → `assess_pending`, next → `/kbd-assess`
5. Updates `project.json` `active_phase` field
6. Outputs a confirmation banner with seeded content preview

## When to use

Run immediately after `/kbd-reflect` completes. This is the bridge between
phases — it automates reading the reflection, determining the next phase, and
preparing all state so any tool can resume cleanly with `/kbd-assess`.

Unlike `/kbd-new-phase` (which requires you to supply a name and goals
manually), `/kbd-next-phase` reads what KBD already knows from the reflection
and seeds the new phase automatically.

## Progress Signals (MANDATORY)

When the new phase is ready, emit:

```
Completed kbd-next-phase — <new-phase-name> ready for /kbd-assess
```

Emit to plain response text — no tool call needed.

## Prerequisites

- `/kbd-reflect` should have run for the current phase (warning issued if not)
- `reflection.md` should exist at `.kbd-orchestrator/phases/<phase>/reflection.md`
- The proposed new phase name must not already exist as a directory

## How to invoke

1. **Read current waypoint** — confirm current phase name and stage
2. **Warn if stage is not reflect_complete** — recommend running `/kbd-reflect` first
3. **Run the bundled `scripts/kbd-next-phase.sh`** — resolve it relative to this
   `SKILL.md` and pass `$ARGUMENTS` as the optional phase name. Do not look for
   the helper under a repository-level `shared/scripts/` directory; installed
   skill packages are self-contained.
4. **Display goals.md** — show the user what was seeded so they can review/edit
5. **Confirm next step** — remind user to run `/kbd-assess`

## Examples

```
/kbd-next-phase                                # auto-name from reflection
/kbd-next-phase skill-pack-upgrade-phase-2     # explicit name
/kbd-next-phase phase-3-realtime-2026-05-26    # explicit name with date slug
```

## Hook integration

`/kbd-next-phase` is the canonical phase-bracket transition. It MUST
fire `phase:before` for the new phase exactly once, after the new phase
directory is created and `current-waypoint.json` is flipped to it. (The
*closing* `phase:after` is the responsibility of `/kbd-reflect`, which
fires it for the previous phase before this skill is invoked.)

```sh
. "$KBD_ORCHESTRATOR_ROOT/shared/lib/waypoint.sh"
. "$KBD_ORCHESTRATOR_ROOT/shared/lib/hooks.sh"

# … seed new phase from previous reflection, flip waypoint …
kbd_hooks_fire phase before "$new_phase_name" 1 1
```

See orchestrator `SKILL.md` → "Hooks" for taxonomy and payload.
