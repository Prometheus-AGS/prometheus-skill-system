# Evidence: change-exec-001-contracts-verification

Date: 2026-08-04

Certified base: `fa7cae63b114a43283e672b3006f1a3a6a81acd2`

Environment: macOS arm64, local execution only

## Scope

This evidence covers the transport-free `prometheus-exec-contracts` crate, the side-effect-bounded `prometheus-exec init|verify|contracts` CLI slice, deterministic JSON Schema/OpenAPI components, and immutable receipt-segment verification. It does not claim that Tier W, Tier P, a daemon, REST, MCP, FFI, or remote execution exists yet.

## Local results

| Gate | Result |
|---|---|
| `cargo fmt --all -- --check` (`substrate/exec-contracts`) | PASS |
| `cargo clippy --all-targets -- -D warnings` (`substrate/exec-contracts`) | PASS |
| `cargo test --all-targets` (`substrate/exec-contracts`) | PASS — 7 contract/property tests |
| `cargo fmt --all -- --check` (`crates/prometheus-exec`) | PASS |
| `cargo clippy --all-targets -- -D warnings` (`crates/prometheus-exec`) | PASS |
| `cargo test --all-targets` (`crates/prometheus-exec`) | PASS — 3 CLI isolation/identity/version tests |
| deterministic regeneration and byte diff | PASS |
| `openspec validate change-exec-001-contracts-verification --strict` | PASS |
| normal dependency scan for KBD, Sovereign Sync, async/network clients, or servers | PASS — none present |
| `prometheus-exec --version` | PASS — `prometheus-exec 1.7.0` |

The offline boundary fixture runs `verify` with isolated `HOME`, `XDG_STATE_HOME`, and `XDG_RUNTIME_DIR`, then proves no state/runtime path was created. The binary's normal dependency tree contains no KBD, Sovereign Sync, `reqwest`, `hyper`, `tokio`, `axum`, or `rmcp` dependency. This is source/binary evidence that the revision-1 verifier has no service or network path; it is not a claim about later daemon modes.

## Artifact identities

| Artifact | SHA-256 |
|---|---|
| `docs/reference/api/prometheus-exec.openapi.json` | `e308b11a6290843b10ab04bb6c0608109a86e081fcc47f469e3b9f5de56aec8e` |
| `docs/reference/api/prometheus-exec.schemas.json` | `38b02287b76695f0101cfd9262dab0e55eee8c7de26b300a5d2b3fcea3631736` |
| local debug `prometheus-exec` | `8164bcb9040d5ada608b1a72a79227a3c3dbe3c5f60678f4f58d0e2a40d8012c` |

The binary hash is local build evidence only. Release installation and signing occur after the runtime changes are integrated.

## External evidence disposition

No GitHub workflow output was used. Linux, Windows, iOS, Android, physical-device, Wasmtime, native-sandbox, remote-peer, and installed-service evidence remain pending because those surfaces are outside this contract-only change.
