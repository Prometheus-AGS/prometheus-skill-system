# change-tli-c4-cross-repo-team-requests

**Title:** Team cards, discovery, and same-repo handoff vs cross-repo GitHub issue requests
**Plan PR:** C4
**Repository:** `prometheus-skill-system`
**Phase:** team-aware-learning-memory-impl
**Depends on:** `change-tli-b7-cross-agent-awareness-beta`, `change-tli-b3b-agent-team-memory-envelope`, `change-tli-b6-file-tier-reduction`
**Backend:** native-kbd
**Base branch:** a worktree branch off `origin/main` of prometheus-skill-system at or after `206ebbd`, at `/Users/gqadonis/Projects/prometheus/worktrees/tli-c4` (created with `git -C /Users/gqadonis/Projects/prometheus/prometheus-skill-pack worktree add`). Delivered through a pull request; the user merges.
**Design section:** `docs/design/team-aware-learning-memory.md` §7 (lead/intake), §1

## Why

Agent teams across repos and monorepos need discoverable cards and issue-based requests (approved plan PR 12).

## What Changes

- `schemas/team.schema.json` optional `card` `{repo, component, owns[], capabilities[], intake:{intakeRole, label:'team:<id>', rules:[{when, route:'handoff'|'issue'}]}}` (intakeRole must exist); mirror in `runtime/src/types.mts` and `validation.mts`.
- New `runtime/src/registry.mts`: publish to `~/.prometheus/knowledge/shared/teams/<repo>--<id>.json` (+ `pk ingest --scope shared` when present); discover by capability overlap and owns-glob match; request → same repo: intake task via `mutateState` in `createHandoff` format with request.sent/received events; otherwise `gh issue create --repo <card.repo> --label team:<id>` (prints the packet and exact command when gh is absent); intake imports open labelled issues idempotently.
- CLI `team-publish|team-discover|team-request|team-intake`; managed-block instructions; SKILL.md and references; rebuild scripts.
- The intake role's next SubagentStart includes the request digest line.

## Scope

- `skills/process/agent-team-creator/schemas/team.schema.json`
- `skills/process/agent-team-creator/runtime/src/types.mts`
- `skills/process/agent-team-creator/runtime/src/validation.mts`
- `skills/process/agent-team-creator/runtime/src/registry.mts`
- `skills/process/agent-team-creator/runtime/src/cli.mts`
- `skills/process/agent-team-creator/runtime/src/project-instructions.mts`
- `skills/process/agent-team-creator/runtime/test-src/teams.integration.mts`
- `skills/process/agent-team-creator/SKILL.md`
- `skills/process/agent-team-creator/scripts/*.mjs`
- `skills/process/agent-team-creator/tests/*.mjs`
- `dist/plugins/**`

## Constraints

- Implement the whole change, then run the gate once (implementation-first, integration-only).
- One cargo/rustc build on the machine at a time.
- Generated hooks/dist are regenerated, never hand-edited (C-01, C-04); bash 3.2 for shell (C-05).
- Generated outputs (`dist/plugins/**`, `hooks/*.json`, `shared/harnesses/generated/*`, `shared/scripts/generated/*`) may be touched by changes with no mutual order: they are never merged by hand. After rebasing onto the latest `main`, rerun the generators; idempotence (C-04) makes the result the same whichever change lands first.
- Hooks exit 0 and print nothing when a dependency is absent.
- Tests use scratch HOME, CODEX_HOME and PROMETHEUS_PLUGIN_ROOT; files mutated by a gate are restored with mktemp + trap.
- Outward-facing steps (tag pushes, issue creation) are confirmed with the user first.
