# change-evolver-010 — Phase-seeding protocol + liter-llm context management reference

**Phase:** pmpo-evolver
**Priority:** MEDIUM — completes inner-loop bridge; delivers liter-llm context management guidance (operator addendum)
**Gaps:** G-14
**Goals:** G4, G1 (addendum: liter-llm context management)
**Model class:** small (script); small (model-discovery authoring)
**Depends on:** all prior changes (defines how evolver plan items become KBD phases)

## Problem

No automated bridge exists from an evolver plan item to a KBD phase (G-14). When the idea-validation gate approves a spec, or when the competitive analysis identifies a capability gap, a new KBD phase must be seeded manually. This breaks the inner-loop automation. Also: the operator addendum requires a complete liter-llm model-discovery reference and context management guide.

## Solution

Create `scripts/evolver-seed-phase.sh` to automate KBD phase creation from evolver plan items. Create `references/context-management.md` with the pmpo-evolver-specific context budget rules. Create `skills/process/liter-llm-bridge/references/model-discovery.md` documenting how to query the configured system for available providers and models.

## New script: scripts/evolver-seed-phase.sh

```bash
#!/usr/bin/env bash
set -euo pipefail
EVOLUTION_NAME="${1:?Usage: evolver-seed-phase.sh <evolution-name> <plan-item-id>}"
PLAN_ITEM_ID="${2:?}"

EVOLVER_DIR=".evolver/${EVOLUTION_NAME}"
STATE_FILE="${EVOLVER_DIR}/state.json"
PHASES_DIR=".kbd-orchestrator/phases"
WAYPOINT=".kbd-orchestrator/current-waypoint.json"

if [ ! -f "${STATE_FILE}" ]; then
  echo "[seed-phase] ERROR: No state.json found at ${STATE_FILE}" >&2
  exit 1
fi

echo "[seed-phase] Seeding KBD phase from evolver plan item: ${PLAN_ITEM_ID}"

# Extract plan item from state.json
ITEM_DATA=$(python3 -c "
import json, sys
with open('${STATE_FILE}') as f:
    state = json.load(f)

plan = state.get('plan', {}).get('items', [])
item = next((i for i in plan if i.get('id') == '${PLAN_ITEM_ID}'), None)
if not item:
    print(json.dumps({'error': 'Plan item not found: ${PLAN_ITEM_ID}'}))
    sys.exit(1)
print(json.dumps(item))
")

if echo "${ITEM_DATA}" | python3 -c "import json,sys; d=json.load(sys.stdin); sys.exit(0 if 'error' not in d else 1)" 2>/dev/null; then
  : # ITEM_DATA is valid
else
  echo "[seed-phase] ERROR: ${ITEM_DATA}" >&2
  exit 1
fi

# Derive phase name from plan item
PHASE_NAME=$(echo "${ITEM_DATA}" | python3 -c "
import json, sys, re
item = json.load(sys.stdin)
name = item.get('name', item.get('id', 'evolver-phase'))
# kebab-case
name = re.sub(r'[^a-z0-9-]', '-', name.lower())
name = re.sub(r'-+', '-', name).strip('-')
print(name[:60])
")

PHASE_DIR="${PHASES_DIR}/${PHASE_NAME}"

if [ -d "${PHASE_DIR}" ]; then
  echo "[seed-phase] Phase directory already exists: ${PHASE_DIR}"
  echo "[seed-phase] Skipping creation — run /kbd-assess ${PHASE_NAME} to continue"
  exit 0
fi

mkdir -p "${PHASE_DIR}"
echo "[seed-phase] Created phase directory: ${PHASE_DIR}"

# Write goals.md from evolver plan item's success_criteria
python3 -c "
import json
item = json.loads('${ITEM_DATA}'.replace(\"'\", '\\\"') if False else '''${ITEM_DATA}''')

goals_lines = ['# Goals — ' + item.get('name', '${PLAN_ITEM_ID}'), '']
goals_lines.append('**Source:** pmpo-evolver plan item \`${PLAN_ITEM_ID}\` from evolution \`${EVOLUTION_NAME}\`')
goals_lines.append('')
goals_lines.append('**Description:** ' + item.get('description', ''))
goals_lines.append('')

criteria = item.get('success_criteria', item.get('acceptance_criteria', []))
if criteria:
    goals_lines.append('## Acceptance Criteria (from evolver spec)')
    goals_lines.append('')
    for i, c in enumerate(criteria, 1):
        goals_lines.append(f'- G{i}: {c}')
else:
    goals_lines.append('## Goals')
    goals_lines.append('')
    goals_lines.append('- G1: (derive goals from evolver plan item description above)')

target_state = item.get('target_state', '')
if target_state:
    goals_lines.append('')
    goals_lines.append('## Target State')
    goals_lines.append('')
    goals_lines.append(target_state)

with open('${PHASE_DIR}/goals.md', 'w') as f:
    f.write('\n'.join(goals_lines) + '\n')

print('goals.md written')
"

# Write progress.json
GOAL_COUNT=$(python3 -c "
import json
item = json.loads('''${ITEM_DATA}''')
criteria = item.get('success_criteria', item.get('acceptance_criteria', []))
print(max(len(criteria), 1))
")

python3 -c "
import json, datetime
progress = {
  'phase': '${PHASE_NAME}',
  'stage': 'assessment_ready',
  'changes_total': 0,
  'changes_completed': 0,
  'active_change': None,
  'next_pending_change': None,
  'assessment_complete': False,
  'plan_complete': False,
  'changes': [],
  'started': datetime.date.today().isoformat(),
  'updated': datetime.datetime.utcnow().isoformat() + 'Z',
  'evolver_source': {
    'evolution_name': '${EVOLUTION_NAME}',
    'plan_item_id': '${PLAN_ITEM_ID}'
  }
}
with open('${PHASE_DIR}/progress.json', 'w') as f:
    json.dump(progress, f, indent=2)
print('progress.json written')
"

# Write evolver-bridge.json stub
python3 -c "
import json
bridge = {
  'evolution_name': '${EVOLUTION_NAME}',
  'evolver_plan_path': '.evolver/${EVOLUTION_NAME}/state.json',
  'source_plan_item_id': '${PLAN_ITEM_ID}',
  'item_to_change_map': {}
}
with open('${PHASE_DIR}/evolver-bridge.json', 'w') as f:
    json.dump(bridge, f, indent=2)
print('evolver-bridge.json stub written')
"

# Update current-waypoint.json
if [ -f "${WAYPOINT}" ]; then
  python3 -c "
import json
with open('${WAYPOINT}') as f:
    w = json.load(f)

w['phase'] = '${PHASE_NAME}'
w['stage'] = 'assessment_ready'
w['previousPhase'] = w.get('phase', '')
w['exact_next_command'] = '/kbd-assess ${PHASE_NAME}'
w['active_change'] = None
w['changes_total'] = 0
w['changes_completed'] = 0

with open('${WAYPOINT}', 'w') as f:
    json.dump(w, f, indent=2)
print('Waypoint updated')
"
fi

echo "[seed-phase] Phase '${PHASE_NAME}' seeded successfully"
echo "[seed-phase] Next: /kbd-assess ${PHASE_NAME}"
echo ""
echo "  Phase dir: ${PHASE_DIR}"
echo "  Goals:     ${PHASE_DIR}/goals.md"
echo "  Progress:  ${PHASE_DIR}/progress.json"
echo "  Bridge:    ${PHASE_DIR}/evolver-bridge.json"
```

## New file: references/context-management.md

**Contents:**

### pmpo-evolver context budget rules

**Rule 1: Feedback collection is always isolated.**
`feedback-digest.sh` runs as a subprocess — never inline. The session reads only the normalized `LearningSignal[]` JSON output, not the raw feed data (which can be thousands of lines).

**Rule 2: Changelog ingestion is always isolated.**
`changelog-fetch.sh` may process many release notes. Run isolated; session reads structured JSON output only.

**Rule 3: Carry-forward aggregation is always isolated.**
May scan dozens of reflection.md files; run as subprocess. Session reads deduplicated JSON list.

**Rule 4: Strategic dreaming is always isolated.**
`post-cycle-dream.sh` loads the full journal.md but runs in its own process. Session reads the 2-3 lesson entries, not the full journal.

**Rule 5: Model class and context budget are correlated.**
- `small` models: keep inputs under 4k tokens. Use for file reads, git log, schema validation, binary classification.
- `medium` models: under 16k tokens. Use for NLP classification, web research synthesis, structured extraction.
- `frontier` models: full window. Use for strategic synthesis, spec generation, competitive analysis. Prefer concise inputs to preserve budget.

**Rule 6: Enumerate before loading.**
Before loading any reference document, check if a summary or index exists first. Prefer `.evolver/<name>/state.json` over re-reading all source files.

**Rule 7: liter-llm cost tracking in development.**
Enable `get_cost` polling after each `complete` call. Target: feedback collection + changelog ingestion ≤10% of frontier-all cost. Use this to verify that class assignments are actually reducing cost before shipping.

### Context budget estimation table

| Task | Model class | Input size estimate | Relative cost |
|------|-------------|---------------------|--------------|
| Perspective routing selection | small | <1k tokens | 1x |
| Commit classification (100 commits) | small | ~2k tokens | 2x |
| Issue sentiment (50 titles) | medium | ~3k tokens | 8x |
| Changelog feature extraction (3 releases) | medium | ~8k tokens | 20x |
| Parity matrix update | frontier | ~12k tokens | 150x |
| Strategic dreaming (journal excerpt) | frontier | ~6k tokens | 75x |
| Idea spec generation | frontier | ~4k tokens | 50x |

## New file: skills/process/liter-llm-bridge/references/model-discovery.md

**Contents:**

### How to discover available providers and models

**Step 1: Check configuration file**
```bash
CONFIG_PATH="${LITER_LLM_CONFIG:-$HOME/.config/liter-llm/config.toml}"
if [ -f "${CONFIG_PATH}" ]; then
  cat "${CONFIG_PATH}"
fi
```

**Step 2: Query liter-llm MCP `list_models` tool**
Returns: array of configured model aliases with resolved `{provider, model_id, class}`:
```json
[
  {"alias": "small", "provider": "anthropic", "model_id": "claude-haiku-4-5-20251001", "class": "small"},
  {"alias": "medium", "provider": "groq", "model_id": "llama-3.3-70b-versatile", "class": "medium"},
  {"alias": "frontier", "provider": "anthropic", "model_id": "claude-sonnet-4-6", "class": "frontier"}
]
```

**Step 3: Verify provider health**
Call liter-llm MCP `health` tool → returns per-provider status:
```json
{
  "anthropic": {"status": "ok", "latency_ms": 245},
  "groq": {"status": "ok", "latency_ms": 89},
  "ollama": {"status": "unreachable", "error": "connection refused"}
}
```

**Decision protocol:**
1. If `small` class configured AND provider healthy → use it for cheap tasks
2. If `medium` class configured AND provider healthy → use it for NLP tasks
3. If neither → fall through to `frontier` with a log warning
4. Never silently upgrade `small` to `frontier` — this defeats cost optimization
5. Never silently downgrade `frontier` to `medium` for strategic synthesis tasks

**Provider capability reference (as of 2026-06-28):**

| Provider | Cheap option | Mid option | Frontier option |
|----------|-------------|------------|----------------|
| Anthropic | claude-haiku-4-5-20251001 | claude-sonnet-4-6 | claude-opus-4-8 |
| Groq | llama-3.1-8b-instant | llama-3.3-70b-versatile | (none — use Anthropic for frontier) |
| Ollama (local) | qwen3:4b, phi4-mini | qwen3:14b | qwen3:32b (needs 32GB RAM) |
| vLLM (self-hosted) | depends on loaded model | depends | depends |

**Cost tracking:**
```bash
# After a complete call
COST=$(liter-llm mcp-call get_cost --session-id <id> 2>/dev/null || echo "0")
echo "[model-routing] Cost so far: $COST"
```

## Acceptance criteria

- [ ] `scripts/evolver-seed-phase.sh` is executable
- [ ] `bash scripts/evolver-seed-phase.sh nonexistent default` exits 1 with clear error
- [ ] Script creates goals.md, progress.json, and evolver-bridge.json stub when state.json has the item
- [ ] `skills/process/pmpo-evolver/references/context-management.md` exists with all 7 rules
- [ ] Context management table has cost estimates for all 7 task types
- [ ] `skills/process/liter-llm-bridge/references/model-discovery.md` exists
- [ ] model-discovery.md covers: config file, list_models, health check, decision protocol, provider reference, cost tracking
