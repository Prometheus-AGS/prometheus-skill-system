---
type: SessionRecord
id: karpathy-session-34575d3ebf4bbf43
title: Karpathy session 34575d3ebf4b
tags:
- karpathy
- session-learning
sources:
- resource: session:1db84095-da34-4e96-ab6a-dfa84d559c8f
generated:
  by: pk/1.11.0
  at: 2026-11-17T16:28:54.753729+00:00
created_at: 2026-11-17T16:28:54.753729+00:00
updated_at: 2026-11-17T16:28:54.753729+00:00
revision: 0
---

## Delta

The pack's skills are already installed globally for all eight, so I installed nothing new. What remains is a set of gaps, listed below, and one mistake of mine you need to act on.

## Rotate your MiMo API key

While checking MiMo's config I printed `~/.config/mimocode/mimocode.jsonc` with a redaction filter that did not match the `apiKey` field. The Xiaomi token-plan key appeared in this session's output and is now in the local transcript. I have not used it or sent it anywhere else, but treat it as exposed and rotate it.

## Where each harness stands

All 218 skills sit once in `~/.agents/skills`, at the current generation. Seven of the eight harnesses reference that folder in their own binaries or docs; Claude Code and Qwen get symlinks instead.

| Harness | Version | Skills reach it through | Evidence |
|---|---|---|---|
| Claude Code | 2.1.326 | links in `~/.claude/skills` | 218 current links |
| Qwen Code | 0.27.1 | links in `~/.qwen/skills` | 218 current links |
| Codex | 0.169.0 | `~/.agents/skills` | its prompt dump lists pack skills; catalogue check says healthy |
| OpenCode | 2.3.0 | `~/.agents/skills` | binary resolves `~/.agents` and `~/.claude` as skill roots |
| Kimi Code | 2.5.3 | `~/.agents/skills` | bundled text says it reads `~/.agents/skills` |
| Zed | 1.28.1 | `~/.agents/skills` | binary names it as the global agent skills folder |
| Xiaomi MiMo | 0.1.15 | `~/.agents/skills` | its system prompt names `.agents/skills` |
| MiniMax Code | app 3.0.68 | `~/.agents/skills`, probably | only the bundled docs of the agent it is built on say so |

I did not start a session in any of them to watch a skill load. Doing that makes model calls on your accounts, so I held back. Only Codex has a free way to check, and I used it.

## Gaps

- **Hooks run only in Claude Code.** Codex has the pack plugin enabled but `hooks = true` is not set under `[features]` in its config, so its hooks will not fire. Kimi's config has no hooks. OpenCode has the pack's plugin entry, which I did not inspect further. Zed, Qwen, MiMo and MiniMax get skills only.
- **MiniMax has a stale second copy.** `~/.minimax/skills` holds 216 `prometheus-…` copies from a generation dated 19 October. The installer leaves them because it can no longer verify the generation that owns them. If MiniMax also reads `~/.agents/skills`, it sees every pack skill twice, one copy a month old.
- **The MiniMax CLI is broken.** `~/.minimax/bin/minimax` and `mavis` are links to a file that no longer exists inside the app, probably since an app update. That is outside the pack; only the desktop app works.
- **Codex lists 58 of the 218 skills by design.** The catalogue policy hides the rest to protect its listing budget; every skill is still callable as `/skill-name`. Several of its pack domain plugins are at older versions than the repo (for example react 1.5.2 against 1.6.0).
- **MiMo is not wired up by the pack installer.** Skills reach it through the shared folder, but the installer adds no MCP servers or hooks for it.
- **`cowork pack status` is out of date.** It still counts skills in the retired per-harness folders, so its numbers do not reflect the real install.

MCP server entries for the pack are already present in the OpenCode, Kimi and MiniMax configs from earlier installs.

## Decision

The stale MiniMax copies are the one thing I can fix now. I would move them to `~/.prometheus/backups/`, not delete them. The risk is that I could not confirm MiniMax reads `~/.agents/skills`; if it does not, moving them leaves MiniMax with no pack skills until they are moved back.

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: 1db84095-da34-4e96-ab6a-dfa84d559c8f
- Captured: 2026-11-17T16:15:44.911735Z
- Project: /Volumes/T7/Projects/prometheus/prometheus-skill-pack

## Changed Paths

- No changed paths detected.
