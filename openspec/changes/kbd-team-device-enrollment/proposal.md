# kbd-team-device-enrollment

## Why

Observed (operator report, 2026-11-10, reproduced): after one operator sets up a KBD plan and
commits it, a teammate cannot change that plan. Every write from the teammate's machine fails
with `signer <key> is not the journal's active enrolled device`.

Root cause in `substrate/kbd-runtime`:

- the genesis signer becomes the only operator key;
- every later event must be signed by an enrolled, active device, and lifecycle/plan commands
  (`pause`, `resume`, `run start`, `cancel`, `resolve`) need an operator key;
- `CommandKind::DeviceEnroll` exists but `prometheus kbd` exposes no command for it; and
- no event adds a key to `operator_key_ids`, so no teammate can ever pause, resume, start or cancel a run, or enroll others.

## What changes

- Runtime: new `CommandKind::OperatorGrant { key_id }` → `EventKind::OperatorGranted { key_id }`.
  Only an active operator may grant; the target must be an enrolled, active, non-operator device.
  Revoking the device (`DeviceRevoked`) already removes operator authority.
- Runtime: `Runtime::local_device_identity()` returns this machine's signing key id and public
  key for the project (same key-discovery order used when signing), without writing the journal.
- CLI: `prometheus kbd device identity | list | enroll | grant-operator | revoke`.
  `enroll --operator` enrolls and grants in two signed events.

## Out of scope

- Journal transport between machines (sovereign-sync / control plane) is unchanged.
- A missing local journal still bootstraps a new run silently; tracked separately.

## Security (A-3)

This is a trust-boundary change: who may sign operator events. Grants are operator-signed
journal events, replay-verified like enrollment, auditable via `prometheus kbd audit`, and
reversible with `device revoke`.
