---
title: Plugin sources and hook failures
description: Where the plugin marketplace source should live, how hook activation fails, and how to recover.
---

# Plugin sources and hook failures

Plugin hooks run from a plugin payload directory: the marketplace source (a `directory` source is read in place) or the native cache. If that payload is incomplete, or the directory is gone, **every hook of the plugin fails on every turn**, in every session that loaded it.

## What to point the marketplace at

Use one **durable release-line checkout** (for example a dedicated `deploy/main` worktree, or the main checkout on `main`), and register that same path for Claude and Codex.

Do not register a topic-branch checkout or worktree. It lives only as long as its branch: when the branch merges and the worktree is removed, the plugin source disappears underneath running sessions. A release-line linked worktree is fine; removing it still removes the source, so doctor notes it.

## What each failure looks like

| Message | Cause | Fix |
|---|---|---|
| `Plugin directory does not exist: <path>` | The registered source was removed. Raised by the harness before any hook runs. | Point the marketplace at a durable checkout, then **reload plugins or restart the session**: a running session keeps the path it loaded. |
| `HOOK_RUNTIME_ERROR` with code `PAYLOAD_INCOMPLETE` | The payload is missing a module its installer imports, and the activated generation is older than the payload. | Update or reinstall the plugin (`/plugin`), then restart the session. |
| `HOOK_RUNTIME_ERROR` with code `BOOTSTRAP_FAILED` | Activation failed for another reason; the message carries the first error line. | Re-run with `PROMETHEUS_HOOK_DEBUG=1` for the full output. |
| `HOOK_RUNTIME_ERROR` with code `NOT_ACTIVATED` | No activated bundle, and no plugin root or bootstrap payload to build one from. | Reinstall the plugin. |

A session on a superseded cache version keeps working while its bundle is still registered under `~/.prometheus/plugins/prometheus-skill-pack/bundles`. It only fails if its bundle is not registered.

## Diagnose

```bash
node scripts/check-plugin-source.js          # human-readable
prometheus doctor --json --check plugins
```

- `plugins.source-topology` **fails** when a registered source does not exist. It **warns** on a topic-branch or dirty source and when clients disagree. A linked release-line worktree is informational.
- `plugins.native-cache-skew` warns when the installed plugin is behind the active generation, and lists live sessions still running a superseded version so they can be restarted. It never fails the run.

## Installer guard

`update-skill-pack.sh` and `refresh-native-plugin-installs.sh` refuse to refresh from a checkout that **is a registered plugin source and is on a topic branch** (exit 3). A checkout that is not registered is never blocked. To accept the risk explicitly, pass `--allow-topic-branch`.

## What the pack verifies before it ships

`npm run check:distribution` copies the built payload to a temporary directory, uses an empty `HOME` so activation must bootstrap from scratch, and executes every shipped hook. A payload missing a module its installer imports fails this check. Both checks run locally; there is no hosted CI.
