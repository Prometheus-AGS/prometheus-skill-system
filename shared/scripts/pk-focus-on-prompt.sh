#!/usr/bin/env bash
# pk-focus-on-prompt.sh — UserPromptSubmit hook: injects pk-focus context for the current prompt.
# Contract: reads JSON from stdin, prints focus output to stdout (or nothing on degraded path).
# Must always exit 0.
#
# Env knobs:
#   PROMETHEUS_FOCUS_SEMANTIC=0   disable the surreal-memory semantic path
#   PROMETHEUS_FOCUS_TIMEOUT=30   max seconds for pk focus before degrading silently
#   SURREAL_MEMORY_URL            override SM base URL (default: http://localhost:23001)
#
# Timeout budget: `pk focus --k 3` performs LLM synthesis over the whole KB and
# measured 22-24s at ~132 entries. The former 5s default expired mid-call on
# ~6.6% of invocations (345/5260 recorded fires), so the hook injected nothing
# while still exiting 0 — a silent context loss that looked healthy in the log.
# 30s covers the measured range with headroom; raise it as the KB grows.
set -uo pipefail

HOOK_LOG_LIB="$(cd "$(dirname "$0")" && pwd)/lib/hook-log.sh"
[ -f "$HOOK_LOG_LIB" ] && source "$HOOK_LOG_LIB"
hook_log_start "UserPromptSubmit" "pk-focus-on-prompt.sh"

# --- Graceful degradation: pk must be on PATH ---
if ! command -v pk &>/dev/null; then
  hook_log_end 0
  exit 0
fi

# --- Read prompt text from stdin JSON ---
PROMPT_JSON="$(cat)"
PROMPT_TEXT="$(printf '%s' "$PROMPT_JSON" | python3 -c \
  "import sys, json; d=json.load(sys.stdin); print(d.get('prompt',''))" 2>/dev/null || true)"

if [ -z "$PROMPT_TEXT" ]; then
  hook_log_end 0
  exit 0
fi

# --- Lexical path (always runs): top-5 longest words from prompt ---
LEXICAL_WORDS="$(printf '%s' "$PROMPT_TEXT" \
  | tr '[:upper:]' '[:lower:]' \
  | tr -cs 'a-z0-9 ' ' ' \
  | tr ' ' '\n' \
  | awk 'length>4' \
  | sort -u \
  | awk '{print length, $0}' | sort -rn | awk '{print $2}' \
  | head -5)"

# --- Semantic path (runs when surreal-memory is reachable and not opt-out) ---
SEMANTIC_WORDS=""
if [[ "${PROMETHEUS_FOCUS_SEMANTIC:-1}" == "1" ]]; then
  SM_URL="${SURREAL_MEMORY_URL:-http://localhost:23001}"
  # Safe JSON encoding of prompt via python3 (guaranteed available if pk is)
  PROMPT_JSON_STR="$(printf '%s' "$PROMPT_TEXT" \
    | python3 -c "import sys, json; print(json.dumps(sys.stdin.read()))" 2>/dev/null || true)"

  if [[ -n "$PROMPT_JSON_STR" ]]; then
    SM_RESPONSE="$(curl -sf --max-time 3 \
      -X POST "${SM_URL}/api/v1/search" \
      -H "Content-Type: application/json" \
      -d "{\"query\": ${PROMPT_JSON_STR}, \"user_id\": \"prometheus-skill-pack\", \"limit\": 3}" \
      2>/dev/null)" || SM_RESPONSE=""

    if [[ -n "$SM_RESPONSE" ]]; then
      # Extract first two words from each memory result → topic tokens
      SEMANTIC_WORDS="$(printf '%s' "$SM_RESPONSE" \
        | python3 -c "
import sys, json
try:
  results = json.load(sys.stdin)
  words = []
  for r in (results if isinstance(results, list) else []):
    mem = r.get('memory', '') or r.get('content', '')
    tokens = [w.lower() for w in mem.split() if len(w) > 4][:2]
    words.extend(tokens)
  print('\n'.join(dict.fromkeys(words)))
except Exception:
  pass
" 2>/dev/null || true)"
    fi
  fi
fi

# --- Merge lexical + semantic, dedup, take top 8 ---
ALL_WORDS="$(printf '%s\n%s' "$LEXICAL_WORDS" "$SEMANTIC_WORDS" \
  | awk 'NF && !seen[$0]++' \
  | head -8 \
  | tr '\n' ' ' \
  | sed 's/[[:space:]]*$//')"

if [ -z "$ALL_WORDS" ]; then
  hook_log_end 0
  exit 0
fi

# --- Run pk focus (with timeout when available — timeout is not on stock macOS) ---
FOCUS_TIMEOUT_SECONDS="${PROMETHEUS_FOCUS_TIMEOUT:-30}"
if command -v timeout &>/dev/null; then
  FOCUS_OUTPUT="$(timeout "$FOCUS_TIMEOUT_SECONDS" pk focus "$ALL_WORDS" --k 3 2>/dev/null)"
  FOCUS_RC=$?
else
  FOCUS_OUTPUT="$(pk focus "$ALL_WORDS" --k 3 2>/dev/null)"
  FOCUS_RC=$?
fi

# Distinguish "budget expired" (124 from timeout) from a genuine pk failure, so
# a too-small budget is diagnosable instead of looking healthy in the log.
#
# hook_log_error interpolates its argument as a bare JSON number, so it must be
# given a line number and nothing else — a message string there emits invalid
# JSON and corrupts the log. The human-readable hint goes to stderr, which the
# harness captures without it counting as hook output.
if [ "$FOCUS_RC" -ne 0 ]; then
  hook_log_error "$LINENO"
  if [ "$FOCUS_RC" -eq 124 ]; then
    printf 'pk-focus-on-prompt: pk focus exceeded %ss budget; raise PROMETHEUS_FOCUS_TIMEOUT\n' \
      "$FOCUS_TIMEOUT_SECONDS" >&2
  fi
  FOCUS_OUTPUT=""
fi

if [ -n "${FOCUS_OUTPUT:-}" ]; then
  printf '\n\n--- prometheus-knowledge context ---\n%s\n--- end pk context ---\n' "$FOCUS_OUTPUT"
fi

hook_log_end 0
exit 0
