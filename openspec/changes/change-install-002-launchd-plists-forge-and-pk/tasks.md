# Tasks: change-install-002-launchd-plists-forge-and-pk

- [ ] Read `~/Library/LaunchAgents/dev.prometheusags.openai-proxy.plist` as template
- [ ] Write `~/Library/LaunchAgents/dev.prometheusags.forge-mcp.plist` (label, binary path, port 8943, RUST_LOG=info)
- [ ] Write `~/Library/LaunchAgents/dev.prometheusags.pk-mcp.plist` (label, binary path, port 8942, RUST_LOG=info)
- [ ] `launchctl load ~/Library/LaunchAgents/dev.prometheusags.forge-mcp.plist`
- [ ] `launchctl load ~/Library/LaunchAgents/dev.prometheusags.pk-mcp.plist`
- [ ] Verify `launchctl list | grep forge-mcp` shows non-zero PID
- [ ] Verify `launchctl list | grep pk-mcp` shows non-zero PID
- [ ] Probe port 8943 is accepting connections
- [ ] Probe port 8942 is accepting connections
