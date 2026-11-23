#!/usr/bin/env bash
set -eu
repo="$(cd "$(dirname "$0")/../../../.." && pwd -P)"
exec bash "$repo/skills/process/kbd-process-orchestrator/skills/kbd-next-phase/scripts/kbd-next-phase.sh" "$@"
