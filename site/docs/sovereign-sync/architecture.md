---
id: architecture
title: Architecture
sidebar_label: Architecture
---

# Sovereign Sync Architecture

## Technology stack

| Component | Crate | Version | Purpose |
|-----------|-------|---------|---------|
| P2P transport | `iroh` | 1.0.0 | QUIC-based peer-to-peer networking |
| Gossip protocol | `iroh-gossip` | 0.101 | Topic-based broadcast to peer set |
| CRDT engine | `loro` | 1.13.x | Conflict-free replicated data types |
| Persistence | `redb` | 2.x | Embedded key-value store |
| MCP server | `rmcp` | 1.8.0 | Official Model Context Protocol SDK |
| HTTP server | `axum` | 0.8 | REST API + AG-UI SSE |
| Rust SDK | `sovereign-client` | 0.1.0 | reqwest + eventsource-stream |

## Topic derivation

Every operator group shares a unique gossip topic derived from their operator ID:

```
Topic = BLAKE3(operator_id || "sovereign-sync-v1")
```

All devices with the same `--operator-id` (a 32-byte hex key) automatically join the same
P2P gossip group. No manual peer exchange is required for same-group devices.

## SyncManifest

The `SyncManifest` declares which domains exist and their privacy classification:

```rust
pub enum PrivacyClass {
    LocalOnly,           // Never transmitted — KB content invariant
    SyncEncryptedOnly,   // QUIC TLS 1.3 + Ed25519 node keys
    SyncPlaintext,       // Public metadata (future)
}

pub enum SyncDomain {
    KbdOrchestrator,  // SyncEncryptedOnly
    OpenSpec,         // SyncEncryptedOnly
    SurrealMemory,    // LocalOnly  ← structural privacy enforcement
    LearnerModel,     // SyncEncryptedOnly
    Custom(String),   // User-defined
}
```

## CRDT merge

Domains use Loro 1.13 for conflict-free merges:

```rust
// Export a snapshot (no prior version)
doc.export(ExportMode::Snapshot)?

// Export only changes since a known version
doc.export(ExportMode::Updates { from: Cow::Owned(vv) })?

// Import an incoming delta
doc.import(delta)?
```

The `LearnerModel` domain uses per-card FSRS merge: timestamps are monotone and mastery
updates use the PFA formula, ensuring convergence regardless of device ordering.

## MCP server tools

When running as `--mode mcp`, sovereign-sync exposes 4 tools:

| Tool | Description |
|------|-------------|
| `search-skills` | Keyword search over the local skill index |
| `sync-status` | Current node state + peer count |
| `sync-push` | Push a domain to all connected peers |
| `sync-peers` | List connected peer node IDs |

Tool names are optionally prefixed with `sovereign:` via `--prefix-tools` to avoid collision
when multiple MCP servers are active in UAR or BossFang contexts.
