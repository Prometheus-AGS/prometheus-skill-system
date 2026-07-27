### change-dgp-008 — KBD lifecycle section
`/opsx:new change-dgp-008`
Expand the single `metaprompting-pmpo-kbd` guide page into a KBD-lifecycle
section: stages (assess/analyze/spec/plan/execute/reflect), hooks, waypoints,
progress signaling — sourced from the orchestrator SKILL.md.
Acceptance: section with ≥4 pages; mermaid stage diagram renders;
**non-duplication**: pages summarize and link to the canonical orchestrator
SKILL.md / CLAUDE.md sections rather than copying them (no verbatim section
>3 paragraphs); build green.
Agent: build. | library: —


## Evidence (implemented 2026-07-27)
- 4 pages (/docs/kbd/{overview,stages,hooks-and-waypoints,quality-gates}); build exit 0; routes verified.
- Mermaid stage diagram compiled into page JS chunks (client render confirmed).
- Non-duplication: every page summarizes + links canonical orchestrator sources
  (SKILL.md files, shared/lib) — no verbatim section >3 paragraphs.
- Gates: skipped (docs-only change per heuristic).
