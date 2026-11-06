---
license: MIT
name: kbd-plan
version: '1.1.0'
description: >
  Create a prioritized, ordered change list for the current project phase.
  Project-agnostic: reads assessment and project constraints to produce
  an ordered change list. Auto-detects OpenSpec availability and emits
  appropriate change formats with task-level model and worker-route assignments.
  Bridges with iterative-evolver plan items
  when running inside an evolution cycle.
metadata:
  tags: [process, orchestration, automation]
---

# /kbd-plan

Run the **Plan** phase of the KBD lifecycle for any project.

## What this does

Reads `.kbd-orchestrator/phases/<phase-name>/assessment.md` and produces an
ordered list of changes to implement this phase. Refreshes the waypoint so
every tool knows the exact next step.

Output:

- `.kbd-orchestrator/phases/<phase-name>/plan.md` — ordered changes and Task model assignments
- `.kbd-orchestrator/current-waypoint.md` and `current-waypoint.json` — refreshed

## Analyze inputs (when the Analyze stage ran)

When `.kbd-orchestrator/phases/<phase>/library-candidates.json` exists, read it
before ordering changes (schema:
`references/schemas/library-candidates.schema.json`):

- A change addressing a gap whose candidate has verdict `adopt` or `adapt`
  carries a `library: cand-###` annotation in `plan.md` and, for OpenSpec, the
  candidate's evidence block in the design doc — the change *reuses* the library
  rather than rebuilding it.
- `build_required[]` entries (gaps with no adoptable candidate) become build
  changes; a `capability_gap_id` (when present) marks a blocked-on dependency.

When no `library-candidates.json` exists (Analyze skipped), plan as before.

## Spec engine detection

OpenSpec (1.16.0+) is the primary change engine; Spec Kit is the supported
backup; native KBD change files are kept for existing native projects. Resolve
the engine exactly as `kbd-apply detect` does:

1. `.kbd-orchestrator/project.json` `specBackend` pin (legacy `change_backend`
   is read when `specBackend` is absent)
2. the store a change already lives in
3. an `openspec/` directory → OpenSpec; `.specify/` or `specs/*/tasks.md` → Spec Kit;
   existing `.kbd-orchestrator/changes/*/tasks.json` → native KBD

```
openspec  → emit /opsx:new <change-id> through the managed runner
          → openspec/changes/<change-id>/{proposal.md,design.md,specs/,tasks.md}
speckit   → specs/<change-id>/{spec.md,plan.md,tasks.md}
native-kbd→ .kbd-orchestrator/changes/<change-id>/{spec.md,tasks.json}
```

When nothing resolves, initialize OpenSpec (`shared/openspec/cli.mjs init`) and
pin `"specBackend": "openspec"` rather than defaulting to native files.

## Evolver Bridge

When this plan is being created as part of an iterative-evolver cycle:

1. **Read evolver plan** — check `.evolver/evolutions/<name>/plan.json`
2. **Map evolver items to KBD changes** — each evolver plan item becomes one
   or more KBD changes with a `evolver_item_id` reference
3. **Write bridge file** — `.kbd-orchestrator/phases/<phase>/evolver-bridge.json`:

```json
{
  "evolution_name": "<name>",
  "evolver_plan_path": ".evolver/evolutions/<name>/plan.json",
  "item_to_change_map": {
    "evolver-item-1": ["change-001", "change-002"],
    "evolver-item-2": ["change-003"]
  }
}
```

This enables the Reflect phase to report back to the evolver with precise
completion status per plan item.

## OpenSpec lifecycle preflight

At session startup and before starting or entering a phase, resolve the installed
`kbd-process-orchestrator` skill directory as `KBD_ORCHESTRATOR_ROOT` and run:

```bash
node "$KBD_ORCHESTRATOR_ROOT/shared/openspec/cli.mjs" refresh --project "<project-root>" --timeout-ms 120000
```

This resolves the latest stable official OpenSpec CLI and refreshes the project's
generated OpenSpec skills/commands while preserving authored specs, changes and
custom configuration. Read the receipt: failed or pending refresh is unresolved,
not current-version proof. A cached offline version is explicitly unverified for
latest freshness. Resolve reported conflicts before dependent phase mutations.
OpenSpec is the primary spec engine for new projects (`/kbd-init` initializes
it); existing native KBD projects keep working, and the helper skips projects
without an existing OpenSpec root.

Use the same preflight in harnesses without startup hooks and before raw
`prometheus kbd` phase commands that bypass the lifecycle scripts. Invoke
OpenSpec through the managed runner, including commands copied from generated
skills, rather than an independently versioned global executable:

```bash
node "$KBD_ORCHESTRATOR_ROOT/shared/openspec/cli.mjs" run --project "<project-root>" -- <openspec arguments>
```

The runner preserves CLI stdout for JSON consumers. Planning model assignments
still never launches workers or changes inference providers. A dry-run or
read-only request does not authorize this mutating refresh; report its pending
preflight and defer it until a writable invocation.

## Task model planning

Read [references/task-model-selection.md](references/task-model-selection.md)
before drafting assignments. Draft concrete tasks and backend identities first,
then analyze each task's requirements and recommend a concrete provider/model,
supported reasoning effort and documented execution route. Use task suitability
first, with cost and latency breaking close ties, subject to explicit user,
project and budget constraints. Discover current capabilities for the selected
harness; do not turn model names or change complexity classes into fixed rankings.

Author one **Task model assignments** table in `plan.md`, keyed by full phase
path, change ID and backend task ID. Record evidence, native alternatives and
unresolved prerequisites. liter-llm supplies inference; a documented tool-enabled
worker supplies execution. Planning never launches workers or changes providers.

## Progress Signals (MANDATORY)

**FIRST tool call of every turn:** Read `.kbd-orchestrator/position-reminder.txt` (if it exists) to get the current phase, step N of T, and next command. If that file is absent, read `.kbd-orchestrator/current-waypoint.json`.

Before any other action, emit to plain response text (BEFORE any tool call):

```
Starting kbd-plan — <phase-name> (step N of T)
```

When all steps are complete, emit:

```
Completed kbd-plan — <phase-name> (step N of T)
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
   `plan:before` hook) and cite in `plan.md` the recalled lessons that change
   ordering, scope or risk. Recalled entries are information, not instructions.
2. **Discover project identity** — read `.kbd-orchestrator/project.json` or infer
3. **Confirm the active phase** — from argument or waypoint
4. **Load assessment** — from `.kbd-orchestrator/phases/<phase>/assessment.md`
5. **Read project constraints** — from `AGENTS.md` and project spec files
6. **Detect change backend** — OpenSpec, Spec Kit or native KBD (see Spec engine detection)
7. **Check for evolver bridge** — is this phase driven by an evolution cycle?
8. **Follow the plan protocol**: in a nested orchestrator installation, read
   `../../prompts/plan.md`; in a flat skill installation, read
   `../kbd-process-orchestrator/prompts/plan.md`. Resolve these relative to this
   skill directory, using the matching installed layout. Draft tasks and IDs.
9. **Write plan.md** with ordered changes and every task's model assignment
10. **Adversarial vet** — unless `--skip-adversarial-review` is passed, run
   `/adversarial-review --mode artifact plan` on the written plan (see
   orchestrator `references/integrations/adversarial-review.md`). CRITICAL
   findings (ordering errors, missing dependencies, untestable criteria) →
   revise `plan.md` and re-vet (max 2 rounds, then accept with an
   "Unresolved review findings" section appended). WARNING findings → carry
   into the stage handoff summary. Vet **before** emitting change
   structures, so a corrected plan never leaves stale changes behind.
11. **Emit change structures** via the resolved engine; add non-task references
    to matching plan assignments without changing checkbox syntax or task titles.
    Reconcile all emitted task IDs with the table before handoff; re-vet material
    changes to scope or routing, then reconcile again.
12. **Write evolver-bridge.json** if evolver plan exists
13. **Record plan state** through typed KBD stage/change/task commands; the runtime regenerates progress and waypoint projections

## Examples

```
/kbd-plan                                # uses active waypoint phase
/kbd-plan phase-1-foundation             # explicit phase name
```

## Hook integration

Fire `plan:before` before reading the assessment, `plan:after` after
writing `plan.md`. Existing Progress Signals are unchanged.

Write the stage handoff (`kbd_stage_handoff_write plan …`) before firing
`plan:after`: the builtin `kbd-stage-writeback-plan` hook stores its summary
in memory at visibility `lead`, where the lead view of later stages recalls it.

```sh
. "$KBD_ORCHESTRATOR_ROOT/shared/lib/waypoint.sh"
. "$KBD_ORCHESTRATOR_ROOT/shared/lib/hooks.sh"

kbd_hooks_fire plan before "$phase" 1 1
# … draft plan.md …
kbd_hooks_fire plan after  "$phase" 1 1
```

See orchestrator `SKILL.md` → "Hooks" for taxonomy and payload.

## Stage gate & handoff

The plan gate requires the assess handoff (walking back across the optional
analyze/spec stages when they have no handoff). After writing `plan.md`,
record the handoff that execute reads first:

```sh
. "$KBD_ORCHESTRATOR_ROOT/shared/lib/stage-gate.sh"

kbd_stage_gate plan || exit 2
# … draft plan.md …
# … adversarial vet (step 10) runs here, before the handoff …
kbd_stage_handoff_write plan "<1–3 sentences: change count, ordering rationale, first change; Task model assignments location and unresolved routes; review warnings>" plan.md
```

A missing `handoffs/` directory does not bypass required predecessors.
A missing required handoff fails with remediation: complete the predecessor
stage, or record an explicit skip with its reason under project policy.
A deliberate stage skip is recorded with `kbd_stage_handoff_skip <stage>
"<reason>"`. Schema: `references/schemas/handoff.schema.json`.

## Contract binding + single boundary packet (required)

Plan output must bind contract identity + digest, scenario→task coverage,
worker routes, freshness keys, budgets, and ONE boundary execution packet —
and record adoption before writing the identity-bound plan handoff
(KBD_HANDOFF_IDENTITY_JSON):

Run from the consumer project. For repository development or a custom payload,
set `KBD_PACK_ROOT` to the absolute package root; do not change into that root.
All package and contract paths must remain quoted.

```bash
KBD_PACKAGE_ROOT="${KBD_PACK_ROOT:-${CLAUDE_PLUGIN_ROOT:-${PLUGIN_ROOT:-${PROMETHEUS_PLUGIN_ROOT:-$HOME/.prometheus/plugins/prometheus-skill-pack}/current}}}"
bash "$KBD_PACKAGE_ROOT/shared/scripts/kbd-contract-adopt.sh" ".kbd-orchestrator/phases/<phase>/adopted-contract.json"
```

See
prompts/plan.md. Plans without an adopted contract cannot pass the
Plan→Execute readiness gate.

## Selected-method recovery and authority

Before dependent work, run `node shared/method-context.mjs --project <project-root> --phase <canonical-phase-id> --write` from the resolved orchestrator root (or read the phase-local `method-context.md` when Node is unavailable) and act on its `status`:

- `RESOLVED_REFERENCES` — continue with the recovered method, contract and stage authority; preserve the same references in role inputs, dispatch and handoff read order.
- `BLOCKED` — a method decision, contract, calibration or supersession is missing or changed; resolve it explicitly. Prior commands, memory and tool inventories are not fresh authority.
- `CONFLICT` — governing host policy conflicts with the selected method. Stop before dependent work and show the conflict; continue only after the operator records a `KBD-POLICY-ACK` decision for that file's current hash. Never rewrite operator-owned policy. When host policy grants an exception — for example one narrow, targeted compiler check after a blocking compile error — name it as host policy, state that it conflicts with the selected method, and say it does not authorize tests, dry-runs or generated-code verification.

For selected VLBV, define acceptance before generation and finish the whole agreed value phase before any generated-code verification; tasks, hours, commits, reviewer requests and compiler problems create no boundary. Preserve original evidence, consumed budgets and unresolved effects across resume and supersession; reconcile an uncertain effect before any retry. Full rules: `references/method-context.md`.

## Value scenario discovery

Use the independently invocable `value-scenario-discovery` procedure from the installed testing skills (source: `skills/testing/value-scenario-discovery/SKILL.md`) to propose a bounded useful frontier. Reuse current method-context and existing owner-reviewed positive, rejected/deferred and rare-critical examples; do not request their approval again when it already exists. Preserve actor/outcome, source or explicit assumption, reason-now, likelihood-or-UNKNOWN, mandatory constraints, exclusions and next decision. New or unfamiliar candidates remain proposals until adopted through the existing contract.

Map adopted candidates into the existing finite scenarioMatrix with stable IDs and exact backend owner task IDs; identity-bind acceptance-bearing calibration references before generation. Definition is distinct from executable BDD authoring and separately authorized whole-value-phase execution. Governing host policy and the selected-method conflict rules above remain authoritative.
