---
id: harness-handoffs
title: Harness Handoffs
sidebar_label: Harness Handoffs
---

# Harness Handoffs

A KBD phase can move between harnesses mid-flight: planned in Claude Code,
executed in Kimi Code, reviewed in Codex. `/kbd-handoff` prepares that move.
It writes one packet the operator pastes into the receiving harness.

KBD has no writer lease. Each `prometheus kbd` command is serialized by the
append-only journal lock and validated against the expected revision, so a
handoff transfers no ownership. The sending harness stops issuing commands;
the receiving harness verifies state and continues.

## Run it

```bash
timeout 120 ~/.claude/skills/kbd-handoff/kbd-handoff.sh --to kimi-code </dev/null
timeout 120 ~/.claude/skills/kbd-handoff/kbd-handoff.sh --to codex --dry-run </dev/null
```

`--to` is required and must name a supported harness: `claude-code`, `codex`,
`kimi-code` or `opencode` (aliases `claude`, `kimi`); anything else exits 2.
`--phase` selects a phase other than the active one, and `--repo` adds a
repository (for example a fork) to the report.

## What it refuses

It exits 1 on a half-closed boundary in `prometheus kbd status --json`: an
open `task` obligation, or a `change` or `phase` obligation the runtime marks
complete or does not know. Repair each one, then retry:

```bash
prometheus kbd guard evaluate --boundary change --edge after \
  --subject <change> --json --repair-projections
```

Obligations on the active phase path, and on the active change while it is
not complete, are expected and ignored. A change or phase outside the active
position that was started and not finished is reported as carried work. A
missing phase or phase directory also exits 1.

A dirty or unpushed worktree, a broken installed skill, or a missing native
role definition produces a warning recorded in the packet, not a refusal.

## What it writes

```text
.kbd-orchestrator/phases/<phase>/harness-handoffs/<harness>-<UTC>.md
.kbd-orchestrator/phases/<phase>/harness-handoffs/<harness>-<UTC>.json
```

Stage handoffs live in `handoffs/` under a different schema; harness handoffs
never touch them. The JSON sidecar follows
`references/schemas/harness-handoff.schema.json` in the orchestrator skill.

The packet contains the position (phase, stage, implementation N of T, next
change and task), repository branches with clean and pushed state, files to
read in order, the plan's operator gates and decisions, task execution rules,
a never-do list quoted from the project's `AGENTS.md`, open prerequisites,
and the exact start prompt for the receiving harness.

The skill is read-only on KBD state. It never edits `progress.json`, the
waypoint, position files or task checkboxes.

## Broken installed skills

The handoff checks every `kbd-*` entry in the target harness's skills
directory. A symlink that points at a removed path, such as the
`kbd-handoff` link left from the former lease skill, is reported with its
target. Reinstall the pack to repair it:

```bash
./install.sh
```
