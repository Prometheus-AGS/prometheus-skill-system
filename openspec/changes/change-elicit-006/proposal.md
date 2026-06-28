---
id: change-elicit-006
title: pmpo-outer-loop stall escalation protocol + SKILLS.md description update
phase: pmpo-elicit
gaps: [G-08, G-09]
goals: [G3, G1]
priority: MEDIUM
effort: S
agent: claude-code
status: done
scope:
  - skills/process/pmpo-outer-loop/SKILL.md
  - SKILLS.md
---

# change-elicit-006 — pmpo-outer-loop wiring + SKILLS.md polish

## Context

`pmpo-outer-loop/SKILL.md` says "escalate via `/pmpo-elicit` (continue / re-plan / stop)"
in two places but provides no operative protocol. Operators following the skill for the first
time have no procedural anchor for what to do when the loop stalls.

`SKILLS.md` line 196 still has the old description: "PMPO artifact elicitation: draw out
requirements, constraints, and goals" — this is the framing from before the ask-or-research
design. It doesn't describe the four-option-class mechanism or async support.

## Scope

### `skills/process/pmpo-outer-loop/SKILL.md` (MODIFY)

In the `### /loop-tick <name>` section, expand the stall/regression point (currently step 3
bullet "regression or max_no_progress_ticks reached → escalate via /pmpo-elicit") into an
operative protocol subsection:

```markdown
#### Stall/regression escalation (operative protocol)

When `max_no_progress_ticks` is reached or a measurable regression is detected:

1. Construct the elicitation:
   - `question`: "Loop '<name>' has stalled after <N> ticks with no progress. How should we proceed?"
   - `options`: ["Continue — run another tick", "Re-plan — revise the evolution goal", "Stop — terminate loop and write final report"]
   - `context`: "Last tick delta vs measurable_criteria: <diff>"
   - `criticality`: blocking
   - `caller`: pmpo-outer-loop/loop-tick

2. On Claude Code: use `AskUserQuestion` with the three options. Record choice
   in `loops/<name>/journal.md` latest entry → `escalation_result`.

3. On other platforms: call `pmpo-elicit-checkpoint.sh` with the above args.
   Checkpoint dir: `.kbd-orchestrator/loops/<name>/elicitations/<caller>-<timestamp>/`.
   Set loop status to "paused-escalation" in `loop.json`. On resume, apply result.

4. Outcome logic:
   - "Continue": reset `no_progress_ticks` counter to 0, run next tick.
   - "Re-plan": invoke `/pmpo-elicit` again with question "What change to the
     evolution goal?" (criticality: high), update `loop.json → goal` with result,
     reset counters, continue.
   - "Stop": write final `/loop-report`, set `loop.json → status = "terminated-by-operator"`.

Record in `decision-log.md`:
```
### <timestamp> — Loop stall escalation
Loop: <name> | Ticks stalled: <N>
Decision: <continue|replan|stop> | Provenance: <user|implicit>
Elicitation ID: <id>
```
```

Also update the "hard ceilings" / bounds paragraph (line ~100 in current SKILL.md) to
cross-reference `references/escalation-points.md`:
```markdown
See `../pmpo-elicit/references/escalation-points.md` for the full platform routing
table and async checkpoint contract.
```

### `SKILLS.md` (MODIFY)

Update the `pmpo-elicit` row (line 196):

**Before:**
```
| `pmpo-elicit` | PMPO artifact elicitation: draw out requirements, constraints, and goals |
```

**After:**
```
| `pmpo-elicit` | Ask-or-research human escalation primitive: present a decision with four option classes (direct answer / named source / autonomous research / explicit implicit), collect a structured answer with provenance, support async pause/resume across all platforms |
```

## Tasks

- [ ] 1. Add stall/regression operative protocol subsection to `pmpo-outer-loop/SKILL.md`
- [ ] 2. Add cross-reference to `escalation-points.md` in pmpo-outer-loop bounds paragraph
- [ ] 3. Update `SKILLS.md` pmpo-elicit description line 196
- [ ] 4. `npm run validate:strict skills/process/pmpo-outer-loop` passes clean
