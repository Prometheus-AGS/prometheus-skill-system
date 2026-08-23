## 1. Implementation

- [ ] 1.0 RE-MEASURE both conditions; the facts moved during this phase. `git diff --quiet -- <path>` (pointer) and `git -C <path> status --porcelain` (content) are different questions
- [ ] 1.1 DECIDE A (publish then pin) or B (restore the pin); record the owner and rationale
- [ ] 1.2A Publish the branch; re-pin to a commit `git branch -r --contains` can resolve
- [ ] 1.2B Restore the parent pointer only; do not touch the submodule checkout

## 2. Verification

- [ ] 2.1 The decision, its owner, and its rationale are recorded
- [ ] 2.2 `git status --porcelain -- skills/imported/prometheus-entity-management` is empty
- [ ] 2.3 (A) `git branch -r --contains <new-pin>` is NON-EMPTY — the criterion the current pin fails
- [ ] 2.4 If the submodule went dirty again, STOP and re-decide — a residual " m " blocks require_clean_source and therefore c405
