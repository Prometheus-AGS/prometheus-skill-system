---
license: MIT
name: kbd-reflect
version: '1.2.0'
description: >
  Generate the phase reflection report after all changes in the phase are
  complete. Seeds the next phase's planning inputs. Aggregates artifact-refiner
  QA results and reports back to iterative-evolver when running inside an
  evolution cycle.
metadata:
  tags: [process, orchestration, automation]
---

# /kbd-reflect

Run the **Reflect** phase of the KBD lifecycle.

## What this does

Generates `.kbd-orchestrator/phases/<phase-name>/reflection.md` summarizing:

- Goal achievement percentage (MET / PARTIAL / NOT MET per goal)
- Delivered changes (from progress.json and archive)
- **Artifact quality summary** (from artifact-refiner logs)
- Technical debt introduced
- Which recalled lessons (from `prior-context.md`) recurred or were applied
- Lessons captured for the knowledge base — one bullet per lesson, `[GLOBAL]` /
  `[USER]` prefixes for wider scopes; `reflect:after` writes them back through
  `memory-writeback.sh` (Codify as Skill? is never written back)
- Next Phase Seed for `kbd-next-phase`

## Contract accuracy (VLBV)

Reflection measures how accurate the phase's adopted contract was, so the next
phase's spec starts from real numbers. Read the counts from the closure packet,
`boundary-results.json`, the contract lineage and the KBD journal — never estimate:

| Metric | Value |
| --- | --- |
| Contract amendments after adoption | 1 (scope correction, signed) |
| BLOCKED caused by the contract (missing fixture, wrong entry point, unrunnable oracle) | 0 |
| NOT_EXERCISED scenarios | 1 (SCN-7, fixture unavailable) |
| Repair attempts used / allowed | 1 / 2 |
| Post-closure escapes (defects found after CLOSED that a scenario should have caught) | 0 |
| Static QA / review findings at B4 step 5 (critical / warning) | 0 / 3 |

Write the same block to `.kbd-orchestrator/phases/<phase>/accuracy.json`; the
prior-context writer carries it into the next phase so discovery and spec review
cite it. Recurring constraint violations from the B4 step-5 refiner pass are listed
underneath, by constraint and file — not by change.

## Evolver Feedback

When this phase is part of an iterative-evolver cycle:

1. **Read evolver-bridge.json** — `.kbd-orchestrator/phases/<phase>/evolver-bridge.json`
2. **Map change completion to evolver items** — using `item_to_change_map`
3. **Write evolver execution results** — update `.evolver/evolutions/<name>/state.json`:
   - Set `execution_results` with per-item completion status
   - Include the contract-accuracy metrics
4. **Trigger evolver reflect** — the outer evolver loop picks up from here

This bridges the KBD inner loop back to the iterative-evolver outer loop,
enabling the evolver's Reflect phase to measure actual progress against
strategic goals.

## Prerequisites

The phase must have completed the B4 sequence (`references/boundaries.md`):

- Every change implemented (`implementation_status: COMPLETE` in `progress.json`)
- The boundary packet ran once (plus any authorized B5 repair) and
  `boundary-results.json` exists
- The cumulative B4 step-5 QA and adversarial review ran (or carry a signed waiver)
- `kbd-phase-close-eval.sh` wrote the closure object and `closure-packet.md`
- After CLOSED: OpenSpec changes verified and archived through `kbd-apply`; native
  changes moved to `.kbd-orchestrator/changes/archive/<date>-<id>/`

These are separate prerequisites: implementation completion drives the N/N
counter, while the boundary verdicts, QA, verification and archival drive
evidence, certification and publication/lifecycle fields. A phase closed unmet
(FAIL or BLOCKED) is still reflected; record what remains.

## Progress Signals (MANDATORY)

**FIRST tool call of every turn:** Read `.kbd-orchestrator/position-reminder.txt` (if it exists) to get the current phase, step N of T, and next command. If that file is absent, read `.kbd-orchestrator/current-waypoint.json`.

Before any other action, emit to plain response text (BEFORE any tool call):

```
Starting kbd-reflect — <phase-name> (step N of T)
```

When all steps are complete, emit:

```
Completed kbd-reflect — <phase-name> (step N of T)
```

**How to get N and T (MANDATORY — never estimate):**
- Read `.kbd-orchestrator/phases/<phase>/progress.json` →
  `completion.implementation.completed` = N and `.total` = T; fall back to
  legacy `changes_completed` / `changes_total` only when canonical fields are absent.
- If `progress.json` is absent, read `current-waypoint.json` →
  `implementationCompleted` / `implementationTotal`, then legacy aliases.

Use the canonical phase name from the argument or `current-waypoint.json`. Emit to plain response text — no tool call needed.

## How to invoke

1. **Read prior context and cite lessons** — read
   `.kbd-orchestrator/phases/<phase>/prior-context.md` (refreshed by the
   `reflect:before` hook). The reflection's `## Recalled Lessons` section states
   which recalled lessons **recurred** this phase and which were applied;
   recurrences belong in Root Cause.
2. **Discover project identity**
3. **Confirm the active phase** — from argument or waypoint
4. **Reconcile, then read `progress.json`** — run
   `kbd-apply reconcile <phase>` first. Exit **0** permits reading the
   reconciled progress. Exit **1** reports drift, which may include ledger-ahead,
   cancelled or archived discrepancies; do not describe all drift as undelivered
   code. Use `--repair` only for unambiguous active-phase backend-complete
   tasks. Exit **2** means invalid input or an incomplete scan: report its errors
   and restore readable input/runtime authority rather than claiming clean.
   Reconciliation supports archived changes and does not change live state.
5. **Read artifact-refiner logs** — aggregate QA results
6. **Load all change data** — from `openspec/changes/archive/` if OpenSpec,
   or `.kbd-orchestrator/changes/archive/` if native KBD
7. **Follow the reflect protocol** in `../prompts/reflect.md`
8. **Write reflection** to `.kbd-orchestrator/phases/<phase>/reflection.md`
9. **Review promotion candidates** — run `pk candidates list --kind promotion`
   (pk >= 1.11.0; skip silently when `pk` or the `candidates` subcommand is
   absent) and present each pending candidate to the human with its scope,
   reasons and evidence count. **Accept or reject only on an explicit human
   instruction** — `pk candidates accept <id>` or `pk candidates reject <id>
   [--reason ...]`. Never accept or reject on your own judgement, and never
   auto-accept to clear the list; an unreviewed candidate stays pending.
   Lessons you already marked `[GLOBAL]` / `[USER]` in the reflection promote
   immediately through `memory-writeback.sh`; candidates are the *proposed*
   promotions that still need a human decision.
10. **Review skill candidates** — run `pk candidates list --kind skill`
    (pk >= 1.11.0; skip silently when `pk` or the `candidates` subcommand is
    absent). Each pending candidate is a `new-skill` (a workflow seen across
    sessions or projects that no skill covers) or a `skill-update` (a skill users
    corrected after it ran). Present each with its full id, type, title and
    evidence count, and note any that answer the reflection's `## Codify as
    Skill?` section. **Accept or reject only on an explicit human instruction**
    — `pk candidates accept --kind skill <id>` (add `--update <skill>` to target
    an installed skill) or `pk candidates reject --kind skill <id> [--reason
    ...]`. Accept only prints the `/pmpo-skill-creator` invocation and the
    evidence path; it never creates or edits a skill, and neither do you at this
    step — the human runs the printed command. An unreviewed candidate stays
    pending.
11. **If evolver bridge exists**: write execution results back to evolver state
12. **Advance only when authorized** through `kbd-next-phase`. For a contract-bound phase that authorization is an operator-signed start-successor manifest (`kbd-next-phase.sh --authorize-successor <manifest> --signature <sig>`); closure alone never starts a successor, and reflection cannot supply that authority
13. **Trigger**: `echo '[kbd] Reflection complete — advance to next phase with /kbd-new-phase'`

## Examples

```
/kbd-reflect                             # uses active waypoint phase
/kbd-reflect phase-1-foundation          # explicit phase name
```

## Hook integration

`kbd-reflect` is the canonical *phase-end* boundary. After the
reflection report is written, fire `reflect:after`, then immediately
fire `phase:after` for the closing phase. `phase:before` for the *next*
phase is fired by `kbd-new-phase` / `kbd-next-phase`, not here.

```sh
. "$KBD_ORCHESTRATOR_ROOT/shared/lib/waypoint.sh"
. "$KBD_ORCHESTRATOR_ROOT/shared/lib/hooks.sh"
. "$KBD_ORCHESTRATOR_ROOT/shared/lib/bottleneck-guard.sh"

kbd_hooks_fire reflect before "$phase" 1 1
# … write reflection.md …
kbd_hooks_fire reflect after  "$phase" 1 1
bound="$(prometheus kbd --path . phase disposition --id "$phase" --json | jq -r '.contractBound')" || exit 1
if [ "$bound" = "true" ]; then
  # Contract-bound phase (issue #192): only the terminal evaluator closes it,
  # against the bound contract snapshot. 0 = CLOSED (or identical no-op);
  # 2/3 = BLOCKED/FAIL recorded, phase stays open; 5 = writer failed;
  # 6 = already closed under other evidence. Report the disposition and stop.
  bash "$PACK_ROOT/shared/scripts/kbd-phase-close-eval.sh" --phase "$phase" || exit $?
else
  export KBD_BOTTLENECK_PATH="${KBD_BOTTLENECK_PATH:-.}"
  guard_enabled=0
  if kbd_bottleneck_active; then
    guard_enabled=1
    kbd_bottleneck_evaluate phase after "$phase" 1 >/dev/null || exit 1
  fi
  prometheus kbd --path . phase transition \
    --command-id "reflect-complete:$phase" \
    --id "$phase" --status complete || exit 1
  if [ "$guard_enabled" = "1" ]; then
    kbd_bottleneck_evaluate phase after "$phase" 0 >/dev/null || exit 1
  fi
fi
kbd_hooks_fire phase   after  "$phase" 1 1   # canonical phase-end boundary
```

`$PACK_ROOT` is the package root (`kbd_package_root` from
`shared/lib/package-root.sh`). A contract-bound phase is never moved to
`complete` with `phase transition`; the runtime refuses that edge and only a
PASS disposition closes it. Reflection text, review findings and the
`## Next Phase Seed` are proposals: they never add requirements to this phase
and never start a successor.

Both phase guard evaluations and the phase transition (or, for a bound phase,
the evaluator's CLOSED result) must succeed before `phase:after` fires. This ordering prevents a rejected or partial reflection
from producing a completion memory record. `kbd-next-phase` reads canonical
phase status and skips this terminal transition and completion hook when
reflection has already completed the phase.

See orchestrator `SKILL.md` → "Hooks" for taxonomy and payload.

## Stage gate & handoff

The reflect gate requires the execute handoff. After the reflection report
is written (and passes the sycophancy gate), record the closing handoff —
`kbd-next-phase` seeds the next phase from it plus `reflection.md`:

```sh
. "$KBD_ORCHESTRATOR_ROOT/shared/lib/stage-gate.sh"

kbd_stage_gate reflect || exit 2
# … write reflection.md …
kbd_stage_handoff_write reflect "<1–3 sentences: deltas found, corrective actions, recommended next phase>" reflection.md
```

A missing `handoffs/` directory does not bypass required predecessors.
A missing required handoff fails with remediation: complete the predecessor
stage, or record an explicit skip with its reason under project policy.
A deliberate stage skip is recorded with `kbd_stage_handoff_skip <stage>
"<reason>"`. Schema: `references/schemas/handoff.schema.json`.
