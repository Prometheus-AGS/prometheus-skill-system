# Evidence: change-exec-003-tier-w-mobile

Date: 2026-08-04

Source boundary: `4da9748` plus the task 5.2 evidence/progress commit

Environment: macOS 26, x86_64 host; Rust 1.97.1; Wasmtime 46.0.0; local execution only

## Evidence boundary

This record certifies the Tier W contracts, Cranelift host execution, Pulley
portable replay, embedded/FFI behavior, deterministic reference component, and
iOS/Android arm64 cross-build profiles on this Mac. It does not use GitHub
Actions as product-test evidence and does not invoke the installed KBD service,
KBD-backed memory, or Sovereign Sync.

Physical iOS and Android execution remains `pending_evidence`: the paired
iPhone was offline and no Android device was attached. Cross-build success is
not represented as device-runtime certification.

## Receipt-producing reference execution

The checked `entity-graph-optimize` component executed through
`EmbeddedExecutionApi` under the exact hash-pin authorization mode. The
producer generated a fresh Ed25519 device identity in an automatically removed
temporary state directory. The archive contains only the public verification
key.

| Field | Observed value |
| --- | --- |
| request ID | `c8595ef3-0fa2-4f1d-ae9f-35e42a24a532` |
| request hash | `sha256:e9b3c756451ab33d40c2a575e817a8aeca64e99d9136ada74121c52a20a61291` |
| run ID | `31b9a730-541b-46ec-8f6b-45d08b275ee7` |
| state / evidence | `succeeded` / `verified` |
| execution tier / backend | `w` / `cranelift` |
| component | `prometheus:component@0.1.0`, Wasmtime `46.0.0` |
| component digest | `sha256:ba438895404a23985d5226735b8f362cf3e8044894a1140852ba0992f2fdbe78` |
| deterministic projection | `sha256:1f22e6ce4501c862cb2a642d6917a2d14e6484477492df620d7e53d124da9c64` |
| fuel consumed | `25,257` |
| peak guest memory | `2 MiB` |
| receipt hash | `sha256:2cf13e35e94a887a134aa2de1ea11f1ad0585ef0a78bfc81ce007bc16af41c95` |
| receipt signer | `ed25519:a14ceb53e49f58ad6efabb257747751542116f1302a77a1b667dd7274042496d` |
| producer wall time | `919 ms` |

The component returned the actual JSON value:

```json
{"kbd":{"available":false},"evolver":{"available":false},"refiner":{"available":false},"openspec":{"available":false}}
```

The three hash-linked lifecycle events are `run.accepted`, `run.started`, and
`run.completed`. Cursor retrieval, terminal receipt retrieval, content-addressed
stdout/stderr retrieval, and an exact same-ID replay all returned the original
durable evidence without re-execution.

## CLI verification entry point and portable replay

The checked local CLI separately invoked the shared core verifier for the
archived request/receipt and replayed the exact component under the Pulley
portable profile. This proves CLI wiring and byte-for-byte agreement with the
embedded entry point; it is not represented as an implementation-independent
verifier:

```bash
EVIDENCE=.kbd-orchestrator/phases/prometheus-exec-code-execution-engine/evidence/change-exec-003-tier-w-mobile
EXEC_PUBLIC_KEY=$(jq -er '.publicKey' "$EVIDENCE/public-key.json")
crates/prometheus-exec/target/debug/prometheus-exec verify \
  --receipt "$EVIDENCE/receipt.json" \
  --public-key "$EXEC_PUBLIC_KEY" \
  --request "$EVIDENCE/request.json" \
  --component "$EVIDENCE/component.wasm" \
  --format json
```

Result: `valid: true`; receipt invariants, signature/key identity, and request
hash all passed. Portable replay reported no mismatches across state,
stdout/stderr, artifacts, failure, component authorization, engine version,
deterministic projection, and backend execution.

## Archived public bundle

The independently usable bundle is under
[`change-exec-003-tier-w-mobile/`](change-exec-003-tier-w-mobile/):

- `request.json` — canonical execution request;
- `receipt.json` — signed verified Tier W receipt;
- `public-key.json` — Ed25519 public material only;
- `component.wasm` — exact hash-pinned reference component;
- `run.json`, `status.json`, and `exact-replay.json` — original and replay state;
- `events.json` — ordered hash-linked lifecycle events;
- `stdout.json` and `stderr.txt` — retrieved CAS output streams;
- `verification.json` — embedded signature and Pulley replay result;
- `cli-verification.json` — separate CLI entry-point verification and replay result;
- `verification-provenance.json` and `cli-verification-provenance.json` —
  explicit producer and replay-backend identities;
- `producer-measurement.json` — bounded producer measurements;
- `review-opus-initial.json`, `review-opus-remediation-1.json`, and
  `review-opus-remediation-2.json` — distinct-model blocker discovery and both
  archive-integrity correction requests;
- `review-opus-final.json` — distinct-model final acceptance with no high or
  medium blocker;
- `SHA256SUMS` — exact archive file identities.

The bundle contains no private key, credential, token, hardware identifier,
provisioning identity, or absolute temporary path.

## Review and artifact refinement

Artifact-refiner completed `specify`, `plan`, `execute`, `reflect`, and
`persist` across three iterations and finalized with `convergence_status =
converged`. Its public distribution is byte-aligned with this bundle and has a
separately validated complete checksum manifest.

A read-only Claude Opus 5 review first found the K/V linker and dead-stripped
measurement blockers, then forced two semantic archive sweeps. The final review
returned `PASS` with no high or medium finding. Its remaining low findings are
recorded with dispositions in `review-opus-final.json`; none expands the scoped
desktop certification claim.

## Local certification matrix

The task 5.1 matrix plus task 5.2 remediation passed 130 tests:

| Surface | Passed |
| --- | ---: |
| contracts | 11 |
| core/CAS/policy/grants/receipt log | 20 |
| service/UDS/events/reconciliation | 23 |
| embedded API | 3 |
| Tier W combined Cranelift + Pulley + estate | 35 |
| Tier W bundled-mobile Pulley profile | 15 |
| FFI | 12 |
| CLI/daemon/doctor | 11 |

Format and warnings-denied Clippy passed for every affected native workspace,
both Tier W feature profiles, and the iOS/Android arm64 FFI targets. Additional
green gates covered strict OpenSpec, dependency direction, deterministic
two-build component reproduction, byte-identical generated contracts, release
mobile cross-builds, and zero orphan daemon/ADB processes.

## Mobile cross-build and size status

| ABI | Baseline | Current | Delta | 12 MiB gate |
| --- | ---: | ---: | ---: | --- |
| iOS arm64 | 8,106,168 bytes | 34,041,468 bytes | +25,935,300 bytes | **FAIL** |
| Android arm64 | 11,818,664 bytes | 43,520,400 bytes | +31,701,736 bytes | **FAIL** |

Both target feature graphs select the embedded mobile/Tier W Pulley execution
profile and bind `jit_permitted=false`. Wasmtime retains its Cranelift compiler
component to translate source Wasm into Pulley bytecode; this is not native JIT
execution. The build entry point rejects the Tier W native-execution profile on
mobile targets and verifies the generated FRB dispatcher is exported.

The prior small deltas were invalid dead-stripping measurements. The fair
baseline and current builds both retain a generated dispatcher; those deltas
exceed the 12 MiB gate. This archive therefore records successful cross-builds
and a failed mobile release-size gate, not mobile release readiness.

## Certification status

| Claim | Status |
| --- | --- |
| desktop x86_64 Tier W execution and signed receipt | **locally runtime-certified on this host** |
| Pulley portable deterministic replay | **locally certified** |
| embedded API / host-native FFI behavior | **locally certified** |
| iOS arm64 and Android arm64 source/cross-build | **locally cross-build-certified** |
| iOS arm64 and Android arm64 retained size gate | **failed; mobile Tier W release pending** |
| physical iOS Tier W runtime | **pending evidence** |
| physical Android Tier W runtime | **pending evidence** |

No broader installed-service, remote-peer, mobile-device, or external deployment
claim should be inferred from this record.
