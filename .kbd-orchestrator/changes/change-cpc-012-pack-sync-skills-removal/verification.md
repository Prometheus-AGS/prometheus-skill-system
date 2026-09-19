# Verification — change-cpc-012-pack-sync-skills-removal

Repository: `prometheus-skill-pack`  
Depends on: change-cpc-008-sync-skills-plugin

## Acceptance criteria

- `npm run validate:codex` and `npm run check:skills-index` pass; `ls skills/learn | grep sync-` is empty.
- No reference to the three skills remains in `.claude-plugin`, `config/codex-catalog.txt`, `docs/codex-plugin.md`, or the generated indexes.

## Verify commands

Every acceptance criterion above maps to a command here; run from the repository named above, locally, after the edit batch.

```verify
test -z "$(ls skills/learn | grep sync-)"
test -z "$(grep -rln 'sync-status\|sync-peers\|sync-push' .claude-plugin config/codex-catalog.txt docs/codex-plugin.md skills-index.json 2>/dev/null)"
npm run generate:skills-index && npm run check:skills-index
npm run build:codex && npm run validate:codex
```

## Evidence

Run locally 2026-09-03 in `prometheus-skill-pack` (HEAD `cfbc262`, uncommitted
working tree at execution). No hosted CI.

| Gate | Result |
|---|---|
| `ls skills/learn \| grep sync-` empty | PASS |
| No sync-* reference in `.claude-plugin`, `config/codex-catalog.txt`, `docs/codex-plugin.md` | PASS |
| `npm run generate:skills-index && npm run check:skills-index` | PASS — 161 skills across 18 categories, up to date |
| `npm run build:codex && npm run validate:codex` | PASS — regenerated (161 skills for Claude and Codex), then `--check` exits 0 with no drift |

**A stale `dist/` was found and fixed, not assumed clean.** `validate:codex`
failed on first run — `generated output is stale: dist/plugins/claude/prometheus-skill-pack`
— because `dist/` still held the three removed `SKILL.md` files from before
this change's edit batch. Regenerated with `npm run build:codex`, confirmed
`ls dist/plugins/claude/prometheus-skill-pack/skills | grep -c '^sync-'` is 0,
then `validate:codex` passed clean.

**A real, machine-scoped artifact was found and left alone, not silently
skipped.** `bash scripts/codex-sync-skills.sh` reported "0 skills synced" —
every catalog entry, including the three removed skills' stale copies at
`~/.codex/skills/{sync-status,sync-peers,sync-push}/`, is protected by a
`.prometheus-generation` receipt file (confirmed: each stale directory has one,
with a real hash). This is the sync script's own by-design guard against
clobbering a signed generation deployment with the legacy sync mechanism —
verified in `scripts/codex-sync-skills.sh`'s source, not assumed. Deleting an
installed, generation-signed artifact on this developer's machine is an
install/uninstall operation outside a repo-scoped KBD change's edit surface,
so it was not done here. The repo-side acceptance criteria above (drift-free
`dist/`, clean catalog grep) are fully met regardless; the installed copy on
this machine will resolve the next time its generation is refreshed through
the platform's normal update path, not through this script.
