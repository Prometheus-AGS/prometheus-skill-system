---
id: stages
title: Stages
---

# The Six Stages

| Stage | Command | Writes | Purpose |
|---|---|---|---|
| Assess | `/kbd-assess` | `assessment.md` | gap report vs phase goals |
| Analyze | `/kbd-analyze` | `analysis.md`, `library-candidates.json` | evidence-backed build-vs-adopt calls |
| Spec | `/kbd-spec` | change specs | machine-verifiable acceptance criteria |
| Plan | `/kbd-plan` | `plan.md` + change structures | ordered change list with candidates annotated |
| Execute | `/kbd-execute` + `/kbd-apply` | `execution.md`, `progress.json` | per-change implementation with QA gates |
| Reflect | `/kbd-reflect` | `reflection.md` | delta analysis; seeds the next phase |

Changes are tracked in `progress.json` (the ledger: `changes[]`,
`completion.implementation`), and the active position lives in
`current-waypoint.json`.

*Canonical source: the per-stage SKILL.md files under
[`kbd-process-orchestrator/skills/`](https://github.com/Prometheus-AGS/prometheus-skill-system/tree/main/skills/process/kbd-process-orchestrator/skills).*
