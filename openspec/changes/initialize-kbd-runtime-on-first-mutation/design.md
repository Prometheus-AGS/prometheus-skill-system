## Context

See `proposal.md` for motivation. Registration creates identity and replica bookkeeping, while the first signed run event remains a separate operator-authority boundary. The CLI already has a legacy-aware `ensure_runtime` helper, but it propagates `NotInitialized` before reaching initialization and is called only from migration. Read paths and mutation paths currently share a replay helper even though only mutations should be allowed to create canonical state.

## Goals / Non-Goals

**Goals:**

- Make automatic initialization an explicit mutation precondition.
- Reuse the existing signed runtime initializer and legacy-state mapping.
- Keep read-only status deterministic and non-mutating.
- Prove process exit behavior through the compiled CLI boundary.

**Non-Goals:**

- Initializing during registration or status.
- Adding Unix-socket HTTP transport.
- Changing event, journal, registry, migration, or projection schemas.
- Repairing projection content that canonical initialization already rejects as ambiguous or unsafe.

## Decisions

### Separate read replay from mutation initialization

Keep `state_or_replay` for read-only commands and add a mutation-specific helper that handles `RuntimeError::NotInitialized` by calling the corrected initializer. This avoids surprising writes from `status`, `conflicts`, `claims`, or other inspection surfaces. The rejected alternative was initializing inside the shared replay helper, which is smaller but makes nominal reads create signed history.

### Repair and reuse the legacy-aware initializer

Change `ensure_runtime` to treat both revision-zero state and `NotInitialized` as an empty runtime, then call `initialize_from_legacy`. Before applying the requested mutation, inventory and import any legacy phase ledgers through the existing backup-and-guard migration path. Refresh the local state variable from the committed import before comparing projections, so the safety guard compares legacy completion against the canonical state that now contains it. The alternative was a new `kbd init` subcommand, which would leave existing skills blocked until operators discover and run a separate remedy.

### Test at both helper and process boundaries

Use focused Rust tests for initialization preservation/idempotency and a CLI integration fixture for the exact registered-empty-runtime workflow. Invoke the compiled binary to assert the status text, first mutation success, and non-zero rejection status. A unit-only test would not cover the issue's reported shell exit behavior.

### Keep installation scoped to the CLI

Only `prometheus` changes. `sovereign-sync` remains healthy and unchanged, so deployment replaces and ad-hoc signs the CLI binary without restarting the daemon.

## Risks / Trade-offs

- [Legacy waypoint data is incomplete or contradictory] → Reuse the existing inventory, backup, read-only classification, and runtime validation; fail with the canonical runtime path instead of guessing.
- [A mutation races with another initializer] → Rely on the runtime's existing exclusive lock and already-initialized validation; integration tests cover one boundary, while existing runtime concurrency tests retain authority.
- [Process tests touch platform credential storage] → Provide a mode-0600 fixture device-key file and isolated `PROMETHEUS_DATA_DIR` so tests do not depend on or modify the host keychain.
- [The daemon is unreachable by design] → Force an unused loopback endpoint in tests and assert the canonical local fallback, matching the managed LaunchAgent configuration.

## Migration Plan

Build and install the updated CLI from the reviewed commit. Existing initialized runtimes are unchanged. Empty registered runtimes initialize only on their next typed mutation; rollback restores the previous CLI binary and leaves any successfully created signed initialization event intact.
