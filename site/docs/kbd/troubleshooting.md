---
id: troubleshooting
title: Troubleshooting
sidebar_label: Troubleshooting
---

# KBD Troubleshooting

## Fast diagnostic sequence

```bash
PROJECT_ROOT="/path/to/project"

# 1. Project identity
jq . "$PROJECT_ROOT/.prometheus/project.json"

# 2. Local CLI state
prometheus kbd --path "$PROJECT_ROOT" status --json | jq .

# 3. Installation diagnosis
prometheus doctor --json | jq .
```

## KBD mutation receives `401`

KBD mutation POSTs reject unsigned, tampered, unknown-device, and revoked-device
envelopes. Confirm that the client signed a schema-v2 command with the current
device key, that the key ID is enrolled and active, and that the command was
not changed after signing. The removed bearer-token setting is not a remedy.

## KBD route receives `404`

`unknown KBD project` means no registered replica resolves to the requested
UUID. Register an existing manifest-bearing checkout and retry; do not infer or
rewrite its UUID from Git evidence.

`kbd runtime is not initialized` means the project is registered but no
committed state exists. Inventory and apply migration:

```bash
prometheus kbd --path "$PROJECT_ROOT" migrate --check
prometheus kbd --path "$PROJECT_ROOT" migrate --apply
```

## Lifecycle looks wrong

Bash is no longer gated by KBD state, so a stale lifecycle cannot block a shell
command (see [Tool guards](./bash-mutation-guard)). It can still cause the
control plane to reject a `prometheus kbd` command. Read the lifecycle and
checkpoint:

```bash
prometheus kbd --path "$PROJECT_ROOT" status --json |
  jq '{lifecycle, checkpoint, exactNextWork}'
```

- Suspended lifecycle: audit and resume explicitly.
- Terminal lifecycle: start a new run/phase.

## Connected service unavailable

Ordinary local status and mutations do not need a sync daemon. Inspect `prometheus contract show --json` for configured discovery and use the installed Companion release's own service/identity runbook. Pack service installation does not repair it. Do not create new keys or delete journals to clear a health failure.

## Runtime reports an integrity conflict

Do not delete the journal or overwrite projections. Capture:

```bash
prometheus kbd --path "$PROJECT_ROOT" audit --json > kbd-audit.json
prometheus kbd --path "$PROJECT_ROOT" status --json > kbd-status.json
```

Then inspect the local runtime diagnostics and, if applicable, the separately owned connected endpoint. Divergent
offline branches, invalid signatures, and revoked devices are safety failures
that require audit—not a forceful file repair.
