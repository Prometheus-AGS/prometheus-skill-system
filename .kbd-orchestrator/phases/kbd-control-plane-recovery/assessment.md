# ASSESSMENT: kbd-control-plane-recovery

Project: prometheus-skill-pack
Date: 2026-08-29
Codebase baseline: The repaired sovereign-sync control transport is running and KBD mutations succeed, but memory hooks, UI/UX routing, and stale registry maintenance still encode obsolete or incomplete control-plane contracts.
Cross-tool progress: none

## IMPLEMENTATION STATUS

- Surreal-memory service availability: **DONE** — `ai.prometheus.surreal-memory-native` is running on `127.0.0.1:23001`; `GET /health` returns 200 with version 1.7.0 and `GET /api/v1/entities/search?q=kbd_lifecycle_event` returns 200.
- KBD memory endpoint discovery: **MISSING** — `shared/lib/memory.sh` has no canonical local default, so a healthy default installation is reported unavailable unless `KBD_AVAILABLE_TOOLS`, `UAR_MEMORY_MCP_URL`, `KBD_MEMORY_MCP_URL`, or `.kbd-orchestrator/memory.config.json` happens to supply a hint. The pack already installs `http://localhost:23001/mcp/sse` in `scripts/install-skills-flat.sh` and `scripts/install-platforms.ts`; the KBD helper should use the same `http://127.0.0.1:23001` local service default while retaining explicit overrides.
- KBD memory write/recall transport: **OBSOLETE** — `memory-log.sh` posts an object-shaped legacy payload to `/api/entities`, while `kbd-memory-recall.sh` posts to `/api/find_relevant`; neither route exists on the current server. The canonical entity contract uses `name`, `entity_type`, and string observations at `/api/v1/entities`, and recall must use the supported entity search route.
- Presentation/A2UI context targeting: **PARTIAL** — this finding is cross-repository. The proposed future path `/Users/gqadonis/Projects/prometheus/universal-agent-runtime/frontend/src/features/presentation` is not implemented. The verified incumbent workspace is `/Users/gqadonis/Projects/prometheus/universal-agent-runtime/frontend/src/features/skills/ui/a2ui-library-page.tsx`, backed by `/Users/gqadonis/Projects/prometheus/universal-agent-runtime/frontend/src/platform/entities/a2ui-library`; both were resolved with local filesystem inspection, and Impeccable resolved the existing page target to the UAR frontend workspace. These files are intentionally absent from this skill-pack repository's tree. The source routing artifact is `skills/process/kbd-process-orchestrator/skills/kbd-inject-agent-rules/references/template-uiux-routing.md`; it does not name the UAR path directly, but it fails to require callers to resolve an existing incumbent target before invoking Impeccable.
- UI/UX skill discovery: **PARTIAL** — local inspection of `/Users/gqadonis/Projects/prometheus/universal-agent-runtime/.agents/skills` found Impeccable, `frontend-design`, `ui-ux-pro-max`, and `web-design-guidelines`, while searches of the active `~/.codex/skills`, `~/.agents/skills`, `~/.claude/skills`, and UAR-local catalogs found no `ux-designer` or `ui-ux-designer` skill. The routing source `skills/process/kbd-process-orchestrator/skills/kbd-inject-agent-rules/references/template-uiux-routing.md` nevertheless mandates `ux-designer`, and its roster attributes the name to Anthropic. An external GitHub search on 2026-08-29 found no official `anthropics/skills` entry by that name; this is landscape evidence, not a claim derived from the packet tree. The pack supplies no deterministic fallback or actionable availability check.
- Sovereign-sync partial authority availability: **DONE** — the daemon is running under launchd with keepalive, serves the Unix socket, and keeps healthy project routes available when registered authorities fail to open.
- Registry hygiene: **MISSING** — the canonical user-local registry at `/Users/gqadonis/Library/Application Support/prometheus/kbd/registry.json` contains 21 replica paths that no longer exist across six project IDs. Three project IDs have no usable registered path and emit `KBD authority startup project is unavailable` on every daemon restart. Registry ownership is implemented in `substrate/kbd-runtime/src/registry.rs`; the `prometheus kbd projects` surface is implemented in `tools/prometheus-cli/crates/prometheus-cli/src/commands/kbd.rs`. The CLI can list and register projects but offers no backup-bearing, explicit prune operation.
- Historical KBD projection evidence: **PARTIAL** — the canonical nested child record is retained under `.kbd-orchestrator/phases/openspec-mirror-drift-cleanup/children/sovereign-sync-service-reliability`, while the duplicate top-level compatibility directory `.kbd-orchestrator/phases/openspec-mirror-drift-cleanup::sovereign-sync-service-reliability` contains only the old memory-unreachable stub. No reconciliation receipt identifies the canonical record and preserves the duplicate under `.kbd-orchestrator/backups` outside the live phase namespace.

## CROSS-TOOL PROGRESS

- NONE — the phase ledger contains no registered implementation changes or tasks.

## SPEC GAP SUMMARY

- Memory documentation in `skills/process/kbd-process-orchestrator/skills/kbd-memory-recall/SKILL.md`, `skills/process/kbd-process-orchestrator/shared/references/memory-retention.md`, `skills/process/kbd-process-orchestrator/references/memory-integration.md`, and the parent orchestrator `SKILL.md` still describes `/healthz`, `/api/entities`, or `find_relevant`, which no longer match the server's REST contract.
- Memory tests assert the fallback stub but never simulate the canonical local service, validate the REST payload, or prove a non-stub recall digest.
- UI/UX routing treats optional third-party skill names as unconditional requirements and does not distinguish an existing implementation target from a proposed future path.
- Registry APIs do not expose an evidence-preserving maintenance operation for missing replica paths, leaving routine worktree removal to become permanent startup noise.
- The top-level `parent::child` compatibility projection has no live progress state and should not remain indistinguishable from a canonical phase.

## BUILD HEALTH

- memory smoke tests: **PASS** — `bash skills/process/kbd-process-orchestrator/shared/lib/tests/test-memory.sh` passed 6/6, but the suite only certifies the obsolete fallback behavior.
- surreal-memory health: **PASS** — local `/health` and canonical entity search returned 200.
- obsolete recall endpoint probe: **FAIL** — `POST /api/find_relevant` returned 404.
- sovereign-sync service: **PASS** — launchd reports the service running with keepalive and an active PID.
- focused registry/runtime tests: **UNKNOWN** — no prune capability exists yet, so no focused test can be run before implementation.
- known violations: memory scripts and docs disagree with the current server contract; the UI/UX roster asserts a skill source that cannot be verified.
- test coverage: **MINIMAL** — fallback-only memory tests exist; target-resolution and registry-prune behavior are uncovered.

## CONSTRAINT CHECK

- AGENTS.md violations: NONE in the inspected implementation. All validation for this phase remains local.
- `.kbd-orchestrator/constraints.md` violations: NONE observed before implementation. Any generated Codex distribution touched by source changes must be regenerated idempotently.

## GOAL PROGRESS

- Restore reliable `kbd-memory-recall` access: **NOT MET** — the default service is undiscovered and the recall route is obsolete.
- Resolve presentation and A2UI context targets before Impeccable: **PARTIAL** — the actual incumbent target is verified in the external UAR workspace, but the routing contract does not enforce existing-target resolution.
- Reconcile `ux-designer` discovery: **NOT MET** — the requirement is stale and has no installed-skill fallback.
- Reduce authority replay noise and reconcile stale state without discarding evidence: **PARTIAL** — service availability is fixed, but stale registrations and the duplicate projection remain live and unclassified.

## ASSESSMENT COMPLETE

The smallest coherent recovery is to align KBD memory scripts with the canonical local REST server, make UI/UX routing capability-aware and existing-target-first, add an explicit registry prune that writes a rollback backup before removing only nonexistent paths, and move the duplicate projection into the existing evidence backup area with a reconciliation receipt. Automatic deletion at daemon startup is rejected because registry cleanup must remain operator-visible and recoverable.
