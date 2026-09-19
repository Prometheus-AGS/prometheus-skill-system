# DIAGNOSIS — the stage-10 intermittent hang

Change: change-drt-009-stage10-hang-fix-and-certification (task 1, Goal-4 gate)
Evidence source: HANG-CAPTURE.md (change-drt-008, archived 2026-09-13) + assessment + drt-006 verification
Producer: glm-5.3 (opencode) · Date: 2026-09-13 · Revision 3 (rounds 1–2 findings addressed; 2-round cap reached — see Unresolved review findings)

## Decision

**Treat the stage-10 hang as not-reproducible-on-current-tree: no product fix
is applied; Goal 3 is satisfied by a statistical certification gate (N = 60
consecutive clean full-run passes + one full suite pass), with the capture
harness upgraded with an --xtrace diagnostic mode (traces to a per-run
file, naming the last executed command of a hang) retained as a standing
tripwire; certification itself runs in standard mode because the fixture
environment captures child stderr into assertion files, making counts
unreliable under xtrace (observed 3/12 — recorded, not hidden).**

## Assumptions

- **(1) The campaign certifies the CURRENT tree, not the 2026-09-09 tree.** The
   original tree no longer exists to test (it has since been edited); the
   certification claim is scoped to what future runs will execute. The
   2026-09-09 tree's fitness is unknowable and is not claimed.
- **(2) Environmental equivalence is unprovable and not assumed.** Load was
   higher during the campaign (50–179) than the unrecorded observation
   conditions; the exact-state probe reproduced the observation's invocation
   sequence (full 128/128 suite → immediate full-run). If the original hang
   was environmental-only, no tree certification can be affected by it; if it
   was tree-code, 38 clean runs (plus 60 certification runs) count against it
   directly.
- **(3) Run independence holds well enough for the N-bound** via the harness's
   scoped pre-run cleanup; residual coupling would make certification
   conservative (a streak under coupling is harder, not easier).
- **(4) A 90 s external timeout bounds "hang" consistently** with the original
   observation (same threshold).

## Falsifier

**Any recurrence of the hang, by any means of observation — captured by the
harness (ps tree, sample stacks, output volumes, xtrace tail naming the last
executed command), OR merely reported (a failed suite run, an operator
observation, a CI-style timeout exit) even with no capture at all.** The
decision "not reproducible" dies on the first recurrence however it is seen;
certification is voided and the investigation reopens with whatever evidence
exists. 38 runs is a claim about 38 runs; the certification campaign adds 60
further trials (98 total).

## Evidence

- 2026-09-09 (assessment): suite 128/128 ×4 same day; ONE `--scenario
  full-run` hang (external timeout 90 → exit 124), retry PASS.
- 2026-09-13 (HANG-CAPTURE.md): campaign K=12 clean; plan-mandated K=24
  escalation clean; exact-state probe clean; one post-review smoke clean —
  **38 runs total, zero hangs** (37 campaign + 1 smoke), every run 12/12
  assertions, 8–17 s.
- The "zero assertions printed" reading that implied an early hang is
  unsound: block-buffered stdout to a file is lost on external-timeout kill,
  so a hang anywhere in the scenario fits the record.

## Interpretations — all UNSUPPORTED; none is favored

1. **Incidentally fixed by same-day drt-006 extractions:** WEAKENED. drt-006's
   verification records `check-research-package.sh` fixed (was hanging), and
   the scenario invokes it — but the same day's four 128/128 suite passes
   already included that assertion, so it was likely fixed before the hang.
   No captured stack ever existed; intra-day timeline order is not
   established.
2. **Environmental precondition not reproduced:** unverifiable after the fact;
   consistent with all observations.

The empty candidate set is not closure: it is why the tripwire stays armed,
why the harness now carries an --xtrace diagnostic mode (buffering-defeated
evidence for the next recurrence), and why certification doubles as
further trials.

## Alternatives considered

- **Commit pass + serial investigation** — rejected: nothing reproduces to
  investigate; six theories were already disproved by serial testing.
- **Unbounded campaign until first hang** — rejected: unbounded wall-clock
  against a 0/38 base rate.
- **Block the child until a recurrence** — rejected: certification with a
  standing tripwire is the risk-managed path.
- **Instrument to defeat block buffering, so the next hang IS diagnosable**
  (round-2 review's addition) — **ADOPTED**: the harness gains an `--xtrace`
  mode (SHELLOPTS=xtrace at child startup; traces flow to each bash's stderr, collected into a per-run .xtrace file — BASH_XTRACEFD deliberately NOT used: fd-3 inheritance broke fixture assertions, observed 3/12, recorded);
  the harness carries it as a standing tripwire; the certification campaign
  itself runs in standard mode (accurate counts), so a recurrence hunt — not
  the campaign — runs under --xtrace. This is the only
  alternative that could unblock Goals 1–2, and it costs nothing to carry.
- **Chosen: N-run certification + xtrace tripwire** (the Decision above).

## Cost of being wrong, and reversibility

If a hang recurs post-certification: the broadened falsifier voids the
certification on any recurrence evidence; a recurrence hunt re-runs the scenario under --xtrace and the tail names
the blocked command (buffering defeated — the 2026-09-09 diagnostic
blindness does not recur); the fix lands as a new change; re-certification follows at
the then-measured rate. Exposure is a flaky stage-10 in future deep runs
(time cost; no data corruption). Honest limit, stated: if a recurrence
happens where neither xtrace nor any observer exists, the voiding still
occurs on the report — but the diagnosis evidence would be weaker than the
instrumented case; that residual is accepted, not claimed away.

## Certification sizing

Exact Clopper-Pearson 95% upper bound for 0 hangs in n = 38:
p_ub = 1 − 0.05^(1/38) = 0.0758. With α = 0.01 for the streak bound —
chosen to match the phase's D1 convention (a false certification must void a
phase gate, so the streak test is held to a 1% false-clean bound; α = 0.05
would under-protect a gate this load-bearing, α = 0.001 would cost N = 84
runs for marginal gain):
N = ceil( ln(0.01) / ln(1−p_ub) ) = ceil( 4.60517 / 0.078835 ) = **59**,
rounded up to **N = 60** for margin. Label: `inferred` (bound-based, not a
measured rate). Total post-decision trials: 38 + 60 = 98.

## Consequences

- Goal 1: **BLOCKED** (bounded 0/38; no command to name — the xtrace tripwire
  exists precisely so a recurrence converts this to a named command).
- Goal 2: **BLOCKED** (no site to date; uncommitted-only boundary recorded in
  HANG-CAPTURE.md).
- Fix task: **branch: none — no fix warranted by the reviewed diagnosis.**
  The tasks-ledger branch vocabulary gains `none` for this outcome; consumers
  checked: the branch line is read only by this change's own task verify
  grep (amended accordingly) — no generator or C-01 validator consumes it.
- Goal 3: certification campaign per the Decision, in standard mode
  (accurate assertion counts); `--xtrace` arms the recurrence hunt.
- Goal 4: this document, under distinct-judge adversarial review (rounds 1–2
  BLOCK; accepted at the 2-round cap with the Unresolved section below).

## Unresolved review findings (round-2 BLOCK accepted at the spec's 2-round cap)

verdict: ACCEPTED WITH UNRESOLVED FINDINGS — rounds 1 and 2 both BLOCK; the
spec's 2-revise-round cap reached; findings at
.kbd-orchestrator/phases/deep-research-onyx-parity/children/stage-10-hang-investigation/review/diagnosis/findings.json
Sycophancy screen record: check-findings-sycophancy.sh invoked with explicit
--findings review/diagnosis/findings.json and counter-key
adv-review-deep-research-onyx-parity-stage-10-hang-investigation-diagnosis;
PASS, score 0.0179, strictness strict; record at review/diagnosis/rejection.md.
Round-2 finding #1 (CRITICAL, decision_fields.assumptions empty): fixed by
renaming the section heading to the parser's bare form — re-extraction
verified (packet builder's decision_fields: assumptions = 4 items, decision
and falsifier non-empty). Finding #3 (CRITICAL, falsifier too narrow) was
reworked into the body's Falsifier section; findings #6 (WARNING,
cost-of-being-wrong) and #8 (WARNING, representativeness) were reworked into
the body's Cost and Assumptions sections; the goals-delivery CRITICAL #2 is
the one carried verbatim below. Round-1's seven findings were
reworked into the body (timeline weakening, arithmetic correction, falsifier
broadening, alternatives, accounting).

> Judge MiniMax-M3 vs producer glm-5.3, cross_model_check verified-distinct,
> both rounds. Reproduced verbatim with dispositions.

1. **CRITICAL** — "The decision reframes an *unidentified* intermittent
   defect as 'not reproducible on current tree' and ships with Goal 1 and
   Goal 2 explicitly BLOCKED — i.e., the change certifies a rate but does not
   deliver the originating goal (identify a cause, prevent recurrence by
   fix)."
   Disposition: accepted as recorded, per the plan stage's terminal
   disposition (plan.md Unresolved #3, itself adversarially reviewed): "A
   second zero-hang campaign leaves Goal 1 BLOCKED with the evidence boundary
   stated; it does not invent a diagnosis." A cause that does not reproduce
   in 38 instrumented runs cannot be identified by honest means; the decision
   ships the strongest defensible claim (bounded, falsifiable by any
   recurrence, instrumented for conversion on recurrence) rather than a
   fabricated one. The judge's standard — no certification without causal
   identification — is a stronger policy than the plan's, and adopting it
   here would mean an unbounded child. The disagreement is inherited, not
   overridden silently.

## Post-reflect evidence addendum — 2026-09-13 (does not alter the Decision; strengthens it)

Timeline archaeology (HANG-CAPTURE.md addendum): the drt-006 extractions PREDATE the 09-09 hang
(15:47/18:47 vs ~20:26) — interpretation 1 above ("incidentally fixed") is DISPROVEN as stated; no
chain file was edited between the hang and the 98 clean campaigns — the same code hung once and
passed 98 times. Consequence for this diagnosis: "not reproducible on current tree" sharpens to
"not reproducible on IDENTICAL code under differing host conditions" — the defect, if in code, is
an environmental-input-dependent race whose triggering conditions were never recorded. The
Decision, its falsifier (any recurrence by any means voids certification), and the --xtrace
tripwire all stand — indeed strengthened: identical code means a recurrence remains genuinely
possible, and the certification's statistical basis (N=60 at the bounded rate) is exactly the right
form of claim for a same-code intermittent defect.
