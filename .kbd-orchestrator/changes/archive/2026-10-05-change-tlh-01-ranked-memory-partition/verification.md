# Verification — change-tlh-01-ranked-memory-partition

Repository: `prometheus-skill-system`
Depends on: none

## Acceptance criteria

- A fixture index whose active-phase entry is the last line and whose first lines are `archive-*` keeps the active-phase entry and moves the archives first.
- `feedback`/GLOBAL entries are kept ahead of older `project` entries when the budget forces a choice, and the kept rank-2 entries appear newest first (asserted on the output order).
- Running the partition twice on its own output moves nothing the second time (idempotent) and the index is <= `--limit` bytes.
- Scratch HOME only; the operator's real MEMORY.md is never read or written by the test.

## Verify commands

Exit 0 = pass, 1 = fail, 2 = BLOCKED (a prerequisite is absent; never reported as a pass).

```verify
/bin/bash scripts/tests/test-memory-partition.sh
npm run check:distribution
```
