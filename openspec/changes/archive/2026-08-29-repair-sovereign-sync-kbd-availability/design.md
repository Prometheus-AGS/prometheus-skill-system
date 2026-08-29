## Context

See `proposal.md` for motivation. The platform-managed daemon already runs under launchd with `RunAtLoad` and `KeepAlive`, but the CLI assumed a legacy TCP address while the daemon exposed a private Unix socket. The daemon also treated any registered-project open failure as fatal to all stateful KBD routes, and local runtime fallback selected a different signer from the managed service. Three stale registry paths reproduce the partial-open condition. KBD child orchestration additionally has two compatibility bugs around runtime labels and fully-qualified child IDs.

## Goals / Non-Goals

**Goals:**

- Use one discoverable local control transport across the daemon, CLI, and doctor paths.
- Preserve least-privilege local access and explicit operator endpoint overrides.
- Keep healthy project authorities available during partial registry degradation.
- Reuse the enrolled managed signer only for canonical platform state.
- Make child creation and stage gating work with canonical fully-qualified child IDs.
- Refresh generated distributions deterministically and certify the installed service across restarts.

**Non-Goals:**

- Deleting stale registry entries without an explicit operator decision.
- Rewriting historical compatibility projections whose counters exceed canonical state.
- Adding another service supervisor or changing the verified launchd policy.
- Changing the public TCP protocol or exposing the private socket beyond the local user.

## Decisions

### Prefer the managed Unix socket with an explicit TCP escape hatch

The CLI control client uses an explicit `PROMETHEUS_CONTROL_ENDPOINT` first, then an existing platform-managed Unix socket, then the legacy TCP endpoint for compatibility. Unix HTTP/1 requests run over a Tokio Unix stream using Hyper's client connection primitives so existing REST payloads and status handling stay unchanged.

This was chosen over restoring a default TCP listener because the daemon's private socket is the deployed contract and avoids an unnecessary loopback port. A Unix-only client was rejected because operators and tests still need a deliberate TCP override.

### Isolate failed project authorities instead of gating the router

Startup opens every registered project independently, records concrete failures, and installs the KBD router whenever at least one authority is healthy. Failed project IDs remain unavailable and visible in local diagnostics; no route is synthesized for them.

This was chosen over failing the entire KBD surface because registry entries are independent security authorities. Automatically pruning failures was rejected because a missing worktree can be temporary and registry removal is an operator-owned destructive decision.

### Scope managed signer discovery to the canonical platform root

The runtime discovers the managed mode-0600 device-key file only when using the default platform canonical data root. Custom and test roots retain their existing isolated signer behavior.

This converges service and interactive identities without copying key bytes into project state. Unconditionally loading the managed signer was rejected because it would break hermetic tests and could authorize mutations in unrelated custom roots.

### Normalize child identity at the orchestration boundary

Runtime child creation assigns its human label before any activation code uses it. The stage gate reads `progress.json.phaseId` when present, which supplies the fully-qualified canonical child ID while preserving basename compatibility for legacy artifacts.

This keeps compatibility artifacts descriptive while treating the append-only runtime as authoritative. Rewriting canonical child IDs to basenames was rejected because it would discard parent-child identity.

### Treat generation and installed restart checks as certification gates

Source edits are propagated through the repository's Codex plugin generator, run twice to prove idempotency, then validated locally. Release binaries are installed and ad-hoc signed before two supervised restarts, health probes, and signed mutations are recorded.

This separates source correctness from distribution and installed-runtime correctness. Hosted CI is excluded by repository policy and is not used as evidence.

## Risks / Trade-offs

- [A stale socket file can exist without a listening daemon] → Classify connection failures explicitly and fall back only according to the documented endpoint-selection order.
- [Partial startup can conceal accumulating registry debt] → Log every failed project with its concrete error and retain the entries for explicit cleanup.
- [Sharing the managed signer broadens local command authority] → Restrict discovery to canonical platform state and preserve filesystem permissions; never log key material.
- [Generated distribution drift can reintroduce old shell behavior] → Run the generator twice, require byte-identical output, and validate the Codex distribution locally.
- [Historical KBD projection drift remains unresolved] → Preserve the migration backup and return ownership to the parent `openspec-mirror-drift-cleanup` phase.

## Migration Plan

1. Build and locally test the runtime, daemon, CLI transport, and shell orchestration changes.
2. Regenerate Codex plugin distributions twice and confirm the second run produces no diff.
3. Install release binaries, apply ad-hoc signatures, and restart the launchd service twice.
4. Verify Unix health and signed KBD mutations after restart while stale registrations remain isolated.
5. Roll back by reinstalling the prior binaries and source-generated distribution; no state schema or registry mutation is required.
