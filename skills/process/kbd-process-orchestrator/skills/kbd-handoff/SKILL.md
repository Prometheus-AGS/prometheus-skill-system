---
license: MIT
name: kbd-handoff
version: '1.0.0'
argument-hint: '--to <harness>'
description: >
  Atomically transfer the active KBD mutation lease to another harness while
  preserving the causal checkpoint and fencing stale writers.
metadata:
  tags: [process, orchestration, control, handoff]
---

# /kbd-handoff

Transfer execution ownership without changing the planned next work.

## Progress Signals (MANDATORY)

Before handoff, emit:

```text
Starting kbd-handoff — <phase-name> to <harness>
```

After the new owner is visible, emit:

```text
Completed kbd-handoff — <phase-name> owned by <harness>
```

## Procedure

1. Require an explicit target harness.
2. Pause and checkpoint the active run if it is still running.
3. Run `prometheus kbd handoff --to <harness>`.
4. Verify that the lease ID changed, the fencing token increased, and the
   former owner can no longer mutate state.
5. Report the run ID, revision, plan revision, exact next work, new owner, and
   lease expiry. The receiving harness resumes explicitly.
6. While it is the writer, the receiving adapter runs `prometheus kbd
   heartbeat` every 30 seconds. The 90-second TTL permits takeover only after
   three missed heartbeat intervals.

Never implement handoff by asking two tools to edit the same JSON file.
