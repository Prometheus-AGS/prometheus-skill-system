# Execution Evidence: repair-kbd-memory-rest-contract

## Task 1.1 — Installed service and canonical routes

Observed locally at `2026-08-29T18:54:51Z` before implementation:

- LaunchAgent `ai.prometheus.surreal-memory-native` is running `/usr/local/bin/surreal-memory-server` with `API_PORT=23001`, `MCP_STDIO=false`, and the SurrealDB endpoint `ws://127.0.0.1:28000`. Launchd reported one run, an active PID, and no prior exit.
- `GET http://127.0.0.1:23001/health` returned HTTP 200 and `{"service":"surreal-memory-server","status":"ok","version":"1.7.0"}`.
- `GET http://127.0.0.1:23001/api/v1/entities/search?q=kbd_lifecycle_event` returned HTTP 200 with a JSON array. The empty result is a valid reachable response and confirms the supported search contract.
- The installed Claude MCP configuration points at `http://localhost:23001/mcp/sse`; KBD must normalize that MCP transport URL to the REST origin `http://127.0.0.1:23001` while retaining explicit configuration precedence.
- Route comparison confirmed `/health` and `/api/v1/entities/search` are live, while obsolete `/healthz`, `/api/find_relevant`, and `/api/entities` return HTTP 404.

Commands were bounded, local, read-only probes using `curl --noproxy '*' --connect-timeout 2 --max-time 5`, plus read-only `plutil` and `launchctl print` inspection. No service restart or external validation was used.

## Task 1.2 — Endpoint discovery and REST normalization

Implemented in `skills/process/kbd-process-orchestrator/shared/lib/memory.sh`:

- Explicit `UAR_MEMORY_MCP_URL` / `KBD_MEMORY_MCP_URL` selection remains first, followed by project `restEndpoint` / legacy `mcpEndpoint`, then the canonical local `http://127.0.0.1:23001` default.
- HTTP(S) discovery URLs are normalized to their service origin, so `/mcp/sse`, `/mcp/http`, and other transport paths cannot leak into REST route construction.
- Availability probes the normalized `/health` route with bounded connection/runtime limits and loopback-specific proxy bypass.
- The process-lifetime positive/negative cache is preserved. MCP-tool-only availability remains supported but intentionally exposes no fake REST URL to shell callers.

Focused test authoring and execution are deferred to task 2.2 because the repository's immutable implementation-first policy prohibits changing or running tests until the coherent change implementation is complete. The configured, canonical-default, cached, invalid, and unreachable cases remain required final integration coverage.

## Task 1.3 — Canonical lifecycle entity writes

Implemented in `skills/process/kbd-process-orchestrator/shared/lib/memory-log.sh` after confirming the installed server's `CreateEntityRequest` contract directly in `tools/surreal-memory-server/src/contracts.rs` and its HTTP 201 handler in `src/api/entities.rs`:

- Writes now target `POST /api/v1/entities` with `name`, `entity_type`, and `observations`.
- Lifecycle metadata is encoded as one compact JSON string inside the string-only observations array; obsolete object observations, camel-case fields, embedded relations, and `/api/entities` were removed.
- Project identity falls back to the signed waypoint's `projectId` when the optional legacy project file is absent, preserving same-project recall metadata.
- Payload encoding and HTTP failures produce at most one fixed diagnostic and always exit successfully so memory mirroring cannot block lifecycle progress.
- HTTP work uses bounded connect/runtime limits and loopback-specific proxy bypass.

The fake-server integration scenario is retained for task 2.2, after the complete write/recall/documentation implementation, as required by the immutable implementation-first policy.

## Task 1.4 — Canonical entity recall and deterministic ranking

Implemented in `skills/process/kbd-process-orchestrator/skills/kbd-memory-recall/kbd-memory-recall.sh` against the service's current entity-search contract:

- Recall now uses bounded `GET /api/v1/entities/search?q=kbd_lifecycle_event` retrieval with loopback proxy bypass; the removed `POST /api/find_relevant` request and its obsolete semantic-search payload are gone.
- Current string observations are decoded as lifecycle JSON, while legacy object observations remain readable during migration.
- Candidates are flattened and ranked deterministically by same-project affinity, query-token overlap, event recency, entity name, and observation index, then limited to the top five.
- Reachable empty searches produce a normal prior-context document with explicit empty sections. Transport failure and invalid response contracts produce distinct atomic stub digests without blocking orchestration.
- Project resolution uses the same optional project-file then signed-waypoint fallback as lifecycle writes, keeping recall and write identity aligned.

The reachable-match, reachable-empty, and unreachable fake-service scenarios remain deferred to task 2.2, after documentation completes the coherent implementation, in accordance with the immutable implementation-first policy.

## Task 2.1 — Owning memory contract documentation

Updated the source recall skill, retention contract, integration reference, and owning orchestrator summary so they describe the implemented service boundary rather than the removed API:

- The documented routes are now `GET /health`, `POST /api/v1/entities`, and `GET /api/v1/entities/search?q=kbd_lifecycle_event`.
- The entity schema documents snake-case request fields and one compact JSON string in `observations`; prior claims about object observations and automatically-created graph relations were removed.
- Discovery precedence now matches the implementation: explicit overrides, project `restEndpoint`/legacy `mcpEndpoint`, canonical local origin, bounded health probe, then MCP-only availability without a fabricated shell URL.
- Recall documentation now states the deterministic same-project, token-overlap, recency, entity-name, and observation-index order and accurately distinguishes reachable-empty, transport-failure, and invalid-contract output.
- The undocumented future `--cross-project` behavior and obsolete MCP-first/`healthz` contract were removed. Cross-project candidates remain lower-ranked rather than falsely described as excluded.

Generated plugin and installed copies are intentionally unchanged here; `reconcile-kbd-control-plane-projections` owns deterministic distribution generation and installation refresh after all source changes are complete.

## Task 2.2 — Local full-integration certification

Replaced the legacy isolated memory smoke checks with nine local full-integration scenarios in `skills/process/kbd-process-orchestrator/shared/lib/tests/test-memory.sh`. The harness invokes the production shell entrypoints as separate Bash processes across a real loopback HTTP boundary and asserts their filesystem output and captured wire contracts:

1. Explicit MCP URL normalization and lifecycle entity write.
2. Process-lifetime positive availability caching after a real health exchange.
3. Reachable recall ranking across current string and legacy object observations.
4. Project `restEndpoint` discovery and reachable-empty digest.
5. Legacy `mcpEndpoint` normalization through production recall.
6. Canonical installed local service discovery through production recall.
7. Explicit unreachable-service fail-open stub and atomic replacement.
8. HTTP write failure with exactly one fixed diagnostic and successful lifecycle exit.
9. Invalid entity-search contract with a distinct atomic fail-open stub.

Exact local results:

- `/bin/bash --version` → `GNU bash, version 3.2.57(1)-release (arm64-apple-darwin25)`.
- `/bin/bash -n` on `memory.sh`, `memory-log.sh`, `kbd-memory-recall.sh`, and `test-memory.sh` → exit 0.
- `/bin/bash skills/process/kbd-process-orchestrator/shared/lib/tests/test-memory.sh` → exit 0, `9/9` full-integration scenarios passed.
- `npm run validate:strict -- skills/process/kbd-process-orchestrator` → exit 0, 23 skills validated. The validator reported 42 pre-existing advisory description warnings and no errors.
- `git diff --check` → exit 0 with no output.

An additional attempt to record this change-level suite through `prometheus kbd gate run --kind integration --scope repair-kbd-memory-rest-contract -- ...` was correctly refused at canonical revision 154 because the active parent phase still has three incomplete changes. It created no unresolved blocker. The passing direct local run is the change evidence; `reconcile-kbd-control-plane-projections` will record the required canonical integration/certification receipt only after the complete parent implementation exists, as mandated by the phase-wide gate policy.

No Cargo process or Rust build was started for this shell/documentation-only change.

### Post-archive adversarial amendment

Later parent-change review made canonical installed-service discovery an explicit
`KBD_MEMORY_LIVE_PROBE=1` scenario so the default suite is hermetic, added a
reachable HTTP-error scenario distinct from transport failure, and added an
MCP-only no-REST-origin scenario. The final default suite passes 10/10 enabled
full-integration scenarios with the live probe explicitly skipped; the explicit
live run passes 11/11. The original 9/9 result above remains the exact result at
archive time rather than being rewritten.
