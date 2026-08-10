#!/usr/bin/env bash
set -euo pipefail

# Contradiction detected hook — fires when Stage 06 detects a contradiction.
# Logs the contradiction and optionally escalates via pmpo-elicit.

JOB_ID="${RESEARCH_JOB_ID:-unknown}"
TOPIC="${RESEARCH_CONTRADICTION_TOPIC:-unknown}"
CLAIM_A="${RESEARCH_CONTRADICTION_CLAIM_A:-}"
CLAIM_B="${RESEARCH_CONTRADICTION_CLAIM_B:-}"
AUTO_ESCALATE="${RESEARCH_AUTO_ESCALATE:-0}"
OUTPUT_DIR="${RESEARCH_OUTPUT_DIR:-$HOME/.research-jobs}"
CLAUDE_PLUGIN_ROOT="${CLAUDE_PLUGIN_ROOT:-}"

NOW=$(date -u +%Y-%m-%dT%H:%M:%SZ)

echo "[on-contradiction] Contradiction detected in job $JOB_ID on topic: $TOPIC"
echo "[on-contradiction]   Claim A: ${CLAIM_A:0:80}..."
echo "[on-contradiction]   Claim B: ${CLAIM_B:0:80}..."

# Append to contradiction log
CONTRA_LOG="$OUTPUT_DIR/$JOB_ID/contradiction-events.log"
echo "$NOW | $TOPIC | A=$CLAIM_A | B=$CLAIM_B" >> "$CONTRA_LOG"

# Optionally escalate via pmpo-elicit (only when AUTO_ESCALATE=1 and script exists)
if [[ "$AUTO_ESCALATE" == "1" ]] && [[ -n "$CLAUDE_PLUGIN_ROOT" ]]; then
  ELICIT_SCRIPT="$CLAUDE_PLUGIN_ROOT/skills/process/pmpo-elicit/scripts/pmpo-elicit-checkpoint.sh"
  if [[ -x "$ELICIT_SCRIPT" ]]; then
    echo "[on-contradiction] Escalating to pmpo-elicit..."
    bash "$ELICIT_SCRIPT" \
      "$OUTPUT_DIR/$JOB_ID/elicitations/contra-$(date +%s)" \
      "Contradiction on '$TOPIC': Claim A says '$CLAIM_A', Claim B says '$CLAIM_B'. Which position should the research take?" \
      "high" "deep-research-stage-06" \
      "$CLAIM_A" "$CLAIM_B"
  else
    echo "[on-contradiction] pmpo-elicit script not found at $ELICIT_SCRIPT — logging only"
  fi
else
  echo "[on-contradiction] Auto-escalate disabled. Contradiction logged for Stage 06 resolution."
fi
