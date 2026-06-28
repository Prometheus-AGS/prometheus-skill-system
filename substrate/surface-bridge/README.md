# surface-bridge

Tier 2 MCP App server for the Prometheus learn domain. Exposes three HTTP
endpoints that harness-agnostic skills reach via MCP to deliver structured
operator input through an HTML shell (A2UI).

## Build

```bash
cargo build --release
```

## Run

```bash
./target/release/surface-bridge
# Listens on 127.0.0.1:7890
```

## macOS launchd install

```bash
cp com.prometheusags.surface-bridge.plist ~/Library/LaunchAgents/
launchctl load ~/Library/LaunchAgents/com.prometheusags.surface-bridge.plist
```

To unload:

```bash
launchctl unload ~/Library/LaunchAgents/com.prometheusags.surface-bridge.plist
```

## Endpoints

| Method | Path | Description |
|--------|------|-------------|
| `GET`  | `/health` | Health check — returns status, version, and PID |
| `POST` | `/mcp/detect-surface-tier` | Returns the active `SURFACE_TIER` and `CLAUDE_HARNESS` env values |
| `POST` | `/mcp/render-ui-intent` | Queues a `UiIntent` for display in the HTML shell |
| `POST` | `/mcp/collect-response` | Polls for operator input submitted through the HTML shell |

## Health check

```bash
curl http://127.0.0.1:7890/health
```

## Note

This is a Tier 2 stub. The iframe/AG-UI layer that displays rendered intents
and submits operator responses is deferred to a future phase. The
`render_ui_intent` handler logs and acknowledges intents; `collect_response`
returns `"pending"` until a response is written to the in-memory store.
