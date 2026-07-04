# Assessment — phase-cowork-push-and-release

_Written: 2026-07-04 | KBD Assess stage_

---

## Current State

### cowork-skills local worktree (`/Users/gqadonis/Projects/prometheus/cowork-skills`)

| Item | State |
|---|---|
| Branch | `main` |
| Commits ahead of `origin/main` | **10** (unpushed) |
| Cargo.toml version | `0.1.5` (not bumped — must bump to `0.2.0`) |
| Cargo.lock version | `0.1.5` |
| GitHub Actions workflows | `ci.yml` + `release.yml` present (added in change-cowork-010) |
| Local build | Not verified post-commit (must confirm `cargo build --release` passes) |

**10 unpushed commits (oldest → newest):**
1. `9d65005` feat(agents): add Zed editor support with dual-path detection
2. `c2a6b72` feat(agents): add Kimi Code CLI and Kimi Desktop agent support
3. `1ea5b11` feat(agents): add MiniMax agent with dual-path detection + document mmx CLI exclusion
4. `a874d5b` feat(plugins): add `cowork plugins install <git-url>` for Claude Code plugin format
5. `c3777ad` feat(codex): idempotent Codex TOML config writer + goal template installer
6. `e6d3026` feat(opencode): idempotent OpenCode plugin registration
7. `22e4706` feat(pack): add cowork pack status/update/repair subcommand
8. `fcd9c51` feat(toolchain): add cowork toolchain subcommand for prometheus stack health
9. `113d2d7` feat(disk): add cowork disk stub subcommand delegating to dsg CLI
10. `f0f695a` ci: add GitHub Actions CI + cross-platform release workflows

### prometheus-skill-pack (this worktree, branch `claude/charming-diffie-309eef`)

| Item | State |
|---|---|
| `tools/cowork-skills` submodule pointer | `53e6b31` (upstream v0.1.5 — stale) |
| `skills/process/cowork-management/` | Present and validates clean |
| `scripts/install-binaries.sh` install_cowork() | Path A: source build from `tools/cowork-skills/cli/`; Path B: GitHub Releases download |
| `CLAUDE.md` cowork commands | Present |
| Main branch (not yet merged) | All work is on `claude/charming-diffie-309eef` |

### prometheus-skill-pack (`main` branch — what `cowork pack update` sees)

| Item | State |
|---|---|
| `skills/process/cowork-management/` | **ABSENT** — not merged to main yet |
| `tools/cowork-skills` submodule | **ABSENT** — not merged to main yet |
| `cowork pack update` behavior | Shells to `scripts/install-skills-flat.sh` on main branch — installs symlinks only, does NOT reinstall the binary |

### cowork binary currently installed

| Item | State |
|---|---|
| `cowork --version` | `0.1.5` (upstream, from `~/.cargo/bin/cowork`) |
| Location | `~/.cargo/bin/cowork` (cargo-installed, not via install-binaries.sh) |
| Has `pack` subcommand | **NO** — v0.1.5 does not have pack/toolchain/disk |

---

## Gap Analysis

### G-01: Push 10 commits to remote

**Gap**: 10 commits exist locally, zero pushed. `origin/main` is at `53e6b31`.

**Action required**:
1. Bump `Cargo.toml` version from `0.1.5` → `0.2.0` in a final commit
2. `git push origin main` from `/Users/gqadonis/Projects/prometheus/cowork-skills`

**Risk**: Push succeeds without conflicts since origin/main has no new commits since fork.

### G-02: Tag v0.2.0 and confirm CI

**Gap**: No tag on the new commits. The `release.yml` workflow triggers on `v*.*.*` tags and builds cross-platform binaries.

**Action required**:
1. After push: `git tag -a v0.2.0 -m "..."` + `git push origin v0.2.0`
2. Monitor CI at `github.com/GQAdonis/cowork-skills/actions` — `release.yml` should produce:
   - `cowork-aarch64-apple-darwin.tar.gz`
   - `cowork-x86_64-apple-darwin.tar.gz`
   - `cowork-x86_64-unknown-linux-musl.tar.gz`
3. Confirm `ci.yml` (non-release PR check) also passes

**Risk**: The `release.yml` uses `cargo-dist` or a custom matrix — confirm it has a valid `GITHUB_TOKEN` permission (`contents: write` is present in the workflow).

### G-03: Advance submodule pointer in skill-pack

**Gap**: `tools/cowork-skills` submodule pin is stale at v0.1.5 (53e6b31). After push, the new HEAD will be `f0f695a` (or whatever SHA the version-bump commit gets).

**Action required**:
1. After push/tag: `git -C tools/cowork-skills fetch && git -C tools/cowork-skills checkout v0.2.0` (or `main`)
2. `git add tools/cowork-skills && git commit -m "chore(tools): advance cowork-skills submodule to v0.2.0"`
3. This commit goes on the `claude/charming-diffie-309eef` branch (or main after merge)

### G-04: Smoke test `cowork pack status` + `cowork toolchain status`

**Gap**: Currently installed binary is v0.1.5 without these subcommands. Must rebuild from source or download v0.2.0 binary.

**Action required**:
1. Build from local worktree: `cargo build --release` in `/Users/gqadonis/Projects/prometheus/cowork-skills/cli/`
2. Install to `~/.local/bin/cowork` (or re-run `scripts/install-binaries.sh`)
3. Verify:
   - `cowork pack status` → shows prometheus-skill-pack version + installed skill counts per platform
   - `cowork toolchain status` → shows Rust toolchain + MCP service health

---

## In-Place Smooth Skill Pack Update Design

The user asked specifically how to update the skill package in place smoothly. This is the key design question for `cowork pack update`.

### Current update path (as designed in pack.rs)

```
cowork pack update
  → resolve_pack_root() → ~/Projects/prometheus/prometheus-skill-pack (dev path)
  → bash scripts/install-skills-flat.sh
  → re-creates symlinks in all platform skill directories
  → does NOT reinstall the cowork binary itself
```

### Gap: the update path does NOT rebuild/reinstall the cowork binary

When the user runs `cowork pack update`, the currently-installed binary shells to `install-skills-flat.sh`, which only manages skill symlinks. If a new version of cowork itself ships (e.g., v0.2.0 → v0.3.0), the user must also run `scripts/install-binaries.sh` to get the updated binary. This is a bootstrapping problem: the old binary can update skills, but cannot update itself.

### Recommended in-place update flow (to document in SKILL.md + `cowork pack update` output)

**For skills only (symlinks)**:
```bash
cowork pack update           # shells to install-skills-flat.sh; fastest
```

**For skills + binary update**:
```bash
# From the skill-pack directory:
bash scripts/install-binaries.sh   # rebuilds cowork + dsg from source or GitHub Releases
cowork pack update                 # then re-installs skills
```

**Or, one combined invocation** (to be documented):
```bash
bash scripts/install-skills-flat.sh && bash scripts/install-binaries.sh
```

**For git pull + full reinstall** (the canonical "update everything"):
```bash
cd ~/Projects/prometheus/prometheus-skill-pack
git pull --recurse-submodules
bash scripts/install-binaries.sh
bash scripts/install-skills-flat.sh
```

### What the plan should add

The plan should include one change that **documents this flow** — a `cowork pack` note in `skills/process/cowork-management/references/COMMANDS.md` explaining:
1. `cowork pack update` handles skill symlinks only
2. `cowork pack update --full` (future v0.3.0 feature, out of scope here) could also re-run `install-binaries.sh`
3. The manual two-step command for a complete update

---

## Open Questions

- **OQ-01**: Should `cowork pack update` also rebuild the binary? The current implementation does not. A `--full` flag could be added in a future phase (not in scope here).
- **OQ-02**: The `pack.rs` resolve_pack_root() has three resolution paths. Path 3 (`~/Projects/prometheus/prometheus-skill-pack`) is a dev convenience. Non-developer installs (via `cowork install --source .`) would use Path 2 (`~/.cowork/prometheus-skill-pack/`). Confirm documentation covers the non-dev case.
- **OQ-03**: The `release.yml` workflow needs a `GITHUB_TOKEN` or `CARGO_REGISTRY_TOKEN` — check that `GQAdonis/cowork-skills` has the required secrets set.

---

## Summary

This phase has a narrow, well-defined scope: push 10 commits, bump version to 0.2.0, tag, confirm CI, advance submodule pointer. The in-place update question reveals a documentation gap: `cowork pack update` only handles skill symlinks, not the binary itself. The plan must include documentation for the two-step full update flow (install-binaries.sh + install-skills-flat.sh) alongside the submodule pointer advance change.

**Total estimated changes: 3**
1. Version bump + push + tag (cowork-skills repo)
2. Submodule pointer advance + documentation note (skill-pack)
3. Smoke test verification record
