# PLAN: phase-readiness-contract

Revision 1, 2026-10-18. Consumes assessment.md + analysis.md + spec.md + tasks.json + verification.md + adopted-contract.json (same dir). Frozen per operator rule 2; amendments only via new canonical `decision record --supersedes`.

## Ordered changes (coverage map: task ↔ scenarios ↔ files)

| Order | Task | Change | Scenarios covered | Files |
|---|---|---|---|---|
| 1 | T1 | Contract schema + template | RC-01 | `skills/process/kbd-process-orchestrator/references/schemas/adopted-contract.schema.json` (NEW), `.../templates/adopted-contract.template.md` (NEW) |
| 2 | T2 | Handoff schema identity extension | RC-04, RC-05 | `.../references/schemas/handoff.schema.json` (EDIT, backward compatible) |
| 3 | T3 | stage-gate.sh semantic validation | RC-04, RC-05 | `skills/process/kbd-process-orchestrator/shared/lib/stage-gate.sh` (EDIT) |
| 4 | T4 | Adoption writer | RC-01, RC-06 | `shared/scripts/kbd-contract-adopt.sh` (NEW, chmod +x) |
| 5 | T5 | Readiness checker | RC-01..04, RC-06 | `shared/scripts/kbd-phase-readiness-check.sh` (NEW, chmod +x) |
| 6 | T6 | Skill/prompt minimal edits | RC-07 | 9 files (see tasks.json) |
| 7 | T7 | This phase's contract file | RC-01 | `.kbd-orchestrator/phases/phase-readiness-contract/adopted-contract.json` (authored in Spec; adoption recorded via existing canonical writer) |
| 8 | T8 | Boundary integration test | RC-01..07 | `shared/scripts/tests/test-kbd-readiness-gate.sh` (NEW, chmod +x) |

Worker route for all tasks: main agent, `kimi-code-plan-global/k3-256k` (operator-confirmed subscription; Extra Usage disabled). No subagents (single authorized route). Owner/reviewer rights: main agent implements; review happens once, sequentially, after complete production (rule 9); review rounds ≤2.

## Freshness / evidence-invalidation fields

- Contract digest: `sha256(jq -S -c 'del(.identity.contractDigest)|del(.identity.planRevision)')` — re-computed by writer and checker with identical normalization.
- Adoption staleness: digest mismatch vs latest non-superseded canonical adoption decision for this contract path ⇒ REJECT-STALE. planRevision is traceability, not an equality gate.
- Source edits after a boundary run invalidate only the affected scenarios' evidence (per-file digests recorded in boundary-results.json); history preserved.

## Startup migration noise (recorded; excluded from acceptance/commit scope)

~164 worktree changes from typed-runtime initialization + managed OpenSpec refresh: historical progress.json regenerated to projectionContractVersion 2 (completion counters preserved), legacy generated tasks.md deleted, position/waypoint rewritten, `.agents`/`.cursor`/`.devin` payload refresh (`authoredPathsChanged: []`). All `generatedBy: kbd-runtime` or refresh receipts; recoverable via git + append-only runtime log; NOT part of this phase's production diff; the closed `phase-learning-deploy-and-debt` release work is not resumed.

## Single boundary execution packet (post-implementation only)

Commands (exact argv, run once, in order, from repo root):
1. `prometheus kbd gate run --kind integration --scope phase-readiness-contract -- bash shared/scripts/tests/test-kbd-readiness-gate.sh` — signed gate receipts; the test executes RC-01..RC-06 semantics incl. the real RC-06 amendment and scratch-isolated negative controls.
2. `npm run validate:strict skills/process/kbd-process-orchestrator` — RC-07 packaging qualification.
3. `node scripts/generate-skill-system-distribution.js --check` — RC-07 generated-payload parity qualification.

Effects: signed gate start/finish receipts; amendment decision (RC-06) in canonical audit; `boundary-results.json` (per-scenario attempted/observed/evidence/status) written to the phase dir; scratch roots removed.

Approvals: none beyond existing operator authorization; no new providers/credentials; no push/publication.

Cleanup: `rm -rf "$TMPDIR"/kbd-rc-*` scratch roots only; canonical history preserved (append-only).

Stop conditions: budget exhaustion / 2 repairs / 2 no-progress ⇒ checkpoint+BLOCKED; frozen criteria met ⇒ terminate (no invented follow-ups).

## Not in this plan (frozen exclusions)

Per adopted-contract.json `exclusions` + `resilienceScope.deferredOptional`.

PLAN COMPLETE — next: register change/tasks via canonical commands, record contract adoption via existing canonical writer (`prometheus kbd decision record`), then Execute T1→T8 (implementation-first; no tests until all production diffs complete).
