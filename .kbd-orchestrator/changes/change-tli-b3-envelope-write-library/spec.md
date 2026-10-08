# change-tli-b3-envelope-write-library

**Title:** learning_write: one attributed write path to surreal-memory, pk and the Karpathy log
**Plan PR:** B3
**Repository:** `prometheus-skill-system`
**Phase:** team-aware-learning-memory-impl
**Depends on:** `change-tli-b1-identity-resolver`, `change-tli-b2-namespaced-subagent-matchers`, `change-tli-a5a-skillpack-pin-v1-10-0`
**Backend:** native-kbd
**Base branch:** a worktree branch off `origin/main` of prometheus-skill-system at or after `206ebbd`, at `/Users/gqadonis/Projects/prometheus/worktrees/tli-b3` (created with `git -C /Users/gqadonis/Projects/prometheus/prometheus-skill-pack worktree add`). Delivered through a pull request; the user merges.
**Design section:** `docs/design/team-aware-learning-memory.md` §2, §6

## Why

Every write path formats its own payload and drops agent identity (design §6). The envelope schema exists (`shared/schemas/learning-envelope.schema.json`) but nothing writes it.

## What Changes

- `shared/scripts/lib/learning_write.py`: build an envelope from resolved identity; validate with jsonschema when importable (structural checks otherwise); encode to surreal-memory per the design table (`user_id`, `agent_id`, `session_id`, `categories` `env:1`, `vis:`, `kind:`, `stage:`, `path:` ×≤5, `author:`, `imp:`, `h:<16>`); one copy per `audience` entry; skip when `h:` already exists in that scope; pk via `pk ingest --type <Kind> --tag team:/role:/vis:`; append the envelope to the Karpathy session record; on store failure queue to `~/.prometheus/memory-outbox/` and exit 0.
- `memory-bridge.sh` `mem_add_memory` delegates to learning_write.
- New SubagentStop hook `subagentstop-learning` (contract, matcher-less, ≤5 s, runs only for resolved roles via `team-role-guard.sh --only-team-roles`): enqueue a learning job with project/team/role and `agent_transcript_path` (A2 worker extracts lessons), and write lines of `last_assistant_message` prefixed `LESSON:`/`GOTCHA:`/`[GLOBAL]`/`[USER]` immediately via learning_write.
- Untrusted-content rule: lesson text is stored verbatim; delivery (B5) fences it.

## Scope

- `shared/scripts/lib/learning_write.py`
- `shared/scripts/lib/memory-bridge.sh`
- `shared/scripts/subagentstop-learning.sh`
- `shared/harnesses/hook-contract.json`
- `shared/scripts/tests/test-learning-write.sh`
- `hooks/hooks.json`
- `hooks/codex-hooks.json`
- `shared/harnesses/generated/*`
- `shared/scripts/generated/hook-dispatch-v1.sh`
- `dist/plugins/**`

## Constraints

- Implement the whole change, then run the gate once (implementation-first, integration-only).
- One cargo/rustc build on the machine at a time.
- Generated hooks/dist are regenerated, never hand-edited (C-01, C-04); bash 3.2 for shell (C-05).
- Generated outputs (`dist/plugins/**`, `hooks/*.json`, `shared/harnesses/generated/*`, `shared/scripts/generated/*`) may be touched by changes with no mutual order: they are never merged by hand. After rebasing onto the latest `main`, rerun the generators; idempotence (C-04) makes the result the same whichever change lands first.
- Hooks exit 0 and print nothing when a dependency is absent.
- Tests use scratch HOME, CODEX_HOME and PROMETHEUS_PLUGIN_ROOT; files mutated by a gate are restored with mktemp + trap.
- Outward-facing steps (tag pushes, issue creation) are confirmed with the user first.
