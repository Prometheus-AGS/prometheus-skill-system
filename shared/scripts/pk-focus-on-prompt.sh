#!/usr/bin/env bash
# pk-focus-on-prompt.sh — UserPromptSubmit hook: injects pk-focus context for the current prompt.
# Contract: reads JSON from stdin, prints focus output to stdout (or nothing on degraded path).
# Must always exit 0.
set -uo pipefail

# --- Graceful degradation: pk must be on PATH ---
if ! command -v pk &>/dev/null; then
  exit 0
fi

# --- Read prompt text from stdin JSON ---
PROMPT_JSON="$(cat)"
PROMPT_TEXT="$(printf '%s' "$PROMPT_JSON" | python3 -c \
  "import sys, json; d=json.load(sys.stdin); print(d.get('prompt',''))" 2>/dev/null || true)"

if [ -z "$PROMPT_TEXT" ]; then
  exit 0
fi

# --- Extract top-5 keywords (strip punctuation, sort by descending length, take top 5) ---
KEYWORDS="$(printf '%s' "$PROMPT_TEXT" \
  | tr '[:upper:]' '[:lower:]' \
  | tr -cs 'a-z0-9 ' ' ' \
  | tr ' ' '\n' \
  | awk 'length>4' \
  | sort -u \
  | awk '{print length, $0}' | sort -rn | awk '{print $2}' \
  | head -5 \
  | tr '\n' ' ' \
  | sed 's/ $//')"

if [ -z "$KEYWORDS" ]; then
  exit 0
fi

# --- Run pk focus (with timeout when available — timeout is not on stock macOS) ---
if command -v timeout &>/dev/null; then
  FOCUS_OUTPUT="$(timeout 2.5 pk focus "$KEYWORDS" --max-articles 3 2>/dev/null || true)"
else
  FOCUS_OUTPUT="$(pk focus "$KEYWORDS" --max-articles 3 2>/dev/null || true)"
fi

if [ -n "$FOCUS_OUTPUT" ]; then
  printf '\n\n--- prometheus-knowledge context ---\n%s\n--- end pk context ---\n' "$FOCUS_OUTPUT"
fi

exit 0
