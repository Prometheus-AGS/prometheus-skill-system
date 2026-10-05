---
id: storage-provider
title: storage-provider
---

# storage-provider

Defines the `StorageProvider` and `CrdtEngine` traits that every learn-domain
persistence backend implements, plus `LocalDirAdapter` (the default filesystem
backend) and `LoroAdapter`. The former iroh-docs transport adapter moved to Companion; it is absent from this crate's current module exports.

It also owns the structural privacy layer: `SyncManifest`, `SyncDomain`, and
`PrivacyClass` enforce KB-content privacy at the type level — a domain that
must not leave the machine cannot be handed to a sync adapter.

*Canonical source: [`substrate/storage-provider`](https://github.com/Prometheus-AGS/prometheus-skill-system/tree/main/substrate/storage-provider) — module map: `local_dir`, `loro_adapter`, `sync_manifest`, `traits`.*
