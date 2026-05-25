# change-install-002-launchd-plists-forge-and-pk

**Phase**: machine-installation-2026-05-25  
**Status**: PENDING  
**Gaps closed**: G-SVC-1, G-SVC-2  
**Depends on**: change-install-001

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
