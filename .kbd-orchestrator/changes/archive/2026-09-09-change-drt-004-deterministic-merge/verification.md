# Verification — change-drt-004-deterministic-merge

Repository: `prometheus-skill-pack`

## Acceptance criteria

- The merge is deterministic: two runs on the same fixture threads produce byte-identical output and make no gateway request.
- Merged artifacts satisfy the stage 02, 03, and 04 validators **with no change to those validators**, and `driver-contract.sh` passes its full count.
- The same URL cited by three threads resolves to one citation number, recorded in `citation-map.json`.
- A dossier citing an unlisted source fails the merge with a CRITICAL finding naming thread, source, and rule.
- The suite passes under `/bin/bash` (3.2).

## Verify commands

Every acceptance criterion above maps to a command here; run from the repository root, locally, after the coherent edit batch. A command that cannot run is recorded BLOCKED with the reason, never skipped silently.

```verify
npm run check:distribution
npm run validate:codex
bash skills/research/deep-research/tests/merge-threads.sh
/bin/bash skills/research/deep-research/tests/merge-threads.sh
bash skills/research/deep-research/tests/driver-contract.sh
```

**This is the change that makes threading a refactor.** If the existing stage validators need modification to accept merged output, the merge is wrong, not the validators.

## Evidence

Run locally 2026-09-09 from the repository root. No hosted CI.

| Gate | Result |
|---|---|
| `tests/merge-threads.sh` (bash 5) | **PASS** — 17 passed, 0 failed |
| `tests/merge-threads.sh` (`/bin/bash` 3.2, C-05) | **PASS** — 17 passed, 0 failed |
| `merge-threads.sh` itself under `/bin/bash` 3.2 | **PASS** — 3 threads → 2 sources, 2 claims, 2 chunks |
| `tests/driver-contract.sh` (`KBD_PRODUCER_MODEL` unset) | **PASS** — 128 passed, 0 failed, **unmodified** |
| `npm run check:distribution` | **PASS** after regenerating |
| `npm run validate:codex` | **PASS** |
| `npm run check:skills-index` | **PASS** |

`run-research.sh` was **not** edited by this change: the merge is wired into
stage 02 by drt-003 task 3, which is the round that owns that edit. The file
shows as modified in the working tree from the earlier, uncommitted rah-003
driver rewrite — verified by `grep -c 'merge-threads\|threads run'` returning 0.

### Negative controls

Determinism, canonicalisation, and the no-search refusal were each verified by
breaking the implementation and confirming the suite fails:

| Break | Result |
|---|---|
| Key sources on the raw URL instead of the canonical one | 3 assertions **FAILED** — the overlap did not collapse (4 sources instead of 2, 1 marker instead of 3) |
| Disable the dossier/sources cross-check | The CRITICAL-refusal assertions **FAILED** — the violation was accepted |
| Inject a timestamp into an emitted artifact | The byte-identity assertion **FAILED** across two runs |

The first control also exposed a flaw in my own test. "All three resolve to ONE
citation number" passed **vacuously** when canonicalisation was broken: with the
URLs unmerged only one marker matched the filter, and one marker trivially has
one distinct number. The assertion now requires `NUMS == 1 && MARKERS == 3`
together, so it can no longer pass by matching nothing.

### Verdict

**PASS.** The merge is a pure function proven byte-identical across runs; the
unmodified stage 02/03/04 validators pass on merged output and the driver suite
still scores 128/128; three spellings of one URL collapse to one citation
number; duplicate claims collapse while keeping all three provenance tuples; and
a dossier citing an unfetched source exits 2 as CRITICAL, naming the thread and
the source, without writing partial artifacts.
