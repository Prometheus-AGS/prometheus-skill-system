# Assessment — pmpo-evolver

**Phase:** pmpo-evolver
**Assessed:** 2026-06-28
**Assessor:** kbd-assess
**Operator directive:** Build an extremely well-defined process for evolving completed/released projects using five research perspectives: competitive analysis, domain trend research, unique-product next-step research, operator-driven idea validation, and Karpathy-style self-learning from usage/feedback/history.

---

## Executive Summary

The `pmpo-evolver` phase has a rich pre-existing foundation: `iterative-evolver` (full PMPO loop with 8 domain adapters, 5 subagents, pluggable state providers), `kbd-evolve` (landscape-first evolution brief generator with tiered research protocol), and `pmpo-outer-loop` (standing loop wrapper that runs iterative-evolver ticks). Together these cover roughly 40–50% of what the operator has defined.

The critical gaps are in the **five evolution perspectives** the operator specified:

1. **Competitive analysis with feature-parity tracking** — `analyze.md` does landscape scans but has no structured competitor-tracking schema, no changelog-diff mechanism, and no "parity gap" output that survives across cycles.
2. **Domain trend / new-standards research** — present in `landscape-research.md` (kbd-evolve) and `analyze.md` (iterative-evolver) but fragmented, with no operationalized domain-classification step that determines *which* standards bodies or communities to watch.
3. **Unique-product next-step research** — `kbd-evolve` briefs handle this partially via `--criteria strategic`, but there is no first-class "unique product path" mode that draws from prior reflections + web research to propose the *continuation arc* of the project's own evolution.
4. **Operator idea validation** — `pmpo-elicit` now handles the elicitation primitive (just shipped), but no skill assembles the full idea-to-spec-to-execution pipeline: idea intake → research → feasibility score → spec draft → gate → KBD phase.
5. **Karpathy self-learning from usage/feedback/history** — the `loop-definition.schema.json` has `feedback_sources` (command / gh-query / file / url) but no semantic source types for: sentiment analysis, feedback systems (GitHub issues, Discord, in-app feedback), usage telemetry, or commit-history analysis. The iterative-evolver's `assessment.schema.json` has no `learning_signals` field.

The existing stack needs a **pmpo-evolver skill** that acts as the *strategy router*: given a released project, it selects and orchestrates the correct evolution perspective (or combines multiple) into a structured evolution brief, then hands off to `iterative-evolver` / `kbd-evolve` / `pmpo-outer-loop` as appropriate.

**Goals G1–G4 are NOT MET. G5 is PARTIALLY MET. G6 is PARTIALLY MET.**

---

## Artifact Inventory

### What exists

| Artifact | Location | Status | Notes |
|----------|----------|--------|-------|
| `iterative-evolver` | `skills/process/iterative-evolver/` | EXISTS — solid | Full PMPO loop, 8 domain adapters, 5 subagents, pluggable state, hooks. Missing: competitor tracking schema, learning_signals, idea-validation mode |
| `kbd-evolve` | `skills/process/kbd-evolve/` | EXISTS — solid | Landscape-first evolution briefs. Missing: feature-parity tracking, changelog diff, idea-validation entry |
| `pmpo-outer-loop` | `skills/process/pmpo-outer-loop/` | EXISTS — solid | Standing loop wrapper. `feedback_sources` schema: command/gh-query/file/url only — no sentiment/telemetry/feedback-system types |
| `loop-definition.schema.json` | `skills/process/pmpo-outer-loop/references/schemas/` | EXISTS | Has `feedback_sources[]` but limited to 4 primitive types; no semantic source types |
| `evolution-state.schema.json` | `skills/process/iterative-evolver/references/schemas/` | EXISTS | Has goals, convergence, iteration history. Missing: `learning_signals`, `competitor_tracking`, `idea_origin` |
| `analyze.md` | `skills/process/iterative-evolver/prompts/` | EXISTS | Landscape scan, benchmarks, trends, opportunities. Missing: structured competitor-diff, changelog tracking, parity matrix |
| `landscape-research.md` | `skills/process/kbd-evolve/references/` | EXISTS | Tiered search protocol (quick/standard/deep). Missing: domain-classification step, standards-body mapping, feedback-system scraping |
| `software.md` (domain adapter) | `iterative-evolver/references/domain/` | EXISTS | Code health, test health, competitive search. Missing: semantic version tracking, changelog ingestion |
| `product.md` (domain adapter) | `iterative-evolver/references/domain/` | EXISTS | NPS, support tickets, feature requests (listed). Missing: operative protocol for collecting and interpreting these |
| `pmpo-elicit` | `skills/process/pmpo-elicit/` | EXISTS — just shipped | Async checkpoint/resume, platform parity. Ready to use as the idea-intake mechanism |

### What does NOT exist (gaps)

| Artifact | Required for | Gap severity |
|----------|-------------|--------------|
| `skills/process/pmpo-evolver/SKILL.md` | G1 — the entry command | HIGH — nothing to invoke |
| `pmpo-evolver` schema (`evolver.schema.json`) | G2 — state for evolution cycles | HIGH |
| Competitor tracking schema + `competitive-analysis.md` reference | G1 competitive perspective | HIGH |
| Changelog-diff mechanism | G1 competitive parity tracking | MEDIUM |
| Domain taxonomy + standards-body mapping reference | G1 trend research perspective | MEDIUM |
| `feedback-sources.md` — semantic source types (sentiment, telemetry, issue tracker) | G1 Karpathy perspective | HIGH |
| Loop-definition schema extension: `mcp-tool`, `sentiment`, `gh-issues`, `telemetry` source types | G1/G5 | MEDIUM |
| Idea-validation pipeline (`validate-idea` sub-skill or prompt) | G1 idea-validation perspective | HIGH |
| `learning-signals.md` — protocol for ingesting commit history, bug fix patterns, usage logs | G1 Karpathy perspective | HIGH |
| Outer-loop wiring: pmpo-evolver → selects correct perspective → hands off to iterative-evolver | G3 | HIGH |
| Inner-loop bridge from perspective research into KBD phase creation | G4 | MEDIUM |
| Platform files for pmpo-evolver (parallel to kbd-goal's platform/ dir) | G6 | LOW |

---

## Gap Analysis by Goal

### G1 — Ship `skills/process/pmpo-evolver/SKILL.md`
**Status: NOT MET**

No `pmpo-evolver` directory exists. The SKILL.md must define:
- The **strategy router**: given a released project, classify which evolution perspective(s) to apply
- The **five perspective modes** (competitive, trend, unique-product, idea-validation, self-learning)
- Entry commands and their argument contracts
- Handoff to `iterative-evolver` / `kbd-evolve` / `pmpo-outer-loop` / `kbd-assess` as appropriate

**What partially covers it:** `kbd-evolve` + `iterative-evolver` together provide ~50% of the surface, but there is no single entry command that routes across all five perspectives.

### G2 — Define the evolver schema (`evolver.schema.json`)
**Status: NOT MET**

`evolution-state.schema.json` exists but does not include:

```json
{
  "learning_signals": [],         // MISSING — Karpathy feedback inputs
  "competitor_tracking": {},      // MISSING — parity delta per competitor
  "idea_origin": {},              // MISSING — tracks where the evolution idea came from
  "perspective": "",              // MISSING — which of the 5 perspectives drove this cycle
  "feedback_digest": {}           // MISSING — structured summary of feedback-source outputs
}
```

A new `pmpo-evolver.schema.json` should extend (not replace) the existing evolution-state schema, adding these fields, or a new top-level schema that wraps an evolution cycle with perspective metadata.

### G3 — Wire evolver into pmpo-outer-loop
**Status: PARTIALLY MET**

`pmpo-outer-loop` already calls `/evolve "<evolution_name>"` in each tick. However:
- The evolver (iterative-evolver) is called without a perspective context — it runs the full PMPO loop for whatever domain it infers
- There is no mechanism for the outer loop to say "this tick: run the competitive perspective" vs "this tick: run the self-learning perspective"
- The outer loop's `feedback_sources` are primitive (command/file/url) — they cannot semantically route to a sentiment analysis or GitHub issues scrape

**Gap:** pmpo-outer-loop needs a `perspective` field in `loop.json` that `loop-tick` passes to `/evolve` as a parameter, which `iterative-evolver` then uses to load the correct analysis sub-protocol.

### G4 — Inner-loop bridge (evolver items → KBD phases)
**Status: PARTIALLY MET**

`evolver-bridge.json` exists and the schema is defined. The `iterative-evolver` README documents the nested architecture (evolver → KBD inner loop). However:
- The bridge is a one-way write: KBD changes write back to the bridge; the evolver reads it
- There is no protocol for the evolver to *seed* KBD phase goals from the evolution plan items
- The `kbd-plan` skill has an evolver-bridge check but it reads `evolver_item_id` from an existing bridge — it does not create the bridge from scratch based on evolver plan output

**Gap:** A `pmpo-evolver-seed-phase.sh` script or equivalent protocol that, given an evolver plan item, creates a new KBD phase with `goals.md` seeded from the plan item's `target` and `success_criteria`.

### G5 — State persistence (`evolution.json` with resumable cursor)
**Status: PARTIALLY MET**

`evolution-state.schema.json` covers: goals, iterations, convergence_status, checkpoints, history. The `state-init.sh` + `state-checkpoint.sh` scripts handle filesystem persistence, and surreal-memory integration is documented.

**Gaps:**
- No `perspective_cursor` — when a cycle uses multiple perspectives sequentially, there is no cursor tracking which perspective has been applied in the current iteration
- `feedback_sources` in loop.json cannot represent mcp-tool sources (needed for surreal-memory recall, Fireflies sentiment, GitHub issue scraping via MCP)
- No `learning_signals[]` field to record what was learned from each perspective in a machine-readable way

### G6 — Platform parity
**Status: PARTIALLY MET**

`iterative-evolver` ships to all platforms via `install-skills-flat.sh` (dynamic find). `pmpo-outer-loop` is installed. `kbd-evolve` is installed.

**Gap:** The new `pmpo-evolver` SKILL.md must be created in `skills/process/pmpo-evolver/` to be auto-discovered. No platform-specific behavioral differences are expected beyond what `pmpo-elicit` already covers — but the idea-validation mode (which uses `AskUserQuestion` on Claude Code) needs a `references/platforms/` section parallel to kbd-goal's.

---

## Five Perspectives — Detailed Gap Analysis

### Perspective 1: Competitive Analysis + Feature Parity

**What exists:** `analyze.md` does a landscape scan with `benchmarks[]`, `gaps_to_close[]`. `landscape-research.md` (kbd-evolve) does tiered search with changelog reading in `deep` mode.

**What's missing:**
- No **competitor registry** — a persistent list of tracked competitors per project (currently re-discovered every cycle)
- No **parity matrix** schema — a structured diff: feature X → competitor has it / we don't / we have it better
- No **changelog ingestion** mechanism: given a competitor GitHub repo, fetch their recent releases/CHANGELOG.md and diff against prior scan to identify what's *new* since last cycle
- No **release velocity tracking**: how fast are competitors shipping? (leading indicator of competitive pressure)

**Design pattern to add:** A `competitive-analysis.md` reference doc + `competitor-registry.json` state file + changelog-diff sub-step in the analyze phase.

### Perspective 2: Domain Trend Research (Standards / Innovations)

**What exists:** `analyze.md` step 1 (Landscape Scan) covers general trends. `landscape-research.md` covers tiered web search.

**What's missing:**
- No **domain classification** step that maps a project to its relevant standards bodies (e.g., IETF RFCs for protocols, W3C for web, NIST for security, ISO for compliance) and community sources (GitHub awesome lists, HN Ask threads, Substack newsletters for the domain)
- No **standards-tracking** mechanism: given a known standard (e.g., "MCP spec"), fetch its changelog and detect breaking or additive changes
- No **innovation horizon** categorization: immediate (0-3 months), near-term (3-12 months), long-term (12+ months)

**Design pattern to add:** A `domain-taxonomy.md` reference that maps domain keywords → standards bodies + community sources + polling frequency. Used by the trend research perspective to know where to look.

### Perspective 3: Unique-Product Next-Step Research

**What exists:** `kbd-evolve` with `--criteria strategic` partially covers this — it reads carry-forwards from the last reflection and scores candidates. The `reflection.md` from the previous phase is always available.

**What's missing:**
- No explicit **"continuation arc" mode**: given that this project is novel with no direct competitors, synthesize the natural next step from (a) the last 3 reflections' carry-forwards, (b) the project's design philosophy, and (c) web research on where the problem domain is heading
- No **carry-forward aggregation**: the evolver does not read and synthesize all prior phase `reflection.md` files — it only reads the current state
- No **"what would the project's creator think next"** prompt pattern (a design-philosophy document that constrains the evolution direction)

**Design pattern to add:** A `design-philosophy.md` template (operator-written, stored per project) that the unique-product perspective reads as a constraint. A `carry-forward-aggregator.sh` script that walks all phase reflection.md files and extracts `## Carry-Forwards` sections into a unified list.

### Perspective 4: Operator Idea Validation

**What exists:** `pmpo-elicit` (just shipped) handles the intake. `zeespec-interrogator` validates specs.

**What's missing:**
- No **idea-validation pipeline** as a unified skill or prompt sequence: idea → research → feasibility score → spec draft → human gate → KBD phase creation
- No **idea intake schema**: structured representation of an operator-submitted idea with context fields (motivation, success criteria, constraints, related prior work)
- No **feasibility research step**: given an idea, research whether it has been tried before (by competitors or open source), what the implementation complexity is, and whether the required dependencies exist
- No **idea-to-spec bridge**: from a validated idea to a `SPEC.md` that `zeespec-interrogator` can score

**Design pattern to add:** A `validate-idea` sub-skill inside `pmpo-evolver/skills/` that implements the full pipeline: intake (via pmpo-elicit) → research → score → spec draft → gate.

### Perspective 5: Karpathy Self-Learning (Usage / Feedback / History)

**What exists:** `loop-definition.schema.json` has `feedback_sources[]` with types: command, gh-query, file, url. The product domain adapter mentions NPS, support tickets, feature requests — but as static assessment criteria, not operative data-collection protocols.

**What's missing:**
- No **semantic feedback source types** in the loop schema: `gh-issues` (with label filter + sentiment), `gh-prs` (merged PRs = shipped features), `commit-history` (pattern analysis: what kinds of bugs are being fixed?), `mcp-tool` (any MCP call as a feedback source), `sentiment-file` (a pre-run sentiment analysis output), `telemetry-url` (API endpoint returning usage metrics)
- No **commit-history analysis** protocol: given a git log, extract: bug fix frequency by component, feature addition cadence, churn hotspots, recent refactors (signals for what needs evolving)
- No **feedback digest** structure: how to normalize outputs from different feedback source types into a common `{signal, severity, count, examples[]}` format that the evolver can reason about
- No **learning_signals persistence**: after processing feedback, what was learned should be stored in evolution state so the next tick doesn't re-derive it
- No **sentiment analysis integration**: `surreal-memory` has `analyze_reflect_phase` (sycophancy) but no general sentiment analysis tool; `mcp__sycophancy-correction__detect_sycophancy` could be repurposed for qualitative feedback analysis

**Design pattern to add:** 
- Extend `loop-definition.schema.json` with new `feedback_sources.type` values: `gh-issues`, `commit-history`, `mcp-tool`, `telemetry-url`, `sentiment-file`
- Add `learning-signals.md` reference document: protocol for collecting, normalizing, and persisting signals from each source type
- Add `learning_signals[]` to `evolution-state.schema.json`
- Add `commit-history-analyze.sh` script: runs `git log --oneline --since=<last-tick>`, classifies commits by type (fix/feat/refactor/chore), outputs JSON

---

## What Is NOT a Gap

- The PMPO loop engine (`iterative-evolver`) is solid and production-ready — no changes needed to the core loop
- `kbd-evolve` landscape brief generation is sound — no changes needed
- `pmpo-outer-loop` standing loop infrastructure is complete — needs schema extension only, not structural change
- `pmpo-elicit` (just shipped) is the correct intake primitive for the idea-validation perspective — no re-work needed
- The evolver-bridge between iterative-evolver and KBD inner loop is defined and implemented — needs a seeding protocol only
- Platform install is handled by dynamic `find` — no explicit skill-list maintenance needed

---

## Identified Gaps (G-01 through G-14)

| ID | Gap | Goal | Perspective | Priority |
|----|-----|------|-------------|----------|
| G-01 | `skills/process/pmpo-evolver/SKILL.md` missing — no strategy router entry command | G1 | All | HIGH |
| G-02 | `pmpo-evolver.schema.json` missing — no schema for perspective metadata + learning signals | G2 | All | HIGH |
| G-03 | No competitor-registry pattern — competitors re-discovered every cycle | G1 | Competitive | HIGH |
| G-04 | No parity matrix schema — no structured feature-delta tracking across cycles | G1 | Competitive | HIGH |
| G-05 | No changelog-ingestion mechanism — can't detect what competitors shipped since last scan | G1 | Competitive | MEDIUM |
| G-06 | No domain-taxonomy reference — no mapping from project keywords to standards bodies + community sources | G1 | Trend | MEDIUM |
| G-07 | No carry-forward aggregation — prior phase reflections not synthesized into a unified list | G1 | Unique-product | MEDIUM |
| G-08 | No idea-validation sub-skill — idea intake → research → feasibility → spec → gate pipeline missing | G1/G4 | Idea validation | HIGH |
| G-09 | No semantic feedback source types in loop-definition schema (gh-issues, commit-history, mcp-tool, telemetry-url) | G5 | Self-learning | HIGH |
| G-10 | No learning-signals reference + persistence — feedback not accumulated in evolution state | G5 | Self-learning | HIGH |
| G-11 | No commit-history analysis script | G5 | Self-learning | MEDIUM |
| G-12 | No perspective cursor in evolution state — multi-perspective cycles can't track which ran | G5 | All | MEDIUM |
| G-13 | No pmpo-evolver → iterative-evolver perspective handoff protocol | G3 | All | MEDIUM |
| G-14 | No evolver-seed-phase protocol — plan items don't automatically become KBD phase goals.md | G4 | All | MEDIUM |

---

## Open Questions for Plan/Execute

1. **Strategy router architecture**: Should `pmpo-evolver` be a new top-level orchestrator that calls `iterative-evolver` as a sub-skill, or should it extend `iterative-evolver` with new perspective modes? Recommendation: new top-level skill that composes the existing stack — avoids modifying the stable iterative-evolver SKILL.md.

2. **Competitor registry location**: Per-project in `.evolver/competitor-registry.json` (local, project-scoped) or per-evolution-name in the evolution state? Recommendation: per-project file, read at the start of every competitive perspective tick.

3. **Changelog ingestion scope**: Full CHANGELOG.md parse or GitHub Releases API only? Recommendation: GitHub Releases API via `gh-query` feedback source (already supported) for structured data; fallback to CHANGELOG.md file scrape via `url` source.

4. **Karpathy feedback sources — telemetry**: Telemetry integration is product-specific (Posthog, Amplitude, custom). Should the schema support arbitrary URL+JSON-path sources? Recommendation: yes — `telemetry-url` source type with `jsonpath` field for metric extraction, similar to the existing `url` type's `interpret` field.

5. **Idea validation — spec generation**: Should the spec draft use `zeespec-interrogator` or produce a new lighter-weight spec format? Recommendation: produce a KBD-compatible `SPEC.md` that `zeespec-interrogator` can score, not a custom format.

---

## Assessment Conclusion

The `pmpo-evolver` phase must build the **strategy routing layer** that the operator described: a skill that takes a released project and drives its evolution along one or more of five perspectives (competitive, trend, unique-product, idea-validation, self-learning), using the existing iterative-evolver/kbd-evolve/pmpo-outer-loop stack as the execution engine.

**14 gaps identified.** Recommended change count: **8–10 changes**, ordered foundation-up: schema first, SKILL.md second, perspective implementations (competitive, trend, unique-product, idea-validation, self-learning) third, outer-loop wiring and inner-loop seed protocol fourth.

**Recommended next step:** `/kbd-plan pmpo-evolver` — produce the ordered change list. Use OpenSpec backend. Priority: G-01 (SKILL.md), G-02 (schema), G-09/G-10 (Karpathy sources, highest operator emphasis), G-03/G-04 (competitor tracking), G-08 (idea validation).

---

## Research Annex — Competitive Landscape Findings

*Research conducted 2026-06-28 via web search + arxiv survey. Validates and enriches the gap analysis above.*

### Key Systems Reviewed

| System | What it does | Relevant to pmpo-evolver |
|--------|-------------|-------------------------|
| **The Kitchen Loop** (arxiv:2603.25697) | 6-phase autonomous product evolution: Backlog → Ideate → Triage → Execute → Polish → Regress. 285+ iterations, 1,094+ merged PRs, zero regressions | HIGH — directly analogous; provides key design patterns |
| **Darwin Gödel Machine** (arxiv:2505.22954) | Self-improving agent via archive of stepping stones; probabilistic parent selection; staged evaluation (10 → 50 → 200 tasks) | HIGH — Archive of Stepping Stones + staged gating patterns directly applicable |
| **Anthropic Dreaming** (May 2026) | Background consolidation of agent session history into persistent memory; auto-approve rates grow from 20% at <50 sessions to 40%+ at 750+ sessions | MEDIUM — model for strategic dreaming between evolver cycles |
| **Hermes Agent** (NousResearch) | SQLite session persistence, ShareGPT trajectory export for model fine-tuning. No competitive scanning or roadmap generation | LOW — conflates agent's own product with user's product; wrong abstraction level |
| **SWE-EVO** (arxiv:2512.18470) | Benchmark showing agents reach only 25% on long-horizon evolution vs. 72% on isolated bug fixes | CONTEXT — confirms the capability gap `/pmpo-evolver` targets is real and large |
| **Metacognitive Self-Improvement** (arxiv:2506.05109) | Argues genuine self-improvement requires: assess own performance → plan what to learn → evaluate whether learning worked | HIGH — directly maps to evolver's outer loop structure |
| **SWE-RL / Meta RLVR** | Bug-injector + bug-solver adversarial co-evolution; verifiable reward signals | MEDIUM — validates Karpathy verifiability constraint |
| **MOSS** (arxiv:2409.16120) | Structured agent memory: working + episodic + semantic layers; each memory tagged with origin phase, confidence, validity conditions | MEDIUM — informs learning_signals schema design |
| **LangGraph v1.0** | Explicit typed state schemas; checkpointers (SQLite/Postgres); time-travel debugging; migration scripts for running threads | LOW — architecture reference for state schema design |

### The 5 Competitive Whitespace Gaps (Nobody Is Doing These)

1. **Competitive scan native to the development loop.** Klue, Clay's Claygent, etc. are standalone monitoring products. No coding agent or harness has competitive scanning that feeds directly into a development backlog. Vacant ground.
2. **Changelog-driven evolution.** No system does: "read competitors' changelogs from last 30 days → identify features shipped → determine which we should have → create KBD phases." All pieces exist; the integration is missing.
3. **Synthetic usage as a feedback source (Kitchen Loop "As a User x 1000").** Most systems use real user feedback. The Kitchen Loop runs synthetic power-user simulation to surface gaps between what the product claims and what it actually delivers. No harness has this natively.
4. **Long-horizon evolution tooling.** SWE-EVO shows 25% performance on long-horizon evolution — but no harness feature is designed specifically for multi-cycle, multi-phase product evolution. `/pmpo-evolver` would be the first.
5. **Strategic dreaming (product-direction memory consolidation).** Anthropic's Dreaming consolidates execution-quality lessons. Nobody consolidates *product-direction lessons* ("this feature class didn't resonate," "this domain trend is accelerating faster than expected"). This layer of strategic memory does not exist in any current agent system.

### 8 Validated Design Patterns for `/pmpo-evolver`

These patterns are each grounded in a specific validated system. All 8 should be incorporated into the SKILL.md and schema design:

| # | Pattern | Source | Where to implement |
|---|---------|--------|-------------------|
| 1 | **Spec-Surface-First Decision Loop** — evolver's judgment unit is "what does our spec claim vs. what does the competitive landscape claim" | Kitchen Loop | `competitive-analysis.md` reference; parity matrix schema |
| 2 | **Verifiable Outcomes Gate** — every evolution idea must become a machine-checkable acceptance criterion before executing | Karpathy RLVR + Kitchen Loop "Unbeatable Tests" | Triage step in SKILL.md; idea-validation sub-skill |
| 3 | **Archive of Stepping Stones** — failed evolution attempts stored with non-zero revisit probability | Darwin Gödel Machine | `.evolver/archive/<change-id>/manifest.json` with `revisit_weight` |
| 4 | **Dreaming Between Cycles (Strategic)** — post-cycle consolidation of product-direction lessons, distinct from PMPO Reflect (execution quality) | Anthropic Dreaming (adapted) | `post-cycle-dream` step in SKILL.md; `evolver-lessons.md` entries |
| 5 | **Staged Idea Evaluation** — Gate 1 (30s plausibility), Gate 2 (5min domain research), Gate 3 (full spec+KBD plan); fail-fast | Darwin Gödel Machine staged benchmarking | idea-validation sub-skill gate structure |
| 6 | **Feedback Source Taxonomy Extension** — add `competitor-scan`, `changelog`, `sentiment-feed`, `usage-trace`, `research-query` to loop.json schema | pmpo-outer-loop gap analysis | loop-definition.schema.json extension |
| 7 | **Metacognitive Self-Assessment Loop** — baseline capture → delta measurement → prediction vs. outcome logging | arxiv:2506.05109 | `learning-signals.md` + `learning_signals[]` in evolver state schema |
| 8 | **Human Gate as First-Class Design Citizen** — escalation declared at design time at: competitive threat, architectural change, budget exceedance, regression detected | Kitchen Loop Drift Control + pmpo-elicit | `escalation_points[]` in loop.json; integrate with pmpo-elicit |

### Revised Gap Priorities (Research-Informed)

The research confirms and sharpens the assessment's gap priorities:

- **G-09 (feedback source taxonomy)** — confirmed highest priority; no existing system solves this; Kitchen Loop + Karpathy both validate the need
- **G-03/G-04 (competitor tracking)** — whitespace #1 in the competitive landscape; high first-mover value
- **G-08 (idea validation with staged gates)** — Darwin Gödel Machine's staged evaluation directly specifies the implementation shape
- **G-02 (schema: `learning_signals` + `competitor_tracking` + `perspective`)** — MOSS and LangGraph validate the multi-layer memory schema pattern
- **G-10 (learning signals persistence)** — Dreaming pattern and MOSS both validate this; strategic dreaming is distinct from execution reflection

### Sources

arxiv:2603.25697 (Kitchen Loop), arxiv:2505.22954 (Darwin Gödel Machine), arxiv:2512.18470 (SWE-EVO), arxiv:2506.05109 (Metacognitive Learning), arxiv:2409.16120 (MOSS), arxiv:2604.14228 (Claude Code analysis), claude.com/blog/new-in-claude-managed-agents (Dreaming), karpathy.bearblog.dev/year-in-review-2025/ (2025 Year in Review).
