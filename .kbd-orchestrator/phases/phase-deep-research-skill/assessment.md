# Assessment — phase-deep-research-skill

**Date:** 2026-07-08  
**Phase:** phase-deep-research-skill  
**Status:** assessment_complete  
**Playbook:** `docs/deep-research-skill-playbook.md` (1,179 lines, 44KB)

---

## 1. Objective

Execute the `docs/deep-research-skill-playbook.md` to create a production-ready **deep-research skill** that defines a standard way to leverage the existing Prometheus skills ecosystem for long-form, structured research — with proper segmentation of search, analysis, verification, and synthesis processes.

---

## 2. What Already Exists (Do NOT Rebuild)

### 2.1 Documentation & Specifications

| Asset | Location | Status |
|-------|----------|--------|
| Master spec | `docs/deep-research/index.md` | 1,685 lines — complete |
| Implementation plan | `docs/deep-research/plan.md` | 167 lines — scoped |
| Fable 5 comparison | `docs/deep-research/fable5-comparison.md` | complete |
| UI prototype | `docs/deep-research/deep-research-ui.html` | 4,336 lines — HTMX + Alpine |
| Research reports | `docs/deep-research/research/` | 8 reports |
| Wiki: master spec | `.prometheus/knowledge/wiki/prometheus-deep-research-skill-master-spec.md` | 67 lines |
| Wiki: Feynman integration | `.prometheus/knowledge/wiki/deep-research-feynman-integration.md` | present |
| Wiki: skill landscape | `.prometheus/knowledge/wiki/deep-research-skill-landscape.md` | present |
| Wiki: threaded research | `.prometheus/knowledge/wiki/threaded-concurrent-research.md` | present |
| Wiki: long-running | `.prometheus/knowledge/wiki/long-running-research-process-management.md` | present |
| Wiki: AG-UI/A2UI | `.prometheus/knowledge/wiki/ag-ui-a2ui-mcp-app-ui-frameworks.md` | present |

### 2.2 Infrastructure to Leverage (13 components)

| Component | Location | Role in Deep Research |
|-----------|----------|----------------------|
| `surreal-memory` | `tools/surreal-memory-server/` | Knowledge graph, vector search, time-travel |
| `liter-llm-bridge` | `skills/process/liter-llm-bridge/` | Cost-aware model routing per stage |
| `native-agent` | `skills/process/native-agent/` | Scaffold `prometheus-research` Rust binary |
| `mcp-server` | `skills/rust/mcp-server/` | MCP server patterns |
| `axum-patterns` | `skills/rust/axum-patterns/` | HTTP server foundation |
| `htmx-alpine-lit` | `skills/htmx/htmx-alpine-lit/` | UI skill patterns |
| `prometheus-entity-skills` | `skills/react/prometheus-entity-skills/` | Graph CRUD |
| `sycophancy-correction` | `skills/imported/sycophancy-correction/` | Bias detection during synthesis |
| `pmpo-elicit` | `skills/process/pmpo-elicit/` | Human escalation for low-confidence findings |
| `zeespec-interrogator` | `skills/process/zeespec-interrogator/` | Requirement extraction from queries |
| Feynman skills | `skills/learn/` | `learn-plan`, `learn-survey`, `learn-kb`, `learn-grade` |
| `kreuzberg` | `skills/document-extraction/kreuzberg/` | Document extraction |
| `iterative-evolver` | `skills/process/iterative-evolver/` | PMPO pattern reference |

### 2.3 Existing Deep Research Skills in User's ~/.claude/skills/

The user already has a basic `deep-research` skill (155 lines, ECC origin) using `firecrawl` + `exa` MCP. The new skill will supersede it with:
- 10-stage structured pipeline (vs. flat 3-step workflow)
- Knowledge graph persistence (vs. ephemeral reports)
- Contradiction detection and resolution (new)
- Feynman quality gate (new)
- `.research` package format (new, OKF-aligned)
- Multi-platform support with native Rust MCP server

---

## 3. Gap Analysis

### 3.1 What Does NOT Exist (Must Build)

**Skill files (primary deliverable):**

| File | Status | Priority |
|------|--------|----------|
| `skills/research/` category directory | MISSING | P0 |
| `skills/research/deep-research/SKILL.md` | MISSING | P0 |
| `skills/research/deep-research/skill.toml` | MISSING | P0 |
| 10 sub-skill `SKILL.md` files (stage-01 through stage-10) | MISSING | P0 |
| Parent `scripts/run-research.sh` | MISSING | P1 |
| Parent `scripts/export-package.sh` | MISSING | P1 |
| Parent `scripts/verify-sources.sh` | MISSING | P1 |
| Parent `scripts/build-graph.sh` | MISSING | P1 |
| Parent `scripts/detect-contradictions.sh` | MISSING | P1 |
| Templates (5 files: research-plan, source-eval, report, etc.) | MISSING | P1 |
| References (9 files: pipeline-arch, package-format, MCP-API, etc.) | MISSING | P2 |
| Hooks (4 files: pre, post, on-contradiction, on-completion) | MISSING | P2 |
| Agent definitions (5 files: claude, codex, opencode, cursor, kimi) | MISSING | P2 |

**Documentation updates:**
| File | Update Needed |
|------|--------------|
| `SKILLS.md` (skills index) | Add "Research" category |
| `README.md` | Add research category |
| `docs/deep-research/index.md` | Add skill cross-reference in §13 |

**Native binary (Phase 2 / optional):**
- `prometheus-research` Rust binary — scaffold via `native-agent` skill
- This is P3: the SKILL.md pipeline works without the binary

### 3.2 Scope Decision

The playbook describes **12 implementation phases**. For this KBD phase, the scope is:

**In scope (all phases 1–12):**
- Complete skill directory structure with all SKILL.md files
- 10 stage sub-skills with full input/output contracts
- Scripts, templates, references, hooks, agent definitions
- Validation and integration with existing docs
- Commit and push

**Out of scope for this phase (P3 future):**
- Building the `prometheus-research` Rust binary (requires `native-agent` invocation — separate phase)
- AG-UI/A2UI streaming integration with the existing HTML UI
- Deploying as a launchd MCP service

---

## 4. Goals for This Phase

| # | Goal | Success Criterion |
|---|------|-------------------|
| G-01 | Create `skills/research/deep-research/` directory structure | All dirs created, `npm run validate:skill` passes |
| G-02 | Write parent `SKILL.md` with 10-stage orchestration | Frontmatter valid, triggers defined, pipeline documented |
| G-03 | Write all 10 sub-skill `SKILL.md` files | Each has frontmatter, input/output contracts, integration refs |
| G-04 | Write scripts, templates, references, hooks, agents | All P1/P2 files present and executable (scripts) |
| G-05 | Pass skill validation | `npm run validate:strict skills/research/deep-research` passes |
| G-06 | Update docs index and SKILLS.md | New "Research" category appears in skill index |
| G-07 | Commit and push | Commit on main with conventional commit message |

---

## 5. Complexity Assessment

### 5.1 Size of Work

- **Primary deliverable:** ~15 SKILL.md files + 23 supporting files = ~38 files total
- **Largest file:** Parent `SKILL.md` (~300–400 lines with full pipeline docs)
- **Sub-skill files:** Each ~100–150 lines × 10 = ~1,200–1,500 lines total
- **Scripts:** ~50 lines each × 5 = ~250 lines shell
- **Estimated effort:** 10–12 hours (aligns with playbook's 11.75 hour estimate)

### 5.2 Risk Areas

| Risk | Likelihood | Mitigation |
|------|-----------|------------|
| SKILL.md frontmatter validation failure | Medium | Copy exact structure from existing valid skills (`mcp-server`, `native-agent`) |
| Sub-skill naming inconsistency | Low | Use exact names from playbook §4.2 table |
| Script executability | Low | `chmod +x` all scripts during creation |
| Integration references pointing to non-existent files | Medium | Reference only existing infrastructure skills, not future binary |
| SKILLS.md structure mismatch | Low | Read existing file before editing |

### 5.3 Parallelization Opportunity

The 10 sub-skill SKILL.md files can be written in parallel (no dependencies between stages at the SKILL.md level). Scripts, templates, and references are also independent. The parent SKILL.md should be written first as it sets the vocabulary and structure.

**Recommended execution order:**
1. Create all directories (1 change)
2. Write parent SKILL.md (1 change)
3. Write all 10 sub-skill SKILL.md files in parallel (1 change, batch)
4. Write skill.toml (1 change)
5. Write scripts + templates in batch (1 change)
6. Write references + hooks + agents in batch (1 change)
7. Validate + fix (1 change)
8. Update docs index + SKILLS.md (1 change)
9. Commit + push (1 change)

---

## 6. Recommended Phase Structure

**9 changes, targeting 7 goals:**

| Change | Description | Goals |
|--------|-------------|-------|
| change-drs-001 | Create directory structure + skill.toml | G-01 |
| change-drs-002 | Write parent `deep-research/SKILL.md` | G-02 |
| change-drs-003 | Write all 10 stage sub-skill `SKILL.md` files | G-03 |
| change-drs-004 | Write scripts (5 files) + templates (5 files) | G-04 |
| change-drs-005 | Write references (9 files) + hooks (4 files) + agents (5 files) | G-04 |
| change-drs-006 | Validate + fix errors | G-05 |
| change-drs-007 | Update `SKILLS.md` + `README.md` + `docs/deep-research/index.md` | G-06 |
| change-drs-008 | Install to user's `~/.claude/skills/` + verify trigger | G-05 |
| change-drs-009 | Commit + push | G-07 |

**Total changes:** 9  
**Total goals:** 7

---

## 7. Handoff to Analyze

**Key gaps found:**
1. The entire `skills/research/` category is absent — must create from scratch.
2. 38 files need to be created, anchored by 12 primary SKILL.md files.
3. No native binary is required for the core skill — SKILL.md-based pipeline is the deliverable.
4. The existing `~/.claude/skills/deep-research` (ECC) should be considered superseded by this skill once installed.
5. Validation tooling (`npm run validate:strict`) is the acceptance gate.

**Open questions for analyze/plan:**
- OQ-01: Should the parent `SKILL.md` emit the 10 stages sequentially or in a configurable DAG? (Recommendation: sequential default with DAG config in `skill.toml`)
- OQ-02: Should stage sub-skills be invocable as top-level skills (e.g., `/stage-02-search`) or only callable from the parent? (Recommendation: parent-callable only; avoids namespace pollution)
- OQ-03: Should the native `prometheus-research` binary scaffold be included in this phase or deferred? (Recommendation: defer — binary generation is a separate KBD phase)
- OQ-04: What model policy should the parent skill declare for the `model_routing` field? (Recommendation: planner=frontier, search=medium, verify=frontier, synthesize=frontier, export=small)

---

## 8. Stage Gate

Assess is the first stage — gate always passes.

**Assessment verdict: PROCEED to analyze → plan → execute**

Next command: `/kbd-analyze phase-deep-research-skill`
