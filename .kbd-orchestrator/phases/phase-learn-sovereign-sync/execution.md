# Execution Plan — phase-learn-sovereign-sync
**Backend:** native-tool (Claude Code)
**Date:** 2026-06-28
**Total changes:** 20 (18 planned + 2 Docusaurus additions per operator request)

## Backend Selection

`native-tool` — Claude Code executes all changes directly. No OpenSpec. No
external dispatch. KBD progress.json is the source of truth.

## Operator Additions at Execute Time

The operator added a Docusaurus documentation site requirement:
- `change-sync-019`: Docusaurus site scaffold + sovereign-sync docs
- `change-sync-020`: Cross-link all existing docs into the site

These are Tier 4 (parallel with 016–018).

## Dispatch Contract

Changes execute in tier order. Within a tier, changes that have no
inter-dependency can parallelize:

| Tier | Changes | Must complete before |
|------|---------|---------------------|
| 0 | 001, 002 | All other tiers |
| 1 | 003, then 004+005+006+007 (parallel) | Tier 2 |
| 2 | 008+009+010+011 (parallel) | Tier 3 |
| 3 | 012, then 013+014+015 (parallel) | Tier 4 |
| 4 | 016+017+018+019+020 (parallel) | — (final tier) |

## QA Gate

Skipped for documentation-only changes (change-sync-019, change-sync-020).
Applied for all code changes (001–018).

## Change Registry

change-sync-001: Delete AutomergeEngine; implement LoroAdapter
change-sync-002: SyncManifest schema + SyncDomain + PrivacyClass
change-sync-003: sovereign-sync crate scaffold
change-sync-004: IrohDocsAdapter implementation
change-sync-005: iroh P2P endpoint + iroh-gossip peer discovery
change-sync-006: Loro merge engine in sovereign-sync
change-sync-007: redb persistence for sync state
change-sync-008: rmcp MCP server (stdio mode)
change-sync-009: AG-UI + A2UI streaming endpoint
change-sync-010: REST API (Axum routes, daemon/server modes)
change-sync-011: MCP client pool (rmcp, mcp-servers.json)
change-sync-012: sovereign-client Rust SDK
change-sync-013: /sync-status skill
change-sync-014: /sync-peers skill
change-sync-015: /sync-push skill
change-sync-016: install-skills-flat.sh extension
change-sync-017: Integration tests
change-sync-018: Workspace Cargo.toml + version bump + CLAUDE.md
change-sync-019: Docusaurus site scaffold + sovereign-sync docs [ADDED]
change-sync-020: Cross-link all existing docs into the site [ADDED]
