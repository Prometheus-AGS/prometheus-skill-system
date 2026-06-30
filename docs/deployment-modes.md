# Deployment Modes

The prometheus-skill-pack operates across four progressive capability tiers. Each tier is a strict superset of the tier below it.

## Capability Matrix

| Capability | Mode 0 CLI | Mode 1 MCP | Mode 2 Full | Mode 3 P2P |
|---|:---:|:---:|:---:|:---:|
| `forge enrich` / `validate` / `reflect` | YES | YES | YES | YES |
| Skill discovery and slash commands | YES | YES | YES | YES |
| surreal-memory knowledge graph | NO | YES | YES | YES |
| Sycophancy correction gate | NO | YES | YES | YES |
| surface-bridge Tier 2 UI (iframe) | NO | NO | YES | YES |
| FSRS-6 spaced retrieval (learner-model) | NO | NO | YES | YES |
| sovereign-sync P2P CRDT replication | NO | NO | NO | YES |
| iroh QUIC transport | NO | NO | NO | YES |
| AG-UI SSE streaming endpoint | NO | NO | NO | YES |

## Mode Descriptions

### Mode 0 — CLI only

**Requires:** Node.js 20+, Rust stable (for forge-rs)

Install skills and run forge operations locally. No persistent memory, no UI beyond text.

```bash
bash scripts/install-skills-flat.sh
forge enrich src/main.rs --language rust
```

Suitable for offline environments and CI pipelines.

---

### Mode 1 — MCP

**Requires:** Mode 0 + surreal-memory MCP + sycophancy-correction MCP

Skills gain access to the surreal-memory knowledge graph and the sycophancy gate. Learning sessions, PMPO reflection, and memory-backed workflows become available.

```bash
# Start surreal-memory server
npm run install:daemons
npm run health

# Verify
bash shared/scripts/detect-toolchain.sh --json | jq .mcp_servers
```

The `.mcp.json` file configures both servers; `install-skills-flat.sh` writes platform-specific MCP config files.

---

### Mode 2 — Full (surface-bridge + learner-model)

**Requires:** Mode 1 + surface-bridge daemon + learner-model binary

The surface-bridge Axum server (`127.0.0.1:7890`) enables Tier 2 UI rendering in harnesses that support MCP App iframes. The learner-model binary provides CRDT-backed mastery tracking and FSRS-6 scheduling.

```bash
# Installed automatically by install-skills-flat.sh
# Manual start:
surface-bridge &
curl -s http://127.0.0.1:7890/health | jq .
```

The learn domain skills (`/feynman-loop`, `/learn-goal`, `/learn-retain`, etc.) operate at full fidelity in Mode 2.

---

### Mode 3 — P2P

**Requires:** Mode 2 + sovereign-sync daemon

The sovereign-sync daemon (`127.0.0.1:7892`) adds iroh QUIC P2P transport for cross-device CRDT synchronization. Learner model state, skill indices, and custom knowledge bases replicate automatically between peers.

```bash
# Daemon starts via launchd (macOS) / systemd (Linux) after install-skills-flat.sh
sovereign-sync --mode daemon

# Check status
curl -s http://127.0.0.1:7892/health | jq .
/sync-status
/sync-peers
```

AG-UI SSE events stream at `http://127.0.0.1:7892/events` for Tauri clients.

## Choosing a Mode

| Scenario | Recommended mode |
|---|---|
| CI pipeline or offline agent | Mode 0 |
| Single-user local development | Mode 1 |
| Active learning and tutoring | Mode 2 |
| Multi-device or collaborative learning | Mode 3 |

## Checking the current mode

```bash
bash shared/scripts/detect-toolchain.sh --json | jq '{mode: .deployment_mode, services: .mcp_servers}'
```

The `deployment_mode` field returns `"cli"`, `"mcp"`, `"full"`, or `"p2p"`.
