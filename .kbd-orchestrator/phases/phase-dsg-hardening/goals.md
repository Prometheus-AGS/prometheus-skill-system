# Goals — phase-dsg-hardening

## Context

`phase-dsg-install-binaries-fix` closed with two carry-forwards:

- **CF-02**: `install-binaries.sh` submodule guards use `if [ -d ... ]` which
  passes even when the submodule directory exists but is uninitialized. With
  `set -euo pipefail`, a failed `cargo build` (no `Cargo.toml`) aborts the
  entire script before the dsg section runs. Guards must check for `Cargo.toml`
  presence, not just directory existence.

- **CF-03**: v0.1.0 and v0.1.1 releases have broken/unusable Path B assets.
  The dsg skill's documentation should point users to v0.1.4+ and the
  Path B download flow should be validated against the correct version.

Additionally, the disk-space-guardian submodule in the skill-pack
(`tools/disk-space-guardian`) is still pinned to the commit that became v0.1.3
(no code change was needed for v0.1.4 — only the runner changed). The skill
documentation should reflect the current recommended release.

## Goals

- G-01: Fix `install-binaries.sh` submodule guards — replace `if [ -d
  "${dir}" ]` guards with `if [ -f "${dir}/Cargo.toml" ]` (or equivalent)
  for all tool sections (`pk`, `cowork`, `dsg`, any others). Script must run
  end-to-end without aborting when a submodule is uninitialized.

- G-02: Update `skills/devops/disk-space-guardian/SKILL.md` to reference
  v0.1.4 as the recommended install version and confirm Path B download
  instructions are accurate.

- G-03: Advance the `tools/disk-space-guardian` submodule pointer to the
  v0.1.4 tag commit (if it differs from current HEAD) and commit.

- G-04: Verify `bash scripts/install-binaries.sh` completes successfully
  end-to-end on this machine (including past the `pk` section without abort).
