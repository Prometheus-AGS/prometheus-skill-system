---
id: quality-gates
title: Quality Gates
---

# Per-Change Quality Gates

## Implementation before evidence

Finish the coherent production implementation for the active specification or
phase before authoring, changing, or running tests. Intermediate compiler
feedback is reserved for a completed edit batch and must be package-scoped.
Acceptance uses the smallest relevant local full-integration gate; unit,
mock-only, snapshot, broad workspace, and hosted-CI results are not completion
evidence.

Only one Cargo or `rustc` process may run on the development machine at a time.
Each worktree keeps a separate target directory and shares reusable objects
through `sccache`. Release, all-target, full-workspace, and Clippy builds are
reserved for a completed implementation's final certification or a requested
artifact.

## Boundary and bottleneck evaluation

KBD records idempotent before/after receipts for OpenSpec tasks, phase/child
transitions, and ZeeSpec interrogate, score, and manifest checkpoints. Each
receipt binds the canonical subject, ordinal, total, phase path, and source
revision. The detector emits the exact progress line and current position,
re-anchors outstanding obligations after compaction, and can repair only
derived waypoint/progress projections without changing the canonical revision.

Missing or out-of-order receipts block certification without guessing history.
Ambiguous authority is preserved for recovery rather than mutated. Ordinary
evaluation is deterministic, local, and network-free; adversarial review is
bounded to phase completion, ambiguous authority, or a repeated violation, and
its output is screened for sycophancy before it can certify work.

When a change reaches `implementation_status: COMPLETE`, two gates run
before archive:

1. **refine-validate** (artifact-refiner) — deterministic checklist: schema,
   file integrity, constraint satisfaction. Cheap; runs first.
2. **adversarial-review** — an isolated, cross-model LLM judge with a
   mandate to find problems, reviewing the diff against its acceptance
   criteria. The judge never shares the implementing session's context, and
   never resolves to the model that produced the work. The same gate vets
   `assessment.md`, `analysis.md`, and `plan.md` before each stage hands off.

CRITICAL findings block certification in `progress.json`; WARNING findings
are logged and carried into handoffs. Skip heuristics: fewer than 3 files, or
docs-only (deploy-sensitive changes force the gates regardless).

*Canonical sources: [`adversarial-review`](https://github.com/Prometheus-AGS/prometheus-skill-system/tree/main/skills/process/adversarial-review) and
[`integrations/adversarial-review.md`](https://github.com/Prometheus-AGS/prometheus-skill-system/tree/main/skills/process/kbd-process-orchestrator/references/integrations).*
