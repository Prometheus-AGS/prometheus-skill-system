# KBD Integration: artifact-refiner

KBD invokes `artifact-refiner` as the **static quality-assurance pass** over a
whole value phase. It enforces the blocking constraints in
`.kbd-orchestrator/constraints.md` and validates the phase's artifacts. It runs
**once per phase, at B4 step 5** (`references/boundaries.md`) — after the
boundary packet has run and recorded `boundary-results.json`, and before
`kbd-phase-close-eval.sh`. It never runs per task or per change, and it never
triggers a build or test: generated code executes only inside the boundary packet.

**Entry command**: `/refine-validate` (or `/refine-code` for constraint-driven
edits made as part of an authorized B5 repair). **State backend**:
`.refiner/artifacts/<phase-id>/state.json`

---

## When KBD invokes artifact-refiner

| KBD stage | Refiner role | Entry point |
|---|---|---|
| Execute, B4 step 5 | Cumulative static constraint validation of the whole phase diff | `/refine-validate "<phase-id>"` |
| Execute, B5 repair (only when authorized) | Constraint-driven refinement of the repaired artifacts | `/refine-code "<phase-id>"` |
| Reflect | Constraint audit across the closed phase | `/refine-validate` |

Change completion (B2) invokes nothing. Evidence, certification and publication
remain independent.

---

## Artifact lifecycle in KBD context

```
all phase production complete (typed KBD transitions)
  → B4 steps 1–4: admit, freeze, boundary packet once, boundary-results.json
  → B4 step 5: /refine-validate "<phase-id>"            (static; no build)
      → checks blocking constraints from .kbd-orchestrator/constraints.md
      → writes .refiner/artifacts/<phase-id>/refinement_log.md
  → B4 step 5: independent adversarial-review, diff mode, cumulative phase diff
      → BLOCK: record certification blocked through typed KBD commands; a fix is a B5 repair
      → PASS: retain the local review receipt
  → B4 step 6: kbd-phase-close-eval.sh
  → B4 step 7 (after CLOSED): kbd-apply verify + kbd-apply archive per change
```

---

## How to invoke (KBD → refiner contract)

```yaml
# Pass this to /refine-validate at B4 step 5
artifact_name: '<phase-id>'
artifact_type: code
content_type: direct:code
constraints:
  # Import directly from .kbd-orchestrator/constraints.md
  # Copy blocking constraints as the constraint list
target_state:
  description: >
    All blocking constraints in .kbd-orchestrator/constraints.md resolved across
    the cumulative phase diff. No forbidden patterns.
workflow_triggers:
  - event: on_refinement_complete
    action:
      type: command
      target: "echo '[kbd] artifact-refiner complete for <phase-id> — proceed to independent adversarial review (B4 step 5)'"
```

There is deliberately no `on_iteration_complete` build trigger: a build inside
the refiner loop would execute generated code outside the boundary packet.

---

## What KBD reads back

- `.refiner/artifacts/<phase-id>/refinement_log.md` — pass/fail history
- Blocking constraint status — all PASS required before `kbd-phase-close-eval.sh`
- Any constraint FAIL → record certification blocked through typed KBD commands;
  keep implementation completion intact. Fixing it is a B5 repair under the
  contract's `repairAttempts`, after which only the affected scenarios rerun.
  Never hand-edit generated progress/waypoint files.

---

## KBD constraint wiring

```
.kbd-orchestrator/constraints.md  ──feeds──▶  artifact-refiner constraint list
```

Never define constraints independently in the refiner invocation.

---

## Review coverage

File count and documentation-only changes do not exempt work from review. QA
and independent adversarial review cover the cumulative phase diff. A skip may
permit development to continue, but records `pending_review`; final local
certification requires a completed receipt or explicit signed waiver. If project
constraints are absent, use the generic template in `references/constraints.md`.
