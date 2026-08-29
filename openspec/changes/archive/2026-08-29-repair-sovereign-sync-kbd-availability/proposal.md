## Why

The managed `sovereign-sync` daemon can be healthy under launchd while KBD is effectively unavailable because local clients target the wrong transport, interactive commands sign with a different identity, or one stale project registration prevents every project route from being installed. KBD must remain usable across restarts and partial registry degradation so a single unhealthy registration cannot block the orchestration process.

## What Changes

- Make managed KBD clients prefer the private same-user Unix control socket while retaining an explicit TCP endpoint override.
- Reuse the managed daemon device signer for default canonical KBD state so interactive commands and the service share an enrolled identity.
- Report `kbd-runtime` authority health as embedded in `sovereign-sync`, avoiding false guidance that implies a separate KBD daemon must be restarted.
- Keep healthy project-authority routes available when other registered projects fail to open, and report each failed project with its concrete error.
- Repair child-phase orchestration seams so runtime child creation initializes its label before activation and stage gates compare fully-qualified child phase IDs.
- Refresh generated Codex plugin distributions and certify the repaired service, CLI, runtime, and shell paths locally.

## Capabilities

### New Capabilities

- None.

### Modified Capabilities

- `sovereign-sync-daemon-health`: Extend daemon-health behavior to cover the managed Unix control transport, signer convergence, and partial registered-project availability across supervised restarts.

## Impact

- Affects `substrate/kbd-runtime`, `substrate/sovereign-sync`, and the `prometheus` CLI control transport.
- Affects KBD child creation and stage-gate shell helpers plus their generated Codex plugin copies.
- Does not change the public TCP API; an explicit configured TCP endpoint remains supported.
- Does not automatically delete stale project registrations or rewrite historical KBD projection evidence.
