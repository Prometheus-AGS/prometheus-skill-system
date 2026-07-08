# Tasks — change-prb-008-tag-v160

- [ ] Update `package.json` version field to `"1.6.0"`
- [ ] Update `plugin.json` version field to `"1.6.0"`
- [ ] Add `prometheus-research-server` entry to `.claude-plugin/marketplace.json`
- [ ] Add `prometheus-research` row to CLAUDE.md substrate crates table
- [ ] `git add substrate/prometheus-research/ package.json plugin.json .claude-plugin/marketplace.json CLAUDE.md`
- [ ] `git status` — verify nothing unexpected is staged
- [ ] `git commit -m "feat(substrate): add prometheus-research Rust binary (v1.6.0)"` with full body
- [ ] `git tag v1.6.0`
- [ ] `git push origin main --tags`
- [ ] Verify: `git log --oneline -3` shows the commit
