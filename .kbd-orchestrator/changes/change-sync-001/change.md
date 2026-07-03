# change-sync-001: Delete AutomergeEngine; implement LoroAdapter

**Phase:** phase-learn-sovereign-sync
**Tier:** 0 (foundation — blocking all other changes)
**Status:** pending
**Library:** cand-001 (loro 1.13.x)
**Gap:** G-11

## Summary

Migrate `substrate/storage-provider` from `automerge = "0.5"` (YAGNI — never
shipped to production) to `loro = "1.13"`. Implement `LoroAdapter` using
patterns from `frf-crdt`. Delete `AutomergeEngine` entirely.

## Files to change

- `substrate/storage-provider/Cargo.toml` — replace automerge with loro
- `substrate/storage-provider/src/lib.rs` — remove AutomergeEngine; re-export LoroAdapter
- `substrate/storage-provider/src/loro_adapter.rs` — new file; LoroAdapter impl
- `substrate/learner-model/Cargo.toml` — update storage-provider dependency path
- `substrate/learner-model/src/lib.rs` — update CRDT calls from automerge API to Loro API

## Key implementation notes

Reference: `frf-crdt` patterns at `/Users/gqadonis/Projects/prometheus/flint-realtime-fabric/crates/frf-crdt`

```rust
// loro_adapter.rs — core pattern
use loro::LoroDoc;

pub struct LoroAdapter {
    doc: LoroDoc,
}

impl CrdtEngine for LoroAdapter {
    fn apply_delta(&mut self, delta: &[u8]) -> Result<(), CrdtError> {
        self.doc.import(delta).map_err(CrdtError::from)
    }
    fn export_updates_since(&self, version: &[u8]) -> Result<Vec<u8>, CrdtError> {
        let vv = serde_json::from_slice(version).map_err(CrdtError::from)?;
        Ok(self.doc.export_updates_from(&vv))
    }
    fn merge_into(&mut self, other: &[u8]) -> Result<(), CrdtError> {
        self.doc.import(other).map_err(CrdtError::from)
    }
}
```

## Tasks

- [ ] Read `substrate/storage-provider/Cargo.toml` and `src/lib.rs`
- [ ] Check frf-crdt for exact Loro API calls in use
- [ ] Remove automerge dep, add loro = "1.13"
- [ ] Implement LoroAdapter in new file
- [ ] Update learner-model CRDT references
- [ ] Run `cargo check` in substrate/ workspace
- [ ] Verify all tests pass
