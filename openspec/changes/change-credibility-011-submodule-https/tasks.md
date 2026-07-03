# Tasks: change-credibility-011-submodule-https

- [ ] Read `.gitmodules` to identify all SSH URLs
- [ ] Replace all `git@github.com:` URLs with `https://github.com/` equivalents
- [ ] Run `git submodule sync` to propagate the URL change
- [ ] Run `git submodule update --init --recursive` — succeeds without SSH key setup
- [ ] Verify no SSH URLs remain in `.gitmodules`
