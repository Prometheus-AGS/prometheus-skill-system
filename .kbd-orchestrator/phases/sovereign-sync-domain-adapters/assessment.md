# Assessment — sovereign-sync-domain-adapters

## Context

This phase's goals were implemented directly in the same session that created
the phase (via `/kbd-new-phase`), before the KBD assess/spec/plan stages ran.
This assessment documents the resulting gap between goals and actual code
state, retroactively, using first-hand knowledge of the implementation.

No OpenSpec capability exists yet for this work (`openspec/specs/` has
`sovereign-sync-daemon-health` and `sovereign-sync-ci`, but no
`sovereign-sync-domain-adapters` or equivalent) — a spec-stage gap noted below.

## Goal-by-goal status

1. **Wire skill-index domain adapter (Public CRDT/index rebuild) into
   sovereign-sync P2P push/pull** — ✅ Done. `SkillIndexAdapter` in
   `substrate/sovereign-sync/src/domains.rs` bridges `SkillIndex`'s real
   local entries to/from JSON; `SkillIndex` gained a `remote` field and
   `search()` merges local + remote. Verified end-to-end by
   `tests/domain_sync.rs::skill_index_replicates_end_to_end_between_two_nodes`
   and live against the running daemon (curl push, `sync/status`).

2. **Wire learner-model:<learner-id> domain adapter (Trusted CRDT merge)
   into sovereign-sync P2P push/pull** — ✅ Done. `LearnerModelAdapter`
   bridges the real `learner-model`/`storage-provider` crates
   (`LearnerModelStore<LocalDirAdapter, LoroAdapter>`). Not covered by a
   dedicated integration test (only exercised via `crdt_export_snapshot_and_apply_roundtrip`-style
   unit coverage in the existing suite) — **gap**: no end-to-end
   two-node replication test for this specific domain, unlike skill-index.

3. **Wire kbd-presence:<project-id> domain adapter (Trusted ephemeral CRDT)
   for non-authoritative KBD presence** — ⚠️ Done, with a scoping change and
   a known gap. Discovered mid-implementation that `kbd_sync.rs` already had
   a mature, tested `KbdPresence`/`KbdPresenceDocument` design under the
   domain name `kbd-control:<project-id>` (not `kbd-presence`) — aligned to
   that existing schema instead of shipping a second, inconsistent one.
   **Gap**: `kbd_sync::KbdPresenceDocument::import_authenticated` gates merge
   on `peer_authorized: bool`, tying presence import to authenticated peer
   transport; that authentication is not wired into the gossip layer, so the
   current `KbdPresenceAdapter` merges any syncable message without a real
   peer-authentication check. Documented in-code; not yet fixed.

4. **Define and enforce domain envelope validation (project/learner identity,
   privacy class) before any peer accepts a delta** — ✅ Done for privacy
   class (structural: `SyncManifest::is_syncable` rejects unregistered/Local
   domains before any adapter runs, verified live — `surreal-memory` returns
   HTTP 403). ⚠️ Partial for identity: `handle_incoming_message` checks
   `envelope.identity` against local project/learner identity for Trusted
   domains, but **a genuine cross-project-rejection test could not be
   written** — `KbdStateV2.project_id` is only populated after a KBD run is
   initialized, which needs more Runtime/Actor test scaffolding than this
   phase built. The rejection branch exists and is exercised by the
   happy-path (matching-identity) test, but a true mismatch has not been
   proven under test.

5. **Add end-to-end replication proof per data-scope.md** — ✅ Done for the
   9-point checklist's core claims (live peer identity via P2PNode
   construction, named domain + identity, real CRDT delta bytes, bytes
   transmitted, destination trust decision structurally proven via the
   surreal-memory rejection, destination import/commit + content-level
   assertion via the skill-index test, negative assertion that Local data
   never moves). ⚠️ **Gap**: real iroh/iroh-gossip network transport
   (`P2PNode::start()`/`broadcast()` over an actual connection between two
   processes) was not exercised — the integration test hands the envelope
   directly to the peer rather than routing it through a live gossip
   connection, and a live smoke test against the real daemon showed
   `P2PNode::start()` not completing within the tested window in this
   sandboxed environment (no error logged; unconfirmed whether this
   reproduces outside the sandbox).

6. **Explicitly exclude surreal-memory, secrets, raw transcripts, and KBD
   authoritative Raft state from any CRDT sync domain** — ✅ Done for
   surreal-memory (explicit `PrivacyClass::Local` registration, tested).
   KBD authoritative Raft state was never touched by the CRDT path by
   construction (the `kbd-control` adapter only ever reads
   `KbdControlPlane::status()` for display, never writes through the CRDT
   merge path back into `Runtime`/`KbdStateV2`). Secrets/raw transcripts were
   never wired to any domain in the first place — no adapter exists for
   them, so exclusion is structural (unregistered domains are never
   syncable), not an explicit denylist entry; acceptable given
   `privacy_for_family`'s default is "unregistered = not syncable."

## Cross-cutting gaps (not in the original goal list, discovered during build)

- **Bug fix, not a goal**: `p2p.rs`'s `broadcast()` left node state stuck at
  `"Syncing"` forever after any failed broadcast (found via live testing).
  Fixed in this session; worth calling out since it affects `sync/status`
  accuracy independent of the domain-adapter feature itself.
- **HTTP timeout**: `prometheus-cli`'s daemon-command client timeout was 2s,
  too short for a command that commits through OpenRaft; bumped to 30s.
  Unrelated to sovereign-sync directly but was blocking KBD operations this
  phase depends on (`prometheus kbd claim`/`/kbd-new-phase` itself).
- **Pre-existing deadlock** (kbd-runtime device-signer enrollment + lease
  conflicts) had to be fixed before this phase's own `/kbd-new-phase` could
  even run. Tracked and fixed separately; not a sovereign-sync gap, but
  materially blocked this phase's KBD lifecycle tracking until resolved.

## Recommendation for next stage

Given the substantial gap between "goals as written" and "what's actually
proven under test," recommend `/kbd-plan` (skipping `/kbd-analyze` — no new
external library research is needed; the remaining gaps are internal
hardening, not landscape research) with three concrete follow-up changes:

1. Wire real peer authentication into `kbd_sync::KbdPresenceDocument`'s
   import path before treating `kbd-control` presence sync as production-
   ready (currently: any syncable message merges, no peer check).
2. Add a `learner-model` end-to-end replication test mirroring
   `skill_index_replicates_end_to_end_between_two_nodes`.
3. Confirm real P2P transport (`start()`/`broadcast()`/`recv()`) works over
   an actual network link on the user's own two machines (Mac Pro + laptop)
   — the one thing not verified in this session's sandboxed environment.

## Adversarial review

Skipped for this retroactive assessment given session cost already far past
typical budget for this phase; flagging as a follow-up rather than blocking
the stage-gate transition. If a fresh session picks up `/kbd-analyze` or
`/kbd-plan` for this phase, running `/adversarial-review --mode artifact
assess` on this file first is recommended before trusting it as planning
input.
