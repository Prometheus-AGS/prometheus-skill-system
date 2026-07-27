---
id: hooks-and-waypoints
title: Hooks & Waypoints
---

# Hooks & Waypoints

**Hooks** — every stage fires `<stage>:before` / `<stage>:after` via
`shared/lib/hooks.sh`; per-task boundaries fire `task:before`/`task:after`
from `/kbd-apply`. Projects wire custom hooks per phase without touching the
skills.

**Waypoints** — `current-waypoint.json` records phase, status, and the next
command, so a session (or a different tool entirely) resumes with one read.
`position-reminder.txt` mirrors it as plain text for prompt injection.

**Stage gates** — each stage requires its predecessor's handoff
(`kbd_stage_gate`), and writes its own (`kbd_stage_handoff_write`) — a
1–3-sentence summary plus artifact list the next stage reads first.

*Canonical source: [`shared/lib`](https://github.com/Prometheus-AGS/prometheus-skill-system/tree/main/skills/process/kbd-process-orchestrator/shared/lib) (`hooks.sh`,
`stage-gate.sh`, `waypoint.sh`, `progress.sh`).*
