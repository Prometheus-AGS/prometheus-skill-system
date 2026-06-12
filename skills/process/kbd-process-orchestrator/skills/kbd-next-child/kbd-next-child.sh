#!/usr/bin/env bash
# skills/kbd-next-child/kbd-next-child.sh
# Advance childPointer to the next child of the active parent phase.

set -euo pipefail
die()  { printf 'kbd-next-child: %s\n' "$*" >&2; exit 1; }
warn() { printf 'kbd-next-child: warn: %s\n' "$*" >&2; }

target="${1:-}"
command -v jq >/dev/null 2>&1 || die "jq is required"

wp=".kbd-orchestrator/current-waypoint.json"
[[ -f "$wp" ]] || die "no current-waypoint.json — run /kbd-new-phase first"
jq -e . "$wp" >/dev/null 2>&1 || die "malformed waypoint at $wp"

parent="$(jq -r '.phase // ""' "$wp")"
[[ -n "$parent" ]] || die "no active phase"

# Children as newline-separated list
children="$(jq -r '.childPhases // [] | .[]' "$wp")"
[[ -n "$children" ]] || die "no children defined — run /kbd-new-child first"
total="$(printf '%s\n' "$children" | wc -l | tr -d ' ')"

prior="$(jq -r '.childPointer // ""' "$wp")"

# Resolve next
if [[ -n "$target" ]]; then
  if ! printf '%s\n' "$children" | grep -qFx "$target"; then
    avail="$(printf '%s\n' "$children" | tr '\n' ' ')"
    die "no such child: $target (available: $avail)"
  fi
  next="$target"
else
  if [[ -z "$prior" ]]; then
    next="$(printf '%s\n' "$children" | head -n 1)"
  else
    next="$(printf '%s\n' "$children" | awk -v p="$prior" 'seen{print;exit} $0==p{seen=1}')"
    [[ -n "$next" ]] || die "already on last child '$prior' — run /kbd-reflect, then /kbd-next-phase"
  fi
fi

# Hook subsystem
KBD_ORCHESTRATOR_ROOT="${KBD_ORCHESTRATOR_ROOT:-$HOME/.claude/skills/kbd-process-orchestrator}"
export KBD_ORCHESTRATOR_ROOT
hooks_avail=0
if [[ -f "$KBD_ORCHESTRATOR_ROOT/shared/lib/hooks.sh" && -f "$KBD_ORCHESTRATOR_ROOT/shared/lib/waypoint.sh" ]]; then
  # shellcheck source=/dev/null
  . "$KBD_ORCHESTRATOR_ROOT/shared/lib/waypoint.sh"
  # shellcheck source=/dev/null
  . "$KBD_ORCHESTRATOR_ROOT/shared/lib/hooks.sh"
  hooks_avail=1
else
  warn "hooks subsystem unavailable (continuing without hook fires)"
fi

from_label="(none)"
if [[ -n "$prior" ]]; then
  from_label="$prior"
  if [[ "$hooks_avail" == "1" ]]; then
    prior_index="$(printf '%s\n' "$children" | grep -nFx "$prior" | head -1 | cut -d: -f1)"
    kbd_hooks_fire child after "$prior" "$prior_index" "$total" \
      || warn "child:after hook fire failed"
  fi
fi

# Atomic waypoint update. Keep path[] consistent: switching siblings at the
# active parent re-points the trailing element to the new child. We rebuild
# path[] as <parent-chain-without-trailing-pointer> + [next].
now="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
new_path_json="null"
if [[ "$hooks_avail" == "1" ]]; then
  full_chain="$(_kbd_path_from_waypoint "$wp" 2>/dev/null || true)"
  if [[ -n "$full_chain" ]]; then
    # shellcheck disable=SC2206
    ft=($full_chain); fd="${#ft[@]}"
    if [[ "$fd" -gt 1 && "${ft[$((fd-1))]}" == "$prior" ]]; then
      base=("${ft[@]:0:$((fd-1))}")
    else
      base=("${ft[@]}")
    fi
    base_chain="$(printf '%s ' "${base[@]}")"; base_chain="${base_chain% }"
    new_path_json="$(jq -cn --argjson b "$(printf '%s' "$base_chain" | jq -R 'split(" ")')" --arg n "$next" '$b + [$n]')"
  fi
fi
jq --arg ptr "$next" --arg parent "$parent" --arg now "$now" --argjson np "$new_path_json" '
  .childPointer     = $ptr |
  (if $np != null then .path = $np else . end) |
  .currentTask      = ("run kbd-assess for " + $parent + "/" + $ptr) |
  .exactNextCommand = ("/kbd-assess " + $parent + "/" + $ptr) |
  .updatedAt        = $now
' "$wp" > "$wp.tmp"
mv -f "$wp.tmp" "$wp"

# child:before for new active child
if [[ "$hooks_avail" == "1" ]]; then
  next_index="$(printf '%s\n' "$children" | grep -nFx "$next" | head -1 | cut -d: -f1)"
  kbd_hooks_fire child before "$next" "$next_index" "$total" \
    || warn "child:before hook fire failed"
fi

printf '\nCompleted kbd-next-child — now on %s/%s\n' "$parent" "$next"
printf '  from: %s\n' "$from_label"
printf '  to:   %s\n' "$next"
printf '  Next: /kbd-assess %s/%s\n' "$parent" "$next"
