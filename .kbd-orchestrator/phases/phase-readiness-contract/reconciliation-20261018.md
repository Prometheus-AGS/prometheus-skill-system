# Reconciliation — 2026-10-18 (reporting only; no repair cycle)

Scope: operator-directed reconciliation of three discrepancies plus separation of
adopted v3 obligations from #184 comment follow-ups. All code, reports, passed
checks, and canonical history are preserved. Nothing herein rewrites history,
reruns the suite, or implements anything.

## Discrepancy 1 — Historical closure: GAP (negative assurance absent)

Code facts (`shared/scripts/kbd-phase-close-eval.sh`, verified by inspection):

- Lines 78–83: the idempotence scan executes FIRST, before evidence validation
  (line 86 presence/JSON, line 92 identity binding, line 109 source freshness,
  line 121 entry-level oracle).
- The scan matches only `PHASE-CLOSED phase=<p> verdict=PASS` — phase name only.
  No run binding, no contract-digest binding, no evidence-freshness check.

Consequence: a same-phase historical PASS under a DIFFERENT runId, or with
stale/tampered/missing evidence, returns `ALREADY-CLOSED` exit 0. The required
assurance "a same-phase historical PASS must not authorize a different run or
stale candidate" does NOT hold in current code.

What existing evidence DOES hold (receipts remain valid):

- RC-01-close: valid complete evidence closed the phase through the canonical
  writer — decision `phase-closure-phase-readiness-contract-2b18d0f8`
  (`PHASE-CLOSED ... verdict=PASS evidence=sha256:2b18d0f8… contract=982855ed…
  run=phase-learning-deploy-and-debt-20261018T020551Z`).
- RC-01-close-idempotent: an UNCHANGED valid replay returned ALREADY-CLOSED with
  decisions +1 total and task state byte-identical. This is exactly the
  "unchanged valid replay remains idempotent" case, and it stays idempotent
  under the proposed fix because its evidence was unchanged and valid.
- Line 147 shows the closure summary already embeds `run=`, `contract=`, and
  `evidence=sha256:` — binding data already exists; no schema change needed.

Smallest proposed patch (NOT applied): move the idempotence scan after the
entry-level oracle validation and bind it:

```bash
select(.summary | contains("PHASE-CLOSED phase=" + $p + " verdict=PASS")
       and contains("run=" + $S_RUN) and contains("contract=" + $C_DIGEST))
```

~8 lines in one file. Validation order: presence → JSON → identity → adoption
digest → freshness → entry oracles → THEN idempotent no-op (else refuse).
New fixture check (mapped under RC-03, no new RC id): status fixture containing
a historical PHASE-CLOSED plus tampered-digest evidence must refuse, not
ALREADY-CLOSED. Affected RC IDs: RC-01-close, RC-01-close-idempotent (receipts
remain valid), RC-03 (one mapped sub-case).

## Discrepancy 2 — Budget detection: truthful statement (predicate does not exist)

Verified by grep across `kbd-phase-close-eval.sh`, `kbd-phase-readiness-check.sh`,
`stage-gate.sh`:

- The evaluator contains NO reference to budgets, timeMinutes, toolCalls,
  repairAttempts, or noProgress. No budget predicate exists.
- The checker (lines 74–75) validates only that budget keys are DECLARED with
  minimum values and that tokenFallback exists when tokenBudget=UNKNOWN. It
  never reads consumption.
- stage-gate.sh: no budget predicate.

Exactly which behavior was exercised by RC-02-budget: an evidence fixture
CONTAINING a BLOCKED entry produced `PHASE-CLOSURE-REFUSED verdict=BLOCKED`
with zero closure and zero continuation. That demonstrates "a result already
marked BLOCKED cannot produce a success closure" — refusal of a BLOCKED-marked
result. It does NOT demonstrate detection of exhausted cumulative consumption
against declared limits. Additionally real (but human-governed, not a machine
predicate): run 3's actual budget exhaustion stopped with checkpoint-blocked.md
+ BLOCKED, and the recorded extension preserved cumulative consumption.

Deterministic termination IS demonstrated (no auto-continue; BLOCKED never
auto-PASS). "Observable consumption" currently lives only in hand-written
checkpoint files; there is no declared-limit-vs-measured-consumption control.

Smallest proposed patch (NOT applied): evaluator reads contract `.budgets` +
`.budgets.extensions[]` (each extension field declaring add/replace/unchanged —
see discrepancy 3) and a `budgetConsumption` object in the evidence header
(per-field measured vs estimate flags); closure refused (BLOCKED) when any
MEASURED dimension is consumed >= declared. The results writer already knows
repairs/no-progress (measured) and marks time/tool as estimates. One fixture
check mapped under RC-02: exhausted measured repair dimension -> refusal.
Standing disclosure: with token telemetry UNKNOWN, this enforces declared
limits against declared consumption records, not an independent meter —
consistent with the adopted tokenFallback clause.

## Discrepancy 3 — Authorization accounting: RESOLVED (my report table was wrong)

The approved extension text: "+90 minutes, +90 tool calls, +2 repair attempts,
no-progress limit 2." Three fields are additive; no-progress was restated at its
existing absolute value. No separate approval incrementing no-progress exists,
and none is claimed. Contract v3's `budgets.extensions[0]` correctly recorded
`noProgressLimit: 2`; my chat report's "2 + 2 = 4" was the error.

Corrected cumulative accounting (measured vs estimate flagged):

| Dimension       | Original | Extension op      | Ceiling | Consumed        | Remaining       |
|-----------------|----------|-------------------|---------|-----------------|-----------------|
| timeMinutes     | 90       | +90 (additive)    | 180     | ~145 (estimate) | ~35 (estimate)  |
| toolCalls       | 180      | +90 (additive)    | 270     | ~137 (estimate) | ~133 (estimate) |
| repairAttempts  | 2        | +2 (additive)     | 4       | 4 (measured)    | 0               |
| noProgressLimit | 2        | unchanged (absolute) | 2    | 0 (measured)    | 2               |

Repair budget stands exhausted (4/4 measured) — hence reconciliation/reporting
only. Operator's new lesson acknowledged (amendment fields must declare
add/replace/unchanged with approval provenance); operator is recording it.

## Closeout qualification (truthful; passed checks and closure preserved)

- Retained: scoped PASS for all 30 evidenced checks; the historical
  PHASE-CLOSED decision stands; the closed-dispatch refusal stands.
- Qualified (BLOCKED-class, two specific assurances only):
  - Q1: historical-PASS negative assurance (discrepancy 1) — not currently
    enforced; smallest patch proposed.
  - Q2: budget-exhaustion detection predicate (discrepancy 2) — does not exist;
    exercised behavior truthfully restated; smallest patch proposed.
- This is NOT a claim that the product failed. Everything else remains PASS.

## #184 separation (issue read in full: body + single 2026-10-18 comment)

Current adopted v3 obligations (this phase) — status:

1. Comment check 1 (validate identity/digest/freshness BEFORE already-closed
   success) = Q1. GAP; smallest patch above.
2. Comment check 2 (budget control detects exhausted cumulative usage) = Q2.
   Predicate absent; smallest patch above.
3. Comment check 3 (B1 mutual identity mapped to the real gate; B2 per-scenario
   argv/exit/observed/status + recomputed freshness; canonical decision-record
   closure) — HELD by existing receipts: RC-01-transition, RC-02-quality-empty/
   oracle, RC-04, RC-05b (gate identity path), boundary-results.json
   sourceDigests + evaluator freshness (B2), closure via `prometheus kbd
   decision record` (canonical mechanism, not a substitute). Standing honest
   limitation: gate handoff-identity validates format; digest comparison lives
   in the checker — disclosed since v3.
4. Distribution checks reported separately from report-schema/hash checks —
   done (dist regenerated, `--check` exit 0).
5. Accounting precision paragraph — discrepancy 3, resolved above.

Future follow-up items (NOT v3 scope; #184 stays OPEN until verified):

- Comment D1–D6 finite delta corpus (claim/configuration matrix, oracle
  adequacy incl. fixed-evaluator PASS/FAIL/BLOCKED/malformed/stale fixtures,
  evidence identity/invalidation across restart, failure classes, feasible
  budgets with envelope math + amendment-field operations, closure/distribution
  regression) — candidate finite successor phase.
- Issue-body acceptance items beyond v3: task-envelope round-trip with the
  configured route, small-model envelope, interrupt/resume journal replay,
  optional-improvement injection, legacy-ledger migration semantics.
- Sequenced per the comment's ordered boundary: #185 host-enforcement pilot,
  mini #59/#60, Phase Exit skill qualification, Know Decision extraction —
  all later, none authorized now.

## Allowance request (only if the operator wants Q1/Q2 fixed in this phase)

- Fix A (Q1): ~15 min, ~8 tool calls, +1 repair attempt.
- Fix B (Q2): ~25 min, ~12 tool calls, +1 repair attempt.
- Combined: +40 minutes, +20 tool calls, +2 repair attempts; noProgressLimit
  remains 2 (absolute, unchanged). Verification reruns only affected checks;
  all 30 existing receipts and the closure decision remain reusable evidence.
- Alternative at zero repair budget: defer Q1/Q2 to the successor D-corpus
  (D2/D5 explicitly cover both) and let this phase's closure stand with the
  Q1/Q2 qualifications recorded here.
