# change-sync-005: iroh P2P endpoint + iroh-gossip peer discovery

**Phase:** phase-learn-sovereign-sync
**Tier:** 1 (parallelize with 004, 006, 007 after 003)
**Status:** pending
**Library:** cand-002 (iroh 1.0.0), cand-003 (iroh-gossip)
**Gap:** G-02

## Summary

Implement the P2P node lifecycle in `substrate/sovereign-sync/src/p2p.rs`.
Three-layer discovery: pkarr bootstrap → iroh-gossip epidemic broadcast → mDNS LAN.
Use `statig` FSM for connection state management.

## Files to change

- `substrate/sovereign-sync/src/p2p.rs` — new file
- `substrate/sovereign-sync/Cargo.toml` — add iroh, iroh-gossip, blake3, statig

## gossip TopicId derivation

```rust
let topic_bytes = blake3::hash(
    &[operator_id.as_bytes(), b"sovereign-sync-v1"].concat()
).into();
let topic_id = TopicId::from_bytes(topic_bytes);
```

## statig FSM states

`Disconnected → Bootstrapping → Connected → Syncing → Idle`

## Tasks

- [ ] Write P2PNode struct with endpoint creation
- [ ] Implement three-layer discovery
- [ ] Implement statig FSM
- [ ] Connect gossip subscriber
- [ ] Test: two local nodes discover each other
