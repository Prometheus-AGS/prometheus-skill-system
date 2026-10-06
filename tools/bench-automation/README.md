# Research benchmark tooling

These scripts operate on existing research packages for the recorded ten-task
subset. They do not by themselves prove task dispatch, search-tool use, complete
reports, benchmark quality or closure of the historical research change.

| Source | Behavior |
|---|---|
| `run-bench-suite.sh` | Agent-driven package coordinator; search and research stages still need their actual collaborators |
| `bench-runbook.sh` | Labels existing packages, adapts package layout, runs scoring and appends result rows |
| `../../skills/research/deep-research/scripts/label-claims.py` | Assigns claim labels from available source evidence |

The scripts default to a home research directory and can mutate package layout,
labels and recorded results. Inspect their entry points and select disposable
package/state roots before a local acceptance run. `BENCH_G5` selects that package
root; model/provider selection belongs to the actual configured route. A default
model string is not evidence that the provider executed.

Scoring incomplete or previously prepared packages is not an end-to-end research
run. Missing stages and skipped packages remain explicit gaps. Reapplying a
runbook is not a promise of exactly-once result recording. Retain source identity,
provider identity, package hashes and the actual collaborator receipts.

Preserve [benchmark attribution](../../skills/research/deep-research/tests/bench/ATTRIBUTION.md),
license notices and protected fixtures. Complete all production work first; tests,
benchmark execution and independent review wait for the final local integration
boundary. Fixture changes follow the signed protected-test approval protocol.
