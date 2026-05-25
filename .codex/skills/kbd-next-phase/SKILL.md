---
name: kbd-next-phase
description: Continue to the next KBD phase, seeded automatically from the previous phase's reflection. Reads the "Recommended Next Phase" section of reflection.md, initializes the new phase directory, and prepares the waypoint for /kbd-assess.
license: MIT
compatibility: Requires .kbd-orchestrator/ directory (run /kbd-init first).
metadata:
  author: prometheus-ags
  version: "1.0.0"
---

Continue to the next KBD phase, automatically seeded from the previous phase's reflection.

## Instructions

1. Read `.kbd-orchestrator/current-waypoint.json`.
   - Note the current `phase` and `stage`.
   - If `stage` is not `reflect_complete`, warn: "The current phase has not completed reflection.
     It is recommended to run `/kbd-reflect` first." Proceed only if confirmed.

2. Check that `.kbd-orchestrator/phases/<current-phase>/reflection.md` exists.
   If not, stop and instruct the user to run `/kbd-reflect` first.

3. Run the next-phase seed script:

   ```bash
   bash "$(git rev-parse --show-toplevel)/.claude-plugin/shared/scripts/kbd-next-phase.sh" $ARGUMENTS
   ```

   - If arguments were provided, use them as the new phase name.
   - If no arguments, the script extracts the suggested name from the reflection.

4. Print the full script output (it contains the confirmation banner).

5. Read `.kbd-orchestrator/phases/<new-phase>/goals.md` and display its contents.

6. Remind the user: "Review the goals, edit if needed, then run `/kbd-assess` to begin."

## Examples

```
/kbd-next-phase                                    # name from reflection
/kbd-next-phase skill-pack-upgrade-phase-2         # explicit name
```
