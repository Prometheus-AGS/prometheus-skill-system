# /kbd-next-phase

Continue to the next KBD phase, automatically seeded from the previous phase's reflection.

## Instructions

0. Check the canonical disposition of the current phase:
   `prometheus kbd phase disposition --id <current-phase>`.
   - `UNBOUND` → legacy phase; continue with step 1.
   - `CLOSED` → continue only if the operator supplied a signed start-successor
     manifest and signature; pass them as
     `--authorize-successor <manifest> --signature <sig>`. Otherwise stop and
     show the operator the commands the script prints (exit 4). Never write or
     sign a manifest yourself; closure alone never starts a successor.
   - `OPEN`, `REFUSED_BLOCKED`, `REFUSED_FAIL` → stop: the phase is not closed.
     Its terminal evaluator decides closure
     (`shared/scripts/kbd-phase-close-eval.sh --phase <current-phase>`).

1. Read `.kbd-orchestrator/current-waypoint.json`.
   - Note the current `phase` and `stage`.
   - If `stage` is not `reflect_complete`, warn the user:
     > "The current phase has not completed reflection. It is recommended to run
     > `/kbd-reflect` first. Proceeding will seed the next phase from whatever
     > reflection content currently exists."

2. Read `.kbd-orchestrator/phases/<current-phase>/reflection.md`.
   - If it does not exist, tell the user to run `/kbd-reflect` first and stop.

3. Run the next-phase seed script:

   ```bash
   bash "${CLAUDE_PLUGIN_ROOT:-$(git -C "$(pwd)" rev-parse --show-toplevel 2>/dev/null)}/skills/process/kbd-process-orchestrator/skills/kbd-next-phase/scripts/kbd-next-phase.sh" $ARGUMENTS
   ```

   - If `$ARGUMENTS` is provided, it is used as the new phase name.
   - If `$ARGUMENTS` is empty, the script extracts the suggested name from the
     reflection's "Recommended Next Phase" section.

4. Print the full output from the script (it contains the confirmation banner).

5. Read `.kbd-orchestrator/phases/<new-phase>/goals.md` and display its contents
   so the user can review and confirm the seeded goals.

6. Remind the user:
   > "Review the goals in `.kbd-orchestrator/phases/<new-phase>/goals.md`, edit
   > if needed, then run `/kbd-assess` to begin the new phase."

## Error cases

- `current-waypoint.json` missing → instruct user to run `/kbd-init` first.
- `reflection.md` missing → instruct user to run `/kbd-reflect` first.
- Proposed phase name already exists → the script will error; suggest an alternate name.
- Exit 2 `NOT-CLOSED` → the contract-bound phase has no PASS closure; nothing was written.
- Exit 4 `SUCCESSOR-UNAUTHORIZED` → no (valid) signed start-successor manifest; nothing was written.
- `ALREADY-STARTED` → the authorized successor already exists; resume it with `/kbd-assess`.

## Examples

```
/kbd-next-phase                                    # name extracted from reflection
/kbd-next-phase skill-pack-upgrade-phase-2         # explicit name
/kbd-next-phase phase-3-realtime-sync-2026-05-26   # explicit name with date
```
