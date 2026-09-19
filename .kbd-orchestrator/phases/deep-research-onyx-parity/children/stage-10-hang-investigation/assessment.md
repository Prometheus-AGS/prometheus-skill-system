ASSESSMENT: deep-research-onyx-parity› stage-10-hang-investigation
Project: prometheus-skill-pack
Date: 2026-09-10
Codebase baseline: The phase's full body of work (driver, six heredoc-to-file extractions, five test suites, adversarial-review research target) sits uncommitted on branch feat/cpc-001-002-integration-contract — 551 changed files. The deep-research driver and its suites exist and, as of this assessment, pass.
Cross-tool progress: parent phase 4 of 7 changes DONE (recorded by kbd-runtime); child phase has no changes registered.

IMPLEMENTATION STATUS (mapped to the child's four goals)
- Goal 1 — identify the exact blocking command with a reproducible trace: PARTIAL. The stage-10 call chain is known by reading (run-research.sh:478 → review_report at :340 → sync_report_status :364 → write-provenance.sh :365 → build-review-packet.sh :367 → judge dispatch :372-377). No blocking command has been isolated. New evidence from this assessment: the hang is intermittent, not deterministic — see Build Health.
- Goal 2 — pre-existing vs introduced, first appearance: NOT MET. Evidence is mixed: driver-contract scored 128/128 four times on 2026-09-09 including once mid-work, and the full suite passes 128/128 today, yet a single-scenario run hung 20 minutes earlier today. First appearance cannot yet be dated.
- Goal 3 — fix so run-research.sh completes a full deep run, proven by driver-contract passing its full count: NOT STARTED as a fix; however the acceptance gate (driver-contract 128/128) is currently green, which means the phase's proof standard needs to be re-stated for an intermittent defect (e.g. N consecutive clean runs), or it will certify nothing.
- Goal 4 — adversarial review of the diagnosis before any fix: NOT STARTED. No diagnosis exists to review. Judge routing note: ~/.prometheus/kbd/models.toml has judge = "k3"; if the diagnosing session runs as k3 the judge role must be repointed first or the review self-blocks by design.

CROSS-TOOL PROGRESS
- change-drt-001 dispatch smoke + thread contracts: DONE
- change-drt-007 install-surface repair: DONE
- change-drt-002 thread scheduler + budgets: DONE
- change-drt-005 multipass report: DONE
- change-drt-004 deterministic merge: IN_PROGRESS, 5/6 tasks; progress.json still lists merge-threads.sh as pending, but merge-threads.sh exists and its suite is recorded green 17/17 in the handoff — projection is stale.
- change-drt-003 director/worker agents: recorded PENDING yet shows 5/6 tasks done — status/tasks inconsistent.
- change-drt-006 bench and metrics: recorded IN_PROGRESS 0/1; the handoff reports 3/4 tasks done with RACE blocked on the very hang this child investigates — projection is stale.
- Child progress.json inherited the parent's COMPLETE evidence/certification/publication summaries ("change archived on 2026-08-30") — wrong for a fresh child; state-quality defect in the rollup.

SPEC GAP SUMMARY
- Handoff inaccuracy corrected by reading: the claim that driver-contract.sh:309 and :326 "call the live gateway" is false. Both set RESEARCH_ADV_DIR to a stub dir whose dispatch-judge.sh is a heredoc stub written at driver-contract.sh:288; run-research.sh:313 prefers RESEARCH_ADV_DIR, and RESEARCH_JUDGE_CMD would outrank even that (:372). No scenario in the suite can reach the live gateway. The real defect there is smaller: those two scenarios exercise the real build-review-packet.sh but a stub dispatch, so the packet-builder path is covered only with a stub ADV dir.
- build-review-packet.sh (919 lines, call 3 in the stage-10 chain) still embeds roughly ten python3 heredoc programs — the same construct that hung indefinitely in six other scripts on this host. It was not extracted. This is a structural suspect, not a confirmed cause: a standalone full deep run completed through it today in under 60 s.
- C-01 distribution drift in the working tree: dist/plugins/{claude,codex}/.../export-package.sh still carries the old heredoc form and dist lacks export-package.py, while the source was extracted; check-research-package.sh drift is whitespace-only. npm run check:distribution compares generated output, so this will fail certification until the distribution is regenerated.
- derive_status_jq (check-research-package.sh:163) loads three whole JSON documents into shell variables and pipes them through jq; measured 0.07 s on a small package and >25 s (unreturned) on a 30-claim package per the handoff. Plausible contributor, not confirmed as the stage-10 blocker.

BUILD HEALTH
- driver-contract.sh (full suite, timeout 240): PASS — 128/128, ~2 min, today.
- driver-contract.sh --scenario full-run: first attempt today HUNG (timeout 90 → exit 124, zero assertions printed); identical second attempt PASS 12/12.
- Standalone driver full deep run with the suite's exact fixture environment (stage-runner, fixture judge, temp output dir): PASS — all ten stages, review PASS recorded, under 60 s.
- Net: build health is green-but-unstable. The defect is intermittent; one hang and two clean passes in three observations today, all on the same uncommitted tree.
- Other suites (per handoff, not re-run): merge-threads 17/17, report-assembly 14/14, thread-contracts 14/14, dispatch-smoke 14/14; scoring-graph.sh stalls on its semantic-blocked scenario (points LITER_LLM_BASE_URL at a closed port; reproduces on the pre-edit detect-contradictions.sh → pre-existing).
- Test coverage of the stage-10 review path: strong — review-clean-verified, review-critical, review-warning, review-unavailable, review-without-verify, review-real-dispatch, and frontmatter-body scenarios all pass.

CONSTRAINT CHECK
- C-01 (generated artifacts in sync): VIOLATED in the working tree for the deep-research skill (dist drift above). Not yet at a commit boundary, so not yet a certification violation, but it must be reconciled before any phase commit.
- C-02 (no committed secrets): no violations observed in the inspected files (not an exhaustive audit).
- AGENTS.md local-only validation: followed — all checks above ran locally; no CI used.
- Implementation-first policy: no tests were modified; the only runs were the acceptance suite and one bounded reproduction, after reading the code.

GOAL PROGRESS
- Goal 1 (identify blocking command with trace): PARTIAL — chain enumerated, blocker not isolated; intermittency established.
- Goal 2 (pre-existing vs introduced): NOT MET — conflicting evidence; scoring-graph stall is pre-existing, the stage-10 intermittent hang is undated.
- Goal 3 (fix proven by full driver-contract count): NOT MET — no fix; the suite passes unfixed, so the proof standard needs redefinition for flakiness.
- Goal 4 (adversarial review of diagnosis before fix): NOT MET — no diagnosis yet; this remains the gate before any fix is written.

RISKS / OPEN QUESTIONS FOR ANALYZE-OR-PLAN
1. Intermittency changes the method: reading plus one-shot reproductions disproved six theories last session; the next step should capture the hung process state (e.g. macOS sample/spindump of the driver process tree, or ps of the pipeline) the next time the stall occurs, instead of testing theories serially. A stall with zero assertions printed means the driver invocation itself never returned — which child process holds it is knowable only from a live capture.
2. The acceptance criterion "driver-contract passes 128" cannot certify a flaky fix; define a repeat-count (e.g. 5 consecutive full suites) before planning the fix.
3. Distribution regeneration (C-01) is owed regardless of the hang outcome; it also changes which export-package.sh the installed plugins run (dist still has the heredoc form).
4. scoring-graph.sh semantic-blocked stall is pre-existing and out of this child's goals, but it will block any "all suites green" certification claim — decide whether it joins scope.

ASSESSMENT COMPLETE
