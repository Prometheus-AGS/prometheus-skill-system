# Tasks

- [x] Decide the authentication source: reuse `KbdStateV2.devices` (enrolled
      signer keys) vs. a new P2P-endpoint-to-trust mapping
- [x] Thread a real `peer_authorized: bool` from `handle_incoming_message`
      into the `kbd-control` domain's merge path
- [x] Route `kbd-control` through `KbdPresenceDocument::import_authenticated`
      instead of the generic `DomainAdapter::import_json` bytes path
- [x] Add a test proving an unauthenticated peer's presence message is
      rejected (mirrors `kbd_sync::tests::presence_requires_an_authenticated_peer_and_contains_no_authority`,
      but through the full `sync_push`/`handle_incoming_message` pipeline
      rather than calling `KbdPresenceDocument` directly)
- [x] `cargo test -p sovereign-sync` green
