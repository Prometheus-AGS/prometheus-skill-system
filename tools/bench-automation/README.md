# Bench Automation Toolkit

Implements the close-out for the parent's G5 (RACE measurement) on the 10-task bench subset.

## Quick start

```bash
export KBD_PRODUCER_MODEL=glm-5.3
# 1. Label all packages (closes the drt-006 verified_claim_ratio=0.0 gap)
#    (bench-runbook.sh does this automatically, but it can be run alone)
python3 skills/research/deep-research/scripts/label-claims.py <pkg>

# 2. Run the full bench (scores all 10 tasks, appends rows to BENCH-RESULTS.md)
tools/bench-automation/bench-runbook.sh
```

## Files

- `../../skills/research/deep-research/scripts/label-claims.py` — adds `verified`/`partial`/`unverified` to each claim in `sources/registry.json` based on credibility score + source tier + verbatim-quote presence in chunk texts. Idempotent.
- `run-bench-suite.sh` — coordinator stub for invoking the per-task pipeline (the LLM-driven stage runner + agent I/O); the actual stage execution is currently agent-driven in this session (the runner stage is a follow-up).
- `bench-runbook.sh` — labels all packages, fixes the nested→flat layout run-bench expects, runs `run-bench.sh`, scores via `score-race.sh` + `score-fact.py`, and appends per-task rows to `tests/bench/BENCH-RESULTS.md`.

## Output layout

```
~/.prometheus/research/bench-g5/
  task-51/         →  symlink to nested slug-dir package
  task-58/         →  ...
  ...
  bench-summary.json   ← per-task RACE array
```

Each package:
```
~/.prometheus/research/bench-g5/task-<id>/
  from-<slug>-<8hex>/
    report.md, plan.md, graph.json, citations.json,
    manifest.json, index.md, checkpoint.json,
    sources/{url-list.json, registry.json, credibility.json,
             chunk-NN.json, contradictions.json, label-summary.json}
    review/{findings.json, packet.json, ...}
```

## Retry semantics

`bench-runbook.sh` is idempotent: re-running labels do not duplicate work, and `run-bench.sh` SKIPs tasks that lack `report.md` (the symlink fix is also non-destructive). The orchestrator (`run-bench-suite.sh`) is a thin coordinator; the real per-task work remains agent-driven for now (a follow-up would wire the LLM stage runner + a gateway web-search proxy for full autonomy).
