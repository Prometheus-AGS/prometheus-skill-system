# change-sync-006: Loro merge engine in sovereign-sync

**Phase:** phase-learn-sovereign-sync
**Tier:** 1 (parallelize with 004, 005, 007 after 003)
**Status:** pending
**Library:** cand-001 (loro 1.13.x)
**Gap:** G-02, G-06

## Summary

Port frf-crdt merge functions into the sync-service context. Privacy gate
enforced in `apply_incoming_delta`: LocalOnly domains rejected at the
merge boundary (structural enforcement of KB content invariant).

## Files to change

- `substrate/sovereign-sync/src/crdt.rs` — new file

## Core functions

```rust
pub fn apply_incoming_delta(
    domain: &SyncDomain,
    delta: &[u8],
    docs: &mut HashMap<SyncDomain, LoroDoc>,
) -> Result<(), SyncError> {
    if domain.config.privacy == PrivacyClass::LocalOnly {
        return Err(SyncError::PrivacyViolation(domain.name()));
    }
    docs.entry(domain.clone())
        .or_insert_with(LoroDoc::new)
        .import(delta)?;
    Ok(())
}

pub fn export_outgoing_delta(
    domain: &SyncDomain,
    since_version: Vec<u8>,
    docs: &HashMap<SyncDomain, LoroDoc>,
) -> Result<Vec<u8>, SyncError> { ... }
```

## Tasks

- [ ] Read frf-crdt merge patterns
- [ ] Implement apply_incoming_delta with privacy gate
- [ ] Implement export_outgoing_delta
- [ ] Implement scan_manifest (walk filesystem per domain)
- [ ] Unit test: LocalOnly domain returns PrivacyViolation error
- [ ] Unit test: two docs merge to same state (CRDT convergence)
