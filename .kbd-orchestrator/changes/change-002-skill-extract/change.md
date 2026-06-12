---
id: change-002-skill-extract
title: Extract orchestrator SKILL.md Hooks section under 500 lines
phase: outer-loop-and-ux
gaps: [U2]
priority: P2
effort: S
agent: claude-code
evolver_item_id: null
status: done
scope:
  - skills/process/kbd-process-orchestrator/SKILL.md
  - skills/process/kbd-process-orchestrator/references/hooks.md
  - skills/process/kbd-process-orchestrator/references/cross-tool-protocol.md # scope expansion: needed 2 more extractions to clear 500 lines
  - skills/process/kbd-process-orchestrator/references/memory-integration.md # scope expansion
---

# change-002 — SKILL.md extraction

## Context

Orchestrator SKILL.md is 619 lines (>500 warn), four phases overdue for
extraction. The "Hooks" section (~100 lines, lines 393-491) is the largest
self-contained block.

## Scope

In:

- Move the full "## Hooks" section body from SKILL.md to a new
  `references/hooks.md` (taxonomy, wiring stanza, debugging).
- Leave a short "## Hooks" pointer in SKILL.md: one paragraph + a link to
  `references/hooks.md`.
- Keep Progress Signals, lifecycle, and Quick Start IN SKILL.md (load-bearing).
- Result: SKILL.md under 500 lines; validate:strict warning cleared.

## Tasks

- [x] 1. Create references/hooks.md with the extracted Hooks content
- [x] 2. Replace the SKILL.md Hooks section with a pointer
- [x] 3. Confirm validate:strict has no line-count warning

## Verification

`npm run validate:strict skills/process/kbd-process-orchestrator` shows no
500-line warning; the Hooks content is intact in references/hooks.md.
