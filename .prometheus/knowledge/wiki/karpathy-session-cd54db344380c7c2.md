---
type: SessionRecord
id: karpathy-session-cd54db344380c7c2
title: Karpathy session cd54db344380
tags:
- karpathy
- session-learning
sources:
- session:733a99fd-b357-4127-87d1-0c794fcde506
timestamp: 2026-10-03T18:35:28.834883+00:00
created_at: 2026-10-03T18:35:28.834883+00:00
updated_at: 2026-10-03T18:35:28.834883+00:00
revision: 0
---

## Delta

I merged 93 older project memories into 6 summary files. The memory folder went from 201 files to 114, and the index from 177 lines (22 KB) to 106 lines (13.7 KB), well under the 24.4 KB limit.

**What was merged:** project memories from May through 2026-08-27, grouped by theme. The six summaries total 117 KB, against about 224 KB of originals.

| Summary file | Sources | Size |
|---|---|---|
| `project_archive_pmpo_kbd_process_skills.md` | 20 | 27.9 KB |
| `project_archive_install_platform_repair.md` | 21 | 25.8 KB |
| `project_archive_kbd_control_plane_jul_aug.md` | 16 | 23.7 KB |
| `project_archive_knowledge_memory_services.md` | 8 | 15.4 KB |
| `project_archive_learn_research_ui_docs.md` | 12 | 13.4 KB |
| `project_archive_sovereign_sync_and_credibility.md` | 16 | 11.3 KB |

- **Kept as-is:** all 55 feedback memories, both reference memories, and project memories from 2026-08-29 on. Those cover phases that are still active: control-plane-to-companion, research-agent-hardening, deep-research and cadence.
- **What the summaries keep:** each has one section per phase, with the gotchas, root causes, paths, commands, version pins and open debt. The service rebuild/restart, embedding executor fix and knowledge redeploy runbooks are kept almost word for word. Each file ends with a list of the memories it replaced.
- **What was dropped:** progress narration, "next command" pointers, and the old file names. The agents judged that every source had something worth keeping.
- **Checked before deleting:** every source file name appears in its summary's closing list. Links in other memories that pointed at a merged file now point at its summary; 7 files were updated.
- **Links:** two links the agents wrote were broken, and I fixed them.
- **Backup:** the full pre-merge folder is in the scratchpad at `memory-backup/`. That folder belongs to this session, so copy it somewhere permanent if you want to keep it.

Still worth your attention:
- **15 broken links that were already broken before this merge** remain, for example `[[rah003-driver-done-20260905]]` and `[[project-drt001-done-20260908]]`. They mostly use slugs that don't match the target files' `name:` fields. I can fix them in a separate pass.
- **Stale advice:** the knowledge-services summary carries an old instruction that `plugin.json` should declare `hooks`. The current CLAUDE.md forbids this. The agent marked the conflict in the summary, but the advice itself is out of date.

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: 733a99fd-b357-4127-87d1-0c794fcde506
- Captured: 2026-10-03T18:35:05.246498Z
- Project: /Users/gqadonis/Projects/prometheus/prometheus-skill-pack

## Changed Paths

- No changed paths detected.
