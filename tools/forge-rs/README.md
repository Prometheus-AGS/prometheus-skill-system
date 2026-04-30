# forge-rs

Code-generation enrichment engine for the Prometheus AGS skill pipeline. Sits at
**Layer 4** — the lowest level of the skill stack — between OpenSpec task output
and the AI agent that implements the code.

## Core Loop

```
OpenSpec task folder
    │
    ▼ forge enrich <task-path>
    │
    ├── 1. Read tasks.md → detect language
    ├── 2. SkillRegistry.resolve(language, description)
    │       → applicable skills from skills/<lang>/<skill>/skill.toml
    ├── 3. ConstitutionChecker
    │       → flags forbidden patterns from .forge/constitution/<lang>.toml
    ├── 4. KarpathyFocus
    │       → pk focus "<topic>" via prometheus-knowledge (MCP or CLI)
    │       → injects prior learned context into the enrichment
    ├── 5. Tera template rendering
    │       → renders all template/*.tera files for resolved skills
    └── 6. Write .forge/enriched/<task-id>.context.md
              ↓ AI agent reads this and implements the code

    ▼ forge reflect <iteration-id>
    │
    ├── Read agent-produced code vs user-accepted diff
    ├── Compute skill drift (acceptance rate per skill/template)
    ├── Update .forge/memory/drift/<lang>-YYYYMMDD.json
    └── pk ingest "forge:reflect:<task-id>" → Karpathy learning loop
```

## Workspace Crates

| Crate | Role |
|---|---|
| `forge-core` | Domain types: `Constitution`, `SkillManifest`, `EnrichmentContext`, `IterationRecord`, `DriftReport` |
| `forge-skills` | Skill registry: discovery, resolution, Tera rendering, dependency ordering |
| `forge-enricher` | Enrichment pipeline: task reading, language detection, `pk focus`, context generation |
| `forge-reflect` | Drift computation, `pk ingest`, iteration archival |
| `forge-mcp` | Axum SSE MCP server: 4 tools (`forge_enrich`, `forge_reflect`, `forge_drift`, `forge_validate`) |
| `forge-cli` | `forge` binary: 8 top-level commands |

## External Tool Dependencies

| Tool | Source | Role |
|---|---|---|
| `prometheus-knowledge` | `tools/prometheus-knowledge/` (git submodule) | `pk focus` pulls Karpathy context; `pk ingest` writes reflection |
| `liter-llm` | `tools/liter-llm/` (git submodule) | Multi-model routing; cheap models for reflection, frontier for enrichment quality gates |

## CLI Reference

```bash
# Core workflow
forge init                                      # scaffold .forge/ in current project
forge enrich <task-path>                        # enrich an OpenSpec task
forge reflect <iteration-id>                    # process iteration into Karpathy loop
forge drift [--language rust]                   # report stale skill candidates
forge validate <file> --language rust           # check against constitution

# Server
forge mcp [--port 8943]                         # start MCP server

# Skill management
forge skill list [--language rust]              # list available skills
forge skill add <name>                          # pull skill from registry
forge skill sync                                # sync skills from skill pack root

# Constitution management
forge constitution rust                         # open Rust constitution in $EDITOR

# Template management (see Template System below)
forge template new skill <lang> <name>          # scaffold a new skill
forge template new template <skill-path> <name> # add template to existing skill
forge template render <skill-path>/<tmpl> [vars]# test-render a template
forge template list [--language rust]           # list all templates
forge template validate <skill-path>            # check Tera syntax
forge template edit <skill-path>/<tmpl>         # open template in $EDITOR
```

## MCP Server

Port `8943` by default. Add to Claude Desktop or any MCP-compatible host:

```json
{
  "name": "forge",
  "url": "http://localhost:8943/mcp",
  "transport": "sse"
}
```

| Tool | Input | Output |
|---|---|---|
| `forge_enrich` | `{ "task_path": "..." }` | Path to enriched context document |
| `forge_reflect` | `{ "iteration_id": "..." }` | Drift summary, Karpathy ingestion status |
| `forge_drift` | `{ "language": "..." }` | Stale skill candidates |
| `forge_validate` | `{ "content": "...", "language": "..." }` | Constitution violation report |

---

## Template System

Templates are **Tera** (`.tera`) files that forge-rs renders with task context and
injects into the enriched context document consumed by the agent.

### Template Location

```
skills/
  <language>/
    <skill-name>/
      skill.toml          ← machine manifest: lists templates, triggers, deps
      SKILL.md            ← human documentation
      templates/
        *.tera            ← Tera templates — rendered by forge enrich
```

### Variables Available in Every Template

| Variable | Value |
|---|---|
| `{{ "{{" }} task_description {{ "}}" }}` | Full content of `tasks.md` |
| `{{ "{{" }} task_id {{ "}}" }}` | Change ID (e.g. `CHANGE-042`) |
| `{{ "{{" }} task_path {{ "}}" }}` | Path to the task folder |
| `{{ "{{" }} acceptance_criteria {{ "}}" }}` | GIVEN/WHEN/THEN blocks |
| `{{ "{{" }} constitution_summary {{ "}}" }}` | Active language constitution |
| `{{ "{{" }} karpathy_focus {{ "}}" }}` | `pk focus` output (may be empty) |

### Project-Level Overrides

Override any skill pack template at the project level:

```
.forge/skills/<language>/<skill-name>/templates/<name>.tera
```

Project-local templates take priority over skill pack templates. This is how you
fit a template to a project's specific conventions without forking the skill.

---

## Meta-Template System

forge-rs includes **meta-templates** — templates that generate other templates.
They live in `tools/forge-rs/templates/meta/` and are invoked via `forge template`.

```
templates/meta/
  new_skill_toml.tera         ← generates skill.toml for a new skill
  new_skill_md.tera           ← generates SKILL.md for a new skill
  new_tera_template.tera      ← generates a new .tera template file
  new_constitution_toml.tera  ← generates a new language constitution
```

### Generating a New Skill

```bash
forge template new skill rust my-axum-extension
# Creates:
#   skills/rust/my-axum-extension/SKILL.md       (from new_skill_md.tera)
#   skills/rust/my-axum-extension/skill.toml     (from new_skill_toml.tera)
#   skills/rust/my-axum-extension/templates/     (empty — ready for templates)

# Add your first template
forge template new template skills/rust/my-axum-extension/ ws_handler.rs
# Creates: skills/rust/my-axum-extension/templates/ws_handler.rs.tera
# Opens in $EDITOR with the meta-template header pre-populated with variable docs

# Validate Tera syntax
forge template validate skills/rust/my-axum-extension/

# Test-render with sample variables
forge template render skills/rust/my-axum-extension/ws_handler.rs.tera \
  --var handler_name=ws_inference \
  --var state_type=AppState
```

### The Meta-Template Feedback Loop

```
1. forge template new template <skill-path> <name>
        │ meta-template generates .tera file with header + TODO
        ▼
2. $EDITOR skills/<lang>/<skill>/templates/<name>.tera
        │ developer fills in Tera code
        ▼
3. forge template validate <skill-path>
        │ checks Tera syntax, reports variable mismatches
        ▼
4. forge enrich <task-path>
        │ forge picks up the new template automatically
        │ renders it into .forge/enriched/<task>.context.md
        ▼
5. agent uses the enriched context to implement code
        ▼
6. forge reflect <task-id>
        │ measures template acceptance rate
        │ if acceptance < 50% → template flagged as stale candidate
        │ drift written to .forge/memory/drift/
        │ ingested to prometheus-knowledge via pk ingest
        ▼
7. review forge drift --language <lang>
        │ see which templates are being overridden
        │ edit template to match actual patterns
        └── go to step 2
```

---

## .forge/ Directory Layout

```
.forge/
  constitution/
    rust.toml             ← Rust standards + forbidden patterns + required skills
    typescript.toml       ← TypeScript/React standards
    flutter.toml          ← Flutter + Riverpod standards
    go.toml               ← Go standards
    python.toml           ← Python + PyO3 standards
    tauri.toml            ← Tauri standards
  enriched/
    <task-id>.context.md  ← enriched context consumed by agent
  memory/
    iterations/
      <task-id>.json      ← completed iteration records
    drift/
      rust-YYYYMMDD.json  ← skill drift summaries (stale candidate detection)
  skills/
    <language>/           ← project-local skill overrides (take priority over skill pack)
```

---

## Language Support

| Language | Constitution | Skills | Templates |
|---|---|---|---|
| Rust | `rust.toml` | `axum-patterns`, `error-handling`, `async-patterns`, `workspace-structure`, `mcp-server`, `actor-model`, `performance` | router, app_error, app_state, middleware, handler |
| TypeScript | `typescript.toml` | `base-patterns` | — |
| React 19 | `typescript.toml` | `react-vite-stack`, `prometheus-entity-skills` | page_component, feature_hook, store, api_client, entity_hook |
| Flutter | `flutter.toml` | `flutter-rust-ffi` | riverpod_notifier, feature_repository, go_router_config |
| HTMX | — | `htmx-alpine-lit` | page, lit_component, react_island, axum_fragment_handler |
| Tauri | `tauri.toml` | `tauri-react-vite` | — |
| Go | `go.toml` | `base-patterns` | — |
| Python | `python.toml` | `pyo3-bridge` | — |

---

## Environment Variables

| Var | Purpose | Default |
|---|---|---|
| `PK_MCP_URL` | prometheus-knowledge MCP endpoint | Falls back to `pk` CLI subprocess |
| `LITER_LLM_URL` | liter-llm endpoint for model routing | Direct API calls |
| `FORGE_SKILLS_ROOT` | Override skill pack root detection | Auto-detected by walking up from CWD |
| `ZEESPEC_STATE_DIR` | ZeeSpec state directory | `.zeespec/` |
| `EDITOR` | Editor for `forge constitution` / `forge template edit` | `vim` |

---

## Development Guide

See [CLAUDE.md](CLAUDE.md) for full development instructions.

### Quick Commands

```bash
cargo build --workspace              # build all crates
cargo build --release -p forge-cli  # release forge binary
cargo test --workspace               # all tests
cargo clippy --workspace -- -D warnings
cargo fmt --all
```

### Adding a New Language

1. Create `constitution-templates/<language>.toml`
2. Add `Language` variant to `forge-core/src/lib.rs`
3. Add detection logic to `forge-enricher/src/lib.rs → detect_language()`
4. Add `include_str!` branch in `forge-cli/src/main.rs → scaffold_constitution()`
5. Create `skills/<language>/` in the skill pack with at least one skill + `skill.toml`

### Adding a New Template to an Existing Skill

```bash
forge template new template skills/<lang>/<skill>/ <name>
# or manually:
touch skills/<lang>/<skill>/templates/<name>.tera
# Add the template to skill.toml [[templates]] section
```
