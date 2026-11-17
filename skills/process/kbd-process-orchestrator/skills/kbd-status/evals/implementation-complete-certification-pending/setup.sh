#!/usr/bin/env bash
set -eu
mkdir -p .kbd-orchestrator/phases/phase-3-invoicing
cat > .kbd-orchestrator/project.json <<'JSON'
{ "name": "orbit-billing", "activePhase": "phase-3-invoicing", "specBackend": "native-kbd" }
JSON
cat > .kbd-orchestrator/current-waypoint.json <<'JSON'
{ "phase": "phase-3-invoicing", "previousPhase": "phase-2-customers", "change": null,
  "status": "reflect_ready", "sourceTool": "codex", "exactNextCommand": "/kbd-reflect",
  "parentPhase": null, "childPhases": [], "childPointer": null, "updatedAt": "2026-11-10T14:00:00Z" }
JSON
cat > .kbd-orchestrator/phases/phase-3-invoicing/progress.json <<'JSON'
{ "phase": "phase-3-invoicing", "last_updated": "2026-11-10T14:00:00Z", "last_updated_by": "codex",
  "changes_total": 4, "changes_completed": 0,
  "completion": { "primaryCounter": "implementation",
    "implementation": { "completed": 4, "total": 4, "status": "COMPLETE" },
    "certification": { "status": "PENDING", "summary": null, "blockers": [] } },
  "changes": {} }
JSON
