---
type: SessionRecord
id: karpathy-session-34b3daf21b8e3058
title: Karpathy session 34b3daf21b8e
tags:
- karpathy
- session-learning
sources:
- resource: session:1db84095-da34-4e96-ab6a-dfa84d559c8f
generated:
  by: pk/1.11.0
  at: 2026-11-17T16:33:08.839711+00:00
created_at: 2026-11-17T16:33:08.839711+00:00
updated_at: 2026-11-17T16:33:08.839711+00:00
revision: 0
---

## Delta

Hooks are now enabled for Codex and Kimi Code, and OpenCode was already set up. The other four cannot be enabled today because the pack has no hook adapter for them. I have not seen a hook fire in a live session of any of these.

| Harness | Hooks now | What I did |
|---|---|---|
| Claude Code | on | nothing; already on through `prometheus-runtime` |
| Codex | on, pending one approval | added `hooks = true` under `[features]` in `~/.codex/config.toml` |
| Kimi Code | on | added seven `[[hooks]]` entries to `~/.kimi-code/config.toml` |
| OpenCode | already registered | nothing |
| Qwen, Zed, MiMo, MiniMax | not available | nothing |

## Codex

- `codex features list` now reports `hooks` as true, and the config still loads.
- The pack plugin is enabled, read straight from the repo, and its hook bundle matches the active generation.
- **You need to do one thing:** the first interactive `codex` session will show a one-time hook-trust prompt. Accept it, or the hooks will not run.

## Kimi Code

The pack generates a Kimi hook file but nothing ever installed it, so I translated it into Kimi's own config format by hand.

- The seven events are SessionStart, PreCompact, PostCompact, UserPromptSubmit, Stop, Interrupt and PostToolUse.
- The block sits between marker comments, so it is easy to remove. The commands point at the active installed generation, so they follow pack updates.
- The config still parses and `kimi provider list` runs with it. The SessionStart command exits 0 both inside and outside a KBD project, and inside this repo it prints the KBD position.
- I set a 5-second timeout on five of them. The generated file says 1000 ms, but Kimi takes whole seconds and one second is tight.
- Pack updates will not maintain this block. If the hook contract changes, it has to be redone, unless the installer learns to write it.

## OpenCode

The pack's plugin is already registered in all three OpenCode config files. I did not start OpenCode to confirm the plugin loads.

One thing in your config works against it: `~/.config/opencode/opencode.json` has permission rules that deny the skills `kbd-*` and `evolve-*`. Those are your rules and I left them, but they mean OpenCode cannot use the KBD skills even though they are installed.

## The four without hooks

The pack's hook contract covers only Claude Code, Codex, OpenCode and Kimi. Qwen Code, Zed, MiMo and MiniMax get skills and nothing else. Enabling them means writing a new adapter per harness, starting with whether each one has a hook mechanism at all. I have not looked into that.

## Backups

Both originals are in `~/.prometheus/backups/hooks-enable-20261117/`, readable only by you.

The MiMo API key from earlier still needs rotating.

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: 1db84095-da34-4e96-ab6a-dfa84d559c8f
- Captured: 2026-11-17T16:21:35.968991Z
- Project: /Volumes/T7/Projects/prometheus/prometheus-skill-pack

## Changed Paths

- No changed paths detected.
