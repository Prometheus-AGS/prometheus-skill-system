---
id: change-elicit-005
title: kbd-goal human-gate wiring via pmpo-elicit
phase: pmpo-elicit
gaps: [G-07]
goals: [G3]
priority: MEDIUM
effort: M
agent: claude-code
status: done
scope:
  - skills/process/kbd-goal/SKILL.md
  - skills/process/kbd-goal/references/platforms/claude-code.md
  - skills/process/kbd-goal/references/platforms/kimi.md
---

# change-elicit-005 — kbd-goal human-gate wiring

## Context

`kbd-goal/SKILL.md` documents human gates as prose:
- "After Ideation: Review IDEAS.md, select your preferred candidate"
- "After Specification: Review SPEC.md, approve or request changes"
- "After Creation: Review STATE.md summary, confirm deployment readiness"

No pmpo-elicit integration exists. The gates are unstructured — the agent asks inline
text and there's no recorded decision, provenance, or elicitation_id in `goal.json`.

This change adds an operative wiring protocol for the two critical gates (Ideation→Spec
and Spec→Creation), plus the STATE.md escalations[] write protocol for in-Creation
elicitations.

## Scope

### `skills/process/kbd-goal/SKILL.md` (MODIFY)

Replace the existing "Human Gates Between Phases" section with the operative protocol:

**Ideation → Spec gate (after kbd-goal-evaluator PASS on ideation):**

```
When not --auto-gates:

1. Invoke /pmpo-elicit:
   - question: "Ideation complete. IDEAS.md has <N> candidates. Which direction to pursue?"
   - Construct 2-4 options from IDEAS.md candidate names
   - hints: ["top candidate", "alternative direction"]
   - criticality: high
   - caller: kbd-goal/ideation

On Claude Code:  use AskUserQuestion with candidate options
On other platforms: write checkpoint to goals/<slug>/elicitations/<id>/

2. On result:
   - Record in goal.json → phases[ideation].human_gate_result:
     {"decision": "<candidate>", "provenance": "<provenance>", "elicitation_id": "<id>"}
   - "revision-needed": re-enter ideation loop with revision notes
   - Any named candidate: proceed to Spec phase, seed SPEC.md from selected idea

When --auto-gates:
   - Record goal.json → phases[ideation].human_gate_result:
     {"decision": "auto-approved", "provenance": "implicit", "elicitation_id": null}
```

**Spec → Creation gate (after kbd-goal-evaluator PASS on spec):**

```
When not --auto-gates:

1. Invoke /pmpo-elicit:
   - question: "Specification complete. SPEC.md ready. How do you want to proceed?"
   - options: ["Approve — begin Creation", "Request revision", "Stop here"]
   - hints: ["spec summary", "key criteria"]
   - criticality: high
   - caller: kbd-goal/spec

2. On result:
   - "Approve — begin Creation": record approved, proceed
   - "Request revision": re-enter Spec loop with revision notes as feedback
   - "Stop here": write final state summary, set goal.json → status = "stopped-at-spec"
   - Record in goal.json → phases[spec].human_gate_result (same structure as above)
```

**STATE.md escalations[] write protocol (during Creation phase):**

```
When any elicitation is triggered during the Creation phase:

1. On checkpoint written: append to STATE.md → escalations[]:
   {"id": "<elicitation-id>", "question": "<question>", "status": "pending", "task_id": "<active-task>"}

2. On resume (result.json written):
   Update the escalations[] entry: {"status": "resolved", "provenance": "<provenance>", "answer": "<answer>"}
```

### `skills/process/kbd-goal/references/platforms/claude-code.md` (MODIFY)

Add a "Human gates" subsection documenting that gates use `AskUserQuestion` in-session
with the candidate options populated from IDEAS.md / SPEC.md summary. No checkpoint
file needed on Claude Code — result is written to goal.json immediately.

### `skills/process/kbd-goal/references/platforms/kimi.md` (MODIFY)

Add a "Human gates" subsection: `kbd-goal-check` detects `pending_elicitation` state in
`goal.json` (added by the checkpoint). The next `/goal next` step is an "elicitation
response" step: the user writes `result.json`, then queues the gate-decision step.

## Tasks

- [ ] 1. Replace "Human Gates Between Phases" section in `kbd-goal/SKILL.md` with operative protocol
- [ ] 2. Add STATE.md escalations[] write protocol section in `kbd-goal/SKILL.md`
- [ ] 3. Add "Human gates" subsection to `references/platforms/claude-code.md`
- [ ] 4. Add "Human gates" subsection to `references/platforms/kimi.md`
- [ ] 5. `npm run validate:strict skills/process/kbd-goal` passes clean
