# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Documentation hierarchy

This file is the **canonical rules source** for the Prometheus engineering stack.
All sibling repositories defer to it for cross-cutting concerns.

| Repository | CLAUDE.md scope | Canonical source |
|---|---|---|
| `prometheus-skill-pack` (this file) | Pack architecture, KBD lifecycle, skill discovery, OpenSpec, BDD rules, progress signaling | **HERE — canonical** |
| `prometheus-knowledge` | Rust workspace, crate architecture, model routing, librarian patterns | Crate-specific only; defer to this file for project-wide rules |

**Precedence:** when a rule in a sibling CLAUDE.md conflicts with this file, this file wins. Add new project-wide rules here, not in sibling files.

## Project Overview

This is a comprehensive, enterprise-grade skills package collection for AI-assisted development. The repository manages centralized Agent Skills across multiple domains (React, Rust, UI/UX, DevOps, Testing, Documentation) with full compliance to the [agentskills.io](https://agentskills.io/specification) standard and Claude Code plugin marketplace requirements.

**Key Characteristics**:

- Multi-domain skill collection with unified management
- Dual-format support: standalone agentskills.io + Claude Code plugin
- Shared utilities, scripts, and templates across skills
- Automated validation and marketplace distribution
- Portable across AI platforms (Claude Code, Kimi Code, MiniMax/Mavis, OpenCode, Codex, Cursor, Windsurf, Gemini CLI, and more)

## Memory — Check Before You Code, Write After You Ship

**This is mandatory, not optional.**

### 1. Check memory at the start of every session

Before writing any code or making any changes, look up relevant context using the first available tool in this priority order:

1. **surreal-memory MCP** (preferred) — available when `create_entity`, `add_memory`, `search_memories`, or `semantic_search` tools are present in the session
2. **Cortex MCP** — available when `cortex_recall` / `cortex_remember` tools are present
3. **File-based memory** — always available at `~/.claude/projects/-Users-gqadonis-Projects-prometheus-prometheus-skill-pack/memory/`

If surreal-memory is listed in `.mcp.json` but tools are absent from the session, use the next available option — never skip memory entirely.

**Lookup queries to run at session start:**
```
# surreal-memory
semantic_search("prometheus-skill-pack recent work")
search_memories(query="<current task topic>", user_id="prometheus-skill-pack")

# Cortex
cortex_recall("recent work prometheus-skill-pack")
cortex_recall("<current task topic>")

# File
Read ~/.claude/projects/-Users-gqadonis-Projects-prometheus-prometheus-skill-pack/memory/MEMORY.md
```

### 2. Write memories after every feature or bug fix

After completing any non-trivial task, immediately create memories. Use this distinction:

| Type | When | surreal-memory scope | Cortex flag | File memory `type:` |
|------|------|---------------------|-------------|---------------------|
| **Global** | Pattern applies to any Rust/skill project | `user_id="global"` | `global=true` | `type: feedback` with "GLOBAL" in name |
| **Project** | Specific to this repo's files/crates/phases | `user_id="prometheus-skill-pack"` | `projectId="prometheus-skill-pack"` | `type: project` |

**What to record:**
- Architecture decisions and why they were made
- Gotchas, bugs, and their fixes (with file paths for project-specific ones)
- Patterns that were validated in this codebase
- Anything that would have saved time if known at the start

### 3. surreal-memory tool reference

When surreal-memory tools are available, use these for structured memory:

```
# Knowledge graph — for architectural entities and relationships
create_entity(name, entityType, observations)
add_observations(entityName, observations)
create_relation(from, to, relationType)
search_entities(query)
semantic_search(query)

# Scoped memory — for session insights and lessons
add_memory(content, user_id, metadata)
search_memories(query, user_id)
hybrid_search_memories(query, user_id)

# Graph traversal
find_path(from, to)
expand_neighbors(entityName)
```

## Essential Commands

### Skill-Pack Management (cowork CLI — preferred)

The `cowork` CLI (alias: `co`) is the primary utility for installing, updating,
and repairing the prometheus-skill-pack across all platforms. It is installed by
`scripts/install-binaries.sh` to `~/.local/bin/cowork`.

```bash
# Install pack to all detected platforms on a new machine
cowork install --source .

# Check skill-pack status (git HEAD, platform links, symlink health)
cowork pack status

# Pull latest commits + re-install to all platforms
cowork pack update

# Repair broken symlinks / stale platform configs
cowork pack repair

# Check toolchain health (Rust, Node, git, cargo-dist)
cowork toolchain status

# Scan for reclaimable build artifacts (delegates to dsg)
cowork disk scan

# Clean stale artifacts (PREVIEW only — use --force to actually trash)
cowork disk clean --dry-run
cowork disk clean --force   # moves to system Trash

# Full health check + auto-fix suggestions
cowork doctor
```

See [`skills/process/cowork-management/SKILL.md`](skills/process/cowork-management/SKILL.md) for the full cowork guide
and [`skills/process/cowork-management/references/COMMANDS.md`](skills/process/cowork-management/references/COMMANDS.md) for the complete command reference.

#### Direct dsg commands (disk-space-guardian CLI)

```bash
# Show disk space summary (reclaimable by ecosystem)
dsg status

# Scan all ecosystems for stale artifacts
dsg scan

# Deep scan — recurse into all home subdirectories
dsg scan --deep

# Preview what would be cleaned (dry-run, never deletes)
dsg clean --dry-run

# Actually clean (moves to system Trash — recoverable)
dsg clean --force

# Clean one ecosystem only
dsg clean --force --ecosystem rust
dsg clean --force --ecosystem node

# JSON output — pipe to pk ingest or log aggregator
dsg scan --json
```

See [`skills/devops/disk-space-guardian/SKILL.md`](skills/devops/disk-space-guardian/SKILL.md) for full documentation and safety rules.

---

### Cross-Platform Installation (shell scripts — lower-level)

```bash
# Install skills to ALL detected platforms (Claude Code, Kimi, MiniMax, OpenCode, Codex, Cursor, etc.)
# Also configures MCP servers (surreal-memory, sycophancy-correction) in platform configs
bash scripts/install-skills-flat.sh

# Uninstall from all platforms
bash scripts/install-skills-flat.sh --uninstall

# Install to specific platforms with full plugin support (OpenCode, Kimi config.toml)
npm run install:platforms

# Check toolchain + service status on any platform
bash shared/scripts/detect-toolchain.sh

# Machine-readable toolchain status
bash shared/scripts/detect-toolchain.sh --json
```

**Platform skill directories:**
- Claude Code: `~/.claude/skills/`
- Kimi Code: `~/.kimi-code/skills/` + MCP config at `~/.kimi-code/config.toml`
- MiniMax: `~/.minimax/skills/` + MCP config at `~/.minimax/mcp/mcp.json`
- OpenCode: `~/.opencode/skills/`
- Codex: `~/.codex/skills/`
- Cursor: `~/.cursor/skills/`

### Submodule Management

```bash
# Initialize submodules (for new clones)
git submodule init
git submodule update

# Update all imported skills to latest
git submodule update --remote

# Update specific imported skill
cd skills/imported/artifact-refiner && git pull origin main && cd ../../..

# Check submodule status
git submodule status
```

### Validation

```bash
# Validate all native skills (excludes imported/ submodules)
npm run validate

# Strict validation — required for new skills (enforces version, license, metadata.tags)
npm run validate:strict

# Validate a specific skill (strict mode)
npm run validate:strict skills/react/skill-name

# Validate a specific skill (lenient mode, includes imported)
npm run validate:skill skills/imported/artifact-refiner

# Check code formatting
npm run check-format

# Auto-fix formatting
npm run format
```

### Build & Distribution

```bash
# Build marketplace distribution (creates symlinks in .claude-plugin/)
npm run build

# Install skills to user scope (~/.claude/skills/)
npm run install:user

# Install skills to project scope (.claude/skills/)
npm run install:project
```

### Testing

```bash
# Run skill tests
npm test

# Lint all skills
npm run lint

# Watch mode for development
npm run dev
```

## Architecture

### Directory Organization

```
prometheus-skill-pack/
├── .claude-plugin/          # Claude Code plugin format
│   ├── plugin.json         # Plugin manifest
│   ├── skills/             # Symlink -> ../skills/
│   ├── agents/             # Symlink -> ../agents/
│   └── hooks/              # Symlink -> ../hooks/
│
├── skills/                 # Main skills directory (agentskills.io)
│   ├── imported/           # Git submodule skills from external repos
│   │   ├── artifact-refiner/  # Submodule: PMPO artifact refinement
│   │   └── README.md       # Imported skills documentation
│   ├── react/              # React domain skills
│   ├── rust/               # Rust domain skills
│   ├── ui-ux/              # UI/UX domain skills
│   ├── devops/             # DevOps domain skills
│   ├── testing/            # Testing domain skills
│   ├── documentation/      # Documentation domain skills
│   └── learn/              # Learn domain skills (v1.5.0)
│       ├── ui-surface/     # Cross-harness UI rendering primitive
│       ├── learn-goal/     # Learning desire + feasibility gate
│       ├── learn-survey/   # Diagnostic placement + recursion floor
│       ├── learn-plan/     # Concept DAG + curriculum builder
│       ├── feynman-loop/   # Core Feynman PMPO loop
│       ├── learn-grade/    # Sycophancy-corrected external grader
│       ├── learn-retain/   # FSRS-6 spaced retrieval
│       ├── learn-practice/ # Deliberate practice track
│       ├── learn-certify/  # OB 3.0 / W3C VC certification
│       ├── learn-kb/       # KB registry + adapter management
│       ├── learn-about-system/ # Prometheus stack meta-learning
│       ├── learn-harness/  # Harness detection + capability map
│       ├── sync-status/    # P2P sync node status
│       ├── sync-peers/     # P2P peer management
│       └── sync-push/      # Push CRDT domain to peers
│
├── substrate/              # Rust crates for learn domain and research
│   ├── storage-provider/   # StorageProvider + CrdtEngine traits + SyncManifest
│   ├── learner-model/      # CRDT learner model + FSRS-6 scheduler
│   ├── surface-bridge/     # Axum MCP App server (Tier 2 UI)
│   ├── sovereign-sync/     # P2P CRDT daemon + MCP server + REST API (v1.5.0)
│   ├── sovereign-client/   # Rust SDK for sovereign-sync REST + SSE
│   └── prometheus-research/ # HTTP+MCP research server on :7891 with AG-UI SSE (v1.6.0)
│
├── shared/                 # Shared resources across all skills
│   ├── scripts/            # Reusable scripts
│   │   ├── validators/     # Validation utilities
│   │   ├── generators/     # Code generation
│   │   ├── formatters/     # Formatting utilities
│   │   └── parsers/        # File parsing
│   ├── templates/          # Reusable file templates
│   └── utils/              # Helper functions
│
├── agents/                 # Specialized subagents
├── hooks/                  # Automation hooks
├── marketplace/            # Marketplace configuration
│   └── marketplace.json    # Distribution manifest
├── scripts/                # Build and validation tools
│   ├── validate-skills.js  # AgentSkills.io validator
│   ├── build-marketplace.js # Symlink builder
│   └── install.js          # Installation script
└── docs/                   # Documentation
    ├── SKILL_TEMPLATE.md   # Template for new skills
    └── CONTRIBUTING.md     # Contribution guidelines
```

### Dual-Format Support

This repository supports two distribution formats simultaneously:

1. **AgentSkills.io Standard** (`skills/` directory):
   - Portable across all AI platforms
   - Direct directory structure
   - Standard format: `SKILL.md`, `scripts/`, `references/`, `assets/`
   - Can be copied to any `~/.claude/skills/` or `.github/skills/` location

2. **Claude Code Plugin** (`.claude-plugin/` directory):
   - Enhanced with `plugin.json` manifest
   - Supports hooks, agents, MCP servers
   - Marketplace distribution via Git
   - Uses symlinks to maintain single source of truth

**Important**: The `skills/` directory is the source of truth. The `.claude-plugin/` directory contains symlinks created by `npm run build`.

**Canonical hooks path**: `hooks/hooks.json` is the physical source of truth for hook definitions. Claude Code auto-loads `hooks/hooks.json` from the plugin root by default, so `plugin.json` must NOT also declare `"hooks": "./hooks/hooks.json"` — that duplicates the default path and fails plugin load with "Duplicate hooks file detected". `.claude-plugin/hooks → ../hooks` is a directory symlink kept only so `.claude-plugin/` mirrors the full plugin layout on disk; it plays no role in hook loading. Always edit `hooks/hooks.json` directly — never edit through `.claude-plugin/hooks/hooks.json`. CI validates the symlink on every PR (`hooks-integrity` job in `.github/workflows/validate.yml`).

### Imported Skills (Git Submodules)

The repository includes a third category for **imported skills** - skills maintained in external repositories:

- **Location**: `skills/imported/`
- **Management**: Git submodules
- **Purpose**: Skills with independent development lifecycles
- **Updates**: Can be updated from their source repositories
- **Versioning**: Can be pinned to specific versions or track latest

Current imported skills:

- `skills/imported/artifact-refiner/` - PMPO-driven artifact refinement engine (v1.1.0)

See `docs/SUBMODULES.md` for complete submodule management guide.

### Shared Resources Pattern

Skills can reference shared utilities via environment variables:

```markdown
## In SKILL.md

Run validation:
\`\`\`bash
bash ${CLAUDE_PLUGIN_ROOT}/shared/scripts/validators/validate-config.sh config.json
\`\`\`
```

Available variables:

- `$CLAUDE_PLUGIN_ROOT` - Root of plugin directory
- `$REPO_ROOT` - Repository root
- `$HOME` - User home directory

## Skill Development Workflow

### Creating a New Skill

1. **Choose category**: Place in appropriate `skills/{category}/` directory (react, rust, ui-ux, devops, testing, documentation, process, learn, or a new category)

2. **Create directory** with kebab-case naming:

   ```bash
   mkdir -p skills/react/react-entity-crud
   cd skills/react/react-entity-crud
   ```

3. **Create `SKILL.md`** using template:

   ```bash
   cp ../../docs/SKILL_TEMPLATE.md SKILL.md
   ```

4. **Edit frontmatter** (required fields):

   ```yaml
   ---
   name: react-entity-crud
   description: Complete CRUD operations for React entity management with hooks and TypeScript
   license: MIT
   metadata:
     author: your-name
     version: '1.0.0'
     category: react
     tags: [react, crud, entity, typescript]
   ---
   ```

5. **Write instructions** following these principles:
   - Keep main file under 500 lines
   - Use third-person voice ("Run the command", not "You should run")
   - Include concrete examples
   - Move detailed content to `references/` directory
   - Use forward slashes for all paths

6. **Add optional directories**:

   ```bash
   mkdir -p scripts references assets
   # scripts/    - Executable code
   # references/ - Detailed documentation
   # assets/     - Templates, schemas, examples
   ```

7. **Validate**:

   ```bash
   npm run validate:skill skills/react/react-entity-crud

   # New skills must also pass strict validation
   npm run validate:strict skills/react/react-entity-crud
   ```

8. **Test locally**:
   ```bash
   npm run install:project
   # Then test in Claude Code with /skill-name
   ```

### Modifying Existing Skills

When updating skills:

1. **Read current state**: Always read `SKILL.md` before modifying
2. **Preserve structure**: Maintain existing section organization
3. **Validate changes**: Run `npm run validate:skill` after edits
4. **Check references**: Update `references/` files if structure changes
5. **Version bump**: Update `metadata.version` for significant changes

## AgentSkills.io Compliance

This repository strictly adheres to the [agentskills.io specification](https://agentskills.io/specification):

### Required Elements

- ✅ `SKILL.md` with YAML frontmatter
- ✅ `name` field: lowercase, hyphens, max 64 chars, pattern `^[a-z0-9]+(-[a-z0-9]+)*$`
- ✅ `description` field: 1-1024 characters, searchable
- ✅ `version` field: semver string, e.g., `'1.0.0'`
- ✅ `license` field: SPDX identifier, e.g., `MIT`
- ✅ `metadata.tags` field: non-empty array of searchable keywords

### Standard Directories

- ✅ `scripts/` - Executable code (optional)
- ✅ `references/` - Documentation loaded on demand (optional)
- ✅ `assets/` - Templates, resources (optional)

### Best Practices Enforced

- ✅ Forward slashes only (never backslashes)
- ✅ Self-contained scripts with package runners (`npx`, `uvx`, `bunx`)
- ✅ Progressive disclosure (main file + references)
- ✅ Structured output from scripts (JSON preferred)
- ✅ Executable permissions on scripts (`chmod +x`)

### Validation

The validator (`scripts/validate-skills.js`) checks:

- YAML frontmatter syntax and schema
- Required fields presence and format
- Name/directory consistency
- Path separator style (forward slashes)
- Script executability
- File structure compliance

## Important Conventions

### Naming

- **Skills**: `kebab-case` only, e.g., `react-entity-crud`
- **Files**: Forward slashes in all paths
- **Scripts**: Executable with `.sh`, `.py`, `.js` extensions

### Skill Size

- **Main SKILL.md**: Keep under 500 lines
- **Progressive disclosure**: Split large content to `references/`
- **Context efficiency**: Skills use lazy-loading architecture

### Script Requirements

- **Self-contained**: Use inline dependency declarations or package runners
- **Cross-platform**: Avoid platform-specific commands
- **Structured output**: JSON when possible for programmatic parsing
- **Error handling**: Non-zero exit codes on failure

### Documentation

- **Third person**: "Run the command" not "You should run"
- **Concrete examples**: Always include working examples
- **When to use**: Describe triggering scenarios clearly
- **No assumptions**: Document all prerequisites

### Shared Resources

- **Location**: `shared/{scripts,templates,utils}/`
- **Reference**: Use `${CLAUDE_PLUGIN_ROOT}/shared/...`
- **Documentation**: Maintain README in each shared directory
- **Reusability**: Prefer shared utilities over duplication

## Marketplace Distribution

The marketplace is configured for Git-based distribution:

1. **Source**: `marketplace/marketplace.json` with frontmatter
2. **Plugins**: Defined as Git repository references
3. **Granularity**: Full package or individual domain packages
4. **Installation**: Users run `/plugin marketplace add Prometheus-AGS/prometheus-skill-system`

### Publishing Checklist

Before releasing:

- [ ] All skills validate strict: `npm run validate:strict`
- [ ] Marketplace builds: `npm run build`
- [ ] Version bumped in `package.json`, `plugin.json`, and every plugin entry in `marketplace/marketplace.json`
- [ ] CHANGELOG updated
- [ ] README reflects new skills
- [ ] Git tag created: `git tag v1.x.x`

## Testing Strategy

### Validation Testing

```bash
# Standard validation (native skills only, 0 errors required)
npm run validate

# Strict validation — gate for new skills (enforces version, license, metadata.tags)
npm run validate:strict

# Specific skill — strict mode (required before submitting a new skill)
npm run validate:strict skills/category/name

# Specific skill — lenient mode (for submodule or legacy skills)
npm run validate:skill skills/category/name
```

### Integration Testing

```bash
# Install to test environment
npm run install:project

# Test in Claude Code
# 1. Launch Claude Code
# 2. Run /reload-plugins
# 3. Try /skill-name or let AI auto-trigger
```

### Manual Testing Checklist

- [ ] Skill triggers on appropriate prompts
- [ ] Instructions are clear and actionable
- [ ] Examples work as documented
- [ ] Scripts execute successfully
- [ ] References load correctly
- [ ] No Windows-style paths present

## Common Patterns

### Skill with Scripts

```markdown
---
name: my-skill
description: Does something useful
---

# My Skill

## Instructions

1. Validate input:
   \`\`\`bash
   bash scripts/validate.sh input.json
   \`\`\`

2. Process data:
   \`\`\`bash
   python3 scripts/process.py --input input.json --output output.json
   \`\`\`
```

### Skill with References

```markdown
---
name: complex-skill
description: Complex workflow with detailed docs
---

# Complex Skill

## Quick Start

[Basic instructions here - keep under 500 lines]

## Detailed Documentation

For in-depth information:

- [Conceptual Guide](references/CONCEPTS.md)
- [API Reference](references/API.md)
- [Extended Examples](references/EXAMPLES.md)
```

### Skill Using Shared Scripts

```markdown
---
name: validated-skill
description: Skill with validation
---

# Validated Skill

## Instructions

1. Validate configuration:
   \`\`\`bash
   bash ${CLAUDE_PLUGIN_ROOT}/shared/scripts/validators/validate-json.sh config.json
   \`\`\`
```

## BDD Immutable-Tests Rule

Code-generation agents operating in projects that use Cucumber/BDD step definitions **may not edit existing tests** to make failing tests pass.

- **May not** edit `tests/steps/*.steps.ts`, `tests/support/*.ts`, or `tests/features/*.feature` as part of code changes.
- **Must** surface failing tests to the user rather than silently updating steps to match new code.
- **May** add new `.feature` files to `tests/features/drafts/` and matching new step definitions.

This rule is enforced two ways:

1. **PreToolUse hook** (agent-time) — `shared/scripts/protect-tests.sh` blocks `Edit`/`Write` on protected paths.
2. **CI gate** (PR-time) — `skills/testing/bdd-lifecycle-loop/scripts/test-file-diff-guard.sh` fails PRs touching protected paths without a `test-change-approved` label.

**Canonical guidance now lives in the operative skill** — see [`skills/testing/bdd-lifecycle-loop/references/immutable-tests.md`](skills/testing/bdd-lifecycle-loop/references/immutable-tests.md) rather than repeating the rationale here. That skill also documents the four-phase BDD loop (author → run → triage → maintain), the flake-budget enforcement, and the visual-baseline refresh workflow.

Related future-work docs (background reading):
- [`BDD-005 testid-drift-detection`](docs/future-work/02-bdd-testing-evolution/BDD-005-testid-drift-detection.md)
- [`BDD-006 immutable-tests-rule`](docs/future-work/02-bdd-testing-evolution/BDD-006-immutable-tests-rule.md)
- [`BDD-007 candidate-test-drafts`](docs/future-work/02-bdd-testing-evolution/BDD-007-candidate-test-drafts.md)

For downstream projects using this skill-pack with BDD suites (e.g. `ssr-frontend`), see the **Immutable Tests Rule** section in that project's `CLAUDE.md`.

## Learn Domain

The learn domain adds a Feynman-Spine learning and education capability to the skill pack. It is architected in four layers so that skills remain portable across all harnesses while substrate crates handle persistence and UI rendering.

### Four-Layer Architecture

| Layer | Location | Purpose |
|---|---|---|
| **A — Substrate** | `substrate/` | Rust crates: storage-provider, learner-model, surface-bridge, sovereign-sync, sovereign-client, prometheus-research |
| **B — UI primitive** | `skills/learn/ui-surface` | Cross-harness rendering via surface tier detection |
| **C — Learning skills** | `skills/learn/` | 12 skills composing the full learning arc |
| **D — KB adapters** | `shared/scripts/content-grounding-kb.sh` | Privacy-safe custom knowledge base integration |

### Substrate Crates

- **`storage-provider`** — `StorageProvider` and `CrdtEngine` traits; `LocalDirAdapter` (default); `SyncManifest` + `SyncDomain` + `PrivacyClass` (structural KB-content privacy enforcement); `IrohDocsAdapter` for P2P-backed storage
- **`learner-model`** — Loro 1.13 CRDT learner model (mastery per concept, FSRS-6 cards, gap records); simplified FSRS-6 scheduler; JSON-RPC `stdin`/`stdout` interface; PFA mastery update (`mastery_new = mastery_old + 0.3 × (score - mastery_old)` at ≥5 observations)
- **`surface-bridge`** — Axum HTTP server on `127.0.0.1:7890`; routes: `/health`, `/mcp/detect-surface-tier`, `/mcp/render-ui-intent`, `/mcp/collect-response`; installed as a macOS launchd service via `install-skills-flat.sh`
- **`sovereign-sync`** — P2P CRDT sync daemon, MCP server, and REST API on `127.0.0.1:7892`; iroh 1.0 + iroh-gossip 0.101 for QUIC P2P transport; Loro 1.13 for CRDT merge; rmcp 1.8 for MCP server (stdio); redb 2 for persistence; AG-UI SSE endpoint for Tauri/web clients; modes: `--mode mcp|daemon|server`; launchd service via `install-skills-flat.sh`
- **`sovereign-client`** — Rust SDK for `sovereign-sync` REST API + AG-UI SSE; reqwest 0.12 + eventsource-stream 0.2; `SovereignClient::new(base_url)` entry point
- **`prometheus-research`** — Background deep-research daemon (v1.6.0); HTTP server on `127.0.0.1:7891`; 5 MCP tools (research_start/status/cancel/export, render_component); AG-UI SSE event stream; A2UI component registry with 8 server-rendered HTMX fragments; HTMX 2.0.8 + htmx-ext-sse 2.2.2 + Alpine.js 3.14.8 vendored; launchd auto-start via `com.prometheus.research.plist`; installed by `scripts/install-binaries.sh`

### Surface Tier Degradation Contract

All learn skills present through `ui-surface`, which resolves one of three tiers:

| Tier | Harness | Mechanism |
|---|---|---|
| 0 | Universal | Plain text / markdown (always works) |
| 1 | Claude Code | `AskUserQuestion`; elsewhere: file-pair (`__ui_intent__.json` / `__ui_response__.json`) |
| 2 | Any with surface-bridge | MCP App iframe via `http://127.0.0.1:7890` |

Skills MUST NOT render directly — they emit a `UiIntent` to `ui-surface`, which resolves the tier.

### KB Adapter Pattern

Four adapter prefixes for `learn-kb add` and `learn-goal --kb`:

| Prefix | Backend | Privacy |
|---|---|---|
| `dify:<kb-name>` | Dify knowledge base MCP | Dify server, requires DIFY_API_KEY |
| `palace:<collection>` | surreal-memory palace RAG | Fully local, no external calls |
| `local:<path>` | Filesystem markdown | Stays on machine |
| `web:<url>` | Firecrawl live fetch | Internet required |

`content-grounding-kb.sh` NEVER forwards KB content to external APIs. It warns if external API env vars (FIRECRAWL_API_KEY, etc.) are set and skips those sources in KB mode.

### Essential Learn Commands

```bash
# Start a learning session
/learn-goal "I want to master X"

# Check current harness capability
/learn-harness

# Learn about the Prometheus stack itself
/learn-about-system --area kbd
/learn-about-system --area skills
/learn-about-system --area harness

# Manage custom knowledge bases
/learn-kb add dify:my-legal-kb
/learn-kb add local:/path/to/clinical-protocols

# Build and install substrate (Rust + launchd)
bash scripts/install-skills-flat.sh

# Check substrate status
bash shared/scripts/detect-toolchain.sh

# Check sovereign-sync P2P status
/sync-status

# Manage P2P peers
/sync-peers

# Push a sync domain to peers
/sync-push skill-index
/sync-push learner-model

# Start sovereign-sync daemon manually (port 7892)
sovereign-sync --mode daemon

# Check daemon health
curl -s http://127.0.0.1:7892/health | jq .
```

### Mastery Criterion

All three conditions are required to close a Feynman loop on a concept:

1. `learn-grade` passes: overall score ≥ 0.7 AND `misconceptions_absent == 1.0`
2. Two novel transfer problems solved at ≥ 0.7
3. Retention check via `learn-retain` at ≥ 24 h interval

Self-reported fluency NEVER closes a loop. Pedagogical sycophancy (making the learner feel good at the cost of accurate feedback) is blocked architecturally by routing `learn-grade` through sycophancy-correction S-02.

### Anti-Sycophancy in Learning

The sycophancy-correction skill is on the critical path of the core loop:

1. `learn-grade` drafts a grade
2. Grade is routed through sycophancy-correction S-02 check
3. A grade that says "no gaps" when gaps are present is **rewritten before delivery**

This is enforced architecturally, not as optional guidance. Pedagogical sycophancy produces worse learning outcomes.

## Karpathy LLM Wiki (pk) — Open Knowledge Format Adoption

The "Karpathy LLM wiki" pattern — an LLM-maintained, persistent, interlinked
markdown knowledge base sitting between raw sources and the agent — is
implemented by the `pk` CLI, shipped from the separate
**prometheus-knowledge-rs** repository
(`github.com/Prometheus-AGS/prometheus-knowledge-rs`). This repo does not vendor
that source; it consumes the built `pk` binary via hooks
(`shared/scripts/pk-focus-on-prompt.sh`, `pk-health.sh`, `pk-lint.sh`, and
`pk ingest` at session Stop) and, from `phase-okf-llm-wiki-adoption` onward,
via a first-class `llm-wiki` skill.

**Ownership split (decided 2026-07-01, phase-okf-llm-wiki-adoption):**

| Concern | Canonical repo |
|---|---|
| Wiki entry frontmatter format, parser, writer | `prometheus-knowledge-rs` (`pk-store`, `pk-core`) |
| `index.md` / `log.md` maintenance, body cross-links, Citations | `prometheus-knowledge-rs` (`pk-librarian`) |
| OKF conformance rules in `pk lint` | `prometheus-knowledge-rs` |
| `llm-wiki` skill, wiki schema doc, hook wiring | `prometheus-skill-system` (this repo) |

**Format decision:** the pk wiki adopts the **Open Knowledge Format (OKF) v0.1**
— vendored at [`shared/references/okf-v0.1.md`](shared/references/okf-v0.1.md)
— for wiki entry frontmatter/body conventions. OKF requires only a non-empty
`type` frontmatter key and mandates permissive consumption (unknown types,
missing optional fields, and broken links are never grounds to reject a
document). The Karpathy LLM Wiki operational pattern itself (ingest / query /
lint, the two reserved files `index.md` + `log.md`, the three-layer
raw-sources/wiki/schema architecture) is vendored at
[`shared/references/llm-wiki-pattern.md`](shared/references/llm-wiki-pattern.md).

At the time of adoption, both the project-local and shared `pk` knowledge
bases were empty (0 entries), so the format change carried no migration cost.
That window closes once real ingestion starts — a future format change would
need an explicit migration path.

## Reflector Sycophancy Gate

The `reflector` SubagentStop hook automatically checks reflection artifacts for sycophantic patterns before they are logged or used to advance the PMPO cycle.

### How it works

1. When the `reflector` subagent stops, `sycophancy-check-reflection.sh` reads the agent output.
2. It invokes the `sycophancy-correction` MCP server at configurable strictness.
3. If the reflection scores ≥ 0.4 or contains `high`/`critical` severity patterns, it is **rejected** with actionable feedback explaining what is missing.
4. A **2-rejection soft cap** prevents infinite loops — after two consecutive rejections the third attempt is accepted with a logged warning.
5. The consecutive rejection count resets to 0 on a passing reflection.

### Configuring strictness

Set `PROMETHEUS_REFLECT_STRICTNESS` in the environment before invoking Claude Code:

| Value | Behavior |
|-------|----------|
| `loose` | Maps to `permissive` — only flags severe patterns |
| `standard` | Standard detection sensitivity (default when omitted) |
| `strict` | **Default for the gate** — raises threshold for all patterns |
| `adversarial` | Also maps to `strict`; reserved for future adversarial mode |

```bash
# Run a session with permissive reflection checking
PROMETHEUS_REFLECT_STRICTNESS=permissive claude

# Disable rejection (treat as permissive; gate still logs)
PROMETHEUS_REFLECT_STRICTNESS=loose claude
```

### What a good reflection looks like

The gate enforces the PMPO Reflect structure: **Delta → Root Cause → Corrective Actions**. A passing reflection must:
- Name specific gaps between what was planned and what was delivered (not a success summary)
- State root causes for each delta
- Provide concrete corrective actions for the next iteration

### Binary prerequisite

The gate requires the `sycophancy-correction` binary (built via `cargo build --release` in `skills/imported/sycophancy-correction/`). When the binary is absent, the gate logs a warning and exits 0 (graceful degradation — the hook never blocks the Stop chain).

### State file

`~/.prometheus/reflect-rejections.txt` tracks consecutive rejections per session. It is a runtime artifact and is not committed.

## Progress Signaling (MANDATORY)

All agents — including Claude Code — must emit progress signals at the start and completion of every phase and every task. These signals keep long conversations scannable and provide an immediate sense of position without re-reading the full thread.

### FIRST ACTION every kbd-* turn

Before emitting any signal or making any tool call, read:

1. `.kbd-orchestrator/position-reminder.txt` — has current phase, step N of T, stage, next command
2. If absent: `.kbd-orchestrator/current-waypoint.json`
3. `.kbd-orchestrator/phases/<phase>/progress.json` — for accurate N and T

### Format

For KBD lifecycle commands (`/kbd-assess`, `/kbd-analyze`, `/kbd-plan`, `/kbd-execute`, `/kbd-reflect`, `/kbd-evolve`):

```
Starting kbd-execute — self-learning-loop-integration (step 3 of 10)
Starting change 3 of 10: change-slli-003
Completed change 3 of 10: change-slli-003
Completed kbd-execute — self-learning-loop-integration (step 3 of 10)
```

For general phases and tasks:

```
Starting phase 2 out of 6:  SP-014 fallback SubagentStop matcher verification
Starting task 1 out of 4:   Locate all fallback hook scripts
Completed task 1 out of 4:  Locate all fallback hook scripts
...
Completed phase 2 out of 6: SP-014 fallback SubagentStop matcher verification
```

### Rules

- **Emit before any work begins** on a phase or task — not after the first file is touched.
- **Emit immediately after completion** — before moving to the next phase or task.
- **Total counts must be accurate.** Read `progress.json` or the plan to get the real totals. Never estimate.
- **Use the canonical name** from `plan.md` or `progress.json`, not a paraphrase.
- **The `(step N of T)` suffix is required** on all kbd-* skill signals — N and T from `progress.json`.
- Signals go to **stdout** (normal response text). They do not require a tool call.
- This rule applies in **every session** regardless of context length.

## Troubleshooting

### Validation Errors

**Error**: "SKILL.md must have YAML frontmatter"

- **Fix**: Add frontmatter with `---` delimiters and required fields

**Error**: "Frontmatter name doesn't match directory"

- **Fix**: Ensure `name:` field matches directory name exactly

**Error**: "Found backslashes in SKILL.md"

- **Fix**: Replace all `\` with `/` in paths

### Build Issues

**Symlinks not created**:

- **Check**: Permissions on `.claude-plugin/` directory
- **Fix**: Run `npm run build` to recreate symlinks

**Skills not loading**:

- **Check**: Restart Claude Code or run `/reload-plugins`
- **Verify**: Skill is in correct location with valid `SKILL.md`

### Installation Issues

**Permission denied on scripts**:

- **Fix**: Run `chmod +x scripts/*.sh` in skill directory

**Module not found in script**:

- **Fix**: Use package runners (`npx`, `uvx`) or inline dependencies

## References

- [AgentSkills.io Specification](https://agentskills.io/specification)
- [Claude Code Plugin Documentation](https://code.claude.com/docs/en/plugins)
- [Contributing Guidelines](docs/CONTRIBUTING.md)
- [Skill Template](docs/SKILL_TEMPLATE.md)

## Session Scratchpad Pattern (XC-003)

When working on a multi-step task within a session, create a `SCRATCHPAD.md` file at the project root to capture in-flight notes, hypotheses, and intermediate decisions. This file is:

- **Not committed** — it stays local to the working session
- **Not a plan** — plans live in `.kbd-orchestrator/phases/*/plan.md`; this is for informal notes
- **Disposable** — delete or clear it at the end of the session

### What goes in SCRATCHPAD.md

- Hypotheses being tested
- Quick decision log ("tried X, didn't work because Y")
- Intermediate shell output snippets that inform next steps
- Short-term reminders ("check if SP-004 is already done before re-implementing")

### What does NOT go in SCRATCHPAD.md

- Final outcomes — those go in `reflection.md` or memory
- Plans — those go in `plan.md`
- Architecture decisions — those go in CLAUDE.md or the KB

### Usage

```bash
# During session — write freely
echo "Investigating SP-014 duplicate: already done in change-006" >> SCRATCHPAD.md

# End of session — clear it
rm SCRATCHPAD.md
```

`SCRATCHPAD.md` is listed in `.gitignore` so it is never accidentally committed.
