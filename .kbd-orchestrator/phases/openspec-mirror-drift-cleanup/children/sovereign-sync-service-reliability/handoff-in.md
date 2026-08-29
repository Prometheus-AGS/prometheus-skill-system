# Handoff in — openspec-mirror-drift-cleanup› sovereign-sync-service-reliability

**Spawned by:** openspec-mirror-drift-cleanup

## Why this child was spawned

The parent workflow was blocked because KBD clients treated the managed
Unix-socket daemon as unreachable, the daemon withheld all stateful routes when
any registered project failed to open, and local fallback used an unenrolled
signer. The child isolates diagnosis, runtime repair, deployment, and restart
proof before returning to the reflected parent.

## Inputs (paths from the parent node)

- .kbd-orchestrator/phases/openspec-mirror-drift-cleanup/assessment.md
- .kbd-orchestrator/phases/openspec-mirror-drift-cleanup/plan.md

## Success criteria

- The installed CLI uses the managed Unix socket without requiring a TCP listener.
- Healthy KBD projects stay routable when stale registrations fail to open.
- Interactive and daemon mutations use the same enrolled managed device key.
- Two forced launchd restarts preserve health and KBD read/write access.
- Local tests, clippy, formatting, and package refresh checks pass.

## Expected deliverables

- Runtime, daemon, CLI transport, and KBD skill fixes with regression tests.
- Installed release binaries and restart evidence.
- Assessment, plan, execution, reflection, and handoff-out artifacts.
