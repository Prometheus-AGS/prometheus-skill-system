---
license: MIT
name: native-agent
version: '1.0.0'
description: >
  Generates a complete, production-ready native Rust agent application with a
  Supabase-style management CLI. The generated agent binary embeds an Axum HTTP
  server supporting A2A, AG-UI, and A2UI protocols, a React 19 assistant-ui chat
  frontend, liter-llm provider routing, simple MCP client tool integration, the
  Prometheus skill pack selection engine, and full Docker support with Docker
  Desktop auto-detection and image loading. One command scaffolds a runnable,
  configurable, containerizable multi-protocol agent.
authors:
  - 'Prometheus AGS'
allowed-tools: file_system code_interpreter sequential_thinking
model_routing:
  policy_source: ".kbd-orchestrator/project.json → model_policy"
  phases:
    agent-specify: frontier
    agent-plan: frontier
    agent-generate: tiered
    agent-validate: small
  routing_reference: "references/model-routing.md"
triggers:
  keywords:
    - native agent
    - create agent
    - new agent
    - agent binary
    - rust agent
    - axum agent
    - a2a agent
    - ag-ui
    - assistant-ui
    - quickie agent
    - local agent
    - agent server
    - docker agent
    - containerize agent
  semantic: >
    Scaffold a new native Rust agent application with protocol support, a chat
    frontend, MCP client tools, Docker support, and a management CLI.
metadata:
  tags: [process, orchestration, automation]
---

# Native Agent Generator

Generates a self-contained, production-ready Rust agent workspace in one command.
The generated project is a permanent artifact — not a prototype.

## What Gets Generated

```
<agent-name>/
├── Cargo.toml                     ← workspace (resolver=2, release profile)
├── agent.toml                     ← agent config: providers, models, MCP servers, skills dirs
├── system_prompt.md               ← agent system prompt (hot-reloadable)
├── .env.example                   ← required env vars
├── Dockerfile                     ← multi-stage: cargo-chef + Node.js → slim Debian
├── docker-compose.yml             ← agent + optional surreal-memory/pk/liter-llm
├── .dockerignore
├── docker-detect.sh               ← Docker CLI/Desktop/Compose detection script
├── CLAUDE.md                      ← agent dev guide (for further AI-assisted work)
├── README.md                      ← agent documentation with Docker instructions
│
├── crates/
│   ├── agent-core/                ← domain types (no I/O)
│   ├── agent-skills/              ← TF-IDF skill discovery + hot-reload
│   ├── agent-mcp/                 ← lightweight JSON-RPC 2.0 MCP client
│   ├── agent-server/              ← Axum: A2A + AG-UI + A2UI + Chat API + static files
│   └── agent-cli/                 ← management binary + docker subcommands
│       ├── src/main.rs
│       └── src/docker.rs          ← docker detect/build/load/up/down/push/shell
│
└── frontend/                      ← React 19 + Vite 8 + assistant-ui
    └── src/
        ├── components/Chat.tsx    ← AG-UI SSE streaming adapter
        ├── components/ProviderConfig.tsx
        └── lib/api.ts
```

## Generated Agent Features

### 0. Prometheus Service Readiness

Before wiring generated agents to local Prometheus MCP services, detect the host OS:

```bash
uname -s
```

- On macOS (`Darwin`), prefer the repository service manager:
  `bash scripts/prometheus-services.sh doctor`, then `install` and `load` if
  `pk-cherry` (`:8942`) or `forge mcp` (`:8943`) are not running. These are user
  LaunchAgents for the logged-in user, with explicit `HOME`, `USER`, `PATH`, and
  service environment.
- Keep `surreal-memory-server` Docker-managed on `:23001`; the LaunchAgent setup
  must only report that port, not claim ownership of it.
- On Linux, keep systemd user service or cron guidance separate. On non-macOS,
  do not recommend LaunchAgents except to say they are unsupported.

### 1. Provider & Model Configuration via liter-llm

Uses liter-llm as the model gateway. Providers configured in `agent.toml`.
The chat frontend shows configured providers and lets users switch — cannot add new
providers (requires editing `agent.toml` + `my-agent providers set-default`).

### 2. Protocol Support (A2A + AG-UI + A2UI)

| Protocol | Endpoint | Purpose |
|---|---|---|
| **A2A** | `GET /.well-known/agent.json` | Agent card |
| **A2A** | `POST /a2a/tasks` | Receive tasks from other agents |
| **AG-UI** | `POST /agui/run` + `GET /agui/events/:id` | SSE stream (CopilotKit compatible) |
| **A2UI** | `POST /a2ui/session` | Prometheus combined protocol |
| **Chat** | `POST /api/chat` | OpenAI-compatible completions |
| **Frontend** | `GET /*` | React 19 chat UI served as static files |

### 3. Lightweight MCP Client

Connects to configured MCP servers and makes their tools available to the LLM.
Aggregates tools from multiple servers. Supports SSE transport.

### 4. Skills Engine

TF-IDF selection over configured skill directories. Auto-injects top-k relevant
skills into the system prompt per conversation turn. Hot-reloadable without restart.

### 5. Supabase-Style Management CLI

```bash
my-agent start [--port 8080] [--background]
my-agent stop / status / logs [--follow]
my-agent providers list / set-default <name>
my-agent models set-default <model>
my-agent mcp add <name> <url> / remove / ping / list
my-agent skills list [--verbose] / reload
my-agent config show / get <key> / set <key> <value>

# Docker subcommands (if Docker is available)
my-agent docker detect              # detect Docker CLI / Desktop / Compose
my-agent docker build [--tag] [--no-cache] [--platform] [--load]
my-agent docker load [--tag]        # build + load into Docker Desktop image store
my-agent docker up [-d] [--build]   # docker compose up
my-agent docker down [--volumes]    # docker compose down
my-agent docker ps                  # container status
my-agent docker logs [-f] [--tail]  # container logs
my-agent docker push [--registry]   # push to registry
my-agent docker shell               # exec into running container
```

### 6. Docker Support

The specify phase automatically detects the Docker environment and adapts:

| Detected | Behavior |
|---|---|
| Docker CLI + daemon running | Offers Dockerfile + compose + build now |
| Docker Desktop installed (macOS) | Offers `--load` to push image into Desktop image store |
| Docker Compose v2 available | docker-compose.yml with companion services |
| Docker not available | Skips Docker files, prints install link |

**Dockerfile** — Multi-stage build:
- Stage 1: `cargo-chef` for dependency layer caching (only rebuilds deps when Cargo.toml changes)
- Stage 2: Full Rust builder (reuses cached deps, builds the binary)
- Stage 3: Node.js builder (builds React frontend)
- Stage 4: `debian:bookworm-slim` runtime with non-root `agent` user

**docker-compose.yml** — Includes all enabled services:
- `agent` — the native agent
- `surreal-memory` — knowledge graph (if enabled)
- `prometheus-knowledge` — Karpathy wiki (if enabled)
- `liter-llm` — model routing proxy (if enabled)

All services share a `agent-net` bridge network. Named volumes for persistence.
Healthchecks on all services.

## Quick Start

```bash
/create-native-agent

# Generated:
cd my-agent
cp .env.example .env    # add API keys

# Option A: Native (Rust binary)
cargo build --release -p agent-cli
npm --prefix frontend run build
./my-agent start                    # http://localhost:8080

# Option B: Docker (automatically built and loaded if Docker Desktop detected)
my-agent docker build               # builds + loads into Docker Desktop
my-agent docker up -d               # starts agent + configured services
open http://localhost:8080

# Option C: docker-compose directly
docker compose up -d
```

## Docker Desktop Auto-Load

When Docker Desktop is detected and running, `my-agent docker build` automatically
adds `--load` to push the image directly into Docker Desktop's local image store.
The image then appears in Docker Desktop → Images immediately — no registry needed.

```bash
my-agent docker detect
# Docker CLI:     ✅ v27.x
# Docker Desktop: ✅ running
# Compose:        ✅ v2.x

my-agent docker build --tag my-agent:latest
# 🔨 Building my-agent:latest...
# → Building React frontend...
# → Building Docker image (loading into Docker Desktop)...
# ✅ Image my-agent:latest built and loaded into Docker Desktop
# Browse: Docker Desktop → Images → my-agent:latest
```

## Agent Networks

```
research-agent (:8081) ─── A2A ──→ forge-agent (:8080)
       ↓                                    ↓
  [docker-compose]                    [docker-compose]
  surreal-memory                      prometheus-knowledge
  prometheus-knowledge                forge-rs MCP
```

Each agent exposes `/.well-known/agent.json`. To wire two agents together,
add the other agent's A2A URL as an MCP server in `agent.toml`.

## Quick Start Commands

- `/create-native-agent` — full interactive scaffold (Docker by default; pass
  `target: librefang-wasm` or `target: both` to additionally produce a
  WASM-deployable skill)

## Build Targets

| Target | Emits | Use Case |
|---|---|---|
| `docker` (default) | Docker stack with native agent binary | Local dev, container orchestration |
| `librefang-wasm` | `crates/agent-skill/` + `skill.toml` for LibreFang | Deploy to bossfang via `/upload-to-bossfang` |
| `both` | Both above in one workspace | Migration / dual-deploy |

When `librefang-wasm` is selected, the WASM crate uses the same `agent-core`
domain types as `agent-server` and follows the LibreFang Guest ABI — see
`skills/rust/librefang-wasm-skill/` for the underlying skill that defines
the WASM contract.
