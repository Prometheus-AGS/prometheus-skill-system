# KBD Integration: artifact-refiner

KBD invokes `artifact-refiner` as the **Quality Assurance engine** for each OpenSpec change's implementation artifacts. It enforces constraints, validates code quality, and iteratively refines until all blocking constraints pass.

**Global skill location**: `.agent/skills/artifact-refiner/SKILL.md`**Entry command**: `/refine-validate` or `/refine-code`**State backend**: `.refiner/artifacts/<artifact-name>/state.json`

---

## When KBD Invokes artifact-refiner

KBD PhaseRefiner RoleEntry Point**Execute** (per-change QA)Validate and refine a completed change's code artifacts`/refine-code`**Execute** (per-change verification)Validate constraints without refinement`/refine-validate`**Reflect** (constraint audit)Check for remaining violations across all changes`/refine-validate`

KBD invokes refiner **per implemented change**, after a typed KBD change
transition records implementation completion. Evidence, certification, and
publication remain independent. QA and independent adversarial review must
finish before driver verification and archival.

---

## Artifact Lifecycle in KBD Context

```
implementation complete through typed KBD transition
  → /refine-code "<change-id>" (artifact-refiner)
      → checks blocking constraints from .kbd-orchestrator/constraints.md
      → iterates until constraints pass or max_iterations reached
      → writes refinement_log.md to .refiner/artifacts/<change-id>/
  → independent adversarial-review diff-mode gate
      → BLOCK: record certification blocked through typed KBD commands; fix and re-review
      → PASS: retain the local review receipt
  → kbd-apply verify
  → kbd-apply archive
  → canonical lifecycle transitions regenerate progress and waypoint views
```

---

## How to Invoke (KBD → Refiner Contract)

```yaml
# Pass this to /refine-code for a completed KBD change
artifact_name: '<change-id>' # e.g. "change-007-complete-team-invitations"
artifact_type: code
content_type: direct:code
constraints:
  # Import directly from .kbd-orchestrator/constraints.md
  # Copy blocking constraints as the constraint list
target_state:
  description: >
    All blocking constraints in .kbd-orchestrator/constraints.md resolved.
    Build passes. Lint clean. No forbidden patterns.
workflow_triggers:
  - event: on_iteration_complete
    action:
      type: command
      target: '<build_health_command from .kbd-orchestrator/project.json>'
  - event: on_refinement_complete
    action:
      type: command
      target: "echo '[kbd] artifact-refiner complete for <change-id> — proceed to independent adversarial review'"
```

---

## What KBD Reads Back

After refinement, KBD checks:

- `.refiner/artifacts/<change-id>/refinement_log.md` — pass/fail history
- Blocking constraint status — all PASS required before archiving
- If any constraint FAIL remains → record certification blocked through typed
  KBD commands; keep implementation completion intact, fix, then rerun QA and
  independent review. Never hand-edit generated progress/waypoint files.

---

## KBD Constraint Wiring

The refiner uses KBD's own constraint file, eliminating duplication:

```
.kbd-orchestrator/constraints.md  ──feeds──▶  artifact-refiner constraint list
```

Never define constraints independently in the refiner invocation. Always source them from `.kbd-orchestrator/constraints.md`.

---

## Review coverage

File count and documentation-only changes do not exempt work from review.
QA and independent adversarial review cover the cumulative diff since the
last accepted local receipt. A skip may permit development to continue, but
records `pending_review`; final local certification requires a completed
receipt or explicit signed waiver. If project constraints are absent, use the
generic template in `references/constraints.md` as the starting point.
