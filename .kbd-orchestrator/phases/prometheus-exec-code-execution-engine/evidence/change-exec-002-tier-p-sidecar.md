# Evidence: change-exec-002-tier-p-sidecar

Date: 2026-08-04

Certified implementation commit: `1b8d905f09fb233aec7eccbf1b3c0de8e032479d`

Original public evidence archive commit: `74f5099`

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
| run ID | `00d22297-e75b-45d5-aefd-464241e27f04` |
| request ID | `d2576f49-aa03-4e92-85bf-a1157fcc56f5` |
| request hash | `sha256:07ba4c36a356cd6a1b50ebee3d056f8c87155b41a370142003aecd3dda841cc5` |
| state | `succeeded` |
| evidence class | `attested` |
| tier / backend | `p` / `seatbelt` |
| exit status | `0` |
| wall clock / peak RSS | `70 ms` / `4 MiB` |
| sandbox profile | `sha256:14819ffbbcfd674dda50f1df407cba83099c661ed41c24817fd2ff54b1a25019` |
| Python toolchain | `sha256:179301dcb41ea78accc3fa0048a7e6f6710d891945a751a34addd622020c1818` |
| receipt hash | `sha256:95810a5221bd432a4f3041b45e8eb065efb735dfc227be66482057c652be83cd` |
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
| release binary SHA-256 | `a77a8f4a861092ac919125f77d1707eefe071d5984e2fa58cdd289978511841c` |
| binary format | `Mach-O 64-bit executable x86_64` |
| signing state | unsigned local build; not installation evidence |
| private identity mode | `0600` |
| UDS mode | `0600` |
| executing device platform | `macos-x86_64` |

The private identity is intentionally absent from the archive. Only the Ed25519 public key and key ID required for independent receipt verification are checked in.

## Restart and doctor evidence

The daemon was terminated with SIGKILL, leaving its socket path behind. A second release-binary daemon invocation recovered the stale socket, reopened the same identity, ledger, receipt log, and CAS, and returned a status response byte-identical to the pre-kill terminal status (`SHA-256 de765e16c37828356528fe3fb4b21d2d1f52e435e5bb41c92b2f1c672a6ef467`).

The restarted doctor reported `healthy: true` with all required checks passing:

- binary identity;
- receipt identity;
- mode-0600 Unix socket;
- same-UID peer `/health`;
- readiness;
- macOS Seatbelt backend;
- two structurally valid records and zero in-flight runs;
- five verified content-addressed blobs.

The focused startup fixture measured:

| Metric | Result | Requirement |
|---|---:|---:|
| health-first UDS bind | `37,120 µs` | `< 1,000,000 µs` |
| 100-request warm `/health` p95 | `235 µs` | `< 10,000 µs` |

These are local single-run measurements on the named host, not universal latency claims.

## Local gate results

All commands ran locally. The complete task-4.1 matrix was rerun after the final audit and independent-review remediation at implementation commit `1b8d905`:

| Workspace | Format | Check | Clippy `-D warnings` | Tests |
|---|---|---|---|---:|
| `substrate/exec-contracts` | PASS | PASS | PASS | 8 |
| `substrate/exec-core` | PASS | PASS | PASS | 19 |
| `substrate/exec-tier-p` | PASS | PASS | PASS | 28 |
| `substrate/exec-service` | PASS | PASS | PASS | 23 |
| `crates/prometheus-exec` | PASS | PASS | PASS | 8 |
| **Total** | | | | **86** |

The 28 Tier P tests include 14 real macOS Seatbelt executions proving Python, Node, and Bash success; denied external reads and writes; denied loopback networking; environment filtering; process-group timeout; memory termination; inherited stack limits; observed CPU/RSS accounting; stream/output bounds; artifact bounds; symlink escape rejection; and unsupported/privileged requests rejected before spawn.

Additional PASS gates:

- deterministic generated OpenAPI/schema byte diff;
- strict OpenSpec validation;
- dependency-direction enforcement;
- SIGKILL/stale-socket restart and exact terminal retrieval;
- signed privileged request becoming durable `grant-pending`, `notSpawned`, and non-terminal across restart;
- corrupted grant metadata causing a non-mutating nonzero doctor result;
- no orphan `prometheus-exec daemon` process after fixtures.

## Task-4.3 audit remediation and fresh release-binary proof

The final requirement audit found two implementation gaps that the earlier green matrix did not exercise:

1. CAS pins and GC existed as primitives, but the daemon did not retain queued request material or receipt-referenced evidence and did not invoke budget GC.
2. Seatbelt enforced wall-clock and output limits, but did not enforce `memoryMb`/`stackKb` or populate observed CPU/RSS usage.

Commit `a08ee42` closed both initial gaps. The independent cross-model review then found transactional ownership failures around CLI uploads, request replay/conflict paths, receipt-publication rollback, and restart reconciliation. Commit `1b8d905` makes upload-to-request ownership transfer atomic under the CAS operation lock, scopes request pins by canonical request hash, preserves grant-pending references, retains receipt evidence before terminal publication, rolls back failed publication, attempts every cleanup even after an error, and preserves request ownership for malformed terminal records without receipts. The Seatbelt runner applies the requested stack ceiling before interpreter startup, samples the exact process group every 10 ms, terminates it on memory breach, fails closed on monitor failure, and records observed CPU and peak RSS.

A fresh optimized binary (`sha256:a77a8f4a861092ac919125f77d1707eefe071d5984e2fa58cdd289978511841c`) built from `1b8d905` repeated the incident-risk use case with a 1 MiB CAS budget:

| Field | Observed value |
|---|---|
| run ID | `00d22297-e75b-45d5-aefd-464241e27f04` |
| request hash | `sha256:07ba4c36a356cd6a1b50ebee3d056f8c87155b41a370142003aecd3dda841cc5` |
| state | `succeeded` |
| output artifact | `sha256:99b31f41b94ec6b079fbf7687a949c851fb55cbe47029d37b7f2cb3be3c13e56`, 50 bytes |
| wall clock / peak RSS | `70 ms` / `4 MiB` |
| receipt hash | `sha256:95810a5221bd432a4f3041b45e8eb065efb735dfc227be66482057c652be83cd` |

Offline verification passed receipt invariants, Ed25519 signature/key identity, exact request hash, and exact artifact hash. After SIGKILL recovery, a fresh non-mutating doctor reported `healthy: true`, all eight required checks passed, five CAS blobs verified, two structurally valid records, and zero in-flight runs. CAS inspection showed receipt-scoped pins for the code, input, stdout, stderr, and output artifact, with no stale upload or request pins. The temporary private identity and runtime tree were destroyed after verification; no private key was archived.

The final adversarial-review gate used producer `gpt-5.6-sol` and distinct judge `gpt-5.4`. It iterated on real lifecycle findings until `findings-final-pass.json` returned `PASS` with zero findings; the strict anti-sycophancy gate also passed with score `0.0`. A protected-test Git comparison from `a08ee42` to `1b8d905` reported zero protected changes.

## Platform dispositions

| Platform | Evidence achieved in change 002 | Honest status |
|---|---|---|
| macOS x86_64 | Release-binary real use case, signed receipt and artifact verification, 14 real Seatbelt tests, UDS/doctor/startup/restart evidence | **locally runtime-certified for Tier P on this host** |
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
| `doctor-restart.redacted.json` | `4d0e8728f6888dc427b8e3a7acc5cdf076148d64962588e0ccba4c06ef431809` |
| `incident-batch.json` | `c0297e78d7c4d80b520a2a81af6f9da84a2f3ecc696b36e5ac5c27bc0f2b72aa` |
| `incident-risk.py` | `5653f1c4aeca3871e10eb903ed9000272c9a922a25679d118167d7d14e96b6bc` |
| `outputs/risk-summary.json` | `99b31f41b94ec6b079fbf7687a949c851fb55cbe47029d37b7f2cb3be3c13e56` |
| `public-identity.json` | `0dc5dabb50e005bb5c7e359f0086622e79eb59329406c4d75fdbbd108b028e30` |
| `receipt.json` | `df5856f5835abd48b3a829b082cb386eca6b02747fb3d92bac1d55d7269c6529` |
| `request.json` | `77f531b4e644d9a295b97dc2643a24cc9296bae62cb2b4ed8982112fda206bfd` |
| `run.json` | `1a60dc3e85a1104e13fe49e743a66799836fb45ad0c1eefe94e52978b2f53fa4` |
| `verify-wrong-root.json` | `41d0ad3586eb58e3138a1b78621e65729675aa97ce6e321ccabfd64299a2856a` |
| `verify.json` | `baaf05defccb35f0fdcfa0cba4f1b6bfe6c2e1f3512daf6afa5cd469744a89d3` |

The archive is public-data-only. Searches for `privateKey`, hardware serials, hardware UUIDs, provisioning identifiers, and activation-lock fields return no matches.

## External evidence disposition

GitHub Actions supplied no product validation. The implementation branch was not pushed by this task. Installed-service certification, Linux runtime execution, Windows Tier W, mobile devices, MCP, remote peers, Docusaurus, and all-tool plugin distribution remain assigned to later changes in the phase.
