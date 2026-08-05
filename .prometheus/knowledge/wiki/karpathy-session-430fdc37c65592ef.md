---
type: SessionRecord
id: karpathy-session-430fdc37c65592ef
title: Karpathy session 430fdc37c655
tags:
- karpathy
- session-learning
sources:
- session:751c647e-97c4-4d8b-bb71-c2a55f99d776
timestamp: 2026-08-05T09:15:33.503885+00:00
created_at: 2026-08-05T09:15:33.503885+00:00
updated_at: 2026-08-05T09:15:33.503885+00:00
revision: 0
---

## Delta

OpenCode is fixed and pushed as `07e07cf`.

## Root cause — two compounding defects

**1. Symlinks used relative paths** (`install-plugin-generation.js:682`). `path.relative` from the link's own directory only works at a fixed depth below `$HOME`. `~/.opencode/skills` is **two levels deep**, so `../../.prometheus/...` resolved into `~/.TOOLS/.prometheus/...` — a sibling of your home directory. All 145 links dangled.

**2. Unchecked fallback** (`register-slash-commands.sh:117`). The code was `primary if primary.exists() else fallback` — it never verified the fallback. Run before skills were installed, it wrote the `prometheus-<name>` path unconditionally, which also didn't exist.

The severity multiplier: OpenCode validates every `{file:...}` at startup and **refuses to boot on the first miss**. So one bad reference didn't degrade one command — it killed the entire server.

## Verified

| Check | Result |
|---|---|
| opencode skills | 317 total, **177 absolute, 0 relative, 0 broken** |
| All 3 opencode configs | 145 commands, **0 broken refs** |
| Re-run registrar | idempotent (145 skipped, 0 added) |
| opencode start | no config error — reaches runtime |
| `npm run validate` | 145 skills, 0 errors |

I also removed backticks from two comments I'd put inside the `python3 << PYEOF` heredoc — it's unquoted, so bash was expanding them as command substitution and erroring on every run. My own bug, caught and fixed.

**Note on scope:** this same relative-link defect affected *every* platform installed through the generation store, not just OpenCode. OpenCode simply failed loudly; the others fail silently. The fix corrects all of them.

The Kimi Desktop plugin package is still outstanding — I stopped mid-build when you redirected. Say the word and I'll finish it.

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: 751c647e-97c4-4d8b-bb71-c2a55f99d776
- Captured: 2026-08-05T09:10:50.564294Z
- Project: /Users/gqadonis/Projects/prometheus/prometheus-skill-pack

## Changed Paths

- No changed paths detected.
