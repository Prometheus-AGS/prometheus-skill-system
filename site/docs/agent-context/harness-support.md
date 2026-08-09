---
title: Harness Support
description: How the structure behaves in Claude Code, Codex, Cursor, Cline, Roo, Kilo, Gemini CLI, Windsurf, OpenCode, Zed, and UAR agents.
---

# Harness Support

One structure, many readers. What every harness gets, what only some get, and
what to check when a harness does something unexpected.

## The portability contract

`AGENTS.md` is the single source of truth. It is plain markdown with no required
schema, which is the whole point of the AGENTS.md convention — a README for
agents, readable by anything that reads files.

`CLAUDE.md` is a **symlink** to it. One file on disk, two names, no drift and no
double-load.

Everything below the root file is progressively more harness-specific:

| Layer | Portability |
|---|---|
| `AGENTS.md` | universal — every listed harness |
| `.claude/rules/*.md` | Claude Code path-scoping; readable as plain docs elsewhere |
| `.claude/hooks/*.sh` | Claude Code events; plain shell, callable from CI or a git hook |
| `.claude/agents/*.md` | Claude Code subagents |
| `.claude/settings.json` | Claude Code only |
| `.prometheus/`, `versions.toml`, `.kbd-orchestrator/` | universal — plain files any agent can read |

A harness that understands none of `.claude/` still gets the invariants, the
authority files, and the learning directory. It loses enforcement, which is
exactly why the `mixed` profile keeps the scaffold in prose.

## Claude Code

The only harness that uses every layer.

| Feature | Behavior |
|---|---|
| Root file | Reads `CLAUDE.md`. The symlink resolves to `AGENTS.md` |
| Compaction | Re-reads the project-root file from disk and re-injects it |
| Path-scoped rules | `.claude/rules/*.md` with a `paths:` glob, loaded on matching file read |
| Hooks | All four wired in `settings.json` |
| Subagents | `artifact-critic` with artifact-only context |
| Skill budget | `skillListingBudgetFraction`, default 0.01 |

**Claude Code does not natively read `AGENTS.md`.** This is the single most
important interop fact and the reason for the symlink. A long-standing feature
request tracks it; until it lands, the symlink (or an `@AGENTS.md` first line) is
the documented workaround.

`settings.local.json` overrides `settings.json`. If hooks do not fire after
bootstrap, check whether a local file is shadowing the hooks block.

## Codex

Reads `AGENTS.md` natively — the convention originated there. Nearest-file-wins in
monorepos, so a nested `AGENTS.md` governs its subtree.

Gets: invariants, project rules, authority files, `.prometheus/`.
Does not get: hooks, path-scoped rules, subagents.

The repo's `.codex/` directory holds prompts and commands and is untouched by the
bootstrap.

## Cursor, Windsurf, Cline, Roo Code, Kilo Code

All read `AGENTS.md`. Cursor additionally supports `.cursor/rules/*.mdc` with its
own frontmatter; the bootstrap does not write those, and an existing set keeps
working alongside `AGENTS.md`.

Cline and Roo read `.clinerules/` and `.roo/` respectively for workflows. Those are
command definitions, not rule files, and the bootstrap leaves them alone.

Gets: invariants and project rules.
Does not get: hooks, Claude-specific path scoping, subagents.

**Watch for duplication.** If a repo already carries the same rules in
`.cursor/rules/` and in `AGENTS.md`, that is two constitutions by another route.
Pick one home.

## Gemini CLI

Reads `AGENTS.md`. Some versions also look for `GEMINI.md`. If your workflow uses
it, symlink it the same way:

```bash
ln -s AGENTS.md GEMINI.md
```

Do not maintain a copy. A copy diverges, and divergence between two rule files is
the failure this architecture exists to remove.

## OpenCode, Amp, Jules, Devin, Factory, Aider, Zed

All consume `AGENTS.md`. Zed has no native rules integration for the OpenSpec-style
workflows, so the CLI plus `AGENTS.md` is the working combination — which is what
the portable root file is for.

Zed workspace tooling sometimes writes a `<!-- zed-workspace:begin -->` region into
the agent file. The migration **preserves** that region verbatim; see
[Use Cases](./use-cases) case 5.

## UAR and liter-llm-routed agents

Prometheus agents running through the Universal Agent Runtime read `AGENTS.md` as
plain context. Because the runtime selects the model before building the prompt, it
is the one place per-model conditioning is genuinely possible — attach the
execution scaffold only when the target model needs it, and serve `lean` bytes to a
frontier model from the same repository.

Until that is wired, the file serves the weakest reader. See
[Model Profiles](./model-profiles).

## Hooks outside Claude Code

The hooks are plain bash reading JSON on stdin and signalling with exit codes. They
are not Claude-specific in any way that matters, and the enforcement they provide is
worth keeping in harnesses that cannot call them.

Two portable options:

**Git pre-commit.** Wire `sycophancy-gate.sh` or a tier check into a pre-commit
hook so the gate applies regardless of which agent produced the change.

**CI.** Run `verify.sh` in the pipeline. It exits 1 on FAIL and 0 on WARN, so it
works as a gate without going red on the machine-wide skill budget.

```yaml
- name: Verify agent context
  run: bash .claude/skills/prometheus-context-bootstrap/scripts/verify.sh --path .
```

## Multi-harness repositories

The realistic case: a repo worked by Claude Code, Codex, and a Cursor user, with
Zed tooling writing its own region.

What holds it together:

1. **One source of truth.** `AGENTS.md`, symlinked to every other name a harness
   expects. Never copied.
2. **`mixed` profile.** Not every reader is frontier-class, and the file cannot
   branch.
3. **Tool regions preserved.** Each tool's fenced region survives migration with
   markers intact, so re-injection keeps working.
4. **Authority in plain files.** `versions.toml` and
   `.kbd-orchestrator/current-waypoint.json` are readable by anything.

What does not hold together automatically: enforcement. Only Claude Code runs the
hooks. In a mixed-harness repo, put the checks that must not be skipped into CI as
well.

## Nested agent files

Claude Code loads a nested `AGENTS.md` on demand when reading files in its
directory. Vendored packages and submodules that ship their own agent files will
pull that content into context alongside your root rules.

Measured example: a vendored package carrying a 5,041-word `AGENTS.md` — working
inside that subtree loads it next to the root file, which is the
duplicate-constitution problem returning through a path the migration does not
scan.

The bootstrap does not touch vendored content, and should not. Record the effect in
`.prometheus/gotchas.md` and fix it upstream in the package that ships it.

## Compatibility summary

| Harness | Root file | Path rules | Hooks | Subagents |
|---|---|---|---|---|
| Claude Code | via symlink | yes | yes | yes |
| Codex | native | no | no | no |
| Cursor | native | own format | no | no |
| Windsurf | native | no | no | no |
| Cline | native | no | no | no |
| Roo Code | native | no | no | no |
| Kilo Code | native | no | no | no |
| Gemini CLI | native (+symlink) | no | no | no |
| OpenCode | native | no | no | no |
| Zed | native | no | no | no |
| Aider, Amp, Jules, Devin, Factory | native | no | no | no |
| UAR / liter-llm | native | no | runtime | runtime |

"Native" means the harness reads `AGENTS.md` without configuration. Claude Code is
the exception that needs the symlink.
