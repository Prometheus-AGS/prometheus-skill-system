---
type: Reference
id: phase-ci-all-green-assessment-for-okf-wiki-adoption
title: Phase CI All Green Assessment for OKF Wiki Adoption
tags:
- okf
- llm-wiki
- ci-triage
- kbd-assess
- prettier
- rust
- bdd-tests
links:
- okf-wiki-adoption-pr-21-ci-triage-and-merge-readiness
- toolchain-binary-sync-and-okf-wiki-adoption-session-completion
sources:
- stdin
timestamp: 2026-07-03T14:39:51.505703+00:00
created_at: 2026-07-03T14:39:51.505703+00:00
updated_at: 2026-07-03T14:39:51.505703+00:00
revision: 0
---

## Context

A `kbd-assess` run completed for `phase-ci-all-green` under the OKF LLM wiki adoption effort. This follows the PR/CI readiness work tracked in [OKF Wiki Adoption PR 21 CI Triage and Merge Readiness](/okf-wiki-adoption-pr-21-ci-triage-and-merge-readiness.md) and the broader completed adoption phase in [Toolchain Binary Sync and OKF Wiki Adoption Session Completion](/toolchain-binary-sync-and-okf-wiki-adoption-session-completion.md).

- **KBD root:** `/Users/gqadonis/Projects/prometheus/prometheus-skill-pack`
- **Captured:** `2026-07-03T14:33:48Z`
- **Source:** `manual:phase-okf-llm-wiki-adoption`
- **Assessment output:** `.kbd-orchestrator/phases/phase-ci-all-green/`
  - `assessment.md`
  - `progress.json`
  - `handoffs/assess.md`

## Phase Goals

- PK wiki entries conform to OKF v0.1:
  - required `type` frontmatter
  - recommended `title`, `description`, `resource`, `tags`, `timestamp`
  - unknown-key tolerance
- Reserved root files maintained on every ingest:
  - `index.md`
  - `log.md`
- Cross-links moved from frontmatter `links` arrays to bundle-relative markdown body links.
- Citations section convention applied per OKF sections 5 and 8.
- Karpathy LLM Wiki operations exposed as first-class repository skills:
  - ingest
  - query
  - lint
- Wiki schema document added.
- `pk lint` enforces OKF v0.1 conformance with permissive consumption semantics.

## CI Assessment Findings

The four red **Validate Skills — failing** badges all correspond to the same `validate.yml` workflow. A workflow badge is red if any job fails.

- **Passing jobs:** 6 of 9
- **Previously fixed:** `gitleaks` job
- **Remaining blockers:** 3 jobs

| Gap | Job | Verified root cause | Fix | Effort |
|---|---|---|---|---|
| GAP-1 | BDD tests | `Cannot find module 'ts-node/register'`; the cucumber script requires a loader that is not installed, while `tsx` is already present as a dev dependency | Switch cucumber to the existing `tsx` loader | Low |
| GAP-2 | forge-rs (`fmt` + `clippy` + `test`) | `cargo fmt --check` reports 53 diffs in `tools/forge-rs`; the directory is vendored into the repo, not a submodule | Run `cargo fmt`, then verify `clippy` and tests | Low |
| GAP-3 | Check Formatting | Prettier flags 123 files; 106 are generated `site/.docusaurus/*` build output and should not be linted; about 17 are real source files | Add `site/` to `.prettierignore` and run `prettier --write` on the real source files | Low |

## Key CI Interpretation

All three failures occur at first gates:

- missing Node loader
- Rust formatting drift
- generated files included in formatting checks

No confirmed logic failure was identified during assessment. The main risk is that fixing first-gate failures may expose second-order failures, including:

- `clippy -D warnings`
- actual Rust test failures
- real BDD step failures

The plan must distinguish between:

1. the first gate passing, and
2. the full job passing after downstream checks execute.

A workflow should not be declared green until a complete run is observed.

## Open Planning Questions

1. `.prettierignore` scope:
   - ignore all of `site/`, or
   - only ignore `site/.docusaurus` and `site/build`
2. Whether to include the unbadged but red `cross-model-qa.yml` workflow in the same remediation effort or defer it.
3. BDD immutable-tests rule:
   - the loader fix is allowed configuration work
   - any real BDD step failures should be surfaced for review, not silently patched

## Proposed Execution Order

Because all three gaps are low-effort and independent, they can be fixed directly or converted into an ordered plan via `/kbd-plan phase-ci-all-green`.

Recommended direct sequence:

1. **GAP-3:** update Prettier ignore behavior and format real source files.
2. **GAP-2:** run `cargo fmt` in `tools/forge-rs`, then verify `clippy` and tests.
3. **GAP-1:** switch cucumber from `ts-node/register` to the existing `tsx` loader and rerun BDD tests.

Each step should be verified locally before opening or updating a PR.

# Citations

1. stdin