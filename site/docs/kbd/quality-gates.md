---
id: quality-gates
title: Quality Gates
---

# Per-Change Quality Gates

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
