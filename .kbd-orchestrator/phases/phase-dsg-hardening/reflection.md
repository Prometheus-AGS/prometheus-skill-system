# Reflection — phase-dsg-hardening

_Generated: 2026-07-06 · Reflect stage. Verified against HEAD (`96c1786`)._

## Goal achievement

| Goal | Status | Verified how |
|------|--------|--------------|
| G-01: Fix `install-binaries.sh` submodule guards | **MET** | 5 guards changed (`[ -d ]` → `[ -f Cargo.toml ]`); live run completed without abort (exit 0) |
| G-02: Update `SKILL.md` to reference v0.1.4+ | **MET** | Added `(recommended, installs v0.1.4+)` note to install section; `validate:strict` = 0 errors |
| G-03: Advance `tools/disk-space-guardian` submodule to v0.1.4 | **MET** | `git submodule status` → `abe2e1cc (v0.1.4)`; committed `96c1786` |
| G-04: Verify `install-binaries.sh` end-to-end without abort | **MET** | Script ran all sections; skipped 5 uninitialized submodules cleanly; dsg installed; `dsg --version` = `dsg 0.1.0` |

**Score: 4/4 goals MET (100%)**

## Delivered changes (2/2)

| Change | Gap | Commit | Description |
|--------|-----|--------|-------------|
| change-hard-001-submodule-guards | GAP-A | `b3fa3dd` | 5 `[ -d ]` → `[ -f Cargo.toml ]` guards in `scripts/install-binaries.sh` |
| change-hard-002-dsg-submodule-advance | GAP-B + G-02 | `96c1786` | Submodule pointer to v0.1.4; SKILL.md note |

## Delta analysis

### What was planned vs what was delivered

**Planned**: Fix 3 guards (`prometheus-knowledge`, `liter-llm`, `surreal-memory-server`). Advance submodule pointer.

**Actual delta**: Assessment found 3 guards. Execution found 2 more (`sycophancy-correction` at line 102, `artifact-refiner/template-forge-rs` at line 122). Both were uninitialized submodules with `[ -d ]` guards. Fixed all 5 in a single change rather than reopening the change.

**Root cause**: The assessment read `git submodule status` but only mapped the `tools/` prefix. `skills/imported/` submodules were not in scope for the original assessment but exhibited the identical bug. Correct action was to fix all instances found during verification rather than leaving 2 active bugs to create a separate change.

### Why the assessment missed them

The assessment searched `grep -n "\[ -[df]"` and identified 10 guards but focused on the ones confirmed to be uninitialized. The `skills/imported/sycophancy-correction` and `skills/imported/artifact-refiner` submodules both showed `-` (uninitialized) in `git submodule status` output — this was available data but the assessment stopped at the 3 highest-confidence bugs. A complete sweep would have found all 5 in the assess stage.

**Corrective action**: Future assessments of install script guards should run `git submodule status` and cross-reference ALL `[ -d ]` guards against ALL submodule paths, not just `tools/`.

## Artifact Quality Summary

| Metric | Value |
|--------|-------|
| Changes with QA (artifact-refiner) | 0/2 |
| First-pass pass rate | n/a |

No artifact-refiner runs. Both changes were verified directly:
- change-hard-001: live `bash scripts/install-binaries.sh` run (exit 0) + `dsg --version` check
- change-hard-002: `git submodule status` + `npm run validate:strict` (0 errors)

## Technical debt introduced

None. This phase removed debt (broken guards, stale submodule pointer). No new `# TODO` or known limitations introduced.

The `template-forge-rs` section now correctly skips when `artifact-refiner` is uninitialized, but emits a `fail()` user-visible message ("template-forge-rs not found — run: git submodule update --init --recursive"). This message was pre-existing; the guard fix makes it visible rather than suppressed by a crash. The message is accurate and actionable — no debt.

## Lessons captured

### Assess lesson: Sweep ALL submodule paths, not just `tools/`

When auditing install script guards, run `git submodule status | awk '{print $2}'` to get the full list of submodule paths, then cross-reference against every `[ -d ]` guard in the script. Stopping at a known-failing section risks leaving identical bugs in sections that happen to appear later.

### Implementation lesson: Fix all instances in one change

When verification reveals additional instances of the same bug class that were missed in the plan, fix them all in the same change rather than creating a follow-up. The alternative (ship a partial fix that still aborts on a different section) defeats the purpose of the change and will confuse the next developer.

## Carry-forwards

None. CF-02 from `phase-dsg-install-binaries-fix` (submodule guard fix) is now fully resolved, including 2 additional guards not originally identified.

## Recommended next phase

The `dsg` / `install-binaries.sh` pipeline is now solid:
- Path A: source build works end-to-end
- Path B: v0.1.4 release has all 4 artifacts (macOS arm64, macOS x86_64, Linux musl, Windows MSVC)
- Install script: skips uninitialized submodules cleanly rather than aborting

No pressing technical debt remains in this area. Recommend returning to product work or CI hardening:

1. **Provision `ANTHROPIC_API_KEY`** as a repo secret + smoke-test `workflow_dispatch` on `cross-model-qa` (left open by `phase-ci-cross-model-qa-and-hardening` OQ-A3).
2. **Broaden stable toolchain pin** to other Rust trees (`prometheus-cli`, `surreal-memory-server`) for full CI/local parity.
3. **Product work** — next roadmap item per the active backlog.
