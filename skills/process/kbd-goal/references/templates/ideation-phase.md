# Ideation Phase Template

The Ideation phase converges on 3 validated ideas before proceeding to
Specification. A discovery agent generates candidates; `kbd-idea-critic`
scores them adversarially; the loop repeats until ≥3 candidates score ≥7.0
aggregate; then a human gate surfaces `IDEAS.md` for selection.

## Loop Flow

```
goal.json (description + context)
    │
    ▼
[1] Discovery agent
    - Reads goal.json for description and any user-supplied inputs
      (notes, annoyances, existing repos referenced in goal description)
    - Proposes 3–5 candidate ideas in structured format
    - Writes candidates to IDEAS.md (draft section)
    │
    ▼
[2] kbd-idea-critic subagent (stronger model — Sonnet)
    - Scores each candidate on 4 rubric dimensions (0–10 each):
        feasibility, pain_addressed, stack_fit, buildability
    - Aggregate = mean of 4 dimensions
    - Returns JSON: {candidates[], survivors[], loop_verdict, loop_reason}
    - Survivors = candidates with aggregate ≥ 7.0
    │
    ▼
[3] Convergence check
    - survivors.length >= 3 → STOP → write final IDEAS.md → human gate
    - survivors.length < 3 → CONTINUE → discovery agent generates more
      candidates (different angle — avoid repeating failed ones)
    │
    ▼ (on STOP)
[4] Human gate
    - IDEAS.md surfaced to user
    - User selects preferred candidate
    - Selection recorded in IDEAS.md → Selected section
    - Phase marked complete; Specification phase begins
```

## Stopping Condition

`≥3 candidates with aggregate ≥7.0 written to IDEAS.md`

Hard limits: `max_turns_per_phase` (default 50 turns) prevents infinite spin.
After `max_no_progress_turns` (default 3) consecutive turns with no new
survivors, loop escalates to human.

## Discovery Agent Instructions

The discovery agent (main builder context, not a separate subagent) should:

1. Read `goal.json` for description, tool, and any user-supplied context.
2. Generate candidates that are:
   - Concrete and buildable (not vague directions)
   - Scoped to the project's existing stack where possible
   - Distinct from each other (no variations of the same idea)
3. Format each candidate as:
   ```markdown
   ### Candidate: <Title>
   **What it does:** One sentence.
   **Why it solves pain:** One sentence.
   **Stack:** Technologies used.
   **Rough implementation:** 2–3 steps.
   ```
4. Append to `IDEAS.md` under `## Draft Candidates — Round N`.

## Critic Agent Invocation

After the discovery agent writes candidates, invoke `kbd-idea-critic`:

```
Invoke kbd-idea-critic with:
- goal.json path: .kbd-orchestrator/goals/<slug>/goal.json
- IDEAS.md path: .kbd-orchestrator/goals/<slug>/IDEAS.md
```

Read the JSON response. Update `IDEAS.md`:
- Add scored table row for each candidate
- Update survivor count
- If `loop_verdict == STOP`: write final `## Survivors` section

## IDEAS.md Format

```markdown
# Ideas — <goal description>

## Scored Candidates

| Candidate | Feasibility | Pain | Stack Fit | Buildability | Aggregate | Verdict |
|-----------|------------|------|-----------|--------------|-----------|---------|
| ...       | 9          | 8    | 9         | 10           | 9.0       | PASS    |

## Survivors (≥7.0 aggregate)

1. **<Title>** (aggregate) — one-sentence description
2. ...
3. ...

## Selected (human gate)

> To proceed: select one candidate. Write "Selected: <Title>" in this section.
```

## Platform Notes

This phase is always KBD-owned — it does not delegate to platform-native
`/goal` commands. The discovery + critic pattern runs identically on all 5
platforms: the agents are invoked as KBD subagents, not as platform primitives.
