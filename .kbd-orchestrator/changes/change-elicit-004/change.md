---
id: change-elicit-004
title: kbd-analyze operative contested-stack escalation protocol
phase: pmpo-elicit
gaps: [G-06]
goals: [G3]
priority: MEDIUM
effort: S
agent: claude-code
status: done
scope:
  - skills/process/kbd-process-orchestrator/skills/kbd-analyze/SKILL.md
---

# change-elicit-004 — kbd-analyze wiring

## Context

`kbd-analyze/SKILL.md` currently says:

> "A contested choice (score gap < 15%) escalates via `/pmpo-elicit` when available;
> otherwise flag it for the user in `analysis.md` and the decision log — never
> silently pick a contested stack."

This is correct intent but prose-only. No operative call protocol exists. When a
session runs `/kbd-analyze` and hits a contested stack, it has no procedural anchor
to follow. This change adds a concrete step-by-step in the Stack Discovery section.

## Scope

### `skills/process/kbd-process-orchestrator/skills/kbd-analyze/SKILL.md` (MODIFY)

In the `## The research pipeline` → `### Modes` → `**Stack discovery**` subsection,
expand the contest clause into an operative protocol block:

```markdown
### Contested stack escalation (score gap < 15%)

When the top two stack options in `stack-recommendation.md` are within 15% of
each other:

1. **Construct the elicitation request:**
   - `question`: "Two stacks are equally matched: <A> (<scoreA>%) vs <B> (<scoreB>%). Which should we use?"
   - `hints`: ["<A> key advantage", "<B> key advantage", "primary tradeoff"]
   - `criticality`: high
   - `caller`: kbd-analyze
   - `write_back_path`: decision-log.md

2. **On Claude Code:** Present via `AskUserQuestion` with the two stack names as options
   plus "Research further" and "Accept highest-ranked (implicit)". Record answer in
   `decision-log.md` with `provenance` and `elicitation_id`.

3. **On other platforms:** Call `pmpo-elicit-checkpoint.sh` with the constructed args.
   Write checkpoint to `.kbd-orchestrator/phases/<phase>/elicitations/<id>/`.
   Pause analysis. On resume, apply result to `decision-log.md`.

4. **Unavailable (no pmpo-elicit skill):** Flag the contest in `analysis.md` under
   "Open Questions", note both options with scores, ask the user inline before
   continuing.

Record in `decision-log.md`:
```
### <timestamp> — Contested stack choice
Options: <A> vs <B> | Score gap: <N>%
Decision: <chosen> | Provenance: <user|research|implicit>
Elicitation ID: <id>
```
```

## Tasks

- [x] 1. Add operative contested-stack protocol to `kbd-analyze/SKILL.md`
- [x] 2. Verify no backslash violations introduced (run `npm run validate:skill skills/process/kbd-process-orchestrator`)
