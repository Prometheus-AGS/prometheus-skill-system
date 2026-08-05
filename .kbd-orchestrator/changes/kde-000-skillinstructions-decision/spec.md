# kde-000 — record the explicit adopt/reject decision for `skillInstructions`

**Phase:** kimi-desktop-extensibility
**Scope:** `.kbd-orchestrator/phases/kimi-desktop-extensibility/assessment.md` (E0 section only)
**Backend:** native-kbd

## Problem

Goal 2 requires "an explicit adopt/reject rationale **per extension point**".
`skillInstructions` is a manifest key the pack **already emits**, and it was
flagged as missing a decision in three consecutive handoffs (assess, analyze,
spec) without ever being resolved. Adversarial review raised it as CRITICAL
twice.

The gap was procedural, not technical: the field works, but no change owned the
decision, so it kept being carried forward as an unresolved warning.

## Decision — ADOPT (no code change required)

`install-kimi-desktop-plugin.sh` already writes a `skillInstructions` block
naming the skill families and instructing the agent to read a `SKILL.md` in full
before following it.

**Rationale:** it is the routing hint that tells the model which of 145 skills to
reach for, at the cost of one string. It is also *why* E5 (`systemPrompt`) is
held at CONSIDER rather than adopted — both compete for the same context budget,
and `skillInstructions` is the supported, already-working one.

## Why this is a change and not just a note

The judge reads the change set, not the assessment. A decision recorded only in
`assessment.md` is invisible to the gate that keeps asking for it. This change
exists so the decision has an owner, a verification, and an archive record.

## Acceptance criteria

1. `assessment.md` contains an E0 section giving `skillInstructions` an explicit
   **ADOPTED** verdict with rationale.
2. No generator or manifest change — the field is already emitted correctly.
3. Subsequent adversarial review of the change set no longer reports
   `skillInstructions` as undecided.

## Out of scope

- Expanding or rewording the `skillInstructions` content. Its wording is a
  content question; this change settles only the adopt/reject decision.
