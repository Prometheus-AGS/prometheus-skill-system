---
type: SessionRecord
id: karpathy-session-af647babea31244a
title: Karpathy session af647babea31
tags:
- karpathy
- session-learning
sources:
- session:751c647e-97c4-4d8b-bb71-c2a55f99d776
timestamp: 2026-08-05T08:59:55.769812+00:00
created_at: 2026-08-05T08:59:55.769812+00:00
updated_at: 2026-08-05T08:59:55.769812+00:00
revision: 0
---

## Delta

Yes — this is doable. Kimi Desktop loads skills, but only inside a **plugin package**, not from a flat skills directory like the other harnesses.

## What I found

`~/Library/Application Support/kimi-desktop/daimon-share/daimon/plugin-packages/<name>/` with:
- `kimi.plugin.json` — manifest (`$schema: kimi.com/schemas/kimi.plugin.schema.json`)
- `skills/<skill-name>/SKILL.md` — same format as your pack
- `README.md`

That's it. Your 145 SKILL.md files are already in the right format — they just need wrapping in one plugin package.

**My memory note was wrong.** It said Kimi Desktop has no skill directory; that was true in June, but the app now has this plugin surface. I checked rather than trusting it.

## Two things worth flagging before I build anything

**1. A live API key is sitting in plaintext.** `daimon-share/config.toml` contains your Kimi coding key at mode `-rw-r--r--` (world-readable). It's the same credential class as the one I just put in `secrets.env` at 0600. Kimi Desktop wrote it, not me — but it's exposed, and you may want it rotated.

**2. I'd be writing into app-managed state.** `plugin-packages/` is populated by Kimi's own plugin installer with version-pinned packages (`release-pins.v2.json`). A hand-placed package might be wiped on update — the same class of trap as the plugin-cache rule in your CLAUDE.md.

## What I'd propose

Generate `prometheus-skill-pack/` as a proper plugin package — manifest plus all 145 skills — and add a `install_to_kimi_desktop()` function to `install-skills-flat.sh` so it re-installs after any update, exactly like the Codex real-directory sync.

I want your go-ahead before writing into that directory, since it's outside the repo and app-managed. Want me to build it?

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: 751c647e-97c4-4d8b-bb71-c2a55f99d776
- Captured: 2026-08-05T08:59:51.219107Z
- Project: /Users/gqadonis/Projects/prometheus/prometheus-skill-pack

## Changed Paths

- .prometheus/knowledge/wiki/index.md
- .prometheus/knowledge/wiki/log.md
- .prometheus/knowledge/wiki/prometheus-exec-code-execution-engine-completion-record.md
- .prometheus/knowledge/wiki/prometheus-exec-code-execution-engine-session-completion.md
- .prometheus/knowledge/wiki/karpathy-session-028a808ed7035690.md
- .prometheus/knowledge/wiki/karpathy-session-389efa53346e2a07.md
- .prometheus/knowledge/wiki/karpathy-session-3be08d21141940be.md
- .prometheus/knowledge/wiki/karpathy-session-67e311889cb9018a.md
- .prometheus/knowledge/wiki/karpathy-session-76818f05d5251f87.md
- .prometheus/knowledge/wiki/karpathy-session-813b672ff73781cb.md
