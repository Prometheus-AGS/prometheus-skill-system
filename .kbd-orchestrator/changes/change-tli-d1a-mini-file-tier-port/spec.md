# change-tli-d1a-mini-file-tier-port

**Title:** Mini: envelope on Karpathy records, identity resolver port, file-tier SubagentStart hook (Claude)
**Plan PR:** D1a
**Repository:** `prometheus-skills-mini`
**Phase:** team-aware-learning-memory-impl
**Depends on:** `change-tli-b7-cross-agent-awareness-beta`, `change-tli-d2-mini-hook-timeouts-seconds`
**Backend:** native-kbd
**Base branch:** a worktree branch off `origin/main` of prometheus-skills-mini at or after `56cbf12`, at `/Users/gqadonis/Projects/prometheus/worktrees/tli-d1a` (created with `git -C /Users/gqadonis/Projects/prometheus/prometheus-skills-mini worktree add`). Delivered through a pull request; the user merges.
**Design section:** `docs/design/team-aware-learning-memory.md` §9 (corrected by analysis D-3/D-7)

## Why

Mini agents get the same untargeted flood. Mini already delivers Karpathy records to pk when present (`lib/karpathy/transport.mjs::deliverToPk`), so the envelope belongs on those records; mini has no Codex hook path (package-builder copies hooks to the Claude package only).

## What Changes

- Carry `shared/schemas/learning-envelope.schema.json` byte-identical as `lib/learning/learning-envelope.schema.json`.
- `lib/learning/identity.mjs`: Node port of B1's resolver (project id precedence, role resolution).
- `lib/karpathy/record.mjs` attaches the envelope to records before `deliverToPk`.
- `hooks/subagentstart-learning.mjs` file-tier hook for Claude Code (team digest file + per-role sections), silent and exit 0 when absent; registered in `hooks/hooks.json` with a seconds timeout.
- No Codex assertions (out of scope this phase).

## Scope

- `lib/learning/learning-envelope.schema.json`
- `lib/learning/identity.mjs`
- `lib/learning/identity.test.mjs`
- `lib/karpathy/record.mjs`
- `hooks/subagentstart-learning.mjs`
- `hooks/subagentstart-learning.test.mjs`
- `hooks/hooks.json`
- `dist/plugins/**`

## Constraints

- Implement the whole change, then run the gate once (implementation-first, integration-only).
- One cargo/rustc build on the machine at a time.
- Generated hooks/dist are regenerated, never hand-edited (C-01, C-04); bash 3.2 for shell (C-05).
- Generated outputs (`dist/plugins/**`, `hooks/*.json`, `shared/harnesses/generated/*`, `shared/scripts/generated/*`) may be touched by changes with no mutual order: they are never merged by hand. After rebasing onto the latest `main`, rerun the generators; idempotence (C-04) makes the result the same whichever change lands first.
- Hooks exit 0 and print nothing when a dependency is absent.
- Tests use scratch HOME, CODEX_HOME and PROMETHEUS_PLUGIN_ROOT; files mutated by a gate are restored with mktemp + trap.
- Outward-facing steps (tag pushes, issue creation) are confirmed with the user first.
