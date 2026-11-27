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
| Plan | `/kbd-plan` | `plan.md` + change structures | ordered changes and scoped task model/route assignments |
| Execute | `/kbd-execute` + `/kbd-apply` | `execution.md`, `progress.json` | task dispatch and driver-owned completion; final QA after production work |
| Reflect | `/kbd-reflect` | `reflection.md` | delta analysis; seeds the next phase |

Changes are tracked in `progress.json` (the ledger: `changes[]`,
`completion.implementation`), and the active position lives in
`current-waypoint.json`.

A new phase can opt in to [outcome-first scope](./outcome-first-phases.md): one
smallest useful user outcome per phase, with resume that does not let blocked
work hold back pending work that does not depend on it.

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

Planning and execution use [Task model assignments](/docs/kbd/task-model-assignments). Existing projects use [OpenSpec lifecycle updates](/docs/kbd/openspec-lifecycle) at startup and phase boundaries.

## Project memory during Execute

The `execute:before` hook recalls lessons once, from the phase goals. During the
phase an agent also makes **one bounded lookup** at three moments, asking about
the work in hand: when a task starts, on resume, and when a material blocker
stops the task.

```sh
KBD_RECALL_QUERY="<outcome> — <task or blocker>" \
  "$KBD_ORCHESTRATOR_ROOT/skills/kbd-memory-recall/kbd-memory-recall.sh" "<phase>" execute
```

The result is written to `task-recall.md` in the phase directory, and
`prior-context.md` is left as it is. A child phase is addressed as
`<parent>/<child>`.

- **Order:** surreal-memory first, then `pk`, then the local learning log. The
  lookup always exits 0. With no source available the agent proceeds and says so.
- **Recall is evidence.** A recalled lesson is checked against the project rules,
  the adopted contract and current results. It cannot authorize the boundary run,
  widen scope, change a priority or replace an approval.
- **What is recorded:** one line per lookup in `execution.md` with the source,
  the lesson applied and the next authorized action.
- **After a verified repair:** the observation, the resolution, its scope and its
  limits are written once through `learning_write.py`. Text that looks like a
  credential is refused at `user` and `global` visibility.
