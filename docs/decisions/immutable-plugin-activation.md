# Decision: activate plugins as immutable content-addressed generations

**Status:** accepted · 2026-08-03 · release 1.6.1

## Context

Hooks and skills are consumed by many harnesses with different filesystem requirements. Editing an active install or copying files independently creates mixed versions, stale absolute paths, and rollback ambiguity.

## Decision

The installer stages a complete plugin payload, computes a canonical manifest and generation hash, verifies all files and modes, materializes the generation under `generations/<sha256>`, certifies all 14 target receipts, then atomically switches `current`. `previous` retains the rollback generation.

Twelve targets link through the active generation. Codex and MiniMax receive verified real-directory copies with generation receipts. Hook configuration uses stable dispatchers that resolve dependencies through `current`; activation and rollback do not rewrite registered hook paths.

Uninstall removes only Prometheus-owned projections. Collisions with unrelated user content are preserved and reported. Hardcoded release-version paths are invalid.

## Alternatives considered

- **In-place update:** rejected because readers can observe mixed payloads and rollback has no boundary.
- **Copy every target independently:** rejected because partial failure creates 14 different active versions.
- **Symlink every target:** rejected because Codex and MiniMax require real skill directories.
- **Rewrite every hook on activation:** rejected because host config mutation expands the failure surface.

## Consequences

Installations retain generation payloads and receipts, and activation needs a staging/verification pass. The additional disk use buys atomicity, deterministic parity, collision safety, and one-step rollback.

## Verification

`node scripts/install-plugin-generation.js --verify` checks the manifest, active and rollback pointers, 14 target modes/receipts, stable dispatchers, and stale paths. Tests cover clean install, collision refusal, copy restoration, activation failure, rollback, and uninstall ownership boundaries.

