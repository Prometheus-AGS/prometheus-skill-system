# Scheduled maintenance templates

These templates describe optional full-pack maintenance jobs. They do not start
services when read or copied into a skill distribution.

| Template family | Default schedule | Entry point |
|---|---|---|
| `ai.prometheus.mem0-compress.plist`, `mem0-compress.cron` | Sunday at 03:00 | `shared/scripts/mem0-compress.sh` |
| `ai.prometheus.pk-lint.plist`, `pk-lint.cron` | Saturday at 03:00 | `shared/scripts/pk-lint.sh` |

Both launchd and cron templates require explicit installation, a retained source
or payload root, writable log destinations and the tools used by their entry
point. Replace template placeholders with your selected paths before enabling a
job. Do not assume an installation at a particular user's home or that a skill
copy has installed the resident memory/knowledge services.

These maintenance jobs are separate from the managed user-service profile.
The full pack's native memory service is not described as Docker-owned by these
templates. Read [installation](../../../docs/guide/19-installation.md) and
[deployment modes](../../../docs/deployment-modes.md) for the selected service
path. Companion is a separately installed optional extension.

After a completed production phase, validate scheduled behavior locally with
disposable learning, knowledge, home and log roots before enabling a real job.
Retain recovery receipts and inspect any cleanup or `--fix` behavior first.
