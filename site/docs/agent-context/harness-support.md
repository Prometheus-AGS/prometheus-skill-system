---
title: Harness Support
description: How the structure behaves in Claude Code, Codex, Cursor, Cline, Roo, Kilo, Gemini CLI, Windsurf, OpenCode, Zed, and UAR agents.
---

# Harness support and layout boundaries

Instruction entrypoints and native capabilities depend on the actual harness version and selected project configuration. Read current official documentation when configuring a CLI; do not treat a file in a checkout as proof that a harness loaded it.

The full bootstrap source supports two layouts:

| Layout | Authored/generated authority | Compatibility entrypoints |
| --- | --- | --- |
| Legacy | Managed regions in `AGENTS.md` | Missing `CLAUDE.md` can link to `AGENTS.md`; existing real prose can retain an import |
| v4 | `rules/src/` rendered into regular `CLAUDE.md` and path rules | Supported instruction names link to `CLAUDE.md` |

Those layout rules belong to the bootstrap target, not every repository. Mini uses its separate Node copy-based bootstrap and preserves supported existing in-project instruction links; no symlink creation requirement is introduced into its runtime.

The project team installer preserves native role definitions, permissions, model settings and concurrency rather than replacing them. Codex hooks, Claude hooks and exported agent files have distinct adapters and activation/trust requirements. Native subagent discovery and invocation remain separate from definition export. When the current tool exposes no delegation control, use the selected role instructions sequentially and disclose that limit.

Do not install mutation fences or hosted test workflows to compensate for a harness gap. Tests, diagnostics and independent review run locally after complete phase production. Use [platform support](/docs/guide/platform-support), [Codex delivery](/docs/guide/plugins-and-marketplace) and [agent teams](/docs/guide/agent-teams) for the actual selected paths.
