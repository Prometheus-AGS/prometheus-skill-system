---
license: MIT
name: zeespec-interrogator
version: '1.0.0'
description: >
  Zachman Framework-based specification interrogation skill. Applies the ZeeSpec
  5W1H method (What, Where, Who, When, Why, How) across 6 dimensions x 10 questions
  to surface undefined system constraints before planning or implementation begins.
  Invoked standalone for ideation-layer GO/NO-GO decisions, or triggered automatically
  by kbd-process-orchestrator and iterative-evolver when spec coverage falls below
  threshold. Produces a structured constraint manifest consumed by the calling process.
authors:
  - 'Prometheus AGS'
allowed-tools: file_system sequential_thinking memory
model_routing:
  policy_source: ".kbd-orchestrator/project.json → model_policy"
  phases:
    zeespec-interrogate: frontier
    zeespec-score: small
    zeespec-manifest: frontier
    zeespec-persist: small
    zeespec-status: small
  routing_reference: "references/model-routing.md"
triggers:
  keywords:
    - zeespec
    - zachman
    - 5w1h
    - interrogate
    - constraint manifest
    - spec coverage
    - go no-go
    - ideation gate
    - undefined requirements
    - under-specified
    - coverage threshold
  semantic: >
    Run a ZeeSpec interrogation on a system, change, or idea to surface
    undefined constraints across the What, Where, Who, When, Why, and How
    dimensions before committing to planning or implementation.
---

# ZeeSpec Interrogator

A PMPO-driven implementation of the Zachman Framework 5W1H specification method.
Surfaces undefined constraints across six interrogation dimensions before any
planning or implementation work begins. Operates at two entry points:

1. **Standalone** — Called directly for ideation-layer GO/NO-GO decisions before
   `iterative-evolver` or `kbd-process-orchestrator` engages.
2. **Triggered** — Called automatically by `kbd-process-orchestrator` (Assess or Plan
   phase) or `iterative-evolver` (Assess phase) when spec coverage falls below the
   configured threshold for a subject area or change.

## The ZeeSpec Method

ZeeSpec applies the Zachman Framework's five interrogatives as a constraint
discovery system. For every subject under interrogation, it asks 10 questions
per dimension — 60 questions total. Each unanswered question represents a gap
where the system will make an implicit assumption, the AI agent will decide for
you, or a runtime failure will surface the omission later.

### Dimensions

| Dimension | Interrogative | Core Concern |
|---|---|---|
| **What** | Data / Entities | What does the system manage, store, or transform? |
| **Where** | Location / Network | Where does execution, storage, and communication occur? |
| **Who** | People / Roles | Who owns, operates, accesses, or is affected by the system? |
| **When** | Time / Events | When do things happen — triggers, schedules, deadlines, ordering? |
| **Why** | Motivation / Purpose | Why does this exist — goals, rules, constraints, value? |
| **How** | Function / Process | How does the system work — behavior, algorithms, protocols? |

Each dimension has 10 canonical questions (see `references/dimensions/`).
Skipped questions are recorded as `implicit` — the AI or system will decide.

## Named Interrogations

Every interrogation session has a subject name — the primary retrieval key across sessions:

```
/zeespec-interrogate "prometheus-forge-rs"
/zeespec-status "prometheus-forge-rs"
```

State is loaded by name at the start of every session, enabling:
- Cross-session continuity for large systems
- Partial interrogation across multiple working sessions
- Caller retrieval of the manifest by subject name

## Coverage Scoring

After each dimension is interrogated, the skill computes a **coverage score**
per dimension and an **aggregate score**. Scores drive three outcomes:

| Score | Status | Action |
|---|---|---|
| >= 85% | `sufficient` | GO — caller may proceed to planning |
| 60–84% | `partial` | CAUTION — caller proceeds with flagged gaps logged |
| < 60% | `insufficient` | NO-GO — caller must resolve gaps before planning |

**Per-dimension critical thresholds** (dimension failure overrides aggregate pass):

| Dimension | Critical below | Rationale |
|---|---|---|
| Why | 70% | Undefined motivation = undefined success criteria |
| Who | 65% | Undefined access/ownership = security and governance gaps |
| When | 60% | Undefined triggers/ordering = undefined system behavior |
| What, Where, How | 50% | Structural gaps tolerated if Why/Who/When are clear |

## Caller Integration

ZeeSpec is designed to be called by other skills and to return a machine-readable
constraint manifest. See `references/integration-contract.md` for the full
calling protocol.

### Called from `kbd-process-orchestrator`

KBD's Assess phase computes a spec coverage estimate per change or domain area.
When that estimate falls below `kbd_coverage_threshold` (default: 70%), KBD
invokes `zeespec-interrogator` with the change or area as subject, waits for the
manifest, then writes the manifest into the OpenSpec proposal spec before proceeding
to Plan. This makes the manifest an enrichment to the existing change structure —
not a separate document.

### Called from `iterative-evolver`

The evolver's Assess phase invokes ZeeSpec when a whole domain is under-constrained
before strategic planning begins. The manifest output feeds the `analysis.json` and
`plan.json` directly.

### Called standalone (ideation gate)

The user invokes ZeeSpec directly before either process starts. The manifest is
written to `.zeespec/<subject>/manifest.json`. The caller reads it and decides
whether to proceed to `iterative-evolver`, to `kbd`, or to abandon the idea.

## State Provider

State is persisted through a tiered provider system resolved at startup:

| Priority | Provider | When Used |
|---|---|---|
| 1 | `$ZEESPEC_PROVIDER_CONFIG` | Env var pointing to config file |
| 2 | `.zeespec-provider.json` | Project-local config |
| 3 | `~/.zeespec/provider.json` | Global config |
| 4 | MCP state tool | `state` server detected in `.mcp.json` |
| 5 | Agent memory | `memory` server detected in `.mcp.json` |
| 6 | Filesystem | Always available — `.zeespec/` directory (default) |

## Execution Model (PMPO Loop)

### Startup

1. **Resolve provider** — `scripts/state-resolve-provider.sh`
2. **Init/resume state** — `scripts/state-init.sh <subject_name>`
3. **Load dimensions** — Read all six `references/dimensions/*.md` files

### Loop

1. **Interrogate** (`prompts/interrogate.md`) — Ask 10 questions per dimension,
   record answers, flag gaps, classify each answer as `defined`, `partial`, or `implicit`
2. **Score** (`prompts/score.md`) — Compute per-dimension and aggregate coverage scores
3. **Manifest** (`prompts/manifest.md`) — Produce the structured constraint manifest
   with GO/CAUTION/NO-GO recommendation and caller-ready output format
4. **Persist** (`prompts/persist.md`) — Write validated state and manifest to provider

After each phase: checkpoint + dispatch workflow triggers.

## Inputs

```yaml
subject_name: string           # Required — retrieval key and human label
subject_description: string    # What are we interrogating? System, change, idea.
caller: string                 # standalone | kbd | iterative-evolver
caller_context:
  phase: optional string       # Which KBD/evolver phase is calling
  change_id: optional string   # If KBD: the OpenSpec change ID
  proposal_path: optional string  # If KBD: path to write enrichment into
dimensions: optional array     # Subset of [what,where,who,when,why,how] — default: all six
coverage_threshold: optional number  # Override default (0.0–1.0, default: 0.70)
workflow_triggers: optional array   # External workflows to fire at lifecycle events
```

## Outputs

```yaml
constraint_manifest:
  subject: string
  subject_description: string
  interrogation_id: string
  coverage:
    aggregate_score: number       # 0.0–1.0
    aggregate_status: sufficient | partial | insufficient
    per_dimension: {}             # score + status per dimension
  go_recommendation: GO | CAUTION | NO-GO
  go_rationale: string
  dimensions:
    what: {}
    where: {}
    who: {}
    when: {}
    why: {}
    how: {}
  gaps:
    critical: []                  # Unanswered questions in critical dimensions
    major: []                     # Unanswered in non-critical dimensions
    implicit: []                  # Skipped — system/AI will decide
  caller_enrichment:              # Formatted for direct insertion into caller context
    openspec_spec_addition: optional string
    planning_constraints: optional string
    blocked_until: optional array  # Which gaps must be resolved before proceeding
```

## Persistent State

```
.zeespec/
  registry.json                         # Maps subject_name → state path
  subjects/
    {subject_name}/
      state.json                        # Current interrogation state
      manifest.json                     # Latest constraint manifest
      checkpoints/                      # Mid-session snapshots
      history/                          # Prior interrogation cycles
```

## Termination Conditions

Interrogation completes when:
- All six dimensions have been interrogated (all 60 questions addressed or explicitly skipped)
- Coverage score is computed and status determined
- Manifest is written and caller enrichment is formatted
- User has confirmed GO/CAUTION/NO-GO recommendation or overridden it

## Quick Start

- `/zeespec-interrogate` — Run a full interrogation on a named subject
- `/zeespec-score` — Score and report coverage on an existing interrogation
- `/zeespec-status` — Show current interrogation progress and manifest
