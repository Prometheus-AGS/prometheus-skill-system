---
type: SessionRecord
id: karpathy-session-f2e4da2d5ad97685
title: Karpathy session f2e4da2d5ad9
tags:
- karpathy
- session-learning
sources:
- resource: session:1db84095-da34-4e96-ab6a-dfa84d559c8f
generated:
  by: pk/1.11.0
  at: 2026-11-17T09:29:46.578422+00:00
created_at: 2026-11-17T09:29:46.578422+00:00
updated_at: 2026-11-17T09:29:46.578422+00:00
revision: 0
---

## Delta

The frontmatter now matches the spec for `version` and `tags`: the official validator passes 156 of 198 native skills, up from 16, and `npm run validate:strict` still exits 0 with no errors. Nothing is committed.

## What changed

- **Skills:** 180 native skills had top-level `version` moved to `metadata.version`, and 145 had `metadata.tags` converted from a list to a comma-separated string. The diff touches only those two frontmatter lines per file.
- **Validator:** `scripts/validate-skills.js` in strict mode now rejects a top-level `version` and list-valued tags, so the old form cannot come back.
- **Helper scripts:** `backfill-strict-fields.js` writes the new form, `install-minimax-skills.js` reads `metadata.version`, and the example in `cedar-skill-gate.sh` is updated.
- **Docs:** `docs/SKILL_TEMPLATE.md` and the compliance section of `CLAUDE.md` state the new convention.
- **Index:** `SKILLS.md` was regenerated. Its only change is the commit pointer, which was already stale before this work.

Two skills had a top-level version that disagreed with their `metadata.version`. I kept the top-level value in both: `kreuzberg` is now `1.0.0` (metadata said `1.0`) and `deep-research` is now `1.1.0` (metadata said `1.0.0`). Check `deep-research` if `1.0.0` was the intended one.

## What I left alone

- Imported submodules.
- The two `adversarial-review` test fixtures, since they may be protected and one is deliberately flawed.

## Why 42 still fail

They carry other top-level fields the spec does not allow, which your decision did not cover:

| Field | Skills | Note |
|---|---|---|
| `language` | 16 | Pack-specific |
| `triggers`, `authors`, `model_routing` | 15, overlapping | Pack-specific; I have not traced what reads them |
| `argument-hint` | 7 | Claude Code extension that does something, so it cannot simply move |
| `disable-model-invocation` | 4 | Claude Code extension, same |
| Flow-style YAML elsewhere | 4 | Two are the fixtures; `karpathy-tokenizer` and `start-business-build` are the others |

The two Claude Code fields are a real conflict between the spec and the harness, so those 11 skills probably need a declared deviation rather than a fix.

## Not run

- `agnix` and the security scanner were not re-run after the change.
- `npm test`, `npm run build` and any install were not run. Installed copies on this machine keep the old frontmatter until the next pack update.
- The mini pack was not touched.

The result is appended to the [Phase 1 report](docs/reports/2026-11-17-certification-phase1-static-baseline.md).

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: 1db84095-da34-4e96-ab6a-dfa84d559c8f
- Captured: 2026-11-17T09:29:28.782114Z
- Project: /Volumes/T7/Projects/prometheus/prometheus-skill-pack

## Changed Paths

- No changed paths detected.
