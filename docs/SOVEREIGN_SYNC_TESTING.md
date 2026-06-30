# Sovereign Sync — Two-Node Validation Guide

This guide walks through validating `sovereign-sync` P2P CRDT sync across two distinct
network namespaces. It covers two setups: Docker Compose (recommended for local testing)
and two physical or virtual hosts.

---

## Prerequisites

The `sovereign-sync` binary must be built first:

```bash
cd substrate/sovereign-sync
cargo build --release
# Binary lands at: target/release/sovereign-sync
```

Or, if you ran the full install:

```bash
bash scripts/install-skills-flat.sh
# Binary lands at: ~/.local/bin/sovereign-sync
```

Verify:

```bash
sovereign-sync --version
```

---

## Option A: Docker Compose (recommended)

This is the fastest path to a two-node test. Both nodes run in containers with separate
network namespaces and communicate over a shared Docker bridge.

### Step 1 — Create the Compose file

```yaml
# docker-compose.sovereign-sync-test.yml
services:
  node-a:
    image: debian:bookworm-slim
    volumes:
      - ~/.local/bin/sovereign-sync:/usr/local/bin/sovereign-sync:ro
      - ./data-a:/data
    environment:
      - SOVEREIGN_SYNC_DATA_DIR=/data
    command: sovereign-sync --mode daemon --port 7892
    ports:
      - "7892:7892"
    networks:
      - sync-net

  node-b:
    image: debian:bookworm-slim
    volumes:
      - ~/.local/bin/sovereign-sync:/usr/local/bin/sovereign-sync:ro
      - ./data-b:/data
    environment:
      - SOVEREIGN_SYNC_DATA_DIR=/data
    command: sovereign-sync --mode daemon --port 7892
    ports:
      - "7893:7892"
    networks:
      - sync-net

networks:
  sync-net:
    driver: bridge
```

```bash
mkdir -p data-a data-b
docker compose -f docker-compose.sovereign-sync-test.yml up -d
```

### Step 2 — Verify both nodes are healthy

```bash
# Node A (port 7892)
curl -s http://127.0.0.1:7892/health | jq .

# Node B (port 7893)
curl -s http://127.0.0.1:7893/health | jq .
```

Both should return `{"status":"ok"}`.

### Step 3 — Push a domain on Node A

```bash
# Create data on Node A
curl -s -X POST http://127.0.0.1:7892/api/v1/sync/push \
  -H "Content-Type: application/json" \
  -d '{"domain": "learner-model"}' | jq .
```

### Step 4 — Get Node A's sync status

```bash
curl -s http://127.0.0.1:7892/api/v1/sync/status | jq .
```

Note the node ID from the response — you need it for step 5.

### Step 5 — Connect Node B to Node A

Node B discovers Node A via the iroh P2P gossip layer. The two containers share the
`sync-net` bridge, so they can reach each other by container name:

```bash
# From Node B's perspective, push the same domain — this triggers peer discovery
docker compose -f docker-compose.sovereign-sync-test.yml exec node-b \
  curl -s -X POST http://127.0.0.1:7892/api/v1/sync/push \
  -H "Content-Type: application/json" \
  -d '{"domain": "learner-model"}' | jq .
```

### Step 6 — Verify peer sync

```bash
# Check peers on Node A
curl -s http://127.0.0.1:7892/api/v1/sync/peers | jq .

# Check peers on Node B
curl -s http://127.0.0.1:7893/api/v1/sync/peers | jq .
```

Both should list the other node as a peer.

### Step 7 — Verify CRDT merge

After a few seconds for QUIC transport to propagate:

```bash
# Check the learner-model domain status on both nodes
curl -s http://127.0.0.1:7892/api/v1/sync/status | jq '.domains["learner-model"]'
curl -s http://127.0.0.1:7893/api/v1/sync/status | jq '.domains["learner-model"]'
```

Both nodes should report the same CRDT state for the `learner-model` domain.

### Teardown

```bash
docker compose -f docker-compose.sovereign-sync-test.yml down
rm -rf data-a data-b
```

---

## Option B: Two physical or virtual hosts

Use this when you have two machines on the same network or over the internet.

### Setup on Host A

```bash
# Copy the binary
scp ~/.local/bin/sovereign-sync user@host-a:~/.local/bin/

# Start the daemon
ssh user@host-a "sovereign-sync --mode daemon --port 7892"
```

### Setup on Host B

```bash
scp ~/.local/bin/sovereign-sync user@host-b:~/.local/bin/
ssh user@host-b "sovereign-sync --mode daemon --port 7892"
```

### Verify both nodes are healthy

```bash
curl -s http://HOST_A_IP:7892/health | jq .
curl -s http://HOST_B_IP:7892/health | jq .
```

### Push and sync

Follow the same Steps 3–7 from Option A, substituting `HOST_A_IP:7892` and
`HOST_B_IP:7892` for `127.0.0.1:7892` and `127.0.0.1:7893`.

---

## Troubleshooting

### "connection refused" on the health endpoint

The daemon is not running or not bound to `0.0.0.0`. By default, `sovereign-sync --mode daemon`
binds to `127.0.0.1:7892`. For remote access, pass `--bind 0.0.0.0`:

```bash
sovereign-sync --mode daemon --port 7892 --bind 0.0.0.0
```

### QUIC/UDP traffic is blocked

iroh uses QUIC (UDP) for P2P transport. If nodes are behind different firewalls, the
UDP port must be open. iroh negotiates its own UDP port (separate from the HTTP REST
port 7892). Check what port iroh is listening on:

```bash
curl -s http://127.0.0.1:7892/api/v1/sync/status | jq '.iroh_addr'
```

Open the UDP port shown in `iroh_addr` on both machines' firewalls.

### Peers not discovering each other

iroh uses the iroh-gossip protocol. Peers discover each other via topic subscription.
If both nodes push the same domain name, they join the same gossip topic and will find
each other. Wait up to 30 seconds after both nodes are running.

### Docker: "binary format error"

The `sovereign-sync` binary must match the container architecture. If your host is
arm64 and your container is amd64 (or vice versa), build for the container's architecture:

```bash
# For an amd64 container on an arm64 host:
cargo build --release --target x86_64-unknown-linux-gnu
```

---

## Reporting results

If you complete a successful two-node sync, please comment on the GitHub discussion
"First external user onboarding" with:

- Which option you used (A or B)
- The output of both `curl -s http://.../api/v1/sync/peers` calls
- Whether the CRDT states matched

This is the evidence needed to close Goal G3 in `phase-external-validation`.
