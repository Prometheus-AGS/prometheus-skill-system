# Decision Log — stage-10-hang-investigation (kbd-analyze)

Phase: deep-research-onyx-parity › stage-10-hang-investigation
Stage: analyze (step 4 of 7)
Date: 2026-09-11
Producer: kimi-for-coding/k3 (opencode)

## Decisions

### D1 — cand-1-proof-standard: ADOPT (build)
Goal 3's proof gate is re-defined as N >= 10 consecutive clean `--scenario full-run`
passes under driver-contract, replacing the single 128/128 green run that certifies
nothing for an intermittent defect. Rationale: at the observed ~1/3 failure rate,
P(10 consecutive clean passes | defect still present) = (2/3)^10 ~= 1.7%. No library
adoption; the mechanism is a documented gate rule plus a repeat-run harness.
Evidence: analysis.md Tier-4 sources (flaky-test elimination literature).

### D2 — cand-2-diagnosis-harness: ADOPT (build)
An instrumented repeat-run harness looping `--scenario full-run` up to K times with
per-call tracing is required to isolate the exact blocked command in the stage-10
chain — Goal 1's missing evidence. No external tool adoption; extends the existing
driver-contract.sh harness in-repo.

### D3 — cand-3-heredoc-extraction: CONDITIONAL (build, gated on D2)
Extract the ~10 python3 heredocs embedded in build-review-packet.sh ONLY IF D2's
harness confirms the hang site is inside that script. The ~64 KB OS pipe-buffer
deadlock mechanism (Python subprocess docs; SO q/68327635) explains the observed
intermittency, and the same extraction pattern has already been proven in six
sibling scripts in this repo. Pre-emptive extraction is rejected: Goal 4 requires
a reviewed diagnosis before any fix is written.

## Contested choices
None. All candidates are build/method decisions; no library or stack adoption was
at stake, so the score-gap < 15% escalation rule does not apply. pmpo-elicit /
AskUserQuestion escalation: not invoked.

## Open questions (carried to Spec)
1. Measured failure rate needed from D2 runs to size N rigorously.
2. Whether the ~64 KB pipe-buffer threshold is actually crossed by
   build-review-packet.sh heredoc output at stage-10 input sizes (harness must
   capture output volume at the hang site).
3. If the hang isolates outside build-review-packet.sh, D3 is void and a new fix
   candidate is required.

## 2026-09-12 — Analyze resumed; adversarial review round 1 BLOCK; corrections applied
The 2026-09-11 analyze session died during the adversarial vet (packet built, no
findings). Resume found the packet had been built against the PARENT phase dir
(embedding the 2026-09-08 Onyx-parity analysis) with producer_model "unknown";
it was rebuilt against the child phase with KBD_PRODUCER_MODEL=k3 before dispatch.
Round 1 (judge MiniMax-M3 vs producer k3, verified-distinct, sycophancy pass):
BLOCK — CRITICAL: FINDING B was false, the library-candidates schema EXISTS at
skills/process/kbd-process-orchestrator/references/schemas/library-candidates.schema.json;
library-candidates.json was rewritten and passes jsonschema validation. Five
WARNINGs also addressed in analysis.md (Goal-1 NOT MET; bisect needs committed
history -> Plan prerequisite; handoff-in disproof re-anchored to verified
file:line citations; Candidate-1 independence caveat + pre-run cleanup; Tier 1-3
negative results enumerated). Round 2 dispatched after revision.

## 2026-09-12 — Spec stage: two changes authored; review round 1 BLOCK; revisions applied
Changes: change-drt-008-hang-capture-harness (D2 harness + D1 instrumentation, goals 1–2) and
change-drt-009-stage10-hang-fix-and-certification (Goal-4 gate + D3 conditional fix + D1
certification, goals 3–4), dependency drt-008 → drt-009. Backend native-kbd per phase precedent.
Child scope widened at spec time to add skills/research/deep-research/** and
skills/process/adversarial-review/** (the stage-10 chain's packet builder IS the adversarial-review
skill's build-review-packet.sh — the 919-line heredoc suspect).
Vet wiring: liter-llm gateway up (401 auth) but MiniMax-M3 judge output failed findings-JSON shape
validation twice (exit 2, unusable output, raw not retained) → harness-native fresh-context
subagent per the exit-3 protocol and the standing user sanction; same-model-collision recorded.
ALSO discovered: build-review-packet.sh --target spec cannot resolve a CHILD phase
(CHANGES_ROOT = dirname(dirname(child))/ lands on phases/<parent>/changes); bridged in-scope via
phase-local symlinked changes dir .kbd-orchestrator/phases/deep-research-onyx-parity/changes/ →
carried as an Open Question for a proper reviewed fix.
Round 1 (harness-native): BLOCK — 1 CRITICAL + 6 WARNINGs + 1 SUGGESTION, all verified legitimate
before revision (timeout-attribution and 9-companion count re-verified against the tree). Revisions:
drt-009 task-1 gate now requires machine evidence (distinct-judge findings + gate PASS record;
same-model fallback explicitly does NOT satisfy Goal 4; unreachable judge ⇒ BLOCKED, no fix);
elsewhere-branch out-of-Scope escalation path added; task 2/3 verify strings made failable
(branch: line, machine-countable "| run" rows ≥ 10, FIX-CERTIFICATION grep moved to task 3);
"matching the suite's scenario timeout" corrected to external-timeout attribution; extraction
precedent re-cited as the verifiable nine .sh/.py companions + drt-006's five fixed scripts;
C-01 reconciliation owner named (phase-close distribution change to be specced at plan);
build_required-semantics warning dispositioned as intentional in drt-009 Open Questions;
drt-008 gains orphan-process and dating-method (method: line) verify hooks. Round 2 dispatched.

## 2026-09-12 — Spec adversarial review round 2: PASS (gateway, verified-distinct); handoff written
Third gateway attempt succeeded where two earlier dispatches failed findings-JSON shape validation
(the shape failures were transient — the same intermittency theme the child investigates).
Round 2: judge MiniMax-M3 vs producer glm-5.3 via rest-gateway, cross_model_check
verified-distinct, ZERO findings, verdict PASS; sycophancy screen PASS (score 0.0, strictness
strict). Caveat recorded: the judge's checked_classes is a literal "..." placeholder, not the
mandated per-class enumeration — the PASS stands on the verified-distinct fresh-context review,
and substantive class coverage evidence exists in round 1's eight findings and their revisions;
noted here rather than laundered. Producer-model label corrected this stage: prior stages
recorded k3 from the handoff-in assumption; this session's harness names glm-5.3, and the
distinct-judge property holds either way. Stage handoff written: nextStage plan.

## 2026-09-12 — Plan: 3 changes ordered; review round 1 BLOCK (drt-010 cut), round 2 BLOCK accepted at cap
Plan written: drt-008 (capture harness, goals 1-2) -> drt-009 (review-gated fix + N-run certification,
goals 3-4) -> drt-011 (C-01 distribution regen; authored at this stage with full change structures).
Strictly serial; no parallelism. Sycophancy pre-screen on the draft: clean (0.0, standard,
pmpo_plan_phase). Review round 1 (MiniMax-M3 vs glm-5.3, verified-distinct, gate PASS): BLOCK —
CRITICAL: change-drt-010-child-phase-packet-resolution was scope smuggling (advances none of the
child's 4 goals); CUT to a standalone post-child change (same-file-after-009 + dating-purity
rationale preserved in the cut record; symlink bridge stays until it lands). Also added streak
semantics, method: enum, machine-gate one-liner. Round 2: BLOCK (3 CRITICAL + 4 WARNING + 3
SUGGESTION) — second and final round, ACCEPTED per the artifact-mode gate with an Unresolved
review findings section appended (CRITICALs verbatim + dispositions): (1) N-formula undefined at
p=0 -> Execute sizes N from the Clopper-Pearson upper bound when 0/K; (2) C-01 ownership claim —
already recorded in drt-008/009 spec Constraints, invisible to a plan-mode packet by construction;
(3) Goal-1 zero-hang path -> BLOCKED + one K-doubling escalation, never an invented diagnosis.
WARNINGs carried to handoff (K-loop stop-vs-full ambiguity resolved for Execute: full K when no
hang; capture then continue when hung; escalated-branch destination defined in drt-009 spec;
reactive-only correlation detection accepted as the honest floor). Waypoint refreshed:
change=change-drt-008-hang-capture-harness, exactNextCommand=/kbd-execute, revision 1106;
current-waypoint.md rewritten (was wholly stale — described the closed
openspec-mirror-drift-cleanup phase).

## 2026-09-13 — Execute: drt-008 DONE+archived (4 review rounds); the hang is GONE (0/38); drt-009 task 1 BLOCKED on gateway
Setup: 3 changes + 7 tasks registered via prometheus kbd (child progress 0/3 → canonical); execution.md
written (SELF/opencode driving kbd-apply); typed stage enter execute recorded. LESSON: manual
current-waypoint.json field edits are superseded by hook projection refreshes — only typed CLI + the
kbd-apply task loop update projections durably.

change-drt-008-hang-capture-harness: all 3 tasks done; QA gate (KBD-wired refine-validate per
constraints.md, refinement_log.md written) 5/5 PASS; adversarial diff review took 4 rounds:
round-1 harness-native review first exposed a PACKET DEFECT (scoped diff came out empty because the
change's files were untracked → builder fell back to git show HEAD = the unrelated cpc-001/002
commit; also found+removed a 46-hour-stale .git/index.lock). Real rounds then found: teardown killed
only direct children (grandchildren could orphan) + per-run tables not committed (CRITICALs, fixed);
claimed-but-unimplemented perturbation marking + wrong N arithmetic (53→55) + start_utc recording
completion time (fixed); replace-vs-merge regression + 3 more (fixed). Round 4: PASS (0 CRITICAL,
2 WARNINGs logged — sort -un breaks root-first invariant; per-tick replace-vs-accumulate — plus
1 SUGGESTION). Gateway unusable for the final round (exit 3, completions dead, /models 401→000);
harness-native same-model used per designed fallback, recorded. Sycophancy gate PASS 0.0. ARCHIVED
to 2026-09-13-change-drt-008-hang-capture-harness.

CAMPAIGN RESULT (the substantive finding): ZERO hangs in 38 runs — K=12 campaign, K=24 plan-mandated
doubling, and an exact-state probe (full 128/128 suite → immediate full-run, the original hang's
sequence) — all clean 12/12, 8-17 s/run, under load 50-179. Goal 1 BLOCKED (bounded 0/38, no
invented diagnosis); Goal 2 BLOCKED (method: uncommitted-only boundary); leading UNVERIFIED
interpretations recorded (incidentally fixed by drt-006's same-day extractions — check-research-package
is invoked by the scenario and hung pre-extraction; or unreproduced environmental precondition).
"Zero assertions printed" reinterpreted: buffered output lost on external-timeout kill — a late
hang is equally consistent. N for certification = 55 (Clopper-Pearson 3/38).

change-drt-009: task 1 (Goal-4 diagnosis review) — DIAGNOSIS.md written (the honest no-reproduction
finding: no fix branch selectable; certification proceeds as statistical gate); decision-mode packet
built (8.4 KB); liter-llm gateway DOWN at connection level (000) → per spec, same-model fallback
does NOT satisfy Goal 4 → task 1 BLOCKED with reason, doomed 90-min retry dispatch killed. Tasks
2-3 do not start while blocked. change-drt-011 holds (avoids double dist regen if 009 later edits).
RESUME: re-run drt-009 task 1 when the gateway serves MiniMax-M3 — packet ready at
review/diagnosis/packet.json.

## 2026-09-12 — Analyze adversarial review round 2: PASS; handoff written
Round 2 (judge MiniMax-M3 vs producer k3, verified-distinct, sycophancy screen
pass): PASS — 0 CRITICAL, 0 WARNING, 4 SUGGESTIONs (chain-notation ambiguity;
Popen-vs-heredoc evidence nuance; heredoc count unverified — verified this
session: 9 `<<'PY'` blocks in the 919-line build-review-packet.sh; cand/build_required
duplication relationship). Stage handoff written: nextStage spec. Incident: a
concurrent process overwrote findings.json at 07:17 with a harness-native
same-model review of the stale parent packet; archived as
findings.concurrent-harness-native-0717.json and superseded by the clean round-2
dispatch. Runtime exactNextCommand still projects /kbd-assess (stale projection).

## 2026-09-13 — Execute resumed after service recovery; drt-009 DONE; scope amendment recorded
Service fix: liter-llm gateway (ai.prometheus.liter-llm-api) was wedged LISTEN-but-000; restarted via
launchctl kickstart → healthy. surreal-memory listening (earlier timeouts transient). Diagnosis review
then ran on the gateway: 2 rounds, judge MiniMax-M3 vs producer glm-5.3, VERIFIED-DISTINCT (Goal-4
machine check satisfied), both rounds BLOCK; accepted at the spec's 2-round cap with the Unresolved
section (goals-delivery objection dispositioned against the plan's terminal disposition). Substantive
review wins: drt-006-timeline interpretation weakened; N corrected to 60 (exact Clopper-Pearson);
falsifier broadened to any-recurrence; buffering-defeat alternative ADOPTED as harness --xtrace
(assertion counts unreliable under it — fixture captures child stderr into assertion files, 3/12
observed; usage note + code comment record it). SCOPE AMENDMENT recorded for drt-009: tests/hang/
run-hang-capture.sh admitted (instrumentation --xtrace + RC=124 recording fix; no standard-mode
clean-run behavior change; 60/60 certification runs standard mode). Certification: 60/60 consecutive
clean full-run passes (load 211-493!) + full suite 128/128 exit 0 → FIX-CERTIFICATION.md, branch:
none. Diff review rounds 1-2 BLOCK (branch-enum spec/verification drift; evidence files added to
diff scope; comment contradiction; decision_fields parse fixed — assumptions now 4 items), round 3
PASS (0 CRITICAL, 5 WARNINGs remediated with traceability). Task-1's earlier BLOCKED line above is
historical: unblocked same day after the service restart.

## 2026-09-13 — Execute COMPLETE: 3/3 implemented (drt-008/009 archived); drt-011 certification BLOCKED (packet-scale dispute)
Judge infrastructure saga: liter-llm :4000 wedged (LISTEN-but-000) → kickstart fixed. User-directed
switch to openai-proxy :8181 for reviews: gpt-5.4 advertised-but-rejected by the ChatGPT-plan Codex
backend (verified live); gpt-5.4-mini/codex-mini rejected; gpt-5.5 + gpt-5.6-* WORK. Wired
[[models]] gpt-5.5 in liter-llm-proxy.toml + judge role repointed in ~/.prometheus/kbd/models.toml;
liter-llm restart required to reload the model table. Result: 400K-context local judge, zero
timeouts across all subsequent rounds.
drt-011 (dist regen): generator ×2 byte-identical (full-tree hash cbc3725f…, all file types —
supersedes the partial-type bda1fc97…); 3 npm checks PASS; 405-file accounting committed as
dist-manifest.txt; child surface verified in BOTH plugin trees; assessment's export-package.py
drift fixed. Review rounds 1-4 (MiniMax then gpt-5.5, verified-distinct): fixed files.txt packet
scope, hash-baseline mislabel, codex sample paths, stale hash; TERMINAL dispute on "all 405 file
contents must be in the judge packet" (~700KB — beyond any judge context; outputs staged in git:
405 files 24618+/4539-; staged index IS the commit set in this workflow). Recorder-semantics
warning (harness exit 0 on hangs) recorded as future harness change. Per no-laundering:
certification dimension BLOCKED with reason, change NOT archived, implementation 3/3 COMPLETE.
Known generator-hygiene finding recorded: tests/hang/captures*/ campaign dirs mirror into the
plugin payload (no exclusion rules — future change).
Stage totals: drt-008 archived (4 rounds), drt-009 archived (3 rounds), drt-011
implemented+blocked-at-certification. SUBSTANTIVE: hang gone 0/98 total runs; N=60 certification
streak + 128/128 suite; goals 1/2 blocked-bounded by honest evidence, goal 3 certified, goal 4
distinct-judge reviewed (MiniMax for the diagnosis gate, gpt-5.5 for diff rounds).

## 2026-09-13 — Child close-out: G2 MET by archaeology (operator-challenged); drt-011 archived under operator waiver
Operator challenged the NOT-MET grading; re-investigation: G2 MET (introduced-this-phase proven via 0
prior commits on every implicated file; first appearance 09-09 ~20:26 assessment-dated; never
code-remediated — no chain edit between hang and 98 clean runs; same bytes hung once/passed 98;
"incidentally fixed by drt-006" DISPROVEN, extractions predate the hang). G1 closed as proven negative
(no surviving hung-run artifact; job checkpoints are 09-08 threaded tests). Completion 75%. Dist
resynced after addenda (4 checks PASS). drt-011 certification resolved BY OPERATOR WAIVER (instruction:
"Continue back to the parent phase and complete that after archiving this child work") — archived.
Child now fully closed: 3/3 implemented, 3/3 archived, certification COMPLETE.
