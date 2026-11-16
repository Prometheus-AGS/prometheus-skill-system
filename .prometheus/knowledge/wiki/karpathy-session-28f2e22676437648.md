---
type: SessionRecord
id: karpathy-session-28f2e22676437648
title: Karpathy session 28f2e2267643
tags:
- karpathy
- session-learning
sources:
- resource: session:5789740d-cbcd-4a6a-8e3e-182eb55399dd
generated:
  by: pk/1.11.0
  at: 2026-11-16T14:58:59.688888+00:00
created_at: 2026-11-16T14:58:59.688888+00:00
updated_at: 2026-11-16T14:58:59.688888+00:00
revision: 0
---

## Delta

The liter-llm build in [install-binaries.sh](scripts/install-binaries.sh) is now guarded: a failed build is recorded, the installer carries on, and the run ends with a non-zero exit naming `liter-llm`. Nothing is committed yet.

- **Stale binary:** after a failed build the block skips the lookup too, because the release directory can still hold a binary from an older build. The copy already in `~/.local/bin` is left alone.
- **Missing binary:** "binary not found after build" now also counts as a failed component in a real run.

## Verification

I ran the block's real text in a harness with a stand-in `cargo`, under macOS bash 3.2 and a scratch home:

| Case | Result |
|---|---|
| New code, build fails | Reports the failure, installs nothing, continues to the config step, exits 1 naming `liter-llm` |
| New code, build succeeds | Installs the binary, exits 0 |
| Old code, build fails | Aborts with exit 101 before the config step or the summary |

The real installer's `--dry-run` also exits 0 and shows the liter-llm lines as before. I did not run a real install or a real failing cargo build.

## Docs

[CLAUDE.md](CLAUDE.md) and [CHANGELOG.md](CHANGELOG.md) are updated to match. CLAUDE.md now also names the sections that still have an unguarded `cargo build` and can still abort the installer: `prometheus`, `forge`, `pk`, the substrate binaries, `surreal-memory-server`, `sycophancy-correction` and `template-forge`.

## Found along the way

`npm run validate:harness-adapters` fails on `main`, with or without this change: "release manifest is stale for `shared/scripts/lib/kbd-model-resolve.sh`". The upstream commit `cd371c40` changed that script without regenerating the hook bundle. Fixing it means running `npm run generate:harness-adapters` and committing the regenerated files.

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: 5789740d-cbcd-4a6a-8e3e-182eb55399dd
- Captured: 2026-11-16T14:58:57.699075Z
- Project: /Users/gqadonis/Projects/prometheus/prometheus-skill-system

## Changed Paths

- CHANGELOG.md
- CLAUDE.md
- scripts/install-binaries.sh
