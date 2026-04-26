# KBD Model Routing Policy

KBD uses **tiered model selection** to minimize frontier API spend while preserving quality at the boundaries where ambiguity lives.

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

Phase KeyClassRationale`kbd-assess`frontierOpen-ended gap analysis requires full reasoning capability`kbd-plan`frontierChange decomposition from ambiguous assessment output`kbd-status`smallRead-only structured report from known files`kbd-reflect`frontierQuality judgment and synthesis across phase evidence`opsx-new`smallDeterministic artifact scaffolding from plan output`opsx-apply-low`smallMechanical CRUD/plumbing, no new abstractions`opsx-apply-medium`mediumCrosses one module boundary, bounded design decisions`opsx-apply-high`frontierNew abstraction, domain/app/infra boundary crossing`opsx-verify`mediumStructured 3-dimension check against known artifact set`opsx-archive`smallFile move + spec delta sync, no reasoning required`refiner-iterate`smallConstraint-diff delta generation per violation, mechanical`refiner-evaluate`mediumConstraint violation judgment requires calibrated scoring

---

## Task Complexity Scoring (for `/opsx:apply` routing)

When `/opsx:apply` is invoked without an explicit complexity override, score the change by reading `design.md` and `tasks.md` before dispatching:

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

When writing `execution.md`, include a `MODEL CLASS` line per change so external orchestrators (prom-lanes, UAR) can route without re-scoring:

```
DISPATCH CONTRACTS

- change-007 → roo-code
  Entry: <prompt>
  Model class: medium
  Concrete model: Qwen3.5-27B-Q4   (resolved from model_policy.registry.medium.t4)
  Model rationale: crosses domain/application boundary, no ambiguous design decisions
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
