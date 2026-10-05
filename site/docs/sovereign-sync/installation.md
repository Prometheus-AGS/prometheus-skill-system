---
id: installation
title: Installation
sidebar_label: Installation
---

# Companion installation and ownership

Use Companion's own `docs/installation.md` from the owner-selected checkout.
The following are candidate-source procedures, not executed release evidence.
Both binaries must be built, approved and identified separately; these installers
do not build them or install pack services.

From that checkout, macOS service preview uses explicit absolute paths:

```bash
bash scripts/install-companion-service.sh --dry-run \
  --bin /absolute/approved/prometheus-companion-headless \
  --pack-root /absolute/installed/pack \
  --device-key-file /absolute/enrolled/device-key.json
```

Preview renders escaped XML without creating paths or invoking launchctl.
Approved application uses the same arguments without `--dry-run` and requires
an existing executable. The installer owns only `ai.prometheus.companion`,
keeps a private sibling backup of the prior plist and requests startup. The
selected key must be readable, regular, non-symlink and exclude group/other
access; preflight is not proof of cryptographic validity or project enrollment.

`--socket`, `--data-dir`, `--skills-dir`, `--sync-config`, `--log-dir`,
`--xdg-config-home` and `--claude-config-dir` preserve explicit selections.
`PROMETHEUS_DATA_DIR` does not relocate the socket or identity. An absent pack
manifest leaves local control independent; no guessed pack root is adopted.
Linux has no supplied Companion service installer and native Windows is outside
this Unix/launchd contract.

Its separate Claude Code package preview is:

```bash
bash scripts/install-skill-package.sh --dry-run \
  --harness claude-code \
  --bin /absolute/approved/sovereign-sync \
  --claude-bin /absolute/installed/claude
```

After authorized application, the `sovereign-sync` user-scope registration starts
that binary in stdio MCP mode; it does not start a host. The installer uses the
[official Claude MCP CLI](https://code.claude.com/docs/en/mcp), not an obsolete
`~/.claude/mcp.json`. Receipts/backups reside under
`<Claude-config-dir>/.prometheus-companion/`. Unknown, edited, malformed or
unowned entries are preserved or refused. Source symlinks require retaining the
selected checkout. Stop concurrent configuration writers; an installer lock
covers only this installer, not every Claude process. Reconcile pending receipts
and exact backups after failure rather than restoring the whole config over
other entries or automatically taking a stale lock.

Use the same selected socket/config/data/key scope on host and clients. Choose
one socket owner and reload the harness after registration. Other harness
installers are not implemented by this package.

## Ownership and evidence

These routes remain useful after relocation. The current recovered Companion
source (`docs/installation.md`, `docs/control-api.md`) has no public remote or
certified release yet. Source inspection is not installed or peer acceptance;
final source/artifact identities and publication links remain release-owned.

Use [local KBD](/docs/kbd/control-plane) without the optional extension, and the
[service operations guide](/docs/guide/service-operations#optional-companion)
for pack/Companion ownership. The [integration contract](/docs/kbd/integration-contract)
is a one-way seam. [Relocation history](https://github.com/Prometheus-AGS/prometheus-skill-system/blob/main/docs/decisions/sovereign-sync-relocated-to-companion.md)
is a decision record, not a Companion repository URL.
