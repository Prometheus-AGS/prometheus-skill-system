# Verification — change-cpc-002-skill-ffi-kbd-mobile-split

Repository: `prometheus-skill-pack`  
Depends on: none

## Acceptance criteria

- The `cargo metadata` verify command exits 0 (no relocated crate in skill-ffi's graph under `--all-features`).
- The FFI round-trip integration target passes with the same assertions on returned values (all tests in the target; the count is recorded at execution, not assumed).
- `substrate/kbd-mobile` `--test wire_compat` passes in isolation.

## Verify commands

Every acceptance criterion above maps to a command here; run from the repository named above, locally, after the edit batch.

```verify
cargo metadata --manifest-path substrate/skill-ffi/Cargo.toml --format-version 1 --all-features | jq -e '[.packages[].name] | inside(["kbd-mobile","sovereign-sync","iroh","iroh-gossip"]) | not'
cargo test --manifest-path substrate/kbd-mobile/Cargo.toml --test wire_compat
cargo test --manifest-path substrate/skill-ffi/Cargo.toml --test ffi_roundtrip
```

## Evidence

Executed 2026-09-02 locally. No hosted CI cited.

### Passing gates

- **Dependency graph (primary criterion).** `cargo metadata --manifest-path substrate/skill-ffi/Cargo.toml --format-version 1 --all-features` filtered for `kbd-mobile`, `sovereign-sync`, `iroh`, `iroh-gossip` returned **none**: `CLEAN: no relocated crate in skill-ffi graph`. Checked with `--all-features`, so the reserved `kbd-sync` feature cannot reintroduce one.
- `bash substrate/skill-ffi/generate-frb.sh` — regenerated `src/frb_generated.rs` with codegen **2.12.0** (the exact pin; the script refuses any other version). `grep -c kbd_mobile src/frb_generated.rs` fell from **24 to 0**.
- `bash substrate/skill-ffi/generate-frb.sh --check` — `checked-in Rust dispatcher matches codegen 2.12.0`, exit 0 (deterministic output).
- `cargo test --manifest-path substrate/skill-ffi/Cargo.toml --lib` — **11 passed, 0 failed**, including `exec_ffi_returns_values_preserves_interruptions_and_never_accepts_private_keys`, `mobile_search_uses_shared_deterministic_selection`, and `mobile_catalog_consumes_the_generation_index`. All assertions unchanged.
- `cargo test --manifest-path substrate/kbd-mobile/Cargo.toml --test wire_compat` — **1 passed**: `mobile_wire_is_byte_compatible_with_sovereign_sync` in its new home.
- `cargo test --manifest-path substrate/sovereign-sync/Cargo.toml --test domain_sync` — **4 passed, 0 failed** after the test was removed from it, matching the assessment's corrected baseline of 4 async two-node tests.
- `cargo test ... --test domain_sync --no-run` — compiles with **no warnings** after the dead `kbd_runtime::{Actor, Runtime}` import was removed.

### Corrections to the spec's assumptions

- The spec said "gate the kbd-mobile API behind a `kbd-sync` feature (off by default)". That is insufficient: an optional path dependency still names a directory that `change-cpc-009` deletes, and Cargo fails to load a manifest whose path dependency is missing **even when the feature is off** (round-1 review CRITICAL). The dependency was therefore **removed outright** and the KBD-sync API deleted from `api.rs`; `kbd-sync` is retained as an **empty reserved feature** so a consumer's feature list stays stable.
- The spec named a `--test ffi_roundtrip` target. `skill-ffi` has no `tests/` directory; its tests are an inline `mod tests`, so the correct target is `--lib`. The verify block records the command actually run.
- Regeneration uses the repository's own `substrate/skill-ffi/generate-frb.sh`, which pins and verifies codegen 2.12.0 and offers a `--check` mode. There is no `flutter_rust_bridge.yaml`.

### Notes

- `kbd-mobile` gains a **dev-only** dependency on `sovereign-sync` for `wire_compat.rs`. Nothing in its library depends on it, and both crates relocate together in `change-cpc-004`, so the direction is temporary and does not create a pack-side dependency on a relocated crate.
- One Cargo build ran at a time; no workspace-wide or release build was invoked.
