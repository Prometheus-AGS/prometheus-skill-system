#!/usr/bin/env bash
# idea-gate-1.sh — Gate 1 plausibility check for validate-idea pipeline
# Usage: idea-gate-1.sh "<idea text>" [<evolution-name>]
# Exit 0 = PASS (idea is novel and feasible enough for Gate 2)
# Exit 1 = REJECT (duplicate, in backlog, or conflicts with design philosophy)
# [MODEL_ROUTING] phase=evolver-idea-gate1 class=small
set -euo pipefail

IDEA="${1:?Usage: idea-gate-1.sh \"<idea text>\" [<evolution-name>]}"
EVOLUTION_NAME="${2:-default}"

echo "[gate-1] Checking plausibility for idea: ${IDEA}" >&2
echo "[gate-1] Evolution: ${EVOLUTION_NAME}" >&2

REJECT_REASON=""

# --- Check 1: Is this already implemented? ---
echo "[gate-1] Check 1: Scanning skills/ for existing implementation..." >&2

KEYWORDS=$(python3 -c "
import re, sys
idea = sys.argv[1].lower()
# Extract meaningful keywords (3+ chars, skip stop words)
stop = {'the','and','for','are','with','this','that','from','have','not','but','all','can','will','its','also','into','more','was','been','has','had','how','what','when','who','any','some','new','our','use','you','add'}
words = re.findall(r'[a-z]{3,}', idea)
keywords = [w for w in words if w not in stop]
print(' '.join(keywords[:5]))
" "${IDEA}")

echo "[gate-1] Keywords to scan: ${KEYWORDS}" >&2

MATCH_FOUND=0
for kw in ${KEYWORDS}; do
  if find skills/ -name "SKILL.md" -exec grep -li "${kw}" {} \; 2>/dev/null | grep -q .; then
    MATCHES=$(find skills/ -name "SKILL.md" -exec grep -li "${kw}" {} \; 2>/dev/null | head -3 | tr '\n' ', ')
    echo "[gate-1] Keyword '${kw}' matched: ${MATCHES}" >&2
    MATCH_FOUND=1
    break
  fi
done

if [ "${MATCH_FOUND}" -eq 1 ]; then
  REJECT_REASON="Idea may duplicate existing skill(s) matching keyword '${kw}'. Matches: ${MATCHES}. Run Gate 2 to confirm or override."
  # Soft block — not an automatic reject; just surface for Gate 2
  echo "[gate-1] WARNING: Potential duplicate detected. Surfacing for Gate 2 review." >&2
fi

# --- Check 2: Is this already in the backlog? ---
echo "[gate-1] Check 2: Checking backlog for existing entry..." >&2

BACKLOG_FILE=".evolver/${EVOLUTION_NAME}/backlog.json"
if [ -f "${BACKLOG_FILE}" ]; then
  BACKLOG_MATCH=$(python3 -c "
import json, sys, re

with open(sys.argv[1]) as f:
    backlog = json.load(f)

idea = sys.argv[2].lower()
# Extract significant words
words = set(re.findall(r'[a-z]{4,}', idea))
items = backlog if isinstance(backlog, list) else backlog.get('items', [])
for item in items:
    item_text = (item.get('text', '') + ' ' + item.get('title', '')).lower()
    item_words = set(re.findall(r'[a-z]{4,}', item_text))
    overlap = words & item_words
    if len(overlap) >= 3:
        print(item.get('id', 'unknown') + ': ' + item.get('text', item.get('title', 'unknown')))
        break
" "${BACKLOG_FILE}" "${IDEA}" 2>/dev/null || echo "")

  if [ -n "${BACKLOG_MATCH}" ]; then
    echo "[gate-1] REJECT: Idea already in backlog: ${BACKLOG_MATCH}" >&2
    printf '{"gate": 1, "passed": false, "reject_reason": "Idea already in backlog: %s", "revisit_weight": 0.1}\n' "${BACKLOG_MATCH}"
    exit 1
  fi
fi

# --- Check 3: Does this conflict with design-philosophy.md? ---
PHILOSOPHY_FILE=".evolver/${EVOLUTION_NAME}/design-philosophy.md"
# Resolve the shared model library. Previously this called `liter-llm complete`, a
# subcommand that does not exist, with `2>/dev/null || echo "NO"` — so the gate
# ALWAYS took the "no conflict" branch and silently passed every idea.
_GATE_LIB=""
for _cand in \
  "$(cd "$(dirname "$0")" && pwd)/../../../../shared/scripts/lib/kbd-model-resolve.sh" \
  "${CLAUDE_PLUGIN_ROOT:-}/shared/scripts/lib/kbd-model-resolve.sh" \
  "${PLUGIN_ROOT:-}/shared/scripts/lib/kbd-model-resolve.sh"; do
  if [ -n "$_cand" ] && [ -f "$_cand" ]; then _GATE_LIB="$_cand"; break; fi
done

if [ -f "${PHILOSOPHY_FILE}" ] && [ -n "$_GATE_LIB" ]; then
  # shellcheck source=/dev/null
  . "$_GATE_LIB"
  echo "[gate-1] Check 3: Checking design philosophy conflict..." >&2

  # [MODEL_ROUTING] phase=evolver-idea-gate1-philosophy class=small
  PHILOSOPHY=$(head -100 "${PHILOSOPHY_FILE}")
  _sys='Answer with only YES or NO.'
  _usr="$(printf 'Philosophy:\n%s\n\nIdea: %s\n\nDoes this idea directly conflict with the philosophy?' "${PHILOSOPHY}" "${IDEA}")"
  if ! CONFLICT="$(kbd_complete "$(kbd_resolve_role critic)" "$_sys" "$_usr" 8)"; then
    # A failed check is NOT a pass. Say so instead of waving the idea through.
    echo "[gate-1] WARN: philosophy check could not run (see message above) —" >&2
    echo "[gate-1]       treating as INDETERMINATE, not as 'no conflict'." >&2
    CONFLICT="INDETERMINATE"
  fi

  if echo "${CONFLICT}" | grep -qi "^YES"; then
    echo "[gate-1] REJECT: Conflicts with design philosophy." >&2
    printf '{"gate": 1, "passed": false, "reject_reason": "Conflicts with design-philosophy.md. Philosophy check returned YES.", "revisit_weight": 0.1}\n'
    exit 1
  fi
fi

# --- PASS ---
if [ -n "${REJECT_REASON}" ]; then
  echo "[gate-1] PASS (with warning): ${REJECT_REASON}" >&2
  printf '{"gate": 1, "passed": true, "warning": "%s", "note": "Potential duplicate — Gate 2 should verify novelty"}\n' "${REJECT_REASON}"
else
  echo "[gate-1] PASS: Idea is novel and not blocked." >&2
  printf '{"gate": 1, "passed": true, "warning": null}\n'
fi

exit 0
