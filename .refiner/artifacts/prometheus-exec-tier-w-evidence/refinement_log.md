# Refinement log

## Iteration 1 — functional and measurement review

The initial distinct-model review rejected a K/V linker mismatch and mobile
measurements whose missing generated dispatcher allowed the exec graph to be
dead-stripped. The linker, stack contract, memory usage, generated bridge, and
verification provenance were corrected. Fair retained measurements then failed
the 12 MiB mobile gate and that failure became the canonical disposition.

## Iteration 2 — archive integrity

The first remediation review confirmed the code fixes but found stale
checksums and a stale artifact-refiner distribution. Both bundles were rebuilt
from the new receipt, their manifests regenerated, and the baseline procedure,
FRB drift check, shared verifier, memory rollback, and generated stack docs were
made explicit.

## Iteration 3 — semantic sweep and acceptance

The second remediation review found one remaining false size-pass sentence in
task 4.4. It was corrected, the memory rollback path received a direct test and
sentinel, the current dispatcher hash was recorded, and check mode stopped
overwriting the checked-in bridge. The final Claude Opus 5 review returned
`PASS` with no high or medium finding.
