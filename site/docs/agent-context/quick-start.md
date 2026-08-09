---
title: Quick Start
description: Install the agent context structure into a new or existing project, and verify it.
---

# Quick Start

Three scripts. Report first, apply second, verify third.

```bash
SK=~/.claude/skills/prometheus-context-bootstrap
# or, from a checkout:
SK=path/to/prometheus-skill-pack/skills/process/prometheus-context-bootstrap
```

## New project

```bash
cd my-project
bash "$SK/scripts/bootstrap.sh" --path . --dry-run   # plan only
bash "$SK/scripts/bootstrap.sh" --path .
bash "$SK/scripts/verify.sh"    --path .
```

Stacks are detected from `Cargo.toml`, `package.json`, `pubspec.yaml`, `go.mod`,
and `pyproject.toml`. Override with `--stacks rust,typescript`.

## Existing project with a Prometheus Base Rules v3 file

`bootstrap.sh` **exits 2** on a v3 file rather than appending to it. Two
constitutions in resident context degrade adherence to both, so it refuses and
points at the migration.

```bash
bash "$SK/scripts/migrate.sh" --path .            # report, writes nothing
bash "$SK/scripts/migrate.sh" --path . --apply    # archive, then bootstrap
bash "$SK/scripts/verify.sh"  --path .
```

Read `.prometheus/MIGRATION-REPORT.md` before committing. It lists every project
section the migration deliberately did not place.

## What gets written

```
AGENTS.md                    # single source of truth
CLAUDE.md                    # symlink to AGENTS.md
.claude/settings.json        # permissions, skill budget, hook wiring
.claude/hooks/*.sh           # tier-guard, single-writer, sycophancy-gate, reanchor
.claude/rules/<stack>.md     # path-scoped; free until a matching file is read
.claude/agents/artifact-critic.md
.prometheus/                 # session-log, decisions, gotchas, postmortems/, knowledge/
.prometheus/model-fleet.md   # which models read this repo
.kbd-orchestrator/current-waypoint.json
versions.toml                # stub, if absent
```

## Flags

| Flag | Meaning |
|---|---|
| `--path <dir>` | Project root. Default `.` |
| `--dry-run` | Print the plan and agent-file diffs. Write nothing |
| `--force` | Re-copy hooks, rules, settings. Never touches operator prose |
| `--stacks a,b` | Override detection |
| `--profile p` | `mixed` (default), `strict`, `lean`. See [Model Profiles](./model-profiles) |
| `--no-hooks` | Skip hooks and the settings hook block |

| Exit | Meaning |
|---|---|
| 0 | Applied, or dry run completed |
| 1 | Usage error |
| 2 | Refused: v3 file, or corrupt marker pair |

## Three write modes

Every path falls into exactly one, and the report names which applied.

**CREATE** — absent, written whole.

**SPLICE** — `AGENTS.md` exists and is not v3. Only the region between
`<!-- prometheus-base:start v1 -->` and `<!-- prometheus-base:end -->` changes.
Written to a temp file and moved, so an interrupted run never leaves a
half-rewritten agent file.

**SKIP** — exists and is not marker-managed. Reported, untouched. `--force`
promotes SKIP to CREATE for hooks, rules, and settings only — never for agent
prose or `.prometheus/` history.

## Verify

```bash
bash "$SK/scripts/verify.sh" --path .
```

```
PASS  AGENTS.md markers                  well-formed
PASS  AGENTS.md size                     1396 managed words (profile mixed, ceiling 1500)
PASS  CLAUDE.md                          symlink -> AGENTS.md (no double load)
PASS  hooks executable                   4 hook(s)
PASS  tier-guard wired
WARN  skill budget measured              2267 skills, ~163254 tok vs ~4000 — 40.8x OVER
PASS  .prometheus layout                 complete

PASS 10   FAIL 0   WARN 2   SKIP 0
```

Four states, and the distinction matters.

**PASS** — checked and holds.
**FAIL** — checked and does not hold. Exit 1.
**WARN** — a real finding this repo cannot fix alone, such as the machine-wide
skill budget. Does not change the exit code, because a gate that always fails
stops being read.
**SKIP** — could not run. Never counted as PASS. A check that did not run is
unverified, and reporting it as passing is how a gate becomes decorative.

## After bootstrap

```bash
/kbd-init                                    # project.json + constraints.md
bash "$SK/scripts/skill-budget.sh" --path .  # measure, do not assume
```

Then write the project-specific content: real pins in `versions.toml`, real build
and test commands in `.claude/rules/<stack>.md`, the fleet in
`.prometheus/model-fleet.md`, and the first entry in `.prometheus/decisions.md`.

The bootstrap installs the frame. The content is yours.
