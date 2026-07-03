---
id: privacy-model
title: Privacy Model
sidebar_label: Privacy Model
---

# Privacy Model

Sovereign Sync's privacy guarantee is architectural, not conventional. It is enforced in the Rust
type system and cannot be bypassed by configuration.

## Core invariant

> **KB content is NEVER forwarded to external APIs or transmitted over the P2P network.**

`surreal-memory` has `PrivacyClass::LocalOnly`. The `crdt` module rejects any attempt to export
or apply a delta for a `LocalOnly` domain with a `SyncError::PrivacyViolation` error.

## Privacy classes

| Class | Transmission | Examples |
|-------|-------------|---------|
| `LocalOnly` | Never leaves the device | `surreal-memory`, palace RAG indexes |
| `SyncEncryptedOnly` | Encrypted via QUIC TLS 1.3 + Ed25519 | `learner-model`, `kbd-orchestrator`, `open-spec` |
| `SyncPlaintext` | Unencrypted (public metadata) | Future: public skill registries |

## Enforcement code

```rust
pub fn apply_incoming_delta(
    manifest: &SyncManifest,
    domain: &SyncDomain,
    delta: &[u8],
    docs: &mut HashMap<SyncDomain, LoroDoc>,
) -> Result<(), SyncError> {
    if !manifest.is_syncable(domain) {
        return Err(SyncError::PrivacyViolation(domain.to_string()));
    }
    // ...
}
```

The same check exists in `export_outgoing_delta`. Neither function is behind a feature flag or
configuration toggle.

## What is transmitted

When a `sync-push learner-model` is invoked:

- Mastery levels per concept (numeric values)
- FSRS card schedules (due dates, stability, difficulty)
- Gap records (which concepts have identified gaps)

What is **not** transmitted:

- Raw KB content (documents, segments, embeddings)
- Conversation history
- Personal notes or annotations stored in `surreal-memory`
- Passwords, API keys, or credentials

## iroh transport security

The P2P transport uses iroh 1.0 with `presets::N0`:

- **QUIC + TLS 1.3** for all peer connections
- **Ed25519 node keys** for peer authentication
- **DNS discovery + relay** via the n0 relay network (no IP address exposure required)
- Relay traffic is encrypted end-to-end — the relay node cannot read payload content
