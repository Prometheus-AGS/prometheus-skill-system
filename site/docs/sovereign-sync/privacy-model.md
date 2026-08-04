---
id: privacy-model
title: Privacy Model
sidebar_label: Privacy Model
---

# Privacy Model

Sovereign Sync uses an explicit, default-deny domain manifest. A file’s
existence does not make it syncable, and there is no recursive “sync my
`.prometheus` directory” mode.

## Core invariant

> A domain absent from the manifest, or registered as `Local`, must not be
> exported to or imported from the P2P transport.

The CRDT export and import functions both call `manifest.is_syncable()`.
Unit/integration tests register `surreal-memory` as `Local` and verify that
both directions return `SyncError::PrivacyViolation`.

## Privacy classes

| Class | Transmission | Examples |
|---|---|---|
| `Local` | Structurally ineligible for CRDT export/import | `surreal-memory`, private RAG indexes, credentials |
| `Trusted` | Eligible only for explicitly trusted peers | learner model, approved project knowledge, KBD presence |
| `Public` | Eligible for any paired peer | public skill-index metadata |

All three classes are content classifications. `Public` is still encrypted in
transit; it does not enable plaintext transport.

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
    // Import only after the gate passes.
}
```

The same check exists in `export_outgoing_delta`. An unregistered domain also
returns false from `is_syncable()`.

:::caution Integration responsibility

`SyncManifest` is a library contract. The transport caller consults it before
placing bytes on the wire. Current domain and transport integration tests
exercise both export and import gates. Future adapters must preserve that call
order.

:::

## Secrets and local-only data

These values must never be domain payloads:

| Data | Why it stays local |
|---|---|
| `device-key.json` and platform signing keys | Copying a private key destroys per-device identity |
| Pairing ticket or group secret outside the confidential pairing channel | It grants topic membership and must not be published or logged |
| API keys, SSH keys, cloud credentials, cookies | Credentials are never workflow state |
| raw prompts, conversations, and harness transcripts | Not part of any declared sync domain |
| `surreal-memory` graph, Memory Palace, embeddings, and private RAG content | Default recommended classification is `Local` |
| service logs and crash dumps | They can contain paths, environment details, or errors |

Project Karpathy wiki entries are **not automatically local-only by type**, but
they are not automatically syncable either. An operator must define a separate
approved-knowledge domain, filter or normalize its contents, classify it, and
connect an adapter. The current daemon has no such adapter.

## Transport security and metadata

The P2P layer uses iroh with `presets::N0`:

- endpoint-to-endpoint QUIC traffic is encrypted;
- endpoint IDs are Ed25519 public identities;
- n0 discovery resolves endpoint IDs to current direct/relay addresses;
- relay payloads remain encrypted end-to-end.

Encryption does not hide all metadata. Public relay/discovery infrastructure
can observe connection addresses, timing, and traffic volume. The current
binary uses public n0 infrastructure and does not expose custom relay,
discovery, or peer-hook configuration. See
[Network configuration](./p2p-network).

## Current bytes-on-wire statement

The daemon can place signed `kbd-control:<project-id>` authority updates,
auxiliary presence, skill-index data, and learner-model data on the P2P wire
when their privacy policy permits it. Loop directories, Karpathy wiki,
OpenSpec trees, arbitrary project files, and unregistered global state are not
automatically transmitted. Endpoint reachability metadata may be published to
n0 discovery. That narrow allowlist is the current operational truth;
the broader domain classifications describe the intended replication contract.
