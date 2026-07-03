---
id: introduction
title: Introduction
sidebar_label: Introduction
---

# Prometheus Skill Pack

Prometheus Skill Pack is an enterprise-grade collection of AI agent skills for Claude Code,
Kimi Code, Codex, OpenCode, and other harnesses. It provides:

- **Process skills** — KBD lifecycle orchestration, PMPO meta-prompting, iterative evolution
- **Language skills** — React, Rust, TypeScript, Go, Python
- **Learn domain** — Feynman-Spine learning engine with CRDT sync
- **Sovereign Sync** — P2P CRDT synchronization for multi-device AI workflows
- **Substrate crates** — Rust binaries for persistence, UI rendering, and P2P sync

## Quick links

- [Full guide](/docs/guide/quick-start)
- [Learn domain](/docs/learn/overview)
- [Sovereign Sync](/docs/sovereign-sync/overview)
- [Full product guide](https://github.com/prometheusags/prometheus-skill-pack/tree/main/docs/guide)

## What's new in v1.5.0

- **Sovereign Sync** — P2P CRDT sync via iroh 1.0 + iroh-gossip + Loro 1.13
- **3 new learn skills** — `/sync-status`, `/sync-peers`, `/sync-push`
- **sovereign-client** Rust SDK — REST + AG-UI SSE client
- **8 integration tests** — REST API, CRDT roundtrip, P2P topic derivation
- **This Docusaurus site** — comprehensive documentation

## Supported harnesses

| Harness | Skills | MCP | Notes |
|---------|--------|-----|-------|
| Claude Code | ✅ | ✅ | Primary harness |
| Kimi Code | ✅ | ✅ | Full parity |
| Codex / Codex Desktop | ✅ | ✅ | |
| OpenCode / OpenCode Desktop | ✅ | ✅ | |
| Cursor | ✅ | — | Skills only |
| Windsurf | ✅ | — | Skills only |
| UAR | ✅ | ✅ | sovereign-sync passthrough mode |
