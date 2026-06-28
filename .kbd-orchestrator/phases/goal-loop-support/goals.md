# Goals — goal-loop-support

## Primary Goal

Add platform-agnostic goal-driven iterative loop support to the KBD orchestrator — implementing or augmenting Claude Code's `/goal` functionality so it works on any AI tool (OpenCode, Codex, Kimi, Cursor, Windsurf, etc.) using the existing KBD assess → analyze → plan → execute → reflect cycle structure.

## Specific Goals

- **G1** Implement a `/kbd-goal` entry point that accepts a goal statement and optional stopping conditions, then drives the full KBD lifecycle autonomously until the goal is met, a turn/time limit is hit, or human escalation is triggered.
- **G2** Support multi-phase goal decomposition: Ideation → Specification → Creation (and optionally Deployment), each as a KBD child phase, with inner loops where sub-tasks require deeper breakdown.
- **G3** Implement a separated evaluator pattern — a second agent/model instance that grades stopping conditions independently from the builder agent, preventing self-grading bias.
- **G4** Integrate with the existing `pmpo-outer-loop` loop definition schema (`.kbd-orchestrator/loops/<name>/loop.json`) so goal state is persistent and resumable across sessions.
- **G5** Platform-agnostic: the mechanism must work on Claude Code (may delegate to native `/goal`), OpenCode, Codex CLI, Kimi Code, and any tool that can run a `claude -p` subprocess or equivalent.
- **G6** Inner-loop support: tasks identified as complex during execution should spawn child KBD phases automatically rather than stalling the parent loop.
- **G7** Skill/MCP discovery: at goal start, identify and load required skills and MCP servers needed for the goal domain.
