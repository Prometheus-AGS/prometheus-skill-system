# Deep Research UI

`deep-research-ui.html` is the browser client for the `prometheus-research`
daemon. It renders live progress for a background research job, displays
A2UI HTMX components streamed via AG-UI SSE, and lets you start / cancel
jobs against a locally-running server on `127.0.0.1:7891`.

**The HTML is not a static preview.** It talks to a running daemon over
HTTP + SSE. You need the daemon on `:7891` for anything beyond the empty
shell to work.

## Three ways to run

Pick whichever fits your environment. All three end with
`http://127.0.0.1:7891/` in a browser.

### 1. Native launchd (macOS) — the default install

If you've run `bash scripts/install-binaries.sh` in the skill-pack root,
the daemon is already installed as a launchd service and comes up on
boot.

```bash
# Check the service is running
launchctl list | grep com.prometheus.research
curl -s http://127.0.0.1:7891/health

# Open the UI (daemon serves it directly)
open http://127.0.0.1:7891/
```

If the service isn't listed, run the install script or start it manually:

```bash
prometheus-research --mode server
```

Then browse to `http://127.0.0.1:7891/`.

### 2. `cargo run` from a checkout (any OS with Rust)

Useful for hacking on the daemon itself or on a machine without the
launchd service installed.

```bash
cd substrate/prometheus-research
cargo run --release -- --mode server
```

Then browse to `http://127.0.0.1:7891/`.

### 3. Docker Compose (any OS with Docker)

Portable path — good for sharing with a colleague or CI runners. Uses
Colima or Docker Desktop.

```bash
cd docs/deep-research
docker compose up
```

Then browse to `http://127.0.0.1:7891/`. `Ctrl+C` in the compose window
stops the container.

See [`docker-compose.yml`](docker-compose.yml) and
[`Dockerfile`](Dockerfile) for the container recipe.

## What each route does

The daemon at `:7891` serves both the UI and the data API from the same
axum server:

| Route | Handler | Notes |
|---|---|---|
| `GET /` | UI shell | The HTML in this directory, embedded in the binary at build time |
| `GET /static/{file}` | vendored JS | `htmx.min.js`, `alpine.min.js`, `hls.min.js`, HTMX extensions |
| `GET /brand/{file}` | KnowMe brand | `tokens.css` + `primary-{light,dark}{,-16,-32,-180,-512}.{svg,png}` |
| `GET /health` | health | `{status:ok}` used by the smoke test |
| `POST /api/v1/jobs` | start job | Kicks off a background research job |
| `GET /api/v1/jobs/{id}` | get job | Latest checkpoint state |
| `DELETE /api/v1/jobs/{id}` | cancel | Requests a job cancel |
| `GET /api/v1/jobs/{id}/events` | SSE | AG-UI event stream for a job |
| `GET /components/{name}` | HTMX | A2UI HTMX HTML fragments |

## Opening the HTML file directly

The HTML uses relative script paths (`./static/…`), so if you open the
raw `deep-research-ui.html` from disk with `file://` the shell renders,
but every API call fails (there's no daemon at `127.0.0.1:7891` from the
`file://` origin's perspective, and CORS will block cross-origin fetches
from a `null` origin).

**In short — the file loads, the buttons are inert.** Use one of the
three run modes above to get the working experience.

## Troubleshooting

- **UI loads but buttons do nothing.** Check `curl http://127.0.0.1:7891/health`.
  If it fails, the daemon isn't running.
- **Service was working, now it's crash-looping in launchd.** Check
  `/tmp/prometheus-research.error.log`. A `--mode mcp` in the plist
  under launchd will crash-loop because MCP mode expects an MCP client on
  stdin. The plist should be `--mode server`.
- **`docker compose up` fails to bind :7891.** Something else on the host
  is already using the port; likely the launchd service. Run
  `launchctl bootout gui/$(id -u) ~/Library/LaunchAgents/com.prometheus.research.plist`
  first, or change the port in `docker-compose.yml`.

## Vercel / static deploy note

`vercel.json` in this directory catches every non-static path and rewrites
to `deep-research-ui.html`. Vercel serves the shell fine but the fetches
to `127.0.0.1:7891` still fail because there's no daemon in Vercel's
runtime. Static deploy is useful for a design preview only, not a
functional demo.
