## 1. Implementation

- [ ] 1.1 Verify the 30 command/workflow files carry the same upgrade as the skills; if they differ in kind, split them out and record the finding
- [ ] 1.2 Confirm no file outside .agent/.agents/.cursor/.opencode is in this change
- [ ] 1.3 Commit naming the generatedBy 1.3.1 -> 1.10.0 bump AND the features (--store, allowed-tools, schemas/view)

## 2. Verification

- [ ] 2.1 `git status --porcelain` shows zero modified files under the four harness trees
- [ ] 2.2 The commit body names both the version bump and the three features
- [ ] 2.3 Task 1.1 recorded what the command files actually contain — a finding either way
- [ ] 2.4 Assert (do not assume) no C-01 source was touched
