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

# 2. Local daemon health (no token required)
curl --fail-with-body http://127.0.0.1:7892/health | jq .

# 3. CLI state
prometheus kbd --path "$PROJECT_ROOT" status --json | jq .

# 4. Service logs on macOS
tail -n 100 "$HOME/.prometheus/logs/sovereign-sync.stderr.log"

# 5. Full installation diagnosis
prometheus doctor --json | jq .
```

## Token receives `401`

The running daemon loaded a different token.

Check:

- project ID used to derive the token path;
- `PROMETHEUS_CONTROL_TOKEN_FILE` in the daemon;
- `PROMETHEUS_CONTROL_TOKEN_FILE` in the harness;
- whether the token was rotated after the daemon started;
- whether Sovereign Sync is focused on the intended repository.

Restart Sovereign Sync after correcting the path. Do not copy `operator_id`
into the bearer-token file.

## KBD route receives `404`

`unknown KBD project` means the daemon is focused on another project. Set
`KBD_FOCUS_PROJECT_PATH` and reload the service.

`kbd runtime is not initialized` means authentication succeeded, but no
committed state exists. Inventory and apply migration:

```bash
prometheus kbd --path "$PROJECT_ROOT" migrate --check
prometheus kbd --path "$PROJECT_ROOT" migrate --apply
```

## Lifecycle or lease looks wrong

Bash is no longer gated by KBD state — a stale lifecycle or lease cannot block a
shell command (see [Tool guards](./bash-mutation-guard)). It can still cause the
control plane to reject a `prometheus kbd` command. Read the lifecycle and lease
independently:

```bash
prometheus kbd --path "$PROJECT_ROOT" status --json |
  jq '{lifecycle, owner: .lease.owner.harness, fence: .lease.fencingToken}'
```

- Suspended lifecycle: audit and resume explicitly.
- Terminal lifecycle: start a new run/phase.
- No lease: claim for the active harness.
- Different owner: hand off or release; do not edit the lease projection.

For Claude Code:

```bash
PROMETHEUS_HARNESS=claude-code \
  prometheus kbd --path "$PROJECT_ROOT" claim
```

## Service environment change is ignored on macOS

`launchctl kickstart` restarts the process with the already-loaded definition.
It does not reload edited plist environment variables.

Use the managed installer:

```bash
bash scripts/install-mcp-services.sh --restart
```

For a manually customized plist, fully boot it out and bootstrap it again:

```bash
LABEL="ai.prometheus.sovereign-sync"
PLIST="$HOME/Library/LaunchAgents/$LABEL.plist"
DOMAIN="gui/$(id -u)"

launchctl bootout "$DOMAIN/$LABEL"
launchctl bootstrap "$DOMAIN" "$PLIST"
launchctl enable "$DOMAIN/$LABEL"
launchctl kickstart -k "$DOMAIN/$LABEL"
```

## Headless daemon refuses to start

When `PROMETHEUS_HEADLESS_VOTER=1`, an existing mode-`0600` device key is
required:

```bash
sovereign-sync --mode init \
  --config "$HOME/.config/sovereign-sync/config.toml"

chmod 600 "$HOME/.config/sovereign-sync/device-key.json"
```

Also ensure `[node].operator_id` is non-empty in `config.toml`.

## Runtime reports an integrity conflict

Do not delete the journal or overwrite projections. Capture:

```bash
prometheus kbd --path "$PROJECT_ROOT" audit --json > kbd-audit.json
prometheus kbd --path "$PROJECT_ROOT" status --json > kbd-status.json
```

Then inspect diagnostics through the authenticated REST endpoint. Divergent
offline branches, invalid signatures, revoked devices, and stale fencing
tokens are safety failures that require audit—not a forceful file repair.
