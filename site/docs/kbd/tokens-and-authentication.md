---
id: tokens-and-authentication
title: Tokens & Authentication
sidebar_label: Tokens & Authentication
---

# Tokens and Authentication

Prometheus uses three different security values. They are not interchangeable:

| Value | Purpose | Typical location |
|---|---|---|
| KBD control token | Authenticates loopback REST requests | canonical project runtime `control-token` |
| Sovereign Sync `operator_id` | Derives the P2P gossip group | `$HOME/.config/sovereign-sync/config.toml` |
| Ed25519 device key | Signs canonical KBD events | OS credential store or `PROMETHEUS_DEVICE_KEY_FILE` |

Comparing the bearer token with `operator_id` will always look like a
mismatch—they serve different protocols.

## Find the token for a project

```bash
PROJECT_ROOT="/path/to/project"
PROJECT_ID="$(jq -r '.projectId' "$PROJECT_ROOT/.prometheus/project.json")"

case "$(uname -s)" in
  Darwin)
    DATA_ROOT="$HOME/Library/Application Support"
    ;;
  *)
    DATA_ROOT="${XDG_DATA_HOME:-$HOME/.local/share}"
    ;;
esac

TOKEN_FILE="$DATA_ROOT/prometheus/kbd/projects/$PROJECT_ID/control-token"
printf 'Token file: %s\n' "$TOKEN_FILE"
```

Do not print the token itself into a terminal transcript, CI log, issue, or
chat.

## Automatic token creation

When `PROMETHEUS_CONTROL_TOKEN_FILE` is unset, the runtime creates the
canonical token atomically on first use. The generated token is 32 random
bytes encoded as unpadded URL-safe Base64.

The file must be:

- a regular file, never a symlink;
- readable only by its owner (`0600`);
- at least 32 characters after trimming;
- composed only of ASCII letters, digits, `-`, and `_`.

Validate metadata and format without revealing the token:

```bash
stat "$TOKEN_FILE"

TOKEN="$(tr -d '\r\n' < "$TOKEN_FILE")"
test "${#TOKEN}" -ge 32
printf '%s' "$TOKEN" | LC_ALL=C grep -Eq '^[A-Za-z0-9_-]+$'
```

If `PROMETHEUS_CONTROL_TOKEN_FILE` is explicitly set, the runtime requires that
file to exist; it will not silently generate a replacement at the configured
path.

## Use an explicit token file

An explicit path is useful for a managed daemon:

```bash
export PROMETHEUS_CONTROL_TOKEN_FILE="$HOME/.config/sovereign-sync/kbd-control-token"
```

Create it safely:

```bash
TOKEN_FILE="$HOME/.config/sovereign-sync/kbd-control-token"
mkdir -p "$(dirname "$TOKEN_FILE")"
umask 077
openssl rand -hex 32 > "$TOKEN_FILE"
chmod 600 "$TOKEN_FILE"
```

Configure the same path in:

1. the Sovereign Sync daemon environment;
2. the harness process or generated hook environment;
3. any CLI or REST client that overrides the canonical default.

Using the canonical project path avoids most environment propagation problems.

## Authenticate a REST call

`/health` is public on loopback. Every other Sovereign Sync route requires:

```http
Authorization: Bearer <control-token>
```

Example:

```bash
TOKEN="$(tr -d '\r\n' < "$TOKEN_FILE")"

curl --fail-with-body \
  -H "Authorization: Bearer $TOKEN" \
  http://127.0.0.1:7892/api/v1/sync/status | jq .
```

A valid token with an uninitialized KBD runtime can still receive HTTP `404`
from a KBD route. That means authentication succeeded but the requested
project state is not initialized. An invalid token receives HTTP `401`.

## Register projects served by Sovereign Sync

The daemon serves every project in its platform registry. Register a checkout
that already declares `.prometheus/project.json`:

```bash
export PROMETHEUS_CONTROL_TOKEN_FILE="$TOKEN_FILE"
prometheus kbd register /path/to/project
sovereign-sync --mode daemon
```

The token authenticates the local control surface; it does not select a
project. REST routes and multi-project MCP calls use the declared project UUID.
The managed launchd/systemd definitions deliberately have no project-path
environment variable. See [Sovereign Sync installation](/docs/sovereign-sync/installation).

## Device signing keys

Interactive canonical runtimes use the supported OS credential store.
Headless voters must set:

```bash
export PROMETHEUS_HEADLESS_VOTER=1
export PROMETHEUS_DEVICE_KEY_FILE="$HOME/.config/sovereign-sync/device-key.json"
```

Initialize the file through Sovereign Sync:

```bash
sovereign-sync --mode init \
  --config "$HOME/.config/sovereign-sync/config.toml"
chmod 600 "$HOME/.config/sovereign-sync/device-key.json"
```

The device key is JSON containing private signing material. Never use it as an
HTTP token or expose it to client-side code.

## Rotate a control token

Sovereign Sync caches the token at startup. Rotate it as a coordinated
operation:

1. pause writers;
2. stop the daemon;
3. replace the regular file atomically;
4. keep mode `0600`;
5. restart the daemon;
6. restart harnesses that received an explicit token path;
7. verify valid-token `200` and invalid-token `401` behavior.

Example replacement:

```bash
umask 077
NEW_TOKEN_FILE="${TOKEN_FILE}.new"
openssl rand -hex 32 > "$NEW_TOKEN_FILE"
chmod 600 "$NEW_TOKEN_FILE"
mv "$NEW_TOKEN_FILE" "$TOKEN_FILE"
```

Never rotate by changing only a shell variable while a daemon still holds the
old token in memory.
