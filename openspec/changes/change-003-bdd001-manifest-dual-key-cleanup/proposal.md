## Why

`docs/videos-manifest.json` in `ssr-frontend` contains **374 entries** across three key formats:
- **29 hex-form keys** (32-char lowercase hex, legacy): e.g. `025e00ae15af902f51290a864ddbc670`
- **249 slug-form keys with `--` separator** (current canonical): e.g. `acquisition-buyers-invoicing--add-a-buyer-to-an-acquisition`
- **96 single-part slug keys** (intermediate form, no `--`): e.g. `acquisition-edit-page-loads-with-all-tabs`

The hex-form keys are the dual-keying problem. The single-part slugs are a third form the original task doc did not anticipate.

Canonical key ambiguity causes:
- Tooling that reads the manifest doesn't know which key is authoritative.
- False-complete validation (a scenario is "covered" if either key form is present).
- Storage waste (two IPFS CIDs pinned for the same scenario content).
- Productization blocker for BDD-004 (reusable BDD video skill requires a clean key contract).

## What Changes

- Write `scripts/migrate-videos-manifest.ts` that:
  1. Classifies each key as hex, slug-with-separator, or single-part-slug.
  2. For hex keys: derives the slug-form equivalent; merges or archives if no mapping exists.
  3. For single-part slugs: leaves them as-is (they are valid, just an older convention) OR normalizes them to `feature--scenario` form if the feature context is recoverable.
  4. Writes cleaned manifest atomically.
  5. Dry-run mode produces a "could-not-map" report before any write.
- Add a validator rule in `scripts/validate-video-coverage.ts` rejecting hex-form keys on future writes.
- Archive unresolvable hex entries to `docs/videos-manifest-legacy.json`.

## Capabilities

### New Capabilities
- `manifest-hex-key-migration`: One-time migration script that normalizes all manifest keys to slug form.
- `manifest-key-validator`: Runtime validator rule rejecting hex-form keys on any future manifest write.

### Modified Capabilities
- `video-coverage-validation`: Extended to reject hex-form keys.

## Impact

- `ssr-frontend/docs/videos-manifest.json` — normalized (374 → ≤374 entries, 0 hex keys)
- `ssr-frontend/docs/videos-manifest-legacy.json` — new, archives unmappable hex entries
- `ssr-frontend/scripts/migrate-videos-manifest.ts` — new migration script
- `ssr-frontend/scripts/validate-video-coverage.ts` — add hex-key rejection rule
- No Playwright test changes; no IPFS unpin in this change (that is BDD-003)
