# Verification — change-drt-001-dispatch-smoke-and-thread-contracts

Repository: `prometheus-skill-pack`

## Acceptance criteria

- `tests/dispatch-smoke.sh` passes: two workers complete, their executions overlap, and neither observes the other's scratch state.
- The smoke result is recorded either way. A failure is not a blocked change — it is the D-03a signal that strategy B becomes the primary dispatch path.
- Every fixture thread artifact validates against its schema; every claim carries a non-empty verbatim quote traceable to a listed source.
- The suite passes under `/bin/bash` (3.2) as well as bash 5.

## Verify commands

Every acceptance criterion above maps to a command here; run from the repository root, locally, after the coherent edit batch. A command that cannot run is recorded BLOCKED with the reason, never skipped silently.

```verify
bash skills/research/deep-research/tests/dispatch-smoke.sh
/bin/bash skills/research/deep-research/tests/dispatch-smoke.sh
bash skills/research/deep-research/tests/thread-contracts.sh
/bin/bash skills/research/deep-research/tests/thread-contracts.sh
```

**This change's outcome may re-plan the phase.** Task 2 is a real decision point: if concurrent dispatch cannot be demonstrated, drt-002 and drt-003 are re-scoped onto process-level workers and drt-007 moves ahead of them. That is a planned branch, not a failure.

## Evidence

Run locally 2026-09-08. No hosted CI.

| Gate | Command | Result |
|---|---|---|
| Dispatch smoke, fixtures | `bash tests/dispatch-smoke.sh` | **14 passed, 0 failed** |
| Dispatch smoke, real harness children, bash 3.2 | `/bin/bash tests/dispatch-smoke.sh --with-harness` | **20 passed, 0 failed** — two real `claude` children, spawned with the same flags `daemon.rs:249-262` uses, overlapped in wall-clock time |
| Blocked-path control | `--with-harness` on a PATH with core utils but no harness | **exit 2** (BLOCKED), not 0 |
| Thread contracts, bash 5 and 3.2 | `bash` / `/bin/bash tests/thread-contracts.sh` | **14 passed, 0 failed** each |
| Negative control: bad quote | quote altered to text absent from the chunk | fails as `quote not found verbatim in chunk-1` |
| Negative control: foreign citation | dossier cites `[src:deadbeef]` | fails as `absent from this thread's sources.json` |
| Negative control: isolation probe | peer id injected into a worker | probe trips, proving the assertion is not vacuous |
| Driver contract suite, unchanged | `bash tests/driver-contract.sh` | **128 passed, 0 failed** — stage numbers untouched |

## Verdict

**PASS.**

- Task 2's decision point resolved: **concurrent dispatch of isolated workers is available via the process path**, so analysis D-03a's fallback does **not** fire and the planned round order stands. Recorded in `smoke-verdict.md`.
- Scope note: the smoke test covers the process path only. In-session subagent dispatch cannot be dispatched from a shell, and the script says so in its own output rather than implying coverage it lacks. `harness-subagents` therefore stays adopt-provisional — not confirmed, not rejected.
- Carried forward: this run used a shell PATH. Under the **installed launchd service** the daemon still cannot resolve a harness (defect D-A), so process workers will not run as a service until drt-007 lands.

## Defect found at verify: task ids must be strings

`kbd-apply verify` returned FAIL on the first attempt with all five task
verify commands passing when run by hand. The cause was in the change files I
wrote at spec time, not in the work.

`nk_mark_done` marks a task complete with
`jq '(.tasks[] | select(.id == $id))'`, where `$id` arrives from the shell as a
**string**. I had written task ids as JSON **integers**, so `1 == "1"` was false
in jq, no task ever matched, and every `end-task` call reported success while
writing nothing. The five tasks still read `done: false`, so `nk_verify`'s
structural check ("all tasks done") failed — correctly.

Evidence: `rah-011`'s archived tasks carry `ids=['1','2','3','4']` as strings and
`done` all true; mine carried `ids=[1,2,3,4,5]` as integers and `done` all false.
`jq -n '[{id:1},{id:"1"}] | map(select(.id == "1")) | length'` returns 1, not 2.

**Fixed in all seven changes of this phase**, not just this one: every
`tasks.json` now uses string ids, so the remaining six will mark done correctly
the first time. drt-001's archived ledger also has `done: true` set, since the
work is genuinely complete and independently verified by the gates above.

This is the second envelope defect the phase has caught in these files — the
spec-stage review found `change_id` vs `changeId` and the missing `done` fields.
Both would have surfaced only at execution.
