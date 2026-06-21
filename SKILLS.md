---
name: prometheus-skill-pack
version: 1.2.0
type: collection
license: MIT
description: >
  Enterprise-grade AI skills for React entity management, GitOps CI/CD, process
  orchestration, iterative evolution, BDD testing, and Rust development — with
  surreal-memory distributed state and native skill/CLI/MCP generation capabilities.
platforms:
  - claude-code
  - kimi-code
  - opencode
  - minimax
  - codex
  - cursor
  - windsurf
  - gemini-cli
  - roo-code
  - amp
repository: https://github.com/Prometheus-AGS/prometheus-skill-system
---

# Prometheus Skill Pack

A comprehensive, enterprise-grade skill collection for AI-assisted development. 35 top-level skills spanning 13 categories, with 95+ total skills including sub-skills.

## Platform Quick Start

### Claude Code (CLI / Desktop)
```bash
# Install globally — skills available as /kbd-init, /evolve, /gitops-bootstrap, etc.
bash scripts/install-skills-flat.sh

# Or via npm
npm run install:user

# Verify
npm run doctor
```

### Kimi Code CLI
```bash
# Install skills and configure MCP servers (surreal-memory, sycophancy-correction)
bash scripts/install-skills-flat.sh

# Skills load from ~/.kimi-code/skills/ automatically
# Use: kimi --skills-dir ~/.kimi-code/skills
```

### MiniMax / Mavis CLI
```bash
# Install skills (copies + _meta.json) and register MCP servers
bash scripts/install-skills-flat.sh

# Skills appear in ~/.minimax/skills/ with _meta.json metadata
# MCP: surreal-memory registered in ~/.minimax/mcp/mcp.json
```

### OpenCode
```bash
# Full install including plugin registration
npm run install:platforms -- --platform opencode

# Or via flat installer
bash scripts/install-skills-flat.sh
```

### Codex CLI
```bash
bash scripts/install-skills-flat.sh
# Skills install to ~/.codex/skills/
# MCP config already present at .codex/config.toml
```

### Cursor / Windsurf / Other Platforms
```bash
bash scripts/install-skills-flat.sh
# Skills symlinked to platform skill directories automatically
```

## Prerequisites

### Required
- Node.js >= 18
- Git

### For Rust/Cargo Skills
```bash
# Check Rust toolchain
rustup show

# Install if missing
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup target add wasm32-unknown-unknown
```

### For surreal-memory (Distributed State)
```bash
# Check if running (all platforms)
curl -s http://localhost:23001/health

# Start via Docker (recommended)
cd tools/surreal-memory-server && docker compose up -d

# Or check if binary is available
which surreal-memory-server
```

### Full prerequisite check
```bash
npm run doctor
# or
bash scripts/check-prerequisites.sh --install
```

## Detecting surreal-memory on Any Platform

surreal-memory is a REST + SSE MCP server. On any platform:

```bash
# Health check
curl -s http://localhost:23001/health | jq .

# MCP SSE endpoint (for MCP clients)
# SSE: http://localhost:23001/mcp/sse
# REST write: POST http://localhost:23001/api/v1/memory

# From scripts — detect and degrade gracefully
if curl -sf http://localhost:23001/health >/dev/null 2>&1; then
    echo "surreal-memory: available"
else
    echo "surreal-memory: not reachable — memory features disabled"
fi
```

## Toolchain Initialization

All platforms can use the shared toolchain detector:
```bash
bash shared/scripts/detect-toolchain.sh
```

This checks: Node, Rust/Cargo, Go, Docker, surreal-memory, and all Prometheus binaries.

## Skills Index

### Architecture (1 skill)

| Skill | Description |
|-------|-------------|
| `clean-architecture` | Clean Architecture patterns for layered, testable, domain-driven codebases |

### DevOps (4 skills)

| Skill | Description |
|-------|-------------|
| `argocd-multicloud` | ArgoCD multi-cloud GitOps deployment patterns for GKE, AKS, and EKS clusters |
| `gitops-bootstrap` | Bootstrap a GitOps-ready Kubernetes repository with Kustomize overlays and ArgoCD |
| `gitops-transform` | Transform existing Kubernetes manifests into a GitOps-ready structure |
| `kustomize-overlay` | Kustomize overlay patterns for environment-specific Kubernetes configurations |

### Document Extraction (1 skill)

| Skill | Description |
|-------|-------------|
| `kreuzberg` | Document extraction with Kreuzberg: PDF, DOCX, HTML to structured text |

### Flutter (1 skill)

| Skill | Description |
|-------|-------------|
| `flutter-rust-ffi` | Flutter-Rust FFI integration patterns for high-performance native modules |

### Go (1 skill)

| Skill | Description |
|-------|-------------|
| `base-patterns` | Idiomatic Go patterns: errors, interfaces, goroutines, and project structure |

### HTMX (1 skill)

| Skill | Description |
|-------|-------------|
| `htmx-alpine-lit` | Hypermedia-driven UI with HTMX, Alpine.js, and Lit web components |

### Process (9 skills)

| Skill | Description |
|-------|-------------|
| `ideation-mindmap` | Generate structured ideation mindmaps with surreal-memory integration |
| `iterative-evolver` | PMPO iterative evolution engine: assess → analyze → plan → execute → reflect |
| `kbd-process-orchestrator` | KBD (Knowledge-Based Development) lifecycle orchestrator with 18 child skills |
| `liter-llm-bridge` | LiterLLM multi-model routing bridge for cost-aware LLM pipelines |
| `native-agent` | Generate, compile, and install native agent binaries, CLIs, and MCP servers |
| `pmpo-elicit` | PMPO artifact elicitation: draw out requirements, constraints, and goals |
| `pmpo-outer-loop` | PMPO outer loop orchestrator for cross-session evolution management |
| `pmpo-skill-creator` | Create, clone, extend, and validate skills (4 child skills) |
| `zeespec-interrogator` | ZeeSpec specification interrogation and requirement extraction |

### Python (1 skill)

| Skill | Description |
|-------|-------------|
| `pyo3-bridge` | PyO3 Rust-Python FFI bindings: build Python extensions in Rust |

### React (2 skills)

| Skill | Description |
|-------|-------------|
| `prometheus-entity-skills` | Complete entity management system: graph CRUD, GraphQL, Prisma, realtime (8 sub-skills) |
| `react-vite-stack` | Modern React + Vite stack with TypeScript, Tailwind, and testing setup |

### Rust (10 skills)

| Skill | Description |
|-------|-------------|
| `actor-model` | Rust actor model patterns with tokio and message-passing concurrency |
| `async-patterns` | Idiomatic async Rust: futures, streams, tokio, and error handling |
| `axum-patterns` | Production Axum web API patterns: routing, middleware, state, extractors |
| `error-handling` | Rust error handling with thiserror, anyhow, and the ? operator |
| `karpathy-tokenizer` | BPE tokenizer implementation in Rust following Karpathy's minBPE approach |
| `librefang-wasm-skill` | LibreFang WASM module patterns: Rust-to-WASM compilation and JS interop |
| `mcp-server` | Build MCP (Model Context Protocol) servers in Rust |
| `performance` | Rust performance optimization: profiling, SIMD, allocation, and benchmarking |
| `prometheus-rust-auditor` | End-to-end Rust workspace audit: Clippy, fmt, deps, safety, CI generation |
| `workspace-structure` | Rust workspace organization: crates, features, dependencies, and build config |

### Tauri (1 skill)

| Skill | Description |
|-------|-------------|
| `tauri-react-vite` | Tauri desktop app with React + Vite frontend and Rust backend |

### Testing (2 skills)

| Skill | Description |
|-------|-------------|
| `bdd-testing` | BDD testing with Cucumber: feature files, step definitions, and reporting |
| `bdd-video-proof` | BDD video proof generation: record test execution as video artifacts |

### TypeScript (1 skill)

| Skill | Description |
|-------|-------------|
| `base-patterns` | TypeScript/JavaScript base patterns: types, async, error handling, React hooks |

## Imported Skills (Git Submodules)

| Skill | Description |
|-------|-------------|
| `artifact-refiner` | PMPO artifact refinement engine: rebrand, refine-content, refine-ui, scaffold |
| `sycophancy-correction` | Rust MCP server for detecting and correcting sycophantic AI output patterns |

## Meta-Operation: Generating Native Skills

The `native-agent` skill supports generating, compiling, and installing additional native skills, CLIs, and MCP servers:

```
/native-agent          # Generate a new native agent binary
/create-native-agent   # Scaffold a new native Rust CLI agent
/start-business-build  # Full business domain skill generation pipeline
```

Prerequisites for meta-operation:
- Rust toolchain (`rustup show`)
- Cargo (`cargo --version`)
- `wasm32-unknown-unknown` target (`rustup target list --installed | grep wasm`)

## Memory Architecture

All process skills integrate with surreal-memory for cross-session state:

- **Knowledge graph**: entities, relations, semantic search
- **Scoped memory**: session insights, lessons learned
- **TaskStreams**: multi-step task progress tracking
- **Mindmaps**: ideation and planning structures

Memory degrades gracefully when surreal-memory is unavailable — all skills function without it.

## Validation

```bash
# Validate all skills
npm run validate

# Strict validation (required for new skills)
npm run validate:strict

# Full system health check
npm run doctor
```
