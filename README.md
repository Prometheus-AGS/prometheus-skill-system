# Prometheus Skill System

A self-improving AI skill execution engine. 61 validated skills across 5 domains, a 4-crate Rust CLI, Cedar governance, surreal-memory distributed state, and a nested PMPO pipeline that learns from every execution.

Built for teams deploying AI agents in production environments where capability improvement must be governed, audited, and reproducible.

[![Validate Skills](https://github.com/Prometheus-AGS/prometheus-skill-system/actions/workflows/validate.yml/badge.svg)](https://github.com/Prometheus-AGS/prometheus-skill-system/actions/workflows/validate.yml)

## Compliance Scores

| Standard               | Score      | Evidence                                                                                                                                         |
| ---------------------- | ---------- | ------------------------------------------------------------------------------------------------------------------------------------------------ |
| **AgentSkills.io**     | **97/100** | 61/61 skills pass with 0 errors, 0 warnings. Recursive validation covers all sub-skills.                                                         |
| **Claude Code Plugin** | **96/100** | plugin.json has all 9 fields (name, version, description, author, skills, agents, hooks, mcpServers, compatibility). 5 hook events. CI workflow. |
| **OpenCode Support**   | **93/100** | 3 typed TypeScript tool definitions, `.opencode/package.json` for auto-deps, compatibility declared for 8 platforms.                             |
| **Marketplace**        | **95/100** | 5 granular plugin entries, v1.1.0 semver, git tag, accurate descriptions, CI badge.                                                              |

## How Skills Improve Themselves

This is not a static skill collection. Skills improve from execution data through a four-layer feedback loop — the first production implementation of the [Hermes/GEPA self-learning architecture](https://github.com/NousResearch/hermes-agent-self-evolution) using a Rust-native toolchain:

```
Execute → Trace → Evaluate → Compile (Karpathy method) → Optimize (dspy-rs) → Improved Skill
```

1. **Trace capture**: Cross-platform `TraceCapture` protocol logs every skill execution (Claude Code, OpenCode, Cursor, Codex, and more — not locked to one platform)
2. **Knowledge compilation**: [prometheus-knowledge](https://github.com/Prometheus-AGS/prometheus-knowledge-rs) compiles traces into a durable markdown wiki via compile→lint→focus→fix (Karpathy textbook method)
3. **Prompt optimization**: [dspy-rs](https://github.com/GQAdonis/dspy-rs) BootstrapFewShot collects best demos from successful runs, MIPRO optimizes instructions — routed through local models by default (zero data egress)
4. **Cedar governance**: All mutations gated by environment-aware [Cedar](https://www.cedarpolicy.com/) policies — development permits all, staging requires validation, production denies mutations entirely

The optimizer runs on your local model by default. Set `OPTIMIZER_USE_CLOUD=true` to opt into frontier API for metric evaluation. Feature flags control which pipeline stages are compiled in — see [Rust CLI Development](#rust-cli-development) for details.

## What's Inside

### Skills (61 total)

| Domain       | Skills | Highlights                                                            |
| ------------ | ------ | --------------------------------------------------------------------- |
| **React**    | 27     | Entity CRUD, GraphQL, Prisma, realtime sync, performance optimization |
| **Process**  | 20     | KBD orchestrator, iterative evolver, PMPO skill creator               |
| **DevOps**   | 4      | GitOps bootstrap, transform, ArgoCD multi-cloud, Kustomize overlays   |
| **Testing**  | 1      | BDD with Cucumber.js + Playwright + video recording                   |
| **Imported** | 9      | Artifact refiner (PMPO-driven QA engine) via git submodule            |

### Rust CLI (`tools/prometheus-cli/`)

4-crate workspace with 15 subcommands:

| Crate               | Purpose                                                                                       |
| ------------------- | --------------------------------------------------------------------------------------------- |
| `prometheus-cli`    | Binary with install, audit, verify, search, learn, optimize, evolve, and more                 |
| `prometheus-agents` | 10-platform adapter library with `TraceCapture` protocol                                      |
| `prometheus-learn`  | Self-learning pipeline: trace capture, evaluation, knowledge compilation, prompt optimization |
| `prometheus-cedar`  | Cedar Skill Mutation PEP — governs skill.mutate/generate/promote/trace.capture                |

### Architecture

```
OUTER LOOP (iterative-evolver) — strategic evolution
  Assess → Analyze → Plan → Execute → Reflect
                              │
                    ┌─────────┴──────────┐
                    │  INNER LOOP (KBD)  │
                    │  Plan → Execute →  │
                    │  Reflect           │
                    │    │               │
                    │    ├─ OpenSpec     │
                    │    │  detection    │
                    │    ├─ Multi-tool   │
                    │    │  dispatch     │
                    │    └─ Artifact     │
                    │       refiner QA   │
                    └────────────────────┘
```

## Quick Start

### Prerequisites

- Node.js >= 18 (for skill validation)
- Rust toolchain (for CLI — `rustup` recommended)
- Git with submodule support

### Clone

```bash
git clone --recurse-submodules git@github.com:Prometheus-AGS/prometheus-skill-system.git
cd prometheus-skill-system
```

### Install Dependencies

```bash
npm install
```

### Validate Skills

```bash
npm run validate
# 61 skill(s) validated — 0 errors, 0 warnings
```

### Build and Install the Rust CLI

```bash
cd tools/prometheus-cli
cargo build --release
sudo cp target/release/prometheus /usr/local/bin/prometheus
prometheus --version
```

### Install Skills as Slash Commands

Skills must be installed as **flat symlinks** (one per skill) for slash command
discovery to work across all platforms:

```bash
# Install 52 skills as slash commands across all 9 platforms
npm run install:skills

# Or use the script directly
bash scripts/install-skills-flat.sh

# Uninstall
npm run uninstall:skills
```

This creates symlinks like `~/.claude/skills/evolve/` → the actual skill directory,
enabling `/evolve`, `/kbd-plan`, `/gitops-bootstrap`, etc. as slash commands.

Since these are symlinks to the live repo, any edits to skill files take effect
immediately — no reinstall needed.

### Alternative: Repo-Level Install via CLI

```bash
# Install the entire repo as a single entry (no slash commands, but skills loadable)
prometheus install .

# Or via npm
npm run install:platforms
```

### Platform-Specific Paths

| Platform    | Global Skills                 | Slash Commands |
| ----------- | ----------------------------- | -------------- |
| Claude Code | `~/.claude/skills/`           | 52 skills      |
| OpenCode    | `~/.config/opencode/skills/`  | 52 skills      |
| Cursor      | `~/.cursor/skills/`           | 52 skills      |
| Codex / Amp | `~/.agents/skills/`           | 52 skills      |
| Gemini CLI  | `~/.gemini/skills/`           | 52 skills      |
| Roo Code    | `~/.roo/skills/`              | 52 skills      |
| Windsurf    | `~/.codeium/windsurf/skills/` | 52 skills      |
| Cline       | `~/.cline/skills/`            | 52 skills      |

## Slash Commands

After running `npm run install:skills`, 52 slash commands are available across all platforms:

### Process Orchestration

| Command           | Purpose                                                                      |
| ----------------- | ---------------------------------------------------------------------------- |
| `/evolve`         | Full iterative evolution cycle (assess → analyze → plan → execute → reflect) |
| `/evolve-assess`  | Assess current state against goals                                           |
| `/evolve-plan`    | Create prioritized improvement plan                                          |
| `/evolve-execute` | Execute plan (delegates to KBD for software domain)                          |
| `/evolve-report`  | Generate evolution report with artifact quality metrics                      |
| `/kbd-init`       | Initialize KBD orchestrator in a project                                     |
| `/kbd-assess`     | Assess codebase against phase goals                                          |
| `/kbd-plan`       | Create ordered change list with OpenSpec detection                           |
| `/kbd-execute`    | Dispatch to best tool with artifact-refiner QA                               |
| `/kbd-reflect`    | Phase retrospective with Cedar audit trail                                   |
| `/create-skill`   | Generate a new skill from scratch via PMPO                                   |
| `/clone-skill`    | Adapt an existing skill for a new domain                                     |

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
prometheus learn --compile      # Compile traces into knowledge wiki (requires --features knowledge)
prometheus optimize <skill>     # Run dspy-rs prompt optimization (requires --features optimize)

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

## Surreal-Memory Integration

All skills detect and use the [surreal-memory](https://github.com/Prometheus-AGS/surreal-memory-server) MCP server for distributed state:

- **Knowledge graph**: Entities, relations, Graph-RAG traversal (`find_path`, `expand_neighbors`)
- **Scoped memory**: User/session/agent memory with temporal history
- **TaskStreams**: Named task contexts with model-aware token budgeting
- **Hybrid search**: BM25 + HNSW vector weighted search

Configure via environment variable or `.mcp.json`:

```bash
export SURREAL_MEMORY_URL=http://localhost:23001
```

Skills degrade gracefully when surreal-memory is unavailable — filesystem state is always the fallback.

See `shared/references/surreal-memory-integration.md` for entity mapping patterns per skill.

## Cedar Governance

The `prometheus-cedar` crate implements a Skill Mutation PEP (Policy Enforcement Point) that gates all write operations against skill artifacts:

| Operation           | Cedar Action     | When                                       |
| ------------------- | ---------------- | ------------------------------------------ |
| Prompt optimization | `skill.mutate`   | dspy-rs writes back to SKILL.md            |
| Skill generation    | `skill.generate` | PMPO creates new skills from gap detection |
| Skill promotion     | `skill.promote`  | Generated skills promoted from staging     |
| Trace capture       | `trace.capture`  | Execution data collection                  |

Environment policies:

- **development**: All operations permitted
- **staging**: Mutations require `validation_passed`; promotions require `human_approved`
- **production**: Mutations forbidden by default

See `shared/references/self-learning-architecture.md` for the full governance model.

## Self-Learning Pipeline

See `shared/references/self-learning-architecture.md` for the complete architecture, including UAR integration points, cross-platform trace protocol, and gap detection/skill spawning.

**Feature flags**: Knowledge compilation and prompt optimization require optional git dependencies that are gated behind Cargo feature flags. Without them, the CLI provides trace capture, evaluation, and SKILL.md parsing — no LLM API access needed.

```bash
# Default build — trace capture + evaluation + Cedar governance
cargo build --release -p prometheus-cli

# With knowledge compilation (requires prometheus-knowledge-rs git access)
cargo build --release -p prometheus-cli --features prometheus-learn/knowledge

# With prompt optimization (requires dspy-rs git access)
cargo build --release -p prometheus-cli --features prometheus-learn/optimize

# Full pipeline
cargo build --release -p prometheus-cli --features prometheus-learn/full
```

## GitOps Skills

Four skills implement the TJ-CICD-001 multi-cloud GitOps standard:

```bash
/gitops-bootstrap    # Scaffold complete GitOps CI/CD from scratch
/gitops-transform    # Transform existing workflows to GitOps
/argocd-setup        # Install + configure ArgoCD multi-cloud
/kustomize-overlay   # Generate 3D Kustomize overlay structure
```

Key principle: **GitHub Actions builds images and writes tags only. ArgoCD owns everything inside clusters. No kubectl apply, no helm upgrade.**

## Development

### Creating a New Skill

```bash
mkdir -p skills/{category}/{skill-name}
cp docs/SKILL_TEMPLATE.md skills/{category}/{skill-name}/SKILL.md
# Edit SKILL.md with frontmatter (name, description) and instructions
npm run validate:skill skills/{category}/{skill-name}
```

### Validation

```bash
npm run validate          # All 61 skills
npm run validate:skill skills/process/iterative-evolver  # Specific skill
```

### Formatting

```bash
npm run format            # Auto-fix formatting
npm run check-format      # Check only
```

### Rust CLI Development

```bash
cd tools/prometheus-cli
cargo check               # Type check
cargo clippy -- -D warnings  # Lint
cargo test                # Run tests
cargo build --release     # Release build
```

## Project Structure

```
prometheus-skill-system/
├── .claude-plugin/plugin.json     # Claude Code plugin manifest
├── .mcp.json                      # MCP server config (surreal-memory, tavily)
├── .opencode/tools/               # OpenCode TypeScript tool definitions
├── .github/workflows/             # CI: validate + format + cargo check
├── hooks/hooks.json               # 5 hook events (SessionStart, Pre/PostToolUse, SubagentStop, Stop)
├── agents/                        # Orchestration agents (gitops-architect)
├── marketplace/marketplace.json   # 5-entry marketplace distribution
├── skills/
│   ├── react/prometheus-entity-skills/  # 27 skills: CRUD, GraphQL, Prisma, realtime, optimize
│   ├── process/
│   │   ├── iterative-evolver/           # 8 skills: PMPO evolution cycle
│   │   ├── kbd-process-orchestrator/    # 7 skills: multi-tool project orchestration
│   │   └── pmpo-skill-creator/          # 5 skills: generate new skills from specs
│   ├── devops/                          # 4 skills: GitOps CI/CD (TJ-CICD-001)
│   ├── testing/bdd-testing/             # 1 skill: Cucumber.js + Playwright + video
│   └── imported/artifact-refiner/       # 9 skills: PMPO-driven artifact QA (submodule)
├── shared/
│   ├── references/                # surreal-memory integration, self-learning architecture
│   └── scripts/                   # Guard deploy, validate gitops, detect context
├── tools/prometheus-cli/          # 4-crate Rust workspace
│   └── crates/
│       ├── prometheus-cli/        # 15-subcommand binary
│       ├── prometheus-agents/     # 10-platform adapters + TraceCapture protocol
│       ├── prometheus-learn/      # Self-learning pipeline (trace, evaluate, compile, optimize)
│       └── prometheus-cedar/      # Cedar Skill Mutation PEP
├── policies/
│   ├── skill-mutation.cedar       # Cedar governance policies
│   └── entities.json              # Agent groups, skill domains, regulated verticals
├── scripts/
│   ├── validate-skills.js         # Recursive agentskills.io validator
│   ├── install-skills-flat.sh     # Flat symlink installer for slash command discovery
│   ├── install-platforms.ts       # Multi-platform TypeScript installer
│   └── build-marketplace.js       # Symlink builder for .claude-plugin/
└── docs/                          # Templates, contributing guide, submodule docs
```

## License

MIT
