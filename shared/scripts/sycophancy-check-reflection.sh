#!/usr/bin/env bash
# sycophancy-check-reflection.sh — SubagentStop gate for the reflector subagent.
#
# Reads the reflection artifact from stdin (Claude Code SubagentStop hook JSON),
# invokes the sycophancy-correction MCP server at configurable strictness,
# and rejects sycophantic reflections with actionable feedback.
#
# Inputs from environment:
#   PROMETHEUS_REFLECT_STRICTNESS  loose|standard|strict|adversarial (default: strict)
#   CLAUDE_PLUGIN_ROOT             required — used to locate the MCP binary
#
# Exit codes:
#   0  — artifact accepted (clean or 2-rejection soft cap reached)
#   2  — artifact rejected; actionable feedback written to stderr
#
# State file: ~/.prometheus/reflect-rejections.txt (counts consecutive rejections)
# Log: ~/.prometheus/hooks.log via hook-log.sh shim
#
# Must always exit 0 when the MCP binary is unavailable (graceful degradation).
set -uo pipefail

# ── Hook-log shim ──────────────────────────────────────────────────────────────
HOOK_LOG_LIB="$(cd "$(dirname "$0")" && pwd)/lib/hook-log.sh"
[ -f "$HOOK_LOG_LIB" ] && source "$HOOK_LOG_LIB"
hook_log_start "SubagentStop" "sycophancy-check-reflection.sh"

# ── Constants ──────────────────────────────────────────────────────────────────
REJECT_CAP=2
STRICTNESS="${PROMETHEUS_REFLECT_STRICTNESS:-strict}"
REJECTION_COUNTER="${HOME}/.prometheus/reflect-rejections.txt"
PROMETHEUS_DIR="${HOME}/.prometheus"

# Map adversarial → strict (MCP server only supports permissive|standard|strict)
case "$STRICTNESS" in
  adversarial) MCP_STRICTNESS="strict" ;;
  loose)       MCP_STRICTNESS="permissive" ;;
  *)           MCP_STRICTNESS="$STRICTNESS" ;;
esac

# ── Locate the MCP binary ──────────────────────────────────────────────────────
_find_mcp_bin() {
  if command -v sycophancy-correction &>/dev/null; then
    command -v sycophancy-correction
    return 0
  fi
  local plugin_root="${CLAUDE_PLUGIN_ROOT:-}"
  if [ -z "$plugin_root" ]; then
    return 1
  fi
  local candidate="${plugin_root}/skills/imported/sycophancy-correction/target/release/sycophancy-correction"
  if [ -x "$candidate" ]; then
    printf '%s' "$candidate"
    return 0
  fi
  return 1
}

MCP_BIN="$(_find_mcp_bin 2>/dev/null || true)"
if [ -z "$MCP_BIN" ]; then
  echo "[sycophancy-gate] WARN: sycophancy-correction binary not found — gate skipped" >&2
  hook_log_end 0
  exit 0
fi

# ── Read hook input from stdin ─────────────────────────────────────────────────
HOOK_INPUT="$(cat)"

# Extract the subagent output / artifact from the hook JSON.
# Claude Code SubagentStop event provides {"subagent_name":"reflector","output":"..."}
REFLECT_TEXT="$(printf '%s' "$HOOK_INPUT" \
  | python3 -c "import sys,json; d=json.load(sys.stdin); print(d.get('output','') or d.get('result','') or '')" \
  2>/dev/null || true)"

if [ -z "$REFLECT_TEXT" ]; then
  echo "[sycophancy-gate] WARN: no output text in SubagentStop event — gate skipped" >&2
  hook_log_end 0
  exit 0
fi

# ── Rejection soft-cap check ──────────────────────────────────────────────────
mkdir -p "$PROMETHEUS_DIR"
REJECT_COUNT=0
if [ -f "$REJECTION_COUNTER" ]; then
  REJECT_COUNT="$(cat "$REJECTION_COUNTER" 2>/dev/null || echo 0)"
fi

if [ "${REJECT_COUNT:-0}" -ge "$REJECT_CAP" ]; then
  echo "[sycophancy-gate] INFO: soft-cap reached (${REJECT_COUNT}/${REJECT_CAP} consecutive rejections) — accepting with warning" >&2
  printf '0\n' > "$REJECTION_COUNTER"
  hook_log_end 0
  exit 0
fi

# ── Invoke MCP server via JSON-RPC ────────────────────────────────────────────
_mcp_analyze() {
  local artifact="$1"
  local strictness="$2"

  # Escape the artifact text for JSON embedding
  local escaped
  escaped="$(printf '%s' "$artifact" | python3 -c \
    "import sys,json; print(json.dumps(sys.stdin.read()))" 2>/dev/null)"
  if [ -z "$escaped" ]; then
    return 1
  fi

  # Use a FIFO for controlled message pacing (avoids race before initialized ack)
  local fifo
  fifo="$(mktemp -u /tmp/sycophancy_mcp_XXXXX)"
  mkfifo "$fifo"
  # shellcheck disable=SC2064
  trap "rm -f '$fifo'" RETURN

  local skill_toml
  local plugin_root="${CLAUDE_PLUGIN_ROOT:-}"
  if [ -n "$plugin_root" ] && [ -f "${plugin_root}/skills/imported/sycophancy-correction/skill.toml" ]; then
    skill_toml="${plugin_root}/skills/imported/sycophancy-correction/skill.toml"
  else
    skill_toml=""
  fi

  # Send: initialize → initialized notification → tools/call
  (
    printf '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"sycophancy-gate","version":"0.1.0"}}}\n'
    sleep 0.2
    printf '{"jsonrpc":"2.0","method":"notifications/initialized","params":{}}\n'
    sleep 0.1
    printf '{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"detect_sycophancy","arguments":{"content":%s,"target":"completion","strictness":"%s"}}}\n' \
      "$escaped" "$strictness"
    sleep 3
  ) > "$fifo" &

  local response=""
  if command -v timeout &>/dev/null; then
    if [ -n "$skill_toml" ]; then
      response="$(timeout 30 "$MCP_BIN" --config "$skill_toml" < "$fifo" 2>/dev/null)" || true
    else
      response="$(timeout 30 "$MCP_BIN" < "$fifo" 2>/dev/null)" || true
    fi
  else
    if [ -n "$skill_toml" ]; then
      response="$("$MCP_BIN" --config "$skill_toml" < "$fifo" 2>/dev/null)" || true
    else
      response="$("$MCP_BIN" < "$fifo" 2>/dev/null)" || true
    fi
  fi

  printf '%s' "$response"
}

RESPONSE="$(_mcp_analyze "$REFLECT_TEXT" "$MCP_STRICTNESS")"

if [ -z "$RESPONSE" ]; then
  echo "[sycophancy-gate] WARN: empty MCP response — gate skipped" >&2
  hook_log_end 0
  exit 0
fi

# ── Parse response: extract sycophancy_score and high/critical patterns ────────
SYCO_SCORE="$(printf '%s' "$RESPONSE" \
  | python3 -c "
import sys, json
lines = sys.stdin.read().splitlines()
for line in lines:
    try:
        obj = json.loads(line)
        # tools/call response embeds result in obj['result']['content'][0]['text']
        for key in ('result',):
            r = obj.get(key, {})
            if isinstance(r, dict):
                content = r.get('content', [])
                if content and isinstance(content, list):
                    text = content[0].get('text', '{}')
                    data = json.loads(text)
                    score = data.get('sycophancy_score', -1)
                    if score >= 0:
                        print(score)
                        break
    except Exception:
        pass
" 2>/dev/null || echo "")"

CRITICAL_PATTERNS="$(printf '%s' "$RESPONSE" \
  | python3 -c "
import sys, json
lines = sys.stdin.read().splitlines()
for line in lines:
    try:
        obj = json.loads(line)
        r = obj.get('result', {})
        if isinstance(r, dict):
            content = r.get('content', [])
            if content and isinstance(content, list):
                text = content[0].get('text', '{}')
                data = json.loads(text)
                classes = data.get('classifications', [])
                high_crit = [c.get('pattern_id','?') + ':' + c.get('severity','?')
                             for c in classes
                             if c.get('severity','') in ('high','critical')]
                if high_crit:
                    print(' '.join(high_crit))
    except Exception:
        pass
" 2>/dev/null || echo "")"

# ── Decision logic ─────────────────────────────────────────────────────────────
# Reject if: score >= 0.4 OR any high/critical patterns detected
SHOULD_REJECT=0
REJECT_REASON=""

if [ -n "$SYCO_SCORE" ]; then
  IS_HIGH="$(python3 -c "print('yes' if float('${SYCO_SCORE}') >= 0.4 else 'no')" 2>/dev/null || echo "no")"
  if [ "$IS_HIGH" = "yes" ]; then
    SHOULD_REJECT=1
    REJECT_REASON="sycophancy_score=${SYCO_SCORE} (threshold: 0.4, strictness: ${STRICTNESS})"
  fi
fi

if [ -n "$CRITICAL_PATTERNS" ]; then
  SHOULD_REJECT=1
  REJECT_REASON="${REJECT_REASON:+${REJECT_REASON}; }high/critical patterns: ${CRITICAL_PATTERNS}"
fi

if [ "$SHOULD_REJECT" -eq 0 ]; then
  # Clean — reset consecutive rejection counter
  printf '0\n' > "$REJECTION_COUNTER"
  echo "[sycophancy-gate] PASS: reflection accepted (score=${SYCO_SCORE:-unknown}, strictness=${STRICTNESS})" >&2
  hook_log_end 0
  exit 0
fi

# ── Rejection path ─────────────────────────────────────────────────────────────
NEW_COUNT=$(( REJECT_COUNT + 1 ))
printf '%d\n' "$NEW_COUNT" > "$REJECTION_COUNTER"

cat >&2 <<REJECTION_MSG

╔══════════════════════════════════════════════════════════════════════╗
║  SYCOPHANCY GATE — REFLECTION REJECTED                               ║
╚══════════════════════════════════════════════════════════════════════╝

Reason: ${REJECT_REASON}
Consecutive rejections: ${NEW_COUNT}/${REJECT_CAP} (soft cap — accept-with-warning at cap)

The reflector output appears to agree too readily with the generation pass.
A strong reflection must:
  1. Name specific DELTAS between what was planned and what was delivered
  2. State ROOT CAUSES for each delta (not "we achieved X")
  3. Provide CORRECTIVE ACTIONS — concrete next steps, not praise

To suppress the gate: PROMETHEUS_REFLECT_STRICTNESS=permissive

REJECTION_MSG

hook_log_end 2
exit 2
