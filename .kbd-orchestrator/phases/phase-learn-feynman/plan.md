# Plan — phase-learn-feynman

**Phase:** phase-learn-feynman
**Backend:** openspec
**Planned:** 2026-06-28
**Change count:** 28
**Ordering rationale:** Spike → Layer A substrate interfaces → Layer B UI primitive → Layer C first wave (core loop) → Layer C second wave (retention/practice/certify) → Knowledge base integration → Meta-learning (KBD self-documentation) → Adoption & meta-learning skills
**First change:** change-learn-001-spike-learner-model-schema

---

## Operator-Added Requirements (from /kbd-plan arguments)

Two additions beyond the assessment are incorporated into this plan:

### A. Custom Knowledge Base Integration
Operators arrive with domain-specific knowledge that general corpora cannot supply:
- A lawyer's private case law notes, a doctor's clinical protocols, a counselor's therapeutic frameworks, a business's proprietary methodology
- These knowledge bases are **owned by the operator**, not public, and cannot be assembled by `learn-goal`'s research loop
- The `content-grounding` substrate must be extended with a **KB adapter** that ingests, indexes, and retrieves from operator-provided knowledge bases (Dify KB, local files, MCP-accessible collections)
- Skills that consume `content-grounding` automatically benefit; no per-skill change required
- This adds 3 changes to the plan: KB adapter, KB-management skill (`learn-kb`), and integration wiring into `learn-goal` and `learn-grade`

### B. Meta-Learning: KBD, Skill Pack, and Harness Adoption
The KBD lifecycle, this skill pack's 100+ skills, and the harness ecosystem (Claude Code, OpenCode, Codex, Kimi, Zed, Cursor) are themselves complex subjects operators need to learn. Adoption failure is a real risk: operators with powerful tools they don't understand don't use them.

The learning system must be able to teach itself:
- `learn-goal "master the KBD lifecycle"` must work as well as `learn-goal "master transformer attention"`
- A `learn-about-system` skill surfaces what the skill pack can do, guides discovery, and builds a `learn-goal` automatically for operator-expressed interest areas
- A `learn-harness` skill provides per-harness capability orientation + Feynman loops over the harness's own concepts
- This adds 3 changes: `learn-about-system`, `learn-harness`, and a meta-grounding corpus for the skill pack and KBD lifecycle (built once, maintained as the pack evolves)

---

## Change List (28 changes)

### Group 0 — Design Spikes (unblock interface commits)

#### change-learn-001 — Spike: learner-model schema + CRDT conflict semantics
**Type:** Design-only (no code, no skill files)
**Deliverable:** `docs/learn/schemas/learner-model.schema.json` + `docs/learn/crdt-conflict-semantics.md`
**Scope:**
- Define `learner_model_seed` JSON format (output of `learn-survey`, consumed by `learner-model`)
- Specify field-level CRDT merge strategy for every `learner-model` field:
  - `mastery`: LWW with vector clock (most recent observation wins)
  - `fsrs_card.stability`: max(local, remote) — conservative, prefer more stable card
  - `fsrs_card.due`: min(local, remote) — conservative, prefer earlier review
  - `gaps`: union-append; gaps are never deleted by merge
  - `credential_evidence`: append-only log
- Specify the PFA update rule applied after ≥5 observations
- Validate semantics with 3 worked conflict examples (device A mastery=0.7, device B mastery=0.6, merge result=0.7 with vector clock; FSRS due conflict; gap dedup)
**Blocks:** change-learn-005 (learner-model Rust crate), change-learn-008 (learn-survey)

---

#### change-learn-002 — Spike: surface-bridge detect-surface-tier probe
**Type:** Design + minimal shell probe (no full Rust service yet)
**Deliverable:** `docs/learn/surface-tier-detection.md` + `shared/scripts/detect-surface-tier.sh`
**Scope:**
- Document exactly how each harness signals its surface capability:
  - Claude Code: check `$CLAUDE_CODE_VERSION`, `$MCP_SERVERS`, `AskUserQuestion` availability
  - OpenCode: check `$OPENCODE_VERSION`, structured prompt file convention
  - Codex: check `$OPENAI_CODEX`, file-based prompt convention
  - Kimi: check `$KIMI_CODE_VERSION`
  - Zed: check `$ZED_AI_CONTEXT`
- Produce a shell probe script that emits one of: `tier0_text | tier1_structured | tier2_mcp_app | tier3_full`
- Determine what MCP App server capabilities look like in the harness MCP handshake (if any)
- Produces `SURFACE_TIER` env var for use by `ui-surface` skill
**Note:** Full Axum MCP App server is deferred to change-learn-004; this spike unlocks `ui-surface` immediately
**Blocks:** change-learn-006 (ui-surface skill)

---

### Group 1 — Layer A Substrate

#### change-learn-003 — `content-grounding` service + public corpus assembly
**Type:** Shell service + schema
**Deliverable:** `shared/scripts/content-grounding.sh` + `docs/learn/schemas/grounding-corpus.schema.json`
**Scope:**
- Shell script: `content-grounding.sh --subject "<>" --level "<>" --budget-sources 20 --budget-minutes 30`
- Source priority chain: primary literature → textbooks → reference implementations → curated surveys → secondary → LLM synthesis (flagged)
- Misconception sources: explicit `--include-misconceptions` flag; uses firecrawl + tavily research for known-wrong-model examples
- Output: `grounding-corpus.json` with provenance per entry
- Reuses `pmpo-elicit` research loop as the orchestration backbone; extends budget parameters
- Provenance record schema: `{source_ref, source_type, content_summary, retrieved_at, confidence, is_misconception: bool}`
**Depends on:** nothing upstream in this phase

---

#### change-learn-004 — `content-grounding` KB adapter (custom knowledge bases)
**Type:** Shell adapter + Dify KB integration
**Deliverable:** `shared/scripts/content-grounding-kb.sh` + `docs/learn/kb-adapter.md`
**Scope:**
- Adapter that extends `content-grounding.sh` with `--kb <kb-id>` flag
- Supported KB sources:
  - **Dify KB** (MCP: `dify-kb` server already in stack): `dify_search` + `dify_list_documents` for retrieval; ingestion via `dify_add_document`
  - **Local files** (`--kb-local <path>`): ingest markdown/PDF/text files into a local vector store (use `palace_ingest` via `surreal-memory` palace API)
  - **surreal-memory palace** (`--kb-palace <palace-id>`): query directly via `palace_search` / `palace_recall`
  - **MCP filesystem** (`--kb-mcp-fs`): read from `mcp__filesystem__*` tools, ingest into palace
- Output: KB sources are merged into `grounding-corpus.json` with `source_type: "operator_kb"` and `kb_id` provenance
- Privacy: operator KB content is never sent to external search APIs; stays local to palace or Dify KB
- **Use cases enabled:** lawyer's case notes, doctor's clinical protocols, counselor's frameworks, business methodology, any specialized operator domain
**Depends on:** change-learn-003

---

#### change-learn-005 — `learner-model` Rust crate (schema + CRDT + FSRS)
**Type:** Rust crate
**Deliverable:** `substrate/learner-model/` Rust crate (new directory)
**Scope:**
- `Cargo.toml`: depends on `automerge`, `fsrs` (fsrs-rs), `serde`, `serde_json`, `tokio`
- Core types: `LearnerModel`, `ConceptState`, `FSRSCard`, `GapRecord`, `SessionRecord`
- Implements automerge document: model state is an automerge `AutoCommit` document
- CRDT merge: `merge(local: &mut Doc, remote_delta: &[u8]) → Result<()>` using field semantics from change-learn-001 schema
- FSRS integration: wraps `fsrs-rs` `FSRS::new(None)` with `next_states()` and `schedule()` calls; `FSRSCard` persisted in automerge document
- Cold-start: `seed_from_survey(survey_result: &SurveyResult) → Result<()>` — writes LLM-seeded Bayesian priors as initial mastery estimates
- Storage: `save(path: &Path)` / `load(path: &Path)` for local-dir persistence; delta export for iroh-docs sync
- MCP interface: exposes `read_mastery(concept_id)`, `write_mastery_update(concept_id, delta, source)`, `get_fsrs_due()`, `append_gap(gap)` as MCP tool stubs (shell-callable JSON RPC)
**Depends on:** change-learn-001 (schema), no other upstream in this phase

---

#### change-learn-004b — `storage-provider` trait crate (abstraction layer)
**Change ID:** change-learn-004b
**Type:** Rust trait crate
**Deliverable:** `substrate/storage-provider/` Rust crate
**Scope:**
- `StorageProvider` trait: `read`, `write`, `merge`, `list`, `watch` (async)
- `CrdtEngine` trait: `create`, `apply_delta`, `export_delta`, `get_value`
- `LocalDirAdapter`: filesystem-backed, no dependencies beyond std
- `automerge-rs` CRDT engine implementation
- Adapter discovery: `StorageProvider::detect() → Box<dyn StorageProvider>` — tries local-dir first, warns if iroh-docs not available
- **Iroh-docs adapter stub** (interface only, not implemented): `IrohDocsAdapter` struct with `unimplemented!()` bodies + TODO comment pointing to iroh crate
- Feature flags: `features = ["local-dir", "iroh-docs-stub", "crdt-automerge"]` — local-dir + automerge on by default
**Note:** Full iroh-docs integration is a follow-on phase (sovereign sync). This phase ships the abstraction and local-dir; the stub prevents API lock-in.
**Depends on:** change-learn-001 (conflict semantics doc)

---

### Group 2 — Layer B UI Primitive

#### change-learn-006 — `ui-surface` skill
**Type:** Skill (Layer B)
**Deliverable:** `skills/learn/ui-surface/SKILL.md`
**Scope:**
- Sources `detect-surface-tier.sh` (from change-learn-002) to resolve tier
- Tier 0: emit markdown; for surveys, emit structured checklist file + response file convention
- Tier 1: `AskUserQuestion` with `options` for Claude Code; structured `__ui_intent__.json` + `__ui_response__.json` file pair for OpenCode/Codex/Kimi/Zed
- Tier 2: MCP App iframe stub (logs "Tier 2 not yet served" until change-learn-014 ships the Axum server)
- Tier 3: deferred (out of scope this phase)
- **Degradation rule** (enforced in SKILL.md): if `detect-surface-tier.sh` returns a tier lower than `preferred_tier`, the skill MUST fall through to the highest available tier, never block
- Documents `intent_type` enum: `survey | explanation | grading | review | report | kb_query`
**Depends on:** change-learn-002 (tier probe)

---

### Group 3 — Layer C First Wave (Core Learning Loop)

#### change-learn-007 — `learn-goal` skill
**Type:** Skill
**Deliverable:** `skills/learn/learn-goal/SKILL.md` + `skills/learn/learn-goal/scripts/` + `skills/learn/learn-goal/references/`
**Scope:**
- Entry: `/learn-goal "<desire>" [--kb <kb-id>] [--depth-override <n>] [--time-budget-hours <n>]`
- Step 1: route through `content-grounding.sh` (standard corpus) + `content-grounding-kb.sh` (if `--kb` provided)
- Step 2: elicit via `pmpo-elicit` (time budget, prior knowledge claim, target level)
- Step 3: run feasibility gate: research-derived time vs. operator available time; thresholds RED/YELLOW/GREEN; sycophancy-correction on the result (gate may not be softened)
- Step 4: emit `learn-goal.json` and `grounding-corpus.json`
- **KB flag**: `--kb <kb-id>` merges operator KB into corpus with `source_type: "operator_kb"`; enables domain-expert personas (e.g., `/learn-goal "understand my firm's case strategy" --kb firm-cases`)
- References: `references/feasibility-thresholds.md`, `references/corpus-assembly-guide.md`
**Depends on:** change-learn-003, change-learn-004, change-learn-006

---

#### change-learn-008 — `learn-survey` skill
**Type:** Skill
**Deliverable:** `skills/learn/learn-survey/SKILL.md` + `skills/learn/learn-survey/scripts/`
**Scope:**
- Entry: `/learn-survey [--goal-path <learn-goal.json>]`
- Sources `learn-goal.json` + `grounding-corpus.json`
- Generates diagnostic items: conceptual (define/explain), procedural (apply/derive), misconception probes (is this statement correct?)
- Misconception probes sourced from misconception entries in `grounding-corpus.json`
- Renders via `ui-surface` at Tier 1 (structured prompts) or Tier 0 (checklist file)
- Produces `survey-result.json` including `recursion_floor`, `mastery_priors`, `misconceptions_detected`, `learner_model_seed`
- Writes `learner_model_seed` to `learner-model` via MCP tool call (change-learn-005)
- **KB context**: if `--kb` was used in `learn-goal`, survey probes include KB-specific concepts (e.g., "Explain the firm's three-pronged discovery strategy")
**Depends on:** change-learn-001 (schema), change-learn-005 (learner-model), change-learn-006 (ui-surface), change-learn-007 (learn-goal output)

---

#### change-learn-009 — `learn-grade` skill
**Type:** Skill
**Deliverable:** `skills/learn/learn-grade/SKILL.md` + `skills/learn/learn-grade/scripts/`
**Scope:**
- Not user-facing; called by `feynman-loop` and `learn-certify`
- Inputs: `explanation_text`, `concept_id`, `audience`, `grounding_corpus_path`, `learner_model_ref`
- Step 1: retrieve relevant grounding corpus entries for the concept (semantic search over corpus)
- Step 2: compare explanation against corpus: completeness, accuracy, misconceptions
- Step 3: run `sycophancy-correction` on grader output (S-02 check: no-gap grade when gaps are present)
- Step 4: generate novel transfer problems (from corpus, NOT from explanation text)
- Step 5: update learner-model mastery via MCP tool
- Output: `grade-result.json`
- **KB-aware**: if `grounding_corpus_path` includes `source_type: "operator_kb"` entries, grader checks against KB content too (enables grading against a doctor's own clinical protocols, not just public medicine)
- Anti-sycophancy enforcement: grader system prompt explicitly states "finding no gaps is the suspicious result, not the expected result"
**Depends on:** change-learn-003, change-learn-004b, change-learn-005, change-learn-006

---

#### change-learn-010 — `feynman-loop` skill
**Type:** Skill
**Deliverable:** `skills/learn/feynman-loop/SKILL.md` + `skills/learn/feynman-loop/scripts/` + `skills/learn/feynman-loop/references/`
**Scope:**
- Entry: `/feynman-loop <concept_id> [--depth <n>] [--audience novice|peer|skeptic] [--goal-path <learn-goal.json>]`
- PMPO mapping:
  - Spec: select concept + audience + depth budget (reads from `learn-goal.json`)
  - Plan: structure the explanation (analogies, sub-concepts, known misconceptions from survey)
  - Execute: operator produces explanation (scaffolded by agent via `ui-surface`); agent annotates gaps in real-time
  - Reflect: call `learn-grade` → `grade-result.json` → candidate child loops
- Recursion logic: gaps below recursion floor → surface as separate-goal suggestion; gaps above floor + depth > 0 → spawn child loop
- Depth accounting: tracks `current_depth` in `feynman-artifact.json` context
- Horizontal escalation: after novice loop closes, suggest `--audience peer`; after peer, suggest `--audience skeptic`
- Loop closure gate: all 3 mastery criteria from assessment §5 must hold
- Output: `feynman-artifact.json` per cycle; appended to session artifact log
- **KB-aware**: if operator KB is active, child loops on KB-specific concepts (e.g., proprietary methodologies) probe against KB grounding only
**Depends on:** change-learn-009 (learn-grade), change-learn-005 (learner-model), change-learn-006 (ui-surface)

---

### Group 4 — Layer C Second Wave

#### change-learn-011 — `learn-plan` skill
**Type:** Skill
**Deliverable:** `skills/learn/learn-plan/SKILL.md` + `skills/learn/learn-plan/scripts/`
**Scope:**
- Entry: `/learn-plan [--goal-path <learn-goal.json>] [--replan]`
- Sources `learn-goal.json`, `survey-result.json`, current `learner-model` mastery state
- Queries concept DAG from surreal-memory: `semantic_search` for prerequisite concepts
- Produces `curriculum.json`: ordered phases, prerequisite gates, time estimates per concept
- Re-plan mode (`--replan`): triggered by `feynman-loop` when mastery estimate diverges > 0.2; updates unstarted phases only; never reorders completed concepts
- Schedule suggestion: distributes curriculum over operator's available hours/week
- Renders via `ui-surface` Tier 0 (ordered markdown list) or Tier 2 (DAG visual via `ideation-mindmap` if available)
- **KB-aware**: curriculum includes KB-specific concept nodes; these are labeled `[operator_kb]` in `curriculum.json`
**Depends on:** change-learn-008 (learn-survey output), change-learn-005 (learner-model), change-learn-006 (ui-surface)

---

#### change-learn-012 — `learn-retain` skill
**Type:** Skill
**Deliverable:** `skills/learn/learn-retain/SKILL.md` + `skills/learn/learn-retain/scripts/`
**Scope:**
- Entry: `/learn-retain [--concept <concept_id>] [--session]`
- Reads FSRS due queue from `learner-model`
- For each due concept: surfaces review prompt via `ui-surface` ("What's still correct about your explanation? What would you change?")
- Grades review response via `learn-grade` (same corpus, lower threshold: ≥ 0.6 to count as retention)
- Calls `fsrs-rs` `next_states()` with rating; updates `FSRSCard` in `learner-model`
- Emits review artifact; updates session log
- **Scheduling integration**: `learn-plan` references review schedule in `curriculum.json`; `learn-retain` can be called from schedule or on-demand
**Depends on:** change-learn-005 (FSRS), change-learn-009 (learn-grade), change-learn-006 (ui-surface)

---

#### change-learn-013 — `learn-practice` skill
**Type:** Skill
**Deliverable:** `skills/learn/learn-practice/SKILL.md` + `skills/learn/learn-practice/scripts/`
**Scope:**
- Entry: `/learn-practice <concept_id> [--type derivation|implementation|transfer]`
- Problem types: derivation (re-derive from first principles), implementation (code/math artifact), transfer (novel context from `learn-grade` pool)
- Difficulty gating: harder problems unlock when `learner-model` mastery > 0.6 for the concept
- Interleaving: default mode rotates across problem types (not blocked by type), following interleaved practice evidence
- Grades responses via `learn-grade`; updates `learner-model` mastery; emits `practice-result.json`
- Practice artifacts added to session log for `learn-certify` evidence bundle
- **KB integration**: for operator KB concepts, implementation problems reference KB artifacts (e.g., "draft a motion using the firm's discovery strategy")
**Depends on:** change-learn-009 (learn-grade), change-learn-005 (learner-model), change-learn-006 (ui-surface)

---

#### change-learn-014 — `learn-certify` skill
**Type:** Skill
**Deliverable:** `skills/learn/learn-certify/SKILL.md` + `skills/learn/learn-certify/scripts/`
**Scope:**
- Entry: `/learn-certify [--checkpoint | --final] [--issuer <endpoint>]`
- Checkpoint mode: grades current curriculum concepts; emits intermediate OB 3.0 VC; updates learner-model
- Final mode prerequisites:
  - ≥ N feynman-artifacts (N = curriculum length)
  - ≥ 3 practice results per target concept
  - Capstone: novel transfer problem + teach-the-skeptic, graded by `learn-grade`
  - Mastery trajectory: no step-change anomaly
- OB 3.0 / W3C VC: self-issued, did-plc signed, evidence bundle = feynman-artifact paths + practice results + grade results + mastery trajectory
- Integrity guardrails: anomaly flag for step-change trajectories; sycophancy-correction on capstone responses
- Trust declaration: operator's did-plc signature asserts self-attestation
- Progress chart: Tier 0 = mastery table; Tier 2 = radar visual via `ui-surface` (deferred until Tier 2 server ships)
- `--issuer` param: signs VC at external 1EdTech-certified issuer endpoint; no hard dependency
**Depends on:** change-learn-009 (learn-grade), change-learn-005 (learner-model), change-learn-012 (retain), change-learn-013 (practice), change-learn-006 (ui-surface)

---

### Group 5 — Knowledge Base Management

#### change-learn-015 — `learn-kb` skill (operator knowledge base management)
**Type:** Skill
**Deliverable:** `skills/learn/learn-kb/SKILL.md` + `skills/learn/learn-kb/scripts/`
**Scope:**
- Entry: `/learn-kb <subcommand>`
- Subcommands:
  - `add --name <id> --source <path|url|dify-kb-id>`: ingest source into a named KB
  - `list`: list available KBs with source count and last-updated
  - `query <kb-id> "<query>"`: test retrieval from a KB
  - `update <kb-id> --source <path>`: add new documents to existing KB
  - `remove <kb-id>`: remove a KB (does not delete source files)
- Storage: KB metadata in `learner-model` session state; content in surreal-memory palace (local) or Dify KB
- Ingestion pipeline:
  - PDF/markdown/text → `palace_ingest` (surreal-memory) OR `dify_add_document` (Dify KB)
  - URL → firecrawl scrape → ingest
  - Dify KB ID → use existing Dify KB directly via `dify_search`
- **Domain personas**: a KB named `firm-cases` with legal documents enables `/learn-goal "understand our discovery strategy" --kb firm-cases`; the grounding corpus for that goal merges public law + firm-cases KB
- Privacy controls: local palace KBs never leave the machine; Dify KBs governed by the Dify instance's access controls
- References: `references/kb-types.md` (local palace vs. Dify vs. MCP filesystem)
**Depends on:** change-learn-003, change-learn-004

---

### Group 6 — Meta-Learning (KBD + Skill Pack Adoption)

#### change-learn-016 — Meta-grounding corpus for KBD lifecycle + skill pack
**Type:** Content artifact (maintained file)
**Deliverable:** `docs/learn/meta-corpus/kbd-lifecycle-corpus.json` + `docs/learn/meta-corpus/skill-pack-corpus.json`
**Scope:**
- `kbd-lifecycle-corpus.json`: grounding corpus for the KBD lifecycle concepts
  - Concepts: kbd-assess, kbd-analyze, kbd-plan, kbd-execute, kbd-reflect, kbd-evolve, OpenSpec, progress signaling, phase gates, waypoint files, hooks, evolver bridge
  - Source: CLAUDE.md, skills/process/*/SKILL.md, .kbd-orchestrator/SKILL.md (if exists), this plan
  - Misconception entries: common wrong mental models ("kbd-plan writes code", "OpenSpec is optional", "progress.json is updated automatically")
  - Confidence annotations per entry
- `skill-pack-corpus.json`: grounding corpus for the skill pack's capabilities
  - Concepts: skill categories (react, rust, process, learn, etc.), installation, platform parity, MCP servers, plugin format, agentskills.io compliance
  - Source: README.md, docs/guide/*, CLAUDE.md
  - Misconception entries: "skills only work on Claude Code", "the pack requires internet", "you have to restart Claude to use new skills"
- Both corpora are static-built at plan time, updated on each pack release (doc-updater integration)
- These corpora are the `--kb` sources used by `learn-goal` when the subject is KBD or this skill pack
**Depends on:** nothing (content-only change)

---

#### change-learn-017 — `learn-about-system` skill
**Type:** Skill
**Deliverable:** `skills/learn/learn-about-system/SKILL.md`
**Scope:**
- Entry: `/learn-about-system [--area <topic>]`
- Discovery layer: surfaces what the skill pack can do, organized by area
  - When called with no args: interactive discovery via `ui-surface` — "What are you trying to do? What kind of project are you working on?"
  - Routes to: `learn-goal` for structured learning intent; `learn-harness` for harness orientation; specific skill suggestions for immediate tasks
- KBD adoption path: for operators new to KBD, provides a guided Feynman loop over KBD lifecycle concepts using `kbd-lifecycle-corpus.json`
  - Triggers: `/learn-about-system --area kbd`
  - Result: calls `learn-goal "master the KBD lifecycle"` with `--kb kbd-lifecycle` as the corpus source
- Skill pack adoption path:
  - `/learn-about-system --area skills` → surveys available skill domains, suggests relevant skills for operator's context
  - Lists installed skills by domain, links to SKILL.md
- **Self-teaching**: the skill pack teaches itself using its own learning infrastructure. `learn-about-system` is the entry point that makes this concrete.
**Depends on:** change-learn-016 (meta-corpus), change-learn-007 (learn-goal), change-learn-006 (ui-surface)

---

#### change-learn-018 — `learn-harness` skill (per-harness capability orientation)
**Type:** Skill
**Deliverable:** `skills/learn/learn-harness/SKILL.md` + `skills/learn/learn-harness/references/`
**Scope:**
- Entry: `/learn-harness [--harness claude-code|opencode|codex|kimi|zed|cursor]`
- Auto-detects harness if not specified (via `detect-surface-tier.sh`)
- Per-harness orientation module:
  - **Claude Code**: skills, MCP servers, hooks, AskUserQuestion, plan mode, /commands, plugin.json
  - **OpenCode**: skill installation, file-based prompts, MCP config, codex-plugin.json format
  - **Codex**: skills, file-based input/output convention, environment setup
  - **Kimi Code**: config.toml, skill directory, MCP config, Kimi-specific patterns
  - **Zed**: AI context, skill discovery, Zed-specific limitations
  - **Cursor**: .cursor/skills/, rules format, sidebar AI interaction pattern
- Each orientation module is a Feynman-eligible subject:
  - `/learn-harness --harness claude-code` → triggers `learn-goal "master Claude Code's skill and MCP ecosystem"` with `--kb kbd-lifecycle` + `--kb skill-pack`
  - Short-circuit option: for operators who just need a capability overview (not a full Feynman loop), renders a Tier 1 capability map via `ui-surface`
- Cross-harness parity table: what each skill supports at each tier; updated by `doc-updater` on pack releases
**Depends on:** change-learn-016 (meta-corpus), change-learn-002 (tier detection), change-learn-006 (ui-surface)

---

### Group 7 — Tier 2 UI Serving (MCP App Server)

#### change-learn-019 — `surface-bridge` Axum MCP App server (Tier 2 substrate)
**Type:** Rust service
**Deliverable:** `substrate/surface-bridge/` Rust crate
**Scope:**
- Axum server: serves A2UI specs over HTTP; exposed as an MCP server tool
- `detect_surface_tier` MCP tool: accepts harness context, returns tier string
- `render_ui_intent` MCP tool: accepts UI intent JSON, returns rendered form/prompt appropriate to tier
- `collect_response` MCP tool: accepts session ID, returns operator response when available
- Supports Tier 2 iframe embed (MCP App pattern): serves a minimal HTML+JS shell that renders `ui-surface` intent as interactive forms
- Graceful degradation: if Axum server is not running, all tools return `tier: "tier0_text"` and text fallbacks
- **Does not ship in initial release** — Tier 0 and Tier 1 ship first (changes 1–18). This change makes Tier 2 available for operators on Claude Code with MCP App support.
- Installed as launchd service on macOS (same pattern as surreal-memory)
**Depends on:** change-learn-002 (probe spec), change-learn-004b (storage-provider)
**Note:** This is the last change in the plan because Tier 0 + Tier 1 are fully functional without it. No other change is blocked by this one after change-learn-006 (ui-surface) ships with the Tier 2 stub.

---

### Group 8 — Integration, Validation, Installation

#### change-learn-020 — skills/learn domain directory structure + validation
**Type:** Infrastructure
**Deliverable:** `skills/learn/` directory with domain README, `.agentskills` markers, validation CI update
**Scope:**
- Create `skills/learn/` domain directory
- Add `skills/learn/README.md` with domain overview, skill dependency diagram, and invocation examples
- Add all 12+ skill stubs to `npm run validate:strict` scope (currently only `skills/learn/ui-surface` will exist; stubs added as empty SKILL.md with valid frontmatter so validation passes)
- Update `marketplace/marketplace.json` with `learn` domain entry
- Update `plugin.json` with `learn` skill category
- Update `README.md` skills table with learn domain
- Update `docs/guide/` with `10-learn-skills.md` (domain overview)
**Depends on:** change-learn-006 (first learn skill)
**Note:** This change should run in parallel with Group 3 (immediately after change-learn-006); it scaffolds the domain so all subsequent changes have a valid home.

---

#### change-learn-021 — learn-goal + learn-survey + feynman-loop integration test
**Type:** Integration test (shell + manual walkthrough)
**Deliverable:** `tests/learn/integration-basic-flow.sh`
**Scope:**
- Happy path: `/learn-goal "understand transformer self-attention"` → `/learn-survey` → `/feynman-loop self_attention_mechanism --depth 2 --audience novice`
- Asserts: `learn-goal.json` exists and is valid; `survey-result.json` has `recursion_floor`; `feynman-artifact.json` produced; `grade-result.json` produced; learner-model updated
- Tests Tier 0 path only (no surface-bridge required); uses file fixtures for grounding corpus
- Tests KB path: `/learn-goal "understand our discovery strategy" --kb test-fixtures/sample-kb`
- Validates sycophancy-correction is invoked on `learn-grade` output
**Depends on:** change-learn-007, change-learn-008, change-learn-009, change-learn-010

---

#### change-learn-022 — learn-retain + learn-practice + learn-certify integration test
**Type:** Integration test
**Deliverable:** `tests/learn/integration-full-loop.sh`
**Scope:**
- Full loop: run `feynman-loop` for 2 concepts → trigger `learn-retain` → run `learn-practice` → run `learn-certify --checkpoint`
- Asserts: FSRS card updated in learner-model after retain; practice-result.json produced; checkpoint VC emitted as valid JSON-LD
- Tests mastery trajectory anomaly detection (fabricated step-change → `integrity_warning: true`)
**Depends on:** change-learn-021 (basic flow), change-learn-012, change-learn-013, change-learn-014

---

#### change-learn-023 — learn-kb integration test
**Type:** Integration test
**Deliverable:** `tests/learn/integration-kb.sh`
**Scope:**
- Ingest a test fixture KB (markdown files in `tests/learn/fixtures/sample-kb/`)
- Run `learn-goal --kb sample-kb`; assert `grounding-corpus.json` has `source_type: "operator_kb"` entries
- Run `learn-grade` with KB corpus; assert KB-specific concepts appear in transfer problems
- Tests `learn-kb list` and `learn-kb query`
**Depends on:** change-learn-015, change-learn-021

---

#### change-learn-024 — learn-about-system + learn-harness integration test
**Type:** Integration test
**Deliverable:** `tests/learn/integration-meta.sh`
**Scope:**
- Run `/learn-about-system --area kbd`; assert routes to `learn-goal` with `kbd-lifecycle-corpus.json` as KB
- Run `/learn-harness --harness claude-code`; assert produces capability map
- Validates that `kbd-lifecycle-corpus.json` has valid schema and contains expected concept IDs
- Validates that `skill-pack-corpus.json` contains expected skill categories
**Depends on:** change-learn-016, change-learn-017, change-learn-018

---

#### change-learn-025 — install-skills-flat.sh update for learn domain
**Type:** Infrastructure
**Deliverable:** Updated `scripts/install-skills-flat.sh` to include `skills/learn/` in installation scope
**Scope:**
- Add `skills/learn/` to the platform skill installation sweep
- Add `substrate/learner-model/` and `substrate/storage-provider/` to the build-and-install script (compile Rust crates, install binaries to `~/.prometheus/bin/`)
- Add `substrate/surface-bridge/` to the launchd service installation (macOS only; skipped on other platforms with a warning)
- Update `shared/scripts/detect-toolchain.sh` to check for `learner-model` binary and `surface-bridge` service status
**Depends on:** change-learn-020

---

#### change-learn-026 — docs/guide update for learn domain
**Type:** Documentation
**Deliverable:** `docs/guide/10-learn-skills.md` + updates to `docs/guide/00-index.md`
**Scope:**
- Complete guide section for the learn domain
- Per-skill: purpose, entry command, inputs, outputs, cross-harness behavior table
- Dependency diagram (skill invocation graph)
- KB adapter guide: how to ingest a custom KB, how to use it in `learn-goal`, privacy notes
- Meta-learning guide: how to use the skill pack to learn the skill pack; KBD adoption path
- Harness-specific quick-starts: "Getting started with learn skills on Claude Code / OpenCode / Kimi"
**Depends on:** change-learn-020

---

#### change-learn-027 — CLAUDE.md update for learn domain
**Type:** Documentation
**Deliverable:** Updated `CLAUDE.md` with learn domain section
**Scope:**
- Add `## Learn Domain` section to CLAUDE.md
- Document: Layer A substrate (surface-bridge, storage-provider, learner-model, content-grounding), Layer B (ui-surface), Layer C skills
- Document KB adapter pattern
- Document Tier 0/1/2 degradation contract
- Document meta-learning (learn-about-system, learn-harness)
- Add `skills/learn/` to the directory structure diagram
**Depends on:** change-learn-020

---

#### change-learn-028 — v1.4.0 release bump + changelog
**Type:** Release
**Deliverable:** Updated `package.json`, `.claude-plugin/plugin.json`, `marketplace/marketplace.json`, `CHANGELOG.md`
**Scope:**
- Bump to v1.4.0
- Changelog: learn domain, KB adapter, meta-learning skills, substrate (learner-model, storage-provider, surface-bridge), 28-change phase
- marketplace.json: add learn domain with skill descriptions and tags
- plugin.json: add learn skills to capabilities list
**Depends on:** all integration tests passing (change-learn-021 through change-learn-024)

---

## Execution Order

```
Parallel group A (no dependencies):
  change-learn-001  spike: learner-model schema
  change-learn-002  spike: surface-tier probe
  change-learn-016  meta-corpus (content-only)

After change-learn-001 + change-learn-002:
  change-learn-003  content-grounding service       (needs nothing)
  change-learn-005  learner-model Rust crate        (needs 001)
  change-learn-004b storage-provider Rust crate     (needs 001)

After change-learn-003:
  change-learn-004  KB adapter                      (needs 003)

After change-learn-002:
  change-learn-006  ui-surface skill                (needs 002)

After change-learn-006:
  change-learn-020  domain directory + validation   (parallel with Group 3)

After change-learn-003 + change-learn-004 + change-learn-006:
  change-learn-007  learn-goal skill

After change-learn-001 + change-learn-005 + change-learn-006 + change-learn-007:
  change-learn-008  learn-survey skill

After change-learn-003 + change-learn-004b + change-learn-005 + change-learn-006:
  change-learn-009  learn-grade skill

After change-learn-009 + change-learn-005 + change-learn-006:
  change-learn-010  feynman-loop skill

After change-learn-010:
  change-learn-021  integration test: basic flow
  change-learn-011  learn-plan skill

After change-learn-011 + change-learn-005 + change-learn-009 + change-learn-006:
  change-learn-012  learn-retain skill
  change-learn-013  learn-practice skill

After change-learn-012 + change-learn-013 + change-learn-009 + change-learn-005 + change-learn-006:
  change-learn-014  learn-certify skill

After change-learn-014:
  change-learn-022  integration test: full loop

After change-learn-003 + change-learn-004:
  change-learn-015  learn-kb skill

After change-learn-015 + change-learn-021:
  change-learn-023  integration test: KB

After change-learn-016 + change-learn-007 + change-learn-006:
  change-learn-017  learn-about-system skill
  change-learn-018  learn-harness skill

After change-learn-017 + change-learn-018:
  change-learn-024  integration test: meta

After change-learn-020 (launchd patterns established):
  change-learn-019  surface-bridge Axum server      (LAST — Tier 2 enhancement)

After change-learn-020:
  change-learn-025  install-skills-flat update
  change-learn-026  docs/guide update
  change-learn-027  CLAUDE.md update

After all integration tests + docs:
  change-learn-028  v1.4.0 release bump
```

---

## Scope Guard (enforced by this plan)

The following are explicitly OUT OF SCOPE for this phase:
- `learn-to-build` (creation bridge to PMPO) — deferred
- Full iroh-docs P2P sync adapter — interface stubbed only; implementation is a follow-on phase
- 1EdTech-certified credential issuer — `--issuer` param documented but not implemented
- DKT (deep knowledge tracing) — requires domain pre-training; excluded by architecture
- Any UI protocol invention — only implements A2UI, AG-UI, MCP Apps
- Tier 3 (full external browser surface) — deferred

If any change during execute tries to implement these, it must be flagged and rejected.

---

## Risk Register

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| Scope creep into Layer A substrate (each crate becomes a product) | HIGH | HIGH | Tier 0 fallback enforced; substrate crates ship only the interface + local-dir adapter in this phase |
| Grader fidelity (learn-grade misses misconceptions) | MEDIUM | HIGH | Anti-sycophancy system prompt; transfer problems generated from corpus not explanation; spike-recommended for grader eval |
| Learner-model CRDT conflict semantics wrong in practice | MEDIUM | MEDIUM | Spike in change-learn-001 resolves; worked examples validate semantics before crate is built |
| KB adapter privacy (operator KB content leaks to external search) | LOW | HIGH | KB queries routed to local palace or Dify KB only; never sent to firecrawl/tavily |
| Meta-corpus staleness (learn-about-system teaches wrong skill pack state) | MEDIUM | LOW | doc-updater integration noted; meta-corpus rebuild on pack release |
| Tier 2 server scope creep (surface-bridge becomes an AG-UI product) | LOW | HIGH | change-learn-019 is explicitly last; Tier 0+1 are fully functional without it |

---

*Plan complete. First change: change-learn-001. Run `/kbd-execute phase-learn-feynman` to begin.*
