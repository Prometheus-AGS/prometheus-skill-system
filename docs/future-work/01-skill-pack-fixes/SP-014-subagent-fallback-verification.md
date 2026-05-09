---
id: SP-014
title: SubagentStop fallback matcher verification
status: planned
priority: P2
estimated_effort: 0.5d
agent_role: hooks-engineer
depends_on: [SP-006]
unblocks: []
related: [SP-013]
created_from_conversation_turn: 3-4
---

# SP-014 — SubagentStop fallback matcher verification

## Problem

`docs/plans/2026-04-29-change-006-karpathy-loop-hooks.md` asserts that a SubagentStop fallback matcher works (matches subagents that don't have a more-specific matcher). The assertion is documented but not verified by test. If the fallback doesn't actually match what the doc claims, hooks intended to apply broadly will silently miss subagents.

## Evidence

1. Read the change-006 plan section about the fallback matcher.
2. Read `hooks.json`. Identify the fallback matcher entry (likely a `*` or `.*` pattern).
3. Inspect: are there any subagent types currently in use that should be matched but might not be?

## Why it matters

This is straightforward verification debt. Either the assertion is correct (and we add a test to lock it in), or it's wrong (and we fix it before SP-013's reflector matcher is wired and we discover the bug there).

Before SP-013 lands, this should be verified. SP-013's value depends on the matcher firing reliably.

## Proposed fix

Write a small test that:

1. Spawns a synthetic Task subagent with a contrived `subagent_type`.
2. Triggers SubagentStop.
3. Asserts the fallback hook fires and writes its expected log line.

Repeat with a few `subagent_type` variants to confirm the fallback fires for all unmatched types.

If the test fails (i.e. fallback doesn't actually match), correct the matcher pattern. Ship the fix and the test.

## Trade-offs and risks

- **Risk: synthetic subagent test is hard to set up.** Mitigation: simplest path is to invoke the Task tool with an unusual subagent_type and observe via SP-006's hook log. No new test infrastructure required.

## Acceptance criteria

- [ ] A test in `shared/scripts/tests/` (or wherever bats tests live) creates a synthetic subagent and asserts the fallback hook fires.
- [ ] If the test reveals a bug, the matcher pattern is corrected.
- [ ] Test passes consistently across 5 invocations.

## Implementation steps

1. Identify the test infrastructure (bats? plain shell?).
2. Write the test with synthetic subagent invocation.
3. Run; if fail, debug the matcher pattern and fix.
4. Run again; lock in.

## Dependencies

SP-006 (hook log) — without it, "did the hook fire?" is hard to verify.

## Open questions

- Is there a way to test SubagentStop without actually invoking a real subagent (e.g. by emitting a mock event into the hook system)? Worth investigating; would speed up the test.
