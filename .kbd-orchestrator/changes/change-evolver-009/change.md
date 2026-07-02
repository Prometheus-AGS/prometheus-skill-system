---
id: change-evolver-009
title: Outer-loop perspective handoff (pmpo-outer-loop wiring)
phase: pmpo-evolver
gaps: [G-13]
priority: MEDIUM — wires outer loop perspective field to evolver; enables continuous evolution loops
goals: G3
agent: claude-code
status: done
scope:
  - scripts/loop-tick.sh
  - pmpo-outer-loop/SKILL.md
---

# change-evolver-009 — Outer-loop perspective handoff (pmpo-outer-loop wiring)

## Problem

`pmpo-outer-loop` has no way to specify which pmpo-evolver perspective should run on each tick (G-13). The `loop.json` has no `perspective` field. The `loop-tick.sh` script calls `/evolve` without a perspective flag, meaning the evolver always auto-routes. For operators who want a dedicated competitive-scan loop or a self-learning loop, there is no way to configure this.

## Solution

Add a `perspective` field to the `loop-definition.schema.json` (additive). Modify `scripts/loop-tick.sh` to pass the perspective flag when `perspective != "auto"`. Add a cross-reference paragraph to `pmpo-outer-loop/SKILL.md` under the `/loop-define` section.

## Modified file: loop-definition.schema.json

Add to the root `properties` object (additive, backward-compatible):
```json
"perspective": {
  "type": "string",
  "enum": ["competitive", "trend", "unique-product", "idea-validation", "self-learning", "combined", "auto"],
  "default": "auto",
  "description": "Which pmpo-evolver perspective to apply on each tick. 'auto' lets the evolver router decide based on data freshness and feedback signals."
}
```

## Modified file: scripts/loop-tick.sh

Read current script first to understand the exact call site. Then add:

```bash
# After reading loop.json, extract perspective
PERSPECTIVE=$(python3 -c "
import json
with open('${LOOP_JSON}') as f:
    loop = json.load(f)
print(loop.get('perspective', 'auto'))
" 2>/dev/null || echo "auto")

# When calling evolve/pmpo-evolver, pass perspective if not auto
if [ "${PERSPECTIVE}" != "auto" ]; then
  EVOLVE_FLAGS="--perspective ${PERSPECTIVE}"
else
  EVOLVE_FLAGS=""
fi

# Existing evolve call becomes:
# /pmpo-evolver "${LOOP_NAME}" ${EVOLVE_FLAGS}
```

The exact insertion point depends on reading the current file. The pattern to find and augment: wherever the script invokes the evolve or pmpo-evolver command.

## Modified file: pmpo-outer-loop/SKILL.md

Under the `/loop-define` section, add a paragraph:

```markdown
**Perspective configuration (pmpo-evolver integration):**
Set `perspective` in your `loop.json` to lock a loop to a specific evolution perspective:
- `auto` (default): the pmpo-evolver strategy router chooses based on data freshness and feedback signals
- `competitive`: every tick runs a competitive landscape scan
- `self-learning`: every tick collects learning signals from configured feedback sources
- `combined`: sequential routing through all relevant perspectives
- Other modes: `trend`, `unique-product`, `idea-validation`

Example `loop.json`:
```json
{
  "loop_name": "my-product-competitive",
  "perspective": "competitive",
  "feedback_sources": [{"type": "competitor-scan", "registry_path": ".evolver/my-product/competitor-registry.json"}]
}
```
```

## Acceptance criteria

- `loop-definition.schema.json` has `perspective` field with the 7-value enum
- Schema validates via `python3 -m json.tool`
- `loop-tick.sh` passes `--perspective <value>` when loop.json `perspective != "auto"`
- `loop-tick.sh` does NOT change behavior when `perspective == "auto"` or field absent (backward-compatible)
- `pmpo-outer-loop/SKILL.md` has the perspective paragraph under `/loop-define`
- `npm run validate:strict skills/process/pmpo-outer-loop` passes (if skill has validation)

## Tasks

- [x] 1. `loop-definition.schema.json` has `perspective` field with the 7-value enum
- [x] 2. Schema validates via `python3 -m json.tool`
- [x] 3. `loop-tick.sh` passes `--perspective <value>` when loop.json `perspective != "auto"`
- [x] 4. `loop-tick.sh` does NOT change behavior when `perspective == "auto"` or field absent (backward-compatible)
- [x] 5. `pmpo-outer-loop/SKILL.md` has the perspective paragraph under `/loop-define`
- [x] 6. `npm run validate:strict skills/process/pmpo-outer-loop` passes (if skill has validation)
