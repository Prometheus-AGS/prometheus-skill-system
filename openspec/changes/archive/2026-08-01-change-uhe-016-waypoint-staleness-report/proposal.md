# Detect waypoint staleness here, fix it upstream

**Change:** `change-uhe-016-waypoint-staleness-report`
**Phase:** uar-host-execution
**Goal:** S5

## Why

See `.kbd-orchestrator/phases/uar-host-execution/plan.md` for full rationale,
acceptance criteria, and the two-round adversarial review record.

## Outcome: one defect confirmed live, one already fixed upstream

### Defect 1 — `kbd-reflect` never writes `.phase` · **CONFIRMED, live**

```console
$ grep -rn '\.phase' ~/.claude/skills/kbd-reflect/*.sh
(no output)
```

Visible in this repo's own waypoint right now:

```
phase: adversarial-review-for-creation     ← closed, has reflection.md
                                              while uar-host-execution is open
```

**The one-line fix**, ready to apply where `kbd-reflect` is authored: after
writing `reflection.md`, update `current-waypoint.json`'s `.phase` to the next
active phase — the same `jq` assignment `kbd-next-phase.sh` already performs at
line 270 (`.phase = $phase`).

### Defect 2 — self-referential `next` · **ALREADY FIXED**

The plan cited `kbd-next-phase.sh:270` writing `/kbd-next-phase <phase>`.
Re-checked at execute time rather than trusted:

```console
$ grep -c '"/kbd-next-phase' ~/.claude/skills/kbd-next-phase/scripts/kbd-next-phase.sh
0
$ sed -n '275p' …/kbd-next-phase.sh
  .exactNextCommand = ("/kbd-assess " + $phase) |
```

Line 270 is now `.phase = $phase`, and `exactNextCommand` is correct. **The
defect the plan described no longer exists.** Reporting it as live would have
been a stale finding — the same class of error the change exists to detect.

The detector still covers it, because "fixed today" is not "cannot regress".

### The deliverable: `scripts/check-waypoint-staleness.sh`

A goal whose right answer is "do not patch here" still needs a change that
produces something. This is that something — **in this repo**, not in the
installed skills, because editing those is worse than useless: the next install
overwrites them, the change is invisible to git, and someone later fixes the same
bug again.

Three checks:

| # | Catches |
|---|---|
| 1 | `.phase` names a **closed** phase (has `reflection.md`) while another is open |
| 2 | `next` is self-referential — an agent following it re-runs the transition instead of advancing |
| 3 | `next` and `exactNextCommand` **disagree** — two fields answering one question differently means half the tools read each |

**Verified to discriminate, not merely to fail:**

```
live waypoint (stale)        -> exit 1, names the closed phase and the open one
synthetic clean waypoint     -> exit 0
synthetic self-referential   -> exit 1, catches BOTH check 2 and check 3
```

`/bin/bash -n` clean — bash 3.2 compatible, since macOS ships 3.2 and launchd
runs `/bin/bash`.

**Task 5 honoured: no installed skill was patched.**
