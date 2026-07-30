# 16a · cowork

`cowork` is the CLI that installs, updates, and repairs the skill pack across
every supported platform, and the one that finds skills you do not have yet.

It is installed to `~/.local/bin/cowork` by `scripts/install-binaries.sh`.

## Read this first: the security posture

`cowork` can install code from GitHub onto your machine. Two of its subcommands
— `install` and `generate` — execute third-party content. Everything below
assumes these rules:

> **Never auto-install a skill you discovered.** `cowork search` answers a
> question; `cowork install` runs someone else's code. Keep those two decisions
> separate, and make the second one deliberately.

**`cowork audit` cannot vet a candidate before you install it.** It scans
*installed* skills and takes no repository argument:

```console
$ cowork audit databasus/databasus
error: unexpected argument 'databasus/databasus' found
```

There is also no `--dry-run` on `install`. So the only pre-execution control is
**reading the source yourself**. After that, the safe order is:

1. Read the repository — what it does, its licence, what its `scripts/` would run.
2. `cowork install <owner/repo> --agent claude-code` — **project scope**, so
   removal is a directory delete and the blast radius is one repository.
3. `cowork audit --project --format json` — scan what is now on disk.
4. `cowork verify` — confirm the installed bytes match the lockfile.

A failing audit means uninstall (`cowork install --uninstall <owner/repo>`), not
"note it and continue". And note what these tools do *not* give you: `audit` is a
scanner, so a clean result means known patterns were absent, not that the skill is
safe; `verify` proves integrity, not intent.

## Discovery: build or adopt?

Before building a skill, check whether one already exists:

```bash
bash skills/process/cowork-management/scripts/discover-skills.sh \
  --capability "postgres backup verification" --limit 5 --out candidates.json
```

This wraps `cowork search` and emits structured candidates
(`skill-candidates/v1`). Every entry is fixed at `verdict: "unevaluated"` by
construction — search relevance is not evaluation, and `stars` is a popularity
signal, never a safety one. The script **never installs**; it prints the adoption
path and stops.

It is deliberately a *different* document from `library-candidates.json`. That
schema constrains `kind`/`registry`/`verdict` to package-shaped enums
(`library`, `npm`, `adopt`) and sets `additionalProperties: false`. Forcing a
GitHub skill repo into it would require claiming an adoption verdict nobody made,
turning an unreviewed search hit into what looks like a vetted decision.

When nothing is found, that is a real answer: build is the remaining option.

Full flow: [`adopting-external-skills.md`](https://github.com/Prometheus-AGS/prometheus-skill-system/blob/main/skills/process/cowork-management/references/adopting-external-skills.md).

## Subcommands

### Pack management

| Command | Effect |
|---|---|
| `cowork pack status` | skill-pack version and installed skill counts per platform |
| `cowork pack update` | re-run `install-skills-flat.sh` across all platforms |
| `cowork pack repair` | detect broken symlinks and repair affected platforms |

`pack repair` is the first thing to try when a platform stops seeing skills.

### Health

| Command | Effect |
|---|---|
| `cowork status` | current status and configuration |
| `cowork doctor` | check for configuration issues |
| `cowork toolchain status` | full toolchain health (Rust, binaries, MCP services) |
| `cowork toolchain check` | exit 0 if all required tools present, 1 otherwise — CI-friendly |
| `cowork toolchain install <tool>` | print install instructions for one tool |

`toolchain check` is the one to wire into CI; `toolchain status` is for humans.

### Skills

| Command | Effect |
|---|---|
| `cowork init` | install built-in skills |
| `cowork list` | list all available skills |
| `cowork search <query>` | search GitHub for skill repositories (`-n/--limit`, default 10) |
| `cowork install <owner/repo>` | install from GitHub — **executes third-party code** |
| `cowork generate <owner/repo>` | generate skills from a repository or local directory |
| `cowork test triggers` | list all triggers with their skills |

`install` accepts `--agent` (16 targets including `claude-code`, `codex`,
`cursor`, `opencode`, `windsurf`), `--skill` to install specific skills,
`--plugin` to preserve the whole repository structure, and `--uninstall` to
reverse.

### Security

| Command | Effect |
|---|---|
| `cowork audit --global\|--project\|--plugins` | security audit of **installed** skills (`--format text\|json\|markdown`) |
| `cowork verify [skill]` | verify checksums against the lockfile (`--update` to re-record) |

Run `verify` again after any `cowork install --update`: an update legitimately
changes the bytes, and the lockfile has to be told so.

### Project configuration

| Command | Effect |
|---|---|
| `cowork config init` | create `skills.toml` in the project |
| `cowork config show` | show current configuration |
| `cowork config add` / `remove` | manage a skill dependency or plugin |
| `cowork config install` | install everything declared in `Skills.toml` |
| `cowork config sync` | sync `Skills.lock` with `Skills.toml` |

This is the reproducible path: declare dependencies in `skills.toml`, commit the
lockfile, and `cowork config install` gives every machine the same set.

### Plugins

| Command | Effect |
|---|---|
| `cowork plugins install <git-url>` | install a Claude Code plugin |
| `cowork plugins list` | list marketplace plugins installed via `/plugin` |
| `cowork plugins status` | plugin system status |
| `cowork plugins enable` / `disable` / `uninstall` | manage an installed plugin |

### Disk

| Command | Effect |
|---|---|
| `cowork disk status` | disk usage summary (delegates to `dsg status --json`) |
| `cowork disk scan` | scan for reclaimable space |
| `cowork disk clean` | clean reclaimable space — preview with `--dry-run` first |

`disk clean --force` moves artifacts to the system Trash rather than deleting
them, so a mistake is recoverable.

## See also

- [08 · Skills Overview](08-skills-overview.md) — what a skill is
- [16 · CLI and Scripts](16-cli-and-scripts.md) — the other pack CLIs
- [17 · Platform Support](17-platform-support.md) — the install targets
- [18 · Plugins and Marketplace](18-plugins-and-marketplace.md) — plugin distribution
