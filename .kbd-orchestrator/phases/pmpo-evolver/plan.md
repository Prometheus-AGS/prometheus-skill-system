# Plan — pmpo-evolver

**Phase:** pmpo-evolver
**Planned:** 2026-06-28
**Backend:** native-kbd (`.kbd-orchestrator/changes/change-evolver-NNN/`) — migrated from OpenSpec 2026-07-02
**Changes:** 10
**Operator addendum:** All phases must incorporate liter-llm model routing — choosing the most capable and cost-efficient model for each task using the liter-llm-bridge MCP tools and the system's configured providers.

---

## Summary

The `pmpo-evolver` phase builds a strategy routing layer above the existing `iterative-evolver` / `kbd-evolve` / `pmpo-outer-loop` stack. The goal is a well-defined process for evolving released projects across five perspectives: competitive analysis, domain trend research, unique-product next-step research, operator idea validation, and Karpathy self-learning from usage/feedback/history.

Research validated eight design patterns (Kitchen Loop, Darwin Gödel Machine, Anthropic Dreaming, Metacognitive Learning) and identified five competitive whitespace gaps none of the current harnesses fill.

The model routing operator addendum is woven throughout: every `pmpo-evolver` phase directive carries a `[MODEL_ROUTING]` signal. Strategy phases (assess, analyze, plan) route to `frontier`. Triage, changelog ingestion, and feedback collection route to `medium`. Status, file writes, and changelog-diff scripts route to `small`. liter-llm resolves these to the cheapest provider/model combination that meets the class requirement.

**Ordering rationale:**
1. Schema first (change-evolver-001) — all wiring changes reference the new fields.
2. Feedback source taxonomy extension (change-evolver-002) — Karpathy perspective is the most novel; loop-definition schema must exist before SKILL.md references it.
3. SKILL.md + model routing table (change-evolver-003) — the entry command; references both schema and feedback sources.
4. Competitor tracking + parity matrix (change-evolver-004) — highest first-mover whitespace value.
5. Learning signals persistence + commit-history analysis (change-evolver-005) — completes the Karpathy perspective.
6. Staged idea validation sub-skill (change-evolver-006) — Darwin Gödel Machine staged gating; depends on schema + SKILL.md.
7. Carry-forward aggregation + domain taxonomy (change-evolver-007) — unique-product and trend perspectives; references prior reflection structure.
8. Strategic dreaming (post-cycle dream) (change-evolver-008) — Anthropic Dreaming pattern; depends on learning signals (005).
9. Outer-loop perspective handoff (change-evolver-009) — wires pmpo-outer-loop `loop.json` perspective field to the evolver; depends on SKILL.md + schema.
10. Phase-seeding protocol + liter-llm context management reference (change-evolver-010) — inner-loop bridge; depends on all perspective changes being defined.

---

## Change List

### change-evolver-001 — `pmpo-evolver.schema.json` + evolution-state schema extensions

**Gaps addressed:** G-02, G-12
**Goals:** G2, G5
**Priority:** HIGH — foundation; all other changes reference these fields
**Agent:** claude-code
**Effort:** M

**Files:**
- `skills/process/pmpo-evolver/references/schemas/pmpo-evolver.schema.json` (NEW)
- `skills/process/iterative-evolver/references/schemas/evolution-state.schema.json` (MODIFY — additive only)

**What it does:**

New `pmpo-evolver.schema.json` defines the strategy-router state:

```json
{
  "evolution_name": "string",
  "perspective": "competitive | trend | unique-product | idea-validation | self-learning | combined",
  "perspective_cursor": {
    "current": "string",
    "completed": ["string"],
    "pending": ["string"]
  },
  "competitor_tracking": {
    "registry_path": ".evolver/{name}/competitor-registry.json",
    "last_scanned": "ISO8601",
    "parity_matrix_path": ".evolver/{name}/parity-matrix.json"
  },
  "learning_signals": [{
    "source_type": "gh-issues | commit-history | sentiment-feed | usage-trace | telemetry-url | research-query",
    "collected_at": "ISO8601",
    "signal": "string",
    "severity": "high | medium | low",
    "count": "integer",
    "examples": ["string"]
  }],
  "idea_origin": {
    "type": "competitive | trend | operator | self-learning | continuation",
    "rationale": "string",
    "first_seen": "ISO8601"
  },
  "evolver_lessons": [{
    "lesson": "string",
    "origin_cycle": "integer",
    "confidence": "high | medium | low",
    "category": "direction | threat | opportunity | falsified-hypothesis"
  }],
  "model_routing": {
    "policy": "liter-llm | harness-native | frontier-all",
    "class_map": {}
  }
}
```

Evolution-state additions (additive, backward-compatible):
- `learning_signals: []` — array of normalized feedback digest entries
- `perspective: string` — which of the 5 perspectives drove this cycle

**Tasks:**
- [ ] 1. Create `skills/process/pmpo-evolver/references/schemas/pmpo-evolver.schema.json`
- [ ] 2. Add `learning_signals[]` and `perspective` fields to `evolution-state.schema.json`
- [ ] 3. Validate both files parse as valid JSON

---

### change-evolver-002 — Feedback source taxonomy extension (loop-definition schema)

**Gaps addressed:** G-09
**Goals:** G5
**Priority:** HIGH — Karpathy perspective's most novel gap; enables self-learning loops
**Agent:** claude-code
**Effort:** M

**Files:**
- `skills/process/pmpo-outer-loop/references/schemas/loop-definition.schema.json` (MODIFY — additive)
- `skills/process/pmpo-evolver/references/feedback-sources.md` (NEW)

**What it does:**

Extends `feedback_sources[].type` enum with semantic source types:

| New type | Fields | Interpret semantics |
|----------|--------|---------------------|
| `gh-issues` | `repo`, `labels[]`, `state`, `since` | count-delta of open issues matching labels; LLM sentiment classification of titles |
| `commit-history` | `repo_path`, `since`, `classify_by` | git log → LLM classifies each commit as fix/feat/refactor/chore; outputs JSON histogram |
| `mcp-tool` | `tool_name`, `arguments`, `jsonpath` | calls an MCP tool and extracts a value; interpret via jsonpath |
| `sentiment-feed` | `url`, `format` (rss/json/csv), `sentiment_field` | fetch URL, parse items, run sentiment classification |
| `telemetry-url` | `url`, `headers`, `jsonpath`, `direction` | fetch JSON API, extract value via jsonpath, interpret as count or float |
| `competitor-scan` | `competitor_ids[]`, `registry_path` | reads competitor-registry.json, runs web search for each, diffs against last scan |
| `changelog` | `repo`, `since_tag`, `format` (github-releases/file) | fetches GitHub releases or CHANGELOG.md since last tick; LLM extracts feature additions |

Also adds `staleness_ttl_minutes` field to all source types — prevents re-fetching on every tick.

`feedback-sources.md` documents:
- Each source type with a concrete example invocation
- How to write an `interpret` value for each type
- Staleness TTL guidance (competitor-scan: 1440min; commit-history: 60min; gh-issues: 30min)
- How the tick normalizes all source outputs into `{signal, severity, count, examples[]}` format
- **Model routing**: sentiment classification within a feedback collection step → `medium` class (via liter-llm `complete` with `model=medium`); gh-issues count-delta → `small`

**Tasks:**
- [ ] 1. Add new type entries to `loop-definition.schema.json` feedback_sources items oneOf
- [ ] 2. Add `staleness_ttl_minutes` field to base feedback_source object
- [ ] 3. Create `skills/process/pmpo-evolver/references/feedback-sources.md`
- [ ] 4. Schema validates clean with python3 json.tool

---

### change-evolver-003 — `skills/process/pmpo-evolver/SKILL.md` + model routing table

**Gaps addressed:** G-01, G-13
**Goals:** G1, G3
**Priority:** HIGH — the entry command; nothing invocable without this
**Agent:** claude-code
**Effort:** L

**Files:**
- `skills/process/pmpo-evolver/SKILL.md` (NEW)
- `skills/process/pmpo-evolver/references/model-routing.md` (NEW)

**What it does:**

`SKILL.md` defines `/pmpo-evolver` — the strategy router entry command. Sections:

**Entry commands:**
```
/pmpo-evolver <evolution-name> [--perspective <mode>] [--depth quick|standard|deep]
/pmpo-evolver-status <evolution-name>
```

**Strategy routing logic:**
1. Load the product's `design-philosophy.md` (if present) and most recent `reflection.md`
2. Read `competitor-registry.json` (if present) — determines if competitive scan data is fresh
3. Emit `[MODEL_ROUTING] phase=evolver-route class=small` — routing selection is cheap
4. Route to the correct perspective mode based on `--perspective` or auto-detection:

| Mode | Auto-detect signal | What it orchestrates |
|------|--------------------|---------------------|
| `competitive` | competitor-registry.json exists AND last_scanned > staleness_ttl | competitor-scan feedback sources + parity matrix update + iterative-evolver with competitive analysis domain |
| `trend` | No competitor registry OR --perspective trend | domain-taxonomy lookup + standards-body research + iterative-evolver analyze phase |
| `unique-product` | No competitor registry AND design-philosophy.md present | carry-forward aggregation + iterative-evolver with strategic criteria |
| `idea-validation` | Operator provided idea text (--idea "...") | validate-idea sub-skill (staged gating) |
| `self-learning` | loop.json feedback_sources contain gh-issues/commit-history/sentiment | feedback digest + learning_signals update + iterative-evolver assess phase |
| `combined` | --perspective combined OR no clear single signal | sequential routing through relevant perspectives with perspective_cursor tracking |

**PMPO loop per-perspective:**
For each mode: Assess → (Analyze where needed) → Plan → Execute (via iterative-evolver + KBD) → Reflect → Strategic Dream → Persist

**Model routing per phase:**
Emits `[MODEL_ROUTING]` directives using liter-llm class map:

| Phase | Class | Rationale |
|-------|-------|-----------|
| Perspective routing selection | small | File reads + schema check |
| Competitive landscape scan | frontier | Cross-domain synthesis, novelty detection |
| Parity matrix generation | medium | Structured comparison, bounded output |
| Trend research synthesis | frontier | Ambiguous external signals |
| Carry-forward aggregation | small | File reads + pattern extraction |
| Idea plausibility gate (Gate 1) | small | Binary yes/no classification |
| Idea domain research (Gate 2) | medium | Web search + bounded synthesis |
| Idea spec generation (Gate 3) | frontier | Novel spec drafting under constraints |
| Feedback source collection | small | Deterministic tool calls |
| Feedback sentiment classification | medium | NLP classification |
| Learning signal synthesis | medium | Pattern extraction across signals |
| Strategic dreaming | frontier | Open-ended strategic synthesis |
| Evolver reflect | frontier | Quality judgment + delta analysis |

All `[MODEL_ROUTING]` directives route through liter-llm when available (see `references/model-routing.md`). When liter-llm is not installed or a class has no configured provider, fall through to the host model with a warning.

`model-routing.md` defines:
- The liter-llm MCP tool invocation contract for each class
- How to detect liter-llm availability (`liter-llm --version` or checking MCP server registration)
- Provider discovery: how to read `~/.config/liter-llm/config.toml` or `LITER_LLM_CONFIG` env var to know which providers/models are available
- Context window management: when to use `small`/`medium` to conserve context budget; recommendation to run feedback collection and changelog ingestion as isolated agent invocations rather than inline to protect the main context window
- Fallback chain: liter-llm `complete(model=<class>)` → harness-native model override → host model

**Tasks:**
- [ ] 1. Create `skills/process/pmpo-evolver/SKILL.md` with all sections above
- [ ] 2. Create `skills/process/pmpo-evolver/references/model-routing.md`
- [ ] 3. `npm run validate:strict skills/process/pmpo-evolver` passes clean
- [ ] 4. File is under 500 lines; verbose sections go to `references/`

---

### change-evolver-004 — Competitor tracking: registry + parity matrix + changelog ingestion

**Gaps addressed:** G-03, G-04, G-05
**Goals:** G1
**Priority:** HIGH — whitespace #1 in competitive landscape; highest first-mover value
**Agent:** claude-code
**Effort:** M

**Files:**
- `skills/process/pmpo-evolver/references/competitive-analysis.md` (NEW)
- `skills/process/pmpo-evolver/scripts/competitor-registry-init.sh` (NEW, executable)
- `skills/process/pmpo-evolver/scripts/changelog-fetch.sh` (NEW, executable)

**What it does:**

`competitive-analysis.md` defines the competitor tracking protocol:

**Competitor registry** (`.evolver/<name>/competitor-registry.json`):
```json
{
  "competitors": [{
    "id": "string",
    "name": "string",
    "url": "string",
    "github_repo": "string (optional)",
    "last_scanned": "ISO8601",
    "last_changelog_tag": "string",
    "feature_claims": ["string"]
  }]
}
```

**Parity matrix** (`.evolver/<name>/parity-matrix.json`):
```json
{
  "features": [{
    "id": "string",
    "name": "string",
    "our_status": "has | missing | partial | better",
    "competitors": {"<competitor-id>": "has | missing | partial | better"},
    "priority": "high | medium | low",
    "last_updated": "ISO8601"
  }]
}
```

**Changelog ingestion** (`changelog-fetch.sh <repo> [--since-tag <tag>]`):
- Uses `gh api repos/<owner>/<repo>/releases` to fetch releases since last tag
- Falls back to fetching `CHANGELOG.md` raw content via URL if no releases
- Passes release notes to liter-llm `complete(model=medium)` to extract: features added, breaking changes, deprecations
- Outputs structured JSON: `{repo, from_tag, to_tag, features_added[], breaking_changes[]}`
- Stores result in `.evolver/<name>/changelogs/<competitor-id>-<timestamp>.json`

**Model routing**: changelog feature extraction → `medium`; parity matrix update (comparing extracted features to our product spec) → `frontier` (requires judgment about equivalence)

`competitor-registry-init.sh <evolution-name>` — interactive (via pmpo-elicit) initialization of the registry: prompts operator for competitor names, repos, and initial feature claims.

**Tasks:**
- [ ] 1. Create `skills/process/pmpo-evolver/references/competitive-analysis.md`
- [ ] 2. Create `scripts/competitor-registry-init.sh` (executable)
- [ ] 3. Create `scripts/changelog-fetch.sh` (executable)
- [ ] 4. Smoke test: `bash changelog-fetch.sh GQAdonis/liter-llm` exits 0 and produces JSON

---

### change-evolver-005 — Learning signals persistence + commit-history analysis

**Gaps addressed:** G-10, G-11
**Goals:** G5
**Priority:** HIGH — most novel Karpathy perspective capability; no existing system has it
**Agent:** claude-code
**Effort:** M

**Files:**
- `skills/process/pmpo-evolver/references/learning-signals.md` (NEW)
- `skills/process/pmpo-evolver/scripts/commit-history-analyze.sh` (NEW, executable)
- `skills/process/pmpo-evolver/scripts/feedback-digest.sh` (NEW, executable)

**What it does:**

`learning-signals.md` defines the full protocol for collecting, normalizing, and persisting signals from each source type:

**Signal collection protocol per source type:**
- `gh-issues`: `gh api repos/<owner>/<repo>/issues` with label/state filters → count open; run liter-llm `complete(model=medium)` on titles to classify sentiment and themes
- `commit-history`: run `commit-history-analyze.sh` → get JSON histogram
- `sentiment-feed`: fetch URL → parse items → liter-llm `complete(model=medium)` classifies each → aggregate counts
- `telemetry-url`: fetch JSON → extract value via jsonpath → compare to baseline stored in evolution state
- `changelog`: run `changelog-fetch.sh` → extract feature additions → compare to our parity matrix

**Normalization format** (all sources → common `LearningSignal`):
```json
{
  "id": "uuid",
  "source_type": "gh-issues | commit-history | ...",
  "source_ref": "string (repo, URL, file path)",
  "collected_at": "ISO8601",
  "signal": "string (human-readable summary)",
  "severity": "high | medium | low",
  "count": "integer (occurrences)",
  "examples": ["string (up to 5 examples)"],
  "model_used": "string (liter-llm class:model that classified this)"
}
```

**Persistence**: signals are appended to `evolution_state.learning_signals[]` and also written to `.evolver/<name>/learning-signals-<tick>.json` for per-tick archival.

`commit-history-analyze.sh <repo_path> [--since <ISO8601>]`:
- Runs `git log --oneline --since=<date> <repo_path>`
- Passes output to liter-llm `complete(model=small)` with classification prompt: categorize each commit as `fix|feat|refactor|chore|docs|test`
- Counts hotspots (files changed most frequently in fix commits = churn debt signals)
- Outputs JSON: `{period, total_commits, breakdown: {fix, feat, refactor, ...}, hotspots: [{file, fix_count}]}`

`feedback-digest.sh <evolution-name>`:
- Reads loop.json `feedback_sources[]` for the named loop
- Collects each source's current data
- Normalizes to `LearningSignal[]`
- Appends to evolution state
- Uses liter-llm for classification; model class per signal type (from `learning-signals.md`)
- Outputs a brief JSON digest: `{collected, high_severity_count, new_signals[]}`

**Model routing**: commit classification → `small`; sentiment classification → `medium`; signal synthesis (what do these signals collectively mean for product direction?) → `frontier`

**Tasks:**
- [ ] 1. Create `skills/process/pmpo-evolver/references/learning-signals.md`
- [ ] 2. Create `scripts/commit-history-analyze.sh` (executable)
- [ ] 3. Create `scripts/feedback-digest.sh` (executable)
- [ ] 4. Smoke test: `bash commit-history-analyze.sh . --since 2026-06-01` exits 0, outputs valid JSON

---

### change-evolver-006 — Staged idea validation sub-skill (validate-idea)

**Gaps addressed:** G-08
**Goals:** G1, G4
**Priority:** HIGH — operator idea-validation perspective; three-gate Darwin pattern
**Agent:** claude-code
**Effort:** M

**Files:**
- `skills/process/pmpo-evolver/skills/validate-idea/SKILL.md` (NEW)
- `skills/process/pmpo-evolver/scripts/idea-gate-1.sh` (NEW, executable)

**What it does:**

`validate-idea/SKILL.md` implements the full idea-intake → research → feasibility → spec → gate pipeline using the Darwin Gödel Machine's staged evaluation pattern:

**Entry:**
```
/validate-idea "<idea text>" [--evolution-name <name>] [--auto-gate]
```
Or called by `/pmpo-evolver --perspective idea-validation --idea "<text>"`.

**Three-gate pipeline:**

**Gate 1 — Plausibility (~30s, model=small):**
- Check: Does this idea align with the product's `design-philosophy.md` (if present)?
- Check: Is this already implemented? (scan existing KBD phases + reflection carry-forwards)
- Check: Is this already in the backlog? (`.evolver/<name>/backlog.json`)
- Output: PASS / REJECT with reason
- Model: liter-llm `complete(model=small)` — binary classification is cheap
- If REJECT: write to `.evolver/<name>/archive/<idea-id>/manifest.json` with `revisit_weight: 0.1`

**Gate 2 — Domain research (~5min, model=medium):**
- Web search for prior art: "has this been done before? by whom? how?"
- Feasibility check: required dependencies (packages, APIs, hardware) — do they exist?
- Competitive scan: do any tracked competitors already have this? (reads parity matrix)
- Output: `{feasibility_score: 0-100, prior_art: [], missing_deps: [], competitive_status: "ahead|parity|behind"}`
- Model: liter-llm `complete(model=medium)` — bounded web research synthesis

**Gate 3 — Spec + human gate (~full, model=frontier):**
- Generate a `SPEC.md` draft using the KBD spec template
- Calculate verifiable acceptance criteria (per Karpathy/Kitchen Loop verifiability constraint)
- If any criterion is not machine-checkable → loop back and reformulate with pmpo-elicit
- Human gate (via pmpo-elicit): present the spec + feasibility summary → APPROVE / REVISE / REJECT
- On APPROVE: create new KBD phase goals.md seeded from spec acceptance criteria → the phase-seeding protocol (change-evolver-010)

**Archive of Stepping Stones**: every idea — regardless of gate outcome — is written to `.evolver/<name>/archive/<idea-id>/manifest.json` with `outcome`, `gate_reached`, `lessons`, and `revisit_weight` (1.0 for pass, 0.3 for Gate 2 reject, 0.1 for Gate 1 reject).

**Model routing:** Gate 1 → `small`; Gate 2 → `medium`; Gate 3 spec generation → `frontier`; human gate elicitation → platform-native (pmpo-elicit handles routing)

`idea-gate-1.sh <idea-text> <evolution-name>` — fast bash implementation of Gate 1 checks (file searches for existing implementations + backlog check). Exits 0=PASS, 1=REJECT.

**Tasks:**
- [ ] 1. Create `skills/process/pmpo-evolver/skills/validate-idea/SKILL.md`
- [ ] 2. Create `scripts/idea-gate-1.sh` (executable)
- [ ] 3. `npm run validate:strict skills/process/pmpo-evolver/skills/validate-idea` passes

---

### change-evolver-007 — Carry-forward aggregation + domain taxonomy reference

**Gaps addressed:** G-06, G-07
**Goals:** G1
**Priority:** MEDIUM — unique-product and trend perspectives
**Agent:** claude-code
**Effort:** S

**Files:**
- `skills/process/pmpo-evolver/references/domain-taxonomy.md` (NEW)
- `skills/process/pmpo-evolver/scripts/carry-forward-aggregate.sh` (NEW, executable)

**What it does:**

`domain-taxonomy.md` maps project domain keywords to:
- Relevant **standards bodies** to watch (IETF for protocols, W3C for web, NIST for security, ISO for compliance, etc.)
- Relevant **community sources** (GitHub awesome-lists by domain, HN Ask threads, community blogs, newsletters)
- **Polling frequency** recommendation per source type (standards bodies: monthly; community sources: weekly; changelogs: per-tick)
- **Search query templates** for each domain category

Organized by domain keyword clusters matching the iterative-evolver's domain adapter list (software, business, product, research, content, operations, compliance) plus the skill-pack-specific domains (agent orchestration, LLM tooling, Rust systems, etc.).

`carry-forward-aggregate.sh <phase-dir-root>`:
- Walks all `.kbd-orchestrator/phases/*/reflection.md` files
- Extracts each `## Carry-Forwards` section
- Deduplicates by semantic similarity (simple grep-based dedup for now; LLM dedup as optional `--deep` flag)
- Outputs JSON: `{total_phases, carry_forwards: [{phase, items: [string], dates: [string]}], deduplicated: [string]}`
- Stores in `.evolver/<name>/carry-forwards.json`

**Model routing**: carry-forward collection → `small` (file reads + grep); deduplication with `--deep` → `medium`; trend synthesis against domain-taxonomy + carry-forwards → `frontier`

**Tasks:**
- [ ] 1. Create `skills/process/pmpo-evolver/references/domain-taxonomy.md`
- [ ] 2. Create `scripts/carry-forward-aggregate.sh` (executable)
- [ ] 3. Smoke test: script runs against current repo, exits 0, outputs valid JSON

---

### change-evolver-008 — Strategic dreaming (post-cycle-dream)

**Gaps addressed:** G-10 (strategic memory layer)
**Goals:** G5
**Priority:** MEDIUM — Anthropic Dreaming pattern adapted for product-direction; distinct from PMPO Reflect
**Agent:** claude-code
**Effort:** S

**Files:**
- `skills/process/pmpo-evolver/references/strategic-dreaming.md` (NEW)
- `skills/process/pmpo-evolver/scripts/post-cycle-dream.sh` (NEW, executable)

**What it does:**

`strategic-dreaming.md` defines the strategic dreaming protocol — the post-cycle consolidation step that runs after each `iterative-evolver` cycle completes, BEFORE the outer loop decides whether to terminate or continue:

**What strategic dreaming is NOT:**
- Not the PMPO Reflect phase (execution quality: did the plan succeed? why did changes fail?)
- Not the evolver's Reflect (goal alignment: are we closer to the target state?)

**What strategic dreaming IS:**
- A lightweight pass over the full cycle journal asking: "what did we learn about *product direction* that we didn't know before?"
- Patterns it looks for: hypotheses that were falsified ("we thought users wanted X; the feedback says Y"), threats that escalated, opportunities that emerged, trends that accelerated faster than expected

**Output format** — `evolver-lessons.md` entries:
```markdown
### <timestamp> — Cycle <N> Strategic Lesson

Category: direction | threat | opportunity | falsified-hypothesis
Confidence: high | medium | low
Lesson: <1-2 sentences: what we learned about product direction>
Evidence: <what in the cycle journal supports this>
Impact on next cycle: <how this should change what we do next>
```

Lessons are also persisted to `evolution_state.evolver_lessons[]` (from schema in change-evolver-001).

`post-cycle-dream.sh <evolution-name>`:
- Reads `journal.md` latest entry + `reflection.md` from the just-completed KBD phase
- Reads `evolution_state.evolver_lessons[]` to avoid re-deriving already-known lessons
- Passes to liter-llm `complete(model=frontier)` with the dreaming prompt
- Appends result to `evolver-lessons.md`
- Updates `evolution_state.evolver_lessons[]`

**Model routing**: strategic dreaming → `frontier` (open-ended synthesis; this is where the strategic intelligence lives)

**Context management note**: post-cycle-dream is run as an isolated invocation (not inline in the evolver session) to avoid consuming the main context window with journal content. The script pipes only the relevant sections.

**Tasks:**
- [ ] 1. Create `skills/process/pmpo-evolver/references/strategic-dreaming.md`
- [ ] 2. Create `scripts/post-cycle-dream.sh` (executable)
- [ ] 3. Document the invocation point in `SKILL.md` (cross-reference only — no SKILL.md rewrite needed since change-003 already includes it)

---

### change-evolver-009 — Outer-loop perspective handoff (pmpo-outer-loop wiring)

**Gaps addressed:** G-13
**Goals:** G3
**Priority:** MEDIUM — wires outer loop to perspective-aware evolver calls
**Agent:** claude-code
**Effort:** S

**Files:**
- `skills/process/pmpo-outer-loop/references/schemas/loop-definition.schema.json` (MODIFY — additive)
- `skills/process/pmpo-outer-loop/SKILL.md` (MODIFY — add perspective routing note)
- `skills/process/pmpo-outer-loop/scripts/loop-tick.sh` (MODIFY — pass perspective to evolve call)

**What it does:**

Adds `perspective` field to `loop.json` schema:
```json
"perspective": {
  "type": "string",
  "enum": ["competitive", "trend", "unique-product", "idea-validation", "self-learning", "combined", "auto"],
  "default": "auto",
  "description": "Which pmpo-evolver perspective to apply on each tick. 'auto' lets the evolver router decide based on data freshness and feedback signals."
}
```

Modifies `loop-tick.sh`: when running `/evolve <name>`, check if `loop.json.perspective != "auto"` — if so, pass `--perspective <value>` to the evolver invocation. When `auto`, let the evolver router choose.

Adds a note in `pmpo-outer-loop/SKILL.md` under `/loop-define`: "When defining a product-evolution loop, set `perspective` to one of the pmpo-evolver modes or `auto` to let the router choose based on data freshness and feedback signals."

**Tasks:**
- [ ] 1. Add `perspective` field to loop-definition schema
- [ ] 2. Modify `scripts/loop-tick.sh` to pass perspective flag to evolve call
- [ ] 3. Add cross-reference paragraph to pmpo-outer-loop SKILL.md
- [ ] 4. `npm run validate:strict skills/process/pmpo-outer-loop` passes

---

### change-evolver-010 — Phase-seeding protocol + liter-llm context management reference

**Gaps addressed:** G-14
**Goals:** G4, G1 (addendum: liter-llm context management)
**Priority:** MEDIUM — completes inner-loop bridge; also delivers liter-llm context management guidance
**Agent:** claude-code
**Effort:** M

**Files:**
- `skills/process/pmpo-evolver/scripts/evolver-seed-phase.sh` (NEW, executable)
- `skills/process/pmpo-evolver/references/context-management.md` (NEW)
- `skills/process/liter-llm-bridge/references/model-discovery.md` (NEW)

**What it does:**

`evolver-seed-phase.sh <evolution-name> <plan-item-id>`:
- Reads `.evolver/<evolution-name>/state.json` → finds the plan item by ID
- Extracts: `description`, `success_criteria[]`, `target_state`
- Creates `.kbd-orchestrator/phases/<phase-name>/goals.md` from these fields
- Creates `.kbd-orchestrator/phases/<phase-name>/progress.json` (initial)
- Writes `.kbd-orchestrator/phases/<phase-name>/evolver-bridge.json` with the `item_to_change_map` stub (to be filled by kbd-plan when the phase is planned)
- Updates `current-waypoint.json` with the new phase as `stage: assessment_ready`
- Outputs: the new phase name and `next: /kbd-assess <phase-name>`

`context-management.md` — the pmpo-evolver-specific context management guide:

**Context budget rules for the evolver:**
1. **Feedback collection is always isolated.** Run `feedback-digest.sh` as a subprocess or background agent — never inline within the strategy routing session. The feedback data is voluminous; the session only reads the normalized `LearningSignal[]` output.
2. **Changelog ingestion is always isolated.** `changelog-fetch.sh` may ingest thousands of lines of release notes. Run isolated; session reads the JSON output only.
3. **Carry-forward aggregation is always isolated.** May scan dozens of reflection.md files; run as subprocess.
4. **The evolver session's context budget is reserved for:** perspective routing, competitive synthesis, learning signal interpretation, and strategic dreaming.
5. **Model class and context budget are correlated.** `small` models (used for file reads, commit classification, gate-1 checks) have small context windows — keep inputs under 4k tokens. `medium` models: under 16k. `frontier` models: full window available, but prefer concise inputs to preserve budget for synthesis.
6. **liter-llm cost tracking:** enable `get_cost` polling after each `complete` call during development to verify cost reduction is happening. Target: feedback collection + changelog ingestion costs should be ≤10% of what frontier-all would cost.

`model-discovery.md` (liter-llm-bridge addition) — documents how to query the configured system to discover available providers and models:
- Read `~/.config/liter-llm/config.toml` or `$LITER_LLM_CONFIG` path
- Call liter-llm MCP `list_models` tool → returns all configured aliases + resolved `provider/model`
- Call `health` tool → verifies which providers are currently reachable
- Decision protocol: if `medium` class is configured and reachable → use it; if not → fall through to `frontier`; never silently use `frontier` where `small` was intended
- Provider capability reference: Anthropic claude-haiku-4-5 = small; claude-sonnet-4-6 = medium/frontier; Groq llama-3.3-70b = medium; Ollama qwen3:4b = small (local); vLLM hosted = depends on loaded model

**Tasks:**
- [ ] 1. Create `scripts/evolver-seed-phase.sh` (executable)
- [ ] 2. Create `skills/process/pmpo-evolver/references/context-management.md`
- [ ] 3. Create `skills/process/liter-llm-bridge/references/model-discovery.md`
- [ ] 4. Smoke test: `bash evolver-seed-phase.sh test-evolution item-1` creates phase directory structure

---

## Dependency Order

```
change-evolver-001   (schema: pmpo-evolver.schema.json + evolution-state extensions)
    ↓
change-evolver-002   (feedback source taxonomy: loop-definition schema extension)
    ↓
change-evolver-003   (SKILL.md + model routing table)  ← depends on 001 + 002
    │
    ├── change-evolver-004   (competitor tracking)       ← depends on 001 + 003
    ├── change-evolver-005   (learning signals)          ← depends on 001 + 002
    ├── change-evolver-006   (validate-idea sub-skill)   ← depends on 001 + 003
    ├── change-evolver-007   (carry-forward + taxonomy)  ← depends on 003
    │
    └── change-evolver-008   (strategic dreaming)        ← depends on 001 + 005
            ↓
    change-evolver-009   (outer-loop wiring)             ← depends on 003
            ↓
    change-evolver-010   (phase-seeding + context guide) ← depends on all

Changes 004, 005, 006, 007 can be parallelized after 003 completes.
Change 008 depends on 005. Changes 009 and 010 are sequential at the end.
```

---

## Model Class Summary (Operator Addendum)

All changes in this phase emit `[MODEL_ROUTING]` directives. The full class map is defined in `change-evolver-003`'s `model-routing.md`, but the key principle is:

| Work type | Class | Why |
|-----------|-------|-----|
| File reads, git log, schema validation | `small` | Deterministic; cheap; no reasoning needed |
| Changelog extraction, sentiment classification, feasibility research | `medium` | Bounded NLP + synthesis; not open-ended |
| Competitive synthesis, strategic dreaming, spec generation, trend analysis | `frontier` | Open-ended synthesis; requires judgment |
| Human gate routing (pmpo-elicit) | platform-native | UI handled by the harness |

liter-llm is the resolver: `complete(model=<class>)` → resolves to the cheapest configured provider/model that meets the class. Falls through to host model if no provider configured for that class.

Context management: feedback collection, changelog ingestion, and carry-forward aggregation are **always run as isolated subprocess invocations** — never inline — to protect the evolver session's context budget for strategic reasoning.

---

## First Change to Apply

**change-evolver-001** — `skills/process/pmpo-evolver/references/schemas/pmpo-evolver.schema.json` + `evolution-state.schema.json` extension.

See `.kbd-orchestrator/changes/change-evolver-001/change.md` for full specification.
