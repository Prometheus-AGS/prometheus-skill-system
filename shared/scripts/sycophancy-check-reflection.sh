#!/usr/bin/env bash
# sycophancy-check-reflection.sh — SubagentStop gate for the reflector subagent.
#
# Thin wrapper over shared/scripts/lib/sycophancy.sh. Behavior is unchanged from
# the pre-extraction version: reads the reflector's output from the SubagentStop
# event, rejects when sycophancy_score >= 0.4 or any high/critical pattern is
# present, with a 2-rejection soft cap, and degrades to exit 0 when the binary
# is unavailable.
#
# Inputs:  PROMETHEUS_REFLECT_STRICTNESS (loose|standard|strict|adversarial, default strict)
# Exit:    0 accepted (clean or soft-cap reached) · 2 rejected (feedback on stderr)
# State:   ~/.prometheus/reflect-rejections.txt (consecutive rejection count)
set -uo pipefail

HERE="$(cd "$(dirname "$0")" && pwd)"
HOOK_LOG_LIB="$HERE/lib/hook-log.sh"
[ -f "$HOOK_LOG_LIB" ] && source "$HOOK_LOG_LIB"
hook_log_start "SubagentStop" "sycophancy-check-reflection.sh"
# shellcheck source=/dev/null
source "$HERE/lib/sycophancy.sh"

REJECT_CAP=2
STRICTNESS="${PROMETHEUS_REFLECT_STRICTNESS:-strict}"
MCP_STRICTNESS="$(syco_map_strictness "$STRICTNESS")"
REJECTION_COUNTER="${HOME}/.prometheus/reflect-rejections.txt"
mkdir -p "${HOME}/.prometheus" 2>/dev/null || true

if ! syco_find_bin >/dev/null 2>&1; then
  echo "[sycophancy-gate] WARN: sycophancy-correction binary not found — gate skipped" >&2
  hook_log_end 0; exit 0
fi

HOOK_INPUT="$(cat)"
REFLECT_TEXT="$(printf '%s' "$HOOK_INPUT" \
  | python3 -c "import sys,json; d=json.load(sys.stdin); print(d.get('output','') or d.get('result','') or '')" 2>/dev/null || true)"
if [ -z "$REFLECT_TEXT" ]; then
  echo "[sycophancy-gate] WARN: no output text in SubagentStop event — gate skipped" >&2
  hook_log_end 0; exit 0
fi

REJECT_COUNT=0
[ -f "$REJECTION_COUNTER" ] && REJECT_COUNT="$(cat "$REJECTION_COUNTER" 2>/dev/null || echo 0)"
if [ "${REJECT_COUNT:-0}" -ge "$REJECT_CAP" ]; then
  echo "[sycophancy-gate] INFO: soft-cap reached (${REJECT_COUNT}/${REJECT_CAP}) — accepting with warning" >&2
  printf '0\n' > "$REJECTION_COUNTER"
  hook_log_end 0; exit 0
fi

RESPONSE="$(syco_analyze "$REFLECT_TEXT" "$MCP_STRICTNESS")"
if [ -z "$RESPONSE" ]; then
  echo "[sycophancy-gate] WARN: empty MCP response — gate skipped" >&2
  hook_log_end 0; exit 0
fi

SYCO_SCORE="$(syco_score "$RESPONSE")"
CRITICAL_PATTERNS="$(syco_critical "$RESPONSE")"

SHOULD_REJECT=0; REJECT_REASON=""
if [ -n "$SYCO_SCORE" ]; then
  IS_HIGH="$(python3 -c "print('yes' if float('${SYCO_SCORE}') >= 0.4 else 'no')" 2>/dev/null || echo no)"
  [ "$IS_HIGH" = "yes" ] && { SHOULD_REJECT=1; REJECT_REASON="sycophancy_score=${SYCO_SCORE} (threshold 0.4, strictness ${STRICTNESS})"; }
fi
[ -n "$CRITICAL_PATTERNS" ] && { SHOULD_REJECT=1; REJECT_REASON="${REJECT_REASON:+${REJECT_REASON}; }high/critical: ${CRITICAL_PATTERNS}"; }

if [ "$SHOULD_REJECT" -eq 0 ]; then
  printf '0\n' > "$REJECTION_COUNTER"
  echo "[sycophancy-gate] PASS: reflection accepted (score=${SYCO_SCORE:-unknown}, strictness=${STRICTNESS})" >&2
  hook_log_end 0; exit 0
fi

NEW_COUNT=$(( REJECT_COUNT + 1 ))
printf '%d\n' "$NEW_COUNT" > "$REJECTION_COUNTER"
cat >&2 <<REJECTION_MSG

╔══════════════════════════════════════════════════════════════════════╗
║  SYCOPHANCY GATE — REFLECTION REJECTED                               ║
╚══════════════════════════════════════════════════════════════════════╝

Reason: ${REJECT_REASON}
Consecutive rejections: ${NEW_COUNT}/${REJECT_CAP} (soft cap — accept-with-warning at cap)

A strong reflection must:
  1. Name specific DELTAS between what was planned and what was delivered
  2. State ROOT CAUSES for each delta (not "we achieved X")
  3. Provide CORRECTIVE ACTIONS — concrete next steps, not praise

To suppress the gate: PROMETHEUS_REFLECT_STRICTNESS=permissive

REJECTION_MSG
hook_log_end 2; exit 2
