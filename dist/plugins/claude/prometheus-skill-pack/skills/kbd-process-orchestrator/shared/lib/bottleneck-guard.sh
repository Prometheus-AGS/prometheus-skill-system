#!/usr/bin/env bash

# Portable adapter for canonical boundary evaluation. The runtime CLI owns all
# policy and state mutation; lifecycle scripts only decide where boundaries sit.

kbd_bottleneck_available() {
  command -v prometheus >/dev/null 2>&1 || return 1
  prometheus kbd --help 2>/dev/null | grep -qE '^[[:space:]]+guard[[:space:]]'
}

kbd_bottleneck_active() {
  local root="${KBD_BOTTLENECK_PATH:-.}"
  kbd_bottleneck_available || return 1
  [ -f "$root/.kbd-orchestrator/current-waypoint.json" ] || return 1
  [ "$(jq -r '.generatedBy // empty' "$root/.kbd-orchestrator/current-waypoint.json" 2>/dev/null)" = "kbd-runtime" ] || return 1
  prometheus kbd --path "$root" status --json >/dev/null 2>&1
}

kbd_bottleneck_evaluate() {
  # kbd_bottleneck_evaluate <task|phase|zeespec> <before|after> <subject> <precommit 0|1>
  local boundary="$1" edge="$2" subject="$3" precommit="${4:-0}"
  local root="${KBD_BOTTLENECK_PATH:-.}"
  kbd_bottleneck_available || return 2
  local args=(kbd --path "$root" guard evaluate
    --boundary "$boundary" --edge "$edge" --subject "$subject"
    --json --repair-projections)
  [ "$precommit" = "1" ] && args+=(--precommit)
  prometheus "${args[@]}"
}

kbd_bottleneck_print_signal() {
  local output="$1"
  printf '%s\n' "$output" | jq -r '
    .exactSignal,
    ("Position: " + .position + " @ revision " + (.authoritativeRevision | tostring))
  '
}
