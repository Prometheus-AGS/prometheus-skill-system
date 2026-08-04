# Change EXEC-003 task 5.1 — local certification

Date: 2026-08-04

Source boundary: `8d67d8e` plus this task's evidence/progress commit

Environment: macOS 26, x86_64 host; Rust 1.97.1; local execution only

## Evidence boundary

This record certifies Tier W source, host-native runtime behavior, deterministic
reference-component generation, and arm64 iOS/Android cross-builds on this Mac.
It uses no GitHub-hosted product tests, installed KBD service, KBD-backed
memory, or Sovereign Sync operation.

Physical iOS and Android runtime evidence remains `pending_evidence` as recorded
in task 4.4. Cross-builds do not upgrade that disposition.

## Test results

All tests passed locally:

| Surface | Tests | Covered behavior |
| --- | ---: | --- |
| `exec-contracts` | 11 | canonical requests, signed receipts, mutation/tamper, P-256 agility, deterministic contracts |
| `exec-core` | 20 | CAS retention/GC, policy, grants, receipt-log tamper/restart/concurrency |
| `exec-service` | 23 | lifecycle, response loss, replay/conflict, SSE resume, UDS, tamper/reconciliation |
| `exec-embedded` standalone | 3 | exact replay, ordered evidence, grant-pending, empty trust |
| `exec-tier-w` combined | 35 | Cranelift/Pulley parity, property corpus, replay, trust tamper/rollback, directory-scope linking, observed-memory rollback, limits, CAS |
| `exec-tier-w` bundled mobile | 15 | Pulley no-JIT profile, bundled pins, mobile limit behavior |
| `skill-ffi` | 12 | returned value, events, receipt/artifacts, verify, interruption, key boundary |
| `prometheus-exec` | 11 | real Python use case, Tier W service route, contracts, non-mutating false-green doctor |
| **Total** | **130** | **all green** |

The task 5.2 review remediation added and passed two Tier W regression tests,
raising the combined Tier W count from 33 to 35 and the cumulative matrix from
128 to 130. The 35-test combined profile and warnings-denied Clippy were rerun
after the final fixes.

The mobile-feature build of `exec-embedded` also compiled under its test profile;
its integration fixture is intentionally standalone-only, while the mobile
profile behavior is exercised by the 15 bundled-mobile Tier W tests and the
cross-built FFI artifact.

## Format and Clippy

`cargo fmt --check` passed for:

- `exec-contracts`
- `exec-core`
- `exec-service`
- `exec-tier-w`
- `exec-embedded`
- `skill-ffi`
- `prometheus-exec`

Warnings-denied Clippy passed for every native crate above and for both Tier W
feature sets:

- combined `estate,pulley,standalone`;
- `--no-default-features --features bundled-mobile`.

Warnings-denied `skill-ffi --lib` cross-Clippy also passed for:

- `aarch64-apple-ios`;
- `aarch64-linux-android` using NDK 28.0.12433566 and its API-35 Clang wrapper.

## Determinism, topology, and mobile cross-build

The following local commands passed:

```bash
scripts/check-exec-dependency-direction.sh
scripts/check-exec-tier-w-reference.sh
openspec validate change-exec-003-tier-w-mobile --strict

ANDROID_NDK_HOME=<ANDROID_NDK> \
CARGO_TARGET_DIR=<current-target-dir> \
  substrate/skill-ffi/build-mobile.sh all
```

The reference checker rebuilt the component twice in isolated directories,
required byte equality between both builds and the checked artifact, and
returned `exec_tier_w_reference=PASS`.

The mobile build entry point required the target-specific embedded mobile and
Tier W Pulley feature graph, rejected the Tier W native-execution feature, and
reported `jit_permitted=false` for both ABIs.

Task 5.2 independent review found the task 4.3 size measurements had allowed the
exec dependency graph to be dead-stripped because no generated FFI dispatcher
was linked. The corrected builds retain and export a generated dispatcher on
both baseline and current artifacts. Their corrected current sizes are:

- iOS arm64: 34,041,468 bytes;
- Android arm64: 43,520,400 bytes.

The fair deltas fail the 12 MiB gate. The cross-build/profile result remains
green, but mobile release readiness is now explicitly pending. See task 4.3's
superseding measurements.

## False-green and cleanup checks

- Contract regeneration matched the checked references byte-for-byte.
- A malformed doctor state returned structured failure without mutation and
  without translating an error/empty result to healthy.
- The real daemon use-case doctor returned healthy only after successful
  readiness, durable execution, and offline receipt verification.
- Test completion left no `prometheus-exec daemon` process.
- Device enumeration cleanup left no ADB server process.
- `git diff --check` passed and the gates created no tracked source change.

## Result

Task 5.1: **PASS** for the named local source/runtime and cross-build boundary.
Its earlier implied mobile size-gate pass is superseded by task 5.2's retained
dispatcher measurement, which **fails** the mobile release gate.

Remaining release work is the task 5.2 evidence bundle, artifact refinement,
distinct-model review, and change handoff. Mobile physical-device runtime status
remains pending and is not a failure of this narrower certification boundary.
