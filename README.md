# Prometheus Skill Pack

> 📚 **Full documentation:** <https://prometheus-ags.github.io/prometheus-skill-system/> (Docusaurus site — guide, learn domain, sovereign sync)

A self-improving AI skill execution engine. Production-grade skills across 8 language
domains, a 4-layer PMPO orchestration pipeline, a Karpathy knowledge learning loop,
a code-generation enrichment engine (forge-rs), a native agent generator, and
Cedar-governed self-optimization.

Built for teams deploying AI agents in production where capability improvement must be
governed, audited, and reproducible.

[![Docs site](https://github.com/Prometheus-AGS/prometheus-skill-system/actions/workflows/docs-pages.yml/badge.svg?branch=main)](https://github.com/Prometheus-AGS/prometheus-skill-system/actions/workflows/docs-pages.yml)

> **Readiness is evidence-scoped, not a percentage.** The repository distinguishes
> locally certified artifacts, disposable runtime tests, installed-service state,
> and external deployment evidence. A green artifact test does not claim that a
> service is installed or externally operated. See
> [the readiness evidence table](docs/production-readiness-report.md).

---

## 📚 Documentation

The complete, official product documentation lives in **[`docs/guide/`](docs/guide/README.md)** — 24 linked
pages covering every skill, tool, CLI, MCP server, hook, and script individually and collectively, with
flow, sequence, and C4 diagrams throughout. This README is the quick tour; the guide is the manual.

| If you want to… | Read |
|---|---|
| Understand the whole system | [Guide index](docs/guide/README.md) · [Introduction](docs/guide/01-introduction.md) |
| Learn the methodology | [Metaprompting, PMPO & KBD](docs/guide/02-metaprompting-pmpo-kbd.md) |
| Understand the loops | [Loop Architecture](docs/guide/03-loop-architecture.md) · [Four-Layer Pipeline](docs/guide/04-four-layer-pipeline.md) |
| Know the substrate | [MCP Servers](docs/guide/05-mcp-substrate.md) · [Memory & Learning](docs/guide/06-memory-and-learning.md) · [Sycophancy Correction](docs/guide/07-sycophancy-correction.md) |
| Browse every skill | [Skills Overview](docs/guide/08-skills-overview.md) · [Process Skills](docs/guide/09-process-skills.md) · [Language Skills](docs/guide/10-language-skills.md) · [Artifact Refiner](docs/guide/11-artifact-refiner.md) · [Native Agent Generator](docs/guide/12-native-agent-generator.md) |
| Reference the engine room | [Tools](docs/guide/13-tools-reference.md) · [Rust Toolchain](docs/guide/14-rust-toolchain.md) · [Hooks & Lifecycle](docs/guide/15-hooks-and-lifecycle.md) · [CLI & Scripts](docs/guide/16-cli-and-scripts.md) |
| Install, run, contribute | [Platform Support](docs/guide/17-platform-support.md) · [Plugins & Marketplace](docs/guide/18-plugins-and-marketplace.md) · [Installation](docs/guide/19-installation.md) · [Updating](docs/guide/20-updating.md) · [Contributing](docs/guide/21-contributing.md) |
| See why it matters | [Advantages & Impact](docs/guide/22-advantages-and-impact.md) · [Glossary & Sources](docs/guide/23-glossary.md) |

The design posture behind it all — *stop prompting, start designing loops* — is laid out in the companion
essay [docs/articles/autonomous-loops-prometheus-skill-pack.md](docs/articles/autonomous-loops-prometheus-skill-pack.md).

---

## The 4-Layer Pipeline

Every piece of work flows through four layers. Each layer feeds the next.

```
┌─────────────────────────────────────────────────────────────────┐
│  LAYER 1: ZeeSpec Interrogator                                  │
│  Zachman Framework 5W1H — 60 questions across 6 dimensions     │
│  GO / CAUTION / NO-GO constraint manifest                       │
│  skills/process/zeespec-interrogator/                           │
└─────────────────────────┬───────────────────────────────────────┘
                          │ constraint manifest
                          ▼
┌─────────────────────────────────────────────────────────────────┐
│  LAYER 2: PMPO Orchestration                                    │
│  pmpo-evolver (strategy router) + iterative-evolver (strategic) │
│  + kbd-process-orchestrator (tactical KBD loop)                 │
│  Assess → Analyze → Plan → Execute → Reflect                    │
│  Named cross-session state · surreal-memory · Cedar governance  │
└─────────────────────────┬───────────────────────────────────────┘
                          │ task manifests
                          ▼
┌─────────────────────────────────────────────────────────────────┐
│  LAYER 3: OpenSpec Change Management                            │
│  Per-change proposals · GIVEN/WHEN/THEN acceptance criteria     │
│  Audit trail · Change-scoped documentation                      │
│  tools/liter-llm — per-phase model routing                      │
└─────────────────────────┬───────────────────────────────────────┘
                          │ enriched implementation context
                          ▼
┌─────────────────────────────────────────────────────────────────┐
│  LAYER 4: forge-rs (Code Enrichment Engine)                     │
│  Language detection → skill resolution → constitution check     │
│  committed prompt snapshot → Tera template rendering            │
│  → .forge/enriched/<task>.context.md → AI agent implements      │
│  → forge reflect → pk ingest (Karpathy learning loop)           │
│  tools/forge-rs/ · tools/prometheus-knowledge/                  │
└─────────────────────────────────────────────────────────────────┘
```

---

## Native Agent Generator

The skill pack includes `/create-native-agent` — a project scaffold that generates
a complete, production-ready Rust agent binary in one command.

```
/create-native-agent
→ prompts for name, description, provider, port
→ generates a complete Rust workspace + React 19 frontend
→ validates with cargo check + npm install
→ ready to run
```

### What the Generated Agent Provides

```
my-agent start [--port 8080] [--background]   # Supabase-style management CLI
my-agent stop / status / logs
my-agent mcp add forge http://localhost:8943/mcp
my-agent skills list / reload
my-agent providers list / set-default anthropic
my-agent models set-default claude-haiku-4-5
```

| Feature | Details |
|---|---|
| **A2A protocol** | Agent card at `/.well-known/agent.json`, task endpoint at `/a2a/tasks` |
| **AG-UI protocol** | SSE stream at `/agui/events/:run_id` with `agui.*` events (CopilotKit compatible) |
| **A2UI protocol** | Prometheus combined protocol at `/a2ui/session` |
| **Chat API** | OpenAI-compatible at `/api/chat` |
| **React 19 frontend** | `assistant-ui` Thread with AG-UI SSE streaming, provider/model switcher |
| **MCP client** | Connects to configured MCP servers (forge-rs, surreal-memory, pk, custom) |
| **Skills engine** | TF-IDF selection from configured skill directories, hot-reloadable |
| **liter-llm routing** | All model calls go through liter-llm for multi-provider support |

### Agent Networks

Multiple generated agents can form a network by pointing at each other's A2A endpoints:

```
research-agent (:8081) ←──── A2A ────→ forge-agent (:8080)
       ↓                                      ↓
  surreal-memory (:23001)              prometheus-knowledge (:8942)
```

See `skills/process/native-agent/references/protocols.md` for the full protocol spec.

---

## Repository Structure

```
prometheus-skill-pack/
├── skills/                      ← All skill manifests + Tera templates
│   ├── process/                 ← Orchestration skills (PMPO pipeline)
│   │   ├── native-agent/            ← Native agent generator (/create-native-agent)
│   │   ├── zeespec-interrogator/    ← Layer 1: constraint interrogation
│   │   ├── iterative-evolver/       ← Layer 2: strategic PMPO loop
│   │   ├── pmpo-evolver/            ← Layer 2: strategy router (5 perspectives + Darwin idea gate)
│   │   │   └── skills/validate-idea/   ← Three-gate idea validation sub-skill
│   │   ├── kbd-process-orchestrator/ ← Layer 2: tactical KBD loop
│   │   ├── pmpo-outer-loop/         ← Layer 3: standing loop (perspective-aware loop-tick)
│   │   ├── pmpo-elicit/             ← Elicitation primitive with provenance
│   │   ├── pmpo-skill-creator/      ← Skill generation via PMPO
│   │   ├── kbd-goal/                ← Goal definition with success criteria + cross-tool parity
│   │   ├── kbd-goal-check/          ← Goal progress check and milestone verification
│   │   └── liter-llm-bridge/        ← Multi-model routing bridge + model-discovery reference
│   ├── rust/                    ← Rust language skills + Tera templates
│   ├── react/                   ← React 19 skills + entity-management
│   ├── flutter/                 ← Flutter + Rust FFI skills
│   ├── tauri/                   ← Tauri desktop skills
│   ├── htmx/                    ← HTMX + Alpine.js + Lit skills
│   ├── typescript/              ← TypeScript base patterns
│   ├── go/                      ← Go language skills
│   ├── python/                  ← Python + PyO3 bridge skills
│   ├── architecture/            ← Cross-language CLEAN architecture
│   ├── testing/                 ← BDD testing (Cucumber.js + Playwright)
│   ├── devops/                  ← GitOps CI/CD skills
│   ├── ui-ux/                   ← UI/UX skills
│   ├── documentation/           ← Documentation skills
│   ├── flint/                   ← Flint Realtime Fabric SDK skills (6 languages)
│   ├── document-extraction/     ← Kreuzberg multi-format extraction
│   ├── learn/                   ← Feynman-Spine learning skills (goal, survey, plan, loop, grade, retain, practice, certify, KB, meta)
│   └── imported/                ← Git submodule skills
│       ├── artifact-refiner/        ← PMPO artifact refinement (submodule)
│       └── sycophancy-correction/   ← 8-pattern detection, Rust MCP server (submodule)
│
├── tools/                       ← Rust workspaces and submodule tools
│   ├── forge-rs/                ← Layer 4: code enrichment engine
│   │   ├── crates/              ← 6-crate Rust workspace
│   │   ├── templates/meta/      ← Meta-templates for generating new templates
│   │   └── constitution-templates/ ← Default language constitutions
│   ├── prometheus-cli/          ← Skill management CLI (4-crate Rust workspace)
│   ├── surreal-memory-server/   ← Knowledge graph + distributed state (submodule)
│   ├── liter-llm/               ← Multi-model routing proxy (submodule)
│   └── prometheus-knowledge/    ← Karpathy learning wiki (submodule)
│
├── shared/references/           ← Cross-skill architecture references
├── agents/                      ← Orchestration agent definitions
├── hooks/hooks.json             ← Lifecycle hooks (6 events: SessionStart, UserPromptSubmit, Pre/PostToolUse, SubagentStop, Stop)
├── policies/                    ← Cedar governance policies
└── .gitmodules                  ← Submodule registry
```

---

## Skills Reference

### Process Skills (`skills/process/`)

| Skill | Layer | Purpose |
|---|---|---|
| `native-agent` | Generator | `/create-native-agent` — scaffolds complete Rust agent workspaces |
| `zeespec-interrogator` | 1 | 60-question Zachman 5W1H constraint interrogation, GO/NO-GO manifests |
| `iterative-evolver` | 2 | Strategic PMPO loop: Assess→Analyze→Plan→Execute→Reflect |
| `pmpo-evolver` | 2 | Strategy router for 5 evolution perspectives: competitive, trend, unique-product, idea-validation, self-learning; liter-llm model routing; Darwin three-gate idea validation |
| `kbd-process-orchestrator` | 1 | Tactical KBD loop (16 child skills): change management, multi-tool dispatch |
| `pmpo-outer-loop` | 3 | Standing loop: `/loop-define`, `/loop-tick`, `/loop-report` — perspective-aware; one tick = one evolver cycle |
| `pmpo-elicit` | Gate | Ask / source / research / decide elicitation primitive with provenance |
| `pmpo-skill-creator` | Meta | Generates and updates skills via PMPO (human-gated `--update`) |
| `liter-llm-bridge` | Meta | Per-phase model class routing via liter-llm |
| `ideation-mindmap` | Onramp | Concept → 6-branch tree via surreal-memory, feeds ZeeSpec |
| `kbd-evolve` | Seed | Landscape survey → ranked evolution brief seeding `/kbd-new-phase` |
| `kbd-goal` | Goal | Structured goal definition with success criteria, timeboxes, and cross-tool parity |
| `kbd-goal-check` | Goal | Goal progress check and milestone verification against `goals.md` |

Full detail on every process skill — commands, state files, child skills, composition — is in
[docs/guide/09-process-skills.md](docs/guide/09-process-skills.md).

### Language Skills

#### Rust (`skills/rust/`)

| Skill | Templates | Purpose |
|---|---|---|
| `axum-patterns` | `router.rs`, `app_error.rs`, `app_state.rs`, `middleware.rs`, `handler.rs` | Axum 0.8 router, extractors, error handling, middleware |
| `error-handling` | — | thiserror/anyhow boundary, `#[cold]` error paths, no unwrap() |
| `async-patterns` | — | Arc/RwLock selection, parking_lot, broadcast channels, graceful shutdown |
| `workspace-structure` | — | resolver=2, domain-driven crate decomposition, workspace deps |
| `mcp-server` | — | JSON-RPC 2.0 dispatch, tool registry, SSE stream, stdio transport |
| `actor-model` | — | mpsc-based actor pattern, typed messages, supervision |
| `performance` | — | jemalloc, `#[cold]`, MaybeUninit, mem::take, parking_lot |

#### React (`skills/react/`)

| Skill | Templates | Purpose |
|---|---|---|
| `react-vite-stack` | `page_component.tsx`, `feature_hook.ts`, `store.ts`, `api_client.ts`, `entity_hook.ts` | React 19 + Vite 8 + TanStack + Zustand 5 + shadcn/ui |
| `prometheus-entity-skills` | — | Entity graph CRUD, GraphQL, Prisma, realtime sync |

#### Flutter (`skills/flutter/`)

| Skill | Templates | Purpose |
|---|---|---|
| `flutter-rust-ffi` | `riverpod_notifier.dart`, `feature_repository.dart`, `go_router_config.dart` | flutter_rust_bridge v2, Riverpod 3.x, GoRouter |

#### HTMX (`skills/htmx/`)

| Skill | Templates | Purpose |
|---|---|---|
| `htmx-alpine-lit` | `page.html`, `lit_component.ts`, `react_island.tsx`, `axum_fragment_handler.rs` | HTMX 2.0.8 + Alpine.js + Lit + HTMX-in-React embedding |

#### Learn (`skills/learn/`)

Feynman-Spine learning: goal, survey, plan, loop, grade, retain, practice, certify, KB management, meta-learning.

| Skill | Purpose |
|---|---|
| `ui-surface` | Cross-harness UI rendering layer |
| `learn-goal` | Entry point: goal declaration + feasibility gate |
| `learn-survey` | Diagnostic placement + learner model seeding |
| `learn-plan` | Adaptive curriculum planner |
| `feynman-loop` | Core Feynman explain/grade/gap/relearn cycle |
| `learn-grade` | External sycophancy-corrected grader |
| `learn-retain` | FSRS spaced repetition reviews |
| `learn-practice` | Deliberate practice (derivation/implementation/transfer) |
| `learn-certify` | OB 3.0 / W3C VC credential issuance |
| `learn-kb` | Custom knowledge base management |
| `learn-about-system` | Meta-learning adoption entry point |
| `learn-harness` | Per-harness capability orientation |

#### Research (`skills/research/`)

| Skill | Stages | Purpose |
|---|---|---|
| `deep-research` | 10-stage pipeline | Long-form deep research with source verification, contradiction resolution, knowledge graph, Feynman quality gate, and `.research` package export |

**Pipeline stages:** Stage 01 Planner → Stage 02 Search → Stage 03 Retrieve → Stage 04 Collect → Stage 05 Verify → Stage 06 Resolve → Stage 07 Graph → Stage 08 Cite → Stage 09 Report → Stage 10 Export

**Key integrations:** surreal-memory (graph persistence), sycophancy-correction (bias detection), liter-llm-bridge (model routing), learn-grade (Feynman quality gate), pmpo-elicit (contradiction escalation)

**Invocation:** `/deep-research "What are the trade-offs of vector databases for production RAG?"`

#### Other Languages

| Directory | Skill | Purpose |
|---|---|---|
| `tauri/` | `tauri-react-vite` | Tauri 2 + React 19 + gen_ui_core sharing |
| `typescript/` | `typescript-base-patterns` | TypeScript 6 strict mode, discriminated unions, Result types, zod |
| `go/` | `go-base-patterns` | Go 1.22 errors, context, slog, module layout |
| `python/` | `pyo3-bridge` | PyO3 0.22 Rust-Python bridge, maturin, skill executor generation |
| `architecture/` | `clean-architecture` | 4-layer CLEAN model across all languages |

---

## forge-rs — Layer 4 Enrichment Engine

forge-rs is the code-generation enrichment engine. It sits between an OpenSpec task
and the AI agent that implements it, injecting language-specific knowledge before
the agent touches any code.

### CLI Reference

```bash
forge init                                      # scaffold .forge/ in current project
forge enrich <task-path>                        # enrich an OpenSpec task
forge reflect <iteration-id>                    # process iteration into Karpathy loop
forge drift [--language rust]                   # report stale skill candidates
forge validate <file> --language rust           # check against constitution
forge mcp [--port 8943]                         # start MCP server
forge template new skill <lang> <name>          # scaffold a new skill
forge template new template <skill-path> <name> # add template to existing skill
forge template validate <skill-path>            # check Tera syntax
```

### MCP Server (port 8943)

```json
{ "name": "forge", "url": "http://localhost:8943/mcp", "transport": "sse" }
```

Tools: `forge_enrich`, `forge_reflect`, `forge_drift`, `forge_validate`

### macOS MCP Services

On macOS, keep the lightweight local MCP services alive as user LaunchAgents for
the logged-in account. This gives them the right `HOME`, `PATH`, user config, and
AI-tool credentials without running system daemons.

```bash
# Build/install all local binaries first, including pk-cherry.
bash scripts/check-prerequisites.sh --build-tools

# macOS only: render LaunchAgents into ~/Library/LaunchAgents and start them.
bash scripts/prometheus-services.sh install
bash scripts/prometheus-services.sh load
bash scripts/prometheus-services.sh status
```

The LaunchAgents manage `pk-cherry` on `127.0.0.1:8942` and `forge mcp` on
`127.0.0.1:8943`. `surreal-memory-server` remains Docker-managed on
`127.0.0.1:23001`; the service script only reports whether that port is ready.
On Linux, use systemd user services or cron-style scheduled jobs instead of
LaunchAgents.

---

## Template System

### Template Discovery

forge-rs scans `skills/<language>/<skill-name>/templates/*.tera`. Each skill's
`skill.toml` declares which templates it contains. Templates are auto-loaded.

### Tera Template Variables

| Variable | Source |
|---|---|
| `{{ "{{" }} task_description {{ "}}" }}` | From `tasks.md` in the OpenSpec task folder |
| `{{ "{{" }} task_id {{ "}}" }}` | Change ID |
| `{{ "{{" }} constitution_summary {{ "}}" }}` | Active language constitution standards |
| `{{ "{{" }} karpathy_focus {{ "}}" }}` | Bounded context from the committed project/shared/global snapshot |

### Meta-Template System

```bash
forge template new skill rust my-skill        # scaffold new skill
forge template new template skills/rust/my-skill/ handler.rs  # add template
forge template validate skills/rust/my-skill/ # check Tera syntax
```

Meta-templates live in `tools/forge-rs/templates/meta/`:
- `new_skill_toml.tera` — generates `skill.toml`
- `new_skill_md.tera` — generates `SKILL.md`
- `new_tera_template.tera` — generates a new `.tera` file with variable docs
- `new_constitution_toml.tera` — generates a language constitution

---

## Architecture Patterns

### React: Component → Hook → Store → API

Components compose hooks. Hooks orchestrate stores. Stores own API calls.
Components NEVER import stores or call fetch() directly.

### Flutter: Widget → Riverpod → Repository → Rust FFI

Widgets watch providers. Notifiers call repositories. Only the Rust FFI repository
calls flutter_rust_bridge functions.

### HTMX: Server Drives, Alpine Declares, Lit Encapsulates

HTMX returns HTML fragments from the server. Alpine handles local state.
Lit encapsulates complex interactive elements. React hosts HTMX islands via `HtmxIsland`.

---

## Tools

| Tool | Source | Role |
|---|---|---|
| `tools/forge-rs` | This repo | Layer 4 code enrichment engine (`forge` binary + MCP :8943) |
| `tools/prometheus-knowledge` | Git submodule | Karpathy learning wiki (`pk` / `pk-cherry` MCP :8942) |
| `tools/liter-llm` | Git submodule | Multi-provider LLM gateway (140+ providers, 22 MCP tools) |
| `tools/surreal-memory-server` | Git submodule | Knowledge graph + scoped memory + MCP :23001 |
| `tools/prometheus-cli` | This repo | Skill management, self-learning, Cedar governance (`prometheus` binary) |
| `tools/prometheus-rust-auditor` | This repo | Staged Rust code-quality remediation pipeline |

Full CLI surfaces, crates, ports, and endpoints for every tool are in
[docs/guide/13-tools-reference.md](docs/guide/13-tools-reference.md).

---

## MCP Server Substrate

Eight MCP servers form the shared substrate that makes loops compound across sessions and tools.
The canonical port table is `scripts/mcp-port-table.json`; full detail is in
[docs/guide/05-mcp-substrate.md](docs/guide/05-mcp-substrate.md).

| Server | Transport | Port | Role |
|---|---|---|---|
| surreal-memory | sse/http | 23001 | Semantic knowledge graph — the memory substrate |
| prometheus-knowledge | sse/http | 8942 | Karpathy flat-file KB — immutable snapshots, search, and receipt reconciliation |
| forge-rs | sse/http | 8943 | Code enrichment — `forge reflect` / `pk ingest` |
| sycophancy-correction | stdio | — | Structural anti-sycophancy gate on reflection output |
| liter-llm | stdio | — | Multi-provider LLM gateway / per-phase routing |
| sequential-thinking | stdio | — | Structured reasoning for multi-step planning |
| tavily | stdio | — | Search-first web access (discovery) |
| firecrawl | stdio | — | Extraction-first web access, self-hostable |

```bash
# Bring up the HTTP MCP services (macOS launchd) and configure all tools
bash scripts/install-mcp-services.sh
bash scripts/configure-mcp-all-tools.sh
bash scripts/check-mcp-health.sh
```

---

## Platform Compatibility

| Platform | Skills | MCP Servers | Plugin Manifest |
|----------|--------|-------------|-----------------|
| **Claude Code** (CLI/Desktop) | ✅ | ✅ `.mcp.json` | ✅ `.claude-plugin/plugin.json` |
| **Kimi Code CLI** | ✅ | ✅ `~/.kimi-code/config.toml` | — |
| **MiniMax / Mavis** | ✅ `_meta.json` | ✅ `~/.minimax/mcp/mcp.json` | — |
| **OpenCode** | ✅ | ✅ `opencode.json` plugin | ✅ `.opencode/plugin.ts` |
| **Codex CLI** | ✅ | ✅ `.codex/config.toml` | — |
| **Cursor** | ✅ | — | — |
| **Windsurf** | ✅ | — | — |
| **Gemini CLI** | ✅ | — | — |
| **Roo Code** | ✅ | — | — |
| **Amp** | ✅ | — | — |

```bash
# Install to all detected platforms in one command
bash scripts/install-skills-flat.sh

# Check toolchain + service status (works on any platform)
bash shared/scripts/detect-toolchain.sh

# Platform-specific installer with MCP config
npm run install:platforms
```

## Mobile (iOS / Android)

Mobile platforms cannot spawn processes, so a skill that shells out to `bash`,
`python3`, or a binary is inert there. Every skill is classified by what it
actually needs at runtime:

| Class | Count | Meaning |
|---|---|---|
| **manifest-only** | **249** | No scripts — **runs on mobile today, unchanged** |
| E0 | 28 | Needs a process; no on-device path |
| E1 | 18 | Portable **with** granted capabilities (filesystem/clock) |
| E2 | 2 | Portable to a Wasm component |
| R | 13 | Remote execution — phone drives a paired desktop |

**249 of 310 skills already work on mobile**, because a manifest-only skill is
instructions a model reads — there is nothing to execute. The portability
problem is confined to the 61 script-bearing skills.

```bash
# Classify every skill (derived, not asserted)
bash skills/process/adversarial-review/scripts/classify-mobile-execution.sh

# Fail CI when the committed classification goes stale
bash skills/process/adversarial-review/scripts/classify-mobile-execution.sh --check

# Build the native FFI library for iOS + Android
bash substrate/skill-ffi/build-mobile.sh
```

Three mechanisms close the gap:

1. **Manifest-only** — nothing to port. Prefer this when authoring new skills.
2. **Wasm components** — `wit/prometheus-component@0.1.0`. The WIT family is
   authored and a reference component validates against it, but **nothing has
   executed it yet**; UAR's Wasm tier is still a stub.
3. **Native FFI** — `substrate/skill-ffi` builds verified artifacts for
   `aarch64-apple-ios` (16,408 B) and `aarch64-linux-android` (454,856 B), using
   `flutter_rust_bridge` 2.12.0 to match what the consuming app already ships.

> ⚠️ **Two Wasm formats.** `skills/rust/librefang-wasm-skill/` emits **core-wasm**
> guests with an `extern "C"` ABI; UAR loads **Component Model** binaries. They do
> not interoperate and there is no adapter. Target `wit/prometheus-component` for
> UAR.

Full detail, reasoning, and best practices: **[Mobile documentation](site/docs/mobile/overview.md)**.

## Getting Started

**New here?** The [5-step Quick Start](docs/QUICK_START.md) gets you to `/learn-goal` working in under 10 minutes.

```bash
git clone --recurse-submodules <repo-url>
cd prometheus-skill-pack

# Build all tools
bash scripts/check-prerequisites.sh --build-tools

# Install skills to all platforms + configure MCP servers
bash scripts/install-skills-flat.sh

# macOS service readiness
bash scripts/prometheus-services.sh install
bash scripts/prometheus-services.sh load

# Initialize forge in your project
forge init

# Create your first native agent
/create-native-agent
```

For the full prerequisite, install, verification, and first-loop walkthrough, see
[docs/guide/19-installation.md](docs/guide/19-installation.md); for keeping everything current, see
[docs/guide/20-updating.md](docs/guide/20-updating.md).

---

## Contributing

Contributions are welcome. Prerequisites, setup, skill creation workflow, forge-rs development,
PR checklist, and submodule policy are in [CONTRIBUTING.md](CONTRIBUTING.md). The deep-dive
workflow guide is in [docs/guide/21-contributing.md](docs/guide/21-contributing.md).

**Quick path:**
```bash
git clone --recurse-submodules https://github.com/Prometheus-AGS/prometheus-skill-system.git
cd prometheus-skill-system
npm install
bash scripts/install-skills-flat.sh
npm run validate:strict   # must pass before opening a PR
```

---

## License

[MIT](LICENSE) © 2026 Travis James

Full documentation: **[docs/guide/](docs/guide/README.md)**
