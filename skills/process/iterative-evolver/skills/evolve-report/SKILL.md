---
name: evolve-report
description: >
  Generate an evolution report from current state data. Produces a human-readable
  markdown report summarizing the full evolution cycle. Aggregates KBD phase
  results and artifact-refiner QA metrics when available. Optionally refines
  the report through artifact-refiner content refinement.
---

# Evolve Report

Generate a comprehensive evolution report from existing state data.

## Setup

1. Load all state files: `evolution_state.json`, `assessment.json`, `analysis.json`, `plan.json`
2. Load `evolution_log.md` and `decisions.md` for history
3. If KBD was used: load `.kbd-orchestrator/phases/<phase>/reflection.md`
4. If artifact-refiner was used: load `.refiner/artifacts/*/refinement_log.md`

## KBD Integration

When the evolution delegated execution to KBD, the report includes:

### Phase Execution Summary

- Changes planned vs. completed vs. blocked
- Tool dispatch breakdown (which AI tools executed what)
- OpenSpec change traceability (if OpenSpec was active)

### Artifact Quality Metrics

Aggregated from artifact-refiner QA logs:

- First-pass pass rate across all changes
- Recurring constraint violations with counts
- Total refinement iterations required
- Quality trend across the phase (improving/degrading)

### Cross-Tool Coordination

- How many tools participated in execution
- Handoff success rate (changes that completed without re-dispatch)
- Waypoint accuracy (how often the next tool resumed correctly)

## Artifact-Refiner Content Refinement

After generating the draft report, optionally invoke artifact-refiner to
polish the report content:

```
/refine-content "evolution-report-<timestamp>"
  ├─ Checks structural completeness
  ├─ Validates data consistency (do numbers add up?)
  ├─ Improves clarity and conciseness
  └─ Writes refined version alongside the draft
```

Skip refinement with `--no-refine` or when artifact-refiner is unavailable.

## Output

Generate a markdown report to `reports/evolution-report-<timestamp>.md` including:

- Executive summary
- Goals and current alignment
- Assessment findings
- Landscape analysis highlights
- Improvement plan overview
- Execution results (if available)
  - KBD phase summary (if KBD was used)
  - Artifact quality metrics (if artifact-refiner was used)
- Iteration history
- Lessons learned
- Recommended next steps

If minimal state exists, generate a partial report with available data and note gaps.
