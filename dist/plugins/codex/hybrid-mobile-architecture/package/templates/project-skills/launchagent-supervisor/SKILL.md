---
name: launchagent-supervisor
description: Author, install, and supervise a macOS LaunchAgent, a Linux systemd --user unit, or a Windows Scheduled Task for a long-running managed daemon. Use when adding a service to the substrate, when a service is crash-looping or silently disappearing, or when asked for auto-restart, self-healing, or a watchdog. Triggers on LaunchAgent, plist, launchd, launchctl, KeepAlive, ThrottleInterval, ProcessType, RunAtLoad, daemon, supervisor, self-healing, restart loop, watchdog, systemd user unit, scheduled task.
---
<!-- TJ-ARCH-MOB-001 compliant -->

# LaunchAgent Supervisor

The failure this prevents is **crash-loop amnesia**: `KeepAlive: true` with no
`ThrottleInterval` lets launchd classify a restarting job as inefficient and
silently remove it. The port goes quiet, no error is logged, and an installer
that only checks "did the file get written" reports healthy.

Render templates with `scripts/render-supervisor-plist.mjs`; never hand-write a
plist. Templates live in `assets/templates/launchagent-supervisor/`.

## The 9 fixes

| # | Fix | Why it matters |
|---|---|---|
| R1.1 | `ThrottleInterval` ≥ 15 | Restarting faster than launchd's threshold gets the job removed, not throttled. |
| R1.2 | `KeepAlive` dictionary form | `SuccessfulExit: false` + `Crashed: true` restarts crashes without fighting planned shutdowns. Bare `true` restarts everything, including a deliberate stop. |
| R1.3 | `ProcessType: Interactive` | Raises launchd's resource threshold; a background-classified job gets terminated under pressure. |
| R1.4 | `StandardOutPath` / `StandardErrorPath` | Without explicit paths the output is unreachable and the failure is invisible. |
| R1.5 | `RunAtLoad: true` | Otherwise the service is down after every reboot until someone runs `launchctl kickstart` by hand. |
| R1.6 | Self-check watchdog | The service periodically confirms it is still registered and re-bootstraps if launchd dropped it. This is what actually closes the amnesia hole. |
| R1.7 | Down-for-too-long notification | A service down past a threshold must surface to the operator rather than waiting to be noticed. |
| R1.8 | PID-file bootstrap lock | A `mkdir` lock survives the crash that created it and deadlocks the next install. A PID file lets a successor reclaim a lock whose owner is gone. |
| R1.9 | Exactly one installer | Two installers racing the same label produce a service whose definition depends on install order. |

## macOS plist shape

```xml
<key>KeepAlive</key>
<dict>
  <key>SuccessfulExit</key><false/>
  <key>Crashed</key><true/>
</dict>
<key>ThrottleInterval</key><integer>15</integer>
<key>ProcessType</key><string>Interactive</string>
<key>RunAtLoad</key><true/>
```

Validate every rendered plist before loading it:

```bash
plutil -lint ~/Library/LaunchAgents/<label>.plist
```

A plist that fails `plutil -lint` is rejected by launchd with no useful message.

## Linux systemd --user

`RestartSec=15` is the `ThrottleInterval` equivalent; `Restart=on-failure` is
the `KeepAlive` dictionary equivalent. Using `Restart=always` reintroduces the
bare-`KeepAlive` bug.

```ini
[Service]
Type=simple
Restart=on-failure
RestartSec=15
StandardOutput=append:%h/Library/Logs/KnowMe/<service>.log
StandardError=append:%h/Library/Logs/KnowMe/<service>.err

[Install]
WantedBy=default.target
```

## Windows Scheduled Task

The weakest of the three supervisors. Trigger at logon, restart on failure at a
fixed interval with a bounded retry count, run as the current user. Treat its
health reporting as unreliable and lean harder on the R1.6 watchdog.

## Verification

```bash
node scripts/render-supervisor-plist.mjs --label ai.prometheus.demo \
  --program /usr/local/bin/demo --out /tmp/demo.plist
plutil -lint /tmp/demo.plist
launchctl print gui/"$(id -u)"/ai.prometheus.demo    # after bootstrap
```

Checking that the plist file exists is not a health check. Confirm launchd
reports the label and the service answers on its port.

## Anti-patterns

- `KeepAlive: true` with no `ThrottleInterval`.
- Treating "the plist was written" as "the service is running".
- Two installers writing the same label.
- A lock directory rather than a PID file.
- Logging to a path the operator is never told about.
