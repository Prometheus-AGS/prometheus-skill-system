## Why

KBD's signed state knows which task is running, but generated waypoint fields and
several consumers still treat an old `exactNextCommand` as the work selector.
After a task completes, status can therefore retain null change/task fields or
point at old operator prose. Context compression and process restart then lose
the actual execution position. Boundary progress also has no deterministic,
outage-safe record tied to canonical KBD identity.

## What Changes

- Derive the active and next change/task cursor from canonical task state after
  every phase, change, and task mutation.
- Preserve exact-next operator text as historical intent and remove it from
  work selection, reminders, gates, renderers, status, and resume paths.
- Resolve repeated task IDs and titles with qualified `change-id/task-id`
  identities at task completion and bottleneck boundaries.
- Regenerate consistent JSON, Markdown, and reminder projections for nested
  phases, top-level reset, child exit, restart, and resume.
- Add `karpathy-progress-memory`, an idempotent canonical boundary recorder with
  project-scoped `pk` ingestion and durable outbox fallback.
- Fire completion memory only after successful task, change, and phase
  transitions.

## Capabilities

### New Capabilities

- `kbd-progress-continuity`: Defines canonical next-work derivation, qualified
  boundary identity, projection agreement, and durable progress-memory records.

### Modified Capabilities

None.

## Impact

This changes `kbd-runtime`, the Prometheus KBD CLI, KBD boundary drivers and
renderers, and process-skill distribution. It adds no external dependency and
does not make generated KBD projections writable by skills or agents.
