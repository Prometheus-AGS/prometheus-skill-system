# Plan — phase-dsg-hardening

_Generated: 2026-07-06 · Plan stage. Analyze skipped (no external research needed)._

## Summary

Two changes, sequential. Change 1 fixes a `set -euo pipefail` script abort bug
affecting all machines where `prometheus-knowledge`, `liter-llm`, or
`surreal-memory-server` submodules are uninitialized. Change 2 advances the
disk-space-guardian submodule pointer to v0.1.4 and makes a minor SKILL.md
update. Both are low-risk, confined to `scripts/install-binaries.sh` and
`tools/disk-space-guardian` submodule state.

## Change order

| Order | Change ID | Gap | Risk | Agent |
|-------|-----------|-----|------|-------|
| 1 | `change-hard-001-submodule-guards` | GAP-A | LOW | code-reviewer after |
| 2 | `change-hard-002-dsg-submodule-advance` | GAP-B + G-02 | LOW | manual verify |

**Ordering rationale**: Change 1 is a prerequisite for G-04 (end-to-end install
verify). Change 2 is independent but logically follows — once the guard fix lands,
the updated submodule pointer ensures Path B downloads v0.1.4 correctly.

## Change 1: `change-hard-001-submodule-guards`

**Goal**: Fix G-01 — replace 3 `[ -d ]` guards with `[ -f Cargo.toml ]` guards

**File**: `scripts/install-binaries.sh`, lines 47, 58, 77

**Before / after** (each of the three sections):
```bash
# BEFORE (line 47):
if [ -d "${REPO_ROOT}/tools/prometheus-knowledge" ]; then

# AFTER:
if [ -f "${REPO_ROOT}/tools/prometheus-knowledge/Cargo.toml" ]; then
```

Same pattern for `liter-llm` (line 58) and `surreal-memory-server` (line 77).

**Acceptance criteria**:
- [ ] Lines 47, 58, 77 all use `[ -f .../Cargo.toml ]` guards
- [ ] `bash scripts/install-binaries.sh` completes without abort on this machine
      (where those 3 submodules are uninitialized)
- [ ] `dsg --version` still returns `dsg 0.1.x` after the script runs
- [ ] No other lines in the script changed

## Change 2: `change-hard-002-dsg-submodule-advance`

**Goal**: Fix G-03 + G-02 — advance submodule pointer + optional SKILL.md note

**Files**:
- `tools/disk-space-guardian` submodule (git submodule pointer)
- `skills/devops/disk-space-guardian/SKILL.md` (optional minor update)

**Actions**:
1. `cd tools/disk-space-guardian && git fetch origin && git checkout v0.1.4`
   (or `git checkout abe2e1c`)
2. `cd ../.. && git add tools/disk-space-guardian`
3. Commit: `chore: advance disk-space-guardian submodule to v0.1.4`
4. Optional: add one line to SKILL.md install section noting `v0.1.4+` minimum

**Acceptance criteria**:
- [ ] `git submodule status tools/disk-space-guardian` shows `abe2e1c` (v0.1.4)
- [ ] Committed on this worktree branch
- [ ] `validate:strict skills/devops/disk-space-guardian` still passes

## G-04 verification (inline with change 1)

After change 1 lands, run:
```bash
bash scripts/install-binaries.sh 2>&1 | tail -20
dsg --version
```
Expected: script completes all sections it can (skips uninitialized submodules
with no abort), dsg section runs, `dsg --version` succeeds.

## No evolver bridge

This phase is not part of an evolver cycle.
