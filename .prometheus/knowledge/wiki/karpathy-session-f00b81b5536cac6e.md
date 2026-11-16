---
type: SessionRecord
id: karpathy-session-f00b81b5536cac6e
title: Karpathy session f00b81b5536c
tags:
- karpathy
- session-learning
sources:
- resource: session:5789740d-cbcd-4a6a-8e3e-182eb55399dd
generated:
  by: pk/1.11.0
  at: 2026-11-16T09:48:51.170084+00:00
created_at: 2026-11-16T09:48:51.170084+00:00
updated_at: 2026-11-16T09:48:51.170084+00:00
revision: 0
---

## Delta

The files were rewritten by the pack's own SessionStart hook, running from the installed runtime rather than from the repo.

- **The hook:** `sessionstart-detect-project-context` runs `shared/scripts/detect-project-context.sh` from `~/.prometheus/plugins/prometheus-skill-pack/current` (pack 1.14.0, installed Nov 13). Line 27 of that copy calls `openspec/cli.mjs refresh --project "$PWD"`, which is a mutating refresh.
- **The proof:** the receipt at `~/.prometheus/openspec/receipts/30201af159695bcd7cc9ffbd/latest.json` records a `refresh` with status `refreshed` at 09:44:20Z (03:44 local), matching the file timestamps. Its output begins "Force updating 7 tool(s): antigravity, codex, devin, cursor, minimax-code, opencode, agents".
- **Why it was so large this time:** the installed runner follows the `registry-latest` policy, and this was the first refresh of this project with OpenSpec 2.0.0 (the previous one, on Nov 6, used 1.16.0). OpenSpec 2.0 consolidates the per-tool mirrors into `.agents/skills`, which accounts for the 70 deletions.
- **Why it differed from upstream:** the 1.14.0 runner's tool list has no `claude` entry, so it dropped `claude own-files` from `.openspec-target`. It also left out the four extra skills that upstream keeps.

The commits you just pulled already fix this in the repo: [detect-project-context.sh](shared/scripts/detect-project-context.sh) now runs a read-only `verify` at session start and the pack pins OpenSpec 2.0.0. The installed runtime is still 1.14.0, though, so the old `refresh` will run again at the next session start. I expect it to dirty at least the same three files again, but I have not tested that.

The repo's documented way to bring the installed runtime up to date is the update script; I have not run it:

```bash
bash scripts/update-skill-pack.sh --force
```

As a stopgap, setting `PROMETHEUS_OPENSPEC_DISABLE=1` in the environment makes the refresh a no-op.

The pre-refresh copies of the 67 files it touched are in `~/.prometheus/openspec/backups/1794822259303-d00183af-a3a4-4dbe-b9cf-51fb8792eb1c`. I also recorded this as a project-scoped gotcha in the learning log.

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: 5789740d-cbcd-4a6a-8e3e-182eb55399dd
- Captured: 2026-11-16T09:48:49.266702Z
- Project: /Users/gqadonis/Projects/prometheus/prometheus-skill-system

## Changed Paths

- No changed paths detected.
