---
license: MIT
name: kbd-reflect
description: >
  Generate the phase reflection report after all changes in the phase are
  complete. Seeds the next phase's planning inputs. Aggregates artifact-refiner
  QA results and reports back to iterative-evolver when running inside an
  evolution cycle.
metadata:
  version: '1.2.0'
  tags: "process, orchestration, automation"
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

For the ordinary acceptance path, read the adopted contract, B4/B5 boundary
results, independent review, closure packet and archive status. Report
implementation, acceptance, certification and publication independently.

For a signed `CLOSED_SUPERSEDED` or `CLOSED_UNMET` exit, read the canonical
`phase disposition --id <phase> --json` response. Use
`.closure.operatorExit` for the operator decision, exercised verdicts and
evidence, withdrawals and destinations, budget class and explicit limits. Fill
the goal table from those facts. List each withdrawal with its destination in
Next Phase Seed. An unexercised goal has no passing verdict. A refused phase
remains open.

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
- Run `kbd-apply reconcile <phase> --json`. In a runtime-owned project report
  `taskCounts.runtime.completed/total` and show the projected and backend counts
  when they disagree. An incomplete scan leaves the backend count unknown.
- For a legacy file-owned phase use the projection's implementation count and
  label it as a change count.

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
   tasks and exact-title OpenSpec checkbox repairs. The runtime owns its generated
   progress view. If its counts disagree, inspect `prometheus kbd migrate-projections
   --dry-run` and the reported task identities before selecting a projection
   migration. Exit **2** means invalid input or an incomplete scan: report its errors
   and restore readable input/runtime authority rather than claiming clean.
   Reconciliation supports archived changes and does not change live state.
5. **Read artifact-refiner logs** — aggregate QA results
6. **Load all change data** — from `openspec/changes/archive/` if OpenSpec,
   or `.kbd-orchestrator/changes/archive/` if native KBD
7. **Follow the reflect protocol** in `../prompts/reflect.md`
8. **Write reflection** to the phase directory and run the packaged finalizer in Hook integration.
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
13. Report the recorded disposition, task counts, and remaining work.

## Examples

```
/kbd-reflect                             # uses active waypoint phase
/kbd-reflect phase-1-foundation          # explicit phase name
```

## Hook integration

From the consumer project, resolve the orchestrator relative to this skill.
In a flat installation it is the sibling `kbd-process-orchestrator` directory.
In a nested installation it is two directories above this skill. Set
`KBD_ORCHESTRATOR_ROOT` to the directory containing `shared/lib/hooks.sh`.

Before collecting prior context, source that library and run
`kbd_hooks_fire reflect before "$phase" 1 1`. After writing the report, run:

```sh
bash "<kbd-reflect skill directory>/scripts/finalize.sh" reflect "$phase"
```

This command checks the reflection gate, delivers reflection writeback, and
uses the typed evaluator for an open bound phase or the guarded transition for
a legacy phase. A recorded terminal disposition delivers phase completion
against the closing phase's identity. Hook status and retry history live beside
the report. A failed auxiliary action can be retried by rerunning the command.
A successor still requires its own authority.

## Stage gate & handoff

The ordinary reflect gate requires the execute handoff. Signed operator exits use the recorded terminal disposition when the predecessor work was withdrawn. After the reflection report
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
