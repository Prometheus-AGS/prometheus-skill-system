---
type: SessionRecord
id: karpathy-session-d587614cc2df94c5
title: Karpathy session d587614cc2df
tags:
- karpathy
- session-learning
sources:
- resource: session:733a99fd-b357-4127-87d1-0c794fcde506
generated:
  by: pk/1.9.0
  at: 2026-10-03T23:18:05.088516+00:00
created_at: 2026-10-03T23:18:05.088516+00:00
updated_at: 2026-10-03T23:18:05.088516+00:00
revision: 0
---

## Delta

PR 3 is up as [#123](https://github.com/Prometheus-AGS/prometheus-skill-system/pull/123). The session-start summary hook now runs a script shipped in the plugin, instead of the hand-placed `~/.local/bin/kbd-open`.

**New script, `shared/scripts/kbd-open.sh`:**
- **Same sections as before:** phase, position reminder, goals, skill updates, learning log, pulse and FSRS cards.
- **Faster knowledge lookup:** it now uses `pk context` across your project, shared and global knowledge. That reads local snapshots with no model call, so the hook's time limit drops from 30 s to 10 s.
- **Three new review sections:** promotion candidates, new-skill candidates, and knowledge gaps seen at least twice. Each prints nothing when its queue is empty, and each shows the command to act on it. These are where the later memory, promotion and skill-discovery work will surface items for you to decide on.
- **Safety:** works under macOS's bash 3.2, always exits 0, and never starts its output with `{`.

**Generator:** it now requires every hook to run a script inside the plugin. The special case for `~/.local/bin/kbd-open` is gone.

**Checks:**
- The adapter, dispatch, distribution and Codex checks all pass.
- I ran the hook end to end through `hook-entry.mjs` with a scratch home and plugin folder, so your live hook setup wasn't touched:
  - In a scratch KBD project with an empty home, it exited 0 with nothing on stderr and showed the phase and goals.
  - With seeded queues, all three new sections appeared correctly, and a gap seen only once was correctly left out.
  - Outside a KBD project, it printed only the header and footer.

After you merge #123, you can delete `~/.local/bin/kbd-open`; nothing uses it any more.

Next are PRs 4, 5 and 6, which make the KBD memory loop work end to end:
- **PR 4 (skill-pack):** fix the reflect template and write-back so lessons actually reach surreal-memory and pk; rewrite recall to read those lessons; run recall automatically when a KBD stage starts; use one project id everywhere.
- **PR 5 (prometheus-knowledge):** make the learning worker's session notes visible to `pk context`.
- **PR 6 (mini):** apply the same changes there.

Merge #123, then tell me to continue.

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: 733a99fd-b357-4127-87d1-0c794fcde506
- Captured: 2026-10-03T23:17:59.652086Z
- Project: /Users/gqadonis/Projects/prometheus/prometheus-skill-pack

## Changed Paths

- .agent/skills/openspec-bulk-archive-change/SKILL.md
- .agent/skills/openspec-new-change/SKILL.md
- .agents/skills/openspec-apply-change/SKILL.md
- .agents/skills/openspec-archive-change/SKILL.md
- .agents/skills/openspec-bulk-archive-change/SKILL.md
- .agents/skills/openspec-continue-change/SKILL.md
- .agents/skills/openspec-explore/SKILL.md
- .agents/skills/openspec-ff-change/SKILL.md
- .agents/skills/openspec-new-change/SKILL.md
- .agents/skills/openspec-onboard/SKILL.md
- .agents/skills/openspec-sync-specs/SKILL.md
- .agents/skills/openspec-verify-change/SKILL.md
- .cursor/commands/opsx-apply.md
- .cursor/commands/opsx-archive.md
- .cursor/commands/opsx-bulk-archive.md
- .cursor/commands/opsx-continue.md
- .cursor/commands/opsx-explore.md
- .cursor/commands/opsx-ff.md
- .cursor/commands/opsx-new.md
- .cursor/commands/opsx-onboard.md
- .cursor/commands/opsx-sync.md
- .cursor/commands/opsx-verify.md
- .cursor/skills/openspec-apply-change/SKILL.md
- .cursor/skills/openspec-archive-change/SKILL.md
- .cursor/skills/openspec-bulk-archive-change/SKILL.md
- .cursor/skills/openspec-continue-change/SKILL.md
- .cursor/skills/openspec-explore/SKILL.md
- .cursor/skills/openspec-ff-change/SKILL.md
- .cursor/skills/openspec-new-change/SKILL.md
- .cursor/skills/openspec-onboard/SKILL.md
- .cursor/skills/openspec-sync-specs/SKILL.md
- .cursor/skills/openspec-verify-change/SKILL.md
- .devin/skills/openspec-apply-change/SKILL.md
- .devin/skills/openspec-archive-change/SKILL.md
- .devin/skills/openspec-bulk-archive-change/SKILL.md
- .devin/skills/openspec-continue-change/SKILL.md
- .devin/skills/openspec-explore/SKILL.md
- .devin/skills/openspec-ff-change/SKILL.md
- .devin/skills/openspec-new-change/SKILL.md
- .devin/skills/openspec-onboard/SKILL.md
