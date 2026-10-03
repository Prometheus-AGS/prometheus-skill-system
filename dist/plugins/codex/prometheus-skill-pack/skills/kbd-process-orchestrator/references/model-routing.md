# KBD Model Routing Policy

KBD retains project-level phase policies and complexity classes. Task execution
uses quality-first recommendations within explicit user, project and budget
constraints; cost and latency break close ties. Read
[task model selection](../skills/kbd-plan/references/task-model-selection.md) for
the authoritative task assignment and harness discovery contract. Optional
cheapest-first discovery helpers do not override this objective.

## Reading the Policy

Every project has a `.kbd-orchestrator/project.json` that contains a `model_policy` block (see `references/schemas/project.template.json`).

Before invoking any phase, read:

```
project.json → model_policy.active_environment  → "local" | "t4" | "l4"
project.json → model_policy.phases.<phase-key>  → "small" | "medium" | "frontier"
project.json → model_policy.registry.<class>.<env> → concrete model name
```

If `model_policy` is absent from `project.json`, treat all phases as `frontier`. Never silently downgrade a frontier phase without explicit config.

---

## Phase → Class Map

Phase KeyClassRationale`kbd-assess`frontierOpen-ended gap analysis requires full reasoning capability`kbd-plan`frontierChange decomposition from ambiguous assessment output`kbd-status`smallRead-only structured report from known files`kbd-reflect`frontierQuality judgment and synthesis across phase evidence`opsx-new`smallDeterministic artifact scaffolding from plan output`opsx-apply-low`smallMechanical CRUD/plumbing, no new abstractions`opsx-apply-medium`mediumCrosses one module boundary, bounded design decisions`opsx-apply-high`frontierNew abstraction, domain/app/infra boundary crossing`opsx-verify`mediumStructured 3-dimension check against known artifact set`opsx-archive`smallFile move + spec delta sync, no reasoning required`refiner-iterate`smallConstraint-diff delta generation per violation, mechanical`refiner-evaluate`mediumConstraint violation judgment requires calibrated scoring`adv-review-preflight`smallEnv/provider scan + config check, deterministic`adv-review-packet`smallPure bash packet assembly, no LLM call`adv-review-judge`frontierOpen-ended defect hunting; must resolve to a model different from the packet's producer_model (judge role, falling back to critic on collision; same-model only with a logged JUDGE_MODEL_COLLISION warning)

---

## Change Complexity Summary (legacy routing context)

These classes describe a change and existing project policy; they are not a
per-task model ranking. `/kbd-plan` analyzes concrete tasks and records scoped
assignments in `plan.md`. `/kbd-execute` and `/kbd-apply` honor those assignments.
For legacy plans, analyze and record explicit task selections before dispatch,
using `design.md`, `tasks.md` and the current harness/provider capabilities.

### Low → `opsx-apply-low` (small model)

- Task count ≤ 3
- No new trait, module, or public type introduced
- Direct analog exists in `openspec/specs/`
- Single file or single architectural layer touched
- No `TODO:` or `DECISION:` markers in `design.md`

### Medium → `opsx-apply-medium` (medium model)

- Task count 4–8
- Crosses one module boundary (e.g., domain + application layer)
- New adapter or port implementation
- No unresolved decision markers in `design.md`
- Prior art exists in `openspec/specs/` for the pattern

### High → `opsx-apply-high` (frontier model)

- Task count &gt; 8
- Crosses domain / application / infrastructure boundaries simultaneously
- New abstraction or interface introduced with no prior art
- `design.md` contains `TODO:` or `DECISION:` markers
- No equivalent pattern in `openspec/specs/`

---

## Dispatch Annotation

When writing `execution.md`, reference each scoped Task model assignments entry
and record its actual concrete model, supported reasoning setting, harness, route
and documented worker mechanism. Retain a model-class summary where needed by an
existing consumer, but never let it replace the concrete assignment. Recheck
availability before dispatch. This illustrative contract uses discovered values:

```
DISPATCH CONTRACTS

- change-007 → roo-code
  Entry: <prompt>
  Assignment: <full phase path> / change-007 / <backend task ID>
  Model class: <project policy summary>
  Concrete model: <discovered provider/model and supported reasoning effort>
  Route and worker: <native or liter-llm; documented tool-enabled execution mechanism>
  Model rationale: <task fit and dated evidence; policy tradeoffs>
  Progress file: .kbd-orchestrator/phases/<phase>/progress.json
```

---

## Environment Override

Set `model_policy.active_environment` in `project.json` to switch the entire project to a different hardware tier. No other files need to change.

EnvironmentUse Case`local`RTX 4070 Ti (12 GB VRAM, 32 GB RAM)`t4`GCP T4 VM on GKE`l4`GCP L4 VM on K3s (24 GB VRAM)

Note: `medium.local` is `null` — medium-class tasks on the local machine should be offloaded to T4 or run on a frontier API.

---

## Fallback Rule

If `model_policy` is absent from `project.json`, treat all phases as `frontier`. Log a warning to `.kbd-orchestrator/phases/<phase>/model-routing.log`:

```
[WARN] model_policy absent from project.json — defaulting all phases to frontier.
       Add model_policy block to enable cost optimization.
```
