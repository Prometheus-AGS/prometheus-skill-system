# change-goal-005: Specification Child-Phase Template

**Phase:** goal-loop-support
**Status:** pending
**Sub-phase:** A (core)
**Depends on:** change-goal-004

## Problem

No Specification phase exists. Without a spec-writer + adversarial spec-reviewer loop, specifications are vague, untestable, and unfit as stopping conditions for later Creation loops.

## Solution

Build a Specification phase template with a hardline `kbd-spec-reviewer` subagent that rejects any criterion it cannot verify. Loop ends only when reviewer returns PASS. Human gate surfaces `SPEC.md` for approval.

## Files

- `skills/process/kbd-goal/references/templates/spec-phase.md` (CREATE)
- `agents/kbd-spec-reviewer.md` (CREATE)

## Tasks

- [ ] Write `spec-phase.md` template documenting the writer→reviewer→revision loop
- [ ] Writer agent: reads `IDEAS.md` selected idea → drafts `SPEC.md` with user stories, exact CLI/API signatures, I/O formats, non-goals, acceptance criteria per story
- [ ] Write `agents/kbd-spec-reviewer.md`: adversarial system prompt — rejects vague criteria ("summarize nicely" → FAIL, "group commits by day max 5 bullets" → PASS); returns `{verdict: PASS|FAIL, gaps: string[]}`
- [ ] Define `SPEC.md` output format: user stories table, CLI/API contract block, acceptance criteria table, non-goals list
- [ ] Document loop exit: reviewer PASS → human gate for `SPEC.md` approval
- [ ] Update `kbd-goal/SKILL.md` with Specification Phase section
