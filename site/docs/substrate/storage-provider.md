---
id: storage-provider
title: storage-provider
---

# storage-provider

Defines the `StorageProvider` and `CrdtEngine` traits that every learn-domain
persistence backend implements, plus `LocalDirAdapter` (the default filesystem
backend) and `IrohDocsAdapter` for P2P-backed storage.

It also owns the structural privacy layer: `SyncManifest`, `SyncDomain`, and
`PrivacyClass` enforce KB-content privacy at the type level — a domain that
must not leave the machine cannot be handed to a sync adapter.

*Canonical source: [`substrate/storage-provider`](https://github.com/Prometheus-AGS/prometheus-skill-system/tree/main/substrate/storage-provider) — module map: `local_dir`, `iroh_docs`, `loro_adapter`.*
