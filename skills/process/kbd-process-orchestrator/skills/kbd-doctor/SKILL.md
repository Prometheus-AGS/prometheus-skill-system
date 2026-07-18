---
name: kbd-doctor
description: Diagnose Prometheus substrate health and orchestrate safe doctor runs through the compiled `prometheus doctor` CLI. Use when the user asks for doctor, health checks, repair planning, refresh planning, or recovery of KBD learning-loop infrastructure.
version: '1.0.0'
license: MIT
argument-hint: "[--json] [--check <id-or-group>] [--fix|--refresh] [--dry-run] [--yes]"
metadata:
  category: process
  tags: [kbd, doctor, repair, refresh, health, recovery, cli]
---

# /kbd-doctor

Run the Prometheus substrate doctor through the canonical CLI surface.

## Source of truth

Always use `prometheus doctor` as the diagnostic authority. Do not reimplement
checks in the skill.

## Before broad repair work

Before any broad `--fix` or `--refresh` action:

1. Verify `~/.prometheus/repair/karpathy-ready.json` exists and `ready` is `true`.
2. Run `pk focus "prometheus doctor repair context"` to load prior repair context.
3. If the readiness artifact is absent or false, stop and return to the Karpathy
   gate repair task before attempting wider recovery.

## Commands

Read-only diagnosis:

```bash
prometheus doctor
prometheus doctor --json
prometheus doctor --check learning
prometheus doctor --check learning.surreal-memory
```

Dry-run repair planning:

```bash
prometheus doctor --fix --dry-run
prometheus doctor --refresh --dry-run
```

Confirmation-gated repair request:

```bash
prometheus doctor --fix
prometheus doctor --refresh
```

These commands print the repair plan and stop at the deny-by-default boundary
unless `--yes` is supplied.

Non-interactive safe repair:

```bash
prometheus doctor --fix --yes
prometheus doctor --refresh --yes
```

## Safety policy

- Safe and reversible actions may be automated.
- Manual-only findings stay manual: credentials, unknown client sections,
  dirty submodules, warehouse deletions, unmanaged LaunchAgents, and any action
  that crosses a deny-by-default boundary.
- `--yes` suppresses prompts only for safe reversible actions.

See:

- `references/repair-policy.md`
- `references/check-catalog.md`

## Required writeback

After any substantive doctor run, write back a reflection in Delta → Root Cause
→ Corrective Actions form. The writeback must capture:

- what failed or degraded;
- what root cause was confirmed;
- what safe actions were taken versus deferred manual actions;
- where backups or rollback artifacts were written.

Successful `--refresh --yes` runs also write:

- `.prometheus/repair/install-refresh-manifest.json`

## Thin wrapper

If a wrapper script is needed, keep it Bash 3.2-compatible and delegate
directly to the CLI.
