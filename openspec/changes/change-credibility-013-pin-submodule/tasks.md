# Tasks: change-credibility-013-pin-submodule

- [ ] `cd skills/imported/sycophancy-correction && git log --oneline -5` to see recent commits
- [ ] `git checkout <HEAD-SHA>` to detach at current HEAD
- [ ] `cd ../../.. && git add skills/imported/sycophancy-correction`
- [ ] Add pin-policy comment to `.gitmodules`
- [ ] Verify `git submodule status` shows exact SHA (no `+` prefix)
- [ ] Verify submodule init from clean state checks out the pinned commit
