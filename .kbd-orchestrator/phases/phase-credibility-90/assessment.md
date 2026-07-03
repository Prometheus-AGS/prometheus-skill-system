# Assessment — phase-credibility-90

**Date:** 2026-06-30
**Previous phase readiness:** 78%
**OQ-01 status:** RESOLVED (user confirms Tavily key rotated externally)

## Current state

All phase-credibility-closure code changes are complete and sitting in the working tree as unstaged/staged modifications. Nothing has been committed. The two gaps that prevented reaching 90% are:

### GAP-90-A: BDD not in CI

- **Location:** `.github/workflows/validate.yml`
- **Finding:** No `bdd-test` CI job exists. `npm run cucumber` is defined in `package.json` but not called in CI.
- **Requirement:** A CI job that builds `tools/forge-rs/target/debug/forge` via `cargo build` then runs `npm run cucumber`. All 7 BDD scenarios (forge-validate.feature × 4, forge-enrich.feature × 3) must pass.
- **Effort:** S — one job block added to validate.yml, approximately 20 lines of YAML.

### GAP-90-B: All changes uncommitted

- **Finding:** `git status` shows ~16 modified files and ~20 untracked files from both phase-credibility-closure and phase-credibility-90 work. None are committed.
- **Requirement:** Stage all relevant files (excluding KBD orchestrator runtime files), commit with a conventional commit message, push to main (or a PR branch if the repo has branch protection).
- **Note:** The Tavily key removal in `scripts/configure-mcp-all-tools.sh` is already committed code-side; OQ-01 (external rotation) is now resolved, so this commit is safe to push.

## What does NOT need assessment

- forge-mcp auth, loopback binding, path confinement — verified in prior phase, source confirmed
- Unit tests — 20 tests pass, `forge-rs-test` CI job already in validate.yml
- Submodule URLs — all HTTPS, confirmed
- .gitignore, package-lock, CONTRIBUTING.md, docs — present

## Changes needed: 2

| # | ID | Title | Effort |
|---|---|---|---|
| 1 | `change-90-001-bdd-ci` | Add BDD test CI job to validate.yml | S |
| 2 | `change-90-002-commit-push` | Stage, commit, and push all phase-credibility-closure + BDD CI changes | S |
