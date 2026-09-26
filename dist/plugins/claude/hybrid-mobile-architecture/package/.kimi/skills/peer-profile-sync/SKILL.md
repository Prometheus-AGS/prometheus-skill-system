---
name: peer-profile-sync
description: ALWAYS invoke when handling user profile data, sensitive personal data, agent-learned facts about the user, device pairing, or any data marked never-leave-the-user — it belongs in the Loro profile vault, synced ONLY device-to-device (including browser instances) over WebRTC with CRDT convergence, structurally excluded from every server sync path. Also invoke before adding ANY cloud persistence of personal context. Triggers on profile, personal data, sensitive data, PII, vault, privacy, private data, device sync, device-to-device, peer sync, WebRTC, DataChannel, pairing, pair device, multi-device, CRDT, Loro, version vector, never store, local only, e2e, agent memory about user, personalization data.
---
<!-- TJ-ARCH-MOB-001 compliant -->

> **Binding:** Prefer simple, surgical, strongly typed changes and preserve
> privacy-class boundaries. Full design:
> [`peer-crdt.md`](references/peer-crdt.md). Privacy classes:
> `sync-doctrine` skill.

# Peer Profile Sync — the vault lane

## What goes in the vault

Profile fields, private preferences, health/behavioral signals, agent-learned
facts about the user. Test: *single owner (the user), multi-device, must never
be persisted server-side.* If it needs server persistence, it is NOT vault
data — move it to lane 1 with explicit consent, don't weaken the vault.

## Non-negotiables (fail closed)

- Vault data is privacy class **`local`**: the sync write queue REFUSES it at
  enqueue (structural, not filter-based); vault tables appear in no
  `SyncScope`; unknown class ⇒ `local`.
- **Loro is the only CRDT** (ADR-LFS-5). No Yjs/Automerge, no y-webrtc, no
  interop. CRDT bytes are opaque (`crdt_state BYTEA`) outside vault modules.
- Networking lives in Rust `gen_ui_core` on native surfaces (webrtc-rs); the
  web's JS shim owns `RTCPeerConnection` mechanics only — protocol decisions
  stay in the shared core state machine. Flutter uses `flutter_webrtc` for
  plumbing under the same rule.
- Cloud involvement = **momentary inference context only** (local LLM
  preferred). Assembled context is never written to lane-1 tables or logs;
  ContentBlocks produced from vault context carry `privacy_class: local` so
  sinks refuse them structurally.

## The mechanics (one doc, one protocol)

- ONE Loro doc per vault; maps `profile` / `preferences` / `agent_facts`.
  Blobs stay outside (content-addressed local files; hash in the doc).
- Persisted as encoded snapshots in `_vault_state(doc_id, crdt_state,
  version_vector, updated_at)` — PGlite / pglite-oxide / SQLite per tier.
- Sync protocol: exchange version vectors → `export_updates_since(peer_vv)`
  deltas both ways → commutative apply → converged. Frames chunked to 16 KiB,
  logical messages ≤256 KiB.
- Topology: full mesh of the USER'S OWN paired devices (roster of Ed25519
  device keys stored IN the doc, so pairing/revocation itself syncs).
- Signaling (offer/answer/ICE) is an untrusted dumb pipe: FRF SignalService in
  production, manual/QR or loopback signaler in dev — same transport contract.
  Peers mutually authenticate against the roster over the DataChannel before
  any delta flows; DTLS underneath, TURN sees only ciphertext.

## Reads, writes, agents

Feature code and agents use the typed `VaultRepository` facade — plain structs
in/out; nobody touches Loro APIs directly. Agents write learned facts as vault
mutations; retrieval over vault content uses the SEPARATE local-only vector
index (`client-rag` skill, `Vault` scope).

## Recovery model (tell the user honestly)

- Lost device: pair a replacement with any surviving device (full bootstrap
  over the channel); revoke the lost key in the roster.
- ALL devices lost: the vault is gone BY DESIGN. Offer only user-initiated,
  passphrase-encrypted export — never automatic cloud backup.

## Checklist

- [ ] Data classified `local`; enqueue-refusal test exists
- [ ] One Loro doc; blobs external; snapshots debounced
- [ ] Pairing = rostered keys; revocation propagates; signaler untrusted
- [ ] Web shim is dumb; protocol state machine shared with native
- [ ] Momentary-context rule enforced on every inference call using vault data
- [ ] Boundary tests: two in-process peers over loopback duplex — pairing,
      bidirectional convergence, new-device bootstrap, enqueue refusal
