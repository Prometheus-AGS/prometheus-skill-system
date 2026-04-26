# Prometheus Skill System

A self-improving AI skill execution engine. 62 skills (382 including sub-skills) across 5 domains, a 4-crate Rust CLI, Cedar governance, surreal-memory distributed state, sycophancy correction, and a nested PMPO pipeline that learns from every execution.

Built for teams deploying AI agents in production environments where capability improvement must be governed, audited, and reproducible.

[![Validate Skills](https://github.com/Prometheus-AGS/prometheus-skill-system/actions/workflows/validate.yml/badge.svg)](https://github.com/Prometheus-AGS/prometheus-skill-system/actions/workflows/validate.yml)

## Compliance Scores

| Standard               | Score      | Evidence                                                                                 |
| ---------------------- | ---------- | ---------------------------------------------------------------------------------------- |
| **AgentSkills.io**     | **97/100** | 382 skills validated (0 errors, 0 warnings). Recursive validation covers all sub-skills. |
| **Claude Code Plugin** | **96/100** | plugin.json has all 9 fields. 5 hook events. 4 MCP servers. CI green.                    |
| **OpenCode Support**   | **93/100** | 3 typed TypeScript tool definitions, `.opencode/package.json`, 9-platform compatibility. |
| **Marketplace**        | **95/100** | 5 granular plugin entries, v1.2.0 semver, git tag, CI badge.                             |

## How Skills Improve Themselves

This is not a static skill collection. Skills improve from execution data through a four-layer feedback loop — the first production implementation of the [Hermes/GEPA self-learning architecture](https://github.com/NousResearch/hermes-agent-self-evolution) using a Rust-native toolchain.

```mermaid
flowchart TB
    subgraph L4["Layer 4: DISTRIBUTION"]
        CLI["prometheus CLI<br/>install · audit · verify · search"]
        PLAT["9 Platforms<br/>Claude Code · OpenCode · Cursor<br/>Codex · Gemini · Roo · Windsurf · Amp · Cline"]
    end

    subgraph L3["Layer 3: OPTIMIZATION"]
        SIG["Skill → dspy-rs Signature"]
        OPT["BootstrapFewShot + MIPRO"]
        CEDAR["Cedar Skill Mutation PEP"]
    end

    subgraph L2["Layer 2: KNOWLEDGE"]
        COMPILE["Compile (Karpathy method)"]
        LINT["Lint → Focus → Fix"]
        WIKI["Durable Markdown Wiki"]
    end

    subgraph L1["Layer 1: MEMORY"]
        SM["surreal-memory"]
        GRAPH["Knowledge Graph + Graph-RAG"]
        TS["TaskStreams + Scoped Memory"]
    end

    CLI --> PLAT
    OPT --> CEDAR
    SIG --> OPT
    COMPILE --> LINT --> WIKI
    SM --> GRAPH --> TS

    L4 ~~~ L3
    L3 ~~~ L2
    L2 ~~~ L1
```

The self-learning loop:

1. **Trace capture**: Cross-platform `TraceCapture` protocol logs every skill execution — not locked to one platform
2. **Knowledge compilation**: [prometheus-knowledge](https://github.com/Prometheus-AGS/prometheus-knowledge-rs) compiles traces into a durable wiki via compile→lint→focus→fix
3. **Prompt optimization**: [dspy-rs](https://github.com/GQAdonis/dspy-rs) BootstrapFewShot + MIPRO — routed through local models by default (zero data egress)
4. **Cedar governance**: All mutations gated by environment-aware policies — development permits, staging validates, production denies
5. **Sycophancy correction**: Every reflect and assess output checked for the [8 canonical sycophancy patterns](https://github.com/Know-Me-Tools/sycophancy-correction-skill) before storage

Feature flags control pipeline stages — see [Rust CLI Development](#rust-cli-development).

## Architecture

### Nested PMPO Pipeline

```mermaid
flowchart LR
    subgraph OUTER["Iterative Evolver (Strategic)"]
        A[Assess] --> AN[Analyze] --> P[Plan] --> E[Execute] --> R[Reflect]
        R -->|loop| A
    end

    subgraph INNER["KBD Orchestrator (Tactical)"]
        KP[kbd-plan] --> KE[kbd-execute] --> KR[kbd-reflect]
    end

    subgraph QA["Quality Gates"]
        AR[artifact-refiner<br/>per-change QA]
        SC[sycophancy-correction<br/>reflect + assess check]
        CEDAR2[Cedar PEP<br/>mutation governance]
    end

    E -->|delegates| KP
    KE --> AR
    KR -->|results| R
    R --> SC
    A --> SC
    KR --> CEDAR2
```

### Sycophancy Correction Integration

```mermaid
flowchart TD
    OUTPUT[LLM Output] --> DETECT{Sycophancy<br/>Detection}
    DETECT -->|Clean < 0.3| STORE[Store as-is]
    DETECT -->|Flag 0.3-0.5| WARN[Warn User +<br/>Annotate]
    DETECT -->|Correct ≥ 0.5| REWRITE[Auto-Rewrite]
    DETECT -->|Block ≥ 0.7 + S-08| REGEN[Regenerate]
    REWRITE --> VERIFY{Re-detect}
    VERIFY -->|Clean| STORE
    VERIFY -->|Still dirty| REGEN

    subgraph PATTERNS["8 Canonical Patterns"]
        S01["S-01 Unprompted Affirmation"]
        S02["S-02 Agreement Without Grounding"]
        S03["S-03 Caveat Collapse"]
        S04["S-04 Self-Rationalization"]
        S05["S-05 Context Bleed Alignment"]
        S06["S-06 Confidence Without Basis"]
        S07["S-07 Scope Creep Flattery"]
        S08["S-08 Reflect Phase Inversion"]
    end
```

### Cedar Governance Flow

```mermaid
flowchart LR
    AGENT[Optimizer Agent] -->|skill.mutate| PEP{Cedar PEP}
    PEP -->|PERMIT| WRITE[Write SKILL.md]
    PEP -->|DENY| BLOCK[Block + Log]

    subgraph POLICIES["Environment Policies"]
        DEV["development: permit all"]
        STG["staging: require validation"]
        PRD["production: deny mutations"]
    end

    PEP --> POLICIES
```

## What's Inside

### Skills (62 total)

| Domain       | Skills | Highlights                                                            |
| ------------ | ------ | --------------------------------------------------------------------- |
| **React**    | 27     | Entity CRUD, GraphQL, Prisma, realtime sync, performance optimization |
| **Process**  | 20     | KBD orchestrator, iterative evolver, PMPO skill creator               |
| **DevOps**   | 4      | GitOps bootstrap, transform, ArgoCD multi-cloud, Kustomize overlays   |
| **Testing**  | 1      | BDD with Cucumber.js + Playwright + video recording                   |
| **Imported** | 10     | Artifact refiner (9) + Sycophancy correction (1) via git submodules   |

### Rust CLI (`tools/prometheus-cli/`)

4-crate workspace with 16 subcommands:

| Crate               | Purpose                                                                                |
| ------------------- | -------------------------------------------------------------------------------------- |
| `prometheus-cli`    | Binary: install, audit, verify, search, learn, optimize, evolve, policy, and more      |
| `prometheus-agents` | 10-platform adapter library with `TraceCapture` protocol                               |
| `prometheus-learn`  | Self-learning pipeline: trace capture, evaluation, knowledge compilation, optimization |
| `prometheus-cedar`  | Cedar Skill Mutation PEP — governs skill.mutate/generate/promote/trace.capture         |

## Quick Start

### Prerequisites

- **Node.js >= 18** — for skill validation and formatting
- **Rust toolchain** — for CLI + MCP servers (`rustup` recommended)
- **Git** — with submodule support
- **Docker** (optional) — for surreal-memory server

### Clone and Setup

```bash
git clone --recurse-submodules git@github.com:Prometheus-AGS/prometheus-skill-system.git
cd prometheus-skill-system

# Check prerequisites (auto-installs Rust via rustup if missing)
npm run setup
# Or check without installing:
npm run check-prereqs
```

The `setup` command detects your environment and installs missing prerequisites:

- Installs Rust via `rustup` if not found
- Installs npm dependencies
- Builds all 3 Rust binaries (prometheus CLI, sycophancy MCP, surreal-memory MCP)
- Installs binaries to `/usr/local/bin/` (or `~/.local/bin/` fallback)

### Validate Skills

```bash
npm run validate
# 382 skill(s) validated (including sub-skills) — 0 errors, 0 warnings
```

### Build and Install

Three Rust binaries to build:

```bash
# 1. Build the Rust CLI (skill management, Cedar governance, sycophancy detection)
cd tools/prometheus-cli && cargo build --release && cd ../..
cp tools/prometheus-cli/target/release/prometheus /usr/local/bin/

# 2. Build the sycophancy MCP server (8-pattern detection + correction)
cd skills/imported/sycophancy-correction && cargo build --release && cd ../../..
cp skills/imported/sycophancy-correction/target/release/sycophancy-correction /usr/local/bin/

# 3. Build the surreal-memory MCP server (knowledge graph + distributed state)
#    Skip if using Docker: docker ps | grep surreal-memory
cd tools/surreal-memory-server && cargo build --release && cd ../..
cp tools/surreal-memory-server/target/release/surreal-memory-server /usr/local/bin/

# Install all 54 skills as slash commands across 9 platforms
npm run install:skills

# Verify everything
prometheus doctor
prometheus --version              # CLI
sycophancy-correction --version 2>&1 || echo "sycophancy MCP ✅"
surreal-memory-server --version 2>&1 || echo "surreal-memory MCP ✅"
```

### Platform-Specific Paths

| Platform    | Global Skills                 | Slash Commands |
| ----------- | ----------------------------- | -------------- |
| Claude Code | `~/.claude/skills/`           | 54 skills      |
| OpenCode    | `~/.config/opencode/skills/`  | 54 skills      |
| Cursor      | `~/.cursor/skills/`           | 54 skills      |
| Codex / Amp | `~/.agents/skills/`           | 54 skills      |
| Gemini CLI  | `~/.gemini/skills/`           | 54 skills      |
| Roo Code    | `~/.roo/skills/`              | 54 skills      |
| Windsurf    | `~/.codeium/windsurf/skills/` | 54 skills      |
| Cline       | `~/.cline/skills/`            | 54 skills      |

## Slash Commands

### Process Orchestration

| Command                  | Purpose                                                                      |
| ------------------------ | ---------------------------------------------------------------------------- |
| `/evolve`                | Full iterative evolution cycle (assess → analyze → plan → execute → reflect) |
| `/evolve-assess`         | Assess current state against goals                                           |
| `/evolve-plan`           | Create prioritized improvement plan                                          |
| `/evolve-execute`        | Execute plan (delegates to KBD for software domain)                          |
| `/evolve-report`         | Generate evolution report with artifact quality metrics                      |
| `/kbd-init`              | Initialize KBD orchestrator in a project                                     |
| `/kbd-assess`            | Assess codebase against phase goals                                          |
| `/kbd-plan`              | Create ordered change list with OpenSpec detection                           |
| `/kbd-execute`           | Dispatch to best tool with artifact-refiner QA                               |
| `/kbd-reflect`           | Phase retrospective with Cedar audit trail                                   |
| `/create-skill`          | Generate a new skill from scratch via PMPO                                   |
| `/clone-skill`           | Adapt an existing skill for a new domain                                     |
| `/sycophancy-correction` | Detect and correct sycophantic patterns in any artifact                      |

### GitOps CI/CD

| Command              | Purpose                                           |
| -------------------- | ------------------------------------------------- |
| `/gitops-bootstrap`  | Scaffold complete multi-cloud GitOps from scratch |
| `/gitops-transform`  | Transform existing CI/CD to GitOps standard       |
| `/argocd-multicloud` | Install + configure ArgoCD across GKE/AKS/EKS     |
| `/kustomize-overlay` | Generate 3D Kustomize overlay structure           |

### React Entity Management

| Command                  | Purpose                                              |
| ------------------------ | ---------------------------------------------------- |
| `/entity-graph-init`     | Initialize entity graph in a project                 |
| `/entity-crud-page`      | Full CRUD page with list, create, edit, delete       |
| `/entity-gql-setup`      | Wire GraphQL with entity descriptors                 |
| `/entity-prisma-setup`   | Generate entity configs from Prisma schema           |
| `/entity-realtime-setup` | Add realtime sync (WebSocket, Supabase, ElectricSQL) |
| `/entity-audit`          | Architecture compliance audit                        |

### Testing

| Command        | Purpose                                          |
| -------------- | ------------------------------------------------ |
| `/bdd-testing` | Generate BDD tests with Cucumber.js + Playwright |

## CLI Commands

```bash
# Skill management
prometheus install <repo>       # Install skills from GitHub or local path
prometheus uninstall <name>     # Remove skill from all platforms
prometheus list [--verbose]     # List installed skills with symlink targets
prometheus search <query>       # Search GitHub for skill repos

# Security & integrity
prometheus audit [--path .]     # Security scan (credentials, injection, anti-patterns)
prometheus verify [--update]    # SHA256 checksum validation against Skills.lock
prometheus doctor               # Health check (platforms, surreal-memory, KBD, evolver)
prometheus validate [path]      # Run agentskills.io validator

# Self-learning pipeline
prometheus learn --seed         # Capture traces from Claude Code session history
prometheus learn --compile      # Compile traces into knowledge wiki
prometheus optimize <skill>     # Run dspy-rs prompt optimization

# Cedar governance
prometheus policy show          # Display loaded Cedar policies
prometheus policy validate      # Validate Cedar policy syntax
prometheus policy check -o skill.mutate -s <skill> -e <env>  # Test a policy decision

# Project state
prometheus status               # Show Skills.toml + KBD waypoint + evolver state
prometheus evolve <name>        # Trigger iterative evolution cycle
prometheus memory ping          # Check surreal-memory server
prometheus build -s svc -o env  # Kustomize build + validation
```

## MCP Servers

The system uses three MCP servers configured in `.mcp.json`:

| Server                  | Transport | Purpose                                                        |
| ----------------------- | --------- | -------------------------------------------------------------- |
| `surreal-memory`        | HTTP SSE  | Knowledge graph, scoped memory, TaskStreams, Graph-RAG         |
| `sycophancy-correction` | stdio     | Detect and correct sycophantic patterns (8 canonical patterns) |
| `tavily`                | stdio     | Web research for landscape analysis                            |
| `sequential-thinking`   | stdio     | Multi-step reasoning for complex planning                      |

```bash
# Verify MCP servers
prometheus memory ping                    # surreal-memory
sycophancy-correction --help 2>&1 || true # sycophancy (stdio mode)
```

## Multi-Model Routing

Not every phase of the pipeline needs a frontier model. Assess, plan, and reflect operate at the boundary of ambiguity and benefit from frontier reasoning. The work in between — scaffolding, mechanical apply, refiner iteration, verify checks, status reports — is structured transformation that smaller or medium models handle reliably.

The system declares a **model class** per stage so external orchestrators can dispatch each phase to the cheapest model that meets the cognitive requirement.

### The Three Classes

| Class      | Cognitive Profile                                              | Typical Models                                                  |
| ---------- | -------------------------------------------------------------- | --------------------------------------------------------------- |
| `small`    | Mechanical transformation, structured writes, status reports   | Qwen 2.5 9B (local), Llama 3.3 70B (Groq), Phi, Mistral 7B      |
| `medium`   | Bounded synthesis, calibrated scoring, single-boundary work    | Mixtral 8x22B, Llama 3.3 70B, Claude Haiku, Command-R+          |
| `frontier` | Open-ended reasoning under ambiguity, cross-domain synthesis   | Claude Sonnet/Opus, GPT-4o / o1, Gemini 2.0 Pro, Mistral Large  |

### Phase → Class Map

| Stage                     | Class    | Why                                                       |
| ------------------------- | -------- | --------------------------------------------------------- |
| `/evolve-assess`          | frontier | Holistic codebase understanding, ambiguity resolution     |
| `/evolve-plan`            | frontier | Decomposition under ambiguity                             |
| `/evolve-reflect`         | frontier | Quality judgment, regression detection                    |
| `/evolve-status`          | small    | Read-only structured reporting                            |
| `/kbd-assess`             | frontier | Cross-domain gap analysis                                 |
| `/kbd-plan`               | frontier | Change scope determines downstream cost                   |
| `/kbd-reflect`            | frontier | Feeds prior_assessments — degrades compound errors        |
| `/kbd-status`             | small    | Read-only                                                 |
| `/opsx:new`               | small    | Deterministic scaffolding                                 |
| `/opsx:apply` (low)       | small    | Mechanical CRUD/plumbing                                  |
| `/opsx:apply` (medium)    | medium   | Single boundary crossing, no design markers               |
| `/opsx:apply` (high)      | frontier | New abstraction, cross-domain                             |
| `/opsx:verify`            | medium   | Structured 3-dim QA against known artifacts               |
| `/opsx:archive`           | small    | File move + spec delta sync                               |
| artifact-refiner iterate  | small    | Constraint-diff delta generation                          |
| artifact-refiner evaluate | medium   | Constraint violation judgment                             |

For a typical KBD phase with 3 changes: **frontier API spend drops from ~20+ calls to ~3** (assess + plan + reflect), with quality preserved where it matters.

### Configuring the Policy

Routing is driven by a `model_policy` block in `.kbd-orchestrator/project.json`. The block has three parts: a `registry` mapping classes to concrete models per environment, a `phases` map assigning each pipeline stage to a class, and an `active_environment` selector.

```json
{
  "model_policy": {
    "registry": {
      "small":    { "local": "Qwen3.5-9B-Q8_0",     "t4": "Qwen3.5-9B-Q8_0",    "l4": "Qwen3.5-9B-Q8_0"    },
      "medium":   { "local": null,                  "t4": "Qwen3.5-27B-Q4",     "l4": "Qwen3.5-35B-A3B-Q4" },
      "frontier": { "local": "claude-sonnet-4-6",   "t4": "claude-sonnet-4-6",  "l4": "claude-sonnet-4-6"  }
    },
    "phases": {
      "kbd-assess": "frontier",
      "kbd-plan":   "frontier",
      "opsx-apply-low":    "small",
      "opsx-apply-medium": "medium",
      "opsx-apply-high":   "frontier"
    },
    "active_environment": "local"
  }
}
```

Switching `active_environment` from `local` → `t4` → `l4` swaps the entire stack to a different hardware tier without touching any other config. See `skills/process/kbd-process-orchestrator/references/schemas/project.template.json` for the full template and `skills/process/kbd-process-orchestrator/references/model-routing.md` for the complete contract including complexity scoring rules.

### How a Phase Honors the Policy

Each phase prompt has a `## Model Selection` section that:

1. Reads its required class from `project.json → model_policy.phases.<phase-key>`.
2. If the hosting model doesn't match a `frontier`-required phase, **stops and emits `MODEL MISMATCH`** rather than silently degrading.
3. Emits a machine-readable directive for orchestrators to parse:

   ```
   [MODEL_ROUTING] phase=kbd-plan class=frontier model=claude-sonnet-4-6 env=local
   ```

The `/kbd-plan` output annotates each change with `Complexity score` and `Model class` so `/opsx:apply` can route per-change without re-scoring. The `/kbd-execute` dispatch contract carries `Model class`, `Concrete model`, and `Model rationale` per change so the executing tool knows exactly what model to use.

### Three Integration Patterns

**Pattern 1 — Native harness model switching (preferred).** The harness parses `[MODEL_ROUTING]` directives and switches the dispatched subagent to the matching model. Claude Code with per-subagent `--model` flag, prom-lanes, and UAR all support this. No additional infrastructure required.

**Pattern 2 — MCP-mediated routing via liter-llm.** When the harness can't switch models natively, route through the `liter-llm-bridge` skill. liter-llm is a Rust LLM proxy that exposes 22 MCP tools; the bridge skill installs it, detects which providers you have configured, and registers its MCP server with your harness. Phases call its `complete` tool with a class alias (`small` / `medium` / `frontier`) and liter-llm dispatches to the appropriate provider.

**Pattern 3 — Manual.** Read the `[MODEL_ROUTING]` directive yourself and re-invoke the next phase with the right model. Useful for one-off frontier work; impractical for full pipelines.

### Setting Up liter-llm-bridge (Pattern 2)

The bridge ships as a skill in `skills/process/liter-llm-bridge/`. Three slash entries handle the lifecycle:

```bash
# 1. Build and install liter-llm from the user's Rust fork
/liter-llm-bridge install
# Clones https://github.com/GQAdonis/liter-llm.git → ~/.local/share/liter-llm/src
# Runs `cargo install --path crates/liter-llm-cli --locked --root ~/.local`
# Verifies `liter-llm --version` and `liter-llm mcp --help` work

# 2. Detect providers, write config, register MCP server with the active harness
/liter-llm-bridge configure
# Scans env for ANTHROPIC_API_KEY, OPENAI_API_KEY, GROQ_API_KEY, OLLAMA_HOST,
# VLLM_BASE_URL, etc. (full list in references/provider-env-vars.md)
# Writes ~/.config/liter-llm/config.toml with class → provider/model aliases
# Auto-detects harness (Claude Code / opencode / cursor / codex) and adds
# liter-llm to its MCP server list

# 3. Document or activate per-phase routing
/liter-llm-bridge route
# By default just documents the contract. With --enable-routing-hook installs
# a harness hook that intercepts dispatched prompts carrying [MODEL_ROUTING]
# directives and routes them through liter-llm's MCP `complete` tool
```

### Provider Detection

`scripts/detect-providers.sh` in the bridge skill scans these env vars and reports class coverage:

| Class      | Providers Detected                                                             |
| ---------- | ------------------------------------------------------------------------------ |
| `frontier` | Anthropic, OpenAI, Google/Gemini, AWS Bedrock, Mistral (Large), OpenRouter     |
| `medium`   | Groq, Together, Mistral, Cohere, Fireworks, Anthropic Haiku, OpenAI mini tiers |
| `small`    | Ollama (local), vLLM (self-hosted), LM Studio, llama.cpp, Groq small models    |

The configure prompt picks the cheapest available provider per class, prioritizing local (Ollama) → fast hosted (Groq) → frontier APIs. You can override the auto-pick by editing `~/.config/liter-llm/config.toml` directly:

```toml
[aliases]
small    = "ollama/qwen2.5:7b"
medium   = "groq/llama-3.3-70b-versatile"
frontier = "anthropic/claude-sonnet-4-6"

[providers.anthropic]
api_key_env = "ANTHROPIC_API_KEY"

[providers.ollama]
base_url = "${OLLAMA_HOST:-http://localhost:11434}"
```

### Fallback and Safety Rules

- **No `model_policy` in `project.json`** → all phases default to `frontier`. The system never silently downgrades a frontier-required phase.
- **No provider configured for a `small`/`medium` class** → routing falls through to the host model with a warning. The cost reduction doesn't happen, but the phase still runs.
- **Non-frontier model attempts a `frontier`-required phase** → stops with `MODEL MISMATCH`. The cheap-runs-frontier-work case is the dangerous silent failure and is never permitted.

### Verifying It Works

```bash
# Status of the bridge config + which harnesses have it registered
bash skills/process/liter-llm-bridge/scripts/configure-mcp.sh status

# Provider coverage report (which classes have viable providers right now)
bash skills/process/liter-llm-bridge/scripts/detect-providers.sh | jq .coverage

# After running a KBD phase, audit dispatch counts by class
grep -h MODEL_ROUTING .kbd-orchestrator/phases/*/execution.md | \
  awk -F'class=' '{print $2}' | awk '{print $1}' | sort | uniq -c
```

If the frontier-class dispatch ratio exceeds ~30% of total dispatches, routing isn't effective — investigate which phases are misclassified and adjust their `Complexity score` annotations or `model_policy.phases` entries.

## Surreal-Memory Integration

All skills detect and use [surreal-memory](https://github.com/Prometheus-AGS/surreal-memory-server) for distributed state. Skills degrade gracefully when unavailable.

```bash
export SURREAL_MEMORY_URL=http://localhost:23001
```

See `shared/references/surreal-memory-integration.md` for entity mapping patterns.

## Cedar Governance

The `prometheus-cedar` crate and UAR's `GovernanceEngine` gate all skill mutations:

| Operation           | Cedar Action     | When                                       |
| ------------------- | ---------------- | ------------------------------------------ |
| Prompt optimization | `skill.mutate`   | dspy-rs writes back to SKILL.md            |
| Skill generation    | `skill.generate` | PMPO creates new skills from gap detection |
| Skill promotion     | `skill.promote`  | Generated skills promoted from staging     |
| Trace capture       | `trace.capture`  | Execution data collection                  |

Environment policies: **development** permits all, **staging** requires validation, **production** denies mutations. Additional policies for healthcare (audit trail) and financial (dual approval) verticals.

## Self-Learning Pipeline

```bash
# Default build — trace capture + evaluation + Cedar governance
cargo build --release -p prometheus-cli

# With knowledge compilation (requires prometheus-knowledge-rs)
cargo build --release -p prometheus-cli --features prometheus-learn/knowledge

# With prompt optimization (requires dspy-rs)
cargo build --release -p prometheus-cli --features prometheus-learn/optimize

# Full pipeline
cargo build --release -p prometheus-cli --features prometheus-learn/full
```

See `shared/references/self-learning-architecture.md` for the complete architecture.

## Sycophancy Correction

Every reflect and assess output is checked for the 8 canonical sycophancy patterns before storage. Research shows sycophancy appears in **58% of LLM interactions** and causes **r=0.902 correlation with abandoning correct answers** in multi-agent debate.

Key patterns enforced:

- **S-08 (Reflect Phase Inversion)**: Forces Delta → Root Cause → Corrective Actions
- **S-03 (Caveat Collapse)**: Requires at least one trade-off/risk/concern
- **S-02 (Agreement Without Grounding)**: Demands evidence for approval statements
- **S-04 (Self-Rationalization)**: Blocks self-congratulatory evaluation

The MCP server (`sycophancy-correction`) is installed globally and referenced in `.mcp.json`.

## Development

### Creating a New Skill

```bash
mkdir -p skills/{category}/{skill-name}
cp docs/SKILL_TEMPLATE.md skills/{category}/{skill-name}/SKILL.md
npm run validate:skill skills/{category}/{skill-name}
```

### Validation

```bash
npm run validate          # All 62 skills
npm run check-format      # Prettier formatting
```

### Rust CLI Development

```bash
cd tools/prometheus-cli
cargo check               # Type check
cargo clippy -- -D warnings  # Lint
cargo build --release     # Release build
```

## Project Structure

```
prometheus-skill-system/
├── .claude-plugin/plugin.json     # Claude Code plugin manifest
├── .mcp.json                      # MCP servers (surreal-memory, sycophancy, tavily)
├── .opencode/tools/               # OpenCode TypeScript tool definitions
├── .github/workflows/             # CI: validate + format + cargo check
├── hooks/hooks.json               # 5 hook events
├── policies/                      # Cedar governance policies + entities
├── agents/                        # Orchestration agents
├── marketplace/marketplace.json   # 5-entry marketplace distribution
├── skills/
│   ├── react/prometheus-entity-skills/  # 27 skills
│   ├── process/
│   │   ├── iterative-evolver/           # 8 skills + sycophancy checks
│   │   ├── kbd-process-orchestrator/    # 7 skills + sycophancy checks
│   │   └── pmpo-skill-creator/          # 5 skills + sycophancy correction pass
│   ├── devops/                          # 4 GitOps skills (TJ-CICD-001)
│   ├── testing/bdd-testing/             # 1 skill
│   └── imported/
│       ├── artifact-refiner/            # 9 skills (submodule)
│       └── sycophancy-correction/       # 1 skill + MCP server (submodule)
├── shared/references/             # Architecture docs, surreal-memory integration
├── tools/prometheus-cli/          # 4-crate Rust workspace
│   └── crates/
│       ├── prometheus-cli/        # 16-subcommand binary
│       ├── prometheus-agents/     # 10 platform adapters + TraceCapture
│       ├── prometheus-learn/      # Self-learning pipeline
│       └── prometheus-cedar/      # Cedar Skill Mutation PEP
├── scripts/
│   ├── validate-skills.js         # Recursive agentskills.io validator
│   ├── install-skills-flat.sh     # Flat symlink installer for slash commands
│   └── install-platforms.ts       # Multi-platform TypeScript installer
└── docs/                          # Templates, contributing guide
```

## License

MIT
