# Waypoint .phase goes stale at reflect — recurring, root cause found
Observed 2026-07-31, twice: at the close of ideation-and-decision-tools
AND again at the close of mobile-skill-portability.

## Symptom
After /kbd-reflect, current-waypoint.json still names the phase from TWO
phases ago. Both times it read `adversarial-review-for-creation` — the
last phase for which /kbd-next-phase had actually run.

## Root cause
`.phase` is written by exactly one place:
```
  ~/.claude/skills/kbd-next-phase/scripts/kbd-next-phase.sh:270
    .phase = $phase | .previousPhase = $previous |
```
Nothing in kbd-reflect sets it:
```console
$ grep -rn '\.phase\s*=' ~/.claude/skills/kbd-reflect/
(no matches)
```

So the field is only ever correct if /kbd-next-phase ran for the CURRENT
phase. When a phase is created some other way, or when reflect closes a
phase before next-phase runs again, `.phase` silently describes history.

## Why it matters
position-reminder.txt and every skill's 'FIRST tool call' instruction tell
an agent to trust this file for its position. A stale `.phase` points the
next session at the wrong phase directory.

## Not fixed here, deliberately
The fix belongs in the kbd-reflect skill (set .phase alongside .status).
That skill is INSTALLED under ~/.claude/skills/, not part of this repo —
editing it from here would be an untracked change to another package,
the same class of mistake as editing a plugin cache.
