# Plan — phase-dsg-cli-foundation

_Generated: 2026-07-04_

## Summary

3 sequential changes. G-02 (install-binaries.sh wiring) and G-03 (--json flag)
are already done and will be credited at reflect time. The remaining work is
purely integration: push the 5 local dsg commits, add a release CI workflow,
then advance the submodule pointer and install the binary to PATH.

No changes can be parallelised — each depends on the prior one:
- change-dsg-002 must push before the tag SHA is available to advance the pointer
- change-dsg-003 should be authored before the push so CI triggers on the tag
- change-dsg-004 requires the tag to exist so the pointer points to a tagged ref

## Changes

| # | ID | Wave | Effort | Goals | Agent |
|---|---|---|---|---|---|
| 1 | `change-dsg-002-push-tag` | 1 | XS | G-01 partial, G-04 partial | general-purpose |
| 2 | `change-dsg-003-release-workflow` | 2 | S | G-04 complete | general-purpose |
| 3 | `change-dsg-004-submodule-install` | 3 | XS | G-01 complete, G-05 | general-purpose |

## Ordering Rationale

Wave 1 (push + tag v0.1.0) must happen before the release.yml can fire on a
real tag event. Wave 2 (release.yml) will be authored locally and committed in
the dsg repo, then the tag push in wave 1 is re-done — or more practically,
wave 2 is authored first (before the tag) so the workflow file is in the repo
when the tag is pushed. Revised execution order:

1. **change-dsg-002**: Add `release.yml` + push 5 commits + tag `v0.1.0`
   (bundle: workflow must be in the repo before tagging so CI fires)
2. **change-dsg-003**: Verify CI triggered; note any cross-platform build results
3. **change-dsg-004**: Advance `tools/disk-space-guardian` submodule pointer +
   install `dsg` to `~/.local/bin/dsg`

## Goal Mapping

| Goal | Change | Expected Status |
|------|--------|----------------|
| G-01: dsg on PATH | change-dsg-004 | MET |
| G-02: install-binaries.sh wired | (already done) | MET — no change needed |
| G-03: --json flag | (already done) | MET — no change needed |
| G-04: CI workflow | change-dsg-002 + change-dsg-003 | MET |
| G-05: submodule pointer | change-dsg-004 | MET |
