# Adversarial review disposition — change-drt-007-install-surface-repair

Two rounds, cross-model judge (producer `claude-opus-5`, judge routed off-family).

## Round 2 — BLOCK, 5 findings, all 5 accepted

| # | Severity | Finding | Disposition |
|---|---|---|---|
| 1 | CRITICAL | The launchd leg is opt-in via `SMOKE_LAUNCHD=1`, so the recorded verification path never proves the primary acceptance criterion | **ACCEPTED.** Task 5's `verify` string and `verification.md` now run the launchd leg as the recorded path. Recorded run: 19 passed / 0 failed, `harness_pid` non-null on `job-1788909316-22a2fcf7`. The default 16-assertion run is explicitly disqualified as evidence for that criterion. |
| 2 | CRITICAL | `verification.md` claimed "Both defects are recorded as repaired" while the change's own spec (line 186) says D-B is "mitigated and made visible, not yet repaired" | **ACCEPTED.** The criterion was self-contradictory. Rewritten to state D-A REPAIRED / D-B VISIBLE-not-repaired, with the task-4 blocker named. |
| 3 | WARNING | The launchd leg's comment claims the `harness_pid` assertion "works on any binary version", but `harness_pid` is recorded by new daemon code | **ACCEPTED — I initially rejected this and was wrong.** My refutation rested on misreading my own probe: `git show HEAD:…/checkpoint.rs \| grep -c harness_pid` returns **0**, which means `harness_pid` is absent at HEAD, not present. It is not universal. The assertion passed only because the *installed* binary is newer than HEAD. Both the self-check and `harness_pid` assertions are now capability-gated: a binary that cannot report either yields a skip naming the remedy, while a `blocked` job still FAILS. Attribution nuance: `checkpoint.rs` is not in this change's `files.txt`, so the code arrived in an earlier uncommitted change (rah-004) — but it is still newer than HEAD, so the finding's substance holds. |
| 4 | WARNING | The `RESEARCH_EVENT_TOKEN` warning inside `run_server` advises starting via `--mode server` — self-contradictory, since it is already running | **ACCEPTED.** Rephrased to state the consequence (ingest POSTs refused) and the real remedy (set it in the plist's `EnvironmentVariables`). |
| 5 | SUGGESTION | The placeholder guard only greps `__PROMETHEUS_`, so a dropped `__HOME__` substitution passes silently | **ACCEPTED.** Guard is now `grep -q '__PROMETHEUS_\|__HOME__'` and the reported list widened to `__[A-Z_]*__`, matching the comment's own claim. |

## Note on round 1, finding 6 (the `--sharing` regression)

Round 1 found `--sharing` and the sovereign-sync build had vanished from
`install-binaries.sh`. I could not prove which edit removed them, so I rebuilt the file from
`git show HEAD:` plus only the intended change. `--sharing` is restored (3 occurrences) and is
now re-checked after any operation that touches the file.
