# Wire real peer authentication into kbd-control presence sync

## Why

`kbd_sync::KbdPresenceDocument::import_authenticated(bytes, peer_authorized:
bool)` already has the correct gate — it refuses to merge unless the caller
asserts the remote peer is authenticated — but `KbdPresenceAdapter` in
`substrate/sovereign-sync/src/domains.rs` (added by
`sovereign-sync-domain-adapters`) bridges through the generic
`DomainAdapter::import_json` path instead, which has no peer-authentication
concept at all: any message that passes the domain-privacy and
project-identity checks in `handle_incoming_message` gets merged. For a
domain explicitly named `kbd-control` and classed `Trusted`, this is a real
gap — presence data should only merge from a peer this device actually
trusts, not just "any syncable message with a matching project id."

## What Changes

- Give `handle_incoming_message` (or a `kbd-control`-specific branch of it)
  a real authentication decision to pass through to
  `KbdPresenceDocument::import_authenticated` — sourced from whatever this
  project's device-trust model already has (e.g. the enrolled-device list
  in `KbdStateV2.devices`, or a lighter-weight P2P-endpoint-to-known-peer
  mapping if pairing state exists at that layer).
- Route the `kbd-control` family through `KbdPresenceDocument` directly
  (not the generic JSON `DomainAdapter` bytes-only path) so the
  authenticated-import gate is actually reachable, rather than working
  around it.

## Impact

- `substrate/sovereign-sync/src/domains.rs` (`KbdPresenceAdapter`)
- `substrate/sovereign-sync/src/rest_api.rs` (`handle_incoming_message`'s
  `kbd-control`/`"kbd-presence"` family branch)
- Possibly `substrate/sovereign-sync/src/kbd_sync.rs` if the authentication
  source needs a new accessor
- No change to `kbd-runtime`'s device-enrollment model itself — this reuses
  whatever exists there, it doesn't add a new trust store
