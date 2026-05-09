---
id: BDD-010
title: Impact-set hash test runner
status: planned
priority: P0
estimated_effort: 1-2d
agent_role: bdd-engineer
depends_on: [BDD-008, BDD-009]
unblocks: [BDD-012]
related: [BDD-011]
created_from_conversation_turn: 5-6
---

# BDD-010 — Impact-set hash test runner

This task delivers the **selective test execution** value: scenarios are skipped between turns when their code dependencies haven't changed.

## Problem

`run-video-proof.ts` re-runs scenarios whose status isn't `passed` and whose video doesn't exist. It does *not* re-run scenarios just because the code they exercise changed. Currently, a `--clean` re-record is the only way to "re-validate everything against current code." That's prohibitively slow at 250+ scenarios.

The flip side is also broken: a passing scenario from a 2-week-old commit may be reporting "passed" against code that has since been rewritten. The pass status is stale.

## Evidence

Inspect `video-proof-state.json`. Note the `lastRunAt`, `videoSizeBytes`, `attempts`, `lastFailure` fields. There is no `validated_against_commit` and no `impact_set_hash`.

## Why it matters

- **Correctness:** a scenario's "passed" status should mean "passed against the current code." Without an impact-set hash, "passed" can mean "passed once, against a now-vanished version."
- **Speed:** a 250-scenario re-record at 30s each = 2 hours. Selective execution skips ~80% of scenarios per typical PR; budget cut to ~25 minutes.

P0 because the absence is structurally limiting.

## Proposed fix

Extend `video-proof-state.json` schema and the runner logic:

**New fields per scenario:**

```json
{
  "id": "...",
  "status": "passed",
  "validated_against_commit": "abc123",
  "impact_set_hash": "sha256:...",
  "videoSizeBytes": 451823,
  "lastRunAt": "..."
}
```

**Runner flow:**

1. At start of each run, compute the current commit SHA.
2. For each scenario, compute the current impact-set hash (via `codegraph_find_tests_for_files` reversed: get the scenario's files-exercised, hash their content + path).
3. If the scenario's stored `impact_set_hash` matches current AND `status: passed` AND `videoSizeBytes > 0` AND `validated_against_commit` is within tolerance: **skip**.
4. Otherwise: run the scenario.
5. On pass, update all four fields.

**Hash composition.** The impact-set hash is `sha256(sorted(file_path:content_hash for file in scenario.exercises_files))`. Plus environment factors per BDD-011.

**Backward compatibility.** Scenarios without an `impact_set_hash` (legacy state) are treated as needing a run. After one full re-record, all entries have hashes.

## Trade-offs and risks

- **Risk: codegraph data is stale (scenario hasn't run since significant changes).** Mitigation: the `validated_against_commit` field. If the gap between current commit and validated_against_commit exceeds a threshold (configurable; default 10 commits), force a re-run regardless of hash match.
- **Risk: hash computation is slow on large scenarios.** Mitigation: only hash the files in the impact-set (typically ~10-50 files per scenario), not the whole repo. Computation is sub-second per scenario.
- **Correctness trap.** Pure source-file closure misses environmental factors (env vars, schema migrations). BDD-011 augments the hash to handle these.
- **Cost: codegraph queries on every run.** The codegraph from BDD-008/009 is in Surreal; queries are fast (~ms). Acceptable.

## Acceptance criteria

- [ ] `video-proof-state.json` schema includes `validated_against_commit` and `impact_set_hash`.
- [ ] Runner skips scenarios where hashes match and the validated commit is within tolerance.
- [ ] Runner runs scenarios where hashes mismatch, regardless of last status.
- [ ] On pass, all four fields update.
- [ ] Backward compatibility: legacy entries without hash trigger a re-run.
- [ ] Performance: a no-change-since-last-run produces ~95%+ skip rate.
- [ ] Performance: a one-component-change re-runs only the scenarios that exercise that component.
- [ ] Run report (`generate-video-run-report.ts`) shows "ran X, skipped Y with cached pass status."

## Implementation steps

1. Update the schema in `run-video-proof.ts` and the report generator.
2. Implement impact-set hash computation (use `codegraph_find_tests_for_files` reversed or a direct query).
3. Implement skip logic in the runner main loop.
4. Update the report generator to surface skip counts.
5. Test against a real PR change with measurable skip rate.

## Dependencies

BDD-008 (static graph), BDD-009 (runtime coverage), BDD-011 (env hash augmentation should be in by the time this is in active use).

## Open questions

- What's the right `validated_against_commit` tolerance? Default 10 commits; tunable. Larger means more skips but more risk of environmental drift slipping through.
- Should the per-PR fast gate use only the static graph (skip BDD-009's runtime layer) for speed? Possibly. Trade-off between correctness and speed; document the choice.
