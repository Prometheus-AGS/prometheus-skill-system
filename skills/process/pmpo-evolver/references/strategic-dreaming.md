# Strategic Dreaming Reference

Post-cycle synthesis pass that asks "what did we learn about product direction?" after a KBD execution phase completes. Distinct from PMPO Reflect (execution quality) and KBD Reflect (goal alignment).

---

## Purpose

Strategic dreaming extracts product-direction lessons from a completed execution cycle. It looks for patterns in what was built, what was deferred, and what the feedback signals said — then generates enduring `evolver_lessons[]` entries that inform future evolution perspectives.

**Not the same as PMPO Reflect:**
- PMPO Reflect asks: "Did we execute well? Where did we fall short?"
- Strategic dreaming asks: "What does this cycle reveal about where the product should go next?"

**Not the same as KBD Reflect:**
- KBD Reflect measures goal achievement vs. plan
- Strategic dreaming identifies emergent product direction signals

---

## When to run

Run `post-cycle-dream.sh` after:
1. `kbd-reflect` has completed and `reflection.md` is written
2. At least one tick of learning signals has been collected
3. The evolver's `state.json` has `execution_results` from the cycle

Do NOT run inline in the evolver session — always as an isolated subprocess.

---

## Output format

The dreaming pass writes to two places:

**1. `evolver-lessons.md`** (append-only, per evolution)

Path: `.evolver/<name>/evolver-lessons.md`

```markdown
## Lesson: <short title>

**Cycle:** <kbd-phase-name>
**Perspective:** <competitive|trend|unique-product|idea-validation|self-learning>
**Confidence:** high | medium | low
**Signal sources:** [carry-forward, learning-signals, reflection, changelog]

<2-3 sentences explaining the product direction insight>

**Actionable implication:** <one sentence: what to do differently in the next cycle>
```

**2. `state.json`** (append to `evolver_lessons[]`)

```json
{
  "id": "lesson-<timestamp>",
  "title": "short title",
  "cycle": "kbd-phase-name",
  "perspective": "trend",
  "confidence": "medium",
  "signal_sources": ["carry-forward", "learning-signals"],
  "body": "2-3 sentence product direction insight",
  "implication": "one sentence actionable implication",
  "created_at": "ISO8601"
}
```

---

## Dreaming prompt

The prompt given to liter-llm `complete --model frontier` for the dreaming pass:

```
You are a strategic product analyst. Given the following artifacts from a completed evolution cycle, identify 2-3 enduring lessons about where this product should evolve next.

## Completed cycle: <phase-name>
<excerpt from journal.md or reflection.md — last 2000 tokens>

## Recent learning signals
<top 5 signals from learning-signals-<timestamp>.json — signal + severity>

## Existing evolver lessons (do not duplicate)
<titles from existing evolver_lessons[] in state.json>

Produce exactly 2-3 lessons. Each lesson must:
- Be about product DIRECTION, not execution quality
- Be falsifiable (could be proven wrong by evidence)
- Include an actionable implication for the next cycle
- NOT duplicate any existing lesson

Output JSON:
{
  "lessons": [
    {
      "title": "string (5-8 words)",
      "perspective": "competitive|trend|unique-product|idea-validation|self-learning",
      "confidence": "high|medium|low",
      "signal_sources": ["carry-forward|learning-signals|reflection|changelog"],
      "body": "string (2-3 sentences)",
      "implication": "string (1 sentence)"
    }
  ]
}
```

---

## Context management

Run as an isolated subprocess to protect the evolver session context:

```bash
LESSONS=$(bash scripts/post-cycle-dream.sh "${EVOLUTION_NAME}" "${PHASE_NAME}")
NEW_COUNT=$(echo "${LESSONS}" | python3 -c "import json,sys; print(json.load(sys.stdin)['lessons_added'])")
echo "[evolver] Strategic dreaming complete: ${NEW_COUNT} new lessons added"
```

**Why isolated:** The dreaming pass ingests journal.md + reflection.md + existing lessons, which can sum to thousands of tokens. Isolating it prevents the main evolver session from hitting context limits.

---

## Deduplication

Before writing a new lesson, check `evolver_lessons[]` in `state.json` by title similarity. Skip if:
- Title overlap > 60% (word-level Jaccard)
- Body references the same specific issue

The dreaming prompt explicitly lists existing lesson titles to reduce duplication at generation time.

---

## Integration with perspectives

Each dreaming run tags lessons with the dominant perspective:

- `competitive` — lesson is about matching or exceeding competitors
- `trend` — lesson is about anticipating domain developments
- `unique-product` — lesson is about what makes this product distinctive
- `idea-validation` — lesson is about which ideas to pursue (or not)
- `self-learning` — lesson is about how the product's own signals guide evolution

The next evolver cycle's `perspective_cursor` can query `evolver_lessons[]` filtered by perspective when entering a specific evolution perspective.
