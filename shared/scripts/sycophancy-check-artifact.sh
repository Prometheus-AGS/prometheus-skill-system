#!/usr/bin/env bash
# sycophancy-check-artifact.sh — PostToolUse(Write|Edit) gate for main-loop
# reflection/assessment artifacts (the reflector SubagentStop gate covers only
# subagent output; this covers files written directly by the main loop).
#
# PostToolUse cannot un-write the file, so teeth come from two mechanisms:
#   1. exit 2 with actionable Delta/Root-Cause/Corrective-Actions feedback so the
#      model rewrites the artifact;
#   2. set "reflect_gate":"rejected" in the phase progress.json — pipeline-enforce
#      then blocks /kbd-new-phase|/kbd-next-phase until a passing artifact clears it.
#
# Path filter: only acts on **/reflection.md and **/assessment.md.
# 2-rejection soft cap (per-artifact). Binary absent → exit 0 (graceful).
set -uo pipefail

HERE="$(cd "$(dirname "$0")" && pwd)"
HOOK_LOG_LIB="$HERE/lib/hook-log.sh"
[ -f "$HOOK_LOG_LIB" ] && source "$HOOK_LOG_LIB"
hook_log_start "PostToolUse" "sycophancy-check-artifact.sh"
# shellcheck source=/dev/null
source "$HERE/lib/sycophancy.sh"

finish() { hook_log_end "${1:-0}"; exit "${1:-0}"; }
command -v python3 >/dev/null 2>&1 || finish 0

INPUT="$(cat 2>/dev/null || true)"
[ -n "$INPUT" ] || finish 0
FILE_PATH="$(printf '%s' "$INPUT" | python3 -c "
import sys, json
try: d = json.load(sys.stdin)
except Exception: print(''); raise SystemExit
ti = d.get('tool_input', {}) or {}
print(ti.get('file_path') or ti.get('path') or '')
" 2>/dev/null || true)"
[ -n "$FILE_PATH" ] || finish 0

# Path filter.
case "$FILE_PATH" in
  */reflection.md|*/assessment.md|reflection.md|assessment.md) : ;;
  *) finish 0 ;;
esac
[ -f "$FILE_PATH" ] || finish 0

# Graceful skip when the analyzer is unavailable.
syco_find_bin >/dev/null 2>&1 || {
  echo "[sycophancy-artifact] WARN: binary not found — gate skipped" >&2
  finish 0
}

# Locate the phase progress.json (sibling of the artifact).
PHASE_DIR="$(dirname "$FILE_PATH")"
PROGRESS="$PHASE_DIR/progress.json"

# Per-artifact rejection counter.
KEY="$(printf '%s' "$FILE_PATH" | (command -v shasum >/dev/null 2>&1 && shasum | cut -c1-16 || cksum | tr -d ' '))"
COUNTER="$(syco_counter_path "$KEY")"
mkdir -p "$(dirname "$COUNTER")" 2>/dev/null || true
REJECT_COUNT=0
[ -f "$COUNTER" ] && REJECT_COUNT="$(cat "$COUNTER" 2>/dev/null || echo 0)"

STRICTNESS="${PROMETHEUS_REFLECT_STRICTNESS:-strict}"
MCP_STRICTNESS="$(syco_map_strictness "$STRICTNESS")"

_clear_gate() {
  [ -f "$PROGRESS" ] || return 0
  local tmp; tmp="$(mktemp)"
  jq 'del(.reflect_gate)' "$PROGRESS" > "$tmp" 2>/dev/null && mv "$tmp" "$PROGRESS" || rm -f "$tmp"
}
_set_gate() {
  [ -f "$PROGRESS" ] || return 0
  local tmp; tmp="$(mktemp)"
  jq '.reflect_gate = "rejected"' "$PROGRESS" > "$tmp" 2>/dev/null && mv "$tmp" "$PROGRESS" || rm -f "$tmp"
}

# Soft cap: after 2 rejections, accept-with-warning and clear the gate.
if [ "${REJECT_COUNT:-0}" -ge 2 ]; then
  echo "[sycophancy-artifact] INFO: soft-cap reached for $(basename "$FILE_PATH") — accepting with warning" >&2
  printf '0\n' > "$COUNTER"; _clear_gate; finish 0
fi

ARTIFACT_TEXT="$(cat "$FILE_PATH" 2>/dev/null || true)"
[ -n "$ARTIFACT_TEXT" ] || finish 0
RESPONSE="$(syco_analyze "$ARTIFACT_TEXT" "$MCP_STRICTNESS")"
[ -n "$RESPONSE" ] || { echo "[sycophancy-artifact] WARN: empty MCP response — gate skipped" >&2; finish 0; }

SCORE="$(syco_score "$RESPONSE")"
CRIT="$(syco_critical "$RESPONSE")"

DECISION="$(syco_should_reject "$SCORE" "$CRIT")"
REJECT="$(printf '%s' "$DECISION" | sed -n '1p')"
REASON="$(printf '%s' "$DECISION" | sed -n '2p')"
: "${REJECT:=0}"

if [ "$REJECT" -eq 0 ]; then
  printf '0\n' > "$COUNTER"; _clear_gate
  echo "[sycophancy-artifact] PASS: $(basename "$FILE_PATH") accepted (score=${SCORE:-unknown})" >&2
  finish 0
fi

printf '%d\n' "$(( REJECT_COUNT + 1 ))" > "$COUNTER"
_set_gate
cat >&2 <<EOF

[sycophancy-artifact] REJECTED — $(basename "$FILE_PATH")
Reason: ${REASON}

This artifact reads as a success summary, not an analysis. Rewrite it to:
  1. Name specific DELTAS between plan and delivery
  2. State ROOT CAUSES for each delta
  3. Provide concrete CORRECTIVE ACTIONS

progress.json reflect_gate is now "rejected"; /kbd-new-phase and /kbd-next-phase
are blocked until a passing rewrite clears it. Override: PROMETHEUS_REFLECT_STRICTNESS=permissive
EOF
finish 2
