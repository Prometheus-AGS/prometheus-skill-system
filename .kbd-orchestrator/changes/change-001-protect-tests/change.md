---
id: change-001-protect-tests
title: protect-tests.sh — close the false CLAUDE.md immutable-tests claim
phase: safeguards
gaps: [H1]
priority: P1
effort: S
agent: claude-code
evolver_item_id: null
status: done
scope:
  - shared/scripts/protect-tests.sh
  - hooks/hooks.json
  - shared/scripts/tests/test-protect-tests.sh
---

# change-001 — protect-tests.sh

## Context

CLAUDE.md's "BDD Immutable-Tests Rule" says the rule is "optionally enforced via
a PreToolUse hook (`shared/scripts/protect-tests.sh`)" — but that script does
not exist. The documentation makes a false claim. This change makes it true.

## Scope

In:

- New `shared/scripts/protect-tests.sh` (PreToolUse Write|Edit|MultiEdit):
  - House conventions: hook-log lib, `set -uo pipefail`.
  - Read `file_path` (and `tool_name`) from stdin JSON.
  - No-op (exit 0) when the project has no `tests/features/` directory.
  - Block (exit 2, stderr guidance citing the BDD rule + BDD-006) when the
    target is an EXISTING `tests/steps/*.steps.ts`, `tests/support/*.ts`, or
    `tests/features/*.feature` AND the tool is Edit/MultiEdit (mutating an
    existing file).
  - Allow: Write to a non-existent path (new file), anything under
    `tests/features/drafts/**`, and any path when `PROMETHEUS_ALLOW_TEST_EDITS=1`
    (logged warning).
- Wire into hooks.json PreToolUse — widen the `Write|Edit` matcher to
  `Write|Edit|MultiEdit` and append protect-tests.sh after cedar-skill-gate.
- New `shared/scripts/tests/test-protect-tests.sh`: block existing step edit;
  allow draft; allow new file; allow override env; no-op without tests/features.

## Tasks

- [x] 1. Write protect-tests.sh (+x)
- [x] 2. Wire into hooks.json (widen matcher, append)
- [x] 3. Write test; run green

## Verification

Test green; hooks.json parses; editing an existing .feature is blocked, a draft
feature is allowed, override env bypasses.
