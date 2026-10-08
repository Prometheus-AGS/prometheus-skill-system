# Recover hook activation and retire stale native caches

Native plugin caches and the signed Prometheus generation store are separate.
Historical caches carry their own installer. A new generation's downgrade guard
cannot intercept an old cache's installer, and a running session can retain old
hook commands until it restarts. Issue #160 requires upgrading the generation,
refreshing native registrations, retiring old cache paths, and restarting sessions.

## Repair sequence

1. Back up the affected plugin configuration, native registrations and generation
   store. Stop affected Claude and Codex sessions and outstanding hook processes.
   Do not remove bytecode or rewrite files inside a signed historical payload.
2. Install the corrected release from the approved source. Normal installation
   recovers pending activation transactions from their verified candidate. If the
   candidate cannot be verified, preserve the journal and restore trusted release
   input before retrying. Never resolve disagreement by trusting an old `current`
   convenience symlink over the authoritative `pointers/current` file.
3. Refresh native registrations through the supported Claude and Codex plugin
   installers. The repository's `scripts/refresh-native-plugin-installs.sh` performs
   that refresh. Ensure every registered Claude scope uses the active release;
   a stale project registration also blocks retirement. Do not hand-edit registry
   JSON. Preserve the same effective `HOME` and `CODEX_HOME` throughout repair.
4. Preview and then apply retirement using the commands below. The command never
   performs a native reinstall itself, and it exits **2 (BLOCKED)** if prerequisites
   are missing. Keep the issue open while any required step remains blocked.
5. Start fresh affected sessions. Verify signed payload integrity, activation
   pointers and links, target projections, and real learning write/recall through
   the installed hook entry points. Run hooks without a caller-supplied
   `PYTHONDONTWRITEBYTECODE` setting, then with a conflicting setting. Confirm no
   generation bytes or modes changed. Do not rely on a Claude settings workaround.

From the approved repository root, with the selected machine home and store:

```bash
node scripts/retire-stale-plugin-caches.mjs \
  --home /absolute/home \
  --plugin-root /absolute/home/.prometheus/plugins/prometheus-skill-pack

node scripts/retire-stale-plugin-caches.mjs \
  --home /absolute/home \
  --plugin-root /absolute/home/.prometheus/plugins/prometheus-skill-pack \
  --apply
```

Preview is the default; `--preview` is also accepted. It reads manifests, hashes
cache trees, reports Claude registration blockers, and makes no writes or native
CLI calls. The active version in a preview is not a verification result. Apply
acquires the generation store lock and invokes the real generation installer's
`--verify` before checking refreshed registrations or relocating anything.
`--targets` selects the installer verification targets (default `all`); use the
same selection as the installation. `--source-root` selects the approved release
checkout containing the installer, and defaults to this script's repository.

A nonempty `CODEX_HOME` overrides the `--home/.codex` fallback. For scratch homes,
unset `CODEX_HOME` or set it explicitly to a scratch destination. With stale Codex
caches present, apply requires a working `codex plugin list --json` and an enabled
registration at the active version. `--codex-bin` or `PROMETHEUS_CODEX_BIN` can
select the supported executable. No Codex or Claude registry is rewritten.

## What retirement changes

Only older, plain `MAJOR.MINOR.PATCH` versions of the umbrella plugin under these
two cache roots are candidates:

- `<home>/.claude/plugins/cache/prometheus-skill-pack/prometheus-skill-pack`
- `<effective CODEX_HOME>/plugins/cache/prometheus-skill-pack/prometheus-skill-pack`

Each candidate's native plugin manifest must agree with its directory identity.
Current and newer versions, unrelated plugins, and every generation-store payload
and rollback generation are preserved. Unknown version names are reported and
preserved for manual inspection. Symlinked old version directories are blocked.
Live `.in_use/<pid>` markers block apply; absence of markers does not prove every
session has stopped. Sessions must still restart after repair.

Caches move intact to `<home>/.prometheus/plugin-cache-quarantine`. Each destination
has a stable identity derived from its original canonical path and a sorted tree
digest covering file contents, symlink targets and modes. Before relocation, a
durable adjacent JSON inventory records original/destination paths, cache version,
digest, active generation, and `pending` status. After the rename it records
`quarantined`. The quarantine must be outside native loader paths and the
generation store. `--quarantine-root` may select another location on the cache's
filesystem; cross-filesystem moves are blocked rather than copied incompletely.

## Interrupted runs and recovery

Re-running apply is idempotent. A pending inventory with an intact original cache
can finish its rename. A pending inventory whose destination already exists is
reconciled after checking its full identity. Both locations existing, missing
payloads, or changed quarantined contents block further work for inspection.
The command does not automatically remove an abandoned store lock: establish
that its recorded process has exited and recover the store through the installer.

Keep inventories beside their quarantined payloads. For deliberate recovery,
stop native sessions, inspect the inventory and tree identity, and restore the
directory to its recorded `from` location only when that path is absent. Preserve
the inventory as the recovery record outside the active quarantine directory.
Restoring a historical cache makes its historical installer executable again;
refresh native registration and use the explicit generation rollback policy if a
downgrade is intended. Normal incident repair should retain the quarantine and
use the corrected native cache.
