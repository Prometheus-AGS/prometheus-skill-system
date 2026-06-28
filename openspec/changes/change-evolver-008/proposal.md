# change-evolver-008 — Strategic dreaming (post-cycle-dream)

**Phase:** pmpo-evolver
**Priority:** MEDIUM — Anthropic Dreaming pattern; distinct from PMPO Reflect
**Gaps:** G-10 (strategic memory layer)
**Goals:** G5
**Model class:** frontier (open-ended strategic synthesis)
**Depends on:** change-evolver-001 (schema: evolver_lessons[]), change-evolver-005 (learning signals — read as input)

## Problem

The existing iterative-evolver Reflect phase measures *execution quality* (did the plan succeed?). The existing KBD Reflect measures *goal alignment* (did we close the gaps?). Neither addresses *product direction learning* — what did this cycle reveal about where the product should go next? This is the Anthropic Dreaming gap (G-10 strategic memory layer).

## Solution

Create `references/strategic-dreaming.md` defining the dreaming protocol and output format. Create `scripts/post-cycle-dream.sh` to run isolated from the main session context and append to `evolver-lessons.md`.

## New file: references/strategic-dreaming.md

**Contents:**

### What strategic dreaming is

A lightweight post-cycle pass that asks: "What did we learn about *product direction* that we didn't know before this cycle?"

It runs AFTER the iterative-evolver cycle completes (after Reflect), and BEFORE the outer loop decides whether to continue or terminate.

**It is NOT:**
- PMPO Reflect (execution quality: which tasks succeeded/failed, what caused failures)
- KBD Reflect (goal alignment: did we close all gaps in the plan)
- A summary of what was built

**It IS:**
- A scan of the cycle journal for *direction signals* — evidence about what the product should become
- Falsified hypothesis detection ("we believed X; evidence shows Y")
- Threat escalation ("this competitor feature is moving faster than anticipated")
- Opportunity crystallization ("this user signal confirms a previously speculative direction")

### Output format (evolver-lessons.md entries)

```markdown
### <timestamp> — Cycle <N> Strategic Lesson

**Category:** direction | threat | opportunity | falsified-hypothesis
**Confidence:** high | medium | low
**Lesson:** <1-2 sentences: what we learned about product direction>
**Evidence:** <what in the cycle journal supports this>
**Impact on next cycle:** <how this should change what we plan next>
```

### When to run

At the end of every `iterative-evolver` cycle that has a `journal.md` entry, before calling `/pmpo-outer-loop tick` to advance the loop.

### Protocol

1. Read the current cycle's `journal.md` entry
2. Read the just-completed `reflection.md` (KBD Reflect output)
3. Read `evolution_state.evolver_lessons[]` to avoid re-deriving known lessons
4. Pass to liter-llm `complete(model=frontier)` with the dreaming prompt
5. Parse output for new `evolver_lesson` entries
6. Append to `evolver-lessons.md`
7. Update `evolution_state.evolver_lessons[]` in `state.json`

### Dreaming prompt structure

```
You are reviewing a product evolution cycle to extract strategic direction lessons.

CYCLE JOURNAL:
<journal-md-content>

REFLECTION SUMMARY:
<reflection-key-findings>

KNOWN LESSONS (do not re-derive):
<existing-evolver-lessons>

TASK: Identify 1-3 new lessons about PRODUCT DIRECTION learned from this cycle.
For each lesson:
- Category: direction | threat | opportunity | falsified-hypothesis
- Confidence: high | medium | low
- Lesson: <what we learned about where the product should go>
- Evidence: <specific evidence from the cycle journal or reflection>
- Impact on next cycle: <concrete action this implies for the next cycle>

Do NOT include:
- Summaries of what was built (that's in reflection)
- Task-level learnings (use vs. should-have-used)
- Success celebrations

Focus ONLY on strategic product direction signals.
```

### Context management

`post-cycle-dream.sh` runs as an isolated subprocess. The main evolver session does NOT need to load the full journal or reflection into its context — only the dream output (the new `evolver_lesson` entries) is surfaced back.

This is critical for context budget management: journal.md can be thousands of lines; the dream output is at most 3-4 lessons.

## New script: scripts/post-cycle-dream.sh

```bash
#!/usr/bin/env bash
set -euo pipefail
EVOLUTION_NAME="${1:?Usage: post-cycle-dream.sh <evolution-name> [--cycle <N>]}"
CYCLE_NUM="${3:-latest}"

EVOLVER_DIR=".evolver/${EVOLUTION_NAME}"
JOURNAL="${EVOLVER_DIR}/journal.md"
STATE="${EVOLVER_DIR}/state.json"
LESSONS_FILE="${EVOLVER_DIR}/evolver-lessons.md"
TIMESTAMP=$(date -u +%Y-%m-%dT%H:%M:%SZ)

echo "[post-cycle-dream] Strategic dreaming for ${EVOLUTION_NAME} cycle ${CYCLE_NUM}"

if [ ! -f "${JOURNAL}" ]; then
  echo "[post-cycle-dream] No journal.md found — skipping dream" >&2
  exit 0
fi

# Read latest reflection.md from .kbd-orchestrator
LATEST_REFLECTION=$(find .kbd-orchestrator/phases -name "reflection.md" -newer "${JOURNAL}" 2>/dev/null | head -1 || true)
if [ -z "${LATEST_REFLECTION}" ]; then
  # Fall back to most recently modified reflection
  LATEST_REFLECTION=$(find .kbd-orchestrator/phases -name "reflection.md" -type f 2>/dev/null | xargs ls -t 2>/dev/null | head -1 || true)
fi

# Read existing lessons to pass as context
EXISTING_LESSONS=""
if [ -f "${STATE}" ]; then
  EXISTING_LESSONS=$(python3 -c "
import json
with open('${STATE}') as f:
    state = json.load(f)
lessons = state.get('evolver_lessons', [])
if lessons:
    print('\n'.join([l.get('lesson','') for l in lessons[-5:]]))  # Last 5 only
")
fi

# Build the dreaming prompt context (truncated to avoid context overflow)
JOURNAL_EXCERPT=$(tail -200 "${JOURNAL}" 2>/dev/null || echo "No journal content")
REFLECTION_EXCERPT=""
if [ -n "${LATEST_REFLECTION}" ]; then
  REFLECTION_EXCERPT=$(head -100 "${LATEST_REFLECTION}" 2>/dev/null || echo "No reflection")
fi

echo "[post-cycle-dream] Running frontier model for strategic synthesis"
# [MODEL_ROUTING] phase=evolver-strategic-dream class=frontier

DREAM_OUTPUT=$(printf "CYCLE JOURNAL (recent):\n%s\n\nREFLECTION:\n%s\n\nKNOWN LESSONS:\n%s" \
  "${JOURNAL_EXCERPT}" "${REFLECTION_EXCERPT}" "${EXISTING_LESSONS}" | \
  liter-llm complete --model frontier \
    --system "You are reviewing a product evolution cycle to extract strategic direction lessons. Identify 1-3 new lessons about PRODUCT DIRECTION. For each: category (direction|threat|opportunity|falsified-hypothesis), confidence (high|medium|low), lesson (1-2 sentences), evidence, and impact_on_next_cycle. Output JSON array: [{category, confidence, lesson, evidence, impact_on_next_cycle}]" \
    2>/dev/null || echo '[{"category": "direction", "confidence": "low", "lesson": "Strategic dreaming requires liter-llm to be configured", "evidence": "liter-llm not available in this session", "impact_on_next_cycle": "Configure liter-llm model=frontier for this to work"}]')

# Parse and append to evolver-lessons.md
python3 -c "
import json, sys

lessons_file = '${LESSONS_FILE}'
timestamp = '${TIMESTAMP}'
evolution_name = '${EVOLUTION_NAME}'

try:
    new_lessons = json.loads('${DREAM_OUTPUT}')
    if not isinstance(new_lessons, list):
        new_lessons = [new_lessons]
except:
    print('[post-cycle-dream] Could not parse dream output as JSON', file=sys.stderr)
    sys.exit(0)

# Append to evolver-lessons.md
with open(lessons_file, 'a') as f:
    for i, lesson in enumerate(new_lessons):
        f.write(f\"\"\"\n### {timestamp} — Cycle ${CYCLE_NUM} Strategic Lesson {i+1}

**Category:** {lesson.get('category', 'direction')}
**Confidence:** {lesson.get('confidence', 'low')}
**Lesson:** {lesson.get('lesson', '')}
**Evidence:** {lesson.get('evidence', '')}
**Impact on next cycle:** {lesson.get('impact_on_next_cycle', '')}

\"\"\")

print(f'[post-cycle-dream] {len(new_lessons)} lessons appended to {lessons_file}')
print(json.dumps({'new_lessons': len(new_lessons), 'lessons': new_lessons}, indent=2))
"

# Update state.json evolver_lessons[]
if [ -f "${STATE}" ]; then
  python3 -c "
import json
with open('${STATE}') as f:
    state = json.load(f)
try:
    new_lessons = json.loads('${DREAM_OUTPUT}')
    if not isinstance(new_lessons, list):
        new_lessons = [new_lessons]
    for lesson in new_lessons:
        lesson['origin_cycle'] = int('${CYCLE_NUM}') if '${CYCLE_NUM}'.isdigit() else 0
    state.setdefault('evolver_lessons', []).extend(new_lessons)
except:
    pass
with open('${STATE}', 'w') as f:
    json.dump(state, f, indent=2)
"
fi

echo "[post-cycle-dream] Complete"
```

## Acceptance criteria

- [ ] `skills/process/pmpo-evolver/references/strategic-dreaming.md` exists with protocol, output format, and dreaming prompt
- [ ] `scripts/post-cycle-dream.sh` is executable
- [ ] Script exits 0 when no journal.md exists (graceful skip)
- [ ] Script exits 0 when liter-llm is not available (graceful fallback with placeholder lesson)
- [ ] `[MODEL_ROUTING] phase=evolver-strategic-dream class=frontier` comment present in script
- [ ] `strategic-dreaming.md` clearly distinguishes strategic dreaming from PMPO Reflect and KBD Reflect
- [ ] Context management note present: dreaming runs isolated to avoid main session context overhead
