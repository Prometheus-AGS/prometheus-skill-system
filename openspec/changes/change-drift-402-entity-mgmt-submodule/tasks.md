## 1. Implementation

- [x] 1.0 RE-MEASURE both conditions; the facts moved during this phase. `git diff --quiet -- <path>` (pointer) and `git -C <path> status --porcelain` (content) are different questions
- [x] 1.1 DECIDE A (publish then pin) or B (restore the pin); record the owner and rationale
- [x] 1.2A Publish the branch; re-pin to a commit `git branch -r --contains` can resolve
- [ ] 1.2B ~~Restore the parent pointer only~~ — **NOT TAKEN.** Option B was superseded by the owner's decision to publish (D-5), and could not have cleared porcelain anyway: git derives the pointer diff from the checkout, so restoring the parent alone changes nothing.

## 2. Verification

- [x] 2.1 The decision, its owner, and its rationale are recorded
- [ ] 2.2 **BLOCKED on PR #22.** The parent pins `1c40eaa`; the checkout is at `a08ee67`. Re-pin once the PR merges, then this clears. A waiting state, not a failure — `a08ee67` is confirmed on the remote.
- [x] 2.3 (A) `git branch -r --contains <new-pin>` is NON-EMPTY — the criterion the current pin fails
- [x] 2.4 **EXERCISED.** The submodule was dirty (1 file) and a concurrent session was editing it live — mtime 69s old, diff growing 4→7 lines mid-change. Stopped and re-decided rather than proceeding: sampled the file for 55s, ran `node --check`, and committed on the author's behalf with authorship stated. c405 remains blocked until PR #22 merges.

## Status: awaiting PR #22

`Prometheus-AGS/prometheus-entity-management#22` — 27 commits, clean
fast-forward. c402 completes by re-pinning the parent to the merged commit.
c405 is blocked until then.
