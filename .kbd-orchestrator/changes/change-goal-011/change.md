---
id: change-goal-011
title: Zed Dual-Track Strategy
phase: goal-loop-support
subphase: B (integration)
depends_on: [change-goal-001, change-goal-002]
agent: claude-code
status: done
scope:
  - scripts/kbd-goal-zed-detect.sh
  - skills/process/kbd-goal/references/platforms/zed.md
  - skills/process/kbd-goal/SKILL.md
---

# change-goal-011 — Zed Dual-Track Strategy

## Problem

Zed has no native `/goal`. When running with an ACP-connected agent (Claude Code, Codex), that agent's native `/goal` is available. When standalone, Zed needs KBD to implement the loop. Without detection, the wrong strategy is used.

## Solution

Build `kbd-goal-zed-detect.sh` to detect the ACP agent (or standalone), and document the dual-track strategy in the `kbd-goal` skill: delegate to the connected agent's bridge, or implement the loop inline via repeated `session/prompt` calls with evaluator invocations.

## Files

- `scripts/kbd-goal-zed-detect.sh` (CREATE)
- `skills/process/kbd-goal/references/platforms/zed.md` (CREATE)
- `skills/process/kbd-goal/SKILL.md` (UPDATE: Zed section)

## Tasks

- Write `kbd-goal-zed-detect.sh`: check `$ZED_ACP_AGENT` env var → output agent name; fallback to `~/.zed/acp-agents.json`; if neither → output `standalone`
- Document Track 1 (ACP-connected): detect Claude Code → use goal-007 bridge; detect Codex → use goal-008 bridge
- Document Track 2 (standalone): after each turn, `kbd-goal` skill invokes `kbd-goal-evaluator` subagent; if FAIL, inject continuation as next Zed `session/prompt` message
- Write `zed.md` platform reference with both tracks, setup steps, and ACP configuration guide
- Add install entry for `~/.zed/skills/` in `install-skills-flat.sh` (already present for kbd-goal; verify)
- Note: `disable-model-invocation: false` on `kbd-goal-evaluator.md` allows Zed agent to invoke it as a skill tool

## Tasks

- [x] 1. Write `kbd-goal-zed-detect.sh`: check `$ZED_ACP_AGENT` env var → output agent name; fallback to `~/.zed/acp-agents.json`; if neither → output `standalone`
- [x] 2. Document Track 1 (ACP-connected): detect Claude Code → use goal-007 bridge; detect Codex → use goal-008 bridge
- [x] 3. Document Track 2 (standalone): after each turn, `kbd-goal` skill invokes `kbd-goal-evaluator` subagent; if FAIL, inject continuation as next Zed `session/prompt` message
- [x] 4. Write `zed.md` platform reference with both tracks, setup steps, and ACP configuration guide
- [x] 5. Add install entry for `~/.zed/skills/` in `install-skills-flat.sh` (already present for kbd-goal; verify)
- [x] 6. Note: `disable-model-invocation: false` on `kbd-goal-evaluator.md` allows Zed agent to invoke it as a skill tool
