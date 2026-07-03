---
type: Reference
id: ci-cross-model-qa-and-hardening-phase-closeout
title: CI Cross-Model QA and Hardening Phase Closeout
tags:
- ci
- cross-model-qa
- hardening
- constant-time-auth
- rust-toolchain
- github-actions
- phase-closeout
links:
- ci-cross-model-qa-hardening-phase-completion-record
- ci-cross-model-qa-and-hardening-executor-completion
- ci-cross-model-qa-hardening-executor-completion-status
sources:
- stdin
- manual:phase-ci-cross-model-qa-and-hardening
timestamp: 2026-07-03T20:22:31.557591+00:00
created_at: 2026-07-03T20:22:31.557591+00:00
updated_at: 2026-07-03T20:22:31.557591+00:00
revision: 0
---

## Phase Status

- **Phase:** `phase-ci-cross-model-qa-and-hardening`
- **Project:** unspecified
- **KBD root:** `/Users/gqadonis/Projects/prometheus/prometheus-skill-pack`
- **Captured:** `2026-07-03T20:14:05Z`
- **Final status:** `reflect_complete (CLOSED)`
- **Goals:** 3/3 met
- **Gate:** `0.0`
- **Main status:** green and clean
- **Open PRs:** 0

This is the substantive closeout for the cross-model QA/security/toolchain hardening work previously tracked by [CI Cross-Model QA Hardening Phase Completion Record](/ci-cross-model-qa-hardening-phase-completion-record.md), [CI Cross-Model QA and Hardening Executor Completion](/ci-cross-model-qa-and-hardening-executor-completion.md), and [CI Cross-Model QA Hardening Executor Completion Status](/ci-cross-model-qa-hardening-executor-completion-status.md).

## Merge and CI Outcome

Merged PRs for the phase:

- **#26:** toolchain pin + `cross-model-qa` workflow fixes
- **#27:** constant-time auth hardening
- **#28:** KBD records / documentation

The last completed `validate.yml` run on `main` at commit `3c624c3` succeeded after the auth merge. The later `9e9a83a` run was for docs-only PR #28 and was expected to pass. `validate.yml` on `main` reported all 9 checks green.

## Goals Verified on `main`

| Goal | Verified outcome |
|---|---|
| **A — cross-model-qa** | Workflow loads cleanly with `actionlint 0`. The previous push `startup_failure` is gone because the workflow now parses correctly, and `on: workflow_dispatch` prevents push-triggered runs. |
| **B — constant-time auth** | `subtle::ConstantTimeEq` is present on `main`; deprecated allowance removed. Live e2e on port `:8943` verified: no token returns 401, wrong token returns 401, correct token returns 200. |
| **C — toolchain pin** | `rust-toolchain.toml` is present on `main` and resolves to stable `rustc 1.96.0`. |

## Important Verification Notes

- For Goal A, success was the **absence of an unintended run**, not a green run. Verification checked that no `cross-model-qa` run existed for the post-fix SHAs.
- `cross-model-qa` now loads cleanly and is no longer red, but a real manual dispatch still requires `ANTHROPIC_API_KEY`, which is unset and remains an owner action.
- The fix should not be overstated as end-to-end cross-model QA execution until the secret is provisioned and a `workflow_dispatch` smoke run is completed.

## Security Finding and Fix

During auth hardening, an initial draft introduced an auth bypass risk:

- Empty or whitespace `FORGE_MCP_TOKEN` could have allowed `Bearer ` to authenticate.
- A security-reviewer caught the issue.
- The implementation was corrected and a regression test was added.

Final state:

- Token comparison uses `subtle::ConstantTimeEq`.
- `#[allow(deprecated)]` was removed.
- Live authorization behavior was verified against the running service:
  - no token: `401`
  - wrong token: `401`
  - correct token: `200`

## Process Lessons

- Avoid embedding `${{ }}` in shell heredocs or commit-message heredocs; it broke a heredoc twice. Prefer `git commit -F` for messages containing GitHub Actions expressions.
- `validate.yml` only triggers on PRs targeting `main`; one PR initially received no CI until it was retargeted/rebased.

## Repository State After Closeout

Both CI phases are complete:

- README-badged CI from the prior all-green phase is green.
- Unbadged `cross-model-qa`, security hardening, and toolchain hardening from this phase are complete.
- All phase PRs are merged to `main`.
- No open PRs remain from the phase.

## Carry-Forward Items

Low-urgency follow-ups:

1. Provision `ANTHROPIC_API_KEY` and run a smoke `workflow_dispatch` of `cross-model-qa`.
2. Extend the stable toolchain pin to:
   - `tools/prometheus-cli`
   - `tools/surreal-memory-server`

These are not blocking the phase closeout.

# Citations

1. [1] stdin
2. [2] manual:phase-ci-cross-model-qa-and-hardening