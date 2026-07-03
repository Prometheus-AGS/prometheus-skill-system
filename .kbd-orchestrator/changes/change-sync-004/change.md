# change-sync-004: IrohDocsAdapter implementation

**Phase:** phase-learn-sovereign-sync
**Tier:** 1 (parallelize with 005, 006, 007 after 003)
**Status:** pending
**Library:** cand-002 (iroh 1.0.0)
**Gap:** G-10

## Summary

Replace all `Err(Unavailable)` stubs in `substrate/storage-provider/src/iroh_docs.rs`
with real iroh 1.0.0 API calls. One iroh-docs namespace per SyncDomain, keyed
by `BLAKE3(operator_id || domain.to_bytes())`.

## Files to change

- `substrate/storage-provider/src/iroh_docs.rs` — replace stubs
- `substrate/storage-provider/Cargo.toml` — add iroh = "1.0"

## iroh 1.0.0 API rename notes

| Old | New |
|-----|-----|
| `NodeAddr` | `EndpointAddr` |
| `NodeId` | `EndpointId` |
| `iroh-net` (separate crate) | Merged into `iroh` |
| `DnsDiscovery` | `iroh::address_lookup::DnsAddressLookup` |
| `LocalDiscovery` | `iroh::address_lookup::LocalAddressLookup` |

## Tasks

- [ ] Read current `iroh_docs.rs` to inventory all stubs
- [ ] Check iroh 1.0.0 docs for exact API (Context7 or docfork)
- [ ] Implement `get()`, `set()`, `delete()`, `iter()` methods
- [ ] Use BLAKE3 namespace derivation per domain
- [ ] Unit test: write key, read back, verify
- [ ] `cargo check`
