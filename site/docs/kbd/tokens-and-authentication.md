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
| Sovereign Sync `operator_id` | Derives the private iroh gossip group | `$HOME/.config/sovereign-sync/config.toml` |
| Ed25519 device key | Signs KBD events, remote commands, claims, and sync envelopes | OS credential store or protected device-key file |

The obsolete KBD bearer-token protocol has been removed. Sovereign Sync binds
its local API to `127.0.0.1`; read routes and non-authoritative sync controls
rely on that loopback boundary. Every KBD mutation POST must additionally carry
a schema-v2 `SignedCommandEnvelope` from an active enrolled device.

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

## Network boundary

The HTTP server must remain loopback-only while non-KBD routes have no request
authentication. Binding it to a non-loopback address requires a separate,
reviewed transport-authentication design. Thin clients should connect through
an authenticated host integration that forwards device-signed KBD commands;
the device signature is not a substitute for securing an exposed HTTP server.

The `operator_id` controls gossip topic membership but is not a credential for
the REST API and is not a device signing key.
