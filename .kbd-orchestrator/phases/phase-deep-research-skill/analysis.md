# Analysis — phase-deep-research-skill

**Date:** 2026-07-08  
**Phase:** phase-deep-research-skill  
**Stage:** analyze  
**Mode:** Stack specified (no discovery needed — architecture fully defined in master spec + playbook)

---

## 1. Scope of Analysis

This analyze pass resolves the four open questions from the assessment and confirms
build-vs-adopt verdicts for every infrastructure dependency in the playbook. No
external research was needed — all answers are derivable from the existing codebase
and the master spec.

---

## 2. Open Questions — Resolved

### OQ-01: Sequential vs DAG stage execution?

**Decision: Sequential default, DAG optional via `skill.toml`**

Evidence:
- All existing multi-stage skills in the pack (`iterative-evolver`, `kbd-process-orchestrator`,
  `native-agent`) execute stages sequentially in `SKILL.md` and configure DAGs via metadata.
- The master spec (§7) describes the 10 stages as a pipeline with natural sequential dependencies:
  Plan must precede Search, Search must precede Retrieve, Collect must precede Verify, etc.
- Only stages 7 (Graph) and 8 (Cite) can run concurrently after Collect+Verify.
- For a SKILL.md-based portable skill (no Rust binary), sequential is the only safe default —
  parallel execution requires process management outside SKILL.md's scope.
- The `[features] threaded = true` flag in `skill.toml` is the correct way to declare DAG
  capability for harnesses that support it.

**Verdict:** Write parent SKILL.md with sequential orchestration. Declare `threaded = true`
in `skill.toml` as a discoverable feature flag for future binary integration.

---

### OQ-02: Stage sub-skills as top-level or parent-only callable?

**Decision: Parent-callable by default; top-level invocation documented but not encouraged**

Evidence:
- Inspecting `skills/process/kbd-process-orchestrator/skills/` and
  `skills/process/iterative-evolver/skills/`: sub-skills all have their own `SKILL.md`
  with frontmatter `name:` fields, making them technically top-level installable, but
  the pattern is to invoke via parent.
- The `evolve/SKILL.md` has `name: evolve` and is usable as `/evolve` — this is the
  accepted pattern in the pack.
- For deep-research stages, allowing `/stage-02-search` as a top-level command would
  pollute the namespace with 10 generic names that could clash with other skills.
- **Better pattern:** Give sub-skills descriptive names scoped to the parent:
  `deep-research-planner`, `deep-research-search`, etc. — but this adds prefix clutter.
- **Optimal pattern:** Keep stage names short in their directory (`stage-01-planner`)
  but do NOT add them to the harness skills list. Document them as internal sub-skills
  invocable only via the parent's instructions.

**Verdict:** Sub-skills are named `stage-0N-<name>` in their directory. Parent SKILL.md
is the only top-level entry point. Sub-skills are invoked by the parent's instructions,
not by direct slash command. The `SKILL.md` frontmatter `name:` for sub-skills should
reflect this: e.g., `name: deep-research-stage-01` (prefixed) to avoid namespace collision
if ever installed standalone.

---

### OQ-03: Include native `prometheus-research` binary scaffold in this phase?

**Decision: Defer to a separate phase (phase-prometheus-research-binary)**

Evidence:
- The `native-agent` skill generates a complete Rust workspace — this is a multi-hour
  generation and validation process that has its own KBD lifecycle.
- The SKILL.md-based pipeline is a complete, functional, portable deliverable on its own.
  It can execute research via the harness's existing tools (firecrawl, tavily, surreal-memory
  MCP, etc.) without any binary.
- All 13 infrastructure components referenced in the playbook are already available as
  MCP servers or harness-accessible tools. The binary adds streaming, checkpointing, and
  long-running process management — all P3 features.
- Binary generation would at least double the scope of this phase (it requires
  `native-agent` invocation → Rust workspace creation → CI wiring → release tagging).
- The playbook explicitly acknowledges this is a separate phase in §5 (Phase 9: Agent Definitions).

**Verdict:** Defer. This phase delivers the complete SKILL.md pipeline. A follow-on phase
(`phase-prometheus-research-binary`) will scaffold the native binary via `native-agent`.

---

### OQ-04: Model policy for `model_routing` field?

**Decision: frontier for Plan/Verify/Synthesize; medium for Search/Retrieve; small for status/export**

Evidence from liter-llm-bridge references and existing skill routing policies:
- `frontier` class maps to `claude-sonnet-5` or `claude-opus-4-8` — deep reasoning required
  for query decomposition (Stage 1), contradiction resolution (Stage 6), and synthesis (Stage 9).
- `medium` class maps to `claude-sonnet-4-6` — adequate for web search execution (Stage 2),
  content retrieval/chunking (Stage 3), source collection (Stage 4).
- `small` class maps to `claude-haiku-4-5` — adequate for citation formatting (Stage 8)
  and package export (Stage 10).
- `frontier` for verification (Stage 5) because credibility scoring requires nuanced judgment.
- `frontier` for knowledge graph building (Stage 7) because entity extraction and relation
  inference require semantic reasoning.

**Final model routing table:**

| Stage | Phase key | Class | Rationale |
|-------|-----------|-------|-----------|
| 1 Planner | `research-plan` | `frontier` | Query decomposition needs deep reasoning |
| 2 Search | `research-search` | `medium` | Keyword generation + search execution |
| 3 Retrieve | `research-retrieve` | `medium` | Content chunking + metadata extraction |
| 4 Collect | `research-collect` | `medium` | Source indexing into graph |
| 5 Verify | `research-verify` | `frontier` | Credibility scoring needs judgment |
| 6 Resolve | `research-resolve` | `frontier` | Contradiction analysis + resolution |
| 7 Graph | `research-graph` | `frontier` | Entity extraction + relation inference |
| 8 Cite | `research-cite` | `small` | Citation formatting — mechanical |
| 9 Report | `research-synthesize` | `frontier` | Report synthesis from evidence |
| 10 Export | `research-export` | `small` | Package assembly — mechanical |

---

## 3. Build-vs-Adopt Verdicts

### Infrastructure (all ADOPT — already in pack)

| Component | Verdict | Rationale |
|-----------|---------|-----------|
| `surreal-memory` MCP | **ADOPT** | Running service at 127.0.0.1:7888; tools available in session |
| `liter-llm-bridge` | **ADOPT** | `liter-llm` binary installed; SKILL.md routing pattern proven |
| `sycophancy-correction` | **ADOPT** | Submodule at `skills/imported/sycophancy-correction/`; binary available |
| `pmpo-elicit` | **ADOPT** | Skill at `skills/process/pmpo-elicit/`; checkpoint scripts available |
| Feynman skills | **ADOPT** | 12 skills in `skills/learn/`; `learn-grade` is the quality gate |
| `kreuzberg` | **ADOPT** | `skills/document-extraction/kreuzberg/`; document extraction |
| `mcp-server` | **ADOPT** | Canonical Axum pattern reference — used for future binary |

### Research tools (harness-level, no build needed)

| Tool | Source | Verdict |
|------|--------|---------|
| `firecrawl_search` | `mcp-server-firecrawl` MCP | **USE** — available in session |
| `tavily_search` | `tavily-mcp` MCP | **USE** — available in session |
| `mcp__surreal-memory__search_memories` | surreal-memory MCP | **USE** |
| `mcp__surreal-memory__semantic_search` | surreal-memory MCP | **USE** |
| `mcp__surreal-memory__create_entity` | surreal-memory MCP | **USE** |

### Nothing to build for this phase

All infrastructure is adopted. The deliverable is **skill documentation and orchestration instructions** — 38 files of `SKILL.md` + supporting text. No new Rust code, no new binaries, no new MCP servers.

---

## 4. Design Decisions

### 4.1 Parent SKILL.md — orchestration model

The parent skill uses **prose-based orchestration**: it describes the 10-stage pipeline in natural language with references to sub-skills. This is the same pattern as `kbd-process-orchestrator` and `iterative-evolver`. The harness interprets the instructions and calls sub-skills as needed.

Explicit sub-skill invocation syntax:
```
Invoke: skills/research/deep-research/skills/stage-01-planner/SKILL.md
```
This follows the reference pattern used in `native-agent` and `iterative-evolver`.

### 4.2 `.research` package format

OKF v0.1 (vendored at `shared/references/okf-v0.1.md`) is the base format.
Research-specific extensions are additive:
```yaml
# OKF required
type: research-report

# Prometheus extensions
confidence: 0.87
verification_status: verified
research_stage: complete
contradiction_count: 2
sources_count: 47
feynman_grade: B+
```

This preserves OKF permissive consumption (unknown fields never break consumers).

### 4.3 Sub-skill naming convention

Directory: `stage-0N-<name>/` (matches playbook exactly)  
Frontmatter `name:`: `deep-research-stage-0N` (prefixed to avoid namespace collision)  
`metadata.category`: `research`  
`metadata.tags`: include both `deep-research` and stage-specific tags

### 4.4 Triggers in parent SKILL.md

Based on the existing `deep-research` ECC skill's triggers (which work well) plus Prometheus-specific extensions:

**Keywords:** research, deep research, investigate, analyze, deep dive, study, comprehensive report, what is the current state of, competitive analysis, market research, technology evaluation, literature review, knowledge synthesis, due diligence  
**Semantic:** "Any request requiring synthesis from multiple sources with citations and verification"

### 4.5 allowed-tools declaration

```
file_system web_search code_interpreter sequential_thinking memory browser tavily firecrawl
```
This matches `iterative-evolver` (the closest existing parallel) plus adds `tavily` and `firecrawl` which the research skill explicitly requires.

---

## 5. Risk Mitigations

| Risk | Mitigation |
|------|-----------|
| Sub-skill `name:` collides with other skills | Prefix all sub-skill names: `deep-research-stage-0N` |
| `metadata.tags` missing in strict validation | Include explicitly in every frontmatter |
| Script chmod not set | Create scripts with `#!/usr/bin/env bash` header + note to run `chmod +x` |
| References to non-existent binary | All references to `prometheus-research` binary are in `references/` docs, not in `SKILL.md` instructions |
| Playbook's `parent:` frontmatter field not in schema | Validator schema has `properties` but no `additionalProperties: false` — unknown fields pass |

---

## 6. Candidate Libraries / Tools

No new libraries needed for this phase. All candidates are adopt-only:

See `library-candidates.json` for the machine contract.

---

## 7. Handoff to Plan

**Key adopt verdicts:** All 13 infrastructure components adopted. Nothing to build.  
**Resolved open questions:** 4/4 resolved — see §2.  
**Design decisions:** 5 architectural decisions locked (orchestration model, package format, naming convention, triggers, allowed-tools).  
**Next stage:** Plan — define the 9 changes with exact file lists and acceptance criteria.
