---
type: SessionRecord
id: karpathy-session-650ef17413eb6c9a
title: Karpathy session 650ef17413eb
tags:
- karpathy
- session-learning
sources:
- session:751c647e-97c4-4d8b-bb71-c2a55f99d776
timestamp: 2026-08-05T14:38:25.828794+00:00
created_at: 2026-08-05T14:38:25.828794+00:00
updated_at: 2026-08-05T14:38:25.828794+00:00
revision: 0
---

## Delta

Fully answered — and the plist already documents it. The env var is even already named `FORGE_MCP_TOKEN`, so my choice coincidentally matched the existing convention rather than inventing one.

## What the forge-rs token is

**It's a token forge *expects* — it authenticates callers to forge, and is used nowhere else.**

From the plist's own comment:

> forge **REQUIRES** a token (no `--no-auth` flag); without this it mints a **random one each boot** and clients 401. This is a localhost-only (127.0.0.1:8943) dev token, committed like the surrealdb root/root creds; the identical value is set in `.mcp.json`.

So the picture:

| | |
|---|---|
| **Direction** | Inbound — clients → forge. Not a credential forge presents to anything else. |
| **Who mints it** | forge, at startup. Random per boot unless `FORGE_MCP_TOKEN` pins it. |
| **Why it's pinned** | A random-per-boot token means every client 401s after each restart. |
| **Scope** | `127.0.0.1:8943` only, per `--bind` default. |
| **Sensitivity** | Deliberately a committed dev credential, same class as the surrealdb `root/root`. |

That explains the 401 exactly: the server had a token configured, my probe sent none. Not broken, not misconfigured — and restarting it couldn't have helped, since the plist pins the same value every boot.

## What this means for the change

`bearerTokenEnvVar: "FORGE_MCP_TOKEN"` is the right wiring, and better than I realized: **the env var name already exists** in the plist, so Kimi reads the same variable the service is configured with. No new convention.

One correction to what I did: I added `FORGE_MCP_TOKEN` to `~/.prometheus/kbd/secrets.env` (mode 0600). Given the plist already sets it and the value is intentionally a committed dev credential, that's redundant — and mildly misleading, since `secrets.env` implies real secrets. I'd rather remove it and let the existing plist remain the single source.

Want me to drop it from `secrets.env`, or leave it so the value is available to processes that don't inherit the plist environment?

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: 751c647e-97c4-4d8b-bb71-c2a55f99d776
- Captured: 2026-08-05T14:38:22.680526Z
- Project: /Users/gqadonis/Projects/prometheus/prometheus-skill-pack

## Changed Paths

- .prometheus/knowledge/wiki/index.md
- .prometheus/knowledge/wiki/log.md
- scripts/install-kimi-desktop-plugin.sh
- scripts/mcp-port-table.json
- .prometheus/knowledge/wiki/karpathy-session-2c9ae2ec0b01465f.md
- .prometheus/knowledge/wiki/karpathy-session-4dd29470b1e9a99f.md
- .prometheus/knowledge/wiki/kimi-desktop-extensibility-executor-complete-unknown-change.md
