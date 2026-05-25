# Tasks: pglite-003

- [ ] Edit `.claude-plugin/plugin.json` — add `"pglite"` and `"electricsql"` to keywords array
- [ ] Edit `.opencode/package.json` — bump `@opencode-ai/plugin` to `^1.15.0` and `@opencode-ai/sdk` to `^1.15.0`
- [ ] Verify JSON validity for both files
- [ ] Run `npm run validate:strict` → confirm 0 errors
- [ ] Commit with message: `chore(plugin): add pglite/electricsql keywords + bump opencode deps to ^1.15.0`
