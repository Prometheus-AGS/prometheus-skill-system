---
type: Reference
id: gitleaks-ci-license-gate-fixed-in-pr-22
title: Gitleaks CI License Gate Fixed in PR 22
tags:
- gitleaks
- ci
- secret-scanning
- github-actions
- okf
- pull-request
links:
- okf-wiki-adoption-pr-21-ci-triage-and-merge-readiness
- toolchain-binary-sync-and-okf-wiki-adoption-session-completion
sources:
- stdin
- manual:phase-okf-llm-wiki-adoption
- https://github.com/Prometheus-AGS/prometheus-skill-system/pull/22
timestamp: 2026-07-03T14:26:08.570748+00:00
created_at: 2026-07-03T14:26:08.570748+00:00
updated_at: 2026-07-03T14:26:08.570748+00:00
revision: 0
---

## Context

During `phase-okf-llm-wiki-adoption` for KBD root `/Users/gqadonis/Projects/prometheus/prometheus-skill-pack`, the OKF wiki adoption work had already been triaged in [OKF Wiki Adoption PR 21 CI Triage and Merge Readiness](/okf-wiki-adoption-pr-21-ci-triage-and-merge-readiness.md) and operationally closed in [Toolchain Binary Sync and OKF Wiki Adoption Session Completion](/toolchain-binary-sync-and-okf-wiki-adoption-session-completion.md). A remaining CI failure was isolated to the gitleaks secret-scanning job.

## Root Cause

- The CI workflow used `gitleaks/gitleaks-action@v2`.
- For org-owned repositories, that action requires a paid `GITLEAKS_LICENSE`.
- Without the license, the action failed in ~7 seconds and performed no scan.

## Fix in PR #22

- Replaced the license-gated GitHub Action with the MIT-licensed gitleaks CLI.
- Pinned gitleaks CLI version: `v8.30.1`.
- CI command scans full git history:

```bash
gitleaks git .
```

- Workflow file changed: `.github/workflows/validate.yml`
- Diff size: `+30/-3`
- Scope: secret-scan job only.

## Validation

- GitHub Actions secret-scanning job passed on PR #22 in 11 seconds.
- Local pre-scan of working tree completed with zero findings:
  - working tree size: ~13 GB
  - findings: `0`
- Local full-history scan completed with zero findings:
  - commits scanned: `225`
  - findings: `0`
- Exact CI command was verified locally:
  - exit code: `0`
  - SARIF output written successfully
- Repository is clean; no baseline file or secret rotation is needed.

## Pull Request State

- PR: #22
- URL: `https://github.com/Prometheus-AGS/prometheus-skill-system/pull/22`
- Status: open
- Mergeability: mergeable

## Out of Scope

Two other failing checks were pre-existing and are not caused by PR #22:

- `Check Formatting`
  - Existing prettier debt in generated `site/.docusaurus/*` files.
- `forge-rs`
  - Existing Rust `fmt`/`clippy` failure.

PR #22 intentionally does not address those checks because it only changes the secret-scanning workflow.

## Next Action

Review and merge PR #22 to make gitleaks green on `main`. Address the remaining formatting and `forge-rs` failures in a separate PR if needed.

# Citations

1. stdin
2. manual:phase-okf-llm-wiki-adoption
3. https://github.com/Prometheus-AGS/prometheus-skill-system/pull/22