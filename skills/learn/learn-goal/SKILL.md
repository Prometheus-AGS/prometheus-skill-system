---
name: learn-goal
description: Entry point for the Feynman learning flow. Accepts a learning desire, assembles a grounded corpus (public and/or custom KB), runs an honest feasibility gate with sycophancy-correction, and produces a goal artifact that downstream learn-* skills consume.
version: '1.0.0'
license: MIT
metadata:
  author: prometheus-skill-pack
  category: learn
  tags: [learn, goal, feynman, feasibility, corpus, entry-point]
---

# learn-goal

## When to invoke

Invoked when the user wants to begin a learning journey. Examples:

- "I want to master Rust's borrow checker"
- "I need to understand cancer immunotherapy pathways"
- "Help me learn the PMPO framework"
- "I want to get good at machine learning in 6 weeks"

The skill gates all downstream learn-* skills. Nothing runs until a valid
`goal.json` artifact exists.

## Flow

### Step 1 — Collect the learning desire

Use ui-surface (Tier 1 preferred) to gather:

| Field | Prompt | Valid values |
|---|---|---|
| `subject` | "What do you want to learn?" | Any non-empty string |
| `target_level` | "What proficiency level are you aiming for?" | novice / practitioner / expert |
| `weekly_hours` | "How many hours per week can you commit?" | Positive number |
| `total_weeks` | "How many weeks is your learning window?" | Positive integer |
| `kb_id` | "Do you have a private knowledge base to include? (optional)" | dify:<name>, palace:<id>, local:<path>, or blank |

Collect fields sequentially if the user's initial message is missing any.
Do not proceed to corpus assembly until all required fields are present.

### Step 2 — Assemble corpus

Resolve `LEARN_GOAL_DIR` as the directory containing this `SKILL.md`, then run
the bundled `content-grounding.sh` with the subject and level:

```bash
bash "${LEARN_GOAL_DIR}/scripts/content-grounding.sh" \
  --subject "$SUBJECT" \
  --level "$TARGET_LEVEL" \
  --budget-sources 10 \
  --output "${GOAL_DIR}/corpus-public.json"
```

If `--kb` was provided, also run `content-grounding-kb.sh` and merge the
`sources` arrays. KB sources go first — higher authority for the custom domain:

```bash
bash "${LEARN_GOAL_DIR}/scripts/content-grounding-kb.sh" \
  --kb "$KB_ID" \
  --subject "$SUBJECT" \
  --level "$TARGET_LEVEL" \
  --output "${GOAL_DIR}/corpus-kb.json"
```

Merge rule: `sources = kb_sources + public_sources`, deduplicated by `source_ref`.

Write the merged corpus to `${GOAL_DIR}/corpus.json`.

### Step 3 — Research time-to-mastery

Use the assembled corpus to estimate realistic time-to-mastery for the target
level. Express as a range: `N–M weeks at P hours/week`.

Ground the estimate in the corpus sources where possible. If corpus is thin,
use well-established domain heuristics (e.g., Dreyfus model stage times).

Do not invent optimistic numbers. Err toward honesty.

### Step 4 — Run feasibility gate

Compare the research-derived estimate to the operator's stated availability:

```
operator_hours_total = weekly_hours × total_weeks
estimate_lower_hours = lower_bound_weeks × P_hours_per_week
```

| Color | Condition |
|---|---|
| GREEN | operator_hours_total ≥ 80% of estimate_lower_hours |
| YELLOW | operator_hours_total is 40–79% of estimate_lower_hours |
| RED | operator_hours_total < 40% of estimate_lower_hours |

### Step 5 — Sycophancy-correct the feasibility

Before showing the feasibility assessment to the user, detect sycophantic
patterns via the `sycophancy-correction` MCP tool:

```
detect_sycophancy(text: <draft_feasibility_assessment>)
```

If sycophancy is detected (score ≥ 0.3 or any HIGH/CRITICAL pattern):
- Rewrite the assessment to be honest about the mismatch
- Do not soften RED to YELLOW to please the user
- Do not add empty encouragement ("but you can do it!")

### Step 6 — Show result to user

Use ui-surface to present the feasibility result:

```json
{
  "intent_type": "feedback",
  "title": "Feasibility: <COLOR>",
  "body": "<sycophancy-corrected assessment>",
  "options": null,
  "multiselect": false,
  "metadata": {"feasibility": "<COLOR>"}
}
```

**RED**: Recommend a more modest scope (lower target level) or longer timeline.
Ask the user if they want to adjust and re-run, or proceed with explicit
acknowledgment of the difficulty.

**YELLOW**: Proceed with explicit acknowledgment: "This is achievable but tight.
You will need to protect your learning time consistently."

**GREEN**: Proceed.

Do not automatically adjust the goal on the user's behalf. The user decides.

### Step 7 — Write goal artifact

After the user confirms the goal (adjusted or original), call `scripts/write-goal.sh`:

```bash
GOAL_JSON=$(jq -n \
  --arg goal_id "$GOAL_ID" \
  --arg subject "$SUBJECT" \
  --arg target_level "$TARGET_LEVEL" \
  --argjson weekly_hours "$WEEKLY_HOURS" \
  --argjson total_weeks "$TOTAL_WEEKS" \
  --arg feasibility "$FEASIBILITY" \
  --arg feasibility_note "$FEASIBILITY_NOTE" \
  --arg corpus_path "${GOAL_DIR}/corpus.json" \
  --arg kb_id "${KB_ID:-null}" \
  --arg created_at "$(date -u +"%Y-%m-%dT%H:%M:%SZ")" \
  '{
    goal_id: $goal_id,
    subject: $subject,
    target_level: $target_level,
    weekly_hours: $weekly_hours,
    total_weeks: $total_weeks,
    feasibility: $feasibility,
    feasibility_note: $feasibility_note,
    corpus_path: $corpus_path,
    kb_id: (if $kb_id == "null" then null else $kb_id end),
    created_at: $created_at
  }')

bash "${SKILL_DIR}/scripts/write-goal.sh" --goal-json "$GOAL_JSON"
```

## Goal artifact schema

```json
{
  "goal_id": "string (uuid-like, e.g. learn-rust-borrow-20240628-a3f2)",
  "subject": "string",
  "target_level": "novice|practitioner|expert",
  "weekly_hours": 5,
  "total_weeks": 8,
  "feasibility": "GREEN|YELLOW|RED",
  "feasibility_note": "string — honest 1–3 sentence assessment",
  "corpus_path": "string — absolute path to corpus.json",
  "kb_id": "string|null",
  "created_at": "ISO 8601 datetime"
}
```

### Goal ID generation

Combine subject slug + target level + date + 4-char random suffix:

```bash
SLUG=$(echo "$SUBJECT" | tr '[:upper:]' '[:lower:]' | sed 's/[^a-z0-9]\+/-/g' | sed 's/^-//;s/-$//')
DATE=$(date -u +"%Y%m%d")
SUFFIX=$(head -c 4 /dev/urandom | xxd -p | head -c 4)
GOAL_ID="${SLUG}-${TARGET_LEVEL}-${DATE}-${SUFFIX}"
```

Goal files live at: `~/.prometheus/learn/goals/<goal-id>/`

## Custom KB integration

If the user provides a `--kb` flag or mentions a specific knowledge domain:

1. Prompt for the KB identifier if not already given:
   - `dify:<name>` — Dify knowledge base
   - `palace:<id>` — surreal-memory palace ID
   - `local:<path>` — local directory of documents

2. Run `content-grounding-kb.sh --kb <id>` alongside the public corpus call.

3. Merge: KB sources precede public sources in the `sources` array.

4. Record `kb_id` in the goal artifact so downstream skills can re-query the
   same KB without asking the user again.

## Feasibility examples

| Scenario | Estimate | Operator budget | Color | Honest note |
|---|---|---|---|---|
| Quantum field theory in 2 weeks at 4h/week | 3–5 years at 20h/week | 8 hours total | RED | QFT requires years of foundational physics first. 2 weeks builds only surface familiarity at best. |
| Python basics in 4 weeks at 10h/week | 3–6 weeks at 8–10h/week | 40 hours total | GREEN | Solid foundation is achievable in this window with consistent practice. |
| ML fundamentals in 6 weeks at 5h/week | 8–12 weeks at 8h/week | 30 hours total | YELLOW | Possible but tight. Priority focus on core concepts; some breadth will be deferred. |

## Downstream handoff

After writing `goal.json`, print:

```
Goal recorded: <subject> (<target_level>) — feasibility: <COLOR>
Next: /learn-survey <goal-id>
```

Downstream skills read `~/.prometheus/learn/goals/<goal-id>/goal.json` for
all context they need. They do not re-prompt the user for fields already
captured here.

## Directory layout

```
skills/learn/learn-goal/
├── SKILL.md          — this file
└── scripts/
    └── write-goal.sh — writes goal.json to ~/.prometheus/learn/goals/<id>/
```
