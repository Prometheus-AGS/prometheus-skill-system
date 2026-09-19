# Verification — change-drt-008-hang-capture-harness

Repository: `prometheus-skill-pack`

## Acceptance criteria

- `run-hang-capture.sh` exists, is executable, parses under `/bin/bash` (3.2), and exposes `--runs`, a per-run timeout, scoped PID-only cleanup, and live capture (`ps` tree + `sample` stacks + output-volume observation) before teardown.
- `HANG-CAPTURE.md` records the full per-run table (duration, exit code, assertions, load average), the measured failure rate as hangs/K with K stated, and — if a hang was captured — the exact blocked command with its stack and the output-volume observation.
- The goal-2 dating claim cites its evidence, **names its method** on a `method:` line (`commit-pass` | `merge-base` | `uncommitted-only`), and states the uncommitted-tree boundary; where the implicated code exists only in uncommitted work, the claim says "introduced by this phase's uncommitted work, undateable from committed history" rather than inventing a date.
- No orphaned driver-descendant processes remain after the campaign ends.
- `driver-contract.sh` and every other pre-existing file is byte-identical to before this change (new files only).

## Verify commands

Run from the repository root, locally, after the coherent edit batch. A command that cannot run is recorded BLOCKED with the reason, never skipped silently.

```verify
bash -n skills/research/deep-research/tests/hang/run-hang-capture.sh
/bin/bash -n skills/research/deep-research/tests/hang/run-hang-capture.sh
test -x skills/research/deep-research/tests/hang/run-hang-capture.sh
grep -q -- '--runs' skills/research/deep-research/tests/hang/run-hang-capture.sh
test -s skills/research/deep-research/tests/hang/HANG-CAPTURE.md
grep -qiE 'failure rate|hangs/' skills/research/deep-research/tests/hang/HANG-CAPTURE.md
grep -qEi 'pre-existing|introduced' skills/research/deep-research/tests/hang/HANG-CAPTURE.md
grep -qiE 'method: *(commit-pass|merge-base|uncommitted-only)' skills/research/deep-research/tests/hang/HANG-CAPTURE.md
if pgrep -f "deep-research/tests/driver-contract.sh" >/dev/null; then exit 1; fi
git status --porcelain skills/research/deep-research/tests/driver-contract.sh
```

The last command must show no modification to `driver-contract.sh`.

## Evidence

To be recorded at execution time: campaign duration, K, hangs observed, and (if captured) the blocked command and stack verbatim in `HANG-CAPTURE.md`. Machine load per run is part of the record; runs under elevated load are marked, not discarded. No hosted CI.
