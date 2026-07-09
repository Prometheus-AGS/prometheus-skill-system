# Proposal — change-bdd-005-lifecycle-loop-skill

Ship a new `skills/testing/bdd-lifecycle-loop/` skill that codifies the
author → run → triage → maintain BDD loop as a repeatable workflow.
Includes: outside-in scenario authoring guidance, `flake-budget.sh`
wrapping `--retry-tag-filter @flaky`, `test-file-diff-guard.sh` CI script,
and prose documenting the immutable-tests rule via `protect-tests.sh`.

## Prior art

Steal patterns from:
- cucumber-js `--retry-tag-filter @flaky` (primitive)
- Trunk.io flake-budget model (workflow shape)
- Standard Cucumber Steps (Rob Moffat) — canonical step library
- Our own `shared/scripts/protect-tests.sh` (already ahead of OSS state)

## Goal
G-03 — BDD lifecycle loop skill.
