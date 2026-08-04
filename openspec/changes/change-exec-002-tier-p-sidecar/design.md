## Context

Change 001 fixed the signed request/receipt and receipt-segment boundary. This change must produce honest Tier P receipts on macOS and Linux, keep service availability separate from agent tool freedom, and survive response loss and daemon restarts. `/usr/bin/sandbox-exec` is present on the certification Mac; Linux adapters can be compiled and fixture-tested here but require Linux runtime evidence.

## Goals / Non-Goals

**Goals:**

- Execute one-shot Python, Node, and Bash jobs only through a supported process-scoped sandbox.
- Externalize inputs/outputs through a work directory and content-addressed store.
- Sign exactly one terminal attested receipt per accepted request and return it on replay.
- Bind health before durable subsystem initialization and expose readiness independently.
- Reuse Cedar and SSH allowed-signers roots without creating a new authorization root.

**Non-Goals:**

- Windows Tier P, sessions, containers/VMs, remote execution, or Wasm execution.
- Intercepting or restricting ordinary agent commands.
- Claiming Linux runtime certification from macOS fixtures.

## Decisions

### Ports and adapters

`exec-core` owns `ExecutionPort`, run orchestration, CAS, policy outcomes, and receipt assembly. `exec-tier-p` implements the port. `exec-service` owns idempotency, events, and transport-independent service methods. This keeps the service testable without spawning a sandbox and prevents REST/MCP from diverging.

### Fail-closed Tier P selection

macOS invokes `/usr/bin/sandbox-exec` with a generated profile whose hash is recorded. Linux selects bwrap when present; a Landlock adapter may report partial enforcement but cannot silently broaden capabilities. Unsupported/missing backends reject before process spawn. There is no direct `Command` fallback.

### Workdir and CAS

Each run gets a private temporary root with read-only materialized inputs and one writable `outputs/`. Stdout, stderr, and discovered output files are bounded, hashed while persisted, and installed atomically below `<cas>/sha256/<prefix>/<digest>`. Receipt/log references pin blobs; GC removes only unpinned blobs and never follows symlinks.

### Policy and grants

The PEP classifies a request as automatic only when it has no network egress, no environment passthrough, and no writable path outside `outputs/`. Cedar evaluates the normalized context and operators can add denies but cannot add a permit that bypasses the hard ceiling. Escalated execution requires a validated `prometheus-exec-grant` SSH manifest or trusted-host interactive grant; its hash is recorded.

### Idempotent service state

Before execution, the service atomically persists `(requestId, requestHash, state)`. Same ID/same hash returns current/original state; same ID/different hash is 409. Terminal receipts and event sequence are durably committed before a completion response. Startup reconciles interrupted non-terminal records to a signed interrupted receipt or safe requeue only when no process was spawned.

### UDS health-first transport

The daemon atomically binds a `0600` UDS and starts a static `/health` route before loading keys, CAS, policy, or state. `/ready` reports each subsystem. Peer credentials must match the daemon UID. TCP is absent in this change. The transport delegates every operation to `exec-service`.

## Risks / Trade-offs

- **[Seatbelt is undocumented]** macOS profiles can drift. → Record exact profile hash, run escape fixtures, and report backend/version in receipts.
- **[Linux unavailable locally]** Compilation is not runtime evidence. → Keep Linux status pending and provide disposable Linux certification commands.
- **[Process tree escape]** Children could survive the parent. → Put the child in an isolated process group, enforce wall timeout, and terminate the group.
- **[Response loss]** A caller may retry after successful execution. → Commit terminal receipt before responding and key idempotency by signed request hash.
- **[Health coupling]** Slow storage/policy initialization can make health timeout. → Static health state is independent; readiness is a bounded state machine.

## Migration Plan

This is additive. Install the sidecar only after local macOS certification. Rollback unloads the LaunchAgent and restores the prior binary; receipt/CAS data is retained and remains verifiable by change-001 tooling.

## Open Questions

- Landlock becomes an executing fallback only after Linux fixtures prove the requested capability set is fully enforced; until then it reports unavailable/partial.
