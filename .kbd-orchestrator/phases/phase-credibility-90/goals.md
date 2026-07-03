# Goals — phase-credibility-90

Advance the prometheus-skill-pack from 78% to ≥90% sycophancy-corrected production readiness, as measured by the weighted issue closure model from the 2026-06-29 credibility assessment.

## Context

The previous phase (phase-credibility-closure) closed 16 of 16 changes and reached 78% readiness. Two carry-forward gaps prevent reaching 90%:

1. **OQ-01 resolved (user confirms):** The Tavily API key `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` has been rotated externally at tavily.com. This unblocks C01 — the commit removing the hardcoded key can now be pushed.

2. **BDD not wired to CI:** The BDD feature files and TypeScript step definitions exist and pass locally. They are not executed in any CI job. A test that does not run in CI does not provide automated regression protection.

## Goals

- [ ] **G1:** Merge the C01 commit (key removal) — confirm the rotated key is in git history as a replaced value, push the branch, and verify gitleaks CI passes clean.
- [ ] **G2:** Add a `bdd-test` CI job to `validate.yml` that builds the `forge` binary and then runs `npm run cucumber`. All 7 BDD scenarios must pass in CI.
- [ ] **G3:** Run the sycophancy gate on a new production readiness claim reflecting G1 + G2 completion. Target: ≥90% weighted closure, score ≤0.10 at strict strictness.

## Definition of Done

All three goals MET and the sycophancy gate score ≤0.10.
