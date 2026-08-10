#!/usr/bin/env bash
# emit-ui-intent.sh — the ideation flow's ONLY user-facing presentation path.
#
# Usage:
#   emit-ui-intent.sh --title <t> --body <b> [--option <o>]... [--type <t>]
#   emit-ui-intent.sh --intent-json '<json>'
#
# Exit: 0 a response was obtained · 1 usage · 3 no response (timeout)
#
# WHY THE FLOW MUST NOT RENDER DIRECTLY
# Tier logic belongs in exactly one place. A skill that prints its own prompt
# works on the harness its author happened to be using and silently degrades
# everywhere else — and nobody notices, because printing text always "succeeds".
# Emitting a UiIntent and letting `ui-surface` resolve the tier is what makes
# Tier 1 delivery on a non-Claude harness possible at all.
#
# TIER 0 IS A FLOOR, NOT A SUCCESS.
# When the flow degrades to text, this script says so on stderr and exits 3 on a
# timeout. A caller that treats "some text appeared" as delivery cannot tell a
# working round trip from a silent fallback — which is exactly the confusion the
# acceptance criteria for this change were written to prevent.
#
# bash 3.2 compatible. No LLM calls.
set -uo pipefail

HERE="$(cd "$(dirname "$0")" && pwd)"
# Resolve ui-surface from the pack root, honouring an installed plugin layout.
for CAND in \
  "${CLAUDE_PLUGIN_ROOT:-}/skills/learn/ui-surface" \
  "$HERE/../../../learn/ui-surface" \
  "$HOME/.claude/skills/ui-surface"; do
  [ -n "$CAND" ] && [ -d "$CAND" ] && UI_SURFACE="$CAND" && break
done
[ -n "${UI_SURFACE:-}" ] || { echo "[emit-ui-intent] ERROR: ui-surface not found" >&2; exit 1; }

DETECT="$UI_SURFACE/scripts/detect-surface-tier.sh"
RENDER="$UI_SURFACE/scripts/render.sh"
[ -f "$RENDER" ] || { echo "[emit-ui-intent] ERROR: render.sh not found in $UI_SURFACE" >&2; exit 1; }

TITLE="" BODY="" TYPE="question" INTENT_JSON=""
OPTS=""   # newline-joined; bash 3.2 has no arrays worth the trouble here
while [ $# -gt 0 ]; do
  case "$1" in
    --title)       TITLE="${2:-}";       shift 2 ;;
    --body)        BODY="${2:-}";        shift 2 ;;
    --type)        TYPE="${2:-}";        shift 2 ;;
    --option)      OPTS="$OPTS${2:-}
";                                       shift 2 ;;
    --intent-json) INTENT_JSON="${2:-}"; shift 2 ;;
    *) echo "usage: $0 --title <t> --body <b> [--option <o>]... | --intent-json <json>" >&2; exit 1 ;;
  esac
done

command -v jq >/dev/null 2>&1 || { echo "[emit-ui-intent] ERROR: jq required" >&2; exit 1; }

if [ -z "$INTENT_JSON" ]; then
  [ -n "$TITLE" ] || { echo "[emit-ui-intent] ERROR: --title or --intent-json required" >&2; exit 1; }
  INTENT_JSON="$(printf '%s' "$OPTS" | jq -R -s -c \
    --arg t "$TITLE" --arg b "$BODY" --arg ty "$TYPE" \
    'split("\n") | map(select(length > 0))
     | {intent_type: $ty, title: $t, body: $b, options: .}')"
fi

# Resolve the tier through ui-surface — never guess it here.
TIER="${SURFACE_TIER:-}"
if [ -z "$TIER" ] && [ -f "$DETECT" ]; then
  EVAL_OUT="$(bash "$DETECT" 2>/dev/null)" && eval "$EVAL_OUT" 2>/dev/null || true
  TIER="${SURFACE_TIER:-tier0_text}"
fi
TIER="${TIER:-tier0_text}"

echo "[emit-ui-intent] tier=$TIER harness=${SURFACE_HARNESS:-unknown}" >&2

RESPONSE="$(bash "$RENDER" --tier "$TIER" --intent-json "$INTENT_JSON" 2>/dev/null)"
RC=$?
if [ "$RC" -ne 0 ]; then
  echo "[emit-ui-intent] ERROR: render.sh failed (exit $RC)" >&2
  exit 1
fi

# A timeout is not a response. Reporting it as one would let a harness that
# never polls look identical to one that answered.
if printf '%s' "$RESPONSE" | jq -e '.error == "timeout"' >/dev/null 2>&1; then
  echo "[emit-ui-intent] NO RESPONSE: the harness did not answer within the timeout." >&2
  echo "[emit-ui-intent]   This is a stated limit, not delivery. Fall back to Tier 0 text." >&2
  exit 3
fi

printf '%s\n' "$RESPONSE"
exit 0
