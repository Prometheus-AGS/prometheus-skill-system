# Plan — phase-cowork-push-and-release

_Generated: 2026-07-04 | KBD Plan stage | OpenSpec backend_

---

## Executive Summary

3 changes in a strict dependency chain — each change must be done before the
next. No parallelism possible because change-002 requires the remote commit SHA
that change-001 produces, and change-003 documents the binary that change-002
makes installable.

```
change-push-001  →  change-push-002  →  change-push-003
(bump + push)       (submodule ptr)      (docs + smoke)
```

---

## Change Roster

### change-push-001: Version bump 0.1.5 → 0.2.0 + push + tag v0.2.0

**Repo**: `/Users/gqadonis/Projects/prometheus/cowork-skills` (local worktree)
**Scope**:
- Bump `cli/Cargo.toml` version field from `0.1.5` → `0.2.0`
- Update `cli/Cargo.lock` (`cargo update --workspace`)
- Commit: `chore(release): bump version to 0.2.0`
- Push: `git push origin main`
- Tag: `git tag -a v0.2.0 -m "release: v0.2.0 — Zed/Kimi/MiniMax platform support, pack/toolchain/disk subcommands, Claude Code/Codex/OpenCode plugin management"`
- Push tag: `git push origin v0.2.0`
- Monitor CI: `ci.yml` (PR checks) and `release.yml` (binary builds)

**Key risk**: The `release.yml` workflow requires `contents: write` permission (present in the file) and access to `GITHUB_TOKEN` (automatic for `GQAdonis`-owned repos). If the workflow fails, the tag can be deleted, fixed, and re-pushed.

**Goal coverage**: G-01, G-02
**Recommended agent**: general-purpose

---

### change-push-002: Advance tools/cowork-skills submodule pointer

**Repo**: prometheus-skill-pack (this worktree, branch `claude/charming-diffie-309eef`)
**Scope**:
- After push: `git -C tools/cowork-skills fetch --tags`
- Checkout the new tag: `git -C tools/cowork-skills checkout v0.2.0`
- Stage: `git add tools/cowork-skills`
- Commit: `chore(tools): advance cowork-skills submodule to v0.2.0`
- Verify `git submodule status tools/cowork-skills` shows the new SHA (not `53e6b31`)

**Goal coverage**: G-03
**Recommended agent**: general-purpose

---

### change-push-003: Document smooth update flow + smoke test

**Repo**: prometheus-skill-pack (this worktree)
**Scope**:

**Documentation** — add `## Updating the Skill Pack` section to
`skills/process/cowork-management/references/COMMANDS.md`:

```markdown
## Updating the Skill Pack

### Skills only (fastest — symlinks only)
```bash
cowork pack update
```

### Full update (skills + binaries — after a new cowork or dsg release)
```bash
cd ~/Projects/prometheus/prometheus-skill-pack  # or wherever the pack lives
git pull --recurse-submodules
bash scripts/install-binaries.sh     # rebuilds cowork + dsg
bash scripts/install-skills-flat.sh  # re-links skills to all platforms
```

### After a cowork binary release only
```bash
bash scripts/install-binaries.sh
```

> Note: `cowork pack update` handles skill symlinks only. It does not rebuild
> the cowork binary itself. Use `install-binaries.sh` when a new cowork version
> ships.
```

**Smoke test** (to run manually and record result):
```bash
# Build from local source (before PR merge to main, use worktree path)
cargo build --release --manifest-path \
  /Users/gqadonis/Projects/prometheus/prometheus-skill-pack/.claude/worktrees/charming-diffie-309eef/tools/cowork-skills/cli/Cargo.toml
# Install to local bin
cp target/release/cowork ~/.local/bin/cowork
cowork --version           # expect: cowork 0.2.0
cowork pack status         # expect: pack version + skill counts per platform
cowork toolchain status    # expect: toolchain health table
```

**Goal coverage**: G-04, documentation OQ-01/OQ-02
**Recommended agent**: general-purpose

---

## Dependency Map

```
change-push-001 (bump + push + tag)
    ↓
change-push-002 (advance submodule pointer to new SHA)
    ↓
change-push-003 (document + smoke test with v0.2.0 binary)
```

---

## Summary Table

| Change ID | Repo | Parallel | Goal | Agent |
|---|---|---|---|---|
| change-push-001 | cowork-skills (local worktree) | No (first) | G-01, G-02 | general-purpose |
| change-push-002 | prometheus-skill-pack (worktree) | No (needs 001) | G-03 | general-purpose |
| change-push-003 | prometheus-skill-pack (worktree) | No (needs 002) | G-04 | general-purpose |

**Total changes: 3** (sequential)

---

## First Change to Apply

```
/kbd-apply change-push-001
```
