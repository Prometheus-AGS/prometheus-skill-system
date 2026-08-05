## Context

Changes 001–003 established canonical signed contracts, a durable transport-independent service, REST/UDS, Tier P and Tier W execution, offline replay, embedded/FFI surfaces, and signed-generation component authorization. The missing release boundary is deliberately cross-cutting: MCP must reuse the service instead of cloning behavior; remote R-class delivery must add enrollment and queue semantics without infecting local crates; certification must cite evidence honestly; and installation, plugin distribution, OpenAPI, and Docusaurus must describe and verify the product actually shipped.

This work runs locally on macOS. It may execute disposable isolated peer fixtures, but it must not invoke the installed KBD or Sovereign Sync services. GitHub workflows remain documentation synchronization and Pages deployment only. Linux runtime, Windows Tier P, remote deployment, the failed mobile size gate, and physical-device runtime cannot be promoted without their named evidence.

## Goals / Non-Goals

**Goals:**

- Provide MCP run/status/events/receipt/artifact/verify parity through the existing `ExecutionService`, `ArtifactStore`, and portable verifier.
- Provide a durable, signed, estate-only remote dispatch kernel with explicit enrollment snapshots and a transport port that can be adapted to Sovereign Sync without requiring a live service.
- Make response loss, restart, replay, expiry, signer mismatch, partial peer success, and remote unavailability explicit and testable.
- Integrate receipt citations and `pending_evidence` into local certification without making a specific execution tool mandatory.
- Install and diagnose the release binary without false-green output, and publish the exact Tier W component through the signed 14-target plugin generation.
- Add canonical generated contracts and authored Docusaurus/guide/ADR material with one local drift/build gate.

**Non-Goals:**

- Invoking KBD as an orchestrator, memory, doctor, or installed service during implementation or certification.
- Starting, reconfiguring, repairing, or certifying the installed Sovereign Sync service.
- Adding TCP exposure, a new enrollment authority, new pairing secrets, interactive sessions, Windows Tier P, mobile background execution, or a transparency log.
- Making `prometheus-exec` mandatory for ordinary agent Bash/Python/Edit/Write work or accepting receipts that cannot be independently verified.
- Claiming remote multi-peer or mobile runtime evidence when only deterministic isolated fixtures or cross-builds exist.

## Decisions

### 1. MCP is a thin adapter over a reusable local facade

Add an `mcp` module to the binary crate using the repository-pinned `rmcp 1.8` line. `ExecMcpServer` receives an `Arc<LocalExecutionFacade>` assembled from the same durable state directory, `ExecutionService`, artifact store, and verifier used by the daemon and CLI. Tool handlers deserialize typed parameters, call the facade, and serialize bounded result envelopes; they do not open a second ledger, execute kernels directly, or accept signing-key material.

The facade exposes submit, run, events-after, receipt, artifact metadata/bytes, and verify operations. REST remains in `exec-service`; MCP remains in the binary because stdio lifecycle and identity/config loading are deployment concerns. Contract-generation code emits MCP input/output schemas next to OpenAPI and checks them into the docs reference.

Alternative rejected: proxy MCP through the UDS HTTP API. That would preserve behavior but make standalone stdio depend on a running daemon, duplicate HTTP error translation, and violate the three-form design.

### 2. Remote routing is a pure durable kernel plus an injected transport

Create `substrate/exec-remote` with no KBD dependency. It owns canonical `SignedRemoteDispatch`, enrollment bindings, per-target records, immutable hash-linked queue segments, replay/expiry checks, aggregate state, and a `RemoteTransport` async trait. It depends only on contracts plus cryptographic/filesystem primitives. The estate feature in `prometheus-exec` may compose this crate; standalone/mobile profiles cannot select it.

The Sovereign adapter is implemented on the transport side of the boundary, using only signed opaque execution-envelope bytes and endpoint IDs. Product tests use an in-memory/disposable transport and isolated directories, identities, clocks, and process state. This proves protocol behavior without contacting the installed service. Local submission at a target calls the existing execution facade, so remote replay inherits the same request ledger and receipt correctness.

Alternative rejected: depend directly on the existing `sovereign-client`. That crate currently imports KBD runtime types and a live REST client, would violate the narrow execution dependency boundary, and would make deterministic offline fixtures depend on an installed daemon.

### 3. Enrollment is an immutable input, never a fourth trust root

`EnrollmentSnapshot` maps endpoint IDs to Ed25519 public keys and has its own canonical hash. Verification checks dispatch signature, origin binding, target membership, issue time, validity window, dispatch replay, and request replay before durable acceptance. The remote kernel can read a redacted exported snapshot but cannot pair devices, mutate allow-lists, or derive group secrets. Complete tickets and secrets never enter logs or receipts.

Alternative rejected: copy device private keys or introduce an execution-specific pairing authority. It would break the existing trust model and make receipt provenance ambiguous.

### 4. Queue state is receipt-oriented and response-loss safe

Each dispatch is written atomically before send. A target writes accepted/rejected state before returning transport acknowledgement; execution is delegated once to the local request ledger. The ledger's idempotent submit returns a durable local run ID, which the target stores before using a bounded terminal wait. A duplicate `Running` delivery reconciles that exact run ID and never submits again. If the wait times out or returns nonterminal, the target keeps the durable `Running` record for a later delivery and does not construct an invalid terminal response. Returned peer receipts are verified against the enrolled key and stored before aggregate terminal publication. Same dispatch/same hash returns the record, while any hash conflict fails. Expired work becomes a terminal rejection rather than disappearing. Aggregates are derived from per-target records and cannot replace or weaken their receipts.

Alternative rejected: one mutable JSON queue file. It is easy to corrupt across writers and cannot provide independently auditable state transitions or safe response-loss reconciliation.

### 5. Remote readiness is optional and isolated

The sidecar publishes local readiness independently. When estate remote is enabled, `/ready` gains a `remote` subsystem record, but transport initialization runs after health binding and cannot prevent local Tier P/W use or offline verification. Doctor inspects only on-disk configuration/queue invariants unless a separately selected, non-excluded live check exists. KBD and `service:sovereign-sync` exclusions are applied before check construction.

Alternative rejected: make the whole daemon unready whenever a peer is offline. Remote execution is an optional route, not a local availability dependency.

### 6. Certification consumes a small portable evidence index

Add a deterministic execution-evidence index schema containing requirement ID, run ID, receipt/request/artifact paths, hashes, verifier identity, environment classification, and status. A local checker resolves every path, verifies hashes and receipt signatures offline, and emits completed, failed, or `pending_evidence`. Judge availability remains separately represented as `pending_review`. Profiles describe evidence properties, not mandated commands.

Alternative rejected: cite daemon database paths or session transcripts. They are not portable, independently verifiable, or stable after cleanup.

### 7. Release distribution extends existing strict contracts

`prometheus-exec` becomes a managed binary in the root binary manifest and strict installer, with atomic backup/install/sign/readback behavior. Root doctor gets a preselected execution check and refresh evidence but never installs or contacts excluded services during diagnosis. The existing signed-generation builder includes the reference component and capability metadata in its manifest/index; the same activation pointer covers both. Existing target receipt generation remains the authority for the 14 supported tools.

Alternative rejected: install a separate component directory outside the generation. That would split payload and index rollback and break provenance continuity.

### 8. Documentation has generated contracts and authored architecture

Add a top-level Execution section to Docusaurus with overview/use cases, architecture/tiers, local API and MCP tools, remote dispatch, receipts/verification/certification, installation/operations, and platform status. Mermaid diagrams are source-controlled and parsed locally. OpenAPI, MCP schemas, CLI/config reference, platform/evidence tables, component hash, target count, and release metadata are deterministic managed blocks; design rationale and examples remain authored. The numbered guide and crate READMEs link to the canonical site.

Alternative rejected: generate all prose. It would produce mechanically current field lists but weak use-case and architectural guidance.

## Risks / Trade-offs

- [Remote adapter source integration can accidentally pull KBD into local crates] → enforce dependency direction and feature-graph checks for estate-off profiles.
- [MCP tool handlers can drift from REST semantics] → route through one facade and run cross-surface response-loss/event-cursor fixtures plus schema drift checks.
- [Remote response loss can cause duplicate execution] → persist origin and target acceptance before acknowledgement and reuse request-ID idempotency at the target.
- [A mixed peer outcome can be summarized as false success] → derive aggregate state from immutable per-peer outcomes and require every success claim to retain its peer receipt.
- [Installed Sovereign state could be mutated during testing] → use isolated temporary homes/transports only and assert no configured service endpoint is contacted.
- [Mobile release appears green despite the known size failure] → keep binary-size status blocked and physical devices pending in generated status and release evidence.
- [Plugin generation grows or activation partially changes] → include component and index in one signed manifest/pointer and test tamper/rollback/receipt parity locally.
- [Documentation generation becomes hosted testing] → keep `docs:sync` deterministic and restrict Pages to packaging/deployment; all semantic/build gates run locally.

## Migration Plan

1. Land the MCP facade and deterministic schemas behind a binary feature that does not alter daemon defaults.
2. Add `exec-remote`, its dependency-direction checks, durable fixtures, and an estate-only adapter without enabling a live installed service.
3. Add certification evidence indexing and local verification fixtures.
4. Extend installers/doctors/manifests, build the release binary, install/sign/read back locally, and publish a new signed generation only after local gates pass.
5. Add generated references, Docusaurus/guide/ADR content, and local `docs:check` coverage.
6. Run focused Rust, remote disposable-peer, MCP, installer/doctor, plugin, OpenAPI/docs, and real-use-case certification. Archive redacted evidence with explicit pending dispositions.
7. Roll back by restoring the prior binary and atomically switching the plugin pointer to the prior generation; remote is feature/config optional and local execution remains available.

## Open Questions

- The production Sovereign transport deployment remains evidence-pending until isolated multi-process peers can run without touching installed state; deterministic in-memory peers certify the protocol kernel only.
- Mobile Tier W remains not release-ready because both measured dispatcher-retained ABI deltas exceed 12 MiB; this change documents and preserves that blocker rather than changing the budget.
- Physical iOS and Android receipt round trips remain pending until connected devices are available.
