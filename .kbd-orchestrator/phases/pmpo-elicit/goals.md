# Goals — pmpo-elicit

## Primary Goal

Build the `/pmpo-elicit` skill — the structured human-escalation primitive referenced throughout the KBD codebase but never implemented. When KBD encounters an ambiguity, a contested decision, or a human gate, it must have a reliable way to pause, present the decision to the operator with full context, collect a structured response, and resume the loop from exactly where it stopped.

## Specific Goals

- **G1** Ship `skills/process/pmpo-elicit/SKILL.md` — a `/pmpo-elicit` slash command that presents a decision, collects a structured response, and writes the result to a well-defined location so the calling loop can resume.
- **G2** Define the elicitation schema (`elicit.json`) — the format for describing a decision point (question, context, options, default, urgency, timeout) and recording the operator's answer.
- **G3** Wire `/pmpo-elicit` into the KBD lifecycle at all documented escalation points: `kbd-analyze` contested stack decisions, `kbd-goal` human gates (Ideation → Spec, Spec → Creation), `pmpo-outer-loop` escalation_points array, and inner-loop `STATE.md → escalations[]`.
- **G4** Support async elicitation — the loop can pause and resume from a checkpoint without losing state; the operator can respond minutes or hours later.
- **G5** Platform-agnostic: the elicitation mechanism must work across Claude Code, Codex, OpenCode, Kimi Code, and Zed, using the same `elicit.json` checkpoint file as the shared state contract.
