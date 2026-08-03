#!/usr/bin/env bash
# memory-writeback.sh — persist an accepted phase reflection to surreal-memory.
#
# Dual use:
#   1. As the orchestrator reflect:end builtin action (no stdin) — resolves the
#      active phase from the waypoint and persists its reflection.md.
#   2. As a Claude Code PostToolUse(Write|Edit) hook (stdin JSON) — fires only
#      when the written file is a reflection.md whose phase progress.json has no
#      reflect_gate=rejected, so a rejected reflection is never persisted.
#
# Extracts the Delta + Root Cause + Corrective Actions sections (the durable
# learning) and routes [GLOBAL]-marked lines to user_id=global. Always exits 0.
set -uo pipefail

HERE="$(cd "$(dirname "$0")" && pwd)"
HOOK_LOG_LIB="$HERE/lib/hook-log.sh"
[ -f "$HOOK_LOG_LIB" ] && source "$HOOK_LOG_LIB"
hook_log_start "PostToolUse" "memory-writeback.sh"
# shellcheck source=/dev/null
source "$HERE/lib/memory-bridge.sh"
finish() { hook_log_end 0; exit 0; }
command -v python3 >/dev/null 2>&1 || finish

# Resolve the reflection.md to persist.
REFLECTION=""
if [ -t 0 ]; then
  : # no stdin (orchestrator action path) — resolve from waypoint below
else
  INPUT="$(cat 2>/dev/null || true)"
  if [ -n "$INPUT" ]; then
    FP="$(printf '%s' "$INPUT" | python3 -c "
import sys, json
try: d = json.load(sys.stdin)
except Exception: print(''); raise SystemExit
ti = d.get('tool_input', {}) or {}
print(ti.get('file_path') or ti.get('path') or '')
" 2>/dev/null || true)"
    case "$FP" in
      */reflection.md|reflection.md) REFLECTION="$FP" ;;
      *) finish ;;   # PostToolUse on a non-reflection file → nothing to do
    esac
  fi
fi

# Locate orchestrator root.
_root() {
  local dir="$PWD"
  while [ -n "$dir" ] && [ "$dir" != "/" ]; do
    [ -d "$dir/.kbd-orchestrator" ] && { printf '%s' "$dir"; return 0; }
    dir="$(dirname "$dir")"
  done
  return 1
}
ROOT="$(_root)" || finish

# Orchestrator-action path: resolve active phase's reflection.md.
if [ -z "$REFLECTION" ]; then
  WP="$ROOT/.kbd-orchestrator/current-waypoint.json"
  [ -f "$WP" ] || finish
  PHASE="$(jq -r '.phase // empty' "$WP" 2>/dev/null)"
  [ -n "$PHASE" ] || finish
  REFLECTION="$ROOT/.kbd-orchestrator/phases/$PHASE/reflection.md"
fi
[ -f "$REFLECTION" ] || finish

# Gate: never persist a rejected reflection.
PROGRESS="$(dirname "$REFLECTION")/progress.json"
if [ -f "$PROGRESS" ]; then
  GATE="$(jq -r '.reflect_gate // ""' "$PROGRESS" 2>/dev/null || true)"
  [ "$GATE" = "rejected" ] && {
    echo "[memory-writeback] skip: reflect_gate=rejected — not persisting" >&2
    finish
  }
fi

# Extract the Delta + Root Cause + Corrective Actions sections as the learning
# payload.
PAYLOAD="$(python3 - "$REFLECTION" <<'PY' 2>/dev/null || true
import sys, re
text = open(sys.argv[1], encoding="utf-8", errors="replace").read()
def section(name):
    m = re.search(r'(?ims)^##\s+' + re.escape(name) + r'\s*$(.*?)(?=^##\s|\Z)', text)
    return (m.group(1).strip() if m else "")
delta = section("Delta")
root_cause = section("Root Cause")
ca = section("Corrective Actions")
phase = ""
m = re.search(r'(?im)^#\s+Reflection\s+[—-]\s+(.+)$', text)
if m: phase = m.group(1).strip()
out = []
if phase: out.append(f"Phase {phase} reflection learnings:")
if delta: out.append("Deltas:\n" + delta)
if root_cause: out.append("Root causes:\n" + root_cause)
if ca: out.append("Corrective actions:\n" + ca)
print("\n\n".join(out))
PY
)"
[ -n "$PAYLOAD" ] || finish

# Route [GLOBAL]-marked corrective actions to global scope; the rest to project.
SCOPE="$MEM_PROJECT"
case "$PAYLOAD" in
  *"[GLOBAL]"*) SCOPE="global" ;;
esac
mem_add_memory "$PAYLOAD" "$SCOPE"
echo "[memory-writeback] persisted reflection from $(basename "$(dirname "$REFLECTION")")" >&2
finish
