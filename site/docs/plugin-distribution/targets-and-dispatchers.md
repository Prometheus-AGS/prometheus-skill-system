---
title: Targets and stable dispatchers
description: The 14-target matrix, copy targets, symlink targets, and hook routing.
---

# Targets and stable dispatchers

One generation is projected into 14 supported skill locations:

| Target | Mode |
| --- | --- |
| `.claude/skills` | symlink |
| `.opencode/skills` | symlink |
| `.kimi-code/skills` | symlink |
| `.minimax/skills` | verified copy |
| `.cursor/skills` | symlink |
| `.codex/skills` | verified copy |
| `.gemini/skills` | symlink |
| `.roo/skills` | symlink |
| `.windsurf/skills` | symlink |
| `.codeium/windsurf/skills` | symlink |
| `.agents/skills` | symlink |
| `.config/zed/skills` | symlink |
| `.zed/skills` | symlink |
| `.cline/skills` | symlink |

Codex and MiniMax require real directories, so the installer copies their skills and writes generation receipts. Every other target links through the active generation. A destination collision is preserved and reported; the installer does not overwrite unrelated user content.

Stable dispatchers live under `~/.prometheus/plugins/prometheus-skill-pack/stable/`. They resolve required scripts and helpers through `current`, including hook dispatch, project detection, memory outbox flush, learning enqueue, and `pk` health. Host configuration points to these stable paths, so activation and rollback do not require rewriting hook registrations.

Verification rejects missing receipts, wrong modes, a dispatcher that resolves outside `generations/`, or a target still resolving through a stale version path.

