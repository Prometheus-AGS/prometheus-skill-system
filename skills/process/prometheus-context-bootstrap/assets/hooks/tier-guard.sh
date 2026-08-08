#!/usr/bin/env bash
# PreToolUse:Bash — block Tier 3 commands outside a milestone phase.
#
# Contract: exit 0 allows, exit 2 blocks and feeds stderr back to the model.
# Any other exit is a hook error and does not block, so every failure path
# below exits 0 deliberately. A guard that blocks because jq is missing is
# worse than one that does not fire.

set -uo pipefail

input="$(cat 2>/dev/null || true)"

# Extract the command. Prefer jq; fall back to sed so a missing jq degrades
# to a weaker check rather than to no check at all.
if command -v jq >/dev/null 2>&1; then
  cmd="$(printf '%s' "$input" | jq -r '.tool_input.command // ""' 2>/dev/null || true)"
else
  cmd="$(printf '%s' "$input" | sed -n 's/.*"command"[[:space:]]*:[[:space:]]*"\(.*\)".*/\1/p' | head -1)"
fi
[[ -n "$cmd" ]] || exit 0

root="${CLAUDE_PROJECT_DIR:-$PWD}"
waypoint="$root/.kbd-orchestrator/current-waypoint.json"

phase="implement"
if [[ -f "$waypoint" ]] && command -v jq >/dev/null 2>&1; then
  phase="$(jq -r '.phase // "implement"' "$waypoint" 2>/dev/null || echo implement)"
fi

# Tier 3: expensive, cache-invalidating, or device-bound.
t3='cargo build.*--release|cargo test.*--release|tauri build|flutter build (ios|apk|appbundle)|go test.*-race|wasm-pack test --headless|playwright test'

if printf '%s' "$cmd" | grep -Eq "$t3"; then
  case "$phase" in
    milestone|release|certify) exit 0 ;;
    *)
      cat >&2 <<EOF
TIER VIOLATION — this is a Tier 3 command and the current phase is "$phase".

  command: $cmd
  source:  $waypoint

Tier 3 runs at milestone or release only. A release build during
implementation invalidates the incremental cache and pays full optimization
for code that is about to change.

Run the Tier 2 equivalent, or set the waypoint phase to "milestone" if this
genuinely is the gate.
EOF
      exit 2
      ;;
  esac
fi

exit 0
