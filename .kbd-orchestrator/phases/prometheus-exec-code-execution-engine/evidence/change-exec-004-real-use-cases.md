# Change 004 real-use-case certification

## Scope

This packet certifies committed product revision `3f2bff5` with release binary
`prometheus-exec 1.7.0` (`sha256:b693cd8df48a6b8634bba9c88b9b7da5a871c5ad919eebf351f89719dc59127b`).
All execution occurred locally on the release host. The installed KBD service,
the KBD skill/wrapper, Sovereign Sync, and GitHub Actions were not invoked.

The signed plugin generation was created in an isolated temporary home and had
generation ID
`452142507447142fb15ad942f3e99538b32535fd909115dc9050241c4ea0de52`.
Its signing key and private execution identities were automatically removed;
the archive contains only public verification material.

## Real MCP and Tier P

The release binary served RMCP 1.8 over stdio and negotiated MCP protocol
`2025-06-18`. The certifier enumerated and exercised all six tools:
`exec-run`, `exec-status`, `exec-events`, `exec-receipt`, `exec-artifact`, and
`exec-verify`.

A real Python use case calculated `6 * 7`, wrote a JSON artifact through the
sandbox-owned output directory, and completed under macOS Seatbelt with an
attested signed receipt. Event resume used an exclusive sequence cursor, the
artifact was returned inside the bounded MCP envelope, and public-key-only MCP
verification returned valid. The redacted protocol/result archive is
`change-exec-004-real-use-cases/mcp-transcript.redacted.json`.

## Signed Tier W, restart, and offline verification

The exact checked reference component
`sha256:ba438895404a23985d5226735b8f362cf3e8044894a1140852ba0992f2fdbe78`
executed through the release daemon under the temporary signed generation.
The terminal receipt reports Cranelift and Wasmtime `46.0.2`.

The daemon was killed with `SIGKILL`; a stale socket was observed; a new daemon
recovered the same state; and status before and after restart was byte-equivalent
as parsed JSON. Doctor diagnosis then passed with the four required exclusions.
The signed receipt and request verified with only the public identity and exact
component. A self-contained copied bundle then passed `verify-bundle` without
daemon or network state. See
`change-exec-004-real-use-cases/tier-w-restart-offline.redacted.json` and
`change-exec-004-real-use-cases/portable-bundle/`.

## Disposable remote peers

The transport-gated disposable-peer suite ran locally:

```text
cargo test --manifest-path substrate/exec-remote/Cargo.toml \
  --features transport --test disposable_peers -- --nocapture
```

All three scenarios passed:

- response loss resumes after offline state and restart without re-execution;
- unknown endpoints, signer mismatch, replay, and expiry fail closed;
- a slow transport is isolated from an independent dispatch.

The redacted output and discovered scenario list are archived beside this
packet. These tests use injected disposable transports. They do not certify a
production Sovereign/P2P adapter or an externally deployed estate.

## Evidence dispositions

| Evidence dimension | Disposition | Basis |
|---|---|---|
| Release artifact | certified | Exact version/hash and local task 6.1 gates |
| Real local MCP/Tier P | certified | Live RMCP tool calls, sandbox run, artifact, events, receipt, verification |
| Real local signed Tier W | certified | Signed generation, component execution, restart, doctor, offline replay |
| Disposable remote runtime | certified | Three transport-feature peer scenarios |
| Installed service | not evaluated here | Installation and loaded-state readback are task 6.3 |
| Production remote adapter | `pending_evidence` | No Sovereign/P2P adapter was invoked |
| Physical iOS/Android runtime | `pending_evidence` | No physical device was attached |
| GitHub workflow output | not evidence | Hosted automation remains docs sync/Pages only |

Warnings: none. Required checks: green. External-only dimensions retain their
explicit `pending_evidence` status and are not promoted by artifact results.
