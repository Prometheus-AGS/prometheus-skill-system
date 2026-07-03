# Tasks: change-credibility-012-machine-state-gitignore

- [ ] Read current `.gitignore` to understand existing patterns
- [ ] Add `.prometheus/`, `.kbd-orchestrator/**/project.json`, `SCRATCHPAD.md`, `.envrc`, `*.local.env` patterns
- [ ] Run `git ls-files .prometheus/ .kbd-orchestrator/ SCRATCHPAD.md` to find currently tracked files
- [ ] Run `git rm --cached` on any currently-tracked files that match new patterns
- [ ] Verify `git status` no longer shows these paths as untracked (they should be ignored)
- [ ] Verify `git ls-files .prometheus/` returns empty
