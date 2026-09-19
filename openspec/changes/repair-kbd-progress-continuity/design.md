## Context

The canonical journal already contains phases, changes, tasks, status, order,
and operator intent. Compatibility projections duplicate that state for mixed
harnesses. The observed failure came from treating the operator's old
`exactNextCommand` as live routing data while change/task fields remained null.
The same ambiguity appears when different changes use the same task ordinal or
title. Progress history currently depends on conversation context and a live
memory service, so neither restart nor an outage has a deterministic receipt.

## Goals / Non-Goals

**Goals:**

- Make task state the only next-work selector.
- Keep all compatibility views aligned after each typed mutation.
- Make repeated task IDs unambiguous at process boundaries.
- Persist one bounded, secret-free record for each successful completion
  boundary without blocking on memory availability.

**Non-Goals:**

- Making a compatibility projection authoritative or writable.
- Reconstructing elapsed time or verification evidence that a boundary driver
  did not supply.
- Treating successful memory delivery as a prerequisite for continued work.
- Changing the signed journal schema solely for progress-memory transport.

## Decisions

### Derive the work cursor during state folding

After phase activation and every change/task register or transition, sort open
work by declared sequence and stable ID, prefer in-progress then blocked work,
and update the active change/task cursor. Projection generation repeats the
derivation as a safety net. This makes replay and live mutation produce the
same cursor without parsing human prose.

### Preserve operator intent without routing from it

`exactNextWork` remains append-only operator context. Projections expose it as
`exactNextCommand`, but status and reminders label it as an operator note.
`nextChange` and `nextTask` are the only compatibility routing fields.

### Qualify task subjects at the boundary

Drivers send `change-id/task-id` to the guard and `change-id:task-id` to hooks.
Readers accept either separator, prefer the qualified identity, and use an
unqualified fallback only when it resolves uniquely. This preserves existing
event payloads while removing collisions.

### Record progress outside canonical projections

`karpathy-progress-memory` validates each versioned event against
`prometheus kbd status --json`, appends one marker-delimited record to
`.prometheus/session-log.md`, and writes a hashed receipt filename. The same
record enters `pk ingest --scope project`; a failed transport enters the
existing durable memory outbox. The recorder never mutates `.kbd-orchestrator`.

### Fire completion only after commit and guard success

Task hooks run after the canonical task transition and signed postcommit guard.
Change and phase drivers fire their dedicated after hook after the terminal
transition succeeds. Canonical status validation rejects premature or partial
completion records.

## Risks / Trade-offs

- [A project overrides or disables a hook] → Canonical state remains correct;
  the missing receipt is visible in hook status and phase evidence.
- [The process stops after the session-log append but before receipt creation]
  → The stable marker prevents a second append and the memory operation remains
  idempotent by event content.
- [A stale lock remains after process death] → The recorder reclaims only its
  own lock after a bounded 60-second stale interval.
- [A transition caller bypasses KBD drivers] → Direct CLI state remains
  canonical, while automatic progress memory is guaranteed only for the named
  boundary drivers.

## Migration Plan

Regenerate the skill indexes, manifests, documentation, and distributions from
source. Existing projects gain derived cursor fields on their next typed
mutation or projection repair. Existing exact-next text remains intact and is
relabelled as historical operator intent. No legacy progress record is
synthesized.
