---
type: Reference
id: plan-for-ci-cross-model-qa-yaml-fix-and-forge-mcp-auth-hardening
title: Plan for CI Cross-Model QA YAML Fix and Forge MCP Auth Hardening
description: "`kbd-plan` completed for `phase-ci-cross-model-qa-and-hardening` at step `0/3` in `/Users/gqadonis/Projects/prometheus/prometheus-skill-pack`."
tags:
- ci
- cross-model-qa
- github-actions
- yaml
- rust-toolchain
- forge-mcp
- auth-hardening
links:
- ci-cross-model-qa-and-hardening-executor-completion
sources:
- stdin
- manual:phase-ci-cross-model-qa-and-hardening
timestamp: 2026-07-03T18:09:44.636980+00:00
created_at: 2026-07-03T18:09:44.636980+00:00
updated_at: 2026-07-03T18:09:44.636980+00:00
revision: 0
---

## Context

`kbd-plan` completed for `phase-ci-cross-model-qa-and-hardening` at step `0/3` in `/Users/gqadonis/Projects/prometheus/prometheus-skill-pack`.

- **Captured:** `2026-07-03T18:00:07Z`
- **Phase:** `phase-ci-cross-model-qa-and-hardening`
- **Status:** `plan_complete`
- **Changes planned:** `3`
- **Next command:** `/kbd-apply change-hard-001`
- **Plan artifacts:** `.kbd-orchestrator/phases/phase-ci-cross-model-qa-and-hardening/{assessment,plan}.md` plus handoffs
- **Waypoint:** `/kbd-apply change-hard-001`

This planning record precedes execution/completion tracking such as [CI Cross-Model QA and Hardening Executor Completion](/ci-cross-model-qa-and-hardening-executor-completion.md).

## Resolved Open Questions

### OQ-A1: Real `cross-model-qa.yml` Error

`actionlint` was installed and used to verify GitHub's actual workflow parser error instead of relying on a PyYAML guess:

```text
cross-model-qa.yml:130: could not parse as YAML: could not find expected ':'
```

Root cause:

- A `run: |` block scalar begins at 10-space indentation.
- A multiline bash assignment `COMMENT="…"` continues at **column 0**.
- Column-0 continuation terminates the YAML block scalar.
- The following markdown line beginning with `**Model:**` is then misread as a YAML mapping key.
- GitHub records a `startup_failure` on every push.

Decision: fix the workflow YAML rather than retire it.

### OQ-A2: Keep Cross-Model QA

Decision: **FIX**. The workflow remains useful as an anti-sycophancy secondary-review mechanism, and the repair is small.

### OQ-A3: `ANTHROPIC_API_KEY` Secret

`ANTHROPIC_API_KEY` appears unset.

Scope decision:

- The YAML fix should make the workflow load cleanly and stop the red startup failure.
- A real workflow dispatch still requires the repository secret.
- Secret provisioning is owner-controlled and out of code scope.
- No fake key will be added.

Open owner question: provision `ANTHROPIC_API_KEY` as a repository secret if actual `cross-model-qa` execution is desired after the YAML repair; otherwise, stopping the startup failure is sufficient for now.

### GAP-C Pre-check

No `#![feature(...)]` usage was found in `forge-rs`; pinning Rust stable is safe.

## Planned Changes

| Change | Gap | Scope | Risk |
|---|---|---|---|
| `change-hard-001` | C | Add `tools/forge-rs/rust-toolchain.toml` pinning stable to fix the nightly-vs-CI mismatch from the prior phase | Low |
| `change-hard-002` | A | Rebuild the `Post PR comment` step using a heredoc plus `--body-file` so shell/markdown text cannot escape the YAML block scalar; verify with `actionlint` | Low |
| `change-hard-003` | B | Implement custom `ValidateRequest` plus `subtle::ConstantTimeEq` for `forge-mcp` bearer authentication; add `subtle`; remove `#[allow(deprecated)]`; add unit tests for accept/reject/missing/malformed token cases | Medium-high |

## PR Strategy

- **PR-A:** `change-hard-001` and `change-hard-002`
  - Trivial configuration/workflow changes.
  - Verifies stable Rust pin and clean `cross-model-qa.yml` parsing.
- **PR-B:** `change-hard-003`
  - Isolated security-sensitive authentication change.
  - Route through `security-reviewer`.

## Verification Requirements

- Every change must be verified locally on stable Rust.
- `validate.yml` must remain green on each PR.
- `change-hard-002` must pass `actionlint` cleanly.
- `change-hard-003` must include unit coverage for bearer-auth accept, reject, missing, and malformed cases.

# Citations

1. stdin
2. manual:phase-ci-cross-model-qa-and-hardening