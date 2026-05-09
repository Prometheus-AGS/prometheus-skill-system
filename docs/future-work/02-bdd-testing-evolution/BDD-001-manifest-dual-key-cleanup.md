---
id: BDD-001
title: Manifest dual-key cleanup migration
status: ready
priority: P0
estimated_effort: 0.5d
agent_role: bdd-engineer
depends_on: []
unblocks: []
related: [BDD-004]
created_from_conversation_turn: 5-6
---

# BDD-001 — Manifest dual-key cleanup migration

## Problem

`docs/videos-manifest.json` in the SSR project has ~250 entries with **dual keying**: 32-char hex (legacy UUID) and slug-form scenario IDs. Some scenarios appear in both forms, so the same scenario maps to two CIDs in the same manifest. The existing validation gate accepts both forms as "present," so the discrepancy doesn't trigger errors.

## Evidence

Inspect the manifest: search for entries matching `^[0-9a-f]{32}$` keys (the hex form) and entries matching `^[a-z0-9-]+--[a-z0-9-]+$` (the slug form). Count overlaps.

`scripts/upload-videos-to-ipfs.ts` has a comment about skipping UUID files; this is an artifact of an earlier scheme that was migrated incompletely.

## Why it matters

- **Canonical key ambiguity.** Tooling that reads `videos-manifest.json` doesn't know which key is authoritative for a given scenario.
- **Storage waste.** Two CIDs per scenario mean two pinned IPFS objects.
- **False sense of completeness.** Validation says "every scenario has a CID" because it accepts either form.
- **Productization blocker for BDD-004.** The reusable BDD video skill cannot ship a clean key-format contract until the existing manifest is normalized.

## Proposed fix

A one-time migration script `scripts/migrate-videos-manifest.ts` that:

1. Reads the manifest.
2. For each entry, classifies as hex or slug.
3. For hex entries: looks up the corresponding slug by reading the historical Cucumber report or by re-deriving from feature/scenario titles. If a slug version exists for the same scenario, drops the hex entry. If not, computes the slug, renames, and keeps.
4. Writes the cleaned manifest atomically.
5. Adds a validator rule that rejects hex-form keys in future writes.

The new validator rule lives in `scripts/validate-video-coverage.ts` (or wherever the manifest schema is enforced).

## Trade-offs and risks

- **Risk: hex entries don't map to a slug.** Some legacy hex CIDs may have lost their scenario context entirely. Mitigation: dry-run mode produces a "could-not-map" list; operator triages. Worst case: legacy hex entries with no mapping are archived to `docs/videos-manifest-legacy.json` for forensic purposes and removed from the main manifest.
- **Risk: in-flight tooling references hex keys.** Mitigation: grep all SSR scripts for the hex pattern. Update or document each.
- **Cost: the IPFS unpin of orphaned hex CIDs is BDD-003's job.** This task only cleans the manifest; the actual unpin happens later.

## Acceptance criteria

- [ ] All keys in `docs/videos-manifest.json` are slug-form.
- [ ] No scenario maps to two entries.
- [ ] Validator rejects hex-form keys on future writes.
- [ ] Legacy hex entries that couldn't be mapped are archived to `docs/videos-manifest-legacy.json` (don't lose them).
- [ ] All SSR scripts that read the manifest still work.
- [ ] Smoke test: regenerate `docs/site/` with the cleaned manifest; all scenarios have functional `▶ Watch` pills.

## Implementation steps

1. Inspect the current manifest. Categorize entries.
2. Write the migration script. Include dry-run.
3. Run dry-run; review the categorization.
4. Run apply; commit the new manifest.
5. Add the validator rule.
6. Regenerate docs site; verify.

## Dependencies

None.

## Open questions

- Are there scenarios *only* in hex form (no slug counterpart)? If yes, derive the slug from the cucumber-report context.
- Does any external system (e.g. a published archive of CIDs) reference the hex keys? Audit; if yes, leave a redirect note in the legacy file.
