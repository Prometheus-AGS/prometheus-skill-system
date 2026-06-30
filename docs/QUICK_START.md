# Quick Start — Prometheus Skill Pack

Get from zero to your first `/learn-goal` session in under 10 minutes.

---

## Prerequisites

Three things must be installed before you start:

| Tool | Minimum | Install |
|------|---------|---------|
| **Rust** | stable (1.75+) | `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \| sh` |
| **Node.js** | 18+ | https://nodejs.org |
| **Git** | any recent | https://git-scm.com |

---

## Step 1 — Clone (with submodules)

```bash
git clone --recurse-submodules https://github.com/Prometheus-AGS/prometheus-skill-system.git
cd prometheus-skill-system
```

> **Already cloned without `--recurse-submodules`?**
> ```bash
> git submodule init && git submodule update
> ```

---

## Step 2 — Install

One script installs skills to all detected platforms (Claude Code, Kimi, OpenCode, etc.)
and builds the Rust tool binaries (`forge`, `pk-watcher`, `sovereign-sync`):

```bash
bash scripts/install-skills-flat.sh
```

This takes 2–5 minutes on first run (Rust compilation). Subsequent runs are fast.

---

## Step 3 — Verify

```bash
bash shared/scripts/detect-toolchain.sh
```

Every row should show ✓ or a version number. If `forge` shows `MISSING`, the Rust
build in Step 2 did not complete — check for errors in its output and retry.

---

## Step 4 — Open Claude Code

Open the `prometheus-skill-system/` directory in Claude Code:

```bash
# From the directory you cloned into:
claude .
```

Claude Code loads the skills automatically. You do not need to run any install command
inside Claude Code.

---

## Step 5 — Run your first learning session

In the Claude Code chat, type:

```
/learn-goal "explain recursion to a 10-year-old"
```

What you should see:

1. `learn-goal` assesses feasibility (GREEN/YELLOW/RED) and asks a few scoping questions
2. `learn-survey` places your starting level
3. `learn-plan` builds a concept dependency graph
4. `feynman-loop` runs the first explanation cycle
5. `learn-grade` scores your explanation — sycophancy-corrected, not a pat on the back

The full Feynman loop (survey → plan → explain → grade → retain) is the core of the
learning domain. From here, explore:

- `/learn-retain` — schedule spaced repetition for concepts you've covered
- `/learn-practice` — derivation and transfer exercises
- `/learn-kb add local:/path/to/notes` — bring in your own content as a KB adapter

---

## Troubleshooting

**`forge: command not found`** — the binary did not land in `~/.local/bin`. Add it to your PATH:
```bash
export PATH="$HOME/.local/bin:$PATH"
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.zshrc
```

**`learn-goal` skill not found** — Claude Code is not loading the skill pack. Confirm
`claude .` was run from the `prometheus-skill-system/` directory and that
`~/.claude/skills/` is populated:
```bash
ls ~/.claude/skills/ | grep learn
```

**MCP services not running** — surface-bridge (port 7890) and sovereign-sync (port 7892)
are launchd services on macOS. Start them with:
```bash
bash scripts/prometheus-services.sh load
bash scripts/prometheus-services.sh status
```

---

## Next steps

| Goal | Read |
|------|------|
| Full install reference | [docs/guide/19-installation.md](guide/19-installation.md) |
| Self-improving loop (forge enrich → reflect) | [docs/guide/14-rust-toolchain.md](guide/14-rust-toolchain.md) |
| P2P sync validation | [docs/SOVEREIGN_SYNC_TESTING.md](SOVEREIGN_SYNC_TESTING.md) |
| Contribute a skill | [docs/CONTRIBUTING.md](CONTRIBUTING.md) |
| Report an issue | [GitHub Issues](https://github.com/Prometheus-AGS/prometheus-skill-system/issues) |
