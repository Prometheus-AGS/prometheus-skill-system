# Goals: phase-team-learning-hardening

Seeded from: `team-aware-learning-memory-impl/reflection.md`
Created: 2026-10-04T22:47:27Z

## Seeded Goals

`phase-team-learning-hardening`. Top three priorities:

1. **Install-time automation of this phase's manual steps:**
   - priority-ranked MEMORY.md partitioning in `memory-index-partition.py`;
   - the Codex `[memories] generate_memories = false` setting applied and checked by the installer and the doctor;
   - a versioned, fail-loud cadence refresh procedure.
2. **Test isolation and load robustness:**
   - move `memory-envelope.integration` to a scratch server;
   - reduce B4's repeated query embedding so SubagentStart recall meets its watchdog under load;
   - add a ledger/tasks reconciliation check to the cadence iteration.
3. **Real-world verification:**
   - the Cortex mirror against real Cortex;
   - recall quality in `prior-context.md` (it surfaced no applicable lessons this phase);
   - a `rebase-regenerate` helper for generated-only conflicts.

---

## Instructions

Review and refine the goals above before running `/kbd-assess`.
Add, remove, or clarify as needed. When ready:

```
/kbd-assess phase-team-learning-hardening
```
