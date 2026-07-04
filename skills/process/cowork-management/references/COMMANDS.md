# cowork — Full Command Reference

All commands are available as `cowork <command>` or via the `co` short alias.

## Global help

```bash
cowork --help
cowork --version
cowork <command> --help
```

---

## init — Install built-in skills

```bash
cowork init                          # Install all built-in skills (global ~/.claude/skills/)
cowork init --local                  # Install to project .claude/skills/
cowork init --list                   # List available built-in skills without installing
cowork init -s memory-skills         # Install specific skill only
cowork init --remove memory-filesystem  # Remove a skill
cowork init --force                  # Force overwrite existing skills
```

| Flag | Short | Description |
|------|-------|-------------|
| `--list` | | List built-in skills without installing |
| `--skill NAME` | `-s` | Install specific skill (repeatable) |
| `--remove NAME` | `-r` | Remove skill (repeatable) |
| `--force` | `-f` | Overwrite existing |
| `--local` | `-l` | Project-local install |

---

## install — Install from GitHub or local source

```bash
cowork install                              # Install current project skills
cowork install user/repo                    # Install from GitHub
cowork install user/repo -s skill1          # Install specific skills only
cowork install user/repo --plugin           # Install as full plugin (preserves repo structure)
cowork install user/repo -a claude-code     # Install to specific agent
cowork install user/repo -a codex -a opencode  # Install to multiple agents
cowork install --list                       # List installed repos
cowork install --uninstall repo             # Uninstall a repo
cowork install user/repo --reinstall        # Remove + reinstall
cowork install user/repo --update           # Git pull + reinstall
cowork install user/repo --no-symlink       # Copy instead of symlink
cowork install user/repo --local            # Install to project scope
```

**Supported agent targets** (`-a` / `--agent`):
`amp`, `antigravity`, `claude-code`, `clawdbot`, `codex`, `cursor`, `droid`,
`gemini-cli`, `github-copilot`, `goose`, `kilo`, `kiro-cli`, `kimi`, `minimax`,
`opencode`, `roo`, `trae`, `windsurf`, `zed`

---

## pack — Prometheus-skill-pack lifecycle

```bash
cowork pack status          # Show pack root, git status, platform links
cowork pack update          # git pull + re-install to all platforms
cowork pack repair          # Fix broken symlinks, re-run install-binaries.sh
```

`pack status` auto-detects the pack root from `PROMETHEUS_PACK_ROOT` env var, or
searches upward from the current directory for a `.kbd-orchestrator/` sentinel.

---

## toolchain — Toolchain health

```bash
cowork toolchain status          # Check Rust, Node.js, git, cargo-dist versions
cowork toolchain install         # Install missing toolchain components
cowork toolchain update          # Update all managed toolchain tools
```

Checks include: `rustc`, `cargo`, `node`, `npm`, `git`, `cargo-dist`, `trash`.

---

## disk — Disk-space reclamation (delegates to dsg)

```bash
cowork disk status                           # Summary of known artifact roots
cowork disk scan                             # Show reclaimable build artifacts
cowork disk scan --deep                      # Deep scan all home subdirectories
cowork disk scan --ecosystem rust            # Rust-only (target/, .cargo/)
cowork disk scan --ecosystem node            # Node-only (node_modules, .npm/)
cowork disk scan --ecosystem python          # Python-only (__pycache__, .venv)
cowork disk scan --ecosystem go              # Go-only (go/pkg/mod/)
cowork disk clean --dry-run                  # Preview what would be trashed (DEFAULT)
cowork disk clean --force                    # Move stale artifacts to Trash
cowork disk clean --force --ecosystem rust   # Rust-scoped clean
```

**Safety**: `--dry-run` is the default. `--force` is required to actually move
anything. All deletes go via system Trash — nothing is permanently removed.

Delegates to the `dsg` binary (disk-space-guardian). If `dsg` is not in PATH,
`cowork disk` falls back to a basic report.

---

## plugins — Plugin marketplace management

```bash
cowork plugins list                         # List installed plugins
cowork plugins install <git-url>            # Install a plugin from a Git repo
cowork plugins install <git-url> --local    # Project-local plugin
cowork plugins uninstall <plugin-id>        # Remove plugin
cowork plugins enable <plugin-id>           # Enable disabled plugin
cowork plugins disable <plugin-id>          # Disable without uninstalling
cowork plugins status                       # Health of all plugins
cowork plugins list-marketplaces            # Show configured marketplace sources
cowork plugins remove-marketplace <name>    # Remove marketplace source
```

Plugin install fetches the repo, reads `plugin.json`, copies `skills/`,
`agents/`, `hooks/` into the appropriate platform directories.

---

## config — Per-project skill configuration

```bash
cowork config init                          # Create .cowork.toml in project root
cowork config init --auto-detect            # Auto-detect platforms and agents
cowork config show                          # Show current config
cowork config enable <skill>                # Enable skill for this project
cowork config disable <skill>               # Disable skill
cowork config add <name> <source>           # Add custom skill source
cowork config remove <name>                 # Remove skill source
cowork config sync                          # Pull latest from all remote sources
cowork config sync --update-remotes         # Also update remote source refs
cowork config apply                         # Apply config (run install-deps, symlinks)
cowork config list-groups                   # Show available skill groups
cowork config priority <skill1> <skill2>    # Set skill priority order
cowork config override <trigger> <skill>    # Override auto-trigger for a pattern
```

---

## generate — Generate skills from code

```bash
cowork generate user/repo                   # Generate from GitHub repo
cowork generate --path ./my-project         # Generate from local directory
cowork generate user/repo --lang rust       # Specify language(s)
cowork generate user/repo --llms-only       # Only generate llms.txt
cowork generate --from-llms ./llms.txt      # Generate from existing llms.txt
cowork generate user/repo -o ./output       # Specify output directory
cowork generate user/repo --ref v1.0.0      # Specific git ref
```

---

## search — Search the agentskills.io registry

```bash
cowork search <query>                       # Full-text search
cowork search rust --limit 20              # Limit results
cowork search --agent-skills               # Search agentskills.io specifically
```

---

## list — List installed skills

```bash
cowork list                                 # All skills (project + global)
cowork list -t global                       # Global only (~/.claude/skills/)
cowork list -t project                      # Project only (.claude/skills/)
cowork list --verbose                       # Detailed info per skill
```

---

## status — Overall status

```bash
cowork status                               # Platform links, skill counts, health
```

---

## doctor — Diagnose and repair

```bash
cowork doctor                               # Full health check + fix suggestions
```

Checks:
- Platform skill directories exist with correct permissions
- All symlinks resolve (no dangling refs)
- Installed binary versions in expected ranges
- `cowork.toml` / `.cowork.toml` schema valid
- Platform configs present (`agents.toml` for Codex, `package.json` for OpenCode,
  `config.toml` for Kimi)

---

## audit — Security audit

```bash
cowork audit                                # Audit all installed skills
cowork audit --verbose                      # Detailed findings
cowork audit --format json                  # JSON output
cowork audit -o report.md                   # Save to file
cowork audit --fix                          # Auto-fix where possible
```

Scans for: credential patterns, dangerous shell patterns, prompt injection
signatures, overly broad file permissions.

---

## verify — Verify skill checksums

```bash
cowork verify                               # Verify all skills against Skills.lock
cowork verify rust-skills                   # Verify specific skill
cowork verify --update                      # Update checksums in lockfile
```

---

## Platform configuration helpers

These are called internally by `install` but can be invoked directly:

```bash
# Codex: merge agents.toml, set goal templates
cowork codex-config

# OpenCode: ensure package.json in skills dir, register plugin
cowork opencode-config
```

---

## Environment variables

| Variable | Effect |
|----------|--------|
| `PROMETHEUS_PACK_ROOT` | Override auto-detected pack root |
| `COWORK_LOG` | Log level: `error`, `warn`, `info`, `debug`, `trace` |
| `COWORK_NO_COLOR` | Disable ANSI color output |
| `DSG_BIN` | Override path to `dsg` binary (used by `cowork disk`) |
