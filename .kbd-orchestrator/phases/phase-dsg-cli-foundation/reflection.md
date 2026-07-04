# Reflection — phase-dsg-cli-foundation

_Generated: 2026-07-04_

## Goal Achievement

| Goal | Description | Status | Evidence |
|------|-------------|--------|----------|
| G-01 | Install dsg to `~/.local/bin/dsg`, `dsg --version` = 0.1.0 | **MET** | `dsg 0.1.0` confirmed from PATH |
| G-02 | Wire dsg build into `scripts/install-binaries.sh` | **MET** | `install_dsg()` at line 216, called at line 285 — was already implemented before phase started; credited at reflect time |
| G-03 | `--json` flag on `dsg status` and `dsg scan` | **MET** | `scanner::report_status_json` and `scanner::report_json` implemented in main.rs — was already in place; credited at reflect time |
| G-04 | GitHub Actions CI workflow (fmt + clippy + test + release builds) | **MET** | `.github/workflows/release.yml` created, pushed, CI run `28714222694` triggered on `v0.1.0` tag; pre-existing `ci.yml` covers fmt/clippy/test |
| G-05 | Submodule pointer at tagged release commit | **MET** | `tools/disk-space-guardian` now at `f51443d` (v0.1.0), confirmed by `git submodule status` |

**Achievement rate: 5/5 (100%)**

## Delivered Changes

| Change | Description | Outcome |
|--------|-------------|---------|
| `change-dsg-002-push-tag` | Add `release.yml`, push 6 commits to origin/main, tag `v0.1.0` | Done — tag `8fd28d4` live |
| `change-dsg-003-release-workflow` | Verify CI triggered; document Path B artifact URLs in COMMANDS.md | Done — run `28714222694` queued, docs updated |
| `change-dsg-004-submodule-install` | Advance submodule pointer to v0.1.0; install dsg to PATH | Done — `dsg 0.1.0` at `~/.local/bin/dsg` |

## Artifact Quality Summary

| Metric | Value |
|--------|-------|
| Changes with QA | 0/3 (artifact-refiner not configured for this phase) |
| All tasks completed | 3/3 (15 tasks total, 15 done) |

No artifact-refiner logs present. Quality was verified manually: `dsg --version`,
`git submodule status`, `gh run list`.

## Technical Debt

- **`install-binaries.sh` Path A target path**: The build now places the binary
  at `tools/disk-space-guardian/target/release/dsg` (Cargo workspace root), NOT
  `tools/disk-space-guardian/dsg/target/release/dsg` (crate subdir). The current
  `install_dsg()` in `install-binaries.sh` must use the workspace-root path. This
  was caught during change-dsg-004 and the install was done correctly, but the
  script itself should be verified — if it references the crate-subdir path, it
  will silently fall through to Path B on a fresh machine. Low risk now that v0.1.0
  is published (Path B will work), but should be corrected.

- **`dsg caches` command is a stub**: Three `caches` subcommands have `[stub]`
  bodies referencing `change-dsg-005`. These emit a clear "not yet implemented"
  message and do not block integration, but the stub is permanently visible to
  `cowork disk caches` users.

- **Release CI not yet confirmed green**: The `release.yml` run was `queued` at
  reflect time. The matrix builds (especially musl and Windows) may have issues
  on first run. A follow-up check in ~10 minutes will confirm whether all 4
  artifacts published to GitHub Releases.

## Lessons Captured

1. **Cargo workspace puts `target/` at the workspace root, not the crate root.**
   When the crate is a member of a workspace (here: `dsg/` inside `disk-space-guardian/`),
   `cargo build --release` outputs to `<workspace-root>/target/release/<binary>`,
   not `<crate>/target/release/<binary>`. Any script that assumes crate-local
   `target/` will fail silently with "no such file". Always check workspace
   membership before writing copy commands.

2. **Pre-existing work inflates apparent scope.** The cowork-integration reflection
   described `dsg` as "spec-only" and implied 5+ implementation changes were needed.
   The actual assessment found 1,635 lines of working Rust with 40 tests, `--json`
   already done, and `install-binaries.sh` already wired. Two of five goals were
   already met before the phase started. Pre-phase assessments must read actual
   file contents, not rely on prior-phase prose.

3. **Release.yml must be committed before the tag push, not after.**
   GitHub CI fires on the tag push event. If `release.yml` does not exist at that
   commit, the workflow never fires for that tag. Ordering matters: author the
   workflow → commit → push main → push tag.

4. **Submodule checkout leaves detached HEAD.** `git -C tools/disk-space-guardian checkout v0.1.0`
   produces a detached HEAD. For a pinned release pointer, this is intentional and
   correct — the submodule should point to the exact tag commit, not track a branch.
   `git submodule status` shows `(v0.1.0)` which is the desired state.

## Recommended Next Phase

The natural follow-on is **`phase-dsg-install-binaries-fix`**: verify the
`install-binaries.sh` Path A target path is correct (workspace root vs crate subdir),
and confirm the release CI matrix completed with all 4 binary artifacts published.

Alternatively, if `cowork disk` functionality is the higher priority, a
**`phase-dsg-caches-implementation`** phase would complete the `caches` stub
commands and close `change-dsg-005`.

Lower priority: the broader prometheus-skill-pack delivery pipeline (next planned
phase was `phase-okf-llm-wiki-adoption` or further credential hardening work).
