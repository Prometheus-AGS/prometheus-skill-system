---
id: pair-two-machines
title: Pair Two Machines
sidebar_label: Pair Two Machines
---

# Pair Two Machines

Pairing has two independent requirements:

1. both machines derive the same gossip topic from the same `operator_id`; and
2. at least one machine bootstraps with the other machine’s distinct iroh
   endpoint ID.

Matching the operator ID does not perform discovery by itself. Copying every ID
is also wrong: signing keys, endpoint identities, KBD node IDs, and API tokens
represent different security boundaries.

:::warning What pairing proves today

The procedure below can establish an iroh-gossip neighbor relationship for the
current process lifetime. Version `0.1.0` does not connect that relationship to
project/global data replication. The REST peer list is a scaffold and the
endpoint ID changes when the daemon restarts.

:::

## Identifier map

| Identifier | Same on both machines? | Persistent today? | Where it lives | Purpose |
|---|---:|---:|---|---|
| Sovereign `operator_id` | **Yes** | Yes | `$HOME/.config/sovereign-sync/config.toml` | Derives the shared gossip topic |
| Repository `projectId` | **Yes for clones of one project** | Yes | `<project>/.prometheus/project.json` | Names the canonical KBD project/runtime |
| iroh endpoint ID | **No** | **No** | Startup log only | Identifies and locates one running P2P endpoint |
| KBD `node_id` | Local compatibility value | Yes in config | `[kbd].node_id` | Identifies the current single journal writer; defaults to `1` |
| KBD device-signing key | **No** | Yes | platform credential store or `device-key.json` | Proves which physical device signed a command |
| KBD control token | **No requirement to match** | Yes | project runtime `control-token` | Authenticates the local loopback REST API |
| learner ID | Yes for one human learner | Yes in model data | learner-model document key | Joins the same logical learner record |

### Values that must match

`operator_id` must match byte-for-byte. The installer generates a recommended
64-character hexadecimal value, although the parser only requires a non-empty
string.

`projectId` must match when two repository checkouts represent the same logical
project. Establish `.prometheus/project.json` once, then distribute that file
through the repository or another reviewed setup channel. Do not let each
machine independently create a different project manifest and later overwrite
one runtime with the other ID.

### Values that must remain different

Never copy `device-key.json` between machines. The signing key is the identity
of one device.

Do not copy `control-token` as a pairing step. It protects a local API and is
not used by iroh.

The compatibility policy accepts one local writer with ID `1`. Multi-voter
configuration is rejected; replica identity is handled by the project registry.

## Before you begin

On both machines:

```bash
bash scripts/check-prerequisites.sh --install --build-tools
bash scripts/install-mcp-services.sh
sovereign-sync --mode status --format json
```

Confirm that both checkouts contain the same project identity:

```bash
jq '{projectId, repositoryFingerprint}' .prometheus/project.json
```

If the manifest does not exist yet, start the KBD runtime on one canonical
checkout first, commit or securely transfer the generated manifest, and only
then initialize the second checkout.

## Step 1: create one operator namespace

On Machine A, generate the shared value once:

```bash
openssl rand -hex 32
```

Place the exact output in the config on both machines:

```toml
[node]
operator_id = "replace-with-the-same-64-hex-character-value"
```

Transfer only this value through a trusted channel. It is not a private key,
but possession lets another node derive the same topic. Do not put a personal
operator namespace in a public issue, build log, or example repository.

Keep each machine’s own `skills_dir`, server port, device key, and token.

## Step 2: start Machine A as the temporary anchor

Machine A can subscribe with no bootstrap peers:

```toml
[peers]
bootstrap = []
```

For initial diagnostics, a foreground process is easier to observe than the
managed service:

```bash
RUST_LOG=sovereign_sync=debug,iroh_gossip=debug \
  sovereign-sync --mode daemon
```

Keep it running. Find this line in its startup output:

```text
P2P endpoint started — node_id=<machine-a-endpoint-id>
```

That value is Machine A’s current iroh endpoint ID. It is public identity
material, not the endpoint’s private key.

For a managed macOS service, the same startup line is normally in:

```bash
rg 'P2P endpoint started' \
  "$HOME/.prometheus/logs/sovereign-sync.stderr.log" | tail -n 1
```

For the managed Linux service:

```bash
journalctl --user -u ai.prometheus.sovereign-sync \
  | rg 'P2P endpoint started' | tail -n 1
```

## Step 3: bootstrap Machine B to Machine A

On Machine B, keep the same `operator_id` and add Machine A’s endpoint ID:

```toml
[node]
operator_id = "the-same-value-used-on-machine-a"

[peers]
bootstrap = [
  "machine-a-endpoint-id"
]
```

Then start or fully restart Machine B:

```bash
# Foreground diagnostic
RUST_LOG=sovereign_sync=debug,iroh_gossip=debug \
  sovereign-sync --mode daemon
```

Or restart the managed definition:

```bash
bash scripts/install-mcp-services.sh --restart
```

Machine B uses the configured endpoint ID with the N0 address resolver. It does
not need Machine A’s IP address in `config.toml`.

Machine A does not need Machine B in its bootstrap list for this first
connection: B joins A’s already-running topic subscription. For more resilient
meshes, every node should eventually have at least one known entry point, but
the current ephemeral endpoint limitation prevents durable configuration.

## Step 4: verify the correct layer

Check each local service:

```bash
curl --fail-with-body http://127.0.0.1:7892/health | jq .
```

Then inspect debug logs for gossip neighbor/connectivity events. Do not use the
following as proof of a live peer in `0.1.0`:

- `GET /api/v1/sync/peers` — currently always returns an empty list;
- `/sync-peers` — reports the same bounded summary;
- `POST /api/v1/sync/push` — acknowledges a queue request but sends no domain
  data;
- a local `/health` response — proves only that the local HTTP process is up.

There is currently no operator-facing end-to-end assertion such as “peer B
applied learner-model version 17.” That observability is part of the remaining
implementation work.

## Step 5: manage restarts

The daemon does not load its iroh endpoint from `device-key.json`. It lets iroh
generate an endpoint key at each process start.

Therefore:

- restarting Machine B gives B a new ID, but B can still reconnect while the
  configured Machine A process remains alive;
- restarting Machine A invalidates the endpoint ID stored by B;
- restarting both machines loses the only configured rendezvous unless one
  config is updated from the other machine’s new startup log.

The temporary recovery procedure is:

1. start the designated anchor with an empty bootstrap list;
2. capture its new endpoint ID;
3. replace the stale ID on the other machine;
4. restart only the joining machine;
5. re-check debug logs.

Do not build production operations around this procedure. Durable pairing
requires a persistent iroh endpoint key plus a supported peer-management and
status surface.

## Rotating and separating operator groups

To remove both machines from an old topic:

1. generate a new `operator_id`;
2. update every intended member through a trusted channel;
3. remove stale bootstrap IDs;
4. restart each daemon;
5. repeat the endpoint exchange.

Machines with different `operator_id` values derive different topics even if
one has the other’s endpoint ID. Machines with the same `operator_id` but no
reachable bootstrap path do not find each other automatically.

## Next

Read [Network configuration](./p2p-network) before pairing across a corporate
firewall, guest Wi-Fi, VPN, cellular network, or air-gapped LAN. Read
[Exactly what syncs](./data-scope) before interpreting a successful neighbor
connection as application-level replication.
