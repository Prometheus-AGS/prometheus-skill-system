---
id: change-cowork-004-claude-code-plugin-install
title: Extend `cowork plugins install <git-url>` for Claude Code plugin format
phase: cowork-integration
priority: P0
effort: M
wave: 2
agent: general-purpose
status: done
gap_id: G-04
verdict: BUILD
scope:
  - /Users/gqadonis/Projects/prometheus/cowork-skills (existing worktree)
  - cli/src/commands/plugins.rs (add execute_install_plugin)
  - cli/src/main.rs (add Install variant to PluginsAction enum + dispatch)
---

# change-cowork-004 — Claude Code plugin install from git URL

## Context

`cowork plugins` currently supports list/status/uninstall/enable/disable/marketplaces.
It has no `install` subcommand. Claude Code's plugin format uses `.claude-plugin/plugin.json`
as the manifest, and tracks installations in:
- `~/.claude/plugins/installed_plugins.json` — per-plugin installation records
- `~/.claude/settings.json` — `enabledPlugins` map

This change adds `cowork plugins install <git-url>` which clones a plugin repo, validates
its `plugin.json`, installs it under `~/.claude/<plugin-name>/`, registers in both JSON
files, and reports the installed skill paths. Idempotent — re-running updates to the latest
git commit.

## Scope

1. Add `Install` variant to `PluginsAction` enum in `main.rs`
2. Implement `execute_install_plugin(git_url, scope)` in `plugins.rs`:
   - Shell-out `git clone` to a temp dir
   - Discover and validate `.claude-plugin/plugin.json`
   - Copy/install plugin directory to `~/.claude/<plugin-name>/`
   - Register in `~/.claude/plugins/installed_plugins.json` (idempotent JSON merge)
   - Add to `settings.json` `enabledPlugins` (idempotent)
   - Report installed skill paths
3. Unit tests: JSON merge idempotency, plugin.json validation

## Implementation Notes

### plugin.json required fields
```json
{
  "name": "rust-skills",
  "version": "1.2.0",
  "skills": ["skills/rust/rust-patterns"],
  "license": "MIT"
}
```

### Installation directory
`~/.claude/<plugin-name>/` — copy entire repo (or .claude-plugin/ contents)

### installed_plugins.json entry
```json
{
  "version": 2,
  "plugins": {
    "rust-skills@rust-skills": [{
      "scope": "user",
      "installPath": "~/.claude/plugins/<plugin-name>/<version>",
      "version": "1.2.0",
      "installedAt": "<ISO8601>",
      "lastUpdated": "<ISO8601>",
      "gitCommitSha": "<sha>"
    }]
  }
}
```

## Verification

- `cargo build --release` exits 0
- `cargo test` — all tests pass including new JSON merge + validation tests
