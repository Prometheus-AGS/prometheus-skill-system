# PLAN: kbd-control-plane-recovery

Project: prometheus-skill-pack
Date: 2026-08-29
OpenSpec available: YES
Changes to implement: 4

## CHANGE LIST (ordered)

1. `repair-kbd-memory-rest-contract`: Make KBD memory logging and recall use the installed surreal-memory service contract.
   - Scope: shell orchestration, REST payloads, tests, skill references
   - Depends on: NONE
   - Recommended agent: Codex
   - Est. complexity: M
   - Complexity score: Medium
   - Model class: medium
   - Customer value: HIGH
   - Details: Preflight the installed service configuration and `/health` route to confirm the canonical local `127.0.0.1:23001` endpoint, then give `memory.sh` that default while preserving explicit overrides, normalize MCP URLs to a REST base, and replace obsolete routes and payloads. Encode lifecycle metadata as a string observation accepted by `POST /api/v1/entities`, recall through `GET /api/v1/entities/search`, rank same-project/token-overlap/recency matches locally, and expand smoke tests to prove writes plus a non-stub digest against a fake canonical server. Update every memory contract reference and verify touched scripts with macOS `/bin/bash` 3.2; distribution generation is deferred exclusively to change 4.

2. `harden-uiux-routing-discovery`: Make UI/UX routing existing-target-first and capability-aware.
   - Scope: injected agent-rule template, skill roster, injector documentation/tests
   - Depends on: NONE
   - Recommended agent: Codex
   - Est. complexity: S
   - Complexity score: Low
   - Model class: small
   - Customer value: HIGH
   - Details: Require callers to resolve an existing implementation target before invoking Impeccable and to treat proposed future paths as design destinations, not context roots. Replace the unverifiable mandatory `ux-designer` attribution with a capability check: consult it when actually installed, otherwise use the installed UI/UX Pro Max plus `frontend-design` review and record that explicit fallback. Verify a dry-run injection updates only the managed fence and that the UAR incumbent target resolves without modifying the dirty UAR worktree; distribution generation is deferred exclusively to change 4.

3. `add-kbd-registry-prune`: Add explicit, recoverable cleanup for missing KBD replica registrations.
   - Scope: kbd-runtime registry API, prometheus CLI, unit/integration tests, operator docs
   - Depends on: NONE
   - Recommended agent: Codex
   - Est. complexity: M
   - Complexity score: Medium
   - Model class: medium
   - Customer value: HIGH
   - Details: Add a dry-run-by-default `prometheus kbd projects --prune-missing` operation with an explicit `--apply` gate. Under the registry lock, identify only paths that no longer exist, write a timestamped registry backup plus checksum/receipt, remove those registrations without deleting retained project runtimes, and return the exact removed entries and project IDs. Test dry-run immutability, apply behavior, backup restoration evidence, preservation of existing paths, and repeat-run idempotence; then apply it to the live registry and restart sovereign-sync to prove zero unavailable-authority warnings.

4. `reconcile-kbd-control-plane-projections`: Preserve historical evidence while removing compatibility artifacts from live discovery and refreshing installed surfaces.
   - Scope: KBD evidence artifacts, generated Codex/plugin distributions, installed skill copies, local service certification
   - Depends on: `repair-kbd-memory-rest-contract`, `harden-uiux-routing-discovery`, `add-kbd-registry-prune`
   - Recommended agent: Codex
   - Est. complexity: M
   - Complexity score: Medium
   - Model class: medium
   - Customer value: MEDIUM
   - Details: First confirm that `.kbd-orchestrator/phases/openspec-mirror-drift-cleanup::sovereign-sync-service-reliability` still exists and contains only compatibility evidence, then move it into `.kbd-orchestrator/backups` with a receipt pointing to the canonical nested child record; do not modify or delete canonical history. Regenerate distributions twice for byte identity, validate them, refresh the user installation through the repository-owned installer, run live memory recall and memory-hook write probes, then restart sovereign-sync twice and certify KBD doctor/status plus the cleaned registry. Record exact local commands and results; no hosted validation is permitted.

## EXECUTION ROUND ORDER

Round 1 (sequential in this shared worktree): `repair-kbd-memory-rest-contract`, `harden-uiux-routing-discovery`, `add-kbd-registry-prune`

Round 2: `reconcile-kbd-control-plane-projections`

The three Round 1 changes touch distinct primary modules, but share a repository and must not regenerate common artifacts independently. The final change owns all distribution generation and depends on all three because its purpose is refresh and live certification of their combined result.

## COMMANDS TO RUN

```text
/opsx:new repair-kbd-memory-rest-contract
/opsx:new harden-uiux-routing-discovery
/opsx:new add-kbd-registry-prune
/opsx:new reconcile-kbd-control-plane-projections
```

## SCOPE CUTS AND TRADE-OFFS

- Do not add a compatibility `/api/find_relevant` route to surreal-memory-server; KBD is the stale client and must migrate to the canonical API.
- Do not auto-prune the registry at daemon startup; cleanup must remain explicit, backed up, and operator-visible.
- Do not create or vendor a third-party `ux-designer` skill under a misleading Anthropic attribution; route to installed capabilities and make absence actionable.
- Do not edit the dirty UAR implementation. This phase repairs reusable orchestration and verifies the external target read-only.
- Do not delete retained KBD runtime directories or canonical nested phase history.
- C-03 does not require `docs/codex-plugin.md` or `CLAUDE.md` changes because no plugin manifest, MCP declaration, hook registration, or installer contract changes; the edited behavior is documented in the owning KBD skill references. If implementation expands into one of those surfaces, update those documents in the same change.

## PLAN COMPLETE
