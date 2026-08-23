## 1. Implementation

- [x] 1.1 Verify the 30 command/workflow files carry the same upgrade as the skills; if they differ in kind, split them out and record the finding
- [x] 1.2 Confirm no file outside .agent/.agents/.cursor/.opencode is in this change
- [x] 1.3 Commit naming the generatedBy 1.3.1 -> 1.10.0 bump AND the features (--store, allowed-tools, schemas/view)

## 2. Verification

- [x] 2.1 `git status --porcelain` shows zero modified files under the four harness trees
- [x] 2.2 The commit body names both the version bump and the three features
- [x] 2.3 Task 1.1 recorded what the command files actually contain — a finding either way
- [x] 2.4 Assert (do not assume) no C-01 source was touched
