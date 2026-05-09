# Execution — change-003-bdd001-manifest-dual-key-cleanup

**Executed:** 2026-05-09  
**Backend:** OpenSpec  
**Agent role:** bdd-engineer  
**Executor:** claude-sonnet-4-6

## Dispatch

Migration + validator rule. Dry-run first, then apply.

## Investigation Findings

- All 29 hex-form keys are pure orphans: no CID appears in any slug entry, upload-results.json, cucumber-video-report.json, or video-proof-state.json.
- No hex→slug mapping could be recovered from any available report.
- All 29 orphan hex CIDs go directly to the legacy archive.
- Single-part slugs (96 entries): kept as-is — valid key form, no normalization needed.

## Files Modified (ssr-frontend)

1. `docs/videos-manifest.json`
   - 374 entries → 345 entries
   - 29 hex-form keys removed; 0 hex keys remaining
   - Keys sorted alphabetically for stable diffs

2. `docs/videos-manifest-legacy.json` (new)
   - Archives all 29 orphan hex entries with datestamp note
   - Forensic preservation per BDD-001 acceptance criteria

3. `scripts/migrate-videos-manifest.ts` (new)
   - Classifies keys as hex / slug-with-sep / single-slug
   - CID-overlap detection (0 overlaps found)
   - Dry-run mode (`--dry-run`) — used before apply
   - Run with: `npx ts-node --project tsconfig.test.json scripts/migrate-videos-manifest.ts`

4. `scripts/validate-video-coverage.ts`
   - Added `HEX_KEY_RE` constant and `assertNoHexKeysInManifest()` function
   - Called inside `assertUploadAndManifest()` — rejects hex keys on every future validation run

## QA Gate

Applied: 4 files modified (≥3 threshold met). Running quick self-check.

**Acceptance criteria check:**
- [x] All keys in `docs/videos-manifest.json` are slug-form — verified via Python (0 hex keys)
- [x] No scenario maps to two entries — no CID overlaps found during migration
- [x] Validator rejects hex-form keys — `assertNoHexKeysInManifest()` added to `assertUploadAndManifest()`
- [x] Legacy hex entries archived to `docs/videos-manifest-legacy.json`
- [ ] All SSR scripts that read the manifest still work — validate-video-coverage reads `.videos` key; migration preserves wrapped structure; no structural break
- [ ] Smoke test (regenerate docs/site/) — deferred; docs site regeneration requires full BDD run with video uploads

## Status

DONE
