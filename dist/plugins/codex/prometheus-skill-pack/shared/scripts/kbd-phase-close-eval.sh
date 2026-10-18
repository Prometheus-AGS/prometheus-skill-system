#!/usr/bin/env bash
# kbd-phase-close-eval.sh — terminal evaluator at the phase close/continue
# boundary (issue #184 subset, amendment v3 behavior B2).
#
# Consumes the phase's adopted scenario RESULTS evidence
# (boundary-results.json) — never task counters, implementationComplete, or
# READY — and records a truthful canonical disposition:
#
#   PASS    every adopted scenario has valid PASS evidence with bound identity
#           => close the phase via the canonical writer (PHASE-CLOSED);
#              re-evaluation is ALREADY-CLOSED only when a closure bound to
#              this phase + run + adopted contract digest exists (validated
#              first); a closed phase refuses further execute dispatch.
#   FAIL    a product oracle was unmet after valid execution
#           => PHASE-CLOSURE-REFUSED verdict=FAIL, stop auto-continue.
#   BLOCKED evidence missing/stale/invalid, validator/env unavailable
#           => PHASE-CLOSURE-REFUSED verdict=BLOCKED, stop auto-continue.
#
# NOT_EXERCISED never counts as PASS. Refusals never mark success and never
# close. The evaluator creates no tasks and no successor phases.
#
# Usage:
#   kbd-phase-close-eval.sh --phase <id>
#       [--evidence-file <f>]   default <phase-dir>/boundary-results.json
#       [--status-file <f>]     fixture override for `prometheus kbd status --json`
#       [--decision-sink <f>]   append disposition lines to a file instead of the
#                               canonical writer (scratch isolation; disclosed seam)
#
# Exit codes: 0 closed/already-closed; 1 usage/io; 2 BLOCKED refusal;
#             3 FAIL refusal.
set -euo pipefail

PHASE=""; EVIDENCE=""; STATUS_FILE=""; SINK=""
while [ $# -gt 0 ]; do
  case "$1" in
    --phase) PHASE="${2:?}"; shift 2 ;;
    --evidence-file) EVIDENCE="${2:?}"; shift 2 ;;
    --status-file) STATUS_FILE="${2:?}"; shift 2 ;;
    --decision-sink) SINK="${2:?}"; shift 2 ;;
    -h|--help) sed -n '1,30p' "$0"; exit 0 ;;
    *) printf 'kbd-phase-close-eval: unknown argument %s\n' "$1" >&2; exit 1 ;;
  esac
done
[ -n "$PHASE" ] || { printf 'usage: kbd-phase-close-eval.sh --phase <id>\n' >&2; exit 1; }
command -v jq >/dev/null || { printf 'jq is required\n' >&2; exit 1; }

ROOT="$PWD"
while [ -n "$ROOT" ] && [ "$ROOT" != "/" ] && [ ! -d "$ROOT/.kbd-orchestrator" ]; do ROOT="$(dirname "$ROOT")"; done
[ -d "$ROOT/.kbd-orchestrator" ] || { printf 'no .kbd-orchestrator above %s\n' "$PWD" >&2; exit 1; }
PHASE_DIR="$ROOT/.kbd-orchestrator/phases/$PHASE"
[ -d "$PHASE_DIR" ] || { printf 'BLOCKED missing=phase-dir(%s)\n' "$PHASE_DIR"; exit 2; }
CONTRACT="$PHASE_DIR/adopted-contract.json"
[ -f "$CONTRACT" ] || { printf 'BLOCKED missing=contract(%s)\n' "$CONTRACT"; exit 2; }
[ -n "$EVIDENCE" ] || EVIDENCE="$PHASE_DIR/boundary-results.json"

if [ -n "$STATUS_FILE" ]; then
  STATUS_JSON="$(cat "$STATUS_FILE")"
else
  STATUS_JSON="$(prometheus kbd --path "$ROOT" status --json 2>/dev/null)" || {
    printf 'BLOCKED missing=canonical-status(prometheus kbd status failed)\n'; exit 2; }
fi

record() { # <id> <command-id> <summary> — canonical writer or disclosed sink
  if [ -n "$SINK" ]; then
    printf '%s\n' "$3" >> "$SINK"
  else
    prometheus kbd --path "$ROOT" decision record --command-id "$2" --id "$1" --summary "$3" >/dev/null
  fi
}

refuse() { # <verdict:FAIL|BLOCKED> <exit> <reason>
  local ts
  ts="$(date -u +%Y%m%dT%H%M%SZ)"
  record "phase-closure-refused-$PHASE-$ts" "phase-close-refused-$PHASE-$ts" \
    "PHASE-CLOSURE-REFUSED phase=$PHASE verdict=$1 reason=$3" || true
  printf '%s phase=%s reason=%s\n' "$1" "$PHASE" "$3"
  exit "$2"
}

# --- evidence presence + JSON validity ---
[ -f "$EVIDENCE" ] || refuse BLOCKED 2 "evidence-missing($EVIDENCE)"
if ! jq empty "$EVIDENCE" 2>/dev/null; then
  refuse BLOCKED 2 "evidence-invalid-json($EVIDENCE)"
fi

# --- identity binding: phase, run, adopted contract digest ---
EV_PHASE="$(jq -r '.phase // empty' "$EVIDENCE")"
[ "$EV_PHASE" = "$PHASE" ] || refuse BLOCKED 2 "evidence-phase-mismatch(evidence=$EV_PHASE gate=$PHASE)"
EV_RUN="$(jq -r '.runId // empty' "$EVIDENCE")"
S_RUN="$(printf '%s' "$STATUS_JSON" | jq -r '.runId // empty')"
[ -n "$EV_RUN" ] && [ "$EV_RUN" = "$S_RUN" ] || refuse BLOCKED 2 "evidence-run-mismatch(evidence=$EV_RUN canonical=$S_RUN)"
REL="${CONTRACT#"$ROOT"/}"
L_DIGEST="$(printf '%s' "$STATUS_JSON" | jq -r --arg path "$REL" '
  [(.decisions // {}) | to_entries[] | select(.value.summary | contains("ADOPTED-CONTRACT path=" + $path + " "))] as $a
  | [$a[] | select(.value as $d | [$a[] | select(.value.supersedes == $d.id)] | length == 0)]
  | if length == 1 then .[0].value.summary | (try (capture("sha256=(?<d>[0-9a-f]{64})") | .d) catch "") else "" end')"
EV_CDIGEST="$(jq -r '.contractDigest // empty' "$EVIDENCE")"
C_DIGEST="$(jq -S -c 'del(.identity.contractDigest) | del(.identity.planRevision)' "$CONTRACT" | shasum -a 256 | awk '{print $1}')"
[ -n "$L_DIGEST" ] || refuse BLOCKED 2 "adoption-missing($REL)"
[ "$L_DIGEST" = "$C_DIGEST" ] || refuse BLOCKED 2 "adoption-stale(decision=$L_DIGEST contract=$C_DIGEST)"
[ "$EV_CDIGEST" = "$C_DIGEST" ] || refuse BLOCKED 2 "evidence-contract-digest-mismatch(evidence=$EV_CDIGEST adopted=$C_DIGEST)"

# --- freshness: recompute digests of the production sources under evidence ---
SRC_CHECK="$(jq -r '.sourceDigests // {} | to_entries[] | .key + " " + .value' "$EVIDENCE" 2>/dev/null)"
[ -n "$SRC_CHECK" ] || refuse BLOCKED 2 "evidence-source-digests-missing"
STALE=""
while IFS=' ' read -r path digest; do
  [ -n "$path" ] || continue
  if [ ! -f "$ROOT/$path" ]; then STALE="${STALE:+$STALE,}missing($path)"; continue; fi
  NOW="$(shasum -a 256 "$ROOT/$path" | awk '{print $1}')"
  [ "$NOW" = "$digest" ] || STALE="${STALE:+$STALE,}stale($path)"
done <<< "$SRC_CHECK"
[ -z "$STALE" ] || refuse BLOCKED 2 "evidence-stale-sources($STALE)"

# --- entry-level oracle: every adopted scenario present, every entry valid PASS ---
REQUIRED="$(jq -r '[.scenarioMatrix[].id] | join(" ")' "$CONTRACT")"
ANALYSIS="$(jq -r --arg req "$REQUIRED" '
  ($req | split(" ")) as $required
  | (.scenarios // []) as $entries
  | [ $required[] as $s | select([$entries[] | select(.id == $s or (.id | startswith($s + "-")))] | length == 0) | "missing-scenario:" + $s ] as $missing
  | [ $entries[] | select((.attempted // "" | length) == 0 or (.observed // "" | length) == 0 or (.evidence // "" | length) == 0) | "empty-evidence:" + .id ] as $empty
  | [ $entries[] | select(.status == "FAIL") | .id ] as $failed
  | [ $entries[] | select(.status != "PASS" and .status != "FAIL") | "not-pass:" + .id + "(" + (.status // "null") + ")" ] as $notpass
  | [$missing[], $empty[]] as $blocked1
  | if ($failed | length) > 0 then "FAIL " + ($failed | join(","))
     elif ($blocked1 | length) > 0 then "BLOCKED " + ($blocked1 | join(","))
     elif ($notpass | length) > 0 then "BLOCKED " + ($notpass | join(","))
     else "PASS" end
' "$EVIDENCE")"
case "$ANALYSIS" in
  FAIL\ *)    refuse FAIL 3 "product-oracle-unmet(${ANALYSIS#FAIL })" ;;
  BLOCKED\ *) refuse BLOCKED 2 "${ANALYSIS#BLOCKED }" ;;
  PASS) ;;
  *)          refuse BLOCKED 2 "evidence-analysis-error" ;;
esac

# --- idempotence (only AFTER full validation above): a historical closure
# --- authorizes ALREADY-CLOSED solely when bound to THIS phase + THIS run +
# --- THIS adopted contract digest. A different run or a superseded contract
# --- cannot inherit a same-phase historical PASS; stale/tampered evidence is
# --- refused earlier and never reaches this scan.
CLOSED="$(printf '%s' "$STATUS_JSON" | jq -r --arg p "$PHASE" --arg r "$S_RUN" --arg c "$C_DIGEST" '
  [(.decisions // {})[] | select(.summary | contains("PHASE-CLOSED phase=" + $p + " verdict=PASS")
                                 and contains("run=" + $r) and contains("contract=" + $c))] | length')"
if [ "$CLOSED" -gt 0 ]; then
  printf 'ALREADY-CLOSED phase=%s run=%s contract=%s (bound idempotent no-op; zero new tasks, zero new dispatch)\n' "$PHASE" "$S_RUN" "${C_DIGEST:0:12}"
  exit 0
fi

# --- budget predicate (declared ceilings vs self-reported consumption) -------
# Ceilings come from the adopted contract: base budgets plus additive
# extensions; the latest extension's explicit resultingCeilings wins when
# present. Consumption comes from the evidence's budgetConsumption header,
# which must name its source and class each value measured/estimate/unknown.
# It is self-reported accounting, NOT independent telemetry: this predicate
# proves the declared-limit comparison and refusal behavior only. Consumption
# ABOVE a ceiling refuses closure; consumption AT a ceiling permits closing
# work already completed within limits and records that no further work is
# admitted; unknown-class values are not enforced (declared fallback governs).
BUDGET_NOTE=""
if jq -e '.budgetConsumption' "$EVIDENCE" >/dev/null 2>&1; then
  BC_SRC="$(jq -r '.budgetConsumption.source // ""' "$EVIDENCE")"
  [ -n "$BC_SRC" ] || refuse BLOCKED 2 "evidence-budget-source-missing"
  CEILINGS="$(jq -c '
    .budgets as $b
    | ([.budgets.extensions[]? | select(.resultingCeilings != null) | .resultingCeilings] | last) as $rc
    | if $rc != null then $rc
      else { timeMinutes: ($b.timeMinutes + ([.budgets.extensions[]?.timeMinutes // 0] | add)),
             toolCalls: ($b.toolCalls + ([.budgets.extensions[]?.toolCalls // 0] | add)),
             repairAttempts: ($b.repairAttempts + ([.budgets.extensions[]?.repairAttempts // 0] | add)),
             noProgressLimit: $b.noProgressLimit } end' "$CONTRACT")"
  BUDGET_CHECK="$(jq -r --argjson c "$CEILINGS" '
    (.budgetConsumption.values // {}) as $v
    | { timeMinutes: "timeMinutes", toolCalls: "toolCalls", repairAttempts: "repairAttempts", noProgress: "noProgressLimit" } as $map
    | [ $map | keys[] | . as $d
        | ($v[$d] // empty) as $val
        | select($val != null and ($val.class // "unknown") != "unknown" and ($val.value // null) != null)
        | ($c[$map[$d]] // 0) as $ceil
        | if $val.value > $ceil then "exceeded:" + $d + "(" + ($val.value | tostring) + ">" + ($ceil | tostring) + "," + $val.class + ")"
          elif $val.value == $ceil then "at-ceiling:" + $d
          else empty end ] as $hits
    | ([$hits[] | select(startswith("exceeded:"))]) as $ex
    | ([$hits[] | select(startswith("at-ceiling:"))]) as $at
    | if ($ex | length) > 0 then "EXCEEDED " + ($ex | join(","))
      elif ($at | length) > 0 then "ATCEILING " + ($at | join(","))
      else "OK" end' "$EVIDENCE")"
  case "$BUDGET_CHECK" in
    EXCEEDED\ *) refuse BLOCKED 2 "budget-exceeded(${BUDGET_CHECK#EXCEEDED }) ceilings=$CEILINGS source=$BC_SRC" ;;
    ATCEILING\ *) BUDGET_NOTE=" budget-note=allowance-fully-consumed(${BUDGET_CHECK#ATCEILING });no-further-work-admitted" ;;
    OK) ;;
  esac
else
  # This contract declares budgets (checker-enforced), so budget evaluation is
  # required: missing accounting is a BLOCKED refusal, never a qualified PASS.
  # (Legacy closures recorded without budget semantics stay labelled as such in
  # history and do not claim Q2 satisfaction.)
  refuse BLOCKED 2 "budget-consumption-missing(contract requires budget evaluation: evidence needs budgetConsumption with source and measured/estimate/unknown classes)"
fi

# --- close the phase (canonical writer; append-only; no tasks, no successor) ---
EV_DIGEST="$(shasum -a 256 "$EVIDENCE" | awk '{print $1}')"
N_ENTRIES="$(jq '.scenarios | length' "$EVIDENCE")"
record "phase-closure-$PHASE-${EV_DIGEST:0:8}" "phase-close-$PHASE-${EV_DIGEST:0:8}" \
  "PHASE-CLOSED phase=$PHASE verdict=PASS evidence=sha256:$EV_DIGEST contract=$C_DIGEST run=$S_RUN entries=$N_ENTRIES$BUDGET_NOTE"
printf 'CLOSED phase=%s verdict=PASS evidence=sha256:%s entries=%s (dispatch for this phase now refused)\n' \
  "$PHASE" "$EV_DIGEST" "$N_ENTRIES"
exit 0
