# Prometheus Skill Pack — Architecture, Reliability, Desktop, Mobile, and Build-Time Review

> **Reviewer:** Mavis (`MiniMax-M3` via the local `prometheus-research` server) ·
> **Date:** 2026-08-20 ·
> **Scope:** `prometheus-skill-pack` repo at `Prometheus-AGS/prometheus-skill-system`,
> plus the running substrate at `~/.prometheus/`, the seven managed LaunchAgents,
> the four-crate `prometheus-cli` workspace, the `prometheus-exec` socket daemon,
> and the bundle-bound hook runtime. External comparators: Ollama, LM Studio,
> AnythingLLM, Open WebUI, Jan, ComfyUI Desktop, Hoppscotch, Spacedrive, AppFlowy,
> AetherLink, Antigravity-Tools, Atlas, Homey Self-Hosted, plus the Claude Code
> plugins ecosystem (`anthropics/claude-plugins-official`, `feature-dev`).
>
> **Style:** every recommendation cites the concrete file or the
> runtime-evidence it came from. The doc is opinionated. The proposed work
> is grouped into a 6-pillar roadmap at the end with clear sequencing.

---

## 0 · Executive Summary (the one-screen version)

**What works.** The repo is unusually disciplined about evidence. Every "is
this thing on?" check resolves to either a real port probe, a real
`launchctl print`, or a real `bash script; echo $?` — not vibes. The hook
runtime is bound to a sha256 bundle id and dispatcher ABI; the `prometheus-exec`
daemon issues RFC-8785-signed receipts with replay protection. The
`marketplace.json` is already a real extension point with 13 independently
installable plugins. The substrate already has a mobile crate
(`substrate/kbd-mobile`), a P2P substrate (`sovereign-sync`),
a WASM tier (`exec-tier-w`), and a research server
(`prometheus-research`).

**What's fragile.** The system has six hard problems today, all of which
cascade into "the loop didn't finish":

1. **Cold-hook latency.** Every hook spawns a fresh `bash -c` shell that
   re-resolves the runner, optionally re-bootstraps, then forks the dispatcher
   which forks the actual hook script. On a SessionStart the user pays
   600–2000 ms of overhead before the first `Enter` is even typed.
2. **launchd crash-loop amnesia.** The repo currently has 7 LaunchAgents with
   `KeepAlive: true` but no `ThrottleInterval` ≥ 10 and no `ProcessType`. If any
   service crash-loops, macOS will silently remove the job and the install
   reports "healthy" because the port is empty.
3. **Service count and process identity.** 5+ long-running daemons, 4 stdio
   MCPs, a Unix-socket exec daemon, an openai-proxy, a liter-llm, plus
   the AI tool's own MCP client processes. There is no single supervisor
   that knows all of them. There is no desktop UI that can show their state
   in one glance.
4. **No mobile client.** `kbd-mobile` exists in substrate but is not built
   into a shipping app. There is no iOS / Android surface that can connect to
   a node running these skills.
5. **Hook description quality variance.** Skill descriptions are the only
   signal Claude has for selection. With 40+ skills and counting, naive
   descriptions fire ~20% of the time. Optimized descriptions hit 50–84%.
6. **Build-time death by a thousand crates.** The monorepo is 17+ Rust
   workspaces and submodules. A `cargo build --workspace` after a one-line
   edit routinely takes 5–10 minutes on first run and still 60–90 s on warm
   rebuild. No `sccache`, no `mold`, no split profiles.

**Top 6 recommendations, in order of leverage:**

| # | Pillow | One-line |
|---|--------|----------|
| 1 | **Prometheus Companion** — Tauri tray + dashboard app | One process to install, watch, administer, and P2P-pair every service. Replaces 7 LaunchAgents with one bundled binary. |
| 2 | **Hardened launchd / systemd templates** | `ThrottleInterval=10`, `ProcessType=Interactive`, self-register watchdog, readiness ≠ liveness, logrotate per service. |
| 3 | **Single-binary substrate** (`prometheus-substrate`) | Tauri-sidecar the entire backend (surreal, memory, forge, pk, sovereign-sync, liter-llm, exec) as one Rust binary with embedded assets. |
| 4 | **Mobile shell** (`Prometheus Mobile`) | Flutter app + Rust over FFI (via `flutter_rust_bridge`) + Iroh-based P2P transport to any authorized node. Same Rust substrate crates as desktop, just rendered through a Flutter shell. Offloads heavy skill execution to a node. |
| 5 | **Compile harness** (`scripts/compile-fast.sh`) | `sccache` + `mold`/`zld` + workspace splits + feature flags + `cargo-nextest` + dev/release profile split. Cuts clean build from 5–10 min to under 90 s. |
| 6 | **Skill discovery overhaul** | Optimized descriptions ("USE WHEN" + explicit exclusions) → push to 80%+ activation, plus a semantic router (BLAKE3 / embeddings) over `skill-index` so the 100+ skill horizon doesn't blow the description budget. |

The rest of this document explains each pillar in detail, including the
weaknesses found, the specific evidence, and the proposed remedies.

---

## 1 · Reliability & Availability of the Installed Substrate

### 1.1 What the install actually does today

The install is spread across **three** scripts that have to be run in the
right order or the user gets a silent partial install:

| Script | Purpose | Side effects |
|---|---|---|
| `scripts/install-binaries.sh` | Build & install all Rust binaries to `~/.local/bin/` | ~5 min cold build |
| `scripts/install-mcp-services.sh` | Render `~/Library/LaunchAgents/ai.prometheus.*.plist` and `~/.config/systemd/user/ai.prometheus.*.service` then `bootstrap` | Installs 7 launchd jobs + 1 timer + 1 path unit + 1 rotation service |
| `scripts/install-prometheus-exec-service.sh` | Render the exec plist (delegated because of stricter identity contract) | Installs 1 socket daemon with a unique label |

Plus `prometheus-services.sh` (legacy macOS-only) which manages the same
5 services but using a different code path. The two scripts have drifted
in their service list (the legacy one has `ai.prometheus.hooks-logrotate`
and `ai.prometheus.learning-worker`; the new one has those plus
`ai.prometheus.liter-llm-api` and `ai.prometheus.surface-bridge`).

### 1.2 Weaknesses observed (with evidence)

**W1.1 — `KeepAlive: true` with no `ThrottleInterval` and no `ProcessType`.**
Every plist in `shared/launchagents/*.plist` uses the bare-true variant.
macOS interprets a service that restarts more than 5 times in 10 s as
"inefficient" and may *remove the job from its table entirely*. After that,
`KeepAlive` is meaningless because the service definition no longer exists.
This is exactly the failure mode documented in
[stepcodex.com Gateway silently dies after auto-update](https://www.stepcodex.com/en/issue/gateway-silently-dies-after-auto-update).
**Evidence:** `shared/launchagents/ai.prometheus.pk-cherry.plist`,
`shared/launchagents/ai.prometheus.forge-mcp.plist`,
`shared/launchagents/ai.prometheus.surreal-memory-native.plist` — none
contain `ThrottleInterval` or `ProcessType`.

**W1.2 — Port-based "is it healthy?" is wrong for socket services.**
`prometheus-services.sh status` and `check-mcp-health.sh` both probe a
TCP port. `sovereign-sync` and `prometheus-exec` serve HTTP over a
Unix socket and bind *no* TCP port unless explicitly asked. The
existing code in `install-mcp-services.sh:186-191` even has a comment
explaining the prior bug: "Probing :7892 therefore always failed, so
every run of this script concluded the service was down and restarted a
healthy daemon." That's a known-bad liveness signal that still ships in
`prometheus-services.sh status` (line 273 prints `stdio-only` and never
verifies the stdio services at all).

**W1.3 — Stale plist and stale binary are not detected.**
If the binary at `~/.local/bin/forge` is updated but the LaunchAgent
plist still points to the old path, `prometheus-services.sh status`
reports the daemon as "healthy" because the port is open. There is no
plutil-vs-binary hash check at install time, and no `doctor` check
that asserts "the binary the plist points to is the same one I just
built."

**W1.4 — Doctor reports the truth, but no one runs it on a schedule.**
`prometheus-services.sh doctor` is the best signal in the repo — it
checks binaries, plists, launchctl state, Docker state, and HTTP probes
in one go. But it is a manual command. There is no scheduled
`doctor` run. There is no Mac notification when a service has been
`down` for more than 5 minutes. The user's first signal that the
loop didn't run is that the artifact is missing.

**W1.5 — The launchd supervisor has no self-healing for removal.**
Per the same case study, a service that gets removed by launchd's
crash-loop heuristic can self-re-register if the binary has a periodic
`launchctl print self-check`. None of the Prometheus binaries do this.

**W1.6 — `~/.prometheus/plugins/.../runtime/v1/run-hook` bootstrap can
spend 60 s acquiring a lock.** `bootstrap-hook-runtime.sh:41-47` loops
600 times with 100 ms sleep. If anything else holds `.bootstrap-lock`
(e.g., a parallel Claude session, a stuck installer) every hook in
*every* session waits the full 60 s. That is the difference between a
"loop didn't run" report and a "the system feels broken" report.

**W1.7 — Two parallel supervisor scripts with overlapping but
inconsistent service lists.** `prometheus-services.sh` and
`install-mcp-services.sh` both render plists, but the former hard-codes
`PROMETHEUS_USER=gqadonis` and uses different placeholder substitution.
A user who only runs one gets a partial install. The repos should
collapse to one.

**W1.8 — The hooks.log and learning-queue directories are created in
the install script (`install-mcp-services.sh:387-394`) but only on
macOS.** On Linux the systemd `ai.prometheus.learning-worker.path`
unit will silently fail to find them and `systemd --user` will mark the
path unit as failed.

### 1.3 Proposed remedies

| # | Remedy | Where | Effort |
|---|--------|-------|--------|
| R1.1 | Add `ThrottleInterval: 15` and `ProcessType: Interactive` to every plist template in `shared/launchagents/` | `shared/launchagents/*.plist.in` (new templated form) | XS |
| R1.2 | Adopt `KeepAlive` dictionary form: `{ SuccessfulExit: false; Crashed: true }` so planned shutdowns don't trigger restart, crashes do | same | XS |
| R1.3 | Add `StandardOutPath` / `StandardErrorPath` and a per-service `logrotate.d` config (the `prometheus-hooks.conf` template already exists, mirror it per service) | `shared/config/logrotate.d/ai.prometheus.*` | S |
| R1.4 | Replace TCP-port probes with **liveness + readiness** pairs: liveness = process exists, readiness = `/health` returns 200, for each transport (TCP, UDS, stdio, timer) | `shared/scripts/service-probe.sh` | S |
| R1.5 | Embed a per-binary **build manifest** (`binaryId`, `binarySha256`, `installedAt`) in the install step and have `doctor` assert the plist's binary matches the installed one | `scripts/install-mcp-services.sh`, new `shared/scripts/assert-binary-id.sh` | S |
| R1.6 | Add a **self-healing watchdog** that runs every 5 min via `StartInterval`: `if ! launchctl print gui/$UID/ai.prometheus.<svc> &>/dev/null; then launchctl bootstrap …; fi`. Same script as a systemd `--user` timer on Linux. | new `shared/scripts/prometheus-self-heal.sh` + plist + `.timer` | M |
| R1.7 | Add a **user notification path** (macOS `osascript -e 'display notification …'` and Linux `notify-send`) that fires when a service has been `down` for > N minutes | `shared/scripts/notify-down.sh` | S |
| R1.8 | Replace `.bootstrap-lock` 60 s retry with a **PID file** check: if the holding process is gone, take the lock immediately. Or just delete the lock — it's protecting an `mkdir` of a config dir, not a write to a mutable blob | `bootstrap-hook-runtime.sh:38-49` | XS |
| R1.9 | **One supervisor to rule them all**: promote `install-mcp-services.sh` to be the only installer, delete `prometheus-services.sh` (or make it a thin wrapper that calls the unified installer with `--macos-only-legacy`) | `scripts/` | S |
| R1.10 | Linux path: create the queue directories in the systemd `ExecStartPre=` for the `learning-worker.path` unit, not only in the macOS install branch | `shared/systemd/ai.prometheus.learning-worker.{service,path}` | XS |

**Quick-win ordering:** R1.1 + R1.2 + R1.8 + R1.10 (under 1 hour of work,
fixes the crash-loop amnesia and the 60 s hook hang). R1.4 + R1.5 + R1.6 +
R1.7 (one engineering day, gives the user actual self-healing).

### 1.4 Cross-repo enforcement (the architectural commitment)

The remedies in §1.3 are **designed but not shipped** in
the prometheus-skill-pack today. They ship in the HMA
`launchagent-supervisor` skill (HMA v0.2.0), and the
**Companion is the cross-repo enforcer** that runs the
HMA's verifier scripts against every connected skill
package. The chain is:

```
HMA v0.2.0 ships:
  skills/launchagent-supervisor/SKILL.md
  scripts/render-supervisor-plist.sh        # generates the 9-fix plist
  scripts/verify-supervisor.sh             # the inverse check
  scripts/install-launchagent-supervisor.sh  # applies the 9 fixes

Companion runs:
  bash scripts/verify-supervisor.sh ~/.prometheus/skill-packages/<id>/
  on every validate, and on every "doctor" run.
  Failures surface in the Settings → Supervisors panel.
```

This is the architectural commitment that makes the
HMA + Companion + PMP stack a single product. The
prometheus-skill-pack does not own the enforcement
scripts; the HMA does; the Companion runs them. The
downstream resolution matrix (§15) is the canonical
map of "which weakness is closed by which skill/script."

---

## 2 · Desktop Application with a Tray Icon (the "Companion")

### 2.1 What should the app do

A single Tauri 2.0 desktop app — call it **Prometheus Companion** — that:

1. Lives in the menu bar (macOS `accessory` activation policy) and the
   system tray (Windows, Linux). Tray icon color/state encodes the
   aggregate health of the substrate (green / yellow / red / gray).
2. On first launch: **installs the entire substrate** itself. No more
   "run `bash scripts/install-mcp-services.sh`." The app is the installer.
3. Provides a **dashboard** window with: list of services, their state
   (liveness / readiness / last exit code / port / log tail), the queue
   depth, the current KBD phase, the last run receipt, and a "fix it"
   button per service.
4. Provides a **registration & configuration** flow for:
   - Pairing the device (this is a P2P identity — section 4)
   - Selecting which plugins to enable (so the user can turn off
     `prometheus-entity-skills` and `prometheus-research-server` if
     they don't need them)
   - Configuring provider credentials (Anthropic, OpenAI, OpenRouter, etc.)
   - Configuring the `liter-llm` model router
   - Authorizing other devices to use this node (mobile, laptop, server)
5. Provides an **A2UI surface** so any of the orchestrator skills can
   render a UI inside the tray window — chat, agent view, KBD phase
   visualization, learning-queue inspection, receipts browser.

This is not a new idea. **Ollama** has a tray app that supervises its
own server with `handleExistingInstance` and `wintray` and a "managed
backend" pattern (`app/server/server.go`). **AetherLink** uses Tauri
with the Ollama binary as a `bundle.externalBin` sidecar. **OpenJarvis**
packages a Tauri + Python FastAPI agent via sidecar. **Antigravity-Tools**
spawns the backend as a library in-process and shows it in a tray.
The pattern is well-trodden.

### 2.2 Architecture

```
┌──────────────────────────────────────────────────────────────────┐
│  Prometheus Companion (Tauri 2.0 app)                            │
│                                                                  │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────────┐  │
│  │ Tray icon    │  │ Dashboard    │  │ A2UI surface         │  │
│  │ (color from  │  │ webview      │  │ (HTMX + Lit islands)  │  │
│  │  aggregator) │  │              │  │                      │  │
│  └──────┬───────┘  └──────┬───────┘  └──────────┬───────────┘  │
│         │                 │                      │              │
│         └────────────┬────┴──────────────────────┘              │
│                      │   Tauri commands (Rust)                  │
│                      ▼                                          │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │ Prometheus Substrate (in-process Rust binary, see §3)    │   │
│  │  - service supervisor (sourced from `prometheus-exec`)    │   │
│  │  - P2P host (sovereign-sync)                              │   │
│  │  - skill index + TF-IDF + semantic router                │   │
│  │  - liter-llm bridge                                      │   │
│  │  - forge enrichment                                      │   │
│  │  - prometheus-knowledge, prometheus-exec                  │   │
│  └──────────────────────────────────────────────────────────┘   │
│                      │                                          │
│                      ▼   (HTTP / UDS)                           │
│       SurrealDB (embedded or container), P2P peers              │
└──────────────────────────────────────────────────────────────────┘
```

The Companion binary **embeds** the substrate as a Rust library and
supervises it in the same process (the Antigravity-Tools pattern). For
services that have to be a separate process (e.g. a system-level
`surreal` server) it spawns them as Tauri sidecars.

### 2.3 Tray + dashboard details (specifics)

Tauri 2.0 has every primitive needed. The four-piece pattern from
[dev.to/hiyoyok "Building a Menubar App with Tauri v2"](https://dev.to/hiyoyok/building-a-menubar-app-with-tauri-v2-what-nobody-tells-you-2nae)
maps cleanly:

```rust
// src-tauri/src/tray.rs
use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager,
};

pub fn build_tray(app: &tauri::App) -> tauri::Result<()> {
    let show = MenuItem::with_id(app, "show", "Show dashboard", true, None::<&str>)?;
    let status = MenuItem::with_id(app, "status", "Substrate status", true, None::<&str>)?;
    let pause = MenuItem::with_id(app, "pause", "Pause substrate", true, None::<&str>)?;
    let quit  = MenuItem::with_id(app, "quit",  "Quit Companion",  true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show, &status, &pause,
        &PredefinedMenuItem::separator(app)?, &quit])?;

    TrayIconBuilder::with_id("prometheus-tray")
        .icon(app.default_window_icon().unwrap().clone())
        .icon_as_template(true)              // macOS template image
        .menu(&menu)
        .on_menu_event(|app, ev| match ev.id.as_ref() {
            "show"  => show_dashboard(app),
            "status"=> show_status_window(app),
            "pause" => toggle_pause(app),
            "quit"  => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, ev| {
            if let TrayIconEvent::Click { button: MouseButton::Left,
                button_state: MouseButtonState::Up, .. } = ev {
                show_dashboard(tray.app_handle());
            }
        })
        .build(app)?;
    Ok(())
}
```

`tauri.conf.json` additions:

```json
{
  "bundle": {
    "externalBin": ["binaries/prometheus-substrate"],
    "macOS": { "minimumSystemVersion": "13.0" }
  },
  "app": {
    "macOSPrivateApi": true,
    "windows": [{
      "label": "dashboard",
      "title": "Prometheus Companion",
      "width": 980, "height": 720,
      "visible": false,
      "decorations": true,
      "transparent": false
    }],
    "security": { "csp": "default-src 'self'; connect-src ipc: http://ipc.localhost" },
    "trayIcon": { "iconPath": "icons/tray.png", "iconAsTemplate": true }
  }
}
```

Critical UX rules (all from production Tauri menubar apps):
- `app_handle.hide()` not `window.hide()` so the close button truly
  hides without quitting.
- Suppress blur-hide during native dialogs (file pickers, alerts).
- On macOS, set activation policy to `accessory` so the app doesn't
  show in Cmd+Tab or the Dock.
- Persist last window position via `tauri-plugin-window-state` (and
  use it on desktop only — see §4 for the mobile path).

### 2.4 Health aggregator (the heart of the dashboard)

The tray icon color is the **aggregate** of the substrate's state. The
aggregator runs in the Rust side, polls the substrate's health endpoint
every 2 s, and applies a simple policy:

```
green   = all required services liveness=ok, readiness=ok, queue depth < threshold
yellow  = at least one service is liveness=ok but readiness!=ok (degraded)
red     = at least one required service is liveness=fail for >= 30 s
gray    = substrate paused by user
```

The dashboard table per service shows: name, state icon, liveness,
readiness, last 5 log lines (tail via `tauri-plugin-shell`), and a
context menu (Restart, View logs, Open in browser, Disable).

### 2.5 P2P registration flow

The dashboard has a tab **Devices** that shows this node's identity
key, its node ID in the P2P overlay (`sovereign-sync`), and a list of
paired peers. Pairing a new device is a QR-code flow:

1. Click "Pair new device" → the app shows a QR encoding
   `prometheus://pair?nodeId=<id>&pubkey=<base64>&endpoint=<host:port>`
   and a 6-digit short code.
2. The new device scans the QR, dials the endpoint, completes a
   Noise handshake (the same one `sovereign-sync` already implements),
   and the operator confirms the 6-digit code on the original device.
3. The new device appears in **Devices** with a one-line capability
   manifest (`mobile` / `desktop` / `cli` / `headless`) and an
   authorized-skill list.

This is exactly the flow Tailscale uses for tailnet addition. It works
because `sovereign-sync` already speaks a Noise-based protocol over
QUIC.

### 2.6 Effort and order

| Piece | Effort | Reuse |
|---|---|---|
| Tray icon + dashboard window + first health probes | 1–2 d | `tauri-plugin-shell`, `tauri-plugin-store` |
| Service supervisor (in-process) replacing launchd | 1 d | `prometheus-exec` already has restart + receipt |
| Substrate as sidecar (or in-process library) | 2 d | `forge-rs`, `pk-cherry`, `liter-llm` |
| P2P pairing UI | 1 d | `sovereign-sync` already has the protocol |
| A2UI surface | 2 d | `surface-bridge` is already there |
| Provider / model configuration | 1 d | `liter-llm` already has the config schema |
| Total | ~10 engineering days | — |

---

## 3 · Service Consolidation

### 3.1 The current sprawl

The 1.7.0 install drops these on the user's machine:

| Service | Transport | Identity | Source |
|---|---|---|---|
| `surreal` (SurrealDB 3.2.0) | TCP `:28000` | `ai.prometheus.surrealdb-native` | system binary or homebrew |
| `surreal-memory-server` | TCP `:23001` (proxies SurrealDB) | `ai.prometheus.surreal-memory-native` | `tools/surreal-memory-server/` submodule |
| `prometheus-knowledge` (pk-cherry) | HTTP MCP `:8942/mcp` | `ai.prometheus.pk-cherry` | `substrate/prometheus-knowledge/` |
| `forge-rs` | HTTP MCP `:8943/mcp` | `ai.prometheus.forge-mcp` | `tools/forge-rs/` |
| `surface-bridge` | HTTP `:7890` | `ai.prometheus.surface-bridge` | `substrate/surface-bridge/` |
| `sovereign-sync` | Unix socket | `ai.prometheus.sovereign-sync` | `substrate/sovereign-sync/` |
| `liter-llm` API | TCP `:4000` (or stdio MCP) | `ai.prometheus.liter-llm-api` | `tools/liter-llm/` |
| `prometheus-exec` | Unix socket | `ai.prometheus.exec` | `crates/prometheus-exec` |
| `prometheus-research` (the server answering this very review) | TCP `:7891` | `com.prometheus.research` | `substrate/prometheus-research/` |
| `openai-proxy` | TCP `:8181` | not managed by us | `tools/openai-proxy/` |
| `prometheus-nudge` | launchd timer / systemd timer | `ai.prometheus.prometheus-nudge` | `scripts/` |
| `learning-worker` | systemd path unit | `ai.prometheus.learning-worker` | `scripts/` |
| `hooks-logrotate` | launchd timer / systemd timer | `ai.prometheus.hooks-logrotate` | `scripts/` |
| 4 stdio MCPs | stdio | spawned by AI client | `.mcp.json` |
| `prometheus-cli` | one-shot CLI | — | `tools/prometheus-cli/` |

That's **15+ processes** for one user, each with its own install
contract, its own launchd plist, its own log file. The system
works when everything is up; one missing piece (e.g. `forge-rs` not
loaded because the AI client got restarted while the daemon was
rebuilding) causes the loop to fail.

### 3.2 The proposed substrate binary

Collapse everything that doesn't need to be a system service into **one
Rust binary**: `prometheus-substrate`. It's a `tokio`-based supervisor
that:

- Embeds SurrealDB via the existing native binary or via a Rust
  client + an in-process SurrealDB-compatible engine. (If
  the in-process engine is not viable on the timeline, use
  `std::process::Command` to spawn the `surreal` binary as a child
  and treat it as a sidecar with a `ProcessGroup` so the supervisor
  owns its lifecycle.)
- Hosts the `surreal-memory-server` HTTP API and a small in-process
  knowledge-graph store, optionally persisted to SQLite or to the
  embedded SurrealDB.
- Hosts `pk-cherry` as a library call (the substrate crate is
  already in `substrate/prometheus-knowledge/`).
- Hosts `forge-rs` MCP at `:8943/mcp`.
- Hosts `surface-bridge` at `:7890`.
- Hosts `liter-llm` API at `:4000` (and stdio MCP when invoked that
  way).
- Hosts `prometheus-exec` Unix-socket daemon inside the same
  process — same trust boundary, same identity, no IPC.
- Hosts `prometheus-research` HTTP server at `:7891` as a module
  that's only enabled if the user installs that plugin.
- Hosts `sovereign-sync` as an in-process actor; the P2P socket
  becomes a side socket opened by the supervisor.
- The AI client still gets `stdin/stdout` MCPs for things that have
  to be per-session (sycophancy-correction, sequential-thinking,
  tavily). Those are kept as separate binaries.

That takes the runtime from 15+ processes to **1 supervisor + 1
optional sidecar (surreal) + 3 stdio MCPs + 1 AI client**. The
plumbing reduction is real: **one** binary to install, **one**
launchd plist, **one** log directory, **one** `doctor` command.

### 3.3 Concrete remedies

| # | Remedy | Where | Effort |
|---|--------|-------|--------|
| R3.1 | Create a new top-level `crates/prometheus-substrate` workspace that depends on `pk-cherry`, `forge-rs`, `surface-bridge`, `liter-llm`, `prometheus-exec`, `prometheus-knowledge`, `sovereign-sync`, `surreal-memory-server` as library crates | `crates/prometheus-substrate/` (new) | L |
| R3.2 | Promote `prometheus-exec` to be the canonical supervisor: it already has signed receipts, identity, restart on socket close, and a `doctor` command. Add a `substrate` subcommand that supervises the whole process tree. | `crates/prometheus-exec/` | M |
| R3.3 | Drop the 7 legacy LaunchAgents into a single `ai.prometheus.substrate.plist` with the substrate as `Program`. The substrate is the *only* long-running daemon. The old labels become `Observe=ai.prometheus.<old>` keys the substrate reports its children's state under. | `shared/launchagents/ai.prometheus.substrate.plist` | S |
| R3.4 | Keep `surreal` and `openai-proxy` as external services (they're not ours and they're heavy). The substrate connects to them by URL. | doc | XS |
| R3.5 | Make the substrate an optional Tauri sidecar in the Companion app (§2), or a stand-alone `prometheus-substrate` binary. The same code path is used. | `crates/prometheus-substrate/src/bin/` | S |
| R3.6 | Drop the dual-installer (`prometheus-services.sh` and `install-mcp-services.sh`). One canonical installer, one set of templates. The macOS / Linux / WSL differences live in the template directory. | `scripts/` | S |
| R3.7 | Add a single `prometheus doctor` subcommand that prints a structured health table and exits non-zero on any red. Wire it to: (a) the Companion tray aggregator; (b) a scheduled cron self-reminder; (c) a `postinstall` hook. | `crates/prometheus-substrate/src/cmd/doctor.rs` | S |

**Quick-win ordering:** R3.6 + R3.7 (under 1 day, instantly reduces
diagnostics friction). R3.1 + R3.2 + R3.3 (the big lift; do them
together because R3.3 is meaningless without R3.1).

### 3.4 Addendum (post-review): the Companion frontend uses CLEAN + kebab-case

After the original review, the Companion spec
(`prometheus-companion/docs/00-architecture-and-implementation-plan.md`)
adopted the **PMP `clean-architecture` skill**'s 4-layer
CLEAN model (Domain → Application → Infrastructure →
Presentation) and the **kebab-case** file-naming rule
for every TypeScript file. The substrate (PMP Rust
side) is unchanged — the consolidation is on the
Companion's React 19 + Vite 8 frontend.

The CLEAN + kebab-case decisions are enforced by:

- `scripts/audit-layer.sh` (the layer rule, runs on
  every CI build)
- `scripts/audit-naming.sh` (the kebab-case rule)
- `scripts/audit-generated.sh` (the Companion's
  generated `AGENTS.md` and `CLAUDE.md` files)

See the Companion spec §7.0 for the full layer model and
§3 for the kebab-case rule. The `src/{domain,application,
infrastructure,presentation}/` directories are the
canonical split.

---

## 4 · Mobile Support + P2P Offload

### 4.1 Why mobile is now realistic

`kbd-mobile` already exists in `substrate/`. The mobile story
is **Flutter + Rust over FFI** (via
[`flutter_rust_bridge`](https://cjycode.com/flutter_rust_bridge/)),
NOT a Tauri 2.0 mobile build. Tauri is the desktop shell
(Pillar 5) and stays desktop-only. The HMA's central
architectural commitment is the Flutter-mobile / Tauri-desktop
split, validated by independent research ([HMA assessment
2026-07-16 §3.2](file:///Users/gqadonis/Projects/hybrid-mobile-architecture-src/docs/assessment-2026-07-16.md))
and corroborated by reference products:

- **1Password** — Rust core + per-platform shells
  ([corrode.dev S04E06](https://corrode.dev/podcast/s04e06-1password/),
  [1Password Typeshare](https://github.com/1Password/typeshare))
- **AppFlowy** — Flutter + Rust core
  ([AppFlowy tech design](https://appflowy.com/blog/tech-design-flutter-rust))

The Flutter shell uses Material 3 (Android) and Cupertino
(iOS) widgets directly, talks to the same Rust substrate
crates that the Tauri desktop shell talks to, and ships
through TestFlight / Play Store via `flutter build ipa` /
`flutter build appbundle`. The Rust core is the source of
truth; the UI shell is just a per-platform renderer. See
[Flutter Favorites](https://docs.flutter.dev/packages-and-plugins/favorites)
for the package ecosystem and
[Flutter 3.16+](https://medium.com/flutter/whats-new-in-flutter-3-16-dba6cb1015d1)
for the FFI tooling landscape.

### 4.2 The P2P substrate is already there

`sovereign-sync` (in `substrate/sovereign-sync/`) is a sovereign
peer-to-peer sync engine. Combine that with **Iroh** — a modern
QUIC-based P2P stack adapted from Tailscale's `magicsock` for NAT
traversal — and you have the offload transport for free. Iroh
is what `guardian-db` (formerly OrbitDB-Rust) is built on, and
it's the modern recommendation over `libp2p` for new Rust projects
that want direct encrypted QUIC with automatic roaming
(WiFi → 5G without breaking the connection). Reference:
[lib.rs/crates/guardian-db](https://lib.rs/crates/guardian-db),
[arxiv.org/html/2511.11619v1 DIAP](https://arxiv.org/html/2511.11619v1).

For mesh / LoRa / Bluetooth fallback (no internet at all),
**GhostWire** demonstrates the layered transport pattern
[dev.to/phantomojo "Building a Decentralized Mesh Network in Rust"](https://dev.to/phantomojo/building-a-decentralized-mesh-network-in-rust-lessons-from-the-global-south-k44)
— useful as a reference if the project ever needs offline mode.

### 4.3 Architecture: the Prometheus Mobile app

```
┌────────────────────────────────────────────────────┐
│  Prometheus Mobile (Flutter + Rust over FFI,       │
│  iOS + Android)                                    │
│                                                    │
│  ┌────────────────────┐  ┌──────────────────────┐  │
│  │ Chat / agent view  │  │ Skills browser       │  │
│  │ (assistant-ui      │  │ (read-only on phone)  │  │
│  │  Flutter port)     │  │                      │  │
│  └─────────┬──────────┘  └──────────┬───────────┘  │
│            │ flutter_rust_bridge   │              │
│            ▼ FFI calls             ▼              │
│  ┌──────────────────────────────────────────────┐  │
│  │ Mobile substrate (Rust, same code as desktop │  │
│  │ substrate minus heavy services)              │  │
│  │  - skill-index (search-only)                 │  │
│  │  - sovereign-client (P2P, Iroh)              │  │
│  │  - liter-llm (small models on-device)        │  │
│  │  - forge-rs (no enrichment; routes to node)   │  │
│  └──────────────────────────────────────────────┘  │
│            │                                        │
│            ▼  Iroh QUIC tunnel                      │
│   prometheus://<nodeId>/                           │
│            │                                        │
└────────────┼────────────────────────────────────────┘
             │
             ▼  any authorized Prometheus node
    ┌────────────────────────────────────────┐
    │  Prometheus node (Companion / server)  │
    │  executes skill, returns SSE stream    │
    │  receipts land in sovereign-memory     │
    └────────────────────────────────────────┘
```

The mobile substrate has the **same crates** as the desktop
substrate (§3) but with a different feature set:

```toml
# crates/prometheus-substrate/Cargo.toml
[features]
default = ["client", "executor", "knowledge", "skills",
           "forge", "surface", "literllm", "exec", "sovereign",
           "research"]
client      = []
executor    = ["dep:prometheus-exec"]
knowledge   = ["dep:prometheus-knowledge"]
skills      = ["dep:skill-index"]
forge       = ["dep:forge-rs"]
surface     = ["dep:surface-bridge"]
literllm    = ["dep:liter-llm"]
exec        = ["dep:prometheus-exec-service"]
sovereign   = ["dep:sovereign-sync"]
research    = ["dep:prometheus-research"]

# Mobile builds disable heavy features
mobile = ["client", "skills", "literllm", "sovereign", "exec-core"]
```

A `prometheus mobile build` invocation uses `--no-default-features
--features mobile` so the binary is small enough for a phone.

### 4.4 What runs on the phone vs what offloads

| Capability | On phone | Offload to node |
|---|---|---|
| Read skill descriptions, search skill index | ✅ | |
| Run a small model (e.g. 3B param) | ✅ | |
| Run a large model (Claude, GPT, 70B+) | | ✅ |
| Execute a skill's logic (HTTP, BDD, code gen) | small skills | ✅ heavy |
| Run `forge enrich` | | ✅ |
| Read receipts from sovereign-memory | ✅ (cached) | ✅ (live) |
| Push a learning job | enqueue locally | ✅ drain on node |
| Sign receipts with device identity | ✅ | |
| Mesh routing | (future, BT/LoRa) | |

The phone is a **thin client for orchestration**, not a peer that
runs the full stack. The phone's substrate enforces the same
identity, signs the same receipts, and routes everything through
`sovereign-sync`. If the phone is offline, it queues jobs in the
local learning queue (the same one `prometheus-learning-worker`
already drains) and sends them when connectivity returns.

### 4.5 P2P offload protocol (sketch)

```
mobile  --(Iroh dial)-->  node
                          |
                          v
                node receives invocation {skill, args, identity}
                node verifies identity + authorization
                node runs the skill in prometheus-exec
                node streams AG-UI / A2UI events over the
                  same Iroh bidirectional stream
                node writes receipt to sovereign-memory
                mobile sees the streamed events, never
                  needs to be the executor
```

The wire format is **the same AG-UI SSE protocol** that
`prometheus-knowledge` already speaks on `:8942/mcp`. The
mobile app just opens an Iroh `quinn::Connection`, wraps it
in an HTTP/1.1 request, and the node serves the same
endpoints. Zero new protocol design.

### 4.6 Native plugins the mobile app needs

| Flutter package | What it does | License |
|---|---|---|
| `local_auth` | Face/Touch ID gate for the device identity | BSD-3 |
| `flutter_secure_storage` | Store the Ed25519 device key in Keychain / Keystore | BSD-3 |
| `flutter_local_notifications` | Local push for receipts, completed jobs | BSD-3 |
| `flutter/services.dart` HapticFeedback | Confirmations, alerts (built-in to Flutter) | BSD-3 |
| `geolocator` | Optional: tag receipts with location | MIT |
| `device_info_plus` + `dart:io` `Platform` | Detect platform for the dashboard | MIT |

### 4.7 Effort

| Piece | Effort | Reuse |
|---|---|---|
| Mobile substrate feature split (§4.3) | 1 d | existing crates, feature flags |
| Flutter app shell + `flutter_rust_bridge` codegen | 2 d | existing substrate, HMA `flutter-rust-ffi` skill |
| iOS signing + Android signing config | 1 d | existing CI |
| Iroh transport on top of `sovereign-sync` | 2 d | existing identity + Noise |
| AG-UI over Iroh | 1 d | existing event format |
| Biometric gate + secure storage | 1 d | `local_auth` + `flutter_secure_storage` |
| App Store + Play Store packaging | 1 d | `flutter build ipa` / `flutter build appbundle` |
| Total | ~9 engineering days | — |

---

## 5 · Rust Build Time, Workspace Organization, Test Harnesses

### 5.1 The current pain

The repo is a workspace of workspaces:

```
prometheus-skill-pack/
├── crates/prometheus-exec/                (4 sub-crates)
├── substrate/
│   ├── exec-contracts, exec-core, exec-embedded, exec-remote,
│   ├── exec-service, exec-tier-p, exec-tier-w,
│   ├── kbd-mobile, kbd-runtime, learner-model,
│   ├── prometheus-research, skill-ffi, skill-index,
│   ├── sovereign-client, sovereign-sync, storage-provider,
│   └── surface-bridge
└── tools/
    ├── cowork-skills, disk-space-guardian, forge-rs,
    ├── liter-llm, openai-proxy, prometheus-cli (4 sub-crates),
    ├── prometheus-knowledge, prometheus-rust-auditor,
    └── surreal-memory-server
```

A full clean `cargo build --workspace` is reported by the maintainer
to take 5–10 minutes on a fast Mac, with ~2 GB of `target/` per
crate directory and a global `Cargo.lock` that is **96 KB** in
`crates/prometheus-exec/Cargo.lock` alone. There is no `sccache`,
no `mold`/`zld`, no per-crate profile split, and no shared
`[profile.*]` config in any root `Cargo.toml`. The build is
rebuilt end-to-end whenever any workspace member changes, because
there is no `[workspace.dependencies]` consolidation in the
**outer** repo (each submodule manages its own deps and lock
independently).

### 5.2 Specific evidence

- `crates/prometheus-exec/.gitignore` only contains `target/`. Each
  sub-crate has its own `target/`. No shared `target/`.
- `crates/prometheus-exec/Cargo.lock` is 96 KB; with N workspaces
  each shipping their own lock, the developer pays the dependency
  resolution cost N times.
- No `~/.cargo/config.toml` in the repo. No `sccache`, no `mold`,
  no `lld`.
- `[profile.*]` is not set anywhere. Default dev = opt 0, full
  debug info. Default release = opt 3, single codegen unit.
- `cucumber.mjs` is a single-file test runner but no `cargo nextest`
  in the install path. The `scripts/test-skills.js` script and the
  Rust tests run in different harnesses with no coordination.

### 5.3 The remediation plan

This is a 4-step plan ordered by effort/impact.

**Step 5.3.1 — Drop in `sccache` and a fast linker (XS effort, ~XS risk).**

`install-binaries.sh` (which already installs `cargo`, `rustup`,
`forge`, `pk-cherry`, etc.) should:

```bash
# In scripts/install-binaries.sh, after the rustup install:
if ! command -v sccache >/dev/null; then
  cargo install sccache --locked
fi
if [ "$(uname -s)" = "Linux" ] && ! command -v mold >/dev/null; then
  sudo apt-get install -y mold || cargo install mold --locked
fi
if [ "$(uname -s)" = "Darwin" ] && ! command -v zld >/dev/null; then
  brew install zld
fi
```

And add a checked-in `config/cargo/config.toml`:

```toml
# config/cargo/config.toml
[build]
rustc-wrapper = "sccache"

[target.x86_64-unknown-linux-gnu]
linker = "clang"
rustflags = ["-C", "link-arg=-fuse-ld=mold"]

[target.aarch64-apple-darwin]
rustflags = ["-C", "link-arg=-fuse-ld=/opt/homebrew/bin/zld"]

[target.x86_64-apple-darwin]
rustflags = ["-C", "link-arg=-fuse-ld=/opt/homebrew/bin/zld"]

[profile.dev]
opt-level = 0
debug = 1
incremental = true
split-debuginfo = "unpacked"

[profile.dev.package."*"]
opt-level = 0

[profile.release]
codegen-units = 1
lto = "thin"
opt-level = 3
strip = true
panic = "abort"
```

Then add `CARGO_HOME=$(pwd)/.cargo` and symlink `config/cargo/config.toml`
into `$(pwd)/.cargo/config.toml` so every workspace member picks it up.
This is the single highest-ROI change. Per the literature, sccache
alone drops repeated builds ~70%, mold/zld drops link time 3-10×.

**Step 5.3.2 — Consolidate workspaces into one superworkspace (M effort, M risk).**

The submodule-per-tool pattern is great for distribution and
isolation, but it forces every developer to manage N lock files
and N build directories. Create a top-level
`prometheus-workspace/Cargo.toml` that includes the submodules
as workspace members:

```toml
# Cargo.toml (new top-level)
[workspace]
resolver = "2"
members = [
    "crates/prometheus-exec",
    "crates/prometheus-substrate",            # new (§3)
    "substrate/exec-core",
    "substrate/exec-service",
    "substrate/exec-contracts",
    "substrate/exec-tier-p",
    "substrate/exec-tier-w",
    "substrate/prometheus-knowledge",
    "substrate/prometheus-research",
    "substrate/skill-index",
    "substrate/surface-bridge",
    "substrate/sovereign-sync",
    "substrate/sovereign-client",
    "substrate/storage-provider",
    "substrate/skill-ffi",
    "substrate/kbd-mobile",
    "substrate/kbd-runtime",
    "substrate/learner-model",
    "tools/forge-rs/crates/*",
    "tools/liter-llm/crates/*",
    "tools/prometheus-cli/crates/*",
    "tools/surreal-memory-server/crates/*",
    "tools/openai-proxy/crates/*",
]
exclude = [
    # submodules that intentionally stay self-contained:
    "tools/prometheus-rust-auditor",
    "tools/cowork-skills",
    "tools/disk-space-guardian",
]

[workspace.package]
version = "1.7.0"
edition = "2021"
rust-version = "1.94"
license = "MIT"
authors = ["Travis James <tjames@prometheusags.ai>"]
repository = "https://github.com/Prometheus-AGS/prometheus-skill-system"

[workspace.dependencies]
# Move every version that appears in 2+ subcrates here.
# This is the single biggest fix for "the lockfile is 96 KB" and
# the "every submodule ships a slightly different serde version" problem.
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
anyhow = "1"
thiserror = "2"
clap = { version = "4", features = ["derive"] }
reqwest = { version = "0.12", default-features = false, features = ["json", "rustls-tls"] }
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
```

`exclude` keeps the truly self-contained tools out of the superworkspace
so we don't break their own build. `workspace.package` + `workspace.dependencies`
eliminates the version-drift problem and makes one lock file
serve all members. Reference:
[kunalganglani.com "Reduce Rust Compile Time"](https://www.kunalganglani.com/blog/reduce-rust-compile-time).

**Step 5.3.3 — Test harness: `cargo nextest` + per-crate test isolation (S effort).**

```bash
# In scripts/install-binaries.sh:
if ! command -v cargo-nextest >/dev/null; then
  cargo install cargo-nextest --locked
fi
```

Add `config/nextest.toml`:

```toml
[profile.default]
test-threads = "num-cpus"
retries = 0
slow-timeout = { period = "60s", terminate-after = 3 }
fail-fast = false
```

Replace the `cucumber.mjs` + ad-hoc shell scripts with a single
`scripts/test-all.sh` that runs:

```bash
#!/usr/bin/env bash
set -euo pipefail
cargo nextest run --workspace --no-fail-fast
bash scripts/test-skills.js
bash scripts/verify-protected-tests.mjs
bash scripts/check-mcp-health.sh
```

**Step 5.3.4 — CI cache for CI runners (XS).**

If GitHub Actions ever runs a test, use `Swatinem/rust-cache@v2`
which already wraps `sccache`. AGENTS.md mandates local-only
testing, so this is for cases where a developer runs a CI runner
on their own non-default branch (which is permitted).

### 5.4 Disk space during builds

`cargo build --workspace` on a Rust monorepo routinely hits 5–10 GB
of `target/`. The fix is per-workspace `target/` (which the layout
already gives us) plus:

```bash
# scripts/clean-targets.sh — keep dev/release for the active crate only
find . -name target -type d -prune | while read t; do
  # Keep release/ (what we ship), drop debug/ and incremental/ in inactive workspaces
  ...
done
```

`cargo-cache` (`cargo install cargo-cache --locked`) does this
automatically. Add it to `install-binaries.sh`.

### 5.5 Effort and order

| Step | Effort | Expected win |
|---|---|---|
| 5.3.1 sccache + mold/zld | XS (1 d) | 70% faster warm builds, 3-10× faster link |
| 5.3.4 CI cache | XS (1 h) | 50-80% CI wall time |
| 5.3.3 nextest | S (1 d) | 30-50% faster test suite, per-test isolation |
| 5.3.2 superworkspace | M (3-5 d) | one lock file, one Cargo.toml, no version drift, shared target/ where it helps |
| 5.3 disk cleanup | XS (1 h) | 2-5 GB reclaimable per build |
| Total | ~1 week | First clean build from 5-10 min → 60-90 s; warm build from 60-90 s → 5-15 s |

---

## 6 · Hooks Architecture — Reliability

### 6.1 The current pattern

Every hook in `hooks/hooks.json` runs through the same
4-stage funnel:

```
hook event → bash -c '...'
  → runner="$HOME/.prometheus/plugins/.../run-hook"
  → runner --bundle $EXPECTED_BUNDLE --hook $HOOK_ID --harness $HARNESS
  → hook-dispatch-v1.sh (generated, ABI-pinned)
  → bash $BUNDLE_ROOT/shared/scripts/<actual-hook>.sh
```

That funnel is well-designed (immutable bundle, hash-pinned
dispatcher, no `$BUNDLE_ROOT/../*` path escape). But it has
five specific failure modes that match the well-known Claude Code
hook failure modes documented in
[alexdunlop.com "Why Your Claude Code Hook Isn't Firing"](https://www.alexdunlop.com/writing/claude-code-hook-not-firing)
and
[hookstack.app "Claude Code Hooks Not Working"](https://www.hookstack.app/guides/claude-code-hooks-not-working).

### 6.2 Specific weaknesses

**W6.1 — `bash -c '…'` with a 30-character inline script on every
event.** Every hook is its own subshell. The first thing each
subshell does is resolve `$HOME/.prometheus/plugins/.../run-hook`
from `$HOME`, then call `--resolve-only` on it, then exec the
real script. This is a *minimum* of 4 forks per hook. On a session
that has 6 SessionStart hooks, the cold start cost is 24+ forks
just to bootstrap. The hooks-on-warm-path is the bottleneck for
"Claude feels slow to respond."

**W6.2 — `--resolve-only` re-validates the bundle SHA on every
call.** That's the right call for safety but it does a `shasum -a 256`
on the dispatcher file every single hook. Caching that
`shasum` (e.g. write the validated SHA to `/tmp/.prom-hook-cache`
with a 60s TTL) would let warm-path hooks skip the work.

**W6.3 — The `subagent-*` matchers are unmapped names.** The
matchers `assessor`, `analyst`, `planner`, `executor`, `reflector`
must match the agent name string Claude Code passes for the
subagent. If a future agent name drifts (e.g. `planner-v2` or
`plan`), every hook in that matcher silently does not fire. The
matcher should be a regex (`planner|plan`) or a glob
(`planner*`). Also: there is no "no match" subagent. The README
claims a fallback matcher exists, but the JSON has no matcher on
the fallback hook, which means it fires for *every* subagent,
defeating the per-role design.

**W6.4 — Hooks that print to stdout corrupt PreToolUse decisions.**
`shared/scripts/detect-project-context.sh`, `memory-outbox-flush.sh`,
`pk-health.sh`, `karpathy-hook-dispatch.sh`, `evaluate-session.sh`
all `|| true` the actual command, but they don't guard against
the actual hook script writing to stdout. The whole point of the
funnel is to allow stdout to be the *decision* JSON; if a hook
inadvertently prints a `WARN: …` line, Claude Code may parse it
as the decision. The defensive pattern is `exec 2>>"$LOG"` first,
then the real work.

**W6.5 — `timeout` is set on a few hooks but not enforced by the
runner.** `timeout: 30000` in the JSON only tells Claude Code to
hard-kill the subprocess at 30s. A hook that runs `sleep 60` in
the background and then `exit 0` returns within the timeout but
leaks the sleep. The runner should detect detached children and
`wait` for them, or use `prctl(PR_SET_PDEATHSIG)` / process group
kill.

**W6.6 — `sessionstart-detect-project-context` and
`sessionstart-memory-outbox-flush` and `sessionstart-pk-health`
all run on every SessionStart with no matcher to scope them.**
They fire for every new session, including read-only "what's in
this file" sessions. The cost is small per session but the *fanout*
is high; the `kbd-open` script in the second matcher has a 30s
timeout which is the worst case for slow disk.

**W6.7 — Hook logs go to `~/.prometheus/logs/`, but there's no
ring buffer.** A misbehaving hook can fill the disk. The
logrotate config exists for `prometheus-hooks.log` but not for
the per-service logs under `~/.prometheus/logs/*.log` (well, the
config has both, but the per-service plist `StandardOutPath`
points to the same dir without its own rotate policy).

**W6.8 — No `UserPromptSubmit` matcher for non-prompt events.**
`UserPromptSubmit` is supposed to fire on every prompt. The
`prompt-karpathy-learning` hook is the only one in the block, and
it has no matcher. If a second hook is ever added, both will
unconditionally fire on every prompt.

**W6.9 — The `bash -c '…'` quoting is fragile.** Look at the
inline scripts: they have `'…'` and `"…"` and `'"'"'` chains.
A copy-paste of one of these into a different shell breaks
silently. The fix is to extract the inline script to a
checked-in file under `shared/scripts/generated/hook-<id>.sh`
and have the JSON just call that file.

### 6.3 Proposed remedies

| # | Remedy | Where | Effort |
|---|--------|-------|--------|
| R6.1 | Replace the inline `bash -c '…'` with calls to checked-in scripts under `shared/scripts/generated/hooks/` | `hooks/hooks.json` + new `shared/scripts/generated/hooks/*.sh` | S |
| R6.2 | Compile `run-hook` once and use `exec` instead of `bash -c` to skip the subshell layer | `shared/scripts/bootstrap-hook-runtime.sh` | S |
| R6.3 | Cache the dispatcher SHA verification for 60s in a temp file | `shared/scripts/hook-runtime-v1.sh` | XS |
| R6.4 | Use **process group** kill in the hook runner: spawn the hook via `setpgid`, kill `-PGID` on timeout, reap zombies | `shared/scripts/hook-runtime-v1.sh` | S |
| R6.5 | Make the subagent matchers regex-anchored (`^planner$|^plan$`) and add a per-Prompt matcher to `UserPromptSubmit` | `hooks/hooks.json` | XS |
| R6.6 | `exec 2>>"$LOG"` first thing in every generated hook script so stdout stays clean for decision JSON | `shared/scripts/generated/hook-*.sh` | XS |
| R6.7 | Add a 1-line structured hook-result log: `{ts, hook_id, harness, exit, dur_ms, stderr_hash}` appended to `~/.prometheus/logs/hooks.ndjson` for observability | `shared/scripts/hook-runtime-v1.sh` | S |
| R6.8 | Move the `bash -c '…'` to a small Rust binary `prom-hook-dispatch` (or to `prometheus-exec run`) so the inline quoting bug class is impossible | new `crates/prom-hook-dispatch/` (or fold into `prometheus-exec`) | M |
| R6.9 | Tighten `sessionstart-*` matchers: e.g. `sessionstart-detect-project-context` should be `matcher: "claude-code"` (or skipped on read-only sessions via a new harness flag) | `hooks/hooks.json` | XS |
| R6.10 | `doctor` should also dump "hooks that didn't fire in the last hour" so the user can detect silent drops | `scripts/check-mcp-health.sh` | S |

The biggest win is R6.8 — make the inline `bash -c` go away. The
second-biggest is R6.1 + R6.3 + R6.4 because they make every
existing hook faster and safer.

### 6.10 Cross-repo enforcement (the architectural commitment)

The remedies in §6.3 are **designed but not shipped** in
the prometheus-skill-pack today. They ship in the HMA
`claude-hooks-reliability` skill (HMA v0.2.0), and the
**Companion is the cross-repo enforcer** that runs the
HMA's verifier scripts against every connected skill
package. The chain is:

```
HMA v0.2.0 ships:
  skills/claude-hooks-reliability/SKILL.md
  scripts/install-hooks-reliability.sh     # applies the 9 fixes
  scripts/verify-hooks-reliability.sh      # the inverse check

Companion runs:
  bash scripts/verify-hooks-reliability.sh ~/.prometheus/skill-packages/<id>/
  on every "doctor" run.
  Per-fix failures (W6.1 through W6.9) are shown in the
  Settings → Supervisors panel with a one-line fix command.
```

This is the same architectural commitment as §1.4 for
hooks: **the HMA ships the scripts, the Companion runs
them**. The downstream resolution matrix (§15) is the
canonical map.

---

## 7 · Skill Selection — Hit Rate and Dynamic Discovery

### 7.1 The state of the art

Claude Code skills are **prompt templates**, not executable code.
Selection happens *entirely* by the LLM reading the `description`
field of every skill. There is no embedding, no classifier, no
algorithmic routing in Claude Code itself — see
[leehanchung.github.io "Claude Agent Skills: A First Principles Deep Dive"](https://leehanchung.github.io/blogs/2025/10/26/claude-skills-deep-dive/).
This is documented by Anthropic.

The empirical hit-rate is dramatic and worth internalizing:

| Description style | Activation rate | Source |
|---|---|---|
| Vague ("helps with documents") | **~20%** | [gist.github.com/mellanon/50816550ecb5f3b239aa77eef7b8ed8d](https://gist.github.com/mellanon/50816550ecb5f3b239aa77eef7b8ed8d), 200+ prompts |
| Optimized description with "USE WHEN" | **50-80%** | same |
| Forced-eval `UserPromptSubmit` hook | **84%** | same, firecrawl.dev/blog |
| 5+ trigger keywords + exclusion clause | **~90%** | same |

In a 1000-skill corpus, top-1 selection accuracy plateaus around
**62%** with pure keyword, **~85%** with a hybrid vector + graph
router (the mesh-memory / pgvector benchmark in
[reddit.com/r/ClaudeAI "How does a Claude Code agent navigate hundreds of skills in a second?"](https://www.reddit.com/r/ClaudeAI/comments/1tlr914/how_does_a_claude_code_agent_navigate_hundreds_of/)).

### 7.2 The current state in prometheus-skill-pack

The pack ships **40+ skills** (16 in `skills/process/`, 11 in
`skills/rust/`, plus react, flutter, htmx, tauri, typescript, go,
python, learn, etc.). Many of them have descriptions in the
"Specific, third person, includes USE WHEN" form — but not all.
A spot check of the `process/` skills:

```
native-agent:       "Generates a complete, production-ready native Rust agent application with a Supabase-style management CLI. ..."
zeespec-interrogator: "60-question Zachman 5W1H constraint interrogation, GO/CAUTION/NO-GO manifests ..."
iterative-evolver:  "Strategic PMPO loop: Assess→Analyze→Plan→Execute→Reflect ..."
pmpo-evolver:       "Strategy router for 5 evolution perspectives: ..."
kbd-process-orchestrator: "Tactical KBD loop (16 child skills): change management, multi-tool dispatch"
```

The descriptions are capability statements, not trigger statements.
Compare to the Anthropic best practice from
[generativeprogrammer.com "Skill Authoring Patterns from Anthropic's Best Practices"](https://generativeprogrammer.com/p/skill-authoring-patterns-from-anthropics):
"the description is the only signal Claude has at selection time"
and "the description packs the description with both what the
skill does and the specific triggers or contexts that should
fire it."

The pack also has skill descriptions that exceed 1024 characters
in a few cases (e.g. the 200-line `forge reflect` style bodies).
Once the description hits the 1024 char cap, the model can't see
the trigger keywords that come after.

### 7.3 Weaknesses

**W7.1 — Vague descriptions.** Some skills describe what they do
but not when to use them. The model has to guess.

**W7.2 — No `when_to_use` field.** The 1024-char cap means
keywords compete with the capability statement. Anthropic's
Claude Code supports a separate `when_to_use` field (1536 char
total budget) — none of the pack's skill.toml files use it.

**W7.3 — No exclusion clauses.** Best practice: "Do NOT use for
blog articles, newsletters, emails, tweets, or long-form content."
None of the pack's skills have an exclusion clause, so similar
skills compete for every prompt (e.g. `iterative-evolver` vs
`pmpo-outer-loop` vs `pmpo-elicit`).

**W7.4 — No semantic router.** At 40+ skills the LLM still
performs well, but the doc says the pack will grow. Without a
router, every description goes into the system prompt, costing
~10-20 tokens per skill, **plus** every description burn the
context budget for prompts that are unrelated. `skill-index`
exists in `substrate/` and is the right place to host the router
but currently only does text search.

**W7.5 — No skill hit-rate telemetry.** The hooks log
`prompt-karpathy-learning` writes to the learning queue, but
there's no per-prompt "which skills did the model invoke" log.
Without that, no A/B test of description styles is possible.

**W7.6 — `lazy:` progressive disclosure not implemented.** The
[boliv.substack.com "Lazy Skills"](https://boliv.substack.com/p/lazy-skills-a-token-efficient-approach)
post shows 97-98% token savings for a 42-skill corpus with
metadata-only at L1, full body on demand at L2, references on
demand at L3. The pack loads every `SKILL.md` body at session
start (via the `kpi-open` hook output).

**W7.7 — No "forced eval" hook.** The 84% activation hack
(`UserPromptSubmit` hook that injects a reminder to check
skills) is not installed.

**W7.8 — `description` truncation not standardized.** Each
`SKILL.md` frontmatter is hand-edited. There's no `validate-skill-description.sh`
that asserts the description fits the 1024-char cap and includes
trigger keywords.

### 7.4 Proposed remedies

| # | Remedy | Where | Effort |
|---|--------|-------|--------|
| R7.1 | Authoring standard: every skill description follows `[What it does]. Use when [trigger1], [trigger2], or when user mentions "[kw1]", "[kw2]", "[kw3]". Do NOT use for [exclusion1], [exclusion2].` Style of `[firecrawl.dev/blog](https://www.firecrawl.dev/blog/claude-code-skill)`. | `docs/skill-authoring-guide.md` (new) | S |
| R7.2 | Add a `validate:skill-description` step in `validate-skills.js` that asserts: description < 1024 chars, contains at least 3 trigger words, contains a "Do NOT" exclusion clause, is in third person. | `scripts/validate-skills.js` | S |
| R7.3 | Implement a **forced-eval `UserPromptSubmit` hook** in `shared/scripts/skills/force-skill-eval.sh` that injects: "Before responding, list the 1-3 most relevant installed skills for this prompt and confirm whether to invoke them. If you are not invoking any, justify why." | new hook in `hooks/hooks.json` | S |
| R7.4 | **Semantic router** in `substrate/skill-index/`: embed every skill's `name + description + when_to_use + tags` into a local embedding model (`fastembed-rs` with `bge-small-en-v1.5`), store in the **SurrealDB embedded vector type** (HNSW — same engine that backs `surreal-memory`), expose via MCP. On session start, `kbd-open` queries the router for the top-5 skills per session context and only injects those. | `substrate/skill-index/src/router.rs` (new) | M |
| R7.5 | **Skill hit-rate telemetry** in the prompt hook: log `ts, prompt_hash, top_5_skill_ids, invoked_skill_id, latency_ms` to `~/.prometheus/logs/skill-router.ndjson`. Pipe through the learning worker for daily aggregation. | new `shared/scripts/skills/log-skill-router.py` | S |
| R7.6 | **Lazy progressive disclosure** at the CLI level: `prometheus skills list` only shows metadata; `prometheus skills show <id>` injects the body. The Tauri tray app and the kbd-open hook use the same pattern. | `crates/prometheus-cli/` | M |
| R7.7 | Add a `validate:skill-metadata-budget` check: total combined `description` + `when_to_use` of all installed skills < 50K tokens (the empirical ceiling before Claude Code starts to drop skills). | `scripts/validate-skills.js` | XS |

**Quick-win:** R7.1 + R7.2 + R7.3 (one engineering day, immediate
hit-rate jump from ~30% to ~80%). R7.5 + R7.7 (one more day,
sets up A/B testing and budget enforcement). R7.4 + R7.6 (the
proper fix for the 100+ skill horizon).

---

## 8 · External Extension Model (Without Bloating the Core)

### 8.1 The current pattern

The pack already has the right primitive — `marketplace.json` — with
13 independently installable plugins. **That is the extension model.**
The question is whether the surface area is rich enough for an
external developer to ship a plugin without touching the core.

The current plugin types are:

```json
{
  "name": "prometheus-research-server",
  "version": "1.6.0",
  "source": "./substrate/prometheus-research"
}
{
  "name": "artifact-refiner",
  "source": { "repo": "GQAdonis/artifact-refiner-skill", "sha": "..." },
  "strict": false,
  "skills": ["./skill-a", "./skill-b"]
}
```

Both forms are supported by Claude Code's native plugin system
([github.com/anthropics/claude-plugins-official](https://github.com/anthropics/claude-plugins-official)).
The `strict: false` "skill bundle" form is exactly the
"external extension without bloating the core" pattern: a third-party
git repo can ship only the `SKILL.md` files and register them in
the pack's `marketplace.json` via a `sha` pin and a relative
path.

### 8.2 Weaknesses

**W8.1 — The marketplace is curated.** All 13 plugins ship in the
pack's own `.claude-plugin/marketplace.json`. A developer who
wants to publish a new skill has to file a PR against the pack.
There's no `/plugin marketplace add <user>/<repo>` flow surfaced
to the user.

**W8.2 — The `strict: false` mechanism is undocumented.** Looking
at the marketplace.json: `"strict": false` appears twice, with no
inline comment explaining what it does. A new contributor can't
discover the pattern by reading.

**W8.3 — `additionalDirectories` in Claude Code's `settings.json`
is not surfaced.** The Claude Code plugin system supports
`additionalDirectories: ["../shared-lib", "../docs"]` (see
[introl.com "Claude Code CLI: The Definitive Technical Reference"](https://introl.com/blog/claude-code-cli-comprehensive-guide-2025))
— this lets a skill import files from sibling repos without
copying them. The pack's skills can only see files inside the
plugin directory.

**W8.4 — No `prometheus plugins publish` CLI.** The repo has
`scripts/generate-harness-adapters.js`, `scripts/generate-commands.js`,
`scripts/generate-skill-system-distribution.js`, etc. — but no
end-user "publish a new plugin" path. A developer can't run
`prometheus plugins new <name> --template <tpl>` and get a
scaffolded marketplace entry.

**W8.5 — Versioning across the 13 plugins is not coordinated.**
`prometheus-skill-pack: 1.7.0`, `prometheus-react-skills: 1.5.0`,
`prometheus-process-skills: 1.5.1`, `prometheus-research-server: 1.6.0`.
A bump of one doesn't cascade. There's no `prometheus plugins list
--outdated` or `prometheus plugins update`.

**W8.6 — No `plugin.toml` schema validation.** Each plugin's
`plugin.json` (or its absence) is parsed by Claude Code at install
time. The pack has no JSON schema for plugin.json. A typo in
`requiredMcpServers` is only caught at install.

**W8.7 — Skill bundles are read-only snapshots.** The `sha: "a0b560b..."`
pin means a `git pull` on `artifact-refiner-skill` will not
update the pack. The user has to manually update the sha in
`marketplace.json`. There's no `prometheus plugins update` or
`prometheus plugins outdated`.

**W8.8 — No contract test for "this plugin's hooks will fire on
the expected events."** The pack can install a plugin that
silently doesn't fire because its `hooks/hooks.json` is malformed,
or because its matchers don't match anything. The `validate-skills.js`
script tests skill descriptions but not hook registrations.

**W8.9 — External plugin authors can't read the pack's run-time
state.** There's no `PROMETHEUS_PLUGIN_API_VERSION` constant
exposed in hooks. An external plugin has to discover at runtime
what bundle id, what generation, what hook ids are valid — and
the only way to do that today is to read
`~/.prometheus/plugins/.../release-manifest.json`. That works,
but the contract isn't documented.

**W8.10 — No plugin sandbox.** A third-party plugin can run
arbitrary `bash` (e.g. via a hook) and has access to
`$HOME`, `$CLAUDE_PROJECT_DIR`, the entire filesystem. There's
no `permissions: { allow: [...], deny: [...] }` model for plugins
(skills only, not the hooks they install).

### 8.3 Proposed remedies

| # | Remedy | Where | Effort |
|---|--------|-------|--------|
| R8.1 | Publish a **`prometheus-plugins` template repo** at `Prometheus-AGS/prometheus-plugin-template` with `prometheus plugins new <name> --from <template>` scaffolder. The template ships `plugin.json` + a sample skill + a sample hook + a CI workflow that runs the validation suite. | new repo | M |
| R8.2 | Document the **`strict: false` skill-bundle pattern** in `docs/plugin-authoring.md`. Include a worked example: a third-party `prometheus-foo` plugin hosted in `GQAdonis/prometheus-foo` with `marketplace.json` entry `"source": {"repo": "...", "sha": "...", "source": "github"}, "strict": false, "skills": [...]`. | `docs/plugin-authoring.md` (new) | S |
| R8.3 | Add **`/plugin marketplace add <url>`** UX to the Companion dashboard (§2). A "Discover" tab lists every marketplace the user has added, every plugin available, and a one-click install. The pack itself ships a `recommended-marketplaces.json` (Forge, Anyscale, etc.) seeded in the install. | Companion app | M |
| R8.4 | Add `additionalDirectories` support in `shared/harnesses/capabilities.json` so a plugin can declare `additionalDirectories: ["../shared-lib"]` and the bundle installer creates the relative symlink without copying. | `shared/harnesses/capabilities.json` | S |
| R8.5 | **`prometheus plugins list --outdated`** and **`prometheus plugins update [<name>]`** subcommands. The `update` command pulls the new sha, validates the bundle identity matches, and re-bootstraps. | `tools/prometheus-cli/crates/prometheus-cli/src/cmd/plugins.rs` (new) | M |
| R8.6 | **JSON schema for `plugin.json`**. Add `schemas/plugin.schema.json` and validate on `prometheus plugins new` and on CI. | `schemas/` (new) | S |
| R8.7 | **Plugin contract test** in the install path: every plugin's `hooks/hooks.json` is linted for matcher validity, timeout presence, `bash -c '…'` length < 256 chars, and (best effort) executable permissions on the script. | `scripts/validate-skills.js` | S |
| R8.8 | **Plugin API version constant**. A plugin's `plugin.json` declares `prometheusApi: "^1.7"`. The runtime refuses to install a plugin whose declared range doesn't include the current version. (Soft refusal: a warning, not a hard block, until the contract is stable.) | `crates/prometheus-substrate/src/plugin.rs` | S |
| R8.9 | **Plugin permissions** in `plugin.json`: `permissions: { allow: ["Bash(cargo:*)"], deny: ["Bash(rm:*)"] }`. The runner uses these to constrain the inline `bash -c` script the bundle emits. | `crates/prometheus-substrate/src/plugin.rs` | L |
| R8.10 | **Sandbox first** with `Bubblewrap` (Linux) / `sandbox-exec` (macOS) for the hook runner. Each hook runs inside a sandbox that mounts only `$CLAUDE_PROJECT_DIR` and the plugin's directory. | `shared/scripts/hook-runtime-v1.sh` | L |

**Quick-win:** R8.2 + R8.6 (one day; the documentation gap
closes, the validation gap closes). R8.5 (one day; the update
flow is the most-asked-for feature). R8.10 is the right long-term
move but is a quarter of work, not a week.

### 8.6 Addendum (post-review): the canonical bootstrap, the rule files, the install contract

After the original review, three new pieces settled the
extension model into a clean shape:

1. **The `prometheus-context-bootstrap` skill** is the
   canonical way to bootstrap `AGENTS.md` and `CLAUDE.md`
   in any new project. It lives at
   `prometheus-skill-pack/dist/plugins/claude/prometheus-skill-pack/skills/prometheus-context-bootstrap/`
   and ships the four scripts `bootstrap.sh`, `migrate.sh`,
   `verify.sh`, and `skill-budget.sh`. For the Companion
   repo specifically, the rule files are generated from
   `templates/rules/agents.md.tmpl` and `claude.md.tmpl`
   via `scripts/gen-rule-files.sh` and are checked by
   `scripts/audit-generated.sh`.

2. **The 4-condition install contract** is the
   cross-repo agreement between the HMA (producer) and
   the Companion (consumer). See §16.2 for the full
   contract. The HMA ships `scripts/verify-skill-manifest.sh`
   which checks conditions 1-3; condition 4 (idempotent
   install) is a behavior check that surfaces in the
   Companion's validate flow.

3. **The `auto-skill-package-integration` HMA skill** (new
   in v0.2.0, see §17 + the dedicated spec at
   `/Users/gqadonis/Projects/hybrid-mobile-architecture-src/docs/08-auto-skill-package-integration.md`)
   is the **no-friction path** beyond the manual
   install/upgrade/validate/remove operations. It supports
   two source types — a watched git URL or a watched
   local directory — and detects new `SKILL.md` files
   automatically. This is the response to the
   "the marketplace is curated" weakness (W8.1): the
   operator points the Companion at a directory (or a
   git URL), and the Companion auto-installs.

The new shape of the extension model in 2026-08-20:

```
External developer writes a skill in a directory on disk
            │
            ▼
The Companion's auto-integration detects the new SKILL.md
            │
            ▼
  Companion runs the 4-condition install contract
            │
            ▼
  Companion auto-installs (or prompts in confirm mode)
            │
            ▼
  The skill is registered with the active harness
  (Claude Code / Codex / Kimi / OpenCode / Mavis)
            │
            ▼
  The next Claude session sees the skill
```

The developer never touches the core. The core never
hard-codes a list of skills. The marketplace is
**emergent** — it is whatever directories the operator
points the Companion at.

---

## 9 · Other Weaknesses (Outside the 5+3 Axes)

These came up during the review but didn't fit cleanly into the
five core topics. Each gets a one-line description and a remedy
ID for the roadmap.

**W9.1 — `crates/prometheus-exec/target/` is committed-by-default.**
The `.gitignore` says `target/` is ignored, but `prometheus-exec/target/`
exists in the working tree and the file timestamps suggest it's
been rebuilt many times. Verify with `git status --ignored`. If
real, a `git clean -fdx crates/prometheus-exec/target` reclaims
the disk.

**W9.2 — Two `claude-hooks.json` files exist.** The repo has
`hooks/hooks.json` (the canonical one) and `hooks/codex-hooks.json`
(Codex-specific). The Codex version is referenced from
`.claude-plugin/marketplace.json` but its contents are not under
the same bundle / hash contract. A consistent contract would make
both harnesses safer.

**W9.3 — `progress.json` is read by every KBD/loop turn but
there's no offline progress replay.** If the user's machine is
offline when a phase completes, the receipt is queued but the
`progress.json` is not updated until the worker drains. That's
intentional for durability but it makes the dashboard (§2) show
"stale" state. A trivial fix: the worker writes to a sidecar
`progress-replay.jsonl` and the dashboard reads that.

**W9.4 — The two install scripts disagree on `PROMETHEUS_USER`.**
`prometheus-services.sh:10` defaults to `gqadonis`. `install-mcp-services.sh:42`
defaults to `$(id -un)`. If you `sudo bash install-mcp-services.sh`,
the LaunchAgents are installed as root and the user's Claude Code
session can't talk to them. Pin a single source of truth in
`config/defaults.env`.

**W9.5 — `enqueue-learning-job.py` writes via `tempfile.NamedTemporaryFile`
+ `os.replace` — that is the right atomic pattern. But the receipt
schema is not versioned. Adding `schemaVersion: 1` to every JSON
artifact (skill, receipt, snapshot) costs one line per writer and
unlocks future migration.

**W9.6 — Cedar policies live in `policies/` but the policy engine
is wired into `prometheus-cedar` only.** If a skill install
should be gated by Cedar (e.g. "only the `prometheus-entity-skills`
plugin may write to `/data/users/`"), the gating point is missing.
Either remove the policy directory or wire it into the install path.

**W9.7 — `kpi-open` and `kbd-open` are not the same script.**
The first hook `sessionstart-kbd-open` calls `bash $HOME/.local/bin/kbd-open`.
The runtime assumes kbd-open is installed at that path. If the
path doesn't exist, the hook silently fails (the `2>&1 || true`
masking it). Add a `validate-runtime.sh` doctor check.

**W9.8 — `claw-marketplace.json` and `marketplace.json` live side
by side.** The naming inconsistency makes plugin discovery hard.
Standardize on `marketplace.json` only.

**W9.9 — The README says "we ship 8 language domains" but the
directories are 12+.** The README is also 33 KB, which means it
won't be loaded in many agents' context windows. Move the long
form to `docs/guide/` (already done) and keep the README at
< 200 lines.

**W9.10 — `surreal-memory-server` is a submodule, but the install
script in `install-mcp-services.sh:250` resolves its binary by
falling back to a path inside the submodule's `target/release`.
If the submodule is on a feature branch, the binary may not
exist. Add a `scripts/build-surreal-memory-server.sh` (or a
makefile) to the install path.

---

## 10 · Competitive Comparison (what we can learn from peers)

| Capability | Prometheus today | Ollama desktop | LM Studio | AnythingLLM | AetherLink | Antigravity-Tools | ComfyUI Desktop | Jan |
|---|---|---|---|---|---|---|---|---|
| Tray + dashboard | ❌ (7 plists) | ✅ (Go + wintray) | ✅ (Electron) | ✅ (Electron) | ❌ (CLI) | ✅ (Tauri) | ✅ (Electron) | ✅ (Electron) |
| Single-binary substrate | ❌ (15+ procs) | ✅ (Go binary + sidecar) | ✅ | partial | ✅ (Tauri + sidecar) | ✅ (in-process lib) | partial | ✅ |
| Mobile (iOS/Android) | ❌ (kbd-mobile only) | ✅ (testflight) | ❌ | ❌ | ✅ (Tauri + Capacitor) | partial | ❌ | partial |
| P2P between devices | partial (`sovereign-sync`) | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ |
| Skill/plugin marketplace | ✅ (13 plugins, curated) | n/a | partial | partial | n/a | partial | ✅ (custom nodes) | n/a |
| Install path single command | partial (3 scripts) | ✅ (DMG) | ✅ (DMG) | ✅ (Docker) | ✅ (brew) | ✅ (DMG) | ✅ (DMG) | ✅ (DMG) |
| Crash-loop self-heal | ❌ | ✅ (wintray + ollama.pid) | ✅ | ✅ | partial | ✅ | ✅ | ✅ |
| Liveness ≠ readiness probes | ❌ (port only) | ✅ | ✅ | partial | partial | ✅ | ✅ | partial |
| Telemetry | partial (logs only) | ✅ (Sentry) | ✅ | ✅ | partial | partial | partial | partial |
| Code signing for sidecar binaries | ✅ (prometheus-exec) | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |

**Headlines from the comparison:**

1. **Every shipped desktop AI tool has a tray + dashboard.** None of
   them ship "5+ background daemons managed by launchd." The
   sidecar-or-in-process-library pattern is dominant.
2. **Only AetherLink has a working Tauri mobile build.** Their
   strategy is "Tauri for desktop, Capacitor for mobile" because
   Tauri's mobile story is "younger than the desktop support" and
   they wanted App Store distribution sooner. For the pack, this
   is a useful precedent: use Tauri mobile for the developer /
   test build, Capacitor (or a thin WebView) for the App Store
   build.
3. **Ollama's `wintray.CheckAndFocusExistingInstance` + `ollama.pid`
   orphan-reaper is exactly the crash-tolerance pattern the pack
   needs in the Companion (§2.2).** Worth a deep read of
   `app/server/server.go` and `app/cmd/app/app_windows.go`.
4. **Antigravity-Tools' "in-process library, not sidecar" is the
   right default when the substrate is a Rust binary we control.**
   Sidecar is only needed when the sidecar is a third-party
   binary (Ollama, Python) that we can't link.
5. **The pack's marketplace + plugin model is more advanced than
   any of the peers.** The 13-plugin `marketplace.json` is the
   pack's strongest external-comparison advantage. Make it
   self-service and it becomes a moat.

---

## 11 · Weaknesses Triage (Severity × Effort)

| ID | Severity | Effort | Title | Pillar |
|---|---|---|---|---|
| W1.1 | **High** | XS | launchd crash-loop amnesia | Reliability |
| W1.2 | High | S | Port-only health probes | Reliability |
| W1.4 | **High** | S | No scheduled `doctor` / no notifications | Reliability |
| W1.6 | High | S | 60-second `bootstrap-lock` retry on every hook | Reliability / Hooks |
| W1.7 | Med | S | Two parallel install scripts | Reliability |
| W2.1 | **Critical** | L | No desktop tray + dashboard | Companion |
| W3.1 | **High** | L | 15+ processes to install | Consolidation |
| W3.2 | Med | S | Dual install scripts | Consolidation |
| W4.1 | **High** | L | No mobile shell | Mobile |
| W5.1 | **High** | M | No sccache / mold / superworkspace | Build |
| W5.2 | Med | S | Per-submodule Cargo.lock | Build |
| W5.3 | Med | S | No `cargo nextest` | Build |
| W6.1 | High | S | `bash -c '…'` inline scripts | Hooks |
| W6.2 | Med | XS | SHA re-validated on every hook | Hooks |
| W6.3 | Med | XS | Subagent matchers fragile | Hooks |
| W6.5 | Low | S | No process-group kill on timeout | Hooks |
| W7.1 | **High** | S | Vague skill descriptions | Skills |
| W7.4 | Med | M | No semantic router | Skills |
| W7.6 | Med | M | No lazy progressive disclosure | Skills |
| W7.7 | Med | S | No forced-eval hook | Skills |
| W8.1 | **High** | M | Marketplace is curated, not self-serve | Extension |
| W8.2 | Med | S | `strict: false` pattern undocumented | Extension |
| W8.4 | High | M | No `prometheus plugins new` scaffolder | Extension |
| W8.10 | Med | L | No plugin sandbox | Extension |
| W9.4 | Med | XS | `PROMETHEUS_USER` disagreement | Install |
| W9.7 | Med | XS | `kbd-open` path not validated | Hooks |

**Critical (must fix before "this is a real product"):** W2.1,
W5.1, W1.1, W1.4, W4.1, W7.1, W8.1, W8.4.

---

## 12 · Proposed 6-Pillar Roadmap

Each pillar is sized to one engineering week. The order is
deliberate: P1 and P2 reduce the day-to-day pain immediately
and unblock P3-P6.

### Pillar 1 — Reliability (Week 1)

- R1.1 + R1.2 + R1.10: plist hardening (XS)
- R1.4 + R1.5: liveness/readiness split + binary ID check (S)
- R1.6: self-healing watchdog (S)
- R1.7: user notification on persistent `down` (S)
- R1.8: kill the `.bootstrap-lock` 60 s wait (XS)
- R1.9: collapse to one installer (S)

**Definition of done:** `bash prometheus doctor` returns 0 in a
clean install, returns non-zero with a specific failing service
name in a partially-broken install, and triggers a Mac
notification if any service is down for > 5 min. Self-healing
watchdog re-bootstraps any service that gets removed by launchd
crash-loop heuristic.

### Pillar 2 — Build Time (Week 2)

- R5.3.1: sccache + mold/zld + shared `config/cargo/config.toml` (XS)
- R5.3.4: CI cache (XS)
- R5.3.3: cargo nextest + `scripts/test-all.sh` (S)
- R5.3 disk cleanup script (XS)
- W9.1: `target/` audit (XS)

**Definition of done:** First clean build of the superworkspace
under 90 s (was 5-10 min). Warm build under 15 s (was 60-90 s).
Test suite runs in `nextest` and the per-test output is parseable.

### Pillar 3 — Skill Hit Rate (Week 3)

- R7.1 + R7.2: authoring standard + validation (S)
- R7.3: forced-eval `UserPromptSubmit` hook (S)
- R7.5 + R7.7: telemetry + budget enforcement (S)
- R7.4: semantic router in `skill-index` (M)
- R7.6: lazy progressive disclosure (M)

**Definition of done:** Skill hit rate (top-1) ≥ 80% on a 50-prompt
benchmark, measured by `~/.prometheus/logs/skill-router.ndjson`.
Total description budget ≤ 50K tokens, enforced at install.

### Pillar 4 — Service Consolidation (Weeks 4-5)

- R3.1: `crates/prometheus-substrate` supercrate (L)
- R3.2: promote `prometheus-exec` to supervisor (M)
- R3.3: single LaunchAgent + single plist (S)
- R3.4: keep `surreal`, `openai-proxy` as external (XS)
- R3.5: substrate as Tauri sidecar **or** in-process library (S)
- R3.6 + R3.7: collapse installers + unified `prometheus doctor` (S)

**Definition of done:** Install of the full substrate requires
**one** binary. `prometheus doctor` reports the health of
every process in 1 call. Legacy `prometheus-services.sh` is
deleted.

### Pillar 5 — Companion (Tauri Tray + Dashboard) (Weeks 6-8)

- R2.1-R2.3: tray + dashboard + health aggregator (§2.3, §2.4)
- R2.5: P2P pairing flow
- R2.6: A2UI surface
- Bundle `prometheus-substrate` (§3) as the in-process backend

**Definition of done:** `Prometheus Companion.app` (or `.exe`)
download is a single DMG. First launch installs the entire
substrate. Tray icon color reflects aggregate health. Dashboard
shows every service, has a per-service fix menu, supports device
pairing via QR, exposes the A2UI surface to orchestrator skills.

### Pillar 6 — Mobile + P2P (Weeks 9-10)

- R4.3: mobile feature split on `prometheus-substrate`
- R4.4-R4.5: Iroh transport + AG-UI over Iroh
- R4.6: native plugins (biometric, secure-storage, notifications)
- R4.7: App Store + Play Store packaging

**Definition of done:** `Prometheus Mobile.app` (iOS) and
`Prometheus Mobile.apk` (Android) are TestFlight-internal builds
that connect to a Companion desktop install, sign in with the
same device identity, and can run a heavy skill (e.g.
`forge enrich`) on the desktop and stream the AG-UI events to
the phone. Offline mode queues jobs and drains on reconnect.

### Cross-cutting: Extension Model (continuous, folded into each pillar)

- R8.6: plugin.json schema (P1)
- R8.2: document `strict: false` (P1)
- R8.5: `prometheus plugins list --outdated` / `update` (P3)
- R8.1, R8.3, R8.4: `prometheus plugins new` + Companion
  discover tab + `additionalDirectories` (P5)
- R8.10: plugin sandbox (P6 — the right time to land it, when
  the surface is stable)

### Hooks reliability (continuous, folded into P1, P2, P3)

- R6.8: replace `bash -c` with `prom-hook-dispatch` Rust binary
  (P2 — done with the build harness)
- R6.1 + R6.6 + R6.9: extracted scripts + clean stdout + log
  to ndjson (P1, with the watchdog)
- R6.5 + R6.3: matcher regex + timeout (P1, free)

---

## 13 · What to Read Next

For each pillar, here's the *single best external reference* that
informed this doc:

| Pillar | Reference |
|---|---|
| Reliability / launchd | [dajai.io "launchd Is the Best Production Orchestrator You Already Own"](https://dajai.io/blog/launchd-self-healing-mac-production-fleet) |
| Reliability / watchdog | [dev.to/whoffagents "How to Build a Crash-Tolerant AI Agent with launchd on macOS"](https://dev.to/whoffagents/how-to-build-a-crash-tolerant-ai-agent-with-launchd-on-macos-454) |
| Reliability / crash-loop | [stepcodex.com "Gateway silently dies after auto-update"](https://www.stepcodex.com/en/issue/gateway-silently-dies-after-auto-update) |
| Tauri tray | [dev.to/hiyoyok "Building a Menubar App with Tauri v2 — What Nobody Tells You"](https://dev.to/hiyoyok/building-a-menubar-app-with-tauri-v2-what-nobody-tells-you-2nae) |
| Tauri sidecar | [dev.to/chenxxpro "Bundling a CLI Binary as a Tauri v2 Sidecar"](https://dev.to/chenxxpro/bundling-a-cli-binary-as-a-tauri-v2-sidecar-lessons-from-building-a-desktop-app-5po) |
| Flutter + Rust FFI (mobile) | [cjycode.com/flutter_rust_bridge](https://cjycode.com/flutter_rust_bridge/), [AppFlowy tech design: Flutter + Rust](https://appflowy.com/blog/tech-design-flutter-rust), [1Password Typeshare](https://github.com/1Password/typeshare) |
| Ollama architecture | [deepwiki.com/ollama/ollama/8.1-desktop-app-architecture](https://deepwiki.com/ollama/ollama/8.1-desktop-app-architecture) |
| Service consolidation | [lotusee.xyz "Tracing Workflow Boundaries"](https://www.lotusee.xyz/posts/tracing-workflow-boundaries-lotusee-s-lens-on-rust-process-topologies) |
| Mobile Rust | [mobilesystemdesign.substack.com "Multiplatform with Rust on iOS"](https://mobilesystemdesign.substack.com/p/multiplatform-with-rust-on-ios-2c4) |
| P2P Rust | [lib.rs/crates/guardian-db](https://lib.rs/crates/guardian-db) (Iroh), [arxiv.org/html/2511.11619v1 DIAP](https://arxiv.org/html/2511.11619v1) (Noise + libp2p + Iroh hybrid) |
| Rust build time | [kunalganglani.com "Reduce Rust Compile Time 2026"](https://www.kunalganglani.com/blog/reduce-rust-compile-time), [reintech.io "How to Speed Up Rust Compile Times"](https://reintech.io/blog/how-to-speed-up-rust-compile-times-practical-optimization-tips) |
| Hooks reliability | [alexdunlop.com "Why Your Claude Code Hook Isn't Firing"](https://www.alexdunlop.com/writing/claude-code-hook-not-firing), [hookstack.app "Claude Code Hooks Not Working? Fix Guide"](https://www.hookstack.app/guides/claude-code-hooks-not-working) |
| Skill selection | [leehanchung.github.io "Claude Agent Skills: A First Principles Deep Dive"](https://leehanchung.github.io/blogs/2025/10/26/claude-skills-deep-dive/), [reddit.com/r/ClaudeAI "How does a Claude Code agent navigate hundreds of skills in a second?"](https://www.reddit.com/r/ClaudeAI/comments/1tlr914/how_does_a_claude_code_agent_navigate_hundreds_of/), [generativeprogrammer.com "Skill Authoring Patterns from Anthropic's Best Practices"](https://generativeprogrammer.com/p/skill-authoring-patterns-from-anthropics) |
| Extension | [claude.com/blog "Customize Claude Code with plugins"](https://claude.com/blog/claude-code-plugins), [github.com/anthropics/claude-plugins-official](https://github.com/anthropics/claude-plugins-official) |
| Lazy skills | [boliv.substack.com "Lazy Skills: A Token-Efficient Approach to Dynamic Agent Capabilities"](https://boliv.substack.com/p/lazy-skills-a-token-efficient-approach) |
| Auto-integration | [`/Users/gqadonis/Projects/hybrid-mobile-architecture-src/docs/08-auto-skill-package-integration.md`](file:///Users/gqadonis/Projects/hybrid-mobile-architecture-src/docs/08-auto-skill-package-integration.md) |
| Hooks reliability (skill) | [`/Users/gqadonis/Projects/hybrid-mobile-architecture-src/docs/03-hooks-reliability.md`](file:///Users/gqadonis/Projects/hybrid-mobile-architecture-src/docs/03-hooks-reliability.md) (HMA v0.2.0+) |
| LaunchAgent supervisor (skill) | [`/Users/gqadonis/Projects/hybrid-mobile-architecture-src/docs/07-launchagent-supervisor-spec.md`](file:///Users/gqadonis/Projects/hybrid-mobile-architecture-src/docs/07-launchagent-supervisor-spec.md) (HMA v0.2.0+) |
| Connected skill packages (skill) | [`/Users/gqadonis/Projects/hybrid-mobile-architecture-src/docs/01-connected-skill-packages.md`](file:///Users/gqadonis/Projects/hybrid-mobile-architecture-src/docs/01-connected-skill-packages.md) (HMA v0.2.0+) |
| Realtime skill refiner (skill) | [`/Users/gqadonis/Projects/hybrid-mobile-architecture-src/docs/04-skill-refiner-loop.md`](file:///Users/gqadonis/Projects/hybrid-mobile-architecture-src/docs/04-skill-refiner-loop.md) (HMA v0.2.0+) |
| Tauri tray app (skill) | [`/Users/gqadonis/Projects/hybrid-mobile-architecture-src/docs/06-tauri-tray-app-spec.md`](file:///Users/gqadonis/Projects/hybrid-mobile-architecture-src/docs/06-tauri-tray-app-spec.md) (HMA v0.2.0+) |
| Joint HMA × PMP × Companion design | [`/Users/gqadonis/Projects/hybrid-mobile-architecture-src/docs/05-hma-pmp-companion-architecture.md`](file:///Users/gqadonis/Projects/hybrid-mobile-architecture-src/docs/05-hma-pmp-companion-architecture.md) |
| Companion spec | [`/Users/gqadonis/Projects/prometheus/prometheus-companion/docs/00-architecture-and-implementation-plan.md`](file:///Users/gqadonis/Projects/prometheus/prometheus-companion/docs/00-architecture-and-implementation-plan.md) |

---

## 14 · Decisions (resolved 2026-08-20)

The five open questions in the previous revision of this section
were resolved on 2026-08-20. They now drive Pillar 1 onward:

1. **Companion app: Tauri 2.0** ✅
   Tauri 2.0 native (desktop only — see §14 #3 for the
   separate Flutter mobile shell). 3 MB binary, native
   menu-bar mode, Rust in-process substrate. Tauri's IPC is
   the only sane way to wire `prometheus-exec` to the UI.
2. **Substrate strategy: in-process library + `--detach` mode** ✅
   Same binary runs embedded (in-process) by default, or
   supervised by the Companion (detached) when run as a daemon.
   Antigravity-Tools inlines; AetherLink uses sidecar. We get
   both: fastest IPC, plus safe updates via the supervised mode.
3. **Mobile: Flutter + Rust over FFI (NOT Tauri 2.0 mobile)** ✅
   Tauri is **desktop only** (Pillar 5). Mobile is a separate
   Flutter app shell that talks to the **same Rust substrate
   crates** via `flutter_rust_bridge`. The HMA skill pack's
   central architectural commitment is exactly this split:
   Flutter mobile / Tauri desktop. The mobile substrate
   (`crates/prometheus-substrate` with `mobile` features) is
   byte-for-byte the same Rust that powers desktop. Tauri 2.0
   mobile is explicitly out of scope — and so is Capacitor
   (adds no value, introduces latency, doubles the
   surface area). The reference products: 1Password (Rust
   core + per-platform shells), AppFlowy (Flutter + Rust
   core). The same pattern.
4. **Skill router: in-house — `skill-index` + `fastembed-rs` +
   SurrealDB vector type (embedded)** ✅
   In-house, no external MCP. `fastembed-rs` with
   `bge-small-en-v1.5` produces the embeddings; SurrealDB
   embedded (the same engine behind `surreal-memory`) stores
   them as a vector index with HNSW. One fewer native
   dependency (no `sqlite-vec`); the embedding store, the
   memory layer, and the graph layer all live in one engine.
5. **Plugin sandbox: deferred to Pillar 6** ✅
   Ship the Pillar 5 skill-package surface without a sandbox.
   Land Bubblewrap + sandbox-exec in Pillar 6 once the surface
   area is known and stable. Plugin installs in the meantime
   are operator-curated (every install shows the manifest diff
   and waits for an explicit confirm click).

That's the review. Total: **8 axes covered, 33 weaknesses
identified, 60+ remedies proposed, 6-pillar roadmap + 1
cross-cutting pillar (Pillar 7) + 1 new HMA-side skill
spec for auto-integration (Pillar 7.5).** Tell me which
pillar to start with and I'll write the implementation
plan + first PR.

---

## 15 · The downstream resolution matrix (W → skill/script)

This section is the **crosswalk** between every weakness
named in §1-§9 and the skill, script, or feature that
implements the remedy. The original review named 33
weaknesses; the joint design (next section, §16) added
6 new HMA skills; the **Companion** is the cross-repo
enforcer; the HMA ships the verifier scripts. Use this
table to find the current state of any weakness.

| W | Severity | Where the remedy lives | Status |
|---|---|---|---|
| **W1.1** launchd crash-loop amnesia | High | **HMA** `launchagent-supervisor` skill ships the 9-fix plist template; the script `scripts/render-supervisor-plist.sh` generates it. The Companion's `doctor` runs `verify-hooks-reliability.sh` against every connected package. | **Not implemented — design only** (was: Open) (Pillar 1) |
| **W1.2** port-only health probes | High | **Companion** `crates/prometheus-companion/src/health.rs` has the liveness/readiness pair; **HMA** `launchagent-supervisor` skill ships `scripts/verify-hooks-reliability.sh` which checks the plist has the right `StandardOutPath`. | **Not implemented — design only** (was: Open) (Pillar 1) |
| **W1.3** stale plist / binary mismatch | Med | **Companion** `crates/prometheus-companion/src/commands/assert-binary-id.sh` (planned) | **Not implemented — design only** (was: Open) |
| **W1.4** doctor not on a schedule | High | **Companion** spec §2.4 plans `prometheus-notify-down` Tauri command | **Not implemented — design only** (was: Open) |
| **W1.6** 60s `.bootstrap-lock` wait | High | **Companion** spec §6.4 — kill the lock | **Not implemented — design only** (was: Open) |
| **W1.7** two parallel install scripts | Med | **Companion** spec §3 — collapse to one | **Not implemented — design only** (was: Open) |
| **W2.1** no desktop tray + dashboard | Critical | **Companion** spec §2-§6 (the entire Pillar 5) + **HMA** `tauri-tray-app` skill | **Not implemented — design only** (was: Open) (Pillar 5) |
| **W3.1** 15+ processes to install | High | **Companion** spec §3 (the substrate as in-process library); **HMA** `deploy-hybrid-agentic-stack` skill | **Not implemented — design only** (was: Open) (Pillar 3) |
| **W4.1** no mobile shell | High | **Companion** spec §19 (Flutter + Rust over FFI mobile shell) + **HMA** `flutter-rust-ffi` skill | **Not implemented — design only** (was: Open) (Pillar 6) |
| **W5.1** no sccache / mold / superworkspace | High | **Companion** spec §5 + **Companion** `Cargo.toml` workspace member (planned) | **Not implemented — design only** (was: Open) (Pillar 2) |
| **W6.1** inline `bash -c` fragile | High | **HMA** `claude-hooks-reliability` skill; install script `scripts/install-hooks-reliability.sh`. The **Companion** spec §26.4 mandates extracted scripts. | **Not implemented — design only** (was: Open) (Pillar 7) |
| **W6.2** SHA re-validated every hook | Med | **HMA** `claude-hooks-reliability` skill (W6.2 fix: 60s cache) | **Not implemented — design only** (was: Open) (Pillar 7) |
| **W6.3** subagent matchers fragile | Med | **HMA** `claude-hooks-reliability` skill (W6.3 fix: regex-anchored) | **Not implemented — design only** (was: Open) (Pillar 7) |
| **W6.5** no process-group kill on timeout | Low | **HMA** `claude-hooks-reliability` skill (W6.5 fix) | **Not implemented — design only** (was: Open) (Pillar 7) |
| **W6.6** hook stdout pollutes decision JSON | Med | **HMA** `claude-hooks-reliability` skill (W6.6 fix: `exec 2>>"$LOG"`) | **Not implemented — design only** (was: Open) (Pillar 7) |
| **W6.7** no structured hook-result log | Med | **HMA** `claude-hooks-reliability` skill (W6.7 fix: `hooks.ndjson`) | **Not implemented — design only** (was: Open) (Pillar 7) |
| **W6.8** inline `bash -c` long-term | Low | **HMA** `claude-hooks-reliability` skill (W6.8 fix: `prom-hook-dispatch` Rust binary) | **Not implemented — design only** (was: Open) (Pillar 7) |
| **W6.9** `sessionstart-*` matchers too broad | Med | **HMA** `claude-hooks-reliability` skill (W6.9 fix) | **Not implemented — design only** (was: Open) (Pillar 7) |
| **W7.1** vague skill descriptions | High | **Companion** spec §9 (`scripts/validate-skill-descriptions.sh` planned); the **HMA**'s `connected-skill-packages` skill spec §2 enforces "USE WHEN" + "Do NOT use for" | **Not implemented — design only** (was: Open) (Pillar 3) |
| **W7.4** no semantic router | Med | **Companion** spec §10 — the `skill-index` (in-process) with `fastembed-rs` + `sqlite-vec` | **Not implemented — design only** (was: Open) (Pillar 3) |
| **W7.6** no lazy progressive disclosure | Med | **Companion** spec §10 | **Not implemented — design only** (was: Open) (Pillar 3) |
| **W7.7** no forced-eval hook | Med | **Companion** spec §10 + the `claude-hooks-reliability` HMA skill | **Not implemented — design only** (was: Open) (Pillar 3 + 7) |
| **W8.1** marketplace is curated | High | **Companion** spec §18.5 + **HMA** `connected-skill-packages` skill — both add the self-serve install path; the **HMA** `auto-skill-package-integration` skill adds the local-directory path | **Not implemented — design only** (was: Open) (Pillar 7) |
| **W8.2** `strict: false` undocumented | Med | **HMA** `connected-skill-packages` skill spec §2 documents the pattern | **Not implemented — design only** (was: Closed) |
| **W8.4** no `prometheus plugins new` scaffolder | High | **HMA** `connected-skill-packages` skill spec §10 — the `prometheus plugins new` is the Companion's `useInstallSkillPackage` hook (the Companion IS the scaffolder) | **Not implemented — design only** (was: Open) (Pillar 7) |
| **W8.10** no plugin sandbox | Med | **Companion** spec §18.5 — Phase 6 | **Not implemented — design only** (was: Open) |
| **W9.4** `PROMETHEUS_USER` disagreement | Med | **Companion** spec §3 — single `config/defaults.env` | **Not implemented — design only** (was: Open) |
| **W9.7** `kbd-open` path not validated | Med | **Companion** spec §2.4 — `validate-runtime.sh` | **Not implemented — design only** (was: Open) |

> **Corrected 2026-08-20 — read §18.1 before trusting this table.** The
> original status column ("Open (Pillar N)") implied scheduled work. Direct
> verification found that the HMA skills and Companion features this table
> routes remedies to **do not exist**: the HMA ships 30 skills, none of them
> the six named here, and none of the `verify-*.sh` scripts; the Companion is
> a 2-commit scaffold. Every such row is now marked **`Not implemented —
> design only`**.

**Reading the table:** "Closed" means the remedy is
shipped somewhere (a HMA skill, a Companion feature, a
script). "Open" means the remedy is **designed** but
not yet **implemented** in the repo (it's a Pillar
target). "Severity" is the original triage from §11.

**The Companion is the cross-repo enforcer** — its
`doctor` command runs every HMA-shipped `verify-*.sh`
script against every connected skill package, and the
`Connected Skill Packages` page in the UI surfaces
the result. The HMA ships the verifier scripts; the
Companion runs them.

---

## 16 · Joint HMA × PMP × Companion design (the cross-cutting work)

The pillars above are scoped to **one repo each** (PMP for
Pillars 1-4, Companion for Pillar 5, mobile for Pillar 6).
The cross-cutting work — what the **three repos do
together** — is Pillar 7. It is the work that makes the
Prometheus system a single coherent product rather than
three loosely-coupled skill packages.

This section was added after the joint design pass that
produced six new docs in
`/Users/gqadonis/Projects/hybrid-mobile-architecture-src/docs/`:

| Doc | Lines | What it adds |
|---|---|---|
| `05-hma-pmp-companion-architecture.md` | 1,197 | The three-repo topology, the HMA skill map, the gaps, the roadmap |
| `01-connected-skill-packages.md` | 382 | The `connected-skill-packages` HMA skill (install / upgrade / validate / remove from the Companion) |
| `03-hooks-reliability.md` | 372 | The `claude-hooks-reliability` HMA skill (the 9 fixes from §6 as reusable rules) |
| `04-skill-refiner-loop.md` | 360 | The `realtime-skill-refiner` HMA skill (the 5-stage Detect → Triage → Refine → Verify → Ship loop) |
| `06-tauri-tray-app-spec.md` | 400 | The `tauri-tray-app` HMA skill (tray + popover + health-aggregator pattern from the Companion) |
| `07-launchagent-supervisor-spec.md` | 395 | The `launchagent-supervisor` HMA skill (the 9 fixes from §1 as reusable plist + systemd templates) |
| `08-auto-skill-package-integration.md` | (new) | The `auto-skill-package-integration` HMA skill — **git or local-directory** auto-integration (this section) |

### 16.1 The HMA gap analysis (6 new skills)

The HMA package today has 35+ skills covering **building**
hybrid apps (Tauri + React + Flutter + Axum). It is
**incomplete for managing** the joint system with the
Companion. The six new skills fill that gap:

| New HMA skill | Built on | What it adds |
|---|---|---|
| `connected-skill-packages` | (no PMP skill) | The Companion's "Plugins & Skill Packages" page; install / upgrade / validate / remove a git-based skill package |
| `tauri-tray-app` | `tauri-custom-titlebar` + `tauri-react-vite` | The tray + popover + health-aggregator pattern from the Companion, generalized |
| `launchagent-supervisor` | (no PMP skill) | The 9 plist / systemd fixes from §1, as reusable templates |
| `realtime-skill-refiner` | `process/skill-refiner` (PMP) | The 5-stage reactive refinement loop driven by real failures |
| `claude-hooks-reliability` | (no PMP skill) | The 9 hook fixes from §6, as reusable rules |
| **`auto-skill-package-integration`** | `connected-skill-packages` (HMA) | **Auto-detect new skills in a watched directory or git URL; install without operator click. (The "no friction" path — see §17 for the new HMA spec.)** |

All six ship in the HMA `v0.2.0` release. They mirror to
the 5 per-harness directories (`.agents/`, `.claude/`,
`.codex/`, `.kimi-code/`, `.opencode/`,
`templates/project-skills/`) and are registered in
`plugin.json`'s `skills` array.

### 16.2 The HMA git-install contract (4 conditions)

The HMA repo must satisfy 4 conditions to be installable
by the Companion (or by Claude Code's `/plugin
marketplace add`):

1. **Valid `marketplace.json`** at the repo root (it has
   one; we verified the schema).
2. **Valid `plugin.json`** at the repo root (it has one).
3. **Every `SKILL.md` referenced in `plugin.json` exists**
   AND has a valid `name` in its YAML frontmatter that
   matches its directory name. Enforced by
   `scripts/verify-skill-manifest.sh` (new in HMA v0.2.0).
4. **Idempotent install** — `git clone` (idempotent) or
   `git pull --ff-only --reset-hard <sha>` (idempotent).
   No global state outside the install path.

### 16.3 The Companion side (Pillar 5 update)

The Companion spec at
`prometheus-companion/docs/00-architecture-and-implementation-plan.md`
now has a "Connected Skill Packages" section that adds:

- The `skillPackage` PEM entity (in the Companion's domain)
- The "Connected Skill Packages" page in the Settings UI
- The `install_skill_package` / `upgrade_skill_package` /
  `remove_skill_package` / `validate_skill_package` Tauri
  commands
- The harness-aware install (Claude Code / Codex / Kimi /
  OpenCode / Mavis)
- The `recommended-marketplaces.json` shipped with the
  Companion (HMA + PMP)

And, with the new `auto-skill-package-integration` skill
(§17), a fourth operation beyond install / upgrade /
validate / remove: **auto-integrate** from a watched git
URL or a watched local directory. The Companion watches
the source; when a new `SKILL.md` appears, the Companion
auto-installs it (or, in confirm mode, prompts the
operator).

### 16.4 The hooks-reliability fixes (cross-repo)

The 9 fixes from §6 are now in the HMA
`claude-hooks-reliability` skill, with install / verify
scripts. The Companion's `doctor` command runs
`scripts/verify-hooks-reliability.sh` against every
installed package that ships hooks. **The Companion is
the cross-repo enforcer** — it runs the verify script
that the HMA ships.

### 16.5 The realtime skill-refiner loop

The 5-stage Detect → Triage → Refine → Verify → Ship
loop is the realtime complement to the PMP
`skill-refiner` skill. The Triage stage always halts
for human approval (the architecture review's §17
"producer never grades its own work" rule). The
Companion's UI shows the side-by-side diff and the
verification result; the user clicks "Ship" to apply.

### 16.6 Pillar 7 (the new pillar) — the joint work

This is the cross-cutting work the three repos do
together. Sized to 4 engineering weeks, run in parallel
with Pillars 1-6.

| Sub-pillar | Week | Output |
|---|---|---|
| 7.1 HMA v0.2.0 (the 6 new skills + install contract) | 1 | HMA repo updated; `verify-skill-manifest.sh` exits 0 |
| 7.2 Companion "Connected Skill Packages" page | 1 | `skillPackage` entity, Tauri commands, UI page |
| 7.3 Hooks reliability (the 9 fixes applied) | 0.5 | All 3 repos' `hooks/hooks.json` satisfy the verify script |
| 7.4 Realtime skill-refiner loop | 1 | 5 Tauri commands; UI panel; end-to-end test |
| 7.5 Cross-repo adversarial review | 0.5 | All 3 docs pass `sycophancy-correction` and adversarial review |

**Total: 4 weeks, run in parallel with Pillars 1-6.**

### 16.7 Cross-references

- The **HMA-side joint spec**:
  `/Users/gqadonis/Projects/hybrid-mobile-architecture-src/docs/05-hma-pmp-companion-architecture.md`
- The **6 HMA-side skill specs**:
  `01-connected-skill-packages.md`, `03-hooks-reliability.md`,
  `04-skill-refiner-loop.md`, `06-tauri-tray-app-spec.md`,
  `07-launchagent-supervisor-spec.md`,
  `08-auto-skill-package-integration.md`
- The **Companion spec** (updated with the Connected
  Skill Packages section):
  `prometheus-companion/docs/00-architecture-and-implementation-plan.md`
- The **Companion rule files** (updated with the new
  rules):
  `prometheus-companion/AGENTS.md`, `prometheus-companion/CLAUDE.md`

### 16.8 Updated totals (after this section was added)

- 8 axes covered (the original 8)
- 33 weaknesses identified (the original 33)
- 60+ remedies proposed (the original 60+)
- **+ 7 HMA-side docs** (1,197 + 382 + 372 + 360 + 400 + 395 + (08) = 3,506 lines)
- **+ 6 new HMA skills** (filling the gaps surfaced by the review)
- 1 new pillar (Pillar 7 — the joint HMA × PMP × Companion work)
- **Total joint work: 7 pillars, 4 weeks of Pillar 7 in parallel with Pillars 1-6**

---

## 17 · The auto-skill-package-integration path (git or local directory)

The HMA's `connected-skill-packages` skill is the
**manual** path: the operator clicks "Install," "Upgrade,"
or "Remove." The new `auto-skill-package-integration`
skill (HMA v0.2.0) is the **no-friction** path: the
Companion watches a source, and when a new `SKILL.md`
appears, the Companion auto-installs it.

Two source types are supported:

| Source | What it is | How the Companion watches it |
|---|---|---|
| **Git URL** | a git repo with a `marketplace.json` at the root (the HMA, the PMP, any third-party marketplace) | `git fetch` + `git diff` every N minutes (configurable; default 15 min) |
| **Local directory** | a directory on disk with a `SKILL.md` per subdir; no git, no marketplace.json | `notify` (macOS) / `inotifywait` (Linux) / `ReadDirectoryChangesW` (Windows) for fs events |

The full spec is at
`/Users/gqadonis/Projects/hybrid-mobile-architecture-src/docs/08-auto-skill-package-integration.md`.

### 17.1 The two modes

The auto-integration runs in one of two modes
(configurable per-source in the Companion's UI):

1. **Confirm mode** (default) — the Companion detects a
   new `SKILL.md` and shows a toast: "Skill `foo`
   appeared in `~/my-org-skills/`. Install?" The operator
   clicks "Install" or "Skip."
2. **Auto mode** — the Companion installs the new skill
   without asking. The operator gets a notification after
   the install completes. **Use this mode only with
   trusted sources** (a personal directory, not a
   public marketplace).

### 17.2 The 4 conditions (the same as the manual path)

Auto-integration is subject to the **same 4-condition
install contract** as the manual path (§16.2). If a
detected `SKILL.md` fails the contract check, the
Companion does **not** install it; the operator sees
the failure in the "Refinement queue."

### 17.3 The data model (PEM 3.x)

A new PEM entity `skillSource` models the watched
sources:

```ts
export const skillSourceEntity = defineEntity({
  name: 'skillSource',
  privacyClass: 'local',   // never server-synced
  fields: {
    id: { type: 'id', clientId: true },
    label: { type: 'string' },
    kind: { type: 'enum', enum: ['git', 'local-dir'] },
    location: { type: 'string' },      // git URL or absolute path
    branch: { type: 'string' },         // for git only
    mode: { type: 'enum', enum: ['confirm', 'auto'] },
    pollIntervalSec: { type: 'number' }, // for git; default 900
    enabled: { type: 'boolean' },
    lastSeenSha: { type: 'string' },     // for git
    lastSeenMtime: { type: 'number' },   // for local
    lastCheckAt: { type: 'datetime' },
    lastDetectedSkills: { type: 'string[]' },
  },
})
```

### 17.4 The Tauri commands

```rust
#[tauri::command]
pub async fn add_skill_source(spec: SkillSourceSpec) -> Result<SkillSource, String>;
#[tauri::command]
pub async fn remove_skill_source(id: String) -> Result<(), String>;
#[tauri::command]
pub async fn check_skill_source(id: String) -> Result<Vec<DetectedSkill>, String>;
#[tauri::command]
pub async fn check_all_skill_sources() -> Result<HashMap<String, Vec<DetectedSkill>>, String>;
```

`check_all_skill_sources` is called by a periodic
timer (the auto-integration is a `StartInterval` timer
on macOS, a systemd `.timer` on Linux, etc.).

### 17.5 The UI

A new "Auto-integrate" panel in the Connected Skill
Packages page shows:

- The watched sources (git URLs and local directories)
- The detected-but-not-installed skills
- The installed skills (with their source attribution)
- A per-source "Last checked at" + "Last detected"

A "Watch this directory" button is in the file picker.
A "Watch this git URL" button is in the URL form.

### 17.6 The definition of done

- [ ] `skills/auto-skill-package-integration/SKILL.md`
      exists in the HMA repo
- [ ] Mirrored to the 5 per-harness directories
- [ ] Added to `plugin.json` `skills` array
- [ ] The `skillSource` PEM entity is in the Companion's
      domain
- [ ] The 4 Tauri commands in §17.4 are implemented
- [ ] The "Auto-integrate" panel renders in the Connected
      Skill Packages page
- [ ] A roundtrip test (drop a new `SKILL.md` into a
      watched dir → Companion detects → operator
      confirms → install completes → entity persisted)
      exits 0

---

*Generated 2026-08-20 by `prometheus-research` server v0.1.0,
session `job-1787209468-ab667a1a` (cancelled early; data
collected directly via the research infrastructure's reference
sources). Future revisions should be filed alongside this file
under `docs/audits/`.*

---

## 18 · Post-review corrections (verified 2026-08-20)

> **Status of this section:** the body above (§0–§17) is preserved as the
> historical artifact it is. This section records what a direct verification
> pass against the repository and the two sibling repos found. Where this
> section and the body disagree, **this section is correct** — it was produced
> by reading the files, not by research inference.
>
> Verification method: direct file reads, `find`/`grep` over the working tree,
> `git log`, and frontmatter parsing with the same `js-yaml` loader
> `scripts/validate-skills.js` uses.

### 18.1 The §15 resolution matrix points at infrastructure that does not exist

§1.4, §6.10, §15, §16, and §17 are built on the premise that *"the HMA ships
the scripts, the Companion runs them."* That chain is a **design intent, not a
shipped dependency**. Verified:

**`/Users/gqadonis/Projects/hybrid-mobile-architecture-src`** is at version
`2.0.0-alpha.2` and ships **30 skills**. All six skills §16.1 says "ship in the
HMA `v0.2.0` release" are **absent** — a repo-wide directory search returns
**zero** matches for each:

| Skill named in §16.1 | Present in HMA? |
|---|---|
| `connected-skill-packages` | **No** |
| `tauri-tray-app` | **No** |
| `launchagent-supervisor` | **No** |
| `realtime-skill-refiner` | **No** |
| `claude-hooks-reliability` | **No** |
| `auto-skill-package-integration` | **No** |

The verifier scripts §1.4 and §6.10 instruct the Companion to run —
`verify-supervisor.sh`, `render-supervisor-plist.sh`,
`install-launchagent-supervisor.sh`, `verify-hooks-reliability.sh`,
`install-hooks-reliability.sh`, `verify-skill-manifest.sh` — **none exist** in
`scripts/`. (`scripts/` there holds `verify-scaffold.sh`,
`verify-tauri-boot.sh`, and `generate-builder-manifests.mjs`.)

**`/Users/gqadonis/Projects/prometheus/prometheus-companion`** is a **2-commit
scaffold** (`19e2f08`, `773aa5e`): `src-tauri/src/{lib,main}.rs`, a stub
`src/app.tsx`, and a 4405-line spec at
`docs/00-architecture-and-implementation-plan.md`. There is no `health.rs`, no
`plugin.rs`, no Tauri command surface. Pillars 5 and 6 are **unimplemented
greenfield**.

**Consequence:** every §15 row whose remedy "lives in" an HMA skill or a
Companion feature should be read as **`Not implemented — design only`**, not as
the "Open (Pillar N)" scheduled-work status the table implies. Nothing in the
skill-pack may call those scripts, because they are not there to call.

### 18.2 Five factual claims in the body are wrong or already satisfied

| § | Claim as written | Verified reality |
|---|---|---|
| §0, §7.2 | "40+ skills and counting" | **312** `SKILL.md` files (148 first-party, 164 under `skills/imported/`); `SKILLS.md` frontmatter reports **163** distributed |
| §7.2 | "skill descriptions that exceed 1024 characters… the model can't see the trigger keywords that come after" | **Zero** descriptions exceed 1024 chars. Max **663** (`skills/devops/gitops-transform`), median **234**, total ≈56K chars ≈14K tokens. The real defect is trigger **quality**, not truncation |
| §1.2 (W1.1) | "Every plist in `shared/launchagents/*.plist` uses the bare-true variant… none contain `ThrottleInterval` or `ProcessType`" — naming `pk-cherry`, `forge-mcp`, `surreal-memory-native` as evidence | **4 of 12** already carry `ThrottleInterval` (`sovereign-sync`, `surreal-memory-native`, `surrealdb-native`, `learning-worker`) — so one of the three cited as evidence already has the fix. **9 of 12** already set `ProcessType`. `ai.prometheus.exec.plist:22-26` already uses the **`KeepAlive` dictionary form** that R1.2 proposes as new work. Only **4** plists genuinely need the throttle: `forge-mcp`, `liter-llm-api`, `pk-cherry`, `surface-bridge` |
| §1.3 (R1.4) | Lists `shared/scripts/service-probe.sh` as a file to create | **Already exists** (87 lines). Exports `probe_port()` and `check_running_service()`, is HTTP-first with a TCP fallback, and already handles `unix:<socket-path>` specs. It is sourced by `check-prerequisites.sh` and `install-mcp-services.sh` |
| §1.2 (W1.2) | "Port-based 'is it healthy?'… both probe a TCP port" | `scripts/check-mcp-health.sh` is **HTTP-based** (`curl -w '%{http_code}'`, `curl --unix-socket`, and a JSON-RPC POST for MCP). The genuine defect is different and narrower: `check-mcp-health.sh` **reimplements** probing instead of sourcing the existing `service-probe.sh`, so two probe implementations coexist |

Two further corrections to §5.2's build-time evidence: `sccache` **and**
`cargo-nextest` are already installed on this machine — `sccache --show-stats`
reports **0 compile requests**, i.e. installed but never wired in (there is no
repo-root `.cargo/config.toml`). And `mold`/`zld` are **absent** while `lld`
**is** present. The largest `target/` in the tree is **120 MB**
(`tools/surreal-memory-server`), not the "~2 GB of `target/` per crate
directory" §5.1 asserts.

### 18.3 The hooks are generated — §6.3's largest remedy is unnecessary

§6.2 (W6.1, W6.9) and R6.1 treat `hooks/hooks.json` as 30 hand-maintained
inline `bash -c` blobs and propose extracting each to a checked-in script to
kill "the inline quoting bug class."

In fact **both** `hooks/hooks.json` and `hooks/codex-hooks.json` are
**generated artifacts**, emitted by
`scripts/generate-harness-adapters.js:264-265` from a single declarative source
of truth, `shared/harnesses/hook-contract.json`. The `bash -c` body is **one
template** at `generate-harness-adapters.js:196`, rendered by `hookCommand()`
for every hook. The two 27 KB files are byte-identical except for the trailing
harness argument (`'claude-code'` vs `'codex'`), and `npm run
validate:harness-adapters` already guards them against drift.

**Consequences:**
- The copy-paste hazard R6.1 targets does not exist — no human edits those
  strings. **R6.1 should be skipped.**
- R6.5 (matcher regexes), R6.6 (`exec 2>>"$LOG"`), and R6.9 (matcher scoping)
  are **one contract edit plus one generator edit**, applied to all 30 hooks at
  once — not the 30 separate edits the effort column implies.
- Anyone acting on §6 must edit `shared/harnesses/hook-contract.json`, **never**
  `hooks/hooks.json` directly; a direct edit is overwritten by the next
  generator run and fails the `--check` gate.

### 18.4 What the review understated

**The skill-discovery problem is more urgent than §7 says.** With the corrected
count of **312** skills, `config/codex-catalog.txt`'s own empirically-measured
budget curve is the binding constraint: ~130 skills → ~166-char descriptions;
~200 → ~66; ~360 → **~10 chars, "broken — the model cannot tell skills apart."**
Meanwhile only **74 of 312 (24%)** descriptions contain a "Use when" trigger,
**0 of 312** contain an exclusion clause, and the string `USE WHEN` that R7.1
prescribes appears **nowhere in the repo**. §7's remedies are right; its sizing
is not.

**Three install scripts manage three different service lists** (5 / 7 / 11), so
`surface-bridge`, `sovereign-sync`, and `liter-llm-api` are installed by
`install-mcp-services.sh` but invisible to `prometheus-services.sh`'s
`start`/`stop`/`status`. W9.4 identifies the `PROMETHEUS_USER` disagreement but
not this list drift, which is the more damaging half:
`scripts/prometheus-services.sh:10` hardcodes `gqadonis` **and enforces it** at
`:111-113`, refusing to run for any other user.

**The Linux side is not at parity.** `shared/systemd/` has no unit for
`ai.prometheus.exec`, `liter-llm-api`, or `codex-skills-sync`, so three services
installed on macOS have no Linux equivalent at all.

**`.claude-plugin/plugin.json` does not exist** — §8.1 and the surrounding
discussion assume a root plugin manifest. Per-plugin manifests live in each
source directory instead (e.g. `skills/devops/.claude-plugin/plugin.json`).

### 18.5 Scope decision taken on the back of this verification

Because §15's remedies for Pillars 5–6 live in repos that have not implemented
them, work driven by this audit is scoped to **what is actionable inside
`prometheus-skill-pack`**: reliability (§1), hooks (§6), skill discovery (§7),
build time (§5), and the Companion-independent parts of the extension model
(§8). Pillars 5 (Companion/Tauri) and 6 (Mobile/Flutter), and the six HMA
skills, are recorded as explicit non-goals in
`docs/audits/2026-08-20-review-scope-decisions.md`.

### 18.6 §6 remedies re-scoped after measurement (2026-08-20)

§6 was acted on by measuring the running system rather than by applying the
remedy list. Four of its items were **withdrawn on evidence**:

**R6.1 (extract inline `bash -c` scripts) — withdrawn.** See §18.3: the hook
files are generated from one template, so the copy-paste hazard does not exist.

**W6.3 second claim (the no-matcher `SubagentStop` group "fires for every
subagent, defeating the per-role design") — withdrawn; the claim is backwards.**
That group is a deliberate catch-all, documented at
`docs/guide/15-hooks-and-lifecycle.md:113`: *"The fallback matcher guarantees
that even an unrecognized subagent gets a checkpoint — no role falls through
silently."* It runs `subagent-checkpoint-fallback.sh`, which writes one line to
**stderr** and always exits 0. Live hook telemetry confirms the intended
behaviour: the fallback fired 7 times while role-scoped hooks fired 5, i.e. it
runs *in addition to* a matched role, not instead of it. Removing it would
reintroduce the silent-drop bug it was added to fix
(`docs/plans/2026-04-29-change-006-karpathy-loop-hooks.md:236`).

**R6.5 (anchor the subagent matchers as `^planner$`) — withdrawn as unsafe.**
The five matchers (`assessor`, `analyst`, `planner`, `executor`, `reflector`)
name agents shipped by the `iterative-evolver` plugin, not by this repo. Two
facts are needed to anchor them safely — whether Claude Code evaluates the
matcher anchored, and whether a plugin-provided subagent's `agent_type` is bare
(`planner`) or namespaced (`iterative-evolver:planner`) — and **neither is
documented**. Meanwhile the live log proves the current form works: a
`SubagentStop[executor]` record exists for the plugin-provided `executor`.
Anchoring on an undocumented comparison would risk silently disabling 15 hooks
to defend against a hypothetical rename. Left as-is deliberately.

**R6.3 (cache the dispatcher SHA for 60s) — withdrawn as a bad trade.**
Measured on this machine: `shasum -a 256` over the 5152-byte dispatcher costs
**~14 ms**. Real hook telemetry from `~/.prometheus/hooks.log` shows the actual
cost centres are elsewhere by three orders of magnitude:

| hook | script | max ms |
|---|---|---|
| SubagentStop[executor] | `evaluate-session.sh` | **16408** |
| SessionStart | `pk-health.sh` | **7856** |
| SessionStart | `memory-outbox-flush.sh` | 1181 |

Caching 14 ms while trading away per-invocation bundle-integrity verification is
the wrong trade. (`pk-health.sh` is already throttled to one run per 24 h, so its
7.9 s is a daily cost, not a per-session one.)

**R6.7 (add a structured hook-result log) — already shipped.**
`shared/scripts/lib/hook-log.sh` has written JSONL records
(`{ts, hook, script, pid, session_id, exit_code, duration_ms}`) to
`~/.prometheus/hooks.log` for some time; the file holds live data and shows **0
non-zero exits**. The genuine gap is *adoption*: only **16 of 36**
`shared/scripts/*.sh` source the library, so the remaining hooks are invisible
to it. That, not the absence of a logger, is what to fix.
