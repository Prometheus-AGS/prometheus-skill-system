---
id: change-001-bdd006-immutable-tests-rule
title: Immutable-tests behavioral rule (BDD-006)
phase: skill-pack-upgrade-2026-05-09
gaps: [BDD-006]
priority: P0
agent: claude-code
status: done
scope:
  - ssr-frontend/CLAUDE.md
  - prometheus-skill-pack/CLAUDE.md
  - shared/scripts/protect-tests.sh
  - .kbd-orchestrator/changes/change-001-bdd006-immutable-tests-rule/execution.md
---

# change-001-bdd006-immutable-tests-rule — Immutable-tests behavioral rule (BDD-006)

## Why

Code-generation agents in this project (and in `ssr-frontend`) currently have no documented prohibition against editing BDD step definitions and feature files as a side-effect of code changes. This allows — and in practice encourages — the failure mode where an agent edits production code and silently rewrites the tests to match, destroying the tests' value as an independent specification of intent.

This is the foundational rule that makes BDD-005 (testid drift detection) and BDD-007 (candidate test drafts) meaningful. Without it, agents who hit a drift error from BDD-005 will simply rewrite the step to match new code, defeating the purpose.

Assessment confirmed: neither `ssr-frontend/CLAUDE.md` nor `prometheus-skill-pack/CLAUDE.md` contains any behavioral restriction on agent edits to `tests/steps/*.steps.ts` or `tests/features/*.feature`.

## What Changes

- Add an explicit **immutable-tests rule** to `ssr-frontend/CLAUDE.md` in the BDD section, stating that code-gen agents may not edit `tests/steps/*.steps.ts`, `tests/support/*.ts`, or `tests/features/*.feature` to make existing tests pass. Agents may add new draft scenarios to `tests/features/drafts/`.
- Add the same rule (abbreviated, cross-referenced) to `prometheus-skill-pack/CLAUDE.md` in the BDD conventions section.
- Add cross-references to BDD-005 and BDD-007 so the trio is navigable.
- Optionally add a `shared/scripts/protect-tests.sh` PreToolUse guard (advisory/warn mode only) as a deterministic backstop.

## Capabilities

### New Capabilities
- `immutable-bdd-tests`: Rule and optional hook that prevents code-gen agents from silently rewriting step definitions or feature files when production code changes.

### Modified Capabilities
- `bdd-test-conventions`: Extends the existing BDD conventions in `ssr-frontend/CLAUDE.md` with an agent permission boundary.

## Impact

- `ssr-frontend/CLAUDE.md` — add rule text to BDD section (~30 lines)
- `prometheus-skill-pack/CLAUDE.md` — add abbreviated rule with cross-reference (~10 lines)
- `shared/scripts/protect-tests.sh` (new, optional) — PreToolUse hook guard
- No production code changes; no test changes; no database or API impact

## Execution Record

See `.kbd-orchestrator/changes/change-001-bdd006-immutable-tests-rule/execution.md` for the full dated execution record (executor, files modified, QA gate). Executed 2026-05-09; status: DONE.

## Tasks

- [x] 1. Modify `/Users/gqadonis/Projects/sansaba/ssr-frontend/CLAUDE.md` per execution record
- [x] 2. Modify `/Users/gqadonis/Projects/prometheus/prometheus-skill-pack/CLAUDE.md` per execution record
