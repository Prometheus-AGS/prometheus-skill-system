## Context
Assess and Analyze recorded reproducible early returns and phase/run attribution defects. The operator's two-repository authorization overrides mini's default reference-only scope for this phase.

## Goals / Non-Goals
Correct observed contradictions without rewriting canonical events, resetting earlier evidence, changing protected tests, installing globally, or changing other products. This is a prerequisite to agent-team creation.

## Decisions
1. Existing new-phase helpers finish metadata/hook postconditions after canonical activation. Validate existing project JSON before mutation; preserve unknown keys and remove only the legacy active_phase alias. Fire phase:before once before success; hook failures remain best-effort. Do not re-fire it for the current repaired phase.
2. Full KBD runtime owns the projection fix: local implementation counts remain; phase evidence/certification/publication are NOT_TRACKED until backed by phase-scoped records. Existing run values remain under explicit runCompletion with completionScopes. The event journal and run completion are unchanged. Waypoint exposes stageId/stageStatus/lifecycle, top-level parentPhase null and nested immediate parent; reminder labels actual stage and lifecycle separately.
3. Rewrite current source instructions that prescribe generated-state edits or bypass KBD apply. Keep legacy schemas as documented migration inputs, not current writer authority. Update current gate descriptions to match fail-on-missing-predecessor implementation.
4. Align stale test-first requirements with governing implementation-first integration policy. Full file-count/docs-only QA exemptions are removed. Record scoped operator authorization for both repositories and branch pushes; do not remove generic reference-only protection for other work or knowledge-log publication restrictions.
5. Repository policies need scope, not artificial uniformity: full prohibits hosted validation. This phase validates both packages locally. Mini's pre-existing CI matrix is not proof of this phase and is not disabled merely because full has a different host policy. Correct the tests-first contradiction in mini canonical project-tooling spec; keep historical archive records intact.
6. Clarify kbd-init ownership: initialization owns full project configuration; phase helpers own activePhase and documented minimal bootstrap. Clarify source-tree required/install-only lifecycle in the older full harness declaration spec.
7. Regenerate distribution/docs/rule outputs through existing generators. If batching generated surfaces until the final team change, name that reconciliation owner and do not claim distribution certification early.

## Scope
Full: substrate/kbd-runtime/src/lib.rs, its integration targets, current kbd-process-orchestrator source skills/prompts/references/helper, current constraints, matching OpenSpec delta and relevant docs/generated distribution.
Mini: scripts/kbd-new-phase.mjs, its real integration target, current KBD skills, rules/src where actual routing conflicts exist, openspec/config.yaml, current constraints, project-tooling delta and docs/generated distribution.
No installed skill cache or global binary is edited. Installed runtime projection refresh needs a locally built corrected binary; otherwise report deployment as pending rather than hand-edit generated state.

## Verification
After all production corrections are implemented: real Runtime public API → journal replay → filesystem projections proves earlier complete run evidence cannot certify a new phase, stages/parents are accurate, and repeated projections are stable. Real helper child process with canonical CLI and hook command proves runtime-mode project metadata, preserved keys and one hook dispatch; malformed project JSON refuses before writes. Integration scans verify current instruction agreement. Native legacy helper receives equivalent process coverage. Apply OpenSpec validation and current distribution/idempotence checks at their declared boundary; deterministic QA then independent diff review before archive.

## Risks
Installed CLI versions may lag source; report the exact binary under test and avoid global installation. Source documentation has generated mirrors; editing mirrors directly would reintroduce drift. Legacy shadow behavior remains supported and requires coverage.

## Verification-discovered correction
Actual task 1.1 completion encountered a broken global OpenSpec shim. Both apply drivers catch progress failure and synthesize remaining=1/0, emitting false change completion. Canonical tasks remain pending. Task 2.1 includes correcting this observed failure before authoring its integration scenarios: require actual backend progress before mutations, propagate errors, and never synthesize completion. Scope adds mini scripts/kbd-apply.mjs and full existing kbd-apply.sh. Use repository-local OpenSpec in this session; do not modify the global launcher.

## QA refinement
Validate OpenSpec progress shape before any inferred completion. Mini remediation text uses Node stageHandoffSkip, never the unavailable Bash helper. Preserve the already-committed user-edit ownership behavior in mini doctor; reconcile its stale unprotected compatibility assertion without altering repair production semantics. Regenerate mini distribution after source changes.

Review refinement: write local goals after successful canonical create/activate/start. On rejected create, no authored phase goals or metadata/hook success is produced. Already committed canonical events are not rolled back if a later separate command fails.
