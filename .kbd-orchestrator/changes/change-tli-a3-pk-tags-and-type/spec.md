# change-tli-a3-pk-tags-and-type

**Title:** pk ingest accepts --type and --tag, and pk context filters by --tag
**Plan PR:** A3
**Repository:** `prometheus-knowledge-rs`
**Phase:** team-aware-learning-memory-impl
**Depends on:** `change-tli-a1-pk-context-scoring`
**Backend:** native-kbd
**Base branch:** a worktree branch off `origin/main` of prometheus-knowledge-rs at or after `1bbaecc`, at `/Users/gqadonis/Projects/prometheus/worktrees/tli-a3` (created with `git -C /Users/gqadonis/Projects/prometheus/prometheus-skill-pack/tools/prometheus-knowledge worktree add`). Delivered through a pull request; the user merges.
**Design section:** `docs/design/team-aware-learning-memory.md` §2, §8 U3

## Why

Recall must filter pk entries by `role:`/`team:`/`vis:` tags without client-side post-filtering, and lessons need OKF `type` values (Lesson, Gotcha, Decision, Progress, Candidate). Today `pk-librarian/src/librarian.rs:413` hard-codes `entry_type = Some("Reference")` and `pk-store/src/store.rs:394` defaults it; pk-cli has no `--tag`/`--type` (assessment A3).

## What Changes

- `pk ingest --type <T>` sets the entry type; `librarian.rs:413` and `store.rs:394` keep `Reference` only as the default when none is given.
- `pk ingest --tag <t>` (repeatable) adds tags to the entry.
- `pk context --tag <t>` (repeatable, all-of) keeps only entries carrying every given tag; applied before scoring.
- pk-mcp `knowledge_search`/`knowledge_ingest` are out of scope (hooks use the CLI).

## Scope

- `pk-cli/src/main.rs`
- `pk-librarian/src/librarian.rs`
- `pk-store/src/store.rs`
- `pk-cli/tests/tags_and_type.rs`

## Constraints

- Implement the whole change, then run the gate once (implementation-first, integration-only).
- One cargo/rustc build on the machine at a time.
- Generated hooks/dist are regenerated, never hand-edited (C-01, C-04); bash 3.2 for shell (C-05).
- Generated outputs (`dist/plugins/**`, `hooks/*.json`, `shared/harnesses/generated/*`, `shared/scripts/generated/*`) may be touched by changes with no mutual order: they are never merged by hand. After rebasing onto the latest `main`, rerun the generators; idempotence (C-04) makes the result the same whichever change lands first.
- Hooks exit 0 and print nothing when a dependency is absent.
- Tests use scratch HOME, CODEX_HOME and PROMETHEUS_PLUGIN_ROOT; files mutated by a gate are restored with mktemp + trap.
- Outward-facing steps (tag pushes, issue creation) are confirmed with the user first.
