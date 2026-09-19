# change-cpc-012-pack-sync-skills-removal

**Title:** Remove the sync-* skills and their plugin surface from the pack  
**Repository:** `prometheus-skill-pack`  
**Phase:** control-plane-to-companion  
**Depends on:** `change-cpc-008-sync-skills-plugin`  
**Backend:** native-kbd

## Why

Once the Companion ships the sync-* skills (change-cpc-008), the pack must stop shipping them: the skills index, the Claude and Codex plugin surfaces, the Codex catalog, and the Codex plugin documentation all reference them. Constraints C-01 and C-03 require the generated surfaces and `docs/codex-plugin.md` to change in the same change.

## What Changes

- Remove `skills/learn/sync-status`, `sync-peers`, `sync-push`; regenerate the skills index; update `.claude-plugin/*` marketplace and plugin entries, `config/codex-catalog.txt`, and rebuild the Codex mirror; update `docs/codex-plugin.md` and the CLAUDE.md Codex section; run `npm run validate:codex`.

## Scope

Files this change may create or edit (tasks.json `files` is the per-task view):

- `.agents/plugins/marketplace.json`
- `.claude-plugin/marketplace.json`
- `.claude-plugin/plugin.json`
- `.codex-plugin/plugin.json`
- `CLAUDE.md`
- `config/codex-catalog.txt`
- `docs/codex-plugin.md`
- `skills/learn/sync-peers/`
- `skills/learn/sync-push/`
- `skills/learn/sync-status/`

## Capabilities

- `pack-open-core` (existing; added by change-cpc-009)

## ADDED Requirements

### Requirement: Pack no longer ships the sync skills
WHEN the pack is installed without the Companion, THEN no sync-* skill is present and no plugin manifest, catalog, or doc references one.

#### Scenario: Codex parity
- **WHEN** `npm run validate:codex` runs
- **THEN** it passes with no drift

#### Scenario: Catalog
- **WHEN** `grep -rn 'sync-status\|sync-peers\|sync-push' .claude-plugin config docs/codex-plugin.md skills-index*` runs
- **THEN** it returns nothing

## Constraints

- Implementation-first, integration-only evidence: no unit tests as delivery evidence; every acceptance criterion below has a command in `verification.md`'s verify block, run after the coherent edit batch, locally only.
- One Cargo build machine-wide at a time; `cargo check -p <crate>` only as a narrowly targeted diagnostic.
- Constraints C-01..C-05 apply; `npm run validate:codex` and `docs/codex-plugin.md` in the same change when plugin surfaces or install flow move; `shared/services.manifest.json` regenerated in the same change when a plist or unit changes.
- The pack never depends on the Companion; the Companion consumes pack crates by git rev (D-02).
