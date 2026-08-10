---
license: MIT
name: create-native-agent
version: '1.0.0'
description: >
  Generate a complete native Rust agent application with Supabase-style CLI,
  A2A/AG-UI/A2UI protocol support, React 19 assistant-ui chat frontend, liter-llm
  model routing, MCP client tools, and the Prometheus skills engine. One command
  scaffolds a runnable multi-protocol agent binary.
metadata:
  tags: [process, orchestration, automation]
---

# /create-native-agent

Scaffolds a complete native Rust agent workspace.

## Inputs (prompted interactively)

```yaml
agent_name:          string   # kebab-case (e.g. "research-agent")
agent_description:   string   # one-line description for the A2A card
default_provider:    string   # anthropic | openai | local (default: anthropic)
default_model:       string   # default model ID
output_dir:          string   # where to create the project (default: ./<agent-name>)
port:                integer  # default server port (default: 8080)
enable_surreal:      bool     # add surreal-memory MCP server to config
enable_forge:        bool     # add forge-rs MCP server to config
enable_pk:           bool     # add prometheus-knowledge MCP server to config (default: true)
target:              enum     # docker | librefang-wasm | both (default: docker)
```

### `target` flag

- **`docker`** (default): emits the standard agent-server + agent-cli + Docker
  pipeline. Identical to pre-1.3 behavior.
- **`librefang-wasm`**: additionally emits a `crates/agent-skill/` cdylib
  targeting wasm32-unknown-unknown plus a `skill.toml` LibreFang manifest
  at the project root. Run `forge package-librefang` to produce a
  `<agent-name>.lf-skill.zip` and `/upload-to-bossfang <url>` to install it.
- **`both`**: emits everything from both targets in a single workspace. Useful
  during migration: developers run the Docker stack locally and ship the WASM
  build to bossfang.

The `agent-tokenizer` crate (rustbpe-backed BPE tokenizer for token-budget
enforcement) is always emitted, regardless of `target`. Karpathy's `rustbpe`
is explicitly chosen over HuggingFace tokenizers (too large) and Python
minbpe (too slow).

### Default companion services

When `enable_pk` is `true` (default), the generated docker-compose includes
a `prometheus-knowledge` service running on `:8942`. This activates the
Karpathy learning loop (`pk focus` / `pk ingest`) by default rather than
requiring explicit opt-in. To disable, answer `n` to the `enable_pk` prompt.

## Setup Protocol

1. Prompt for inputs (use defaults where sensible)
2. Validate: `agent_name` is kebab-case, `port` is in 1024–65535 range
3. Check output directory does not already exist (or prompt to overwrite)
4. Generate all project files from templates
5. Run `cargo check --workspace` to verify Rust compilation
6. Run `npm install` in `frontend/`
7. Print success message with next steps

## Generated Files (complete list)

### Rust Workspace
- `Cargo.toml` — workspace manifest
- `crates/agent-core/Cargo.toml` + `src/lib.rs`
- `crates/agent-skills/Cargo.toml` + `src/lib.rs`
- `crates/agent-mcp/Cargo.toml` + `src/lib.rs`
- `crates/agent-server/Cargo.toml` + `src/lib.rs`
- `crates/agent-cli/Cargo.toml` + `src/main.rs`
- `crates/agent-tokenizer/Cargo.toml` + `src/lib.rs` — rustbpe wrapper for token-budget enforcement (always emitted)
- `crates/agent-skill/Cargo.toml` + `src/lib.rs` + `src/host.rs` — LibreFang WASM skill (only when `target` ∈ {`librefang-wasm`, `both`})
- `skill.toml` — LibreFang manifest at project root (only when `target` ∈ {`librefang-wasm`, `both`})

### Configuration
- `agent.toml` — provider/model/MCP/skills config
- `system_prompt.md` — agent system prompt
- `.env.example` — required environment variables
- `.gitignore`

### Frontend
- `frontend/package.json`
- `frontend/vite.config.ts`
- `frontend/index.html`
- `frontend/src/main.tsx`
- `frontend/src/App.tsx`
- `frontend/src/components/Chat.tsx`
- `frontend/src/components/ProviderConfig.tsx`
- `frontend/src/components/SkillsPanel.tsx`
- `frontend/src/lib/api.ts`
- `frontend/src/lib/runtime.ts`

### Documentation
- `README.md`
- `CLAUDE.md`

## Post-Generation Instructions

Print on success:
```
✅ Native agent '{{ agent_name }}' created in ./{{ agent_name }}/

Next steps:
  cd {{ agent_name }}
  cp .env.example .env
  # Add your API keys to .env

  # First run (compiles Rust workspace + builds React frontend):
  cargo build --release -p agent-cli
  npm --prefix frontend run build
  ./target/release/{{ agent_name | replace(from="-", to="_") }} start

  # Or for development (hot-reload frontend):
  ./target/release/{{ agent_name | replace(from="-", to="_") }} start --dev
  # Frontend dev server: http://localhost:5173 (proxies to agent on :{{ port }})

  # Chat interface: http://localhost:{{ port }}

Agent-to-agent: other agents can reach this one at:
  A2A card: http://localhost:{{ port }}/.well-known/agent.json
  AG-UI:    http://localhost:{{ port }}/agui/run
```
