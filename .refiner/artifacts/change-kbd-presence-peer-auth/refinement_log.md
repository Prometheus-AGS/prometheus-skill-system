# Refinement log — change-kbd-presence-peer-auth

Lightweight validation (no `.kbd-orchestrator/constraints.md` constraint
applies to this diff; full multi-agent adversarial-review skipped given
session cost — 53/53 tests green, including 8 new focused unit tests, is the
primary evidence for a security-sensitive change).

## Design decision (task 1, superseded)

Original plan (`is_peer_authorized`/Raft membership) was found to be the
wrong subsystem — Raft membership only reflects consensus voters (this
project is single-voter standalone), not P2P gossip peers. Revised design,
approved by the user: each `kbd-control` push is signed with the sending
node's own `kbd_runtime::DeviceSigner` (the same Ed25519 identity `Event`
signing already uses); the receiver verifies the signature against the
claimed signer's public key in its own already-replicated `KbdStateV2.devices`,
requiring `DeviceStatus::Active`. No new pairing ceremony, no dependency on
Raft membership.

## Changed files

- `substrate/kbd-runtime/src/lib.rs` — `DeviceSigner::sign_base64`, free
  `verify_ed25519_signature` helper (new, minimal API surface reused from
  existing Ed25519 primitives already in the crate)
- `substrate/sovereign-sync/src/domains.rs` — `SyncEnvelope` gains
  `signer_key_id`/`signature` + `signable_bytes`/`sign`/`verify`; removed
  the now-dead `KbdPresenceAdapter` (superseded — kbd-control is no longer a
  generic `DomainAdapter`, wire format is `KbdPresenceDocument`-specific)
- `substrate/sovereign-sync/src/rest_api.rs` — `AppState` holds a
  `KbdPresenceDocument` directly; `build_push_envelope`/
  `handle_incoming_message` special-case `kbd-control` before the generic
  adapter path (`build_presence_push_envelope`/`import_presence_message`);
  `presence_peer_is_authorized` factored out as a pure, directly-testable
  function; 8 new unit tests covering: valid sign/verify roundtrip, tampered
  payload, wrong public key, active enrolled device (authorized), unsigned
  (rejected), unknown signer (rejected), revoked device (rejected), forged
  signer_key_id with a different actual signing key (rejected)

## Constraint check (`.kbd-orchestrator/constraints.md`)

All N/A — no Codex plugin surface, generator, secrets, or launchd-script
changes in this diff.

## Build/test evidence

- `cargo test -p sovereign-sync`: 53/53 passed (35 lib incl. 8 new + 3
  domain_sync + 15 integration_tests)
- `cargo check -p kbd-runtime` / `cargo check -p sovereign-sync --tests`: clean

## Verdict

PASS — no blocking constraints, tests green, security-sensitive design
change was explicitly reviewed and approved by the user before
implementation (not self-certified). Proceed to archive.
