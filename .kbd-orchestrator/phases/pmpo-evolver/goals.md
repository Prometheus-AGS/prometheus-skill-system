# Goals — pmpo-evolver

**Phase:** pmpo-evolver
**Created:** 2026-06-28
**Previous phase:** pmpo-elicit

---

## Goals

- G1: Ship `skills/process/pmpo-evolver/SKILL.md` — a `/pmpo-evolver` slash command that drives the strategy layer above individual KBD phases
- G2: Define the evolver schema (`evolver.schema.json`) — loop definition, iteration state, synthesis fields, and termination conditions
- G3: Wire the evolver into the KBD outer loop (`pmpo-outer-loop`) so it decides *what phase to run next* based on accumulated reflection results
- G4: Integrate evolver output with the KBD inner loop — evolver plan items become KBD phases; phase reflections feed back to the evolver's Reflect stage
- G5: Support evolver state persistence — `evolution.json` with iteration history, synthesis snapshots, and a resumable cursor so long multi-phase evolutions survive context compaction and tool switches
- G6: Platform-agnostic: evolver state files and CLI work identically on Claude Code, Codex, OpenCode, Kimi, and Zed

---

## Success Criteria

- `/pmpo-evolver` is invocable and produces a phase plan from a high-level evolution goal
- An evolution cycle (assess → plan → execute → reflect) can run across multiple KBD phases with the evolver tracking aggregate progress
- Reflections from individual KBD phases are automatically synthesized into the evolver's running state
- The evolver can be paused and resumed (across sessions and tools) without losing iteration history
- All file formats validate against the evolver schema
