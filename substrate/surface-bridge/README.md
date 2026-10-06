# surface-bridge

Tier 2 MCP App server for the Prometheus learn domain. Exposes HTTP
endpoints that harness-agnostic skills reach via MCP to deliver structured
operator input through an HTML shell (A2UI).

## Local build (after the completed production phase)

```bash
cargo build --release
```

## Run

```bash
./target/release/surface-bridge
# Listens on 127.0.0.1:7890
```

## Optional service install (macOS launchd / Linux systemd --user)

```bash
npm run install:daemons     # installs/starts all Prometheus daemons, including surface-bridge
npm run uninstall:daemons   # stops them
```

Skill installation can build this binary without starting it. Service installation
is a separate operation, and its exit status does not prove the UI is usable. The
service installer requires Bash 4+; macOS system Bash 3.2 is insufficient.

Templates: `shared/launchagents/ai.prometheus.surface-bridge.plist` (macOS),
`shared/systemd/ai.prometheus.surface-bridge.service` (Linux). Both are
rendered and installed by `scripts/install-mcp-services.sh`.

## Endpoints

| Method | Path | Description |
|--------|------|-------------|
| `GET`  | `/health` | Health check — returns status, version, and PID |
| `POST` | `/mcp/detect-surface-tier` | Returns the active `SURFACE_TIER` and `CLAUDE_HARNESS` env values |
| `POST` | `/mcp/render-ui-intent` | Queues a `UiIntent` for display in the HTML shell |
| `POST` | `/mcp/submit-response` | Stores an operator response in the in-memory response store |
| `POST` | `/mcp/collect-response` | Polls for operator input submitted through the HTML shell |

## Health check

```bash
curl http://127.0.0.1:7890/health
```

## Note

This is a Tier 2 stub. The iframe/AG-UI layer that displays rendered intents
and submits operator responses is deferred to a future phase. The
`render_ui_intent` handler logs and acknowledges intents; `collect_response`
returns `"pending"` until a response is submitted to the in-memory store;
collection consumes that response. Restarting the service loses this store.
