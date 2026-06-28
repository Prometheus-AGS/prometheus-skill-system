# change-goal-009: OpenCode Plugin Auto-Install

**Phase:** goal-loop-support
**Status:** pending
**Sub-phase:** B (integration)
**Depends on:** change-goal-002
**Library:** opencode-goal-plugin (ADOPT, @prevalentware/opencode-goal-plugin)

## Problem

OpenCode has no native `/goal`. The `@prevalentware/opencode-goal-plugin` provides it, but users must install it manually. `install-skills-flat.sh` doesn't handle it.

## Solution

Add an OpenCode section to `install-skills-flat.sh` that detects OpenCode, checks for the plugin, installs it if missing, and writes KBD-tuned defaults to OpenCode's config.

## Files

- `scripts/install-skills-flat.sh` (UPDATE: add OpenCode goal plugin section)
- `skills/process/kbd-goal/references/platforms/opencode.md` (CREATE)

## Tasks

- [ ] Add to `install-skills-flat.sh` OpenCode section:
  - `command -v opencode` check
  - `opencode plugins list | grep -q opencode-goal-plugin` check
  - If missing: `npx @prevalentware/opencode-goal-plugin install`
  - Write config to `.opencode/config.toml` or `~/.opencode/config.toml`: `auto_continue=true`, `max_auto_turns=20`, `no_progress_token_threshold=5000`, `max_no_progress_turns=3`, `default_token_budget=200000`
- [ ] Document in `opencode.md`: KBD uses `create_goal` agent tool to set goal state; `update_goal` for phase transitions; `session.idle` events drive continuation
- [ ] Update `kbd-goal/SKILL.md` OpenCode section
