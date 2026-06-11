---
id: change-002-position-prompt-hook
title: UserPromptSubmit position injection hook
phase: position-and-handoff-guarantee
gaps: [F1]
priority: P1
effort: S
agent: claude-code
evolver_item_id: null
status: proposed
scope:
  - shared/scripts/position-on-prompt.sh
  - hooks/hooks.json
  - shared/scripts/tests/test-position-render.sh
---

# change-002 — UserPromptSubmit position injection hook

## Context

The per-turn position guarantee starts with the model *seeing* the position
every turn. UserPromptSubmit stdout (exit 0) is injected into context — the
proven pk-focus-on-prompt.sh mechanism.

## Scope

In:

- New `shared/scripts/position-on-prompt.sh`:
  - House conventions: hook-log lib, `set -uo pipefail`, ALWAYS exit 0.
  - Sources `lib/waypoint-render.sh`; if render output empty → silent exit 0.
  - Emits the rendered block plus one instruction line:
    "MANDATORY: begin your response with the Position line above and end with a
    'Next:' line reflecting the updated state."
- `hooks/hooks.json`: append command to the existing UserPromptSubmit group:
  `bash ${CLAUDE_PLUGIN_ROOT}/shared/scripts/position-on-prompt.sh 2>/dev/null`,
  timeout 5000. Edit the canonical file directly (never via .claude-plugin symlink).
- Test: extend test-position-render.sh with a hook-invocation case (pipe fixture
  stdin JSON, assert block + instruction on stdout, exit 0).

Out: Stop-gate enforcement (change-003).

## Tasks

- [ ] 1. Write `shared/scripts/position-on-prompt.sh` (+x)
- [ ] 2. Wire into `hooks/hooks.json` UserPromptSubmit
- [ ] 3. Extend test; run green

## Verification

Test green; `jq . hooks/hooks.json` parses; manual: `echo '{}' | bash shared/scripts/position-on-prompt.sh` prints block in this repo.
