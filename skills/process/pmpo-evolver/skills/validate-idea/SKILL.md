---
name: validate-idea
description: Staged idea validation for product evolution — three gates from plausibility to specification using the Darwin Gödel Machine pattern, with Archive of Stepping Stones persistence
version: '1.0.0'
license: MIT
metadata:
  author: prometheus-ags
  category: process
  tags: [idea-validation, evolution, darwin, staged-gating, liter-llm, pmpo-evolver]
---

# validate-idea

Staged idea validation sub-skill for the `idea-validation` evolution perspective. Takes an operator-supplied idea through three gates before committing to a KBD phase.

## Progress Signals (MANDATORY)

Before any other action, emit:

```
Starting validate-idea — <idea title>
```

When the three-gate pipeline finishes, emit:

```
Completed validate-idea — <idea title> (verdict: <pass|fail>, gate: <1|2|3>)
```

Emit to plain response text — no tool call needed.

## Entry

```
/validate-idea "<idea text>" [--evolution-name <name>] [--auto-gate]
```

Or invoked by `/pmpo-evolver --perspective idea-validation --idea "<text>"`.

**`--auto-gate`**: Skip manual approval at Gate 3 if feasibility_score > 60. Use only in autonomous loop contexts.

---

## Three-gate pipeline (Darwin Gödel Machine pattern)

### Gate 1 — Plausibility (~30s)

**Model:** `[MODEL_ROUTING] phase=evolver-idea-gate1 class=small`

Fast check using bash + file search. No web access needed.

**Checks:**
1. Is this already implemented? (scan `skills/` directories for matching keyword)
2. Is this already in the backlog? (check `.evolver/<name>/backlog.json`)
3. Does this conflict with `design-philosophy.md` (if present)? (liter-llm binary classification)

**Run:** `bash scripts/idea-gate-1.sh "<idea>" "<evolution-name>"`

**Exit codes:** 0 = PASS, 1 = REJECT

**On REJECT:** Write archive entry with `revisit_weight: 0.1`. Stop pipeline.

---

### Gate 2 — Domain research (~5min)

**Model:** `[MODEL_ROUTING] phase=evolver-idea-gate2 class=medium`

Web research to assess feasibility and prior art.

**Research tasks:**
1. Prior art: "has this been done before? by whom? how? under what license?"
2. Feasibility: required packages, APIs, or services — do they exist and are they accessible?
3. Competitive check: read `parity-matrix.json` (if present) — do competitors already have this?

**Output:**
```json
{
  "feasibility_score": 75,
  "prior_art": ["github.com/example/similar-project (MIT)"],
  "missing_deps": [],
  "competitive_status": "ahead | parity | behind",
  "recommendation": "PROCEED | DEPRIORITIZE | PIVOT",
  "research_summary": "string"
}
```

**Scoring is delegated — the researcher does not grade its own research.**

Gate 2 produces `research_summary`, `prior_art`, and `missing_deps`. It does
**not** assign the score that routes the idea. Hand the research to
`agents/kbd-idea-critic.md`, which scores it on a separate dispatch:

```
Task(subagent_type="kbd-idea-critic", prompt=<the Gate 2 research output>)
```

This is the same producer≠judge rule the rest of this pack enforces — the agent
that gathered evidence for an idea is the worst-placed one to judge whether the
evidence is sufficient. `kbd-idea-critic` states it directly: *"the idea that
proposed the idea should never also grade it."*

**Routing** (on the critic's aggregate, not Gate 2's self-assessment):
- `aggregate < 3.0` → REJECT (archive with `revisit_weight: 0.3`)
- `aggregate 3.0–7.0` → PROCEED but require human gate at step 3
- `aggregate > 7.0` → PROCEED; auto-approve Gate 3 if `--auto-gate`

> If the critic is unavailable, **do not fall back to self-scoring** — record
> `scored_by: "SELF — UNVERIFIED"` in the archive entry and require the human
> gate regardless of score. A self-assigned score that routes an idea is worse
> than no score, because it carries the appearance of review.

---

### Gate 3 — Spec + human gate

**Model:** `[MODEL_ROUTING] phase=evolver-idea-spec class=frontier`

Generate a complete spec, verify verifiability of acceptance criteria, then present for human approval.

**Spec sections (SPEC.md):**

```markdown
# Spec: <idea title>

## Problem statement
<what problem this solves>

## Proposed solution
<what will be built>

## Acceptance criteria
- AC1: <machine-checkable criterion>
- AC2: <machine-checkable criterion>
...

## Non-goals
<what this explicitly does NOT do>

## Dependencies
<packages, APIs, services required>

## Estimated effort
xs | s | m | l | xl
```

**Verifiability check:** For each acceptance criterion, classify as:
- `machine-checkable`: can be verified by a script, test, or automated check
- `human-judgeable`: requires subjective assessment

If any criterion is `human-judgeable` → reformulate with pmpo-elicit: "Rewrite '<criterion>' as a machine-checkable test."

**Human gate:** Present spec + Gate 2 findings via pmpo-elicit.

**Responses:**
- `APPROVE` → proceed to KBD phase seeding (calls `evolver-seed-phase.sh`)
- `REVISE <feedback>` → return to spec generation with operator feedback, max 3 iterations
- `REJECT` → archive with `revisit_weight: 0.0`

**On APPROVE:** `bash scripts/evolver-seed-phase.sh <evolution-name> <idea-id>`

---

## Archive of Stepping Stones

Every idea, regardless of gate outcome, is written to `.evolver/<name>/archive/<idea-id>/manifest.json`:

```json
{
  "id": "idea-<timestamp>",
  "text": "the original idea text",
  "submitted_at": "ISO8601",
  "gate_reached": 1,
  "outcome": "PASS | REJECT",
  "reject_reason": "string (if rejected)",
  "lessons": ["what we learned even from a rejected idea"],
  "revisit_weight": 0.1,
  "gate1_result": {"passed": false, "reason": "..."},
  "gate2_result": {"feasibility_score": 25, "recommendation": "PIVOT"},
  "gate3_spec_path": ".evolver/<name>/archive/<idea-id>/SPEC.md"
}
```

**Revisit weights:**
- `1.0` — approved and executed in a KBD phase
- `0.5` — approved but not yet executed (backlog)
- `0.3` — rejected at Gate 2 (feasibility concern; worth revisiting when deps mature)
- `0.1` — rejected at Gate 1 (duplicate or conflict; low priority)
- `0.0` — hard reject at Gate 3 (human decision not to pursue)

**How revisit_weight is used:** The next evolver cycle's Assess phase reads the archive and surfaces high-weight unexecuted items as candidates for the current cycle's backlog.

---

## Platform compatibility

Works across Claude Code, Codex, OpenCode, Kimi, and Zed. Uses only bash + python3 for Gate 1; liter-llm for Gate 2 and Gate 3 (with graceful fallback to host model).

---

## Examples

```
/validate-idea "Add a visual diff viewer for SKILL.md changes between versions"
/validate-idea "Port the outer loop to work with Cursor's agent API" --evolution-name prometheus-skill-pack
/validate-idea "Add voice interface to pmpo-elicit" --auto-gate
```
