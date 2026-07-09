# Proposal — change-bdd-008-validate-smoke-tests

Add per-skill `scripts/smoke-test.sh` (minimal 1-scenario end-to-end
verification) to each new/refactored skill. Run
`npm run validate:strict` against all four skills. Wire the smoke tests
into the CI workflow so they run on PRs touching `skills/testing/`.

## Goal
G-07 — Cross-platform install + validation.
