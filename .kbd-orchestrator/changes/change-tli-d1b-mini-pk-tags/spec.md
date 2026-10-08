# change-tli-d1b-mini-pk-tags

**Title:** Mini: use pk --type/--tag when delivering Karpathy records
**Plan PR:** D1b
**Repository:** `prometheus-skills-mini`
**Phase:** team-aware-learning-memory-impl
**Depends on:** `change-tli-a5b-mini-pin-operator-request`, `change-tli-d1a-mini-file-tier-port`
**Backend:** native-kbd
**Base branch:** a worktree branch off `origin/main` of prometheus-skills-mini at or after `56cbf12`, at `/Users/gqadonis/Projects/prometheus/worktrees/tli-d1b` (created with `git -C /Users/gqadonis/Projects/prometheus/prometheus-skills-mini worktree add`). Delivered through a pull request; the user merges.
**Design section:** `docs/design/team-aware-learning-memory.md` §9

## Why

Role-filtered pk recall in mini needs the v1.10.0 tag/type flags, available only after the operator pin (A5b).

## What Changes

- `lib/karpathy/transport.mjs::deliverToPk` passes `--type` and `--tag team:/role:/vis:` derived from the envelope when `pk --version` ≥ 1.10.0, else unchanged.

## Scope

- `lib/karpathy/transport.mjs`
- `lib/karpathy/transport.test.mjs`
- `dist/plugins/**`

## Constraints

- Implement the whole change, then run the gate once (implementation-first, integration-only).
- One cargo/rustc build on the machine at a time.
- Generated hooks/dist are regenerated, never hand-edited (C-01, C-04); bash 3.2 for shell (C-05).
- Generated outputs (`dist/plugins/**`, `hooks/*.json`, `shared/harnesses/generated/*`, `shared/scripts/generated/*`) may be touched by changes with no mutual order: they are never merged by hand. After rebasing onto the latest `main`, rerun the generators; idempotence (C-04) makes the result the same whichever change lands first.
- Hooks exit 0 and print nothing when a dependency is absent.
- Tests use scratch HOME, CODEX_HOME and PROMETHEUS_PLUGIN_ROOT; files mutated by a gate are restored with mktemp + trap.
- Outward-facing steps (tag pushes, issue creation) are confirmed with the user first.
