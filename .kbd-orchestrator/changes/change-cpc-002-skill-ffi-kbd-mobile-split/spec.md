# change-cpc-002-skill-ffi-kbd-mobile-split

**Title:** Remove the skill-ffi dependency on kbd-mobile so no pack crate references a relocated crate  
**Repository:** `prometheus-skill-pack`  
**Phase:** control-plane-to-companion  
**Depends on:** none  
**Backend:** native-kbd

## Why

kbd-mobile is the mobile sync wire and moves to the Companion (D-02). `substrate/skill-ffi` depends on it by path, and `mobile_wire_is_byte_compatible_with_sovereign_sync` lives in sovereign-sync's tests. After change-cpc-009 deletes `substrate/kbd-mobile`, the pack must still resolve, so the path dependency must be gone entirely, not merely optional.

## What Changes

- Remove the kbd-mobile dependency from skill-ffi's manifest entirely and delete the KBD-sync API surface from `api.rs`/`lib.rs`; a `kbd-sync` feature name is reserved as an empty no-op feature so third-party FFI consumers see a stable feature list. The FFI round-trip tests run unchanged.
- Move `mobile_wire_is_byte_compatible_with_sovereign_sync` into `substrate/kbd-mobile/tests/wire_compat.rs` with a dev-dependency on sovereign-sync in kbd-mobile's manifest (both crates move together in change-cpc-004, so the interim dev-dependency direction is acceptable).
- Regenerate `frb_generated.rs` for the reduced surface at the exact `flutter_rust_bridge =2.12.0` pin.

## Scope

Files this change may create or edit (tasks.json `files` is the per-task view):

- `substrate/kbd-mobile/Cargo.toml`
- `substrate/kbd-mobile/tests/wire_compat.rs`
- `substrate/skill-ffi/Cargo.toml`
- `substrate/skill-ffi/src/api.rs`
- `substrate/skill-ffi/src/frb_generated.rs`
- `substrate/skill-ffi/src/lib.rs`
- `substrate/sovereign-sync/tests/domain_sync.rs`

## Capabilities

- `skill-ffi-portability` (new)

## ADDED Requirements

### Requirement: skill-ffi resolves and round-trips with no kbd-mobile dependency
skill-ffi SHALL compile with no dependency on kbd-mobile, sovereign-sync, or iroh in any feature configuration, and its FFI round-trip tests SHALL assert returned values as before.

#### Scenario: Dependency graph
- **WHEN** `cargo metadata` is run for skill-ffi with all features
- **THEN** no kbd-mobile, sovereign-sync, iroh, or iroh-gossip package is listed

#### Scenario: Round-trip evidence
- **WHEN** the FFI test target runs
- **THEN** all existing round-trip tests pass unchanged

### Requirement: Wire-compatibility test travels with kbd-mobile
`mobile_wire_is_byte_compatible_with_sovereign_sync` SHALL live in kbd-mobile's own test target and pass in isolation.

#### Scenario: Isolated run
- **WHEN** `cargo test --manifest-path substrate/kbd-mobile/Cargo.toml --test wire_compat` runs
- **THEN** it passes

## Constraints

- Implementation-first, integration-only evidence: no unit tests as delivery evidence; every acceptance criterion below has a command in `verification.md`'s verify block, run after the coherent edit batch, locally only.
- One Cargo build machine-wide at a time; `cargo check -p <crate>` only as a narrowly targeted diagnostic.
- Constraints C-01..C-05 apply; `npm run validate:codex` and `docs/codex-plugin.md` in the same change when plugin surfaces or install flow move; `shared/services.manifest.json` regenerated in the same change when a plist or unit changes.
- The pack never depends on the Companion; the Companion consumes pack crates by git rev (D-02).
