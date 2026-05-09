---
id: BDD-012
title: Two-phase test gates (PR fast / release thorough)
status: planned
priority: P1
estimated_effort: 1d
agent_role: bdd-engineer
depends_on: [BDD-010, BDD-011]
unblocks: []
related: []
created_from_conversation_turn: 5-6
---

# BDD-012 — Two-phase test gates

## Problem

BDD-010 + BDD-011 deliver fast selective execution. But selective execution is *speculative*: it skips scenarios assuming the impact-set hash captures every reason a scenario might behave differently. Real environmental drift (CDN config, IPFS gateway, Playwright version, OS update) can invalidate cached pass statuses even when the hash matches.

Without periodic full re-records, "everything's green" loses meaning over time.

## Evidence

Reason about the failure mode: the IPFS gateway upgrades its TLS config; some scenarios start failing on real runs. Cached pass statuses say "still good." The next deploy is broken in production before the cache is invalidated.

## Why it matters

Selective execution is correct for short-term fast feedback. Release confidence requires periodic full validation. Two phases serve two needs.

## Proposed fix

Two distinct CI gates:

**1. Per-PR fast gate (`bdd:pr-gate`).** Uses BDD-010+011 selective execution. Skips up to ~95% of scenarios when impact sets haven't changed. Runs in <30 minutes typical.

**2. Per-release thorough gate (`bdd:release-gate`).** Forces a full re-record (`run-video-proof.ts --clean`) regardless of cached pass statuses. Runs nightly and on every release branch creation. Catches environmental drift the per-PR gate misses.

If `bdd:release-gate` hasn't passed in N days (default 7), `bdd:pr-gate` rejects merges to `main` until a fresh release-gate run completes. This bounds the staleness of the cached pass statuses.

## Trade-offs and risks

- **Cost: nightly full re-record is 2+ hours.** Acceptable as a nightly job. Bounded by 1 run/day.
- **Risk: nightly run fails for transient flake.** Mitigation: the `bdd:release-gate` workflow retries up to 2 times on failure before alerting. Quarantine handling from BDD-002 applies.
- **Risk: developers ignore release-gate failures.** Mitigation: a dashboard showing release-gate status; merges to main blocked when release-gate is red for >24 hours.
- **Risk: the 7-day staleness window is too long.** Tunable via `RELEASE_GATE_MAX_AGE_DAYS`.

## Acceptance criteria

- [ ] Two CI workflows exist: `bdd-pr-gate.yml` and `bdd-release-gate.yml`.
- [ ] PR gate uses selective execution (BDD-010 + BDD-011).
- [ ] Release gate uses `run-video-proof.ts --clean` (full re-record).
- [ ] Release gate runs nightly + on release branch creation.
- [ ] PR gate blocks merge if release gate hasn't passed in N days.
- [ ] Dashboard surfaces release-gate status.
- [ ] Documentation explains the contract.

## Implementation steps

1. Author the two workflow files.
2. Implement the staleness check (a CI step that reads the latest release-gate run from GitHub API).
3. Wire merge protection.
4. Add dashboard (a small static page or a comment-on-PR mechanism).
5. Document in `tests/README.md`.

## Dependencies

BDD-010 (selective execution must work first), BDD-011 (env hash augmentation should be in for correctness).

## Open questions

- Should weekend release-gate runs catch up if the daily run failed? Yes — weekend runs have a `--catch-up` flag that runs multiple times if needed to re-establish the pass baseline.
- Is there a "smoke test" middle tier (e.g. ~50 representative scenarios) for medium-cost daytime checks? Possibly. Don't over-engineer initially.
