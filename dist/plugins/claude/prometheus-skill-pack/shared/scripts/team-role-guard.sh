#!/bin/bash
# team-role-guard.sh — run a SubagentStop hook target only for the right agents.
#
#   team-role-guard.sh <target> [args...]                  skip team roles
#   team-role-guard.sh --only-team-roles <target> [args...] run ONLY for team roles
#
# Codex reports the bare agent name (`planner`), so a Codex matcher cannot tell
# an iterative-evolver agent from an agent-team role with the same name. This
# guard reads the hook payload, asks agent_identity.py whether `agent_type`
# resolves to a role in the active team, and either skips silently (exit 0) or
# runs the real target with the same arguments and the same stdin.
#
# <target> is a bundle-relative path, validated like hook-dispatch-v1.sh does.
# Never blocks a hook chain on its own failure: an unreadable payload is treated
# as "not a team role". bash 3.2 compatible.

set -u

mode="skip-team-roles"
if [ "${1:-}" = "--only-team-roles" ]; then
  mode="only-team-roles"
  shift
fi
relative="${1:-}"
[ -n "$relative" ] || { printf 'team-role-guard: missing target\n' >&2; exit 64; }
shift
case "$relative" in
  /*|../*|*/../*) printf 'team-role-guard: unsafe target: %s\n' "$relative" >&2; exit 65 ;;
esac

BUNDLE_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd -P)"
target="$BUNDLE_ROOT/$relative"
[ -f "$target" ] || { printf 'team-role-guard: missing target: %s\n' "$relative" >&2; exit 66; }

payload_file="$(mktemp "${TMPDIR:-/tmp}/team-role-guard.XXXXXX")"
trap 'rm -f "$payload_file"' EXIT
cat > "$payload_file"

is_team_role=1
if command -v python3 >/dev/null 2>&1; then
  agent_type="$(python3 -c 'import json,sys
try:
    d=json.load(open(sys.argv[1]))
    print(d.get("agent_type","") if isinstance(d,dict) else "")
except Exception:
    print("")' "$payload_file" 2>/dev/null)"
  cwd="$(python3 -c 'import json,sys
try:
    d=json.load(open(sys.argv[1]))
    print(d.get("cwd","") if isinstance(d,dict) else "")
except Exception:
    print("")' "$payload_file" 2>/dev/null)"
  [ -d "$cwd" ] || cwd="$PWD"
  if [ -n "$agent_type" ] && \
     python3 "$BUNDLE_ROOT/shared/scripts/lib/agent_identity.py" --cwd "$cwd" --is-team-role "$agent_type" >/dev/null 2>&1; then
    is_team_role=0
  fi
fi

if [ "$mode" = "skip-team-roles" ] && [ "$is_team_role" -eq 0 ]; then
  exit 0
fi
if [ "$mode" = "only-team-roles" ] && [ "$is_team_role" -ne 0 ]; then
  exit 0
fi
bash "$target" "$@" < "$payload_file"
