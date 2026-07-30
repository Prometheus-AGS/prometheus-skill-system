# Prometheus Deep Research Skill — Integration Playbook

**Date:** 2026-07-04  
**Version:** 1.0.0  
**Status:** Implementation-ready playbook  
**Target:** `skills/research/deep-research` within the Prometheus Skill Pack  
**Prerequisites:** Read `docs/deep-research/index.md` (Master Specification), `docs/SKILL_TEMPLATE.md`, and `SKILLS.md` (Skill Pack Index)

---

## Table of Contents

1. [Executive Summary](#1-executive-summary)
2. [System Analysis](#2-system-analysis)
3. [Integration Points Map](#3-integration-points-map)
4. [Skill Architecture](#4-skill-architecture)
5. [Step-by-Step Implementation](#5-step-by-step-implementation)
6. [File Structure & Templates](#6-file-structure--templates)
7. [MCP Server Integration](#7-mcp-server-integration)
8. [UI Protocol Integration](#8-ui-protocol-integration)
9. [Testing & Validation](#9-testing--validation)
10. [Platform Distribution](#10-platform-distribution)
11. [References](#11-references)

---

## 1. Executive Summary

This playbook provides the complete step-by-step instructions for adding the **Prometheus Deep Research** skill to the Prometheus Skill Pack (`/Users/gqadonis/Projects/prometheus/prometheus-skill-pack`). The skill implements the 10-stage research pipeline defined in `docs/deep-research/index.md` and integrates with the existing skill ecosystem (process orchestration, entity management, MCP servers, surreal-memory, native-agent generation, liter-llm, Feynman learning, and HTMX/Alpine UI).

**What this skill adds:**
- A portable `SKILL.md`-based deep research agent usable across Claude Code, Codex, OpenCode, Cursor, Windsurf, Kimi, MiniMax, Roo, Amp, and Gemini
- A native Rust MCP server (`prometheus-research`) with AG-UI/A2UI streaming
- A `.research` package format for persistent, queryable knowledge assets
- Integration with the existing `surreal-memory` distributed state system
- Feynman learning loop integration (`learn-plan`, `learn-survey`, `learn-kb`, `learn-grade`)
- Threaded/concurrent research with per-thread context isolation

**What already exists (don't rebuild):**
- Master spec: `docs/deep-research/index.md` (1,685 lines, 22 sections)
- UI prototype: `docs/deep-research/deep-research-ui.html` (4,336 lines, HTMX + Alpine.js)
- Research reports: `docs/deep-research/research/` (8 reports)
- Fable 5 comparison: `docs/deep-research/fable5-comparison.md`
- Implementation plan: `docs/deep-research/plan.md`

**What this playbook creates:**
- The skill directory structure under `skills/research/deep-research/`
- `SKILL.md` with frontmatter, triggers, and instructions
- `skill.toml` with metadata and dependencies
- Sub-skills for each pipeline stage
- Integration scripts and hooks
- Test suite and validation

---

## 2. System Analysis

### 2.1 Existing Skill System Structure

The Prometheus Skill Pack uses a **hierarchical directory structure** with skills organized by category:

```
skills/
├── architecture/          # clean-architecture
├── devops/                # argocd-multicloud, gitops-bootstrap, etc.
├── document-extraction/   # kreuzberg
├── flutter/               # flutter-rust-ffi
├── go/                    # go-base-patterns
├── htmx/                  # htmx-alpine-lit
├── process/               # kbd-process-orchestrator, iterative-evolver, etc.
├── python/                # pyo3-bridge
├── react/                 # prometheus-entity-skills, react-vite-stack
├── rust/                  # actor-model, axum-patterns, mcp-server, etc.
├── tauri/                 # tauri-react-vite
├── testing/               # bdd-testing, bdd-video-proof
├── typescript/            # typescript-base-patterns
└── imported/              # artifact-refiner, sycophancy-correction (submodules)
```

**Key insight:** There is no `research/` category yet. We will create it.

### 2.2 Skill File Format

Each skill requires at minimum:

```
skills/<category>/<skill-name>/
├── SKILL.md               # Required: YAML frontmatter + instructions
├── skill.toml             # Optional: metadata, dependencies, scripts
├── scripts/               # Optional: executable scripts
├── references/            # Optional: detailed docs, API refs
├── templates/             # Optional: code templates, prompts
├── assets/                # Optional: images, diagrams
└── agents/                # Optional: agent definitions for harnesses
```

**Required frontmatter format** (from `docs/SKILL_TEMPLATE.md`):

```yaml
---
name: skill-name
description: Clear, concise description (max 1024 chars)
license: MIT
compatibility: Node.js >=18, Claude Code >=1.0.0
metadata:
  author: your-name
  version: '1.0.0'
  category: react|rust|ui-ux|devops|testing|documentation
  tags: [tag1, tag2, tag3]
---
```

### 2.3 Complex Skill Patterns (from existing skills)

**`kbd-process-orchestrator`** (18 child skills):
- Uses `skills/` subdirectory with child skill folders
- Each child has its own `SKILL.md`
- Parent `SKILL.md` orchestrates with references to children
- Uses `hooks/` for lifecycle integration
- Has `agents/` for harness-specific agent definitions

**`native-agent`** (generates Rust binaries):
- Uses `templates/` for code generation
- Uses `prompts/` for LLM prompt templates
- Has `references/` for detailed architecture docs
- Uses `skill.toml` for build configuration

**`iterative-evolver`** (PMPO engine):
- Uses `scripts/` for automation
- Uses `skills/` subdirectory for child evolver skills
- Has `workflows/` for process definitions
- Integrates with `surreal-memory` via hooks

### 2.4 Existing Infrastructure to Leverage

| Infrastructure | Location | How Deep Research Uses It |
|----------------|----------|--------------------------|
| **surreal-memory** | `tools/surreal-memory-server/` | Knowledge graph persistence, per-thread context, vector search, time-travel |
| **liter-llm** | `skills/process/liter-llm-bridge/` | Cost-aware model routing across research stages |
| **native-agent** | `skills/process/native-agent/` | Generate `prometheus-research` Rust binary |
| **mcp-server** | `skills/rust/mcp-server/` | Build MCP server for tool integration |
| **prometheus-entity-skills** | `skills/react/prometheus-entity-skills/` | Graph CRUD for knowledge assets |
| **htmx-alpine-lit** | `skills/htmx/htmx-alpine-lit/` | UI skill for HTMX + Alpine.js patterns |
| **Feynman skills** | `skills/learn/` | `learn-plan`, `learn-survey`, `learn-kb`, `learn-grade`, `learn-practice`, `learn-retain`, `learn-certify` |
| **sycophancy-correction** | `skills/imported/sycophancy-correction/` | Bias correction during research synthesis |
| **pmpo-elicit** | `skills/process/pmpo-elicit/` | Human escalation for low-confidence findings |
| **zeespec-interrogator** | `skills/process/zeespec-interrogator/` | Requirement extraction from research queries |

---

## 3. Integration Points Map

### 3.1 Where Deep Research Fits

```
┌─────────────────────────────────────────────────────────────┐
│                   Prometheus Skill Pack                       │
├─────────────────────────────────────────────────────────────┤
│  skills/                                                    │
│  ├── research/                    ← NEW CATEGORY              │
│  │   └── deep-research/           ← THIS SKILL              │
│  │       ├── SKILL.md                                         │
│  │       ├── skill.toml                                       │
│  │       ├── skills/            ← Sub-skills (10 stages)    │
│  │       │   ├── stage-01-planner/                           │
│  │       │   ├── stage-02-search/                          │
│  │       │   ├── stage-03-retrieve/                         │
│  │       │   ├── stage-04-collect/                           │
│  │       │   ├── stage-05-verify/                           │
│  │       │   ├── stage-06-resolve/                          │
│  │       │   ├── stage-07-graph/                            │
│  │       │   ├── stage-08-cite/                             │
│  │       │   ├── stage-09-report/                           │
│  │       │   └── stage-10-export/                           │
│  │       ├── scripts/                                        │
│  │       ├── references/                                     │
│  │       ├── templates/                                       │
│  │       └── hooks/                                          │
│  ├── process/                 ← USES: kbd-process-orchestrator│
│  ├── rust/                    ← USES: mcp-server, axum       │
│  ├── htmx/                    ← USES: htmx-alpine-lit        │
│  └── learn/                   ← USES: Feynman skills        │
│                                                              │
│  tools/                                                      │
│  └── surreal-memory-server/   ← USES: graph + vector store   │
│                                                              │
│  docs/                                                       │
│  └── deep-research/           ← EXISTS: specs, UI, reports   │
└─────────────────────────────────────────────────────────────┘
```

### 3.2 Skill Dependencies (skill.toml)

```toml
[dependencies]
# Process orchestration
"skills/process/kbd-process-orchestrator" = { required = true, version = ">=1.0.0" }
"skills/process/liter-llm-bridge" = { required = true, version = ">=1.0.0" }
"skills/process/pmpo-elicit" = { required = false, version = ">=1.0.0" }

# Infrastructure
"skills/rust/mcp-server" = { required = true, version = ">=1.0.0" }
"skills/rust/axum-patterns" = { required = true, version = ">=1.0.0" }
"tools/surreal-memory-server" = { required = false, version = ">=1.0.0" }

# UI
"skills/htmx/htmx-alpine-lit" = { required = false, version = ">=1.0.0" }

# Learning
"skills/learn/learn-plan" = { required = false, version = ">=1.0.0" }
"skills/learn/learn-survey" = { required = false, version = ">=1.0.0" }
"skills/learn/learn-kb" = { required = false, version = ">=1.0.0" }
"skills/learn/learn-grade" = { required = false, version = ">=1.0.0" }

# Bias correction
"skills/imported/sycophancy-correction" = { required = false, version = ">=1.0.0" }

# Native agent generation (for building prometheus-research binary)
"skills/process/native-agent" = { required = false, version = ">=1.0.0" }
```

### 3.3 Data Flow Architecture

```
User Query
    │
    ▼
┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│  SKILL.md   │───→│  Research   │───→│ 10-Stage    │
│  (portable) │    │ Pipeline    │    │ Pipeline    │
└─────────────┘    │ (SKILL.md)  │    │ (sub-skills)│
                   └─────────────┘    └──────┬──────┘
                                              │
                   ┌──────────────────────────┼──────────┐
                   │                          │          │
                   ▼                          ▼          ▼
            ┌─────────────┐          ┌─────────────┐  ┌─────────────┐
            │ surreal-    │          │  liter-llm  │  │  Feynman    │
            │ memory      │          │  (routing)  │  │  skills     │
            │ (graph +    │          └─────────────┘  └─────────────┘
            │  vector)    │
            └─────────────┘
                   │
                   ▼
            ┌─────────────┐
            │  .research  │
            │  package    │
            │  (OKF +     │
            │   prom-ext) │
            └─────────────┘
                   │
                   ▼
            ┌─────────────┐    ┌─────────────┐
            │  AG-UI /    │───→│  HTMX UI    │
            │  A2UI       │    │  (docs/)    │
            │  (MCP App)  │    └─────────────┘
            └─────────────┘
```

---

## 4. Skill Architecture

### 4.1 Parent Skill: `deep-research`

The parent `SKILL.md` is the entry point. It:
- Declares triggers (keywords that activate the skill)
- Provides the high-level orchestration instructions
- References sub-skills for each stage
- Defines the `.research` package format
- Documents integration points

### 4.2 Sub-Skills: 10 Pipeline Stages

Each stage is a self-contained sub-skill that can be invoked independently or as part of the pipeline:

| Stage | Sub-skill | Purpose | Key Integration |
|-------|-----------|---------|-----------------|
| 1 | `stage-01-planner` | Decompose query into sub-questions | `zeespec-interrogator` for requirement extraction |
| 2 | `stage-02-search` | Web search + source discovery | `liter-llm-bridge` for cost-aware routing |
| 3 | `stage-03-retrieve` | Retrieve + chunk content | `kreuzberg` for document extraction |
| 4 | `stage-04-collect` | Collect + index sources | `surreal-memory` for graph storage |
| 5 | `stage-05-verify` | Verify source credibility | `sycophancy-correction` for bias detection |
| 6 | `stage-06-resolve` | Resolve contradictions | `pmpo-elicit` for human escalation |
| 7 | `stage-07-graph` | Build knowledge graph | `prometheus-entity-skills` for graph CRUD |
| 8 | `stage-08-cite` | Generate citations + confidence | `surreal-memory` for citation linking |
| 9 | `stage-09-report` | Synthesize report | `learn-grade` for quality assessment |
| 10 | `stage-10-export` | Export `.research` package | `learn-plan` for curriculum generation |

### 4.3 Native Agent: `prometheus-research`

Generated via `native-agent` skill. The binary:
- Embeds an Axum HTTP server
- Exposes MCP endpoints for tool integration
- Streams AG-UI/A2UI events for real-time UI updates
- Persists state to `surreal-memory`
- Supports long-running processes with checkpointing

---

## 5. Step-by-Step Implementation

### Phase 1: Create Directory Structure (15 minutes)

```bash
# Navigate to project root
cd /Users/gqadonis/Projects/prometheus/prometheus-skill-pack

# Create the research category and deep-research skill
mkdir -p skills/research/deep-research/{skills,scripts,references,templates,assets,hooks,agents}

# Create sub-skill directories for each stage
for stage in stage-01-planner stage-02-search stage-03-retrieve stage-04-collect stage-05-verify stage-06-resolve stage-07-graph stage-08-cite stage-09-report stage-10-export; do
  mkdir -p "skills/research/deep-research/skills/$stage"
done

# Verify structure
find skills/research/deep-research -type d | sort
```

### Phase 2: Create Parent SKILL.md (2 hours)

Copy the template below into `skills/research/deep-research/SKILL.md`. This is the primary skill file that the harness loads.

**Key decisions:**
- **Triggers:** Must include keywords that activate the skill across platforms
- **Allowed tools:** Must declare all tools the skill uses (file_system, code_interpreter, web_search, sequential_thinking, etc.)
- **Model routing:** Use `liter-llm-bridge` for cost-aware routing across stages
- **Sub-skills:** Reference each stage skill with relative paths

### Phase 3: Create skill.toml (30 minutes)

Create `skills/research/deep-research/skill.toml` with:
- Metadata (name, version, description, author)
- Dependencies (reference the integration points map above)
- Scripts (build, test, validate, install)
- MCP server configuration
- UI protocol bindings

### Phase 4: Create Sub-Skill SKILL.md Files (4 hours)

Each of the 10 stages needs its own `SKILL.md` with:
- YAML frontmatter (name, description, triggers, allowed-tools)
- Stage-specific instructions
- Input/output contracts
- Integration with parent skill
- Example usage

**Recommended approach:** Write all 10 in one session, using a consistent template. Each should be ~100-150 lines.

### Phase 5: Create Scripts (1 hour)

Create executable scripts in `skills/research/deep-research/scripts/`:

```bash
# Research runner (invokes the pipeline)
touch scripts/run-research.sh
chmod +x scripts/run-research.sh

# Package exporter (exports .research package)
touch scripts/export-package.sh
chmod +x scripts/export-package.sh

# Verification checker (checks source credibility)
touch scripts/verify-sources.sh
chmod +x scripts/verify-sources.sh

# Graph builder (builds knowledge graph from sources)
touch scripts/build-graph.sh
chmod +x scripts/build-graph.sh

# Contradiction detector
touch scripts/detect-contradictions.sh
chmod +x scripts/detect-contradictions.sh
```

### Phase 6: Create Templates (30 minutes)

Create templates in `skills/research/deep-research/templates/`:

```bash
# Research plan template
touch templates/research-plan.md

# Source evaluation rubric
touch templates/source-evaluation.md

# Contradiction resolution template
touch templates/contradiction-resolution.md

# Report template (OKF + Prometheus extensions)
touch templates/report-template.md

# .research package manifest template
touch templates/research-package-manifest.json
```

### Phase 7: Create References (1 hour)

Create reference documents in `skills/research/deep-research/references/`:

```bash
# Pipeline architecture
touch references/pipeline-architecture.md

# .research package format specification
touch references/research-package-format.md

# MCP server API reference
touch references/mcp-api-reference.md

# AG-UI event schema
touch references/ag-ui-event-schema.md

# Integration with surreal-memory
touch references/surreal-memory-integration.md

# Feynman learning integration
touch references/feynman-integration.md

# Google OKF alignment
touch references/okf-alignment.md

# Threaded research architecture
touch references/threaded-research.md

# Long-running process management
touch references/long-running-processes.md
```

### Phase 8: Create Hooks (30 minutes)

Create hooks in `skills/research/deep-research/hooks/`:

```bash
# Pre-research hook (validates prerequisites)
touch hooks/pre-research.sh
chmod +x hooks/pre-research.sh

# Post-research hook (caches results, updates knowledge graph)
touch hooks/post-research.sh
chmod +x hooks/post-research.sh

# On-contradiction hook (alerts, escalates)
touch hooks/on-contradiction.sh
chmod +x hooks/on-contradiction.sh

# On-completion hook (exports, notifies)
touch hooks/on-completion.sh
chmod +x hooks/on-completion.sh
```

### Phase 9: Create Agent Definitions (30 minutes)

Create harness-specific agent definitions in `skills/research/deep-research/agents/`:

```bash
# Claude Code agent definition
touch agents/claude.md

# Codex agent definition
touch agents/codex.md

# OpenCode agent definition
touch agents/opencode.md

# Cursor agent definition
touch agents/cursor.md

# Kimi agent definition
touch agents/kimi.md
```

### Phase 10: Testing & Validation (1 hour)

```bash
# Validate the skill structure
npm run validate:skill skills/research/deep-research

# Test with the doctor command
npm run doctor

# Install locally for testing
npm run install:project

# Test in Claude Code (after installation)
# Launch Claude Code, verify skill appears: /skills
# Test: /deep-research "What is the current state of quantum computing?"
```

### Phase 11: Integration with Existing Docs (30 minutes)

Link the new skill to the existing deep-research documentation:

1. Add a reference in `docs/deep-research/index.md` Section 13 (Integration with Existing Prometheus Stack)
2. Add the skill to `SKILLS.md` in the Skills Index (new "Research" category)
3. Update `README.md` with the new Research category

### Phase 12: Commit & Push (15 minutes)

```bash
cd /Users/gqadonis/Projects/prometheus/prometheus-skill-pack

# Add all new files
git add skills/research/

# Commit
git commit -m "feat: add deep-research skill with 10-stage pipeline

- New skill category: research/
- Parent skill: deep-research with SKILL.md, skill.toml
- 10 sub-skills: planner, search, retrieve, collect, verify,
  resolve, graph, cite, report, export
- Scripts: run-research, export-package, verify-sources,
  build-graph, detect-contradictions
- Templates: research-plan, source-evaluation, report (OKF-aligned)
- References: pipeline architecture, .research format, MCP API,
  AG-UI schema, surreal-memory integration, Feynman integration,
  OKF alignment, threaded research, long-running processes
- Hooks: pre-research, post-research, on-contradiction, on-completion
- Agent definitions: Claude, Codex, OpenCode, Cursor, Kimi
- Integrates with: kbd-process-orchestrator, liter-llm-bridge,
  surreal-memory, mcp-server, axum-patterns, htmx-alpine-lit,
  Feynman skills, sycophancy-correction, pmpo-elicit

See docs/deep-research/index.md for master specification."

# Push
git push origin main
```

---

## 6. File Structure & Templates

### 6.1 Parent SKILL.md Template

See the full template in the file that this playbook creates. Key sections:

- **YAML frontmatter** with triggers, allowed-tools, model routing
- **When to Use** — specific scenarios that activate the skill
- **Quick Start** — `/deep-research <query>` or `/research <query>`
- **10-Stage Pipeline** — overview with links to sub-skills
- **.research Package Format** — specification for the output format
- **Integration Guide** — how to use with existing Prometheus stack
- **Examples** — 3-5 example research queries with expected outputs
- **Common Issues** — troubleshooting section

### 6.2 Sub-Skill SKILL.md Template

Each sub-skill follows this structure:

```yaml
---
name: stage-01-planner
description: Decompose a research query into sub-questions, identify knowledge gaps, and create a structured research plan.
license: MIT
metadata:
  author: Prometheus AGS
  version: '1.0.0'
  category: research
  tags: [deep-research, planner, query-decomposition]
parent: skills/research/deep-research/SKILL.md
---

# Stage 1: Planner

## Purpose

Decompose the user's research query into a structured plan of sub-questions, identify knowledge gaps, and estimate the scope and resources needed.

## Input

- `query`: string — the user's research question
- `depth`: 'shallow' | 'deep' | 'exhaustive' — desired research depth
- `kb_ids`: string[] — optional knowledge base IDs to seed from

## Output

- `plan`: ResearchPlan — structured plan with sub-questions, stages, and estimated tokens
- `gaps`: KnowledgeGap[] — identified gaps that require external search

## Instructions

### Step 1: Query Analysis

Analyze the query to identify:
- Core concepts and entities
- Temporal scope (historical, current, future)
- Domain boundaries
- Implicit assumptions

### Step 2: Sub-question Decomposition

Break the query into 3-7 sub-questions that:
- Are independently answerable
- Cover the full scope of the original query
- Have clear verification criteria

### Step 3: Knowledge Gap Identification

For each sub-question, identify:
- What is already known (from KBs or prior research)
- What needs to be discovered
- Potential sources to consult

### Step 4: Plan Assembly

Assemble the research plan with:
- Stage sequence (which stages to run, in what order)
- Estimated token budget per stage
- Confidence thresholds for verification
- Contradiction handling strategy

## Example

```
Input: "What is the current state of quantum computing in 2026?"
Output: {
  plan: {
    subQuestions: [
      "What are the latest quantum computing hardware milestones in 2026?",
      "Which companies have achieved quantum advantage in practical applications?",
      "What are the current limitations and error rates of quantum systems?",
      "What quantum algorithms are seeing real-world deployment?"
    ],
    stages: [2, 3, 4, 5, 7, 8, 9, 10],
    estimatedTokens: 150000,
    confidenceThreshold: 0.85
  }
}
```

## Integration

This stage is invoked automatically by the parent `deep-research` skill. It can also be invoked independently via `/stage-01-planner <query>`.
```

### 6.3 skill.toml Template

```toml
[package]
name = "deep-research"
version = "1.0.0"
description = "Deep research agent with 10-stage pipeline, knowledge graphs, and persistent .research packages"
authors = ["Prometheus AGS"]
license = "MIT"
category = "research"
tags = ["research", "deep-research", "knowledge-graph", "mcp", "ag-ui"]

[dependencies]
"skills/process/kbd-process-orchestrator" = { required = true, version = ">=1.0.0" }
"skills/process/liter-llm-bridge" = { required = true, version = ">=1.0.0" }
"skills/rust/mcp-server" = { required = true, version = ">=1.0.0" }
"skills/rust/axum-patterns" = { required = true, version = ">=1.0.0" }
"skills/htmx/htmx-alpine-lit" = { required = false, version = ">=1.0.0" }
"tools/surreal-memory-server" = { required = false, version = ">=1.0.0" }

[scripts]
build = "bash scripts/build.sh"
test = "bash scripts/test.sh"
validate = "npm run validate:skill skills/research/deep-research"
run = "bash scripts/run-research.sh"
export = "bash scripts/export-package.sh"

[mcp]
server_name = "prometheus-research"
transport = "stdio"
# Server binary generated via native-agent skill

[ui]
protocol = "ag-ui"
# HTMX UI located at docs/deep-research/deep-research-ui.html
# Served by prometheus-research Axum server

[features]
threaded = true
long_running = true
feynman_integration = true
okf_alignment = true
knowledge_graph = true
contradiction_detection = true
```

---

## 7. MCP Server Integration

### 7.1 Generating the MCP Server

Use the `native-agent` skill to generate the `prometheus-research` binary:

```bash
# In Claude Code or any harness with native-agent skill:
/native-agent

# Specify:
# - Name: prometheus-research
# - Protocols: A2A, AG-UI, A2UI, MCP
# - Features: surreal-memory integration, liter-llm routing, threaded research
# - Frontend: HTMX + Alpine.js (reuse docs/deep-research/deep-research-ui.html)
```

The generated project structure:

```
prometheus-research/
├── Cargo.toml
├── agent.toml
├── system_prompt.md
├── Dockerfile
├── docker-compose.yml
├── crates/
│   ├── research-core/           # Domain types: ResearchJob, Source, Contradiction, etc.
│   ├── research-pipeline/       # 10-stage pipeline implementation
│   ├── research-mcp/            # MCP server: tools, resources, prompts
│   ├── research-agui/           # AG-UI/A2UI event streaming
│   ├── research-graph/          # Knowledge graph operations (surreal-memory client)
│   ├── research-threads/        # Threaded research with context isolation
│   ├── research-checkpoints/    # Long-running process checkpointing
│   └── research-cli/            # CLI: start, stop, status, export, list
└── frontend/                    # HTMX + Alpine.js (from docs/deep-research/)
```

### 7.2 MCP Tools

The MCP server exposes these tools:

| Tool | Description | Input | Output |
|------|-------------|-------|--------|
| `research_plan` | Create a research plan from a query | `query`, `depth`, `kb_ids` | `plan` |
| `research_search` | Execute web search for sources | `sub_questions`, `sources` | `results` |
| `research_retrieve` | Retrieve and chunk content from sources | `urls`, `source_type` | `chunks` |
| `research_collect` | Collect and index sources into graph | `chunks`, `job_id` | `indexed_sources` |
| `research_verify` | Verify source credibility | `source_ids`, `threshold` | `verified_sources` |
| `research_resolve` | Resolve contradictions | `contradictions`, `strategy` | `resolution` |
| `research_graph` | Build/update knowledge graph | `sources`, `relations` | `graph` |
| `research_cite` | Generate citations and confidence scores | `sources`, `claims` | `citations` |
| `research_report` | Synthesize report from sources | `sources`, `graph`, `template` | `report` |
| `research_export` | Export .research package | `job_id`, `format` | `package_path` |
| `research_status` | Get job status | `job_id` | `status` |
| `research_list` | List research jobs | `filter`, `limit` | `jobs` |
| `research_stop` | Stop a running job | `job_id` | `success` |
| `research_clone` | Clone a job with modifications | `job_id`, `modifications` | `new_job_id` |

### 7.3 MCP Resources

| Resource | URI | Description |
|----------|-----|-------------|
| `research://jobs/{id}` | `research://jobs/123` | Full job state |
| `research://jobs/{id}/sources` | `research://jobs/123/sources` | Source list |
| `research://jobs/{id}/graph` | `research://jobs/123/graph` | Knowledge graph |
| `research://jobs/{id}/contradictions` | `research://jobs/123/contradictions` | Contradictions |
| `research://jobs/{id}/report` | `research://jobs/123/report` | Generated report |
| `research://kbs` | `research://kbs` | Available knowledge bases |
| `research://templates` | `research://templates` | Report templates |

---

## 8. UI Protocol Integration

### 8.1 AG-UI Event Streaming

The `prometheus-research` server streams AG-UI events to the HTMX UI:

```json
{
  "type": "agent.status",
  "job_id": "job-123",
  "status": "running",
  "stage": 3,
  "stage_name": "retrieve",
  "progress": 35,
  "tokens": 45000,
  "cost": 0.75,
  "timestamp": "2026-07-04T10:30:00Z"
}
```

```json
{
  "type": "agent.message",
  "job_id": "job-123",
  "message": "Retrieved 12 chunks from 5 sources",
  "level": "info",
  "timestamp": "2026-07-04T10:30:05Z"
}
```

```json
{
  "type": "a2ui.media_card",
  "job_id": "job-123",
  "media": {
    "type": "audio",
    "title": "Recovered Fragment",
    "stream_url": "https://ipfs.prometheusags.ai/...",
    "confidence": "41%"
  }
}
```

### 8.2 HTMX UI Connection

The existing `docs/deep-research/deep-research-ui.html` connects to the server via:

```javascript
// SSE connection for AG-UI events
const eventSource = new EventSource('/api/v1/jobs/' + jobId + '/events');
eventSource.onmessage = (event) => {
  const data = JSON.parse(event.data);
  // Update Alpine.js state
  this.handleAguiEvent(data);
};
```

### 8.3 A2UI Components

A2UI components dispatched by the research agent:

- **`a2ui.media_card`** — Audio/video/image cards with confidence badges
- **`a2ui.graph_view`** — Interactive knowledge graph visualization
- **`a2ui.source_list`** — Source cards with scores and metadata
- **`a2ui.contradiction_panel`** — Side-by-side contradiction comparison
- **`a2ui.progress_ring`** — Circular progress with stage indicators

---

## 9. Testing & Validation

### 9.1 Skill Validation

```bash
# Validate structure
npm run validate:skill skills/research/deep-research

# Strict validation (required for new skills)
npm run validate:strict

# Full system health check
npm run doctor
```

### 9.2 Test Queries

Use these test queries to validate the skill across all stages:

1. **Simple (1-2 stages):** "What is the capital of France?" → should complete quickly
2. **Moderate (3-5 stages):** "What are the current trends in AI safety research?" → should hit search, retrieve, collect, graph, report
3. **Complex (all 10 stages):** "Analyze the competitive landscape of quantum computing in 2026, including hardware milestones, commercial applications, and key players. Identify contradictions between different sources." → should exercise the full pipeline

### 9.3 Integration Tests

> **Prescriptive, not descriptive.** The `scripts/test-*.sh` files below are part of what
> this playbook asks you to *build* — none of them exist in the repo today. For a working
> model-routing check right now, use `bash scripts/check-model-config.sh` and
> `bash skills/process/liter-llm-bridge/scripts/configure-models.sh verify`.

```bash
# Test surreal-memory integration
bash scripts/test-surreal-memory.sh

# Test liter-llm routing (see note above — use check-model-config.sh today)
bash scripts/test-liter-llm.sh

# Test MCP server
bash scripts/test-mcp-server.sh

# Test AG-UI streaming
bash scripts/test-agui-stream.sh

# Test .research package export
bash scripts/test-export.sh
```

### 9.4 Performance Benchmarks

| Metric | Target | Measurement |
|--------|--------|-------------|
| Pipeline latency (simple) | <30s | End-to-end for simple query |
| Pipeline latency (complex) | <5min | End-to-end for complex query |
| Token efficiency | <150K tokens | For deep research query |
| Source verification rate | >90% | Percentage of sources verified |
| Contradiction detection | >80% | Recall for contradictions |
| Graph coverage | >95% | Entities linked in knowledge graph |
| Report quality (Feynman grade) | >B+ | `learn-grade` assessment |

---

## 10. Platform Distribution

### 10.1 Installation Methods

```bash
# Method 1: Flat installer (all platforms)
bash scripts/install-skills-flat.sh
# Installs to ~/.<platform>/skills/ automatically

# Method 2: Platform-specific
npm run install:platforms -- --platform claude
npm run install:platforms -- --platform codex
npm run install:platforms -- --platform opencode
npm run install:platforms -- --platform kimi
npm run install:platforms -- --platform cursor

# Method 3: Manual (for development)
npm run install:project
```

### 10.2 Platform-Specific Configuration

**Claude Code:**
- Skill appears as `/deep-research` or `/research`
- Uses `agents/claude.md` for agent behavior
- Integrates with Claude's built-in web search

**Codex:**
- Skill appears as `/deep-research`
- Uses `agents/codex.md` for agent behavior
- Leverages Codex's tool use capabilities

**OpenCode:**
- Skill appears in OpenCode's skill registry
- Uses `agents/opencode.md` for agent behavior
- Integrates with OpenCode's plugin system

**Kimi:**
- Skill loads from `~/.kimi-code/skills/`
- Uses `agents/kimi.md` for agent behavior
- MCP servers registered in `~/.kimi/mcp/mcp.json`

**Cursor:**
- Skill appears in Cursor's AI rules
- Uses `agents/cursor.md` for agent behavior
- Integrates with Cursor's composer

### 10.3 MCP Server Registration

```bash
# After generating prometheus-research binary:
# Claude Code
claude config mcpServers.prometheus-research "{\"command\":\"prometheus-research\",\"args\":[\"mcp\"]}"

# Codex
codex config mcpServers.prometheus-research "{\"command\":\"prometheus-research\",\"args\":[\"mcp\"]}"

# OpenCode
# Add to .opencode/mcp.json

# Kimi
# Add to ~/.kimi/mcp/mcp.json
```

---

## 11. References

### 11.1 Internal Documents

| Document | Path | Purpose |
|----------|------|---------|
| Master Specification | `docs/deep-research/index.md` | Full architecture, pipeline, and feature spec |
| UI Prototype | `docs/deep-research/deep-research-ui.html` | HTMX + Alpine.js working prototype |
| Fable 5 Comparison | `docs/deep-research/fable5-comparison.md` | UI design analysis and recommendations |
| Implementation Plan | `docs/deep-research/plan.md` | Multi-tenancy redesign plan |
| Research Reports | `docs/deep-research/research/` | 8 detailed research reports |
| Skill Template | `docs/SKILL_TEMPLATE.md` | Standard skill format |
| Importing Skills | `docs/IMPORTING_SKILLS.md` | Submodule and import guide |
| Skill Pack Index | `SKILLS.md` | All existing skills and categories |

### 11.2 External Standards

| Standard | URL | How We Use It |
|----------|-----|---------------|
| Google OKF v0.1 | https://github.com/google/open-knowledge-format | Base format for `.research` packages |
| AgentSkills.io | https://agentskills.io/specification | Skill format compliance |
| MCP Protocol | https://modelcontextprotocol.io | Server and client protocol |
| AG-UI Protocol | https://ag-ui.com | UI event streaming |
| A2UI Protocol | https://a2ui.dev | Component dispatch |
| A2A Protocol | https://a2a.dev | Agent-to-agent communication |

### 11.3 Related Skills (Dependencies)

| Skill | Path | Role in Pipeline |
|-------|------|-----------------|
| kbd-process-orchestrator | `skills/process/kbd-process-orchestrator/` | Process lifecycle management |
| liter-llm-bridge | `skills/process/liter-llm-bridge/` | Cost-aware model routing |
| mcp-server | `skills/rust/mcp-server/` | MCP server scaffolding |
| axum-patterns | `skills/rust/axum-patterns/` | HTTP server patterns |
| native-agent | `skills/process/native-agent/` | Generate prometheus-research binary |
| htmx-alpine-lit | `skills/htmx/htmx-alpine-lit/` | UI patterns and components |
| prometheus-entity-skills | `skills/react/prometheus-entity-skills/` | Graph CRUD operations |
| sycophancy-correction | `skills/imported/sycophancy-correction/` | Bias detection and correction |
| pmpo-elicit | `skills/process/pmpo-elicit/` | Human escalation primitive |
| zeespec-interrogator | `skills/process/zeespec-interrogator/` | Requirement extraction |
| learn-plan | `skills/learn/learn-plan/` | Curriculum generation from research |
| learn-grade | `skills/learn/learn-grade/` | Quality assessment |
| learn-survey | `skills/learn/learn-survey/` | Knowledge gap identification |
| learn-kb | `skills/learn/learn-kb/` | Knowledge base management |

---

## Appendix A: Complete Directory Structure

```
skills/research/
└── deep-research/
    ├── SKILL.md                          # Parent skill entry point
    ├── skill.toml                        # Metadata, dependencies, scripts
    ├── AGENTS.md                         # Agent behavior rules
    ├── README.md                         # Skill documentation
    │
    ├── skills/                           # 10 pipeline stage sub-skills
    │   ├── stage-01-planner/
    │   │   └── SKILL.md
    │   ├── stage-02-search/
    │   │   └── SKILL.md
    │   ├── stage-03-retrieve/
    │   │   └── SKILL.md
    │   ├── stage-04-collect/
    │   │   └── SKILL.md
    │   ├── stage-05-verify/
    │   │   └── SKILL.md
    │   ├── stage-06-resolve/
    │   │   └── SKILL.md
    │   ├── stage-07-graph/
    │   │   └── SKILL.md
    │   ├── stage-08-cite/
    │   │   └── SKILL.md
    │   ├── stage-09-report/
    │   │   └── SKILL.md
    │   └── stage-10-export/
    │       └── SKILL.md
    │
    ├── scripts/                          # Automation scripts
    │   ├── run-research.sh               # Main pipeline runner
    │   ├── export-package.sh             # .research package exporter
    │   ├── verify-sources.sh             # Source credibility checker
    │   ├── build-graph.sh                # Knowledge graph builder
    │   ├── detect-contradictions.sh      # Contradiction detector
    │   ├── test-surreal-memory.sh        # surreal-memory integration test
    │   ├── test-liter-llm.sh             # liter-llm routing test
    │   ├── test-mcp-server.sh            # MCP server test
    │   ├── test-agui-stream.sh           # AG-UI streaming test
    │   └── test-export.sh                # Export format test
    │
    ├── templates/                        # Code and document templates
    │   ├── research-plan.md              # Research plan template
    │   ├── source-evaluation.md          # Source evaluation rubric
    │   ├── contradiction-resolution.md   # Contradiction resolution template
    │   ├── report-template.md            # Report template (OKF-aligned)
    │   └── research-package-manifest.json # .research package manifest
    │
    ├── references/                       # Detailed reference docs
    │   ├── pipeline-architecture.md      # Pipeline architecture deep dive
    │   ├── research-package-format.md    # .research format specification
    │   ├── mcp-api-reference.md          # MCP server API reference
    │   ├── ag-ui-event-schema.md         # AG-UI event schema
    │   ├── surreal-memory-integration.md # surreal-memory integration guide
    │   ├── feynman-integration.md          # Feynman learning integration
    │   ├── okf-alignment.md              # Google OKF alignment guide
    │   ├── threaded-research.md          # Threaded research architecture
    │   └── long-running-processes.md     # Long-running process management
    │
    ├── hooks/                            # Lifecycle hooks
    │   ├── pre-research.sh               # Pre-research validation
    │   ├── post-research.sh              # Post-research caching
    │   ├── on-contradiction.sh           # Contradiction alert/escalation
    │   └── on-completion.sh              # Completion export/notification
    │
    ├── agents/                           # Harness-specific agent definitions
    │   ├── claude.md                     # Claude Code agent rules
    │   ├── codex.md                      # Codex agent rules
    │   ├── opencode.md                   # OpenCode agent rules
    │   ├── cursor.md                     # Cursor agent rules
    │   └── kimi.md                       # Kimi agent rules
    │
    └── assets/                           # Images, diagrams, icons
        └── prometheus-research-logo.svg
```

---

## Appendix B: Checklist

Use this checklist to track implementation progress:

### Phase 1: Structure
- [ ] Create `skills/research/deep-research/` directory
- [ ] Create all subdirectories (skills, scripts, references, templates, assets, hooks, agents)
- [ ] Create 10 stage sub-skill directories

### Phase 2: Parent Skill
- [ ] Write `SKILL.md` with YAML frontmatter
- [ ] Define triggers and keywords
- [ ] Document 10-stage pipeline overview
- [ ] Document .research package format
- [ ] Write integration guide
- [ ] Add 3-5 examples
- [ ] Add troubleshooting section

### Phase 3: Configuration
- [ ] Write `skill.toml` with metadata
- [ ] Define all dependencies
- [ ] Configure MCP server bindings
- [ ] Configure UI protocol bindings
- [ ] Add feature flags

### Phase 4: Sub-Skills
- [ ] Stage 1: Planner
- [ ] Stage 2: Search
- [ ] Stage 3: Retrieve
- [ ] Stage 4: Collect
- [ ] Stage 5: Verify
- [ ] Stage 6: Resolve
- [ ] Stage 7: Graph
- [ ] Stage 8: Cite
- [ ] Stage 9: Report
- [ ] Stage 10: Export

### Phase 5: Scripts
- [ ] run-research.sh
- [ ] export-package.sh
- [ ] verify-sources.sh
- [ ] build-graph.sh
- [ ] detect-contradictions.sh
- [ ] All test scripts

### Phase 6: Templates
- [ ] research-plan.md
- [ ] source-evaluation.md
- [ ] contradiction-resolution.md
- [ ] report-template.md
- [ ] research-package-manifest.json

### Phase 7: References
- [ ] pipeline-architecture.md
- [ ] research-package-format.md
- [ ] mcp-api-reference.md
- [ ] ag-ui-event-schema.md
- [ ] surreal-memory-integration.md
- [ ] feynman-integration.md
- [ ] okf-alignment.md
- [ ] threaded-research.md
- [ ] long-running-processes.md

### Phase 8: Hooks
- [ ] pre-research.sh
- [ ] post-research.sh
- [ ] on-contradiction.sh
- [ ] on-completion.sh

### Phase 9: Agents
- [ ] claude.md
- [ ] codex.md
- [ ] opencode.md
- [ ] cursor.md
- [ ] kimi.md

### Phase 10: Testing
- [ ] Validate skill structure
- [ ] Run doctor command
- [ ] Test simple query
- [ ] Test moderate query
- [ ] Test complex query
- [ ] Test surreal-memory integration
- [ ] Test liter-llm routing
- [ ] Test MCP server
- [ ] Test AG-UI streaming
- [ ] Test export format

### Phase 11: Documentation
- [ ] Update `SKILLS.md` with new Research category
- [ ] Update `README.md` with Research category
- [ ] Link from `docs/deep-research/index.md`
- [ ] Write skill README.md
- [ ] Write AGENTS.md

### Phase 12: Commit
- [ ] Add all files to git
- [ ] Write commit message
- [ ] Push to origin/main
- [ ] Verify CI passes
- [ ] Test on target platforms
