#!/usr/bin/env bash
# pk-focus-on-prompt.sh — UserPromptSubmit hook: injects pk-focus context for the current prompt.
# Contract: reads JSON from stdin, prints focus output to stdout (or nothing on degraded path).
# Must always exit 0.
#
# Env knobs:
#   PROMETHEUS_FOCUS_SEMANTIC=0   disable the surreal-memory semantic path
#   SURREAL_MEMORY_URL            override SM base URL (default: http://localhost:23001)
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
      -X POST "${SM_URL}/api/v1/memory/search" \
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
if command -v timeout &>/dev/null; then
  FOCUS_OUTPUT="$(timeout 2.5 pk focus "$ALL_WORDS" --max-articles 3 2>/dev/null || hook_log_error "$LINENO")"
else
  FOCUS_OUTPUT="$(pk focus "$ALL_WORDS" --max-articles 3 2>/dev/null || hook_log_error "$LINENO")"
fi

if [ -n "${FOCUS_OUTPUT:-}" ]; then
  printf '\n\n--- prometheus-knowledge context ---\n%s\n--- end pk context ---\n' "$FOCUS_OUTPUT"
fi

hook_log_end 0
exit 0
