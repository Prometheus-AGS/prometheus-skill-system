---
id: change-onboard-002-linux-systemd-note
title: Linux/systemd note in QUICK_START.md troubleshooting
phase: phase-first-user-onboarding
agent: claude-code
status: done
scope:
  - docs/QUICK_START.md
---

# change-onboard-002-linux-systemd-note — Linux/systemd note in QUICK_START.md troubleshooting

## Summary

Extend the "MCP services not running" troubleshooting item in `docs/QUICK_START.md`
to include the Linux/systemd path alongside the existing macOS/launchd instructions.

## Problem

The troubleshooting section's MCP services note references
`scripts/prometheus-services.sh load`, which internally calls `launchctl` — a
macOS-only command. A Linux user following this guide gets no guidance on how to
start the services. The `install-skills-flat.sh` script does install systemd
service units on Linux, but the Quick Start doesn't tell users how to activate them.

## Change

**File:** `docs/QUICK_START.md`

**Location:** `## Troubleshooting` → `**MCP services not running**` paragraph

**Replace the existing paragraph with:**
```markdown
**MCP services not running** — On macOS, surface-bridge (port 7890) and sovereign-sync
(port 7892) are launchd services. Start them with:
```bash
bash scripts/prometheus-services.sh load
bash scripts/prometheus-services.sh status
```
On Linux, use systemd:
```bash
systemctl --user start prometheus-surface-bridge prometheus-sovereign-sync
systemctl --user status prometheus-surface-bridge prometheus-sovereign-sync
```
```

## Acceptance criteria

- The macOS launchd instructions are preserved unchanged
- The Linux systemd commands appear immediately after the macOS section
- Port numbers (7890, 7892) are consistent with the rest of the documentation
- No other content in the file is modified

## QA gate

Documentation-only change. QA gate skipped per execute protocol.

## Tasks

- [x] 1. The macOS launchd instructions are preserved unchanged
- [x] 2. The Linux systemd commands appear immediately after the macOS section
- [x] 3. Port numbers (7890, 7892) are consistent with the rest of the documentation
- [x] 4. No other content in the file is modified
