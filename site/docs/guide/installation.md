---
id: installation
title: Installation
sidebar_label: Installation
---

# Installation

## One-command install

```bash
bash scripts/install-skills-flat.sh
```

This installs skills to all detected platforms (Claude Code, Kimi Code, MiniMax, Codex,
OpenCode, Cursor) and builds + installs Rust substrate crates.

## What gets installed

### Skills

- Copied to `~/.claude/skills/` (Claude Code)
- Copied to `~/.kimi-code/skills/` (Kimi Code)
- Copied to `~/.opencode/skills/` (OpenCode)
- Copied to `~/.codex/skills/` (Codex)
- Copied to `~/.cursor/skills/` (Cursor)

### MCP servers

- `surreal-memory` — knowledge graph + palace RAG
- `sycophancy-correction` — anti-sycophancy gate
- `sovereign-sync` — P2P CRDT sync daemon

### Rust binaries (built from source)

- `~/.local/bin/learner-model` — FSRS-6 scheduler
- `~/.local/bin/surface-bridge` — Tier 2 UI server (port 7890)
- `~/.local/bin/sovereign-sync` — P2P sync daemon (port 7892)

## Prerequisites

- Node.js ≥ 18
- Rust stable (for substrate crates)
- macOS or Linux (launchd services are macOS-only; Linux systemd support planned)

## Verify installation

```bash
# Check all platforms
bash shared/scripts/detect-toolchain.sh

# JSON output
bash shared/scripts/detect-toolchain.sh --json
```

## Uninstall

```bash
bash scripts/install-skills-flat.sh --uninstall
```
