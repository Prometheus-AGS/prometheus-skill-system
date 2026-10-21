#!/usr/bin/env bash
# kbd-phase-readiness-check.sh — deterministic Plan→Execute readiness gate.
# Read-only against canonical state; never writes success evidence, never
# fabricates PASS. Emits exactly READY or a rejection with the minimal
# missing/failed list.
#
# Usage:
#   kbd-phase-readiness-check.sh --phase <phase-id>
#       [--phase-dir <dir>]      fixture override (scratch trees)
#       [--status-file <f>]      fixture override for `prometheus kbd status --json`
#       [--contract <f>]         fixture override for the contract path
#       [--disposition-file <f>] fixture override for `prometheus kbd phase disposition --json`
#
# A phase bound in canonical state (issue #192) is checked against its bound
# digest; decision-summary text is consulted only for unbound legacy phases.
#
# Checks (in order; all failures collected where recoverable):
#   contract   adopted-contract.json exists + passes structural validation
#   identity   contract projectId/runId/phaseId agree with canonical status
#   adoption   latest non-superseded adoption decision for the contract path
#              carries the same digest (frozen normalization) — mismatch or
#              superseded => REJECT-STALE; absent => BLOCKED (adoption)
#   coverage   every scenarioMatrix ownerTask exists in the phase tasks.json
#   quality    unique scenario ids; per-scenario productionEntry/oracle/
#              negativeControl/ownerTask/evidence/fixtures non-empty;
#              userOperation, exclusions, requiredSafetyChecks present (B1)
#   budgets    required budget keys valid; UNKNOWN token budget has fallback
#   packet     boundaryPacket complete (postImplementationOnly, commands,
#              stopConditions)
#
# Exit codes: 0 READY; 1 usage/io; 2 BLOCKED (minimal missing list on stdout);
#             3 REJECT-IDENTITY; 4 REJECT-STALE; 5 contract malformed.
set -euo pipefail

PHASE=""; PHASE_DIR=""; STATUS_FILE=""; CONTRACT=""; DISPOSITION_FILE=""
while [ $# -gt 0 ]; do
  case "$1" in
    --phase) PHASE="${2:?}"; shift 2 ;;
    --disposition-file) DISPOSITION_FILE="${2:?}"; shift 2 ;;
    --phase-dir) PHASE_DIR="${2:?}"; shift 2 ;;
    --status-file) STATUS_FILE="${2:?}"; shift 2 ;;
    --contract) CONTRACT="${2:?}"; shift 2 ;;
    -h|--help) sed -n '1,24p' "$0"; exit 0 ;;
    *) printf 'kbd-phase-readiness-check: unknown argument %s\n' "$1" >&2; exit 1 ;;
  esac
done
command -v jq >/dev/null || { printf 'jq is required\n' >&2; exit 1; }

ROOT="$PWD"
while [ -n "$ROOT" ] && [ "$ROOT" != "/" ] && [ ! -d "$ROOT/.kbd-orchestrator" ]; do ROOT="$(dirname "$ROOT")"; done
if [ -z "$PHASE_DIR" ]; then
  [ -n "$PHASE" ] || { printf 'usage: --phase <id> or --phase-dir <dir>\n' >&2; exit 1; }
  [ -d "$ROOT/.kbd-orchestrator" ] || { printf 'no .kbd-orchestrator above %s\n' "$PWD" >&2; exit 1; }
  PHASE_DIR="$ROOT/.kbd-orchestrator/phases/$PHASE"
fi
[ -d "$PHASE_DIR" ] || { printf 'BLOCKED missing=phase-dir(%s)\n' "$PHASE_DIR"; exit 2; }
[ -n "$PHASE" ] || PHASE="$(basename "$PHASE_DIR")"
[ -n "$CONTRACT" ] || CONTRACT="$PHASE_DIR/adopted-contract.json"
case "$CONTRACT" in /*) ;; *) CONTRACT="$PWD/$CONTRACT" ;; esac

# --- contract presence + structure ---
[ -f "$CONTRACT" ] || { printf 'BLOCKED missing=contract(%s)\n' "$CONTRACT"; exit 2; }
if ! jq empty "$CONTRACT" 2>/dev/null; then
  printf 'MALFORMED contract(%s): not valid JSON\n' "$CONTRACT" >&2; exit 5
fi
SCHEMA_ERR="$(jq -r '
  [
    (if .kind == "kbd.adopted-contract" then empty else "kind" end),
    (if (.identity.projectId // "" | length) > 0 then empty else "identity.projectId" end),
    (if (.identity.runId // "" | length) > 0 then empty else "identity.runId" end),
    (if (.identity.phaseId // "" | length) > 0 then empty else "identity.phaseId" end),
    (if .digestNormalization.algorithm == "sha256" then empty else "digestNormalization.algorithm" end),
    (if (.scenarioMatrix // [] | length) > 0 then empty else "scenarioMatrix" end),
    (if (([.scenarioMatrix[]?.id] | unique | length) == (.scenarioMatrix | length)) then empty else "scenarioMatrix-duplicate-ids" end),
    (if ([.scenarioMatrix[]? | select((.productionEntry // "" | length) == 0 or (.oracle // "" | length) == 0 or (.negativeControl // "" | length) == 0 or (.ownerTask // "" | length) == 0 or (.evidence // "" | length) == 0 or (.fixtures // "" | length) == 0)] | length) == 0 then empty else "scenarioMatrix-quality(productionEntry/oracle/negativeControl/ownerTask/evidence/fixtures)" end),
    (if (.releaseClaim.userOperation // "" | length) >= 10 then empty else "releaseClaim.userOperation" end),
    (if (.exclusions // [] | length) > 0 then empty else "exclusions" end),
    (if (.requiredSafetyChecks // [] | length) > 0 then empty else "requiredSafetyChecks" end),
    (if (.budgets.timeMinutes // 0) >= 1 and (.budgets.toolCalls // 0) >= 1 and (.budgets.noProgressLimit // 0) >= 1 and (.budgets.repairAttempts // -1) >= 0 then empty else "budgets" end),
    (if .budgets.tokenBudget == "UNKNOWN" and ((.budgets.tokenFallback // "") | length) == 0 then "budgets.tokenFallback" else empty end),
    (if .boundaryPacket.postImplementationOnly == true and (.boundaryPacket.commands // [] | length) > 0 and (.boundaryPacket.stopConditions // [] | length) > 0 then empty else "boundaryPacket" end)
  ] | map(select(. != null)) | join(",")
' "$CONTRACT")"
if [ -n "$SCHEMA_ERR" ]; then
  printf 'MALFORMED contract(%s): %s\n' "$CONTRACT" "$SCHEMA_ERR" >&2; exit 5
fi

# --- canonical status (real or fixture) ---
if [ -n "$STATUS_FILE" ]; then
  STATUS_JSON="$(cat "$STATUS_FILE")"
else
  STATUS_JSON="$(prometheus kbd --path "$ROOT" status --json 2>/dev/null)" || {
    printf 'BLOCKED missing=canonical-status(prometheus kbd status failed)\n'; exit 2; }
fi

# --- identity ---
C_PROJECT="$(jq -r '.identity.projectId' "$CONTRACT")"
C_RUN="$(jq -r '.identity.runId' "$CONTRACT")"
C_PHASE="$(jq -r '.identity.phaseId' "$CONTRACT")"
S_PROJECT="$(printf '%s' "$STATUS_JSON" | jq -r '.projectId // empty')"
S_RUN="$(printf '%s' "$STATUS_JSON" | jq -r '.runId // empty')"
IDENTITY_FAIL=""
[ "$C_PROJECT" = "$S_PROJECT" ] || IDENTITY_FAIL="projectId(contract=$C_PROJECT canonical=$S_PROJECT)"
[ "$C_RUN" = "$S_RUN" ] || IDENTITY_FAIL="${IDENTITY_FAIL:+$IDENTITY_FAIL,}runId(contract=$C_RUN canonical=$S_RUN)"
[ "$C_PHASE" = "$PHASE" ] || IDENTITY_FAIL="${IDENTITY_FAIL:+$IDENTITY_FAIL,}phaseId(contract=$C_PHASE gate=$PHASE)"
if [ -n "$IDENTITY_FAIL" ]; then
  printf 'REJECT-IDENTITY %s\n' "$IDENTITY_FAIL"; exit 3
fi

# --- adoption + staleness ---
REL="${CONTRACT#"$ROOT"/}"
MISSING=""
C_DIGEST="$(jq -S -c 'del(.identity.contractDigest) | del(.identity.planRevision)' "$CONTRACT" | shasum -a 256 | awk '{print $1}')"
DISP_JSON=""
if [ -n "$DISPOSITION_FILE" ]; then
  DISP_JSON="$(cat "$DISPOSITION_FILE" 2>/dev/null || true)"
elif [ -z "$STATUS_FILE" ]; then
  DISP_JSON="$(prometheus kbd --path "$ROOT" phase disposition --id "$PHASE" --json 2>/dev/null || true)"
fi
B_DIGEST="$(printf '%s' "$DISP_JSON" | jq -r 'select(.schema == "kbd.phase-disposition/v1") | .binding.contractDigest // empty' 2>/dev/null || true)"
if [ -n "$B_DIGEST" ]; then
  if [ "$B_DIGEST" != "$C_DIGEST" ]; then
    printf 'REJECT-STALE contractDigest(bound=%s contract=%s) — a bound contract changes only through a signed amendment (kbd-contract-adopt.sh --supersedes --amendment --signature)\n' \
      "$B_DIGEST" "$C_DIGEST"
    exit 4
  fi
  LATEST="BOUND"
else
LATEST="$(printf '%s' "$STATUS_JSON" | jq -r --arg path "$REL" '
  [(.decisions // {}) | to_entries[] | select(.value.summary | contains("ADOPTED-CONTRACT path=" + $path + " "))] as $adoptions
  | [$adoptions[] | select(.value as $d | [$adoptions[] | select(.value.supersedes == $d.id)] | length == 0)]
  | if length == 0 then "" elif length == 1 then .[0].value | .id + " " + (try (.summary | capture("sha256=(?<d>[0-9a-f]{64})") | .d) catch "NODIGEST") + " " + ((.planRevision // "null") | tostring) else "AMBIGUOUS" end
')"
fi
if [ "$LATEST" = "AMBIGUOUS" ]; then
  printf 'BLOCKED missing=adoption-unambiguous(multiple non-superseded adoption decisions for %s)\n' "$REL"; exit 2
fi
if [ -z "$LATEST" ]; then
  MISSING="adoption(no canonical adoption decision for $REL)"
elif [ "$LATEST" != "BOUND" ]; then
  L_DIGEST="$(printf '%s' "$LATEST" | awk '{print $2}')"
  L_REV="$(printf '%s' "$LATEST" | awk '{print $3}')"
  if [ "$L_DIGEST" != "$C_DIGEST" ]; then
    printf 'REJECT-STALE contractDigest(decision=%s contract=%s) planRevision(decision=%s) — amend via kbd-contract-adopt.sh --supersedes\n' \
      "$L_DIGEST" "$C_DIGEST" "$L_REV"
    exit 4
  fi
fi

# --- scenario coverage vs tasks.json ---
TASKS_FILE="$PHASE_DIR/tasks.json"
if [ -f "$TASKS_FILE" ]; then
  COV_MISSING="$(jq -r --slurpfile tasks "$TASKS_FILE" '
    [$tasks[0].tasks[].id] as $have
    | [.scenarioMatrix[] | . as $s | (.ownerTask | split("+"))[]
       | select(. as $t | [$have[] | select(. == $t)] | length == 0)
       | "\($s.id):\(.)"]
    | join(",")
  ' "$CONTRACT" 2>/dev/null)"
  [ -z "$COV_MISSING" ] || MISSING="${MISSING:+$MISSING,}coverage($COV_MISSING)"
else
  MISSING="${MISSING:+$MISSING,}coverage(tasks.json absent)"
fi

# --- verdict ---
if [ -n "$MISSING" ]; then
  printf 'BLOCKED missing=%s\n' "$MISSING"
  exit 2
fi
printf 'READY phase=%s contract=%s digest=%s planRevision=%s\n' \
  "$PHASE" "$REL" "${C_DIGEST:-unknown}" "$(printf '%s' "$STATUS_JSON" | jq -r '.planRevision // "?"')"
exit 0
