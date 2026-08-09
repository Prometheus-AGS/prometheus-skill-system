---
name: learn-about-system
description: Zero-friction adoption entry point for the Prometheus Skill Pack. Routes operators into the Feynman learning loop for the KBD lifecycle, skill pack capabilities, or AI harness orientation. The skill pack teaches itself using its own learning infrastructure.
version: '1.0.0'
license: MIT
metadata:
  author: prometheus-skill-pack
  category: learn
  tags: [learn, adoption, meta-learning, kbd, skill-pack, harness, onboarding]
---

# learn-about-system

## When to invoke

The first thing a new operator runs when they want to understand the Prometheus
Skill Pack or the KBD lifecycle. Invoked as:

```
/learn-about-system [--area kbd|skills|harness]
```

Without `--area`: enters interactive discovery mode and asks what the operator
wants to learn about before routing.

## Self-teaching loop

This skill demonstrates the core proposition of the learn domain: the skill
pack teaches itself using its own learning infrastructure. The same Feynman loop
that teaches "linear algebra" or "Rust ownership" also teaches "how to use the
KBD lifecycle" and "what skills exist in this pack".

This is not a special case. It uses the identical flow:

```
learn-about-system → learn-goal (with pre-built corpus) → learn-survey
  → feynman-loop → learn-grade → learn-retain
```

The pre-built meta-corpora (`kbd-lifecycle-corpus.json`,
`skill-pack-corpus.json`) replace the live corpus assembly step. Everything
downstream is unchanged.

See [references/self-teaching-loop.md](references/self-teaching-loop.md) for
the full cycle description.

## Interactive discovery mode (no --area)

When invoked with no arguments, use ui-surface to ask:

```json
{
  "intent_type": "question",
  "title": "What would you like to learn about?",
  "body": "Choose a topic to get started:",
  "options": [
    "The KBD lifecycle (assess, analyze, plan, execute, reflect, evolve)",
    "The skill pack capabilities (what skills exist and what they do)",
    "A specific AI harness (Claude Code, OpenCode, Codex, Kimi, Zed)",
    "Something else (describe it and I'll set up a learning goal)"
  ],
  "multiselect": false,
  "metadata": {}
}
```

Route the response:

| Selection | Route |
|---|---|
| 1 — KBD lifecycle | `--area kbd` |
| 2 — Skill pack capabilities | `--area skills` |
| 3 — AI harness | `--area harness` |
| 4 — Something else | `/learn-goal` (standard entry, no pre-built corpus) |

## Routing by --area

### `--area kbd`

Loads the pre-built KBD lifecycle corpus and routes directly to `learn-goal`,
bypassing corpus assembly (corpus is already built).

```bash
LEARN_ABOUT_SYSTEM_DIR="<directory containing this SKILL.md>"
CORPUS_PATH="${LEARN_ABOUT_SYSTEM_DIR}/references/kbd-lifecycle-corpus.json"
SUBJECT="KBD Lifecycle — Prometheus Skill Pack development process"
TARGET_LEVEL="practitioner"
```

Steps:
1. Read `kbd-lifecycle-corpus.json` and surface the source count and
   misconception count to the operator.
2. Emit the session start message (see Handoff section).
3. Call `/learn-goal` with corpus pre-loaded — the goal intake step begins at
   **Step 3 (time-to-mastery research)**, skipping Steps 1–2 (corpus assembly),
   because the corpus is already provided.
4. After the goal artifact is written, print the `/learn-survey <goal-id>` prompt.

### `--area skills`

Loads the pre-built skill pack corpus and follows the same flow.

```bash
LEARN_ABOUT_SYSTEM_DIR="<directory containing this SKILL.md>"
CORPUS_PATH="${LEARN_ABOUT_SYSTEM_DIR}/references/skill-pack-corpus.json"
SUBJECT="Prometheus Skill Pack capabilities and architecture"
TARGET_LEVEL="practitioner"
```

Same steps 1–4 as `--area kbd`.

### `--area harness`

Ask which harness the operator wants to learn about:

```json
{
  "intent_type": "question",
  "title": "Which AI harness?",
  "body": "Select the harness you want to orient to:",
  "options": [
    "Claude Code",
    "OpenCode",
    "Codex",
    "Kimi Code",
    "Zed",
    "Other (I'll describe it)"
  ],
  "multiselect": false,
  "metadata": {}
}
```

Route to `/learn-harness --harness <name>` with the selected harness. If "Other"
is selected, collect the harness name via a follow-up prompt, then route to
`/learn-goal` with subject set to `"<harness-name> AI harness — setup and usage"`.

## What the operator learns

### For `kbd` area, key concepts covered:

- The six lifecycle stages: `new-phase`, `assess`, `analyze`, `plan`, `execute`,
  `reflect` (and the optional `evolve` extension)
- Progress signaling: reading `position-reminder.txt`, emitting
  `Starting <skill> (step N of T)` / `Completed <skill> (step N of T)` signals
- OpenSpec as the change management backend (proposals, tasks, status lifecycle)
- Waypoint files: `current-waypoint.json` and `position-reminder.txt`
- The sycophancy gate on reflection (binary path, strictness knob,
  2-rejection soft cap)

### For `skills` area, key concepts covered:

- Skill domains: `react`, `rust`, `learn`, `process`, and others
- Dual-format distribution: agentskills.io standard + Claude Code plugin
- Cross-platform installation (`install-skills-flat.sh`) and MCP server setup
- surreal-memory vs Cortex memory distinction and when to use each
- `hooks/hooks.json` canonical path and symlink structure

## Common misconceptions addressed

Both corpora contain `misconception` entries that `learn-survey` probes during
assessment. Examples the operator will encounter:

**KBD area:**
- "kbd-plan writes code" — it produces a plan artifact only; `kbd-execute` writes code
- "position-reminder.txt is optional" — it is required; all kbd-* skills read it
  before any other action
- "reflect is just a summary of what was done" — it must follow Delta → Root Cause
  → Corrective Actions structure or the sycophancy gate rejects it

**Skills area:**
- "skills only work on Claude Code" — they follow the agentskills.io standard
  and are cross-platform (OpenCode, Codex, Kimi, Zed, Cursor, and others)
- "the .claude-plugin/ directory is the source of truth" — `skills/` is the
  source of truth; `.claude-plugin/` contains symlinks built by `npm run build`
- "surreal-memory and Cortex are interchangeable" — surreal-memory is preferred
  and offers graph-RAG; Cortex is the fallback when surreal-memory tools are
  absent

## Handoff

After routing to the appropriate corpus and confirming the subject and target
level, emit:

```
Starting Feynman learning loop for: <subject>
Corpus: <N> source entries, <M> misconception entries
Run /learn-survey <goal-id> when ready to assess your starting knowledge.
```

Downstream skills read `~/.prometheus/learn/goals/<goal-id>/goal.json` for all
context. They do not re-prompt the operator for fields captured here.

## ui-surface integration

All prompts in this skill pass through ui-surface:

```bash
UI_SURFACE_DIR="<directory containing the installed ui-surface/SKILL.md>"
TIER_JSON=$(bash "${UI_SURFACE_DIR}/scripts/detect-surface-tier.sh" --json)
TIER=$(echo "$TIER_JSON" | jq -r '.tier')

RESPONSE=$(bash "${UI_SURFACE_DIR}/scripts/render.sh" \
  --tier "$TIER" \
  --intent-json "$INTENT_JSON")
```

Never render prompts directly. ui-surface handles tier detection and harness
parity across Claude Code, OpenCode, Codex, Kimi, and Zed.

## Directory layout

```
skills/learn/learn-about-system/
├── SKILL.md                          — this file
└── references/
    └── self-teaching-loop.md         — full cycle description with entry → retain arc
```
