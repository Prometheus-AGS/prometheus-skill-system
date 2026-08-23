## 1. Implementation

- [ ] 1.1 C-02 scan: `git diff -- .prometheus/` and the untracked files for sk-, api[_-]?key, bearer, token= — RE-RUN, do not inherit
- [ ] 1.2 Commit the 3 modified wiki files + 2 untracked karpathy-session files
- [ ] 1.3 DECIDE .devin/: tracked or .gitignore
- [ ] 1.4 DECIDE .agents/skills/.openspec-target: tracked, ignored, or generated under C-01

## 2. Verification

- [ ] 2.1 The C-02 scan was re-run and its result recorded in the change
- [ ] 2.2 .devin/ and .openspec-target each carry a recorded decision, not a default
- [ ] 2.3 `git status --porcelain -- .prometheus .devin .agents/skills/.openspec-target` is empty
