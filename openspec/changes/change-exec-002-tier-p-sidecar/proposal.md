## Why

Signed receipt contracts are useful only when a runtime can enforce the limits and sandbox profile they describe. Prometheus needs a macOS/Linux native-process path for Python, Node, and Bash that fails closed when no supported sandbox exists, plus a restart-safe local service that never becomes a dependency for ordinary agent work.

## What Changes

- Add a transport-independent execution kernel, content-addressed artifact store, receipt writer, idempotency ledger, and policy/grant enforcement point.
- Add Tier P adapters for macOS Seatbelt and Linux bwrap with Landlock capability reporting; unsupported platforms return `tier_unavailable` instead of executing unsandboxed.
- Add Cedar-backed auto-approval for reversible, network-free, output-scoped runs and SSH/interactive grant records for privileged capabilities.
- Add a UDS sidecar with atomic mode-0600 socket creation, same-user peer enforcement, static `/health`, bounded `/ready`, run/status/event/receipt/artifact APIs, and graceful restart recovery.
- Extend `prometheus-exec` with `daemon`, `run`, `status`, and `doctor` commands while preserving offline `init` and `verify` behavior.

## Capabilities

### New Capabilities

- `native-execution-sandbox`: Process execution under an explicitly identified OS sandbox with resource and output limits and no unsandboxed attestation fallback.
- `execution-policy-grants`: Cedar policy decisions, tighten-only operator policy, and SSH/interactive grants recorded in receipts.
- `execution-artifact-cas`: Atomic content-addressed stdout, stderr, and artifact storage with pin-aware budget enforcement.
- `execution-sidecar-service`: Restart-safe idempotent run orchestration, UDS authentication, health/readiness, REST/SSE retrieval, and local doctor behavior.

### Modified Capabilities

None.

## Impact

- Adds `substrate/exec-core`, `substrate/exec-tier-p`, and `substrate/exec-service`.
- Expands `crates/prometheus-exec` into the macOS/Linux local execution sidecar.
- Adds local installer/doctor surfaces in later integration tasks; no GitHub product-test workflow is introduced.
- Native execution remains optional evidence production and does not police an agent's direct shell or Python use.
