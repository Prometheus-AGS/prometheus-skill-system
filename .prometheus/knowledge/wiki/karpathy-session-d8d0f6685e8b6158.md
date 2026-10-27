---
type: SessionRecord
id: karpathy-session-d8d0f6685e8b6158
title: Karpathy session d8d0f6685e8b
tags:
- karpathy
- session-learning
sources:
- resource: session:3f374a11-0ce4-40d3-a1dc-4964cece6769
generated:
  by: pk/1.11.0
  at: 2026-10-05T17:17:23.235533+00:00
created_at: 2026-10-05T17:17:23.235533+00:00
updated_at: 2026-10-05T17:17:23.235533+00:00
revision: 0
---

## Delta

The install is finished except one item. `prometheus doctor` reports no required failures, only 5 warnings.

**What I did**
- **Binaries:** I rebuilt and installed all Rust binaries from the pulled source. That includes `prometheus` 1.11.2, `pk`, `forge`, `liter-llm`, `cowork`, `dsg`, `prometheus-research`, `template-forge` and `surreal-memory-server`.
- **Exec pin:** I recertified it to `fe2f2c5e…`. `prometheus-exec` is installed and its doctor passes.
- **Skills:** I moved the 7 colliding skills in `~/.agents/skills` to `<name>.foreign-20261005`. The plugin generation `26bdb347…` is now installed and verified.
- **Platform skills:** I checked the skill directories for Claude Code, Codex, OpenCode, Kimi Code, MiniMax and Zed. All are populated, and the MCP configs are in place.
- **Plugins:** the Claude and Codex plugins are both at 1.11.2. The new generation has no `dist/` tree, so I registered both marketplaces from this repo checkout, as the Codex docs do. I removed the Claude marketplace entry before confirming that, so the plugin was briefly uninstalled. It is reinstalled now.
- **Stale binaries:** old Aug/Jul copies of `forge`, `liter-llm`, `pk`, `pk-cherry`, `prometheus`, `prometheus-exec` and `prometheus-learning-worker` in `/usr/local/bin` were shadowing the new builds. I renamed each to `*.bak-20261005` and symlinked it to `~/.local/bin`. Delete the backups when you're happy.
- **Services:** I restarted exec, forge-mcp, pk-cherry, liter-llm-api, learning-worker, surface-bridge and research. The HTTP endpoints for openai-proxy, forge, pk, surface-bridge, research and surreal-memory all return 200.

**Still open**
- **SurrealDB 3.3.0:** `scripts/install-mcp-services.sh --restart` stopped at its version check. The upstream scripts now expect SurrealDB 3.3.0 at `~/.prometheus/bin/surreal`, and the machine has 3.2.0. I didn't upgrade the database that holds your memory data. As a result the launchd plists weren't re-rendered, though the running agents work. I can install 3.3.0 and re-run the script if you want that.
- **Doctor warnings:** the Claude marketplace source has uncommitted changes (only the wiki files); 69 learning-queue records are over 6 hours old; the KBD rollout gate is still in shadow mode; and no harness has a measured skill-discovery budget.
- **`sovereign-sync`:** it is loaded in launchd, but port 7892 doesn't answer. It should be disabled unless you opted into `--sharing`.
- **Uncommitted changes:** `config/prometheus-exec-binary.json` (the new pin) plus the wiki `index.md` and `log.md` edits. Nothing is committed or pushed.

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: 3f374a11-0ce4-40d3-a1dc-4964cece6769
- Captured: 2026-10-05T17:17:21.428286Z
- Project: /Users/gqadonis/Projects/prometheus/prometheus-skill-system

## Changed Paths

- config/prometheus-exec-binary.json
- crates/prometheus-exec/.prometheus/
