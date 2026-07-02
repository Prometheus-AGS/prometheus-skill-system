---
id: change-slli-002-mcp-launchd-services
title: MCP servers as launchd services
phase: self-learning-loop-integration
gaps: [MCP-SVC-1 through MCP-SVC-7]
priority: 2 of 10
agent: claude-code
status: done
scope:
  - launchd/dev.prometheusags.surreal-memory.plist
  - launchd/dev.prometheusags.pk-mcp.plist
  - launchd/dev.prometheusags.forge-mcp.plist
  - launchd/dev.prometheusags.sycophancy-correction.plist
  - launchd/dev.prometheusags.liter-llm.plist
  - launchd/dev.prometheusags.sequential-thinking.plist
  - launchd/dev.prometheusags.tavily.plist
  - launchd/dev.prometheusags.prometheus-nudge.plist
  - scripts/install-mcp-services.sh
  - scripts/check-mcp-health.sh
  - scripts/prometheus-services.sh
---

# change-slli-002-mcp-launchd-services — MCP servers as launchd services

## Summary

Install all 7 MCP servers as macOS launchd launch agents so they are always running, addressable by known URLs/ports, and survive reboot. This is a prerequisite for configuring all AI tools to point at them (change-slli-003).

Builds on `openspec/changes/change-install-002-launchd-plists-forge-and-pk` (which covers only forge and pk) and extends it to the full service set.

## Files Created

### Plist Files (installed to ~/Library/LaunchAgents/)

- `launchd/dev.prometheusags.surreal-memory.plist` → port 23001
- `launchd/dev.prometheusags.pk-mcp.plist` → port 8942 (HTTP MCP mode: `pk-mcp --http --port 8942`)
- `launchd/dev.prometheusags.forge-mcp.plist` → port 8943 (HTTP MCP mode: `forge-mcp --http --port 8943`)
- `launchd/dev.prometheusags.sycophancy-correction.plist` → port 8944 (HTTP mode)
- `launchd/dev.prometheusags.liter-llm.plist` → port 8945 (`liter-llm mcp --transport http --port 8945`)
- `launchd/dev.prometheusags.sequential-thinking.plist` → port 8946
- `launchd/dev.prometheusags.tavily.plist` → port 8947
- `launchd/dev.prometheusags.prometheus-nudge.plist` → cron (every 4 hours, no port)

### Scripts

- `scripts/install-mcp-services.sh` — idempotent installer:
  1. Copies plists to `~/Library/LaunchAgents/`
  2. Runs `launchctl bootout` + `launchctl bootstrap` for each (handles already-loaded gracefully)
  3. Waits up to 10s per service for port to be open
  4. Reports load status table
- `scripts/check-mcp-health.sh` — health check:
  - TCP connect test for each port
  - `launchctl list | grep prometheusags` for PID
  - Renders a GREEN/RED status table
- `scripts/prometheus-services.sh` updated — now delegates to launchctl instead of direct process management

### Plist Template

Each plist follows this structure:
```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>Label</key>            <string>dev.prometheusags.SERVICE</string>
  <key>ProgramArguments</key> <array>...</array>
  <key>EnvironmentVariables</key>
  <dict>
    <key>PATH</key>           <string>/usr/local/bin:/opt/homebrew/bin:/usr/bin:/bin</string>
    <!-- API keys loaded from ~/.prometheus/.env via launchd EnvironmentFile workaround -->
  </dict>
  <key>RunAtLoad</key>        <true/>
  <key>KeepAlive</key>        <true/>
  <key>StandardOutPath</key>  <string>/Users/USER/.prometheus/logs/SERVICE.log</string>
  <key>StandardErrorPath</key><string>/Users/USER/.prometheus/logs/SERVICE-err.log</string>
  <key>ThrottleInterval</key> <integer>10</integer>
</dict>
</plist>
```

## Environment Variable Handling

API keys (TAVILY_API_KEY, etc.) are read from `~/.prometheus/.env` by a wrapper script loaded as the `ProgramArguments` entry. The wrapper does `source ~/.prometheus/.env && exec <actual-binary>`.

## Acceptance Criteria

- `launchctl list | grep prometheusags` shows all 8 labels (7 servers + nudge)
- `scripts/check-mcp-health.sh` reports GREEN for all 7 MCP server ports
- All services survive `sudo launchctl reboot` (macOS)
- Log files exist at `~/.prometheus/logs/`
- Script is idempotent: running `install-mcp-services.sh` twice leaves same state

## Tasks

- [x] 1. `launchctl list | grep prometheusags` shows all 8 labels (7 servers + nudge)
- [x] 2. `scripts/check-mcp-health.sh` reports GREEN for all 7 MCP server ports
- [x] 3. All services survive `sudo launchctl reboot` (macOS)
- [x] 4. Log files exist at `~/.prometheus/logs/`
- [x] 5. Script is idempotent: running `install-mcp-services.sh` twice leaves same state
