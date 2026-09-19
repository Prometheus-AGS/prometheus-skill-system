# Verification — change-drt-009-stage10-hang-fix-and-certification

Repository: `prometheus-skill-pack`

## Acceptance criteria

- **Ordering (Goal 4), machine-checked:** `review/diagnosis/findings.json` shows a judge distinct from the producer (`cross_model_check: verified-distinct`, or `judge_model != producer_model`) and the sycophancy screen's PASS record exists; DIAGNOSIS.md cites both, dated before the first fix edit. If no distinct judge was reachable, task 1 reads BLOCKED with the reason and tasks 2–3 did not run.
- **Branch honesty:** FIX-CERTIFICATION.md carries a `branch: extraction|elsewhere|escalated|none` line matching what was actually done; if extraction ran, an empty byte-diff per extracted program is recorded; if the diagnosis named an out-of-Scope file, the branch reads `escalated` and no out-of-Scope edit exists.
- **Certification (D1), machine-countable:** FIX-CERTIFICATION.md states N with its derivation, the measured-or-bounded rate p with a `verified`/`inferred` label, a consecutive-clean-run table with at least N ≥ 10 rows each beginning `| run`, and one full driver-contract suite pass at its full count.
- **No laundering:** any gate that could not run reads BLOCKED with its reason; a verify string passing for the wrong reason is explicitly called out, not counted (change-drt-006 task-4 precedent).
- `driver-contract.sh` remains unmodified by this change, on every branch.

## Verify commands

Run from the repository root, locally, after the coherent edit batch. A command that cannot run is recorded BLOCKED with the reason, never skipped silently.

```verify
test -s skills/research/deep-research/tests/hang/DIAGNOSIS.md
jq -e '(.cross_model_check == "verified-distinct") or ((.judge_model // "") != (.producer_model // ""))' .kbd-orchestrator/phases/deep-research-onyx-parity/children/stage-10-hang-investigation/review/diagnosis/findings.json
bash -n skills/process/adversarial-review/scripts/build-review-packet.sh
test -s skills/research/deep-research/tests/hang/FIX-CERTIFICATION.md
grep -qE 'branch: *(extraction|elsewhere|escalated|none)' skills/research/deep-research/tests/hang/FIX-CERTIFICATION.md
test "$(grep -cE '^\| *run ' skills/research/deep-research/tests/hang/FIX-CERTIFICATION.md)" -ge 10
git status --porcelain skills/research/deep-research/tests/driver-contract.sh
```

The last command must show no modification to `driver-contract.sh`. When task 1 is BLOCKED (no distinct judge reachable), the jq and FIX-CERTIFICATION gates are expected to fail — that failure is the honest state and is recorded as BLOCKED, never bypassed.

## Evidence

To be recorded at execution time. The certification table is the deliverable: the branch line, N, p and its label, one `| run` row per consecutive clean run, the final suite count, and — on the extraction branch — the empty fidelity diffs. The diagnosis review's findings file and screen record are cited in DIAGNOSIS.md so the Goal-4 gate is auditable after the fact. No hosted CI.
