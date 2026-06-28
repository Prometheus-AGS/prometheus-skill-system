# Specification Phase Template

The Specification phase turns a selected idea into a spec precise enough to
drive the Creation loop's stopping condition. A spec-writer drafts `SPEC.md`;
`kbd-spec-reviewer` stress-tests it adversarially; the writer revises; the loop
ends only when the reviewer returns PASS. Then a human gate approves `SPEC.md`.

## Why This Phase Matters

The loop's stopping condition IS the spec's acceptance criteria. A vague spec
produces a vague evaluator check, which produces a self-certifying builder.
The reviewer's job is to make the spec airtight before a single line of code
is written.

## Loop Flow

```
IDEAS.md (selected idea)
    │
    ▼
[1] Spec-writer (main builder context)
    - Reads IDEAS.md → Selected candidate
    - Reads goal.json for stack and constraints
    - Drafts SPEC.md with:
        • User stories (As a / I want / So that)
        • Exact CLI/API signatures (not "a CLI" — the actual flags and formats)
        • I/O formats (exact schema or examples)
        • Acceptance criteria per story (machine-checkable)
        • Explicit non-goals
    │
    ▼
[2] kbd-spec-reviewer subagent (adversarial Sonnet)
    - Reads SPEC.md
    - Tests each acceptance criterion: is it machine-verifiable?
    - Returns {verdict: PASS|FAIL, gaps[], verdict_reason}
    │
    ▼
[3] Convergence check
    - reviewer verdict == PASS → write final SPEC.md → human gate
    - reviewer verdict == FAIL → writer reads gaps[], revises SPEC.md → repeat
    │
    ▼ (on PASS)
[4] Human gate
    - SPEC.md surfaced to user
    - User approves or requests changes
    - On approval: phase marked complete; Creation phase begins
      (SPEC.md acceptance criteria become the per-task stopping conditions)
```

## Stopping Condition

`kbd-spec-reviewer returns PASS on SPEC.md`

Hard limit: `max_turns_per_phase` (default 50). After 3 consecutive reviewer
cycles with the same gap unfixed, loop escalates to human with the stuck gap
highlighted.

## Spec-Writer Instructions

The spec-writer (main builder context) should produce a `SPEC.md` with:

### Required Sections

**User Stories**
```markdown
| # | As a... | I want to... | So that... |
|---|---------|-------------|------------|
| US-01 | developer | run `standup` with no args | I get yesterday's commits grouped by day |
```

**CLI / API Contract**
```markdown
## CLI Contract

\`\`\`
standup [--since <YYYY-MM-DD>] [--repo <path>] [--format slack|markdown|json]
\`\`\`

Defaults: `--since` = yesterday, `--repo` = current directory, `--format` = markdown
```

**Acceptance Criteria**
```markdown
| ID   | Story | Criterion | Verifiable by |
|------|-------|-----------|---------------|
| AC-01 | US-01 | Output groups commits by calendar day (UTC) | test/verify-grouping.sh |
| AC-02 | US-01 | Each day has max 5 bullets; overflow truncated with count | test/verify-truncation.sh |
| AC-03 | US-02 | Exit code 0 on success, 1 on invalid flag, 2 on missing repo | test/exit-codes.sh |
```

**Non-Goals**
```markdown
## Non-Goals

- Does NOT send to Slack automatically
- Does NOT parse linked issue tracker tickets
- Does NOT support multiple repos in one run
```

## Reviewer Invocation

After the spec-writer writes `SPEC.md`, invoke `kbd-spec-reviewer`:

```
Invoke kbd-spec-reviewer with:
- SPEC.md path: .kbd-orchestrator/goals/<slug>/SPEC.md
```

Read the JSON response:
- `verdict == PASS`: proceed to human gate
- `verdict == FAIL`: read `gaps[]`; revise `SPEC.md` addressing each gap; repeat

## Platform Notes

This phase is always KBD-owned. The reviewer runs as a KBD subagent on all
5 platforms. The `SPEC.md` acceptance criteria table becomes the stopping
condition input for the Creation phase's `kbd-goal-evaluator` checks.
