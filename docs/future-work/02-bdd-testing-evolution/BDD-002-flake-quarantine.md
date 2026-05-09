---
id: BDD-002
title: Flake quarantine system (@quarantine tag + retry policy)
status: ready
priority: P0
estimated_effort: 1d
agent_role: bdd-engineer
depends_on: []
unblocks: []
related: [BDD-001, BDD-010]
created_from_conversation_turn: 5-6
---

# BDD-002 — Flake quarantine system

## Problem

`run-video-proof.ts` uses a `failFast`-style approach: on the first failed scenario, the whole run exits. That's correct for proving release-readiness, but it's **wrong for normal development flow**.

At 250+ scenarios with Playwright, even a 0.5% per-scenario flake rate produces 1-2 spurious failures per full run. Currently those failures stop the entire pipeline, killing throughput. Developers respond by tagging tests `@no-guide-video` to dodge the gate, which silently erodes coverage.

## Evidence

1. Read `run-video-proof.ts` — note the failFast behaviour on first scenario failure.
2. Look for scenarios already tagged `@no-guide-video`. Each is a former victim of this problem.
3. Talk to a developer who has run video proof in the last week. Confirm flake-induced retries are a regular occurrence.

## Why it matters

Two competing requirements:
- **Release gate must catch real failures.** Failing fast is correct here.
- **Development gate must absorb known flake.** A retry policy is correct here.

Without a quarantine mechanism, developers either babysit the runner (manual retry) or escape via `@no-guide-video` (lost coverage). Neither is sustainable.

## Proposed fix

A two-piece system:

**1. `@quarantine` tag.** A scenario tagged `@quarantine` indicates known flake. The runner gives quarantined scenarios up to 3 retries before declaring failure. Non-quarantined scenarios still fail-fast on the first failure.

**2. Quarantine state machine.** A scenario's quarantine status is tracked in `tests/reports/quarantine-state.json`:
- A scenario is added to quarantine via the `@quarantine` tag in its `.feature` file.
- After 5 consecutive successful runs (with no retry needed), a script suggests promoting it back to standard (CI runs the suggestion as an advisory, doesn't auto-promote).
- After 10 consecutive runs requiring retry, an alert escalates: "this scenario isn't merely flaky, it's broken — investigate."

**3. Reporting.** The video run report (`generate-video-run-report.ts`) gains a "Quarantined scenarios" section showing each scenario's retry count this run and rolling window. Quarantined scenarios that didn't need retry are highlighted as candidates for re-promotion.

## Trade-offs and risks

- **Risk: quarantine becomes the dumping ground.** Developers tag everything `@quarantine` and never investigate. Mitigation: the 10-consecutive-retries escalation; the 5-clean-runs re-promotion suggestion.
- **Risk: 3 retries hides genuinely-broken scenarios.** Mitigation: each retry logs to the report. A scenario that's been retried in N consecutive sessions is suspicious.
- **Cost: extra runtime when retries happen.** Bounded — at most 3x for quarantined scenarios. Acceptable.

## Acceptance criteria

- [ ] `@quarantine` tag is recognized by the runner.
- [ ] Quarantined scenarios get up to 3 retries before failure.
- [ ] Non-quarantined scenarios fail fast (current behaviour).
- [ ] `tests/reports/quarantine-state.json` tracks per-scenario retry history.
- [ ] Run report shows quarantined scenarios with retry counts.
- [ ] Re-promotion suggestion fires for scenarios with 5+ clean consecutive runs.
- [ ] Escalation alert fires for scenarios needing retry in 10+ consecutive runs.
- [ ] Documentation in `tests/README.md` explains the convention.

## Implementation steps

1. Add `@quarantine` tag handling to `run-video-proof.ts` (read tag list per scenario).
2. Add the quarantine state file read/write.
3. Implement retry logic: on scenario failure, if quarantined, retry up to 3 times.
4. Update the report generator to surface quarantine info.
5. Add the re-promotion suggestion logic.
6. Document in `tests/README.md` and `skills/bdd-testing/SKILL.md`.

## Dependencies

None.

## Open questions

- Should quarantine be enforced in CI's release-gate path? Recommend: in release gate, retries are still allowed but failure of any scenario (quarantined or not) blocks the release. The retries protect against transient infrastructure flake; persistent flake is still a release blocker.
- Should there be a `@quarantine(reason: "selector unstable")` parameterized form? Useful but Cucumber's tag syntax doesn't support parameters cleanly. Default to a simple boolean tag; document reasons in the feature file's description.
