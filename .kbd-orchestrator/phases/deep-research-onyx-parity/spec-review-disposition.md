# Spec-stage adversarial review — disposition

Judge k3, producer `claude-opus-5`, artifact mode, two rounds (cap reached).
Receipts under `review/spec/` (`round1/`, then round 2). Both rounds passed the
findings sycophancy gate. **Fifteen findings, all accepted, none rejected.**

## Round 1 — 4 CRITICAL, 4 WARNING

| # | Finding | Disposition |
|---|---|---|
| 1 | drt-003 rewires stage 02 to the scheduler with no dependency on drt-004's merge, so stage 02 could be wired halfway and produce no contract artifacts | **Fixed.** drt-003 now depends on drt-004, and its task 3 requires *both* `threads run` and `merge-threads` — wired together or not at all |
| 2 | drt-006 benchmarks the pipeline but declared no dependency on the scheduler or the agents, so the plan permitted scoring the old sequential pipeline | **Fixed.** drt-006 now depends on drt-002, 003, 004, and 005. Benchmarking the sequential pipeline would measure the thing this phase exists to replace |
| 3 | **The dispatch runtime was never reconciled**: a Rust `Semaphore` cannot cap in-session Agent-tool dispatches, so the phase's central requirement was unenforceable as written | **Fixed, and this was the most valuable finding.** drt-002 now states that the scheduler spawns harness CLI processes, names the two strategies as two runtimes, and scopes the code-enforced cap to strategy B. Strategy A's bound is advisory at dispatch and enforced post hoc by the merge — the same honesty the no-search rule already used |
| 4 | The launchd end-to-end criterion that *is* the D-A fix had no command in drt-007's verify block | **Fixed.** Added `tests/installed-service-smoke.sh` as both a task and a verify command |
| 5 | drt-001's thread-contract criteria had no validating command anywhere | **Fixed.** Added `tests/thread-contracts.sh` as a task and a verify command |
| 6 | drt-003 and drt-005 edit `run-research.sh` (launchd-invoked) but neither verified under `/bin/bash` 3.2, contrary to the C-05 bullet they both carry | **Fixed.** Both verify blocks now run the driver suite under `/bin/bash` |
| 7 | Two citation-numbering authorities: drt-005's outline pre-assigned numbers while drt-004's merge owns `citation-map.json` | **Fixed.** The merge is the single authority; the outline carries claim ids only and the assembler resolves them. drt-005 now depends on drt-004 |
| 8 | The C-01 services-manifest obligation asserted for the plist edit rests on the research plist being a generator input — and it is not | **Fixed, and my claim was wrong.** `generate-service-manifest.mjs:35-36` reads only `shared/launchagents` and `shared/systemd`; `com.prometheus.research` is absent from the manifest. Analysis D-11b over-applied C-01; corrected as D-11c. The real gap it exposed — a launchd service the manifest does not track — is raised as an open question in drt-007 rather than fixed as a side effect of a `PATH` change |

## Round 2 — 2 CRITICAL, 4 WARNING, 1 SUGGESTION

| # | Finding | Disposition |
|---|---|---|
| 1 | drt-007's requirements and verification still imposed the manifest obligation its own What Changes section had just corrected | **Fixed.** The requirement is rewritten as the negative assertion the correction actually makes: prove inapplicability by command, and note that it returns if the plist ever migrates |
| 2 | drt-005 task 1 still told the outline to pre-assign citation numbers, contradicting the spec fixed in round 1 | **Fixed.** Task 1 now says claim ids only |
| 3 | drt-004 task 3 edits files outside its declared Scope | **Fixed.** Scope gains `shared/scripts/lib/` and `run-research.sh` |
| 4 | `SKILLS.md` is a task target in drt-003 and drt-005 but absent from both Scopes | **Fixed.** Added to both |
| 5 | The "stale install is visible" requirement had no behavioral gate — only a grep | **Fixed.** The smoke test now installs a marker-less stub-driver fixture and asserts the daemon reports it stale with both paths and sizes |
| 6 | **All seven `tasks.json` used a different envelope from the one the driver reads** (`change_id` vs `changeId`, no `schemaVersion`, no `done` fields) | **Fixed, and this would have broken execution.** Every file now matches the predecessor's proven envelope; `kbd-apply progress` reads all seven correctly |
| 7 | The bash-3.2 criterion covered "the suite" but only one of drt-001's two suites ran under `/bin/bash` | **Fixed.** Both do now |

## What the review changed structurally

Round 1 finding 3 changed the architecture's description, not just its wording:
the phase now states plainly that it has two dispatch runtimes with two different
enforcement stories. Finding 8 corrected a decision made at analyze. Finding 6
would have failed at the first `/kbd-apply` invocation.

No finding was rejected in either round.
