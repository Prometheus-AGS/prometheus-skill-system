#!/usr/bin/env bash
set -eu
mkdir -p .kbd-orchestrator/phases/phase-3-invoicing
cat > .kbd-orchestrator/project.json <<'JSON'
{ "name": "orbit-billing", "activePhase": "phase-3-invoicing", "specBackend": "native-kbd" }
JSON
cat > .kbd-orchestrator/current-waypoint.json <<'JSON'
{ "phase": "phase-3-invoicing", "previousPhase": "phase-2-customers", "change": "change-012-proration",
  "status": "executing", "sourceTool": "codex", "exactNextCommand": "/kbd-apply change-012-proration",
  "parentPhase": null, "childPhases": [], "childPointer": null, "updatedAt": "2026-11-10T14:00:00Z" }
JSON
cat > .kbd-orchestrator/phases/phase-3-invoicing/progress.json <<'JSON'
{ "phase": "phase-3-invoicing", "last_updated": "2026-11-10T14:00:00Z", "last_updated_by": "codex",
  "completion": { "primaryCounter": "implementation",
    "implementation": { "completed": 2, "total": 4, "status": "IN_PROGRESS" },
    "certification": { "status": "PENDING", "summary": null, "blockers": [] } },
  "changes": {
    "change-010-invoice-model": { "status": "DONE", "completed_by": "antigravity" },
    "change-011-tax-lines": { "status": "DONE", "completed_by": "codex" },
    "change-012-proration": { "status": "IN_PROGRESS", "started_by": "codex" },
    "change-013-pdf-export": { "status": "PENDING" } } }
JSON
