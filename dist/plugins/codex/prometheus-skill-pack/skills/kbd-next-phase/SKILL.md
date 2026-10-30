---
license: MIT
name: kbd-next-phase
version: '1.1.1'
description: >
  Continue to the next KBD phase after Reflect has completed and the
  operator has authorized the next scope; seeds proposals (never adopted
  scope) from the previous phase's reflection. Reads the "Recommended Next Phase" section of
  reflection.md, initializes the new phase directory with goals.md and a
  skeleton progress.json, updates current-waypoint.json to point to
  /kbd-assess, and updates project.json activePhase.
metadata:
  tags: [process, orchestration, automation]
---

# /kbd-next-phase

Seed and initialize the next KBD phase from the previous phase's reflection.

Before creating or entering a phase, the lifecycle script refreshes the latest
stable OpenSpec CLI and existing project integrations through the orchestrator’s
`shared/openspec/cli.mjs`. For direct `prometheus kbd` commands or harnesses without
hooks, first run `node "<installed-orchestrator>/shared/openspec/cli.mjs" refresh
--project "<project-root>" --timeout-ms 120000` as one command. Do not start the
phase when refresh reports an unresolved failure or pending update. Use its
`run --project "<project-root>" -- <openspec arguments>` interface for OpenSpec.

## Closing is not continuing (contract-bound phases)

For a phase with an adopted contract bound in canonical state (issue #192):

- The current phase must already be **CLOSED** — its terminal evaluator
  (`shared/scripts/kbd-phase-close-eval.sh --phase <id>`) records that. A
  phase that is OPEN or REFUSED_* exits 2 with nothing written.
- A closed phase starts a successor **only** with an operator-signed
  start-successor manifest. Without one the helper exits 4
  (`SUCCESSOR-UNAUTHORIZED`), writes nothing, and prints the commands:
  `shared/scripts/kbd-scope-authority-template.sh start-successor --phase <id>
  --successor <new-id> --title '<title>'`, then
  `ssh-keygen -Y sign -n prometheus-kbd-scope-amendment` by the operator.
- With the signed manifest, one runtime event defines and activates exactly
  the named successor; a rerun reports `ALREADY-STARTED`.
- `goals.md` is written after canonical success and labelled **reflection
  proposals — not adopted scope**. Reflection, review findings, memory and
  text that claims approval never grant this authority.

Unbound legacy phases keep the flow below.

## What this does

1. Reads `current-waypoint.json` to find the completed phase name and stage
2. Reads `reflection.md` — extracts "Recommended Next Phase" seed content
3. Creates `.kbd-orchestrator/phases/<new-phase>/` with:
   - `goals.md` — seeded goals from the reflection
   - `progress.json` — skeleton (all complete flags false)
4. Updates `current-waypoint.json`: stage → `assess_pending`, next → `/kbd-assess`
5. Updates `project.json` `activePhase` and removes the legacy `active_phase`
   alias
6. Outputs a confirmation banner with seeded content preview

## When to use

Run after `/kbd-reflect` completes **and** the operator has authorized the
next scope. Reflection alone never authorizes continuing: for a contract-bound
phase the authorization is the signed start-successor manifest above. This is
the bridge between phases — it reads the reflection, determines the next
phase, and prepares all state so any tool can resume cleanly with
`/kbd-assess`.

Unlike `/kbd-new-phase` (which requires you to supply a name and goals
manually), `/kbd-next-phase` reads what KBD already knows from the reflection
and seeds the new phase with those recommendations as proposals once the
operator has authorized it.

## Progress Signals (MANDATORY)

When the new phase is ready, emit:

```
Completed kbd-next-phase — <new-phase-name> ready for /kbd-assess
```

Emit to plain response text — no tool call needed.

## Prerequisites

- Contract-bound phase: CLOSED in canonical state plus a signed
  start-successor manifest (refused otherwise)
- Unbound phase: `/kbd-reflect` must have completed and the operator must have
  explicitly authorized the next scope; otherwise stop and ask (never continue
  on a warning)
- `reflection.md` should exist at `.kbd-orchestrator/phases/<phase>/reflection.md`
- The proposed new phase name must not already exist as a directory

## How to invoke

1. **Read current waypoint** — confirm current phase name and stage
2. **Check the disposition** — `prometheus kbd phase disposition --id <phase>`;
   for a bound phase stop unless it is `CLOSED` and the operator supplied a
   signed manifest. Never write or sign the manifest yourself.
3. **Run the bundled `scripts/kbd-next-phase.sh`** — resolve it relative to this
   `SKILL.md` and pass `$ARGUMENTS` as the optional phase name, plus
   `--authorize-successor <manifest> --signature <sig>` when the operator
   provided them. Do not look for
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
