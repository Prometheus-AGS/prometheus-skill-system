#!/usr/bin/env bash
# loop-tick.sh — execute one outer-loop tick
# Usage: bash loop-tick.sh <loop-name> [--dry-run]
# Exit: 0=continue, 1=escalate, 2=terminate, 3=error

set -euo pipefail

LOOP_NAME="${1:-}"
DRY_RUN=false
[[ "${2:-}" == "--dry-run" ]] && DRY_RUN=true

if [[ -z "$LOOP_NAME" ]]; then
  echo "Usage: $0 <loop-name> [--dry-run]" >&2
  exit 3
fi

LOOPS_DIR=".kbd-orchestrator/loops/${LOOP_NAME}"
LOOP_FILE="${LOOPS_DIR}/loop.json"
JOURNAL="${LOOPS_DIR}/journal.md"
DECISION_LOG="${LOOPS_DIR}/decision-log.md"

if [[ ! -f "$LOOP_FILE" ]]; then
  echo "ERROR: loop.json not found at ${LOOP_FILE}" >&2
  exit 3
fi

# Read loop state
current_tick=$(jq -r '.current_tick // 0' "$LOOP_FILE")
no_progress=$(jq -r '.no_progress_ticks // 0' "$LOOP_FILE")
max_ticks=$(jq -r '.termination.max_ticks // 20' "$LOOP_FILE")
max_no_progress=$(jq -r '.termination.max_no_progress_ticks // 2' "$LOOP_FILE")
status=$(jq -r '.status // "active"' "$LOOP_FILE")
evolution_name=$(jq -r '.evolution_name' "$LOOP_FILE")

# Guard: already terminal
if [[ "$status" == "completed" || "$status" == "escalated" ]]; then
  echo "Loop ${LOOP_NAME} is already in terminal state: ${status}"
  exit 2
fi

new_tick=$((current_tick + 1))
echo "=== Loop tick ${new_tick} / max ${max_ticks}: ${LOOP_NAME} ==="

# Guard: hard ceiling
if [[ $new_tick -gt $max_ticks ]]; then
  echo "TERMINATE: max_ticks (${max_ticks}) reached"
  new_status="completed"
  outcome="terminate"
  reason="max_ticks reached"
else
  # Evaluate feedback sources
  sources_count=$(jq '.feedback_sources | length' "$LOOP_FILE")
  passed=0
  failed=0
  progress_detected=false

  for i in $(seq 0 $((sources_count - 1))); do
    src_type=$(jq -r ".feedback_sources[$i].type" "$LOOP_FILE")
    interpret=$(jq -r ".feedback_sources[$i].interpret // \"exit-code\"" "$LOOP_FILE")
    label="source[$i]:${src_type}"

    case "$src_type" in
      command|gh-query)
        cmd=$(jq -r ".feedback_sources[$i].run" "$LOOP_FILE")
        echo "  Evaluating ${label}: ${cmd}"
        if [[ "$DRY_RUN" == "true" ]]; then
          echo "  [dry-run] skipping command"
          result_exit=0
        else
          result_exit=0
          eval "$cmd" >/dev/null 2>&1 || result_exit=$?
        fi
        if [[ "$interpret" == "exit-code" ]]; then
          [[ $result_exit -eq 0 ]] && { passed=$((passed+1)); progress_detected=true; } || failed=$((failed+1))
        else
          # count-delta and others: treat exit-0 as progress signal
          [[ $result_exit -eq 0 ]] && { passed=$((passed+1)); progress_detected=true; } || failed=$((failed+1))
        fi
        ;;
      file)
        path=$(jq -r ".feedback_sources[$i].path" "$LOOP_FILE")
        echo "  Evaluating ${label}: ${path}"
        [[ -f "$path" ]] && { passed=$((passed+1)); progress_detected=true; } || failed=$((failed+1))
        ;;
      url)
        url=$(jq -r ".feedback_sources[$i].fetch" "$LOOP_FILE")
        echo "  Evaluating ${label}: ${url}"
        if [[ "$DRY_RUN" == "true" ]]; then
          echo "  [dry-run] skipping HTTP fetch"
          passed=$((passed+1))
          progress_detected=true
        else
          http_code=$(curl -s -o /dev/null -w "%{http_code}" --max-time 10 "$url" 2>/dev/null || echo "000")
          [[ "$http_code" =~ ^2 ]] && { passed=$((passed+1)); progress_detected=true; } || failed=$((failed+1))
        fi
        ;;
      *)
        echo "  WARN: unknown feedback source type '${src_type}'" >&2
        ;;
    esac
  done

  echo "  Feedback: ${passed} passed, ${failed} failed"

  # Check goal satisfaction (all criteria passing = goal met)
  goal_satisfied_flag=$(jq -r '.termination.goal_satisfied // true' "$LOOP_FILE")
  if [[ "$goal_satisfied_flag" == "true" && $failed -eq 0 && $sources_count -gt 0 ]]; then
    echo "TERMINATE: all measurable criteria satisfied"
    new_status="completed"
    outcome="terminate"
    reason="goal satisfied"
    new_no_progress=0
  elif [[ $progress_detected == false ]]; then
    new_no_progress=$((no_progress + 1))
    echo "  No progress detected (${new_no_progress}/${max_no_progress} consecutive)"
    if [[ $new_no_progress -ge $max_no_progress ]]; then
      echo "ESCALATE: max_no_progress_ticks (${max_no_progress}) reached"
      new_status="escalated"
      outcome="escalate"
      reason="max_no_progress_ticks reached"
    else
      new_status="active"
      outcome="continue"
      reason="${failed} feedback sources still failing"
    fi
  else
    new_no_progress=0
    new_status="active"
    outcome="continue"
    reason="progress detected (${passed}/${sources_count} passing)"
  fi
fi

timestamp=$(date -u +"%Y-%m-%dT%H:%M:%SZ")

# Update loop.json
if [[ "$DRY_RUN" == "false" ]]; then
  updated_json=$(jq \
    --argjson tick "$new_tick" \
    --argjson np "${new_no_progress:-0}" \
    --arg status "$new_status" \
    --arg ts "$timestamp" \
    '.current_tick = $tick | .no_progress_ticks = $np | .status = $status | .last_tick_at = $ts' \
    "$LOOP_FILE")
  echo "$updated_json" > "$LOOP_FILE"

  # Append to journal
  mkdir -p "$LOOPS_DIR"
  {
    echo ""
    echo "## Tick ${new_tick} — ${timestamp}"
    echo ""
    echo "**Outcome:** ${outcome} (${reason})"
    echo "**Feedback:** ${passed:-0} passed / ${failed:-0} failed"
    echo "**Evolution:** ${evolution_name}"
    echo ""
  } >> "$JOURNAL"

  # Append to decision log
  {
    echo "| ${timestamp} | tick-${new_tick} | ${outcome} | ${reason} |"
  } >> "$DECISION_LOG"
fi

echo ""
echo "Tick ${new_tick} complete — outcome: ${outcome} (${reason})"
echo "Next step: ${outcome}"

case "$outcome" in
  terminate) exit 2 ;;
  escalate)  exit 1 ;;
  continue)  exit 0 ;;
esac
