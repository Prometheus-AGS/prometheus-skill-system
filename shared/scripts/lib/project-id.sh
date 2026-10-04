#!/bin/bash
# project-id.sh — bash 3.2 wrapper around project_id.py, the single project-id
# resolver for every memory and learning path.
#
#   project-id.sh            -> prints the project id
#   project-id.sh --json     -> {"projectId","source","userScope"}
#   . project-id.sh; prometheus_project_id   (sourced: defines the function)
#
# Never fails the caller: without python3 it falls back to
# PROMETHEUS_PROJECT_ID, then project:unknown.

_PROMETHEUS_PROJECT_ID_LIB="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

prometheus_project_id() {
  if command -v python3 >/dev/null 2>&1; then
    python3 "$_PROMETHEUS_PROJECT_ID_LIB/project_id.py" "$@" 2>/dev/null && return 0
  fi
  if [ "${1:-}" = "--json" ]; then
    printf '{"projectId":"%s","source":"fallback","userScope":"user:unknown"}\n' "${PROMETHEUS_PROJECT_ID:-project:unknown}"
  else
    printf '%s\n' "${PROMETHEUS_PROJECT_ID:-project:unknown}"
  fi
  return 0
}

# Executed directly (not sourced): run the resolver.
if [ "${BASH_SOURCE[0]}" = "$0" ]; then
  prometheus_project_id "$@"
fi
