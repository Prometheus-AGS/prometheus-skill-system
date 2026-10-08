# Spec review resolution — team-aware-learning-memory-impl

Judge: harness-native, fresh context (`cross_model_check: same-model-collision`; the preflight was degraded).

## Round 1 — BLOCK (10 CRITICAL, 6 WARNING, 2 SUGGESTION)

**CRITICAL findings, all fixed:**
- Two `! grep` gates under `set -e` could never fail (B1, D3). Both are rewritten as `if grep …; then exit 1; fi`, and B1's pattern and path now match its verification.
- Eight changes edited files they shared with no ordering between them. New dependencies fix this: B3→B2 (B3 now owns the team-role learning group), E1→A5a, C1a→A5a, C2→B7, D3→B7, C3b→C1b, C4→B3b+B6, B6→B3b, B7→B6, D1a→D2.

**WARNING and SUGGESTION findings, all fixed:**
- C1a gains a pk-cli `candidates` test.
- A4 runs with `--include-ignored` plus a ≥4-tests-ran assertion, and an in-process ConnectInfo 403 test.
- B5 gains a ≤2,000-token assertion and Codex trust-path evidence.
- B7 now depends on B6.
- Task file lists gained the version, lock and generated files.
- D1a gains a B7-on-main precondition.
- Gates that need pk ≥1.10/≥1.11 return BLOCKED on an older pk.
- D1b gains the BLOCKED-ON-OPERATOR pin check.

## Round 2 — BLOCK (4 CRITICAL, 4 WARNING, 1 SUGGESTION)

**CRITICAL findings, all fixed systemically:**
- `set -e` ignores a failure anywhere except the last command of an `a && b && c` chain, so the regeneration, validator, runtime-test, install and parity chains could print "verify OK" after a failure.
- Every verify line is now emitted as `( cmd ) || exit $?`. Exit codes, including 2 for BLOCKED, are preserved. Only the main-shell lines (the A4 server start with its trap) stay unwrapped.
- Behavioural check: `( false && true ) || exit $?` exits 1.

**WARNING and SUGGESTION findings, all fixed:**
- Pin-equality checks now require non-empty values on both sides, and version matches are anchored.
- D1a uses a per-run temp file.
- B7 runs `report-learning-delivery.py --require-reduction`.
- B6 adds a user-confirmed live-index partition task with size evidence ≤4,096 bytes, checked by the gate.
- A4 reports BLOCKED when the gate server fails to start, and uses per-run log files.
- A defect found while fixing these: the A4 env assignments were placed after the log assignment and would have become unexported shell variables. Reordered.

## Unresolved review findings

- **(WARNING, accepted)** The Constraints rule names `hooks/*.json`, `shared/harnesses/generated/*` and `shared/scripts/generated/*` as regenerate-after-rebase outputs. Today the only unordered overlap is `dist/plugins/**`. The broader wording is deliberate: all four trees come from the same idempotent generators (C-04). Every gate that touches them runs regeneration followed by `git diff --exit-code`.
- The maximum of 2 review rounds is reached. The round-2 fixes have not been re-reviewed by a judge. They are mechanical, and `bash -n` passes on all 25 `verify.sh` files.
