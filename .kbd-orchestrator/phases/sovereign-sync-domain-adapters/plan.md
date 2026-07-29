# Plan — sovereign-sync-domain-adapters

Change backend: **OpenSpec** (`openspec/` exists at project root).

Three changes, addressing the three gaps in `assessment.md`'s recommendation.
Ordered so the cheapest, most-informative change runs first — if real P2P
transport doesn't work at all, that's more urgent than hardening a feature
that depends on it.

## Change order

### 1. `change-verify-p2p-transport` — verify real P2P transport on real hardware
**Why first:** cheapest (no code, just verification + logging), and its
result determines urgency of the other two — if gossip genuinely doesn't
connect between two real machines, that's a bigger problem than presence
auth or a missing test.
**Recommended agent:** none (manual/operator verification — start the daemon
on both the Mac Pro and the laptop already paired earlier this session,
push a domain from one, confirm it lands on the other via `sync/status`
peers list and a content check).
**Depends on:** nothing.

### 2. `change-kbd-presence-peer-auth` — wire real peer authentication into kbd-control presence sync
**Why:** `kbd_sync::KbdPresenceDocument::import_authenticated` already has the
right gate (`peer_authorized: bool`) but nothing upstream supplies a real
authentication decision — `KbdPresenceAdapter::import_json` in `domains.rs`
currently accepts any syncable message. Needs a real answer to "is this
peer authorized" before presence sync is production-safe.
**Recommended agent:** rust-reviewer (security-sensitive: peer trust gate).
**Depends on:** understanding whatever peer/device trust model change-verify-p2p-transport's
findings suggest (e.g. if pairing already established a trust list, reuse it
rather than inventing a second one).

### 3. `change-learner-model-e2e-test` — add a learner-model end-to-end replication test
**Why:** `skill_index_replicates_end_to_end_between_two_nodes` proves the
pipeline works for skill-index; `learner-model` uses the same pipeline but
has no equivalent test, so a regression there wouldn't be caught.
**Recommended agent:** rust-reviewer or tdd-guide.
**Depends on:** nothing (can run independently/in parallel with change 2).

## Notes

- No `library-candidates.json` exists (Analyze stage was skipped for this
  phase) — all three changes are internal hardening, not library adoption.
- No evolver bridge — this phase is not part of an iterative-evolver cycle.
- Adversarial review of this plan skipped given session cost (~$215 at time
  of writing); flagged as a follow-up before treating this plan as final,
  same caveat as the assessment stage.
