---
name: sync-status
description: Show the current P2P sync status of the sovereign-sync node — connection state, peer count, and domain health.
version: '1.0.0'
license: MIT
metadata:
  author: prometheus-skill-pack
  category: learn
  tags: [sovereign-sync, p2p, sync, status, learn, iroh]
---

# /sync-status

Show the live status of the local sovereign-sync node: connection state, number
of connected peers, and a summary of which sync domains are up to date.

## When to use

Trigger on:
- "sync status", "check sync", "is sync running", "how many peers"
- Before a `/sync-push` to confirm the node is `Connected`
- After install to verify the node started correctly

## Prerequisites

A `sovereign-sync` daemon or server must be running. Check the launchd service:

```bash
launchctl list | grep sovereign-sync
```

Start one manually with:

```bash
sovereign-sync --mode daemon
```

### Transport: Unix socket by default, TCP only on request

The daemon listens on a **Unix domain socket**, NOT loopback TCP. Curling
`http://127.0.0.1:7892` returns nothing (exit 7 / HTTP 000) on a perfectly
healthy node — that address is only bound when `--tcp` is passed explicitly.

Default socket path (`data_local_dir()/prometheus/run/sovereign-sync.sock`):

| Platform | Path |
|---|---|
| macOS | `~/Library/Application Support/prometheus/run/sovereign-sync.sock` |
| Linux | `~/.local/share/prometheus/run/sovereign-sync.sock` |

`--socket <PATH>` overrides it. `--tcp` switches to loopback TCP on `--port`
(default 7892) and then requires a mode-0600 bearer token via `--token-file`.

## Instructions

1. Confirm the node is healthy. Prefer the built-in check — it resolves the
   socket path itself, so it works regardless of transport or platform:

   ```bash
   sovereign-sync --mode status
   ```

   For scripting, request JSON:

   ```bash
   sovereign-sync --mode status --format json
   ```

   Exit 0 means healthy. The output reports the resolved `endpoint` and
   measured latency (`p50Ms`, `p95Ms`, `p99Ms`, `maximumMs`, plus `failures`
   and `timeouts`). Use `--p99-budget-ms` / `--max-budget-ms` to fail the
   command when latency exceeds a threshold.

2. Fetch detailed sync status over the socket:

   ```bash
   SOCK="${HOME}/Library/Application Support/prometheus/run/sovereign-sync.sock"  # macOS
   # SOCK="${HOME}/.local/share/prometheus/run/sovereign-sync.sock"               # Linux
   curl -s --unix-socket "$SOCK" http://localhost/api/v1/sync/status | jq .
   ```

   Add `--unix-socket "$SOCK"` to every REST call in the sync skills. If the
   node was started with `--tcp`, use `http://127.0.0.1:7892` instead and pass
   the bearer token.

3. Interpret the response. It carries two state fields plus the node ID:

   ```json
   {
     "node_state": "Ready",
     "transport": { "state": "ready", "nodeId": "b12cd66c...", "peers": [],
                    "attempt": 1, "lastError": null, "nextRetryMs": null }
   }
   ```

   Both state fields use the TRANSPORT vocabulary, capitalised differently
   (`node_state` is PascalCase, `transport.state` is lowercase):

   | State | Meaning |
   |-------|---------|
   | `Initializing` / `initializing` | Node started but has not joined the network |
   | `Bootstrapping` / `bootstrapping` | Connecting to known bootstrap peers |
   | `Ready` / `ready` | Transport is up; gossip available |

   `Ready` is a rollup. Internally the P2P layer distinguishes `Connected`,
   `Syncing`, and `Idle`, but all three surface as `Ready` — the REST API does
   not expose which one is active, so do NOT report "Connected" or "Syncing"
   as if the endpoint returned it.

   A single node with no peers enrolled reports `Ready` with an empty `peers`
   array. That is healthy, not a fault — there is simply nothing to connect to.

   The node's own ID is `transport.nodeId`. It is not on `/health`.

4. Read each domain's `adapter` and `privacy`. An adapter of `never-synced`
   paired with privacy `local_only` is the privacy guarantee working as
   designed, not a sync failure.

5. Report to the user: node state, transport state, peer count
   (`peers` array length), and each domain with its adapter and privacy class.

## Output format

```
sovereign-sync status
─────────────────────
State     : Ready
Peers     : 0
Node ID   : b12cd66c0e34c6ea…
Version   : 1.7.0
Transport : unix socket
Domains   : kbd-control (wired/trusted), learner-model (wired/trusted),
            skill-index (wired/public), surreal-memory (never-synced/local_only)
```

## Troubleshooting

**`curl http://127.0.0.1:7892/...` returns nothing (exit 7 / HTTP 000)** — this
is the expected result on a healthy node. The daemon listens on a Unix socket
unless started with `--tcp`. Use `sovereign-sync --mode status`, or add
`--unix-socket "$SOCK"` to the curl. Do NOT conclude the service is down, and
do not restart it on this basis alone.

**Node not reachable** — confirm the process and socket exist before restarting:
```bash
launchctl list | grep sovereign-sync
ls -l "${HOME}/Library/Application Support/prometheus/run/sovereign-sync.sock"
sovereign-sync --mode daemon &
```

Note that `lsof -p <pid>` lists established peer connections alongside the
listener; an entry there is not necessarily the socket clients should dial.
Trust the `endpoint` reported by `--mode status`.

**State stays `Bootstrapping`** — no peers are reachable. Add a peer address:
```bash
/sync-peers add <node-id>@<address>
```

**State is `Disconnected`/`Initializing`** — the node has not joined the
network. Only check firewall rules on port 7892 if the daemon was started with
`--tcp`; the default Unix socket does not use a port at all.

## Privacy guarantee

The status endpoint returns only network metadata (state, peer IDs, domain
names). No KB content is exposed. `PrivacyClass::LocalOnly` domains are visible
by name in the domain list but their payloads never leave the device.
