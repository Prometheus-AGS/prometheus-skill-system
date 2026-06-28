#!/usr/bin/env bash
# post-cycle-dream.sh — Strategic dreaming pass after a KBD execution cycle
# Usage: post-cycle-dream.sh <evolution-name> [<kbd-phase-name>]
# Run as an isolated subprocess — never inline in the evolver session.
# [MODEL_ROUTING] phase=evolver-post-dream class=frontier
set -euo pipefail

EVOLUTION_NAME="${1:?Usage: post-cycle-dream.sh <evolution-name> [<kbd-phase-name>]}"
PHASE_NAME="${2:-}"

EVOLVER_DIR=".evolver/${EVOLUTION_NAME}"
mkdir -p "${EVOLVER_DIR}"

STATE_FILE="${EVOLVER_DIR}/state.json"
LESSONS_FILE="${EVOLVER_DIR}/evolver-lessons.md"

echo "[post-cycle-dream] Evolution: ${EVOLUTION_NAME}, Phase: ${PHASE_NAME:-auto}" >&2

# Locate reflection.md
if [ -n "${PHASE_NAME}" ] && [ -f ".kbd-orchestrator/phases/${PHASE_NAME}/reflection.md" ]; then
  REFLECTION_FILE=".kbd-orchestrator/phases/${PHASE_NAME}/reflection.md"
else
  REFLECTION_FILE=$(find .kbd-orchestrator/phases -name "reflection.md" -newer "${STATE_FILE:-/dev/null}" 2>/dev/null | head -1 || true)
fi

REFLECTION_EXCERPT=""
if [ -n "${REFLECTION_FILE}" ] && [ -f "${REFLECTION_FILE}" ]; then
  REFLECTION_EXCERPT=$(head -80 "${REFLECTION_FILE}")
  echo "[post-cycle-dream] Using reflection: ${REFLECTION_FILE}" >&2
fi

# Locate latest learning signals
LATEST_SIGNALS_FILE=$(find "${EVOLVER_DIR}" -name "learning-signals-*.json" 2>/dev/null | sort | tail -1 || true)
SIGNALS_EXCERPT=""
if [ -n "${LATEST_SIGNALS_FILE}" ] && [ -f "${LATEST_SIGNALS_FILE}" ]; then
  SIGNALS_EXCERPT=$(python3 -c "
import json
with open('${LATEST_SIGNALS_FILE}') as f:
    d = json.load(f)
signals = d.get('new_signals', [])[:5]
for s in signals:
    print(f\"- [{s.get('severity','?')}] {s.get('source_type','?')}: {s.get('signal','')}\")
")
  echo "[post-cycle-dream] Using signals: ${LATEST_SIGNALS_FILE}" >&2
fi

# Read existing lesson titles for deduplication
EXISTING_TITLES=""
if [ -f "${STATE_FILE}" ]; then
  EXISTING_TITLES=$(python3 -c "
import json
with open('${STATE_FILE}') as f:
    state = json.load(f)
lessons = state.get('evolver_lessons', [])
for l in lessons:
    print('- ' + l.get('title', ''))
" 2>/dev/null || echo "")
fi

if [ -z "${REFLECTION_EXCERPT}" ] && [ -z "${SIGNALS_EXCERPT}" ]; then
  echo "[post-cycle-dream] No input artifacts found — producing placeholder lesson" >&2
  TIMESTAMP=$(date -u +%Y-%m-%dT%H:%M:%SZ 2>/dev/null || \
    python3 -c "from datetime import datetime,timezone; print(datetime.now(timezone.utc).strftime('%Y%m%dT%H%M%SZ'))")
  printf '{"lessons_added": 0, "note": "No input artifacts found — run after kbd-reflect completes"}\n'
  exit 0
fi

# Build prompt
PROMPT="You are a strategic product analyst. Given the following artifacts from a completed evolution cycle, identify 2-3 enduring lessons about where this product should evolve next.

## Evolution: ${EVOLUTION_NAME}
## Completed cycle: ${PHASE_NAME:-unknown}

## Reflection excerpt
${REFLECTION_EXCERPT:-No reflection available.}

## Recent learning signals (top 5)
${SIGNALS_EXCERPT:-No learning signals available.}

## Existing lessons (do not duplicate these)
${EXISTING_TITLES:-None yet.}

Produce exactly 2-3 lessons. Each lesson must:
- Be about product DIRECTION, not execution quality
- Be falsifiable (could be proven wrong by evidence)
- Include an actionable implication for the next cycle
- NOT duplicate any existing lesson

Output JSON only:
{
  \"lessons\": [
    {
      \"title\": \"string (5-8 words)\",
      \"perspective\": \"competitive|trend|unique-product|idea-validation|self-learning\",
      \"confidence\": \"high|medium|low\",
      \"signal_sources\": [\"carry-forward\",\"learning-signals\",\"reflection\",\"changelog\"],
      \"body\": \"string (2-3 sentences)\",
      \"implication\": \"string (1 sentence)\"
    }
  ]
}"

# Call liter-llm if available, otherwise emit placeholder
if ! command -v liter-llm > /dev/null 2>&1; then
  echo "[post-cycle-dream] liter-llm not available — emitting placeholder lessons" >&2
  TIMESTAMP=$(date -u +%Y-%m-%dT%H:%M:%SZ 2>/dev/null || \
    python3 -c "from datetime import datetime,timezone; print(datetime.now(timezone.utc).strftime('%Y%m%dT%H%M%SZ'))")
  printf '{"lessons_added": 0, "note": "liter-llm unavailable — install liter-llm and re-run for strategic dreaming"}\n'
  exit 0
fi

# [MODEL_ROUTING] phase=evolver-post-dream class=frontier
RESPONSE=$(echo "${PROMPT}" | liter-llm complete --model frontier 2>/dev/null)

TIMESTAMP=$(date -u +%Y-%m-%dT%H:%M:%SZ 2>/dev/null || \
  python3 -c "from datetime import datetime,timezone; print(datetime.now(timezone.utc).strftime('%Y%m%dT%H%M%SZ'))")

# Parse response and write lessons
python3 -c "
import json, sys, os, re

response = '''${RESPONSE}'''
timestamp = '${TIMESTAMP}'
evolution_name = '${EVOLUTION_NAME}'
phase_name = '${PHASE_NAME:-unknown}'
evolver_dir = '${EVOLVER_DIR}'
lessons_file = '${LESSONS_FILE}'
state_file = '${STATE_FILE}'

# Extract JSON from response
json_match = re.search(r'\{.*\}', response, re.DOTALL)
if not json_match:
    print(json.dumps({'lessons_added': 0, 'error': 'Could not parse liter-llm response as JSON'}))
    sys.exit(0)

try:
    data = json.loads(json_match.group(0))
    lessons = data.get('lessons', [])
except json.JSONDecodeError as e:
    print(json.dumps({'lessons_added': 0, 'error': str(e)}))
    sys.exit(0)

# Write to evolver-lessons.md (append)
with open(lessons_file, 'a') as f:
    for lesson in lessons:
        f.write('\n## Lesson: ' + lesson.get('title', 'Untitled') + '\n\n')
        f.write('**Cycle:** ' + phase_name + '\n')
        f.write('**Perspective:** ' + lesson.get('perspective', 'unknown') + '\n')
        f.write('**Confidence:** ' + lesson.get('confidence', 'medium') + '\n')
        sources = lesson.get('signal_sources', [])
        f.write('**Signal sources:** ' + ', '.join(sources) + '\n\n')
        f.write(lesson.get('body', '') + '\n\n')
        f.write('**Actionable implication:** ' + lesson.get('implication', '') + '\n')

# Update state.json evolver_lessons[]
state = {}
if os.path.exists(state_file):
    with open(state_file) as f:
        try:
            state = json.load(f)
        except json.JSONDecodeError:
            state = {}

existing_lessons = state.get('evolver_lessons', [])
for i, lesson in enumerate(lessons):
    existing_lessons.append({
        'id': f'lesson-{timestamp}-{i}',
        'title': lesson.get('title', 'Untitled'),
        'cycle': phase_name,
        'perspective': lesson.get('perspective', 'unknown'),
        'confidence': lesson.get('confidence', 'medium'),
        'signal_sources': lesson.get('signal_sources', []),
        'body': lesson.get('body', ''),
        'implication': lesson.get('implication', ''),
        'created_at': timestamp
    })

state['evolver_lessons'] = existing_lessons
with open(state_file, 'w') as f:
    json.dump(state, f, indent=2)

print(json.dumps({'lessons_added': len(lessons), 'lessons_file': lessons_file}))
"

echo "[post-cycle-dream] Done." >&2
