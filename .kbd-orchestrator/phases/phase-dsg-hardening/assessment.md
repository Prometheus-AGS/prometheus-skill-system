# Assessment — phase-dsg-hardening

_Generated: 2026-07-05 · Assess stage._

## Goals recap

| ID | Goal |
|----|------|
| G-01 | Fix `install-binaries.sh` submodule guards — `[ -d ]` → `[ -f Cargo.toml ]` for tool sections that can be uninitialized submodules |
| G-02 | Update `skills/devops/disk-space-guardian/SKILL.md` to reference v0.1.4 |
| G-03 | Advance `tools/disk-space-guardian` submodule pointer from v0.1.3 → v0.1.4 tag commit |
| G-04 | Verify `bash scripts/install-binaries.sh` completes end-to-end without abort |

## Current state (inspected 2026-07-05)

### G-01: Submodule guard audit

`scripts/install-binaries.sh` has 10 guards. Status per tool:

| Line | Guard | Tool path | Submodule? | Initialized? | Bug? |
|------|-------|-----------|-----------|-------------|------|
| 36 | `[ -d forge-rs ]` | `tools/forge-rs` | No (plain dir) | N/A | No — dir always present |
| 47 | `[ -d prometheus-knowledge ]` | `tools/prometheus-knowledge` | **Yes** (uninitialized `-`) | **No** | **YES — dir exists empty, `cargo build` will fail** |
| 58 | `[ -d liter-llm ]` | `tools/liter-llm` | **Yes** (uninitialized `-`) | **No** | **YES — same bug** |
| 77 | `[ -d surreal-memory-server ]` | `tools/surreal-memory-server` | **Yes** (uninitialized `-`) | **No** | **YES — same bug** |
| 102 | `[ -d sycophancy-correction ]` | `skills/imported/sycophancy-correction` | Yes (submodule) | **Yes** | No — initialized |
| 122 | `[ -d template-forge-rs ]` | `skills/imported/artifact-refiner/...` | Yes (submodule) | Yes | No — initialized; has own error message |
| 142 | `[ -d cli_dir ]` | `tools/cowork-skills/cli` | Yes (submodule) | **Yes** (v0.2.0) | No — initialized + falls to Path B |
| 219 | `[ -f Cargo.toml ]` | `tools/disk-space-guardian` | Yes (submodule) | Yes (v0.1.3) | **Fixed** — already uses `-f Cargo.toml` |

**Confirmed bug**: Three uninitialized submodules (`prometheus-knowledge`, `liter-llm`, `surreal-memory-server`) have `[ -d ]` guards that pass even when the submodule directory is empty. With `set -euo pipefail`, the first `cargo build` failure aborts the entire script. In this worktree, `prometheus-knowledge` is the first culprit at line 47.

**Correct pattern** (already used by dsg at line 219): `[ -f "${dir}/Cargo.toml" ]`

**Note on forge-rs and prometheus-cli**: These are plain directories (not submodules), always present. `forge-rs` has a `[ -d ]` guard that is fine. `prometheus-cli` has no guard at all (hardcoded build at line 27) — this would also fail if the directory is missing, but it is not a submodule issue.

### G-02: SKILL.md version reference

`skills/devops/disk-space-guardian/SKILL.md` (frontmatter):
- `version: '1.0.0'` — skill version, not dsg binary version. Correct.
- Install instruction at line 204: `bash /path/to/prometheus-skill-pack/scripts/install-binaries.sh` — correct.
- Manual build at line 207–208: `cd tools/disk-space-guardian && cargo build --release` — correct.
- **No explicit version pin** to a specific dsg release. This is actually fine — the skill describes the tool behavior, not a pinned version. The install script uses `releases/latest` redirect which will resolve to v0.1.4.
- The skill does not say "requires v0.x.x" anywhere — no update strictly needed here.

**Verdict**: G-02 has minimal actual work. The SKILL.md is version-agnostic by design. The only useful update would be a "minimum version" note in the install section. Low priority.

### G-03: Submodule pointer

`git submodule status tools/disk-space-guardian` → `b7d8f30` (v0.1.3 tag)

The v0.1.4 tag points to `d7a66f07` (different commit — the `macos-13 → macos-latest` CI change). The submodule pointer in this worktree is stale by one commit.

**Required action**: Update submodule to point to `abe2e1c` (HEAD of disk-space-guardian remote, which is also tagged v0.1.4 — wait, `git rev-parse v0.1.4` returned `d7a66f07` which is the tag object; the commit is `abe2e1c`).

**Clarification**: `d7a66f07` is the annotated tag object. The commit it points to is `abe2e1c`. The submodule should point to `abe2e1c`.

### G-04: End-to-end install verification

Last verified run: dsg Path A succeeded (`dsg --version` = `dsg 0.1.0`). The `pk` section (line 47) aborted the script before dsg ran — dsg was tested via inline function bypass.

Once G-01 is fixed, `install-binaries.sh` should run cleanly past the `pk` section (guard will correctly skip the uninitialized submodule).

## Gap summary

| Gap ID | Goal | Severity | File | Description |
|--------|------|----------|------|-------------|
| GAP-A | G-01 | **HIGH** | `scripts/install-binaries.sh` | Lines 47, 58, 77: `[ -d ]` guards on uninitialized submodules abort script |
| GAP-B | G-03 | LOW | `.gitmodules` / submodule ptr | `tools/disk-space-guardian` pinned to v0.1.3 (`b7d8f30`); should advance to v0.1.4 (`abe2e1c`) |
| GAP-C | G-02 | TRIVIAL | `skills/devops/disk-space-guardian/SKILL.md` | Optional: add minimum version note in install section |
| GAP-D | G-04 | DEPENDS | `scripts/install-binaries.sh` | End-to-end verify — blocked on GAP-A fix |

## Open questions

- **OQ-1**: Should `forge-rs` (plain dir, not submodule) also get a `Cargo.toml` guard? Currently it has `[ -d ]` — if the dir is removed this would pass silently. Low risk since it's not a submodule. **Recommendation**: leave as-is to avoid scope creep.
- **OQ-2**: Should the prometheus-cli hardcoded build (no guard) get a safety check? Same answer — not a submodule, low risk, out of scope.
- **OQ-3**: Does advancing the submodule pointer require a PR to main, or can it be committed directly on this worktree branch? The worktree is on `claude/charming-diffie-309eef` — changes here flow to a PR.

## Assessment verdict

Phase is well-scoped. **Two concrete changes**:
1. Fix three `[ -d ]` guards in `install-binaries.sh` → `[ -f Cargo.toml ]`
2. Advance disk-space-guardian submodule pointer to v0.1.4 commit

G-02 (SKILL.md) is trivially addressed alongside change-2 or skipped. G-04 is a verification step, not a code change.

**Recommended**: 2 OpenSpec changes, analyze not needed (no external research required).
