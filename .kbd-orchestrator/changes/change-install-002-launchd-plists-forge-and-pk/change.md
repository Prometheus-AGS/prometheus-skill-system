---
id: change-install-002-launchd-plists-forge-and-pk
title: Launchd plists forge and pk
phase: machine-installation-2026-05-25
gaps: [G-SVC-1, G-SVC-2]
depends_on: [change-install-001]
agent: claude-code
status: done
---

# change-install-002-launchd-plists-forge-and-pk — Launchd plists forge and pk

## Summary

Create and load launchd plist files for `forge-mcp` (port 8943) and `pk-mcp` (port 8942), modeled on the existing `dev.prometheusags.openai-proxy.plist`.

## Files Created

- `~/Library/LaunchAgents/dev.prometheusags.forge-mcp.plist`
- `~/Library/LaunchAgents/dev.prometheusags.pk-mcp.plist`

## Acceptance Criteria

- `launchctl list | grep forge-mcp` → shows PID (non-zero)
- `launchctl list | grep pk-mcp` → shows PID (non-zero)
- Port 8943 is open (TCP connect succeeds)
- Port 8942 is open (TCP connect succeeds)

## Tasks

- [x] 1. Read `~/Library/LaunchAgents/dev.prometheusags.openai-proxy.plist` as template
- [x] 2. Write `~/Library/LaunchAgents/dev.prometheusags.forge-mcp.plist` (label, binary path, port 8943, RUST_LOG=info)
- [x] 3. Write `~/Library/LaunchAgents/dev.prometheusags.pk-mcp.plist` (label, binary path, port 8942, RUST_LOG=info)
- [x] 4. `launchctl load ~/Library/LaunchAgents/dev.prometheusags.forge-mcp.plist`
- [x] 5. `launchctl load ~/Library/LaunchAgents/dev.prometheusags.pk-mcp.plist`
- [x] 6. Verify `launchctl list | grep forge-mcp` shows non-zero PID
- [x] 7. Verify `launchctl list | grep pk-mcp` shows non-zero PID
- [x] 8. Probe port 8943 is accepting connections
- [x] 9. Probe port 8942 is accepting connections
