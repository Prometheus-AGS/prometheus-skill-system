---
id: change-prb-006-launchd-plist
title: Write launchd plist and wire into install-binaries.sh
phase: phase-prometheus-research-binary
priority: P1
effort: S
wave: 3
agent: general-purpose
status: pending
gap_id: G-07
verdict: BUILD
depends_on: change-prb-003-mcp-server
scope:
  - substrate/prometheus-research/com.prometheus.research.plist
  - scripts/install-binaries.sh
---

# Change: launchd plist + install-binaries.sh wiring

## Problem

`prometheus-research` has no auto-start mechanism. It must be launched manually
every session, and MCP harnesses cannot use it without a running process.

## Solution

### launchd plist

`com.prometheus.research.plist` starts `prometheus-research --mode mcp` on login:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>Label</key>        <string>com.prometheus.research</string>
  <key>ProgramArguments</key>
  <array>
    <string>/Users/<user>/.local/bin/prometheus-research</string>
    <string>--mode</string>
    <string>mcp</string>
  </array>
  <key>RunAtLoad</key>    <true/>
  <key>KeepAlive</key>    <true/>
  <key>StandardOutPath</key>  <string>/tmp/prometheus-research.log</string>
  <key>StandardErrorPath</key><string>/tmp/prometheus-research.error.log</string>
</dict>
</plist>
```

Use `$(whoami)` substitution at install time (not hardcoded username).

### install-binaries.sh changes

Add a new section after the existing binary installs:

```bash
# ── N. prometheus-research ──────────────────────────────────────────────
if [ -f "${REPO_ROOT}/substrate/prometheus-research/Cargo.toml" ]; then
    info "Building prometheus-research..."
    (cd "${REPO_ROOT}/substrate/prometheus-research" && cargo build --release 2>&1 | tail -3)
    install_bin "${REPO_ROOT}/substrate/prometheus-research/target/release/prometheus-research" \
                "${BIN_DIR}/prometheus-research"
    ok "prometheus-research → ${BIN_DIR}/prometheus-research"

    # Install launchd plist
    PLIST_SRC="${REPO_ROOT}/substrate/prometheus-research/com.prometheus.research.plist"
    PLIST_DST="${HOME}/Library/LaunchAgents/com.prometheus.research.plist"
    sed "s|<HOME>|${HOME}|g" "${PLIST_SRC}" > "${PLIST_DST}"
    launchctl bootout "gui/$(id -u)" "${PLIST_DST}" 2>/dev/null || true
    launchctl bootstrap "gui/$(id -u)" "${PLIST_DST}"
    ok "prometheus-research launchd service registered"
else
    info "skip prometheus-research (substrate not built)"
fi
```

## Acceptance Criteria

- [ ] `substrate/prometheus-research/com.prometheus.research.plist` exists with correct XML
- [ ] `scripts/install-binaries.sh` contains a `prometheus-research` section
- [ ] After running `bash scripts/install-binaries.sh`, binary exists at `~/.local/bin/prometheus-research`
- [ ] `launchctl list | grep prometheus.research` shows the service
- [ ] `prometheus-research --mode mcp` accepts stdin JSON-RPC without errors
