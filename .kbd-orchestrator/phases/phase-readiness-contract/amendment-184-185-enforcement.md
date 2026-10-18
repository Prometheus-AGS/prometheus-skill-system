# AMENDMENT PLAN: phase-readiness-contract — bounded #184/#185 enforcement

Status: **SAVED, NOT IMPLEMENTED — BLOCKED on budget (see §7).**
Authorization: operator message 2026-10-18 + `task-6/KBD-BOUNDED-AMENDMENT-PROMPT.txt` (bounded scope amendment; explicit, version-scoped; subsequent substantive changes need separate adoption).
Supersedes nothing; amends `adopted-contract.json` (v2, digest `248c84d3…`) → v3 via canonical writer with `--supersedes contract-adoption-phase-readiness-contract-248c84d3` as the FIRST implementation step after budget approval. No hand-edits to journal/signatures/waypoint/counters. Old COMPLETE/8-of-8 is task completion, not acceptance.

## 1. Preserved (untouched by this amendment)

- Worktree, branch `codex/kbd-phase-readiness`, all T1–T8 production diffs.
- RC-01–RC-07 identities, historical adoption decisions (v1 superseded, v2 active), all three signed gate receipts, `checkpoint-blocked.md`, the original (malformed) `boundary-results.json` as historical evidence — old failures are corrected by NEW valid runs, never rewritten as PASS.
- Frozen exclusions: no full #184/#185, no global tool fence, no new frameworks, no mini/UAR work, no push/merge/publication, no credential/provider changes, Kimi subscription only (Extra Usage off).

## 2. In-scope corrections (existing scope, already authorized)

| # | Defect | Fix | State |
|---|---|---|---|
| C1 | `boundary-results.json` invalid JSON (writer `jq -R .` per-line) | `jq -Rs .` slurp | **Applied** (test script, results section) |
| C2 | RC-06 record grep `^ADOPTED ` fails on idempotent rerun | accept `ADOPTED`/`ALREADY-ADOPTED` + audit-lineage oracle (≥2 decisions, supersedes=1) | **Applied** |
| C3 | RC-06-stale fixture wrong-failure (v1 copy is v2 post-amendment; silent jq fallback; missing tasks.json ⇒ coverage BLOCKED instead of REJECT-STALE) | explicit two-decision fixture (old superseded by new) + synthesized old-digest contract + tasks.json present | Pending (designed, not applied) |
| C4 | RC-07-parity BLOCKED: submodules uninitialized | `git submodule update --init --recursive` in this worktree, re-run packet cmd 1–3 | Pending |

## 3. New production behavior B1 — Plan→Execute transition enforcement (#186 completion, #184 foundation)

Real entry: `kbd_stage_gate execute` in `skills/process/kbd-process-orchestrator/shared/lib/stage-gate.sh` (the seam every orchestrator dispatch passes; RC-05 already proves it). Changes:

1. stage-gate.sh: Plan→Execute transition calls the FULL readiness validation (`shared/scripts/kbd-phase-readiness-check.sh`), not only handoff-identity. Validation failure ⇒ boundary does not complete, zero dispatch, exit≠0 with remediation.
2. kbd-phase-readiness-check.sh: add contract-quality gates (today it checks structure only): ≥1 useful user operation (`releaseClaim.userOperation` non-trivial); scenario ids non-empty AND unique; every scenario has decidable oracle + real productionEntry + fixtures + negativeControl; exclusions/budgets/requiredSafetyChecks non-empty; boundaryPacket bounded (commands + stopConditions); reject empty arrays, all-skipped, oracle-less, "tests pass"-style oracles. Mutual match: project/run/phase/adopted digest/handoff identity.
3. Honesty boundary: investigate whether the compiled `prometheus kbd` lifecycle itself invokes any gate at transition/dispatch. If it does not, the shell gate remains a convention, NOT an un-bypassable runtime guarantee — that residual is recorded as an explicit limitation/BLOCKED line in the final report, never claimed as enforced.

## 4. New production behavior B2 — terminal evaluator at phase close/continue boundary (#184 subset)

New `shared/scripts/kbd-phase-close-eval.sh` (+x), real entry consumed at the phase's close/continue-dispatch boundary:

1. Consumes the phase's `boundary-results.json` ONLY — never task counters, `implementationComplete`, or READY. Validates evidence binding per entry: JSON validity; identity (project/run/phase) vs canonical status; contract digest vs latest non-superseded adoption; per-scenario argv/exit/observed/status present; NOT_EXERCISED ≠ PASS; freshness — recompute digests of the production files each scenario exercised (script paths recorded per scenario); unchanged unrelated files invalidate nothing.
2. Verdicts: all required scenarios PASS + safety checks honored ⇒ close phase PASS via canonical writer (`prometheus kbd decision record` closure decision), stop dispatch for THIS phase only, zero auto tasks/next-phase. FAIL(product oracle unmet after valid execution) or BLOCKED(missing/stale/invalid evidence, validator/env unavailable, budget exhausted) ⇒ canonical termination/checkpoint record, stop auto-continue, never a success closure. Re-evaluation idempotent (ALREADY-CLOSED no-op).
3. Budget truth: evaluator reads declared time/tool-call bounds (token UNKNOWN ⇒ fallback bounds); exhaustion never auto-PASS.

## 5. Demonstration mapping (reuses RC-01..07 — NO RC-08)

| Maps to | Demonstration (real entry + real canonical collaborator + observable counterexample; negatives in isolated scratch HOME/XDG/project roots) |
|---|---|
| RC-01 | valid plan passes real `kbd_stage_gate execute` transition; valid complete evidence closes PASS via evaluator; double-eval idempotent, zero new decisions/tasks |
| RC-02 | checker rejects contract with empty scenarioMatrix / missing oracle at real gate, zero dispatch; budget-exhaustion fixture ⇒ evaluator BLOCKED, zero continued dispatch |
| RC-03 | missing/stale/invalid evidence (tampered digest, invalid JSON, unexecuted entries) ⇒ evaluator refuses closure |
| RC-04 | wrong project/run/digest at real gate ⇒ REJECT-IDENTITY, zero dispatch |
| RC-05 | malformed handoff at real gate (existing); valid FAIL result ⇒ evaluator does NOT close PASS |
| RC-06 | v3 amendment (this plan): old records preserved, new digest accepted, old-digest evidence rejected, adoption without `--supersedes` ineffective (exit 4) |
| RC-07 | generated dist + scratch-installed payload semantics consistent (post-submodule-init) |

## 6. Verification (single bounded run, after coherent production changes only)

Re-run only failed/invalidated items: packet cmd 1 (`prometheus kbd gate run --kind integration --scope phase-readiness-contract -- bash shared/scripts/tests/test-kbd-readiness-gate.sh`, extended with §5 sub-cases), cmd 2 (validate:strict), cmd 3 (distribution --check). Produces NEW valid `boundary-results.json`; old one preserved in git/gate history. PASS requires every required scenario valid PASS AND actual canonical closeout succeeds.

## 7. Budget accounting (no reset) and extension request

| Dimension | Adopted | Consumed at checkpoint 2026-10-18T03:00Z | Consumed since (reads, 2 edits, blocker record) | Remaining |
|---|---|---|---|---|
| timeMinutes | 90 | ~85 | ~10 | **~0 — EXHAUSTED** |
| toolCalls | 180 | ~75 | ~15 | ~90 |
| repairAttempts | 2 | 2 | 0 | **0 — EXHAUSTED** |
| noProgressLimit | 2 | 0 | 0 | 2 |

Stop condition met (repairAttempts; timeMinutes). Canonical blocker `phase-readiness-contract-amendment-budget-gate` recorded. **Requested finite extension (amendment scope only): +90 min, +90 tool calls, +2 repair attempts, no-progress 2.** No self-renewal; implementation starts only on explicit operator approval.
