#!/usr/bin/env bash
# Run from the consumer project root. Hook delivery never authorizes closure.
set -uo pipefail
die() { printf 'kbd-reflect: %s\n' "$*" >&2; exit 2; }
mode=${1:-}; phase=${2:-}
case "$mode" in reflect|phase-after) ;; *) die 'usage: finalize-reflection.sh reflect|phase-after PHASE' ;; esac
[[ -n "$phase" && "$phase" != *'/'* && "$phase" != *'\'* ]] || die "invalid phase ID"
export KBD_ORCHESTRATOR_ROOT="$(cd "$(dirname "$0")/../.." && pwd -P)"
dir="$(node "$KBD_ORCHESTRATOR_ROOT/shared/method-context.mjs" --project . --phase "$phase" --phase-dir)" || die "unknown phase directory for $phase"
[[ -d "$dir" ]] || die "missing phase directory $dir"
lib="$KBD_ORCHESTRATOR_ROOT/shared/lib"
for file in hooks waypoint bottleneck-guard stage-gate phase-disposition; do
  [[ -r "$lib/$file.sh" ]] || die "installation missing $lib/$file.sh"
  . "$lib/$file.sh" || die "cannot load $lib/$file.sh"
done
view="$(prometheus kbd --path . phase disposition --id "$phase" --json)" || die "cannot read canonical disposition for $phase"
token="$(jq -er .disposition <<<"$view")" || die 'invalid disposition'
export KBD_HOOK_PHASE_DIR="$dir" KBD_HOOK_PHASE_ID="$phase"
export KBD_HOOK_DISPOSITION="$token"

dispatch() {
  # Serialize delivery by phase across shells and release automatically on exit.
  KBD_HOOK_DELIVERY_KEY="$2" python3 - "$KBD_HOOK_PHASE_DIR" "$lib" "$1" "$phase" <<'PY'
import fcntl
import subprocess
import sys
from pathlib import Path

directory, lib, kind, phase = sys.argv[1:]
with (Path(directory) / ".after-hooks.lock").open("a") as lock:
    fcntl.flock(lock, fcntl.LOCK_EX)
    result = subprocess.run(["bash", "-c",
        '. "$1/hooks.sh" && kbd_hooks_fire "$2" after "$3" 1 1',
        "kbd-after", lib, kind, phase], pass_fds=(lock.fileno(),))
    sys.exit(result.returncode)
PY
}

if [[ "$mode" == phase-after ]]; then
  case "$token" in
    CLOSED|CLOSED_SUPERSEDED|CLOSED_UNMET)
      key="$(jq -er '.closure.closureKey' <<<"$view")" || die 'missing canonical closure identity' ;;
    UNBOUND)
      [[ "$(jq -r .phaseStatus <<<"$view")" == complete ]] || die "legacy phase is still open"
      key="$(jq -er '[.projectId,.runId,.phaseId,"complete"]|@json' <<<"$view")" || exit 2 ;;
    *) die "phase $phase has disposition $token" ;;
  esac
  dispatch phase "$key"
  exit $?
fi

report="$KBD_HOOK_PHASE_DIR/reflection.md"
[[ -s "$report" ]] || die "missing reflection $report"
# A signed exit may explicitly withdraw the entire execute stage. The runtime
# has already sealed that scope; its reflection describes the recorded outcome.
case "$token" in
  CLOSED_SUPERSEDED|CLOSED_UNMET) ;;
  *) kbd_stage_gate reflect "$KBD_HOOK_PHASE_DIR" || exit 2 ;;
esac
if [[ -f "$dir/progress.json" ]]; then
  jq -e '(.reflect_gate // "") != "rejected"' "$dir/progress.json" >/dev/null || die 'reflection rejected or progress unreadable'
fi
pack="$(kbd_package_root)"
review="$pack/shared/scripts/sycophancy-check-artifact.sh"
[[ -f "$review" ]] || die "installation missing $review"
jq -nc --arg file "$report" '{tool_input:{file_path:$file}}' | bash "$review" || die 'reflection review failed'
state="$(prometheus kbd --path . status --json)" || die 'cannot read review state'
for file in "$report" "${report#"$PWD/"}"; do
  key="$(printf '%s' "$file" | shasum | cut -c1-16)"
  jq -e --arg id "sycophancy:$key" '.blockers[$id].resolved != false' <<<"$state" >/dev/null || die 'reflection has an unresolved review blocker'
done
key="$(shasum -a 256 < "$report")" || die 'cannot read reflection revision'
dispatch reflect "${key%% *}" || exit $?
case "$token" in
  CLOSED|CLOSED_SUPERSEDED|CLOSED_UNMET) ;;
  UNBOUND)
    guard=0
    if kbd_bottleneck_active; then
      guard=1
      kbd_bottleneck_evaluate phase after "$phase" 1 >/dev/null || exit 2
    fi
    prometheus kbd --path . phase transition --command-id "reflect-complete:$phase" --id "$phase" --status complete || exit $?
    if [[ "$guard" == 1 ]]; then kbd_bottleneck_evaluate phase after "$phase" 0 >/dev/null || exit 2; fi ;;
  *) exec bash "$pack/shared/scripts/kbd-phase-close-eval.sh" --phase "$phase" ;;
esac
exec bash "$0" phase-after "$phase"
