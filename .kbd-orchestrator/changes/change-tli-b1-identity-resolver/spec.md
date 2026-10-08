# change-tli-b1-identity-resolver

**Title:** One project-id resolver and an agent identity resolver used by every memory path
**Plan PR:** B1
**Repository:** `prometheus-skill-system`
**Phase:** team-aware-learning-memory-impl
**Depends on:** none
**Backend:** native-kbd
**Base branch:** a worktree branch off `origin/main` of prometheus-skill-system at or after `206ebbd`, at `/Users/gqadonis/Projects/prometheus/worktrees/tli-b1` (created with `git -C /Users/gqadonis/Projects/prometheus/prometheus-skill-pack worktree add`). Delivered through a pull request; the user merges.
**Design section:** `docs/design/team-aware-learning-memory.md` §1

## Why

Four project-id derivations exist and `shared/scripts/lib/memory-bridge.sh:16` defaults to the literal `prometheus-skill-pack`; nothing maps a hook payload's `agent_type` to a team role (assessment B, design §1).

## What Changes

- `shared/scripts/lib/project_id.py` (+ `project-id.sh`, bash 3.2): precedence `PROMETHEUS_PROJECT_ID` > `.prometheus/project.json` `projectId` > runtime project UUID (`prometheus kbd status --json`, 2 s timeout) > `project:<sha256(git common dir)>` so all worktrees share one id; `--json` also returns `userScope` (`PROMETHEUS_USER_ID` or `user:<sha256(global git email)[:16]>`).
- `shared/scripts/lib/agent_identity.py`: from a hook payload resolve `(projectId, teamId, roleId, harness)` — strip `<plugin>:`, map Codex `_`→`-`, match team manifest role ids, else longest `owns` glob over touched paths, else `unresolved`; team from `.agent-team/project-routing.json` `activeTeam`, else the sole team, else `@solo`; `--is-team-role <agent_type>` exits 0/1.
- Callers switch to the resolver: `shared/scripts/lib/memory-bridge.sh`, `skills/process/kbd-process-orchestrator/skills/kbd-memory-recall/kbd-memory-recall.sh`, `shared/scripts/evaluate-session.sh`, `shared/scripts/enqueue-learning-job.py` (which now also writes `projectId`/`teamId`/`roleId` into jobs for A2).
- Regenerate dist (C-01).

## Scope

- `shared/scripts/lib/project_id.py`
- `shared/scripts/lib/project-id.sh`
- `shared/scripts/lib/agent_identity.py`
- `shared/scripts/lib/memory-bridge.sh`
- `skills/process/kbd-process-orchestrator/skills/kbd-memory-recall/kbd-memory-recall.sh`
- `shared/scripts/evaluate-session.sh`
- `shared/scripts/enqueue-learning-job.py`
- `shared/scripts/tests/test-identity.sh`
- `dist/plugins/**`

## Constraints

- Implement the whole change, then run the gate once (implementation-first, integration-only).
- One cargo/rustc build on the machine at a time.
- Generated hooks/dist are regenerated, never hand-edited (C-01, C-04); bash 3.2 for shell (C-05).
- Generated outputs (`dist/plugins/**`, `hooks/*.json`, `shared/harnesses/generated/*`, `shared/scripts/generated/*`) may be touched by changes with no mutual order: they are never merged by hand. After rebasing onto the latest `main`, rerun the generators; idempotence (C-04) makes the result the same whichever change lands first.
- Hooks exit 0 and print nothing when a dependency is absent.
- Tests use scratch HOME, CODEX_HOME and PROMETHEUS_PLUGIN_ROOT; files mutated by a gate are restored with mktemp + trap.
- Outward-facing steps (tag pushes, issue creation) are confirmed with the user first.
