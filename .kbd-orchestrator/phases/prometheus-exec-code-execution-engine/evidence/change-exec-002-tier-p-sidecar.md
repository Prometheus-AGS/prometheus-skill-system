# Evidence: change-exec-002-tier-p-sidecar

Date: 2026-08-04

Certified implementation commit: `364fc5c87d79813de759fcff2e7fc9722d7fc717`

Environment: macOS 26.5.2, MacPro7,1, x86_64 execution architecture; local execution only

## Evidence boundary

This record certifies the change-002 Tier P sidecar on the named macOS host. It covers real Seatbelt execution, signed request/receipt linkage, exact artifact verification, durable status retrieval, SIGKILL restart recovery, UDS permissions and peer health, readiness, doctor behavior, local Rust gates, portable Linux planning fixtures, and Linux-musl cross-compilation.

It does **not** claim:

- a locally installed or LaunchAgent-managed service;
- a signed installation binary (the evidence build is an unsigned local release binary identified by hash);
- Linux Tier P runtime certification;
- Windows Tier P availability;
- Tier W, MCP, FFI, remote dispatch, or mobile behavior;
- any GitHub-hosted product-test evidence.

No KBD service, KBD-backed memory, Sovereign Sync operation, or hosted CI workflow was invoked to create this record.

## Real use case: sandboxed incident-risk aggregation

The release binary executed [`incident-risk.py`](change-exec-002-tier-p-sidecar/incident-risk.py) under macOS Seatbelt. The program read a declared content-addressed [`incident-batch.json`](change-exec-002-tier-p-sidecar/incident-batch.json), computed a deterministic risk summary, printed it, and wrote [`outputs/risk-summary.json`](change-exec-002-tier-p-sidecar/outputs/risk-summary.json) through `PROMETHEUS_OUTPUT_DIR`.

Input:

```json
{"incidents":[{"severity":3,"exposure":4},{"severity":5,"exposure":2},{"severity":2,"exposure":7}]}
```

Returned stdout and output artifact:

```json
{"incident_count":3,"max_risk":14,"total_risk":36}
```

The terminal run was:

| Field | Observed value |
|---|---|
| run ID | `17e6fa74-50fc-4c12-a4d9-6bc8627e2c6e` |
| request ID | `3b7df805-f1bf-4d97-b87a-85c63f1c0880` |
| request hash | `sha256:99ee5010d1e43eeb965f02359e1eafeac28658f8a21c099d7dfd1090abc48c43` |
| state | `succeeded` |
| evidence class | `attested` |
| tier / backend | `p` / `seatbelt` |
| exit status | `0` |
| wall clock | `108 ms` |
| sandbox profile | `sha256:8278427b9632278e5e3dfd1d712ee0e8cd984ecd483ec7a2fbc2049286a60869` |
| Python toolchain | `sha256:179301dcb41ea78accc3fa0048a7e6f6710d891945a751a34addd622020c1818` |
| receipt hash | `sha256:899016657d6f72b9cd75d63a4ef05de601fc50f2dd268a1f4e9abaa8265e7618` |
| output artifact | `sha256:99b31f41b94ec6b079fbf7687a949c851fb55cbe47029d37b7f2cb3be3c13e56`, 50 bytes |

The complete public artifacts are archived in [`change-exec-002-tier-p-sidecar/`](change-exec-002-tier-p-sidecar/):

- [`request.json`](change-exec-002-tier-p-sidecar/request.json) — signed request;
- [`receipt.json`](change-exec-002-tier-p-sidecar/receipt.json) — signed terminal receipt;
- [`public-identity.json`](change-exec-002-tier-p-sidecar/public-identity.json) — public verification material only;
- [`run.json`](change-exec-002-tier-p-sidecar/run.json) — terminal API response;
- [`verify.json`](change-exec-002-tier-p-sidecar/verify.json) — successful offline request/receipt/artifact verification;
- [`verify-wrong-root.json`](change-exec-002-tier-p-sidecar/verify-wrong-root.json) — expected nonzero verification when the artifact tree is absent;
- [`doctor-restart.redacted.json`](change-exec-002-tier-p-sidecar/doctor-restart.redacted.json) — all required checks passing after SIGKILL restart, with the temporary root redacted.

## Independent offline verification

From the repository root, with the locally built release binary:

```bash
EVIDENCE_DIR=.kbd-orchestrator/phases/prometheus-exec-code-execution-engine/evidence/change-exec-002-tier-p-sidecar
PUBLIC_KEY=$(jq -er '.publicKey' "$EVIDENCE_DIR/public-identity.json")
crates/prometheus-exec/target/release/prometheus-exec verify \
  --receipt "$EVIDENCE_DIR/receipt.json" \
  --public-key "$PUBLIC_KEY" \
  --request "$EVIDENCE_DIR/request.json" \
  --artifacts "$EVIDENCE_DIR" \
  --format json
```

Observed result: `valid: true`, no failures, with all four checks present:

- `receipt.invariants`
- `receipt.signature`
- `request.hash`
- `artifact.hash`

Pointing `--artifacts` at a tree without `outputs/risk-summary.json` returned `valid: false`, an `artifact.read` failure, and exit status 1. The verifier did not translate missing evidence into success.

## Release binary and host boundary

| Check | Observed value |
|---|---|
| `prometheus-exec --version` | `prometheus-exec 1.7.0` |
| release binary SHA-256 | `18b109b4818b6e868b4ccf7346585c8c069d491eb7bc698b11e0f215602b1780` |
| binary format | `Mach-O 64-bit executable x86_64` |
| signing state | unsigned local build; not installation evidence |
| private identity mode | `0600` |
| UDS mode | `0600` |
| executing device platform | `macos-x86_64` |

The private identity is intentionally absent from the archive. Only the Ed25519 public key and key ID required for independent receipt verification are checked in.

## Restart and doctor evidence

The daemon was terminated with SIGKILL, leaving its socket path behind. A second release-binary daemon invocation recovered the stale socket, reopened the same identity, ledger, receipt log, and CAS, and returned a status response byte-identical to the original terminal `run` response (`SHA-256 6d67210e7c8de3b24e5b2aaaa8fc7f15113050fc26e22c2dcff4c70d426467ff`).

The restarted doctor reported `healthy: true` with all required checks passing:

- binary identity;
- receipt identity;
- mode-0600 Unix socket;
- same-UID peer `/health`;
- readiness;
- macOS Seatbelt backend;
- one structurally valid record and zero in-flight runs;
- five verified content-addressed blobs.

The focused startup fixture measured:

| Metric | Result | Requirement |
|---|---:|---:|
| health-first UDS bind | `37,120 µs` | `< 1,000,000 µs` |
| 100-request warm `/health` p95 | `235 µs` | `< 10,000 µs` |

These are local single-run measurements on the named host, not universal latency claims.

## Local gate results

All commands ran locally. The final task-4.1 matrix at implementation commit `364fc5c` was:

| Workspace | Format | Check | Clippy `-D warnings` | Tests |
|---|---|---|---|---:|
| `substrate/exec-contracts` | PASS | PASS | PASS | 7 |
| `substrate/exec-core` | PASS | PASS | PASS | 17 |
| `substrate/exec-tier-p` | PASS | PASS | PASS | 23 |
| `substrate/exec-service` | PASS | PASS | PASS | 23 |
| `crates/prometheus-exec` | PASS | PASS | PASS | 7 |
| **Total** | | | | **77** |

The 23 Tier P tests include 11 real macOS Seatbelt executions proving Python, Node, and Bash success; denied external reads and writes; denied loopback networking; environment filtering; process-group timeout; stream/output bounds; artifact bounds; symlink escape rejection; and unsupported/privileged requests rejected before spawn.

Additional PASS gates:

- deterministic generated OpenAPI/schema byte diff;
- strict OpenSpec validation;
- dependency-direction enforcement;
- SIGKILL/stale-socket restart and exact terminal retrieval;
- signed privileged request becoming durable `grant-pending`, `notSpawned`, and non-terminal across restart;
- corrupted grant metadata causing a non-mutating nonzero doctor result;
- no orphan `prometheus-exec daemon` process after fixtures.

## Platform dispositions

| Platform | Evidence achieved in change 002 | Honest status |
|---|---|---|
| macOS x86_64 | Release-binary real use case, signed receipt and artifact verification, 11 real Seatbelt tests, UDS/doctor/startup/restart evidence | **locally runtime-certified for Tier P on this host** |
| Linux x86_64-musl | Tier P, service, and CLI warnings-denied cross-Clippy; nine portable bwrap/Landlock plan fixtures | **source/cross-build/fixture-certified only; runtime pending** |
| Windows | No process sandbox adapter or runtime execution; Windows Tier P is an explicit v1 non-goal | **Tier P unavailable by design; no runtime or cross-build claim** |

### Linux details

The local Linux evidence proves that the Linux-specific source compiles for `x86_64-unknown-linux-musl`, bwrap plans are deterministic, capabilities cannot broaden the plan, network isolation is explicit, writable output layering follows the read-only run root, escaped layouts are rejected, and partial/even fully reported Landlock enforcement remains unavailable until runtime certification.

It does not prove that bubblewrap or Landlock executed on a Linux kernel. The current sidecar therefore reports its sandbox subsystem as not runtime-certified on non-macOS hosts and does not start a native runner. Linux FR-01 and FR-11 runtime acceptance remain pending and must not be inferred from cross-Clippy or portable plan tests.

The exact local cross command used Clang as the musl linker driver:

```bash
CC_x86_64_unknown_linux_musl=/usr/bin/clang \
CFLAGS_x86_64_unknown_linux_musl=--target=x86_64-unknown-linux-musl \
cargo clippy --target x86_64-unknown-linux-musl --all-targets -- -D warnings
```

This passed independently for `exec-tier-p`, `exec-service`, and `prometheus-exec`.

### Windows details

Change 002 has no Windows Tier P backend. There is no AppContainer/restricted-token adapter, and an unsupported platform must not execute a native process or emit an attested Tier P receipt. Tier W on Windows belongs to change 003; AppContainer Tier P remains deferred under PAGS-SPEC-EXEC-001 decision D-01. No Windows runtime, compilation, or parity claim is made here.

## Archived artifact hashes

| Artifact | SHA-256 |
|---|---|
| `doctor-restart.redacted.json` | `112697c3f23582bf339d0219f5c5e549240efb44b12b9328fade51cc6dd922d6` |
| `incident-batch.json` | `c0297e78d7c4d80b520a2a81af6f9da84a2f3ecc696b36e5ac5c27bc0f2b72aa` |
| `incident-risk.py` | `5653f1c4aeca3871e10eb903ed9000272c9a922a25679d118167d7d14e96b6bc` |
| `outputs/risk-summary.json` | `99b31f41b94ec6b079fbf7687a949c851fb55cbe47029d37b7f2cb3be3c13e56` |
| `public-identity.json` | `5ffe20c1e31ae6a35fe82be5c88fce21ac95b90d286f767511c2bbbd6bd7a9ee` |
| `receipt.json` | `8555667efa1a6599643e49ac299c146c6dc1d8fec49b164751faef94d7adf374` |
| `request.json` | `bb11d5f4c701c4701b556483b37cf9ba16f05412c44ab0b1f27e8759d8e33410` |
| `run.json` | `6d67210e7c8de3b24e5b2aaaa8fc7f15113050fc26e22c2dcff4c70d426467ff` |
| `verify-wrong-root.json` | `52a27001d1ada2e0cc5afa1da40f50d95ecea4c1643102e8349fd20f162ac5f1` |
| `verify.json` | `503d2594c77926ff4abdc3d8d71fbf948568ee59ca0d97b8f1cc2c244270237f` |

The archive is public-data-only. Searches for `privateKey`, hardware serials, hardware UUIDs, provisioning identifiers, and activation-lock fields return no matches.

## External evidence disposition

GitHub Actions supplied no product validation. The implementation branch was not pushed by this task. Installed-service certification, Linux runtime execution, Windows Tier W, mobile devices, MCP, remote peers, Docusaurus, and all-tool plugin distribution remain assigned to later changes in the phase.
