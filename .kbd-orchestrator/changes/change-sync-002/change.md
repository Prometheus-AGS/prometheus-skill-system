# change-sync-002: SyncManifest schema + SyncDomain + PrivacyClass

**Phase:** phase-learn-sovereign-sync
**Tier:** 0 (foundation — blocks sync-service and IrohDocsAdapter)
**Status:** pending
**Gap:** G-01, G-12

## Summary

Define the core type layer for domain namespacing and privacy enforcement.
`PrivacyClass::LocalOnly` is the structural enforcement of the KB content
invariant — domains marked LocalOnly are never placed in iroh sync payloads.

## Files to change

- `substrate/storage-provider/src/sync_manifest.rs` — new file (entire manifest)

## Key types

```rust
pub enum SyncDomain {
    KbdOrchestrator,
    OpenSpec,
    SurrealMemory,
    LearnerModel,
    Custom(String),
}

pub enum PrivacyClass {
    LocalOnly,           // NEVER transmitted; KB content enforced here
    SyncEncryptedOnly,
    SyncPlaintext,
}

pub struct DomainConfig {
    pub domain: SyncDomain,
    pub storage_prefix: String,
    pub privacy: PrivacyClass,
    pub iroh_namespace_key: [u8; 32],  // BLAKE3(operator_id || domain)
}

pub struct SyncManifest {
    pub operator_id: [u8; 32],
    pub domains: Vec<DomainConfig>,
}
```

SurrealMemory palace prefix → LocalOnly (this is where the KB content privacy
invariant is enforced architecturally, not by convention).

## Tasks

- [ ] Write `sync_manifest.rs` with all types
- [ ] Add `pub mod sync_manifest;` to `lib.rs`
- [ ] Write unit test: LocalOnly domain panics/errors if passed to a sync function
- [ ] Run `cargo check`
