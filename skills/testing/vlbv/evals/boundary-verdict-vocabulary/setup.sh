#!/usr/bin/env bash
set -eu
# A minimal KBD-managed project: the session hook keys off this directory.
mkdir -p .kbd-orchestrator/phases/phase-3-invoicing
cat > .kbd-orchestrator/project.json <<'JSON'
{ "name": "orbit-billing", "activePhase": "phase-3-invoicing", "specBackend": "native-kbd", "verificationMethod": "VLBV" }
JSON
cat > .kbd-orchestrator/current-waypoint.json <<'JSON'
{ "phase": "phase-3-invoicing", "change": "change-012-invoice-export", "status": "executing", "updatedAt": "2026-11-10T14:00:00Z" }
JSON
