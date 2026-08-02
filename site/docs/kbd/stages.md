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

## Lifecycle state is separate from stage state

A phase or task can be `pending`, `in_progress`, `blocked`, `complete`, or
`cancelled`. The run itself has a separate lifecycle:

| Lifecycle | Meaning | Mutation policy |
|---|---|---|
| `ready` | Runtime exists and can accept an initial transition | Allowed |
| `running` | Work may proceed | Allowed and journal-serialized |
| `pause_requested` | An interrupt or operator pause is being checkpointed | Denied |
| `paused` | Durable checkpoint exists | Denied |
| `blocked` | External or operator blocker is active | Denied |
| `completed` | Run reached its terminal success state | Denied |
| `cancelled` | Operator terminated the run | Denied |
| `failed` | Run ended unsuccessfully | Denied |

`prometheus kbd resume` only resumes a suspended run after validating its plan
revision. It does not reopen a completed, cancelled, or failed run.
Start a new phase/run for new work after a terminal state.

## Independent completion dimensions

KBD no longer treats “implementation complete” as proof that work is shipped.
The runtime records four independent dimensions:

1. `implementation`
2. `evidence`
3. `certification`
4. `publication`

For example, a Docusaurus change can have implementation and certification
complete while publication remains pending until GitHub Pages deploys the
saved build.

*Canonical source: the per-stage SKILL.md files under
[`kbd-process-orchestrator/skills/`](https://github.com/Prometheus-AGS/prometheus-skill-system/tree/main/skills/process/kbd-process-orchestrator/skills).*
