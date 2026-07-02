# Phase Reflection: phase-okf-llm-wiki-adoption

**Project:** prometheus-skill-system (+ prometheus-knowledge-rs, cross-repo)
**Date:** 2026-07-02
**Phase completion:** 100% (8/8 changes)
**Changes completed:** 8 / 8

Gate: sycophancy-correction — analyze_reflect_phase LLM backend was 401 (its
own env gap); fell back to detect_sycophancy (pattern-based, strict) → score
0.0, no S-08. Audit: sycophancy/reflect-2026-07-02T17-15-23Z.json.

## Delta — where execution diverged from the plan

1. The core deliverable was undeployed beyond this machine for the entire phase. The four format-layer commits in prometheus-knowledge-rs (91aee74, 965aea9, 673c8e4, 231a5be) were committed LOCAL ONLY; no push/PR happened during execute. "8/8 done" overstated durability — until the closing push, a fresh clone would not have the OKF format at all.
2. change-okf-007 (the llm-wiki skill) was completed by a DIFFERENT parallel session, not the planned executor. progress.json was out of sync (showed 5/8 while 007 was already archived on disk). I only caught it by noticing an unexpected archive dir and skills/documentation/ tree, then had to verify and reconcile it into the count mid-execute.
3. The MCP surface was serving a STALE pre-OKF binary the whole phase, and its launchd plist had NO LLM env vars at all — so knowledge_ingest was silently broken (401 against api.openai.com) independent of OKF. This latent break only surfaced because change-okf-008 actually exercised the MCP write path; earlier changes verified only via the CLI, which masked it.
4. Two real build interruptions from environment contention: one genuine cargo deadlock (build alive, zero rustc children, 26 min no progress — required killing an orphaned hung live-test from an ended session holding the package-cache lock), and target/debug being wiped mid-phase by a parallel cargo clean. Both cost significant wall-clock and required diagnosis.
5. The plan asserted "OpenSpec available: NO." That was wrong — openspec/ and the openspec CLI exist repo-wide (91 changes). I had not checked for the directory during assess/plan. Caught at first kbd-apply detect (it resolved to openspec), corrected by pinning specBackend: native-kbd, and the deeper "openspec validate fails on all 91 changes" issue was spun off as a separate task.
6. The pk ingest LLM path has a known intermittent (~1/15) malformed-200 failure from the local openai-proxy. I diagnosed but did not root-cause it (a parallel task hardened the client side, 9645c12); it remains proxy-side. A clearly-labeled test-marker entry was also left in the live KB.

## Root Cause

1. The execution.md approval gate ("confirm before pushing to the knowledge-rs remote") was correct, but I treated local commits as "done" without tracking the push as an explicit outstanding deliverable in progress.json — so durability silently lagged completion. The gate protected against unwanted pushes but had no counterpart reminding that the push still had to happen to close the loop.
2. No cross-session lock or shared claim on changes: the plan scheduled 007 as parallelizable, and another session took it, but nothing reconciled its completion back into this phase's progress.json. The file-based KBD state has no multi-writer coordination.
3. Verification strategy front-loaded CLI checks (fast, my-shell env) and deferred the MCP/hook surface to the final change. The MCP server runs under launchd with a different environment; a CLI-only verification cannot see a launchd-env deploy gap. The gap predated the phase (plist last edited Jun 28) but was invisible until the integration change.
4. The machine hosts several concurrent cargo/agent sessions (nightly checks, UAR worktrees, the openai-proxy live tests) contending for CPU and the global ~/.cargo/.package-cache lock. One ended session left a hung live-backend test holding the lock. No isolation between sessions' builds.
5. Assess/plan inspected the codebase and specs but did not run the backend-detection that kbd-apply uses; "OpenSpec available" was inferred from the absence of openspec in the change-authoring flow, not from an ls of the repo root.
6. The proxy flake is in a different repo (openai-proxy) and non-deterministic; root-causing it was out of scope and correctly deferred. The test-marker was left because no delete tool is exposed and removing the file alone would desync index.md/log.md.

## Corrective Actions

1. Before declaring a cross-repo phase closed, add the remote push/PR as an explicit tracked change or a progress.json field, not just an execution.md caveat. This phase's closing push is being done now; future phases should schedule it as a first-class step gated on the approval, not as an afterthought.
2. When a phase plan marks changes parallelizable, the reconciliation step (re-scan archive/ + re-count progress.json before reflect) should be an explicit reflect prerequisite, not an accident of observation. Consider a kbd-apply "resync" that recomputes changes_completed from the archive dir.
3. For any phase touching a service that runs under launchd/systemd, verify against the SERVICE's environment early (not just the interactive shell), and diff the service's deployed binary age against the build. Would have surfaced both the stale binary and the missing LLM env before the last change.
4. For Rust-heavy phases on this shared machine, expect build contention; check build liveness by file-write freshness (not just process-alive) before assuming progress, and treat a genuinely stalled cargo (no rustc children) as killable. This heuristic worked once diagnosed — codify it.
5. Fold a `kbd-apply detect` (or an ls of openspec/ + .specify/) into the assess checklist so backend availability is observed, not inferred.
6. Keep the proxy-flake workaround (retry-once on ingest) documented; leave root-cause to the openai-proxy task. For KB hygiene, either expose a delete that also regenerates index/log, or stop writing test markers to the live KB.

## Goals

| Goal | Status | Notes (verified against archived changes + live e2e, not the plan) |
| --- | --- | --- |
| OKF frontmatter conformance (required type + recommended fields, unknown-key tolerance) | MET | Writer emits type + optionals; permissive parser (only type required, extra map round-trips, legacy docs parse). Verified by pk-store unit tests + live MCP-ingested page. |
| index.md/log.md maintained per OKF §6/§7 on every ingest | MET | regenerate_index + append_log in Librarian::compile; integration test + live e2e (both files maintained on disk). |
| Cross-links as bundle-relative body links + Citations (§5/§8) | MET | Body-link extraction (pulldown-cmark) drives the link graph; compile prompt emits inline links + Citations. Live page shows both, links derived from body. |
| llm-wiki operations (ingest/query/lint) as first-class skills + schema doc | MET | skills/documentation/llm-wiki/ (SKILL.md + 2 refs); passes validate:strict. Delivered by a parallel session, verified here. |
| pk lint enforces OKF §9 conformance with permissive semantics | MET | Deterministic conformance (type = error; optionals/broken-links/orphans = warning); LLM lint best-effort; deterministic type auto-fix. Unit + integration tests + live pk lint --fix. |

Overall: 5/5 MET, from a 0/5 baseline at assessment.

## Delivered Changes

- `change-okf-001-vendor-specs` — vendor OKF + Karpathy docs, CLAUDE.md decision (by: claude-code)
- `change-okf-002-pk-workspace-baseline` — clone knowledge-rs, build/test baseline, diagnose ingest flake (by: claude-code)
- `change-okf-003-permissive-okf-parser` — OKF §9 permissive parser + reserved filenames (by: claude-code)
- `change-okf-004-okf-writer-and-id-mapping` — OKF writer + path-based concept IDs + traversal guard (by: claude-code)
- `change-okf-005-index-log-and-body-links` — index/log maintenance + body-derived link graph + Citations (by: claude-code)
- `change-okf-006-okf-lint` — OKF §9 conformance lint + deterministic auto-fix (by: claude-code)
- `change-okf-007-llm-wiki-skills` — llm-wiki skill + schema doc (by: a parallel session; verified + reconciled by claude-code)
- `change-okf-008-integration-verification` — hooks/MCP/e2e verification, MCP redeploy, goal re-check (by: claude-code)

## Technical Debt

- knowledge-rs commits were local-only through the phase (being pushed at close). If the push is skipped, the entire format layer is undeployed.
- Test-marker entry left in the live KB (~/.prometheus/knowledge/wiki/okf-v0-1-integration-verification-after-change-okf-008.md); harmless but not real knowledge.
- pk ingest intermittent proxy flake (~1/15) unresolved at the proxy; mitigated by retry-once and the client-side hardening in 9645c12.
- The debug binaries were installed to ~/.local/bin/pk and /usr/local/bin/pk-cherry during verification; a release build + reinstall is the proper global install (in progress at close).
- The BDD draft (tests/features/drafts/okf-wiki-ingest.feature) has no step definitions yet — intentionally, per the immutable-tests rule, but the contract is unexecuted.

## Architecture Integrity

- AGENTS.md violations: NONE. The CLAUDE.md documentation-hierarchy rule was honored — the cross-cutting OKF decision was recorded in this repo's canonical CLAUDE.md, crate-specific work stayed in knowledge-rs.
- Constraint violations: N/A (no constraints.md present).
- The BDD immutable-tests rule was honored: the new feature is a draft under tests/features/drafts/, no existing tests edited.

## Cross-Tool Coordination Notes

- Progress tracking: GAPS FOUND — change-okf-007 was completed by a parallel session without any update to this phase's progress.json; I had to detect and reconcile it manually. File-based state has no multi-writer safety.
- Handoff quality: MIXED — the assess→plan→execute handoffs I wrote were clear and self-consistent, but the plan carried an unchecked "OpenSpec: NO" assertion that had to be corrected at execute. The shared working tree of knowledge-rs (three sessions committing) required per-file staging discipline to avoid cross-contaminating commits, which worked but was manual.
- Recommendations: a progress.json resync from archive/ before reflect; a service-environment verification step for launchd-hosted components; fold backend detection into assess.

## Lessons Learned

- Local commits are not deployment. For cross-repo work behind an approval gate, track the push as an explicit deliverable or the phase closes "done but undeployed."
- Verify against the real runtime environment (launchd service env, deployed binary age), not just the interactive shell — a CLI-green phase can hide a broken service.
- On a shared build machine, distinguish "slow under contention" (rustc children active, file writes advancing) from "genuinely stalled" (cargo alive, zero rustc, no writes) before killing anything; kill orphaned lock-holders from ended sessions.
- Permissive-consumption design paid off: making the parser lenient (only type required, unknown keys preserved) BEFORE the writer changed meant no existing entry ever failed to load, and the live KB's pre-OKF legacy entry parsed and indexed as "Uncategorized" with no special-casing.
- Deterministic-vs-LLM separation in lint was the right call: OKF conformance stays reliable when the flaky lint model is down, because conformance never routes through the model.

## Next Phase Focus

Recommended next phase: **phase-okf-push-and-harden** (or fold into the next knowledge-rs cycle).

Top priorities:
1. Push/PR the knowledge-rs OKF commits to origin and confirm CI is green there (the durability gap this phase leaves open).
2. Implement step definitions for the drafted OKF BDD feature and wire it into CI so the ingest→index/log→links→Citations→lint contract is executable, not just documented.
3. Resolve or formally accept the openai-proxy intermittent-malformed-response flake, and add a real delete/GC path so KB hygiene (test markers, stale entries) is maintainable.

## Context for Next Phase

The OKF format is live in the local pk stack (binaries installed, MCP server redeployed) but the source is unpushed until this phase's closing action. The format spans two repos: format/lint in knowledge-rs (pk-store/pk-librarian), skill/schema/hooks here. The KB is real and hook-populated (reflect/Stop hooks ingest session summaries), so it is no longer empty — a future format change would now carry a migration cost that this phase did not.
