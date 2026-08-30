---
id: tokens-and-authentication
title: Identity & Authentication
sidebar_label: Identity & Authentication
---

# Identity and Authentication

Prometheus uses separate identifiers and keys for routing, synchronization,
and authorization. They are not interchangeable:

| Value | Purpose | Typical location |
|---|---|---|
| Project ID | Immutable project identity | `.prometheus/project.json` |
| Replica ID | Identifies one checkout/device replica | platform KBD registry |
| Machine ID | Identifies the local registry owner | platform KBD registry |
| Sovereign group secret | Derives the private iroh gossip group | Mode-`0600` P2P identity; transferred only in pairing tickets |
| Ed25519 device key | Signs KBD events, remote commands, claims, and sync envelopes | OS credential store or protected device-key file |

The obsolete KBD bearer-token protocol has been removed. Sovereign Sync uses a
mode-`0600` Unix-domain socket by default and verifies that the client belongs
to the same operating-system user. Explicit loopback TCP mode requires a token
from a mode-`0600` file; the service exposes no unauthenticated TCP constructor.
Every KBD mutation POST must additionally carry a schema-v2
`SignedCommandEnvelope` from an active enrolled device.

## Device signing keys

Initialize the device key through Sovereign Sync:

```bash
sovereign-sync --mode init \
  --config "$HOME/.config/sovereign-sync/config.toml"
```

Interactive canonical runtimes use the supported OS credential store. A
headless installation may use a host-protected file:

```bash
export PROMETHEUS_DEVICE_KEY_FILE="$HOME/.config/sovereign-sync/device-key.json"
chmod 600 "$PROMETHEUS_DEVICE_KEY_FILE"
```

The file contains private signing material. Never copy it between devices,
commit it, print it, or expose it to frontend code. Each replica must have its
own key and enrollment record so revocation remains device-specific.

## Signed command contract

KBD command requests contain:

- an inner schema-v2 command with `projectId`, `replicaId`-derived routing,
  `commandId`, and the current causal `frontier`;
- `signerKeyId` identifying an active enrolled device; and
- an Ed25519 signature over canonical command bytes plus that key ID.

Unsigned, schema-v1, tampered, unknown-device, and revoked-device command
requests fail closed. Use `prometheus kbd` or `sovereign-client` to construct
the signature; do not hand-roll canonicalization in shell scripts.

Only the operator-signed genesis event may bootstrap its own signing key. The
folded state records that key as operator authority; a signed key-rotation event
moves the authority binding to its replacement. A new device is trusted only
after an active operator key signs a `DeviceEnrolled` event. Every causal event,
including a conflict loser or resolution record, is authorized before conflict
selection; presenting a new public key alongside a self-signed event never
enrolls it. The Loro authority accepts only signed schema-v2 events. Unsigned
schema-v1 history is handled solely by the explicit, backed-up legacy journal
migration path and cannot enter through peer imports.

## Register projects served by Sovereign Sync

The daemon serves every project in its platform registry. A checkout is
registerable only when it already declares `.prometheus/project.json`:

```bash
prometheus kbd register /path/to/project
prometheus kbd projects --json
```

Registration never creates or infers project identity from a path, Git origin,
or commit. REST routes and multi-project MCP calls use the declared project
UUID. No project-path or bearer-token environment variable selects the active
project.

## Maintain missing registry entries

Removed worktrees and temporary checkouts can leave registrations whose paths
no longer exist. Inventory them first; dry run is the default and leaves the
registry byte-for-byte unchanged:

```bash
PROJECT_ROOT="/path/to/project"

prometheus kbd --path "$PROJECT_ROOT" projects \
  --prune-missing --json | jq .
```

Review every `candidates[]` path and project ID. A network mount or removable
volume that is temporarily unavailable also looks absent, so restore or mount
it before applying. The command propagates filesystem metadata errors rather
than treating an unreadable path as proof that it is missing.

Apply only after reviewing that inventory:

```bash
prometheus kbd --path "$PROJECT_ROOT" projects \
  --prune-missing --apply --json | tee registry-prune-report.json
```

`--apply` without `--prune-missing` is rejected. Apply takes the exclusive
registry lock and evaluates paths again, so a checkout that reappeared after
dry run is retained. A successful mutation removes only the absent registration
keys. It never deletes project runtime directories, journals, checkpoints, or
an existing replica that shares the same project UUID.

The JSON report contains `backupPath`, `backupSha256`, `checksumPath`, and
`receiptPath`. All point into one timestamped
`registry-maintenance-backups/<operation-id>/` directory. That directory holds:

| File | Recovery purpose |
|---|---|
| `registry.json` | Exact pre-change registry bytes |
| `registry.sha256` | SHA-256 integrity record for the backup |
| `receipt.json` | Removed entries, retained count, source hash, and planned post-change registry hash |
| `ROLLBACK.md` | Paths and ordered manual recovery instructions for this operation |

Retain the directory. Repeating the apply after all missing entries are removed
reports zero removals and creates no additional backup.

### Roll back an applied prune

Rollback restores registry membership; it does not reconstruct a deleted
checkout and must never remove the retained runtime tree.

1. Stop `sovereign-sync` so it cannot reopen or rewrite the registry.
2. Read `receipt.json` and compare the live registry SHA-256 with
   `plannedRegistrySha256`. A match proves that operation's atomic replacement
   completed. If the live hash matches `backupSha256`, the pre-change bytes are
   already present. If it matches neither value, stop and preserve all files for
   audit instead of guessing.
3. Verify the backed-up `registry.json` against `registry.sha256` using the
   platform SHA-256 tool.
4. Follow the operation's `ROLLBACK.md`: acquire the exclusive `registry.lock`,
   restore the exact backup bytes through an atomic same-directory replacement,
   fsync the registry directory, and then release the lock.
5. Restart `sovereign-sync`, then verify both machine registration and project
   authority:

   ```bash
   prometheus kbd --path "$PROJECT_ROOT" projects --json | jq .
   prometheus kbd --path "$PROJECT_ROOT" status --json | jq .
   prometheus doctor --check control.kbd-runtime
   ```

Keep the backup and receipt as audit evidence after recovery.

## Network boundary

The HTTP server must remain loopback-only while non-KBD routes have no request
authentication. Binding it to a non-loopback address requires a separate,
reviewed transport-authentication design. Thin clients should connect through
an authenticated host integration that forwards device-signed KBD commands;
the device signature is not a substitute for securing an exposed HTTP server.

The group secret controls gossip topic membership but is not a credential for
the REST API and is not a device signing key.
