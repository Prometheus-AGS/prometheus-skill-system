# change-tli-e1-pin-v1-11-0

**Title:** Tag pk v1.11.0 (promotion + skill discovery) and bump the skill-pack pin; operator request for mini
**Plan PR:** E1
**Repository:** `prometheus-skill-system`
**Phase:** team-aware-learning-memory-impl
**Depends on:** `change-tli-a5a-skillpack-pin-v1-10-0`, `change-tli-c1a-pk-promotion-candidates`, `change-tli-c3a-pk-skill-discovery`
**Backend:** native-kbd
**Base branch:** a worktree branch off `origin/main` of prometheus-skill-system at or after `206ebbd`, at `/Users/gqadonis/Projects/prometheus/worktrees/tli-e1` (created with `git -C /Users/gqadonis/Projects/prometheus/prometheus-skill-pack worktree add`). Delivered through a pull request; the user merges.
**Design section:** `docs/design/team-aware-learning-memory.md` §3, §6

## Why

C1b and C3b call `pk candidates` and the new worker output through the installed pk binary.

## What Changes

- Bump pk to 1.11.0, tag `v1.11.0`, push (confirm with user).
- Skill-pack gitlink, `skill-system.json` import, pk-* Cargo tag pins and version matrix to v1.11.0.
- File the mini operator pin request as in A5b.

## Scope

- `tools/prometheus-knowledge`
- `skill-system.json`
- `tools/prometheus-cli/Cargo.toml`
- `tools/prometheus-cli/Cargo.lock`
- `tools/forge-rs/Cargo.toml`
- `tools/forge-rs/Cargo.lock`
- `config/release-version-matrix.json`

## Constraints

- Implement the whole change, then run the gate once (implementation-first, integration-only).
- One cargo/rustc build on the machine at a time.
- Generated hooks/dist are regenerated, never hand-edited (C-01, C-04); bash 3.2 for shell (C-05).
- Generated outputs (`dist/plugins/**`, `hooks/*.json`, `shared/harnesses/generated/*`, `shared/scripts/generated/*`) may be touched by changes with no mutual order: they are never merged by hand. After rebasing onto the latest `main`, rerun the generators; idempotence (C-04) makes the result the same whichever change lands first.
- Hooks exit 0 and print nothing when a dependency is absent.
- Tests use scratch HOME, CODEX_HOME and PROMETHEUS_PLUGIN_ROOT; files mutated by a gate are restored with mktemp + trap.
- Outward-facing steps (tag pushes, issue creation) are confirmed with the user first.
