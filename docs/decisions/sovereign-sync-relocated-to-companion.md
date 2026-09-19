# Decision: retire the four sovereign-sync OpenSpec main specs

**Status:** accepted · 2026-09-04 · `change-cpc-009-pack-removal`
**Phase:** control-plane-to-companion

## The problem

Four OpenSpec main specs under `openspec/specs/` describe the sovereign-sync
daemon's behavior: `sovereign-sync-daemon-health` (5 requirements — local
status command, health classification), `sovereign-sync-ci` (2 requirements —
CI workflow expectations for the daemon crate), `iroh-docs-adapter` (3
requirements — the `storage-provider` P2P document adapter), and
`mcp-client-pool` (3 requirements — pooled stdio MCP client behavior used by
the daemon's tooling).

`change-cpc-004-relocate-sovereign-sync` moved the crate itself
(`sovereign-sync`, `sovereign-client`, `kbd-mobile`, and the `iroh_docs.rs`
adapter) into `prometheus-companion`. These four specs kept describing code
that no longer lives in this repository — a spec describing an absent
subsystem is not documentation, it is a claim the codebase can no longer
support.

## The decision

Retire all four specs from the pack: delete
`openspec/specs/{sovereign-sync-daemon-health,sovereign-sync-ci,iroh-docs-adapter,mcp-client-pool}/`.
`openspec archive` operates on completed *changes*, folding their deltas into
main specs — there is no OpenSpec verb for retiring a main spec directly, so
this is a plain removal, recorded here rather than left undocumented.

The requirements these specs described are not gone from the world — they now
live in `prometheus-companion`:

| Retired pack spec | Companion equivalent |
|---|---|
| `sovereign-sync-daemon-health` | `crates/sovereign-sync/src/main.rs` (`--mode status`), `crates/prometheus-substrate/src/health.rs` (change-cpc-007) |
| `sovereign-sync-ci` | The Companion's own CI/local-gate discipline (`bash scripts/audit-all.sh`) |
| `iroh-docs-adapter` | `crates/storage-provider/src/iroh_docs.rs` (relocated by change-cpc-004) |
| `mcp-client-pool` | `crates/sovereign-sync/src/mcp_client_pool.rs` (relocated by change-cpc-004) |

The Companion does not maintain these as OpenSpec main specs — it has its own
verification discipline (`verification.md` per change, integration-only
evidence). This decision record is the pointer for anyone who searches the
pack's spec history and finds these four gone.

## What is NOT affected

Nothing else in `openspec/specs/` changes. `docs/integration-contract.md` (the
open, versioned contract seam between the pack and any extension) is
unaffected — it never depended on these four specs and does not name them.

The three contract-v1 identifiers the pack keeps by design — the socket
filename `sovereign-sync.sock`, the environment variable
`SOVEREIGN_SYNC_SOCKET`, and the keychain item name
`sovereign-sync/device-key.json` — are unrelated to these specs; they are the
pack CLI's own discovery mechanism for an optional, absent-by-default control
endpoint (seam 1), and stay wherever the CLI's own code already uses them.
