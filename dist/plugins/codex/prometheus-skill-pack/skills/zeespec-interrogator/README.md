# ZeeSpec Interrogator

ZeeSpec Zachman Framework interrogation skill for the Prometheus skill pack.
Surfaces undefined system constraints across 6 dimensions × 10 questions before
planning or implementation begins. Produces a constraint manifest consumed by
`kbd-process-orchestrator`, `iterative-evolver`, or standalone callers.

## Architecture

```
prompts/
  meta-controller.md      — orchestration entry point, provider resolution, loop driver
  interrogate.md          — 6-dimension questioning phase (60 questions, gap flagging)
  score.md                — per-dimension + aggregate coverage scoring
  manifest.md             — constraint manifest generation with GO/CAUTION/NO-GO
  persist.md              — provider-agnostic state persistence

references/
  zeespec-framework.md    — ZeeSpec 5W1H theory and Zachman background
  coverage-scoring.md     — threshold logic, per-dimension criticality, scoring rubrics
  integration-contract.md — caller protocol (kbd, iterative-evolver, standalone)
  model-routing.md        — model class definitions per phase
  schemas/
    interrogation-state.schema.json
    constraint-manifest.schema.json
    coverage-score.schema.json
  dimensions/
    what.md / where.md / who.md / when.md / why.md / how.md

scripts/
  state-init.sh           — initialize or resume named interrogation
  state-checkpoint.sh     — mid-session snapshots
  state-finalize.sh       — archive completed interrogation
  state-resolve-provider.sh — 5-tier provider resolution waterfall
  workflow-dispatch.sh    — lifecycle trigger dispatch
  score-coverage.sh       — compute coverage score from answered questions

skills/
  zeespec-interrogate/    — /zeespec-interrogate slash entry point
  zeespec-score/          — /zeespec-score slash entry point
  zeespec-status/         — /zeespec-status slash entry point
```

## Entry Points

| Command | Purpose |
|---|---|
| `/zeespec-interrogate` | Full interrogation on a named subject |
| `/zeespec-score` | Score and report coverage on existing interrogation |
| `/zeespec-status` | Show current progress and manifest |

## Caller Integration

ZeeSpec is invoked as a triggered sub-skill from:
- `kbd-process-orchestrator` — when Assess/Plan phase detects coverage < 70%
- `iterative-evolver` — when Assess phase detects domain is under-constrained
- User directly — for standalone ideation-layer GO/NO-GO decisions

The constraint manifest at `.zeespec/<subject>/manifest.json` is the primary
output artifact consumed by callers. See `references/integration-contract.md`.

## Quick Start

```bash
# Standalone ideation gate
/zeespec-interrogate "prometheus-forge-rs"

# Check status mid-session
/zeespec-status "prometheus-forge-rs"

# Called by kbd (usually automatic, but can be run manually)
/zeespec-interrogate "uar-cedar-policy-middleware" --caller kbd --change-id CHANGE-042
```
