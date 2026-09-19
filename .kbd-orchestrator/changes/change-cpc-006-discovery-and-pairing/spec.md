# change-cpc-006-discovery-and-pairing

**Title:** Add mDNS and Mainline DHT address lookup with pairing as admission, DHT publication off by default  
**Repository:** `prometheus-companion`  
**Phase:** control-plane-to-companion  
**Depends on:** `change-cpc-004-relocate-sovereign-sync`  
**Backend:** native-kbd

## Why

Goal 3 asks for Kademlia-style discovery without hand-maintained peer lists. iroh 1.0.x provides `iroh-mdns-address-lookup` and `iroh-mainline-address-lookup` (0.5.0, depend on iroh ^1.0.0). Admission stays Ed25519 pairing per the Companion spec §25.2 and HMA doctrine; the spec's Noise attribution is wrong and DHT publication exposes reachability records, so it is opt-in (D-01, D-10, D-13).

## What Changes

- Add both lookup crates and wire them into the endpoint builder alongside the N0 preset; keep the `[peers].bootstrap` list as a third source.
- Config: `[discovery] mdns = true, dht = false` defaults; DHT opt-in per operator; document the exposure.
- Pairing: keep ticket export/import and `authorize_endpoint`; add QR encoding of the ticket in the Companion; correct spec §14 (remove Noise) and amend the §25.2 iroh row to state the DHT opt-in.
- Integration: two real endpoints on loopback discover each other via mDNS with an empty bootstrap list and join gossip.

## Scope

Files this change may create or edit (tasks.json `files` is the per-task view):

- `crates/prometheus-substrate/src/pairing.rs`
- `crates/sovereign-sync/Cargo.toml`
- `crates/sovereign-sync/src/config.rs`
- `crates/sovereign-sync/src/p2p.rs`
- `crates/sovereign-sync/tests/discovery.rs`
- `docs/00-architecture-and-implementation-plan.md`

## Capabilities

- `companion-discovery` (new)

## ADDED Requirements

### Requirement: Peers discover each other without a bootstrap entry
WHEN two nodes with an empty bootstrap list start on the same network, THEN each resolves the other's EndpointId via mDNS and joins the gossip topic.

#### Scenario: LAN discovery
- **WHEN** the two-endpoint discovery test starts both endpoints with `.start()`
- **THEN** each observes `NeighborUp` for the other within the budget

### Requirement: DHT publication is opt-in
WHEN `[discovery] dht` is unset or false, THEN no pkarr record is published to the Mainline DHT; WHEN true, THEN publication occurs and is logged.

#### Scenario: Default
- **WHEN** the node starts with default config
- **THEN** the Mainline lookup is not registered on the endpoint

### Requirement: Admission remains pairing
An endpoint that is discovered but not authorized SHALL be refused at the sync layer.

#### Scenario: Unauthorized peer
- **WHEN** an unpaired endpoint sends a sync envelope
- **THEN** it is rejected before any bytes are applied

## Cross-repo execution rule (resolves assessment Q3)

The skill-pack KBD run `sovereign-sync-service-reliability-20260829` owns this phase and every change in it. Changes whose Repository is `prometheus-companion` are executed in the Companion checkout, mirrored into the Companion's `openspec/changes/<change-id>/` as `proposal.md` + `tasks.md` for that repository's own history, and their evidence (commands, outputs, commit hashes) is recorded in this change's `verification.md` in the pack. The Companion's runtime records nothing for this phase until its open phase `docs-review-for-build-assessment` is reconciled (assess artifacts exist on disk dated 2026-08-23 while its runtime still reports `ready`); that reconciliation is a task of change-cpc-003. No change mixes repositories: pack-side edits live in pack-side changes.

## Constraints

- Implementation-first, integration-only evidence: no unit tests as delivery evidence; every acceptance criterion below has a command in `verification.md`'s verify block, run after the coherent edit batch, locally only.
- One Cargo build machine-wide at a time; `cargo check -p <crate>` only as a narrowly targeted diagnostic.
- Companion AGENTS.md applies: no `unwrap()` in library code, exact pins, iroh in the Infrastructure layer, §0.2 observed problems only, `bash scripts/audit-all.sh` must exit 0 (the scaffold ships the audit scripts, commit 773aa5e).
- New dependencies are a human pin: `iroh-mdns-address-lookup 0.5.0` and `iroh-mainline-address-lookup 0.5.0` are staged in `docs/decisions/versions-pins-proposal.md` by change-cpc-003; this change stays BLOCKED until the operator's `versions.toml` edit includes them.
- The pack never depends on the Companion; the Companion consumes pack crates by git rev (D-02).

## Open Questions

- Whether short-code (spake2) pairing is v1 or QR tickets suffice.

## Unresolved review findings

Adversarial review (artifact mode, cross-model judge k3 via rest-gateway, `cross_model_check: verified-distinct`) ran two rounds on the change set. Round 1 (4 CRITICAL) was corrected in the regenerated set. Round 2 returned the CRITICAL below; its fix is applied in this change but, under the two-round cap, the finding is carried verbatim so plan and execute inherit it explicitly rather than trusting this document's own fix.

- CRITICAL (round 2, change-cpc-006-discovery-and-pairing/tasks.json): "change-cpc-006 adds two brand-new crate dependencies with exact pins via agent edits, with no human pin-edit step, violating the carried constraint that new crate pins in the Companion are a human edit." Evidence: task 1 edited `crates/sovereign-sync/Cargo.toml` directly; change-cpc-003's staged pin list omitted both lookup crates; change-cpc-005 introduced `dirs_next` with no pin step. Disposition: the two lookup crates and `dirs_next` are added to change-cpc-003's staged pins proposal, this change's task 1 now asserts the operator's `versions.toml` contains them before any Cargo edit, and every Companion change carries the same constraint bullet. Not re-vetted.
