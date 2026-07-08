# Tasks — change-prb-008-tag-v160

- [x] Update `package.json` version field to `"1.6.0"`
- [x] Update `plugin.json` version field to `"1.6.0"`
- [x] Add `prometheus-research-server` entry to `.claude-plugin/marketplace.json`
- [x] Add `prometheus-research` row to CLAUDE.md substrate crates table
- [x] `git add substrate/prometheus-research/ package.json plugin.json .claude-plugin/marketplace.json CLAUDE.md`
- [x] `git status` — verify nothing unexpected is staged
- [x] `git commit -m "feat(substrate): add prometheus-research Rust binary (v1.6.0)"` with full body
- [x] `git tag v1.6.0`
- [x] `git push origin main --tags`
- [x] Verify: `git log --oneline -3` shows the commit
