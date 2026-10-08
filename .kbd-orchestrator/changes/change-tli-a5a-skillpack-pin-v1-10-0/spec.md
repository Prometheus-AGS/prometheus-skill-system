# change-tli-a5a-skillpack-pin-v1-10-0

**Title:** Tag pk and surreal-memory v1.10.0 and bump the skill-pack pins
**Plan PR:** A5a
**Repository:** `prometheus-skill-system`
**Phase:** team-aware-learning-memory-impl
**Depends on:** `change-tli-a1-pk-context-scoring`, `change-tli-a2-pk-worker-attribution`, `change-tli-a3-pk-tags-and-type`, `change-tli-a4-surreal-lean-search-categories-rekey`
**Backend:** native-kbd
**Base branch:** a worktree branch off `origin/main` of prometheus-skill-system at or after `206ebbd`, at `/Users/gqadonis/Projects/prometheus/worktrees/tli-a5a` (created with `git -C /Users/gqadonis/Projects/prometheus/prometheus-skill-pack worktree add`). Delivered through a pull request; the user merges.
**Design section:** `docs/design/team-aware-learning-memory.md` §8

## Why

B3/B4 call the A1–A4 behaviour through the installed binaries; the skill-pack must ship them (analysis D-3). Mini's half is operator-owned (A5b).

## What Changes

- After A1–A4 merge: bump the pk workspace version and surreal-memory version to 1.10.0 on their `main`, tag `v1.10.0`, push the tags (outward-facing: confirm with the user first).
- Skill-pack: move gitlinks `tools/prometheus-knowledge` and `tools/surreal-memory-server` to the tagged commits; update `skill-system.json` `imports[].commit` (enforced by `scripts/lib/skill-system.js`).
- Pin `pk-core`/`pk-librarian`/`pk-store` to `tag = "v1.10.0"` in `tools/prometheus-cli/Cargo.toml` and `tools/forge-rs/Cargo.toml`; `cargo update -p pk-core -p pk-librarian -p pk-store` in each, sequentially.
- `config/release-version-matrix.json` exemptions 1.9.0 → 1.10.0; installation docs expect 1.10.0.

## Scope

- `tools/prometheus-knowledge`
- `tools/surreal-memory-server`
- `skill-system.json`
- `tools/prometheus-cli/Cargo.toml`
- `tools/prometheus-cli/Cargo.lock`
- `tools/forge-rs/Cargo.toml`
- `tools/forge-rs/Cargo.lock`
- `config/release-version-matrix.json`
- `docs/guide/19-installation.md`
- `site/docs/operations/installation-and-upgrades.md`

## Constraints

- Implement the whole change, then run the gate once (implementation-first, integration-only).
- One cargo/rustc build on the machine at a time.
- Generated hooks/dist are regenerated, never hand-edited (C-01, C-04); bash 3.2 for shell (C-05).
- Generated outputs (`dist/plugins/**`, `hooks/*.json`, `shared/harnesses/generated/*`, `shared/scripts/generated/*`) may be touched by changes with no mutual order: they are never merged by hand. After rebasing onto the latest `main`, rerun the generators; idempotence (C-04) makes the result the same whichever change lands first.
- Hooks exit 0 and print nothing when a dependency is absent.
- Tests use scratch HOME, CODEX_HOME and PROMETHEUS_PLUGIN_ROOT; files mutated by a gate are restored with mktemp + trap.
- Outward-facing steps (tag pushes, issue creation) are confirmed with the user first.
