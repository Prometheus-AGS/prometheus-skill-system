# change-tli-c3a-pk-skill-discovery

**Title:** Whole-transcript parsing and skill candidates in the learning worker
**Plan PR:** C3a
**Repository:** `prometheus-knowledge-rs`
**Phase:** team-aware-learning-memory-impl
**Depends on:** `change-tli-c1a-pk-promotion-candidates`
**Backend:** native-kbd
**Base branch:** a worktree branch off `origin/main` of prometheus-knowledge-rs at or after `1bbaecc`, at `/Users/gqadonis/Projects/prometheus/worktrees/tli-c3a` (created with `git -C /Users/gqadonis/Projects/prometheus/prometheus-skill-pack/tools/prometheus-knowledge worktree add`). Delivered through a pull request; the user merges.
**Design section:** `docs/design/team-aware-learning-memory.md` §6

## Why

The worker reads only the final assistant message (`build_session_packet`), so repeated workflows and post-skill corrections are invisible.

## What Changes

- `pk-learning-worker/src/transcript.rs`: parse the whole transcript (user prompts, Write/Edit artifacts under docs/ reports/ *.md, Bash commands, Skill invocations, corrections after a skill).
- `pk-learning-worker/src/skill_candidates.rs`: workflow fingerprints into `learning-index/workflows.jsonl`; new-skill candidate when seen in ≥ 3 sessions or ≥ 2 projects with no covering skill; update candidate (in `propose-skill-update.sh` format) when a skill is followed by corrections, attributed to the role that ran it.
- `pk candidates accept --kind skill` prints the `/pmpo-skill-creator` invocation; never creates a skill itself.

## Scope

- `pk-learning-worker/src/transcript.rs`
- `pk-learning-worker/src/skill_candidates.rs`
- `pk-learning-worker/src/main.rs`
- `pk-cli/src/main.rs`
- `pk-learning-worker/tests/skill_discovery.rs`

## Constraints

- Implement the whole change, then run the gate once (implementation-first, integration-only).
- One cargo/rustc build on the machine at a time.
- Generated hooks/dist are regenerated, never hand-edited (C-01, C-04); bash 3.2 for shell (C-05).
- Generated outputs (`dist/plugins/**`, `hooks/*.json`, `shared/harnesses/generated/*`, `shared/scripts/generated/*`) may be touched by changes with no mutual order: they are never merged by hand. After rebasing onto the latest `main`, rerun the generators; idempotence (C-04) makes the result the same whichever change lands first.
- Hooks exit 0 and print nothing when a dependency is absent.
- Tests use scratch HOME, CODEX_HOME and PROMETHEUS_PLUGIN_ROOT; files mutated by a gate are restored with mktemp + trap.
- Outward-facing steps (tag pushes, issue creation) are confirmed with the user first.
