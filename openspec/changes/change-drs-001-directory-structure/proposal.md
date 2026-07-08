---
id: change-drs-001-directory-structure
title: Create skills/research/deep-research/ directory tree + skill.toml
phase: phase-deep-research-skill
priority: P0
effort: S
wave: 1
agent: general-purpose
status: pending
gap_id: G-01
verdict: BUILD
scope:
  - skills/research/deep-research/
  - skills/research/deep-research/skills/stage-01-planner/
  - skills/research/deep-research/skills/stage-02-search/
  - skills/research/deep-research/skills/stage-03-retrieve/
  - skills/research/deep-research/skills/stage-04-collect/
  - skills/research/deep-research/skills/stage-05-verify/
  - skills/research/deep-research/skills/stage-06-resolve/
  - skills/research/deep-research/skills/stage-07-graph/
  - skills/research/deep-research/skills/stage-08-cite/
  - skills/research/deep-research/skills/stage-09-report/
  - skills/research/deep-research/skills/stage-10-export/
  - skills/research/deep-research/scripts/
  - skills/research/deep-research/references/
  - skills/research/deep-research/templates/
  - skills/research/deep-research/hooks/
  - skills/research/deep-research/agents/
  - skills/research/deep-research/skill.toml
---

# change-drs-001 — Directory structure + skill.toml

## Context

No `skills/research/` category exists. This change creates the entire directory
tree and the `skill.toml` metadata + dependency file. All subsequent changes
depend on this structure existing.

## Strategy

1. Create all directories with `mkdir -p`
2. Write `skill.toml` with package metadata, dependencies, scripts, MCP config,
   UI config, and feature flags
3. Verify directory structure with `find`

## Scope

Create these directories:
```
skills/research/
└── deep-research/
    ├── skills/
    │   ├── stage-01-planner/
    │   ├── stage-02-search/
    │   ├── stage-03-retrieve/
    │   ├── stage-04-collect/
    │   ├── stage-05-verify/
    │   ├── stage-06-resolve/
    │   ├── stage-07-graph/
    │   ├── stage-08-cite/
    │   ├── stage-09-report/
    │   └── stage-10-export/
    ├── scripts/
    ├── references/
    ├── templates/
    ├── hooks/
    └── agents/
```

Create `skills/research/deep-research/skill.toml`:
```toml
[package]
name = "deep-research"
version = "1.0.0"
description = "Deep research agent with 10-stage pipeline, knowledge graphs, and persistent .research packages"
authors = ["Prometheus AGS"]
license = "MIT"
category = "research"
tags = ["research", "deep-research", "knowledge-graph", "mcp", "ag-ui", "okf"]

[dependencies]
"skills/process/kbd-process-orchestrator" = { required = true, version = ">=1.0.0" }
"skills/process/liter-llm-bridge" = { required = true, version = ">=1.0.0" }
"skills/process/pmpo-elicit" = { required = false, version = ">=1.0.0" }
"skills/rust/mcp-server" = { required = false, version = ">=1.0.0" }
"skills/rust/axum-patterns" = { required = false, version = ">=1.0.0" }
"skills/htmx/htmx-alpine-lit" = { required = false, version = ">=1.0.0" }
"tools/surreal-memory-server" = { required = false, version = ">=1.0.0" }
"skills/imported/sycophancy-correction" = { required = false, version = ">=1.0.0" }
"skills/learn/learn-grade" = { required = false, version = ">=1.0.0" }
"skills/learn/learn-plan" = { required = false, version = ">=1.0.0" }
"skills/document-extraction/kreuzberg" = { required = false, version = ">=1.0.0" }

[scripts]
validate = "npm run validate:skill skills/research/deep-research"
run = "bash scripts/run-research.sh"
export = "bash scripts/export-package.sh"

[mcp]
server_name = "prometheus-research"
transport = "stdio"
# Server binary generated in phase-prometheus-research-binary (deferred)

[ui]
protocol = "ag-ui"
# HTMX UI at docs/deep-research/deep-research-ui.html

[features]
threaded = true
long_running = true
feynman_integration = true
okf_alignment = true
knowledge_graph = true
contradiction_detection = true
```

## Acceptance Criteria

- [ ] `find skills/research/deep-research -type d | sort` shows all 16 directories
- [ ] `skills/research/deep-research/skill.toml` exists and is valid TOML
- [ ] `cat skills/research/deep-research/skill.toml | grep 'name = "deep-research"'` matches
