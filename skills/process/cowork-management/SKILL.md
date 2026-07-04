---
name: cowork-management
version: '1.0.0'
license: MIT
description: >
  Install, update, and manage AI coding agent skills across 20+ platforms using
  the cowork CLI. Covers Claude Code, Codex, OpenCode, Kimi Code, Zed, Cursor,
  Windsurf, GitHub Copilot, and more. Also manages prometheus-skill-pack updates,
  toolchain health checks, and disk-space reclamation via dsg delegation.
metadata:
  author: Travis James
  category: process
  tags: [cli, skill-management, cowork, install, platform, toolchain, prometheus]
---

# cowork-management

Use the `cowork` CLI (alias: `co`) to install, update, and maintain AI agent
skills across every platform that supports them.

## When to use

- Installing or updating the prometheus-skill-pack on a new machine
- Checking which platforms are configured and healthy
- Repairing broken symlinks or stale installs after an OS upgrade
- Scanning for reclaimed disk space without leaving the skill-pack workflow
- Configuring a new tool (Codex, OpenCode, Kimi) to consume skills

## Quick start

```bash
# Check overall status of installed skills and platforms
cowork status

# Install prometheus-skill-pack on all detected platforms
cowork install --source .

# Update the pack to the latest commit
cowork pack update

# Check toolchain health (Rust, Node, git, cargo-dist, etc.)
cowork toolchain status

# Scan disk for reclaimable build artifacts
cowork disk scan

# Run doctor to identify and auto-fix common install issues
cowork doctor
```

## Command groups

### pack — skill-pack lifecycle

```bash
cowork pack status          # show pack root, git status, platform links
cowork pack update          # git pull + re-install to all platforms
cowork pack repair          # fix broken symlinks, re-run install-binaries.sh
```

### install — install skills to platforms

```bash
cowork install --source .                     # install current directory as a skill pack
cowork install --source git@github.com:...    # install from remote repo
cowork install --platform claude-code         # install to Claude Code only
cowork install --platform codex               # install to Codex only
cowork install --platform opencode            # install to OpenCode only
```

Supported platforms: `claude-code`, `codex`, `opencode`, `kimi`, `zed`,
`cursor`, `windsurf`, `minimax`, `copilot` (and more via auto-detect).

### toolchain — toolchain health

```bash
cowork toolchain status          # check Rust, Node, git, cargo-dist versions
cowork toolchain install         # install missing toolchain components
cowork toolchain update          # update all managed toolchain tools
```

### disk — disk-space reclamation (delegates to dsg)

```bash
cowork disk scan                            # show reclaimable build artifacts
cowork disk scan --deep                     # deep scan all home subdirs
cowork disk scan --ecosystem rust           # Rust-only scan
cowork disk clean --dry-run                 # preview what would be removed
cowork disk clean --force                   # actually move to Trash (irreversible!)
cowork disk status                          # quick summary of known artifact roots
```

### plugins — plugin marketplace management

```bash
cowork plugins list                         # list installed plugins
cowork plugins install <git-url>            # install a plugin from a repo
cowork plugins uninstall <plugin-id>        # remove a plugin
cowork plugins status                       # health of all plugins
cowork plugins list-marketplaces            # show configured marketplace sources
```

### config — per-project skill configuration

```bash
cowork config init                          # create .cowork.toml in project root
cowork config show                          # show current config
cowork config enable <skill>                # enable a skill for this project
cowork config disable <skill>               # disable a skill
cowork config add <name> <source>           # add a custom skill source
cowork config sync                          # pull latest from all remote sources
```

### doctor — diagnose and repair

```bash
cowork doctor                               # full health check + auto-repair suggestions
```

Doctor checks:
- Platform skill directories exist and have correct permissions
- All symlinks resolve
- Installed binary versions match expected ranges
- `cowork.toml` / `.cowork.toml` schema is valid
- Platform-specific configs (Codex `agents.toml`, OpenCode `package.json`) are present

### search / list / audit / verify

```bash
cowork search <query>                       # search agentskills.io registry
cowork list                                 # list all installed skills
cowork list --platform codex                # list skills for a specific platform
cowork audit                                # audit skill quality (frontmatter, paths, etc.)
cowork verify                               # verify skill checksums
```

## Platform-specific notes

### Claude Code
Skills install to `~/.claude/skills/`. Plugin manifests go in `.claude-plugin/`.
Use `cowork install --platform claude-code` or rely on the full install which
auto-detects Claude Code from the running environment.

### Codex
Skills land in `~/.codex/skills/`. Codex reads `agents.toml` for skills config.
`cowork install --platform codex` runs `cowork codex-config` to merge entries.

### OpenCode
Skills go in `~/.opencode/skills/`. OpenCode reads a `package.json` in the
skills directory.  `cowork install --platform opencode` calls `opencode-config`
to wire the plugin.

### Kimi Code CLI
Skills install to `~/.kimi-code/skills/`.  `config.toml` in `~/.kimi-code/`
is updated by cowork to register MCP servers and skill paths.

### Zed
Skills copy to `~/.config/zed/skills/`.

## Prometheus-skill-pack integration

The prometheus-skill-pack ships `cowork` as a first-class binary (installed by
`scripts/install-binaries.sh`). The cowork submodule lives at
`tools/cowork-skills` in the pack repo.

To update the pack across all platforms on a machine:

```bash
# One-shot: pull latest, rebuild cowork, reinstall everywhere
cowork pack update

# If install-binaries.sh itself changed, rebuild the binary first:
cd tools/cowork-skills/cli && cargo build --release
install_bin target/release/cowork ~/.local/bin/cowork
cowork pack update
```

## Detailed reference

- [Full command reference](references/COMMANDS.md)
