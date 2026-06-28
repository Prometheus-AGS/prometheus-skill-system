# Tasks — change-learn-028

- [ ] Bump version to `1.4.0` in `package.json` and `.claude-plugin/plugin.json` (update the `"version"` field in both files; confirm they match)
- [ ] Update `marketplace/marketplace.json`: set version to `1.4.0` and add learn domain tags (`"learning"`, `"feynman"`, `"spaced-repetition"`, `"knowledge-base"`, `"meta-learning"`) to the top-level tags array and the learn domain entry
- [ ] Write `CHANGELOG.md` v1.4.0 entry with sections: `### Learn Domain` (list all learn-domain skills with one-line descriptions), `### KB Adapter` (local, Dify, URL scrape), `### Meta-Learning Skills` (learn-about-system, learn-harness), `### Substrate Crates` (surface-bridge, storage-provider, learner-model)
- [ ] Verify `npm run validate:strict` passes for all learn skills: run the command, confirm exit 0, and fix any validation errors before marking this task complete
- [ ] Run `npm run build` to rebuild `.claude-plugin/` symlinks and confirm no broken links; then do a final `git status` check to confirm all expected files are staged and no unintended files are included
