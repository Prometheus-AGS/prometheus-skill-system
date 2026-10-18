#!/usr/bin/env bash
# kbd-contract-adopt.sh — adopt a phase's completion contract through the
# existing canonical writer (prometheus kbd decision record). Canonical writer
# only: never hand-writes journals, waypoints, or signatures.
#
# Usage:
#   kbd-contract-adopt.sh <contract.json> [--supersedes <decision-id>] [--command-id <id>]
#
# Behavior:
#   1. Validate the contract structurally (schema-lite; full schema:
#      skills/process/kbd-process-orchestrator/references/schemas/adopted-contract.schema.json).
#   2. Cross-check identity against canonical state (projectId, runId, phaseId).
#   3. Compute the digest with the FROZEN normalization:
#        jq -S -c 'del(.identity.contractDigest) | del(.identity.planRevision)'
#      (contractDigest excluded to avoid self-referential recursion;
#       planRevision excluded because the canonical Decision binds it.)
#   4. Idempotence: if the latest non-superseded adoption decision for this
#      contract path already carries this digest, report ALREADY-ADOPTED and
#      exit 0 without recording a duplicate.
#   5. Stamp identity.planRevision + identity.contractDigest (digest-neutral),
#      then record the adoption decision. Amendments require explicit
#      --supersedes <prior-decision-id>; history is append-only.
#
# Exit codes: 0 adopted/already-adopted; 1 usage/io; 2 invalid contract;
#             3 identity mismatch; 4 amendment without --supersedes.
set -euo pipefail

fail() { printf 'kbd-contract-adopt: %s\n' "$1" >&2; exit "${2:-1}"; }
command -v jq >/dev/null || fail "jq is required" 1
command -v prometheus >/dev/null || fail "prometheus CLI is required" 1

CONTRACT=""
SUPERSEDES=""
COMMAND_ID=""
while [ $# -gt 0 ]; do
  case "$1" in
    --supersedes) SUPERSEDES="${2:?--supersedes needs a decision id}"; shift 2 ;;
    --command-id) COMMAND_ID="${2:?--command-id needs a value}"; shift 2 ;;
    -h|--help) sed -n '1,30p' "$0"; exit 0 ;;
    -*) fail "unknown flag: $1" 1 ;;
    *) [ -z "$CONTRACT" ] && CONTRACT="$1" || fail "unexpected argument: $1" 1; shift ;;
  esac
done
[ -n "$CONTRACT" ] || fail "usage: kbd-contract-adopt.sh <contract.json> [--supersedes <id>]" 1
[ -f "$CONTRACT" ] || fail "contract not found: $CONTRACT" 1

# Repo root: walk up for .kbd-orchestrator.
ROOT="$PWD"
while [ -n "$ROOT" ] && [ "$ROOT" != "/" ] && [ ! -d "$ROOT/.kbd-orchestrator" ]; do ROOT="$(dirname "$ROOT")"; done
[ -d "$ROOT/.kbd-orchestrator" ] || fail "no .kbd-orchestrator found above $PWD" 1
case "$CONTRACT" in /*) ;; *) CONTRACT="$PWD/$CONTRACT" ;; esac
REL="${CONTRACT#"$ROOT"/}"
[ "$REL" != "$CONTRACT" ] || fail "contract must live inside the repo ($ROOT)" 1

jq empty "$CONTRACT" 2>/dev/null || fail "contract is not valid JSON: $CONTRACT" 2

# --- 1. Structural validation (schema-lite, matches adopted-contract.schema.json required set) ---
SCHEMA_ERR="$(jq -r '
  def needstr(p): if (getpath(p) // "" | type) == "string" and (getpath(p) | length) > 0 then empty else "missing:" + (p | join(".")) end;
  [
    needstr(["contractVersion"]),
    (if .kind == "kbd.adopted-contract" then empty else "kind-mismatch" end),
    needstr(["identity","projectId"]), needstr(["identity","runId"]), needstr(["identity","phaseId"]),
    (if .identity.contractDigest == null or (.identity.contractDigest | test("^[0-9a-f]{64}$")) then empty else "bad-contractDigest-format" end),
    needstr(["digestNormalization","command"]), needstr(["digestNormalization","stalenessRule"]),
    (if .digestNormalization.algorithm == "sha256" then empty else "digest-algorithm-mismatch" end),
    needstr(["adoptionSource"]), needstr(["decisionOwner"]), needstr(["amendmentPolicy"]),
    needstr(["releaseClaim","userOperation"]),
    (if (.releaseClaim.supportedEnvironments // [] | length) > 0 then empty else "missing:releaseClaim.supportedEnvironments" end),
    needstr(["releaseClaim","currentCapabilities"]),
    (if (.statusVocabulary // [] | length) >= 4 then empty else "statusVocabulary-too-short" end),
    (if (.scenarioMatrix // [] | length) > 0 then empty else "scenarioMatrix-empty" end),
    (if ([.scenarioMatrix[]? | select((.id // "" | test("^[A-Z]+-[0-9]+$")) | not)] | length) == 0 then empty else "scenario-id-pattern" end),
    (if ([.scenarioMatrix[]? | select((.productionEntry // "" | length) == 0 or (.oracle // "" | length) == 0 or (.negativeControl // "" | length) == 0 or (.ownerTask // "" | length) == 0 or (.evidence // "" | length) == 0)] | length) == 0 then empty else "scenario-fields-missing" end),
    (if (.functionalScope // [] | length) > 0 then empty else "functionalScope-empty" end),
    (if (.exclusions // [] | length) > 0 then empty else "exclusions-empty" end),
    (if (.requiredSafetyChecks // [] | length) > 0 then empty else "requiredSafetyChecks-empty" end),
    (if (.budgets.timeMinutes // 0) >= 1 and (.budgets.toolCalls // 0) >= 1 and (.budgets.repairAttempts // -1) >= 0 and (.budgets.noProgressLimit // 0) >= 1 then empty else "budgets-invalid" end),
    (if (.budgets.tokenBudget // "" | length) > 0 then empty else "missing:budgets.tokenBudget" end),
    (if .budgets.tokenBudget == "UNKNOWN" and ((.budgets.tokenFallback // "") | length) == 0 then "tokenBudget-UNKNOWN-needs-fallback" else empty end),
    needstr(["budgets","exhaustion"]), needstr(["evidenceInvalidation"]),
    needstr(["workerRoutes","agent"]), needstr(["workerRoutes","model"]),
    (if .boundaryPacket.postImplementationOnly == true then empty else "boundaryPacket.postImplementationOnly-must-be-true" end),
    (if .boundaryPacket.singleRun == true then empty else "boundaryPacket.singleRun-must-be-true" end),
    (if (.boundaryPacket.commands // [] | length) > 0 then empty else "boundaryPacket.commands-empty" end),
    (if (.boundaryPacket.stopConditions // [] | length) > 0 then empty else "boundaryPacket.stopConditions-empty" end),
    (if (.status.implementationComplete | type) == "boolean" then empty else "missing:status.implementationComplete" end)
  ] | map(select(. != null)) | join("\n")
' "$CONTRACT")"
if [ -n "$SCHEMA_ERR" ]; then
  printf 'kbd-contract-adopt: contract fails structural validation:\n%s\n' "$SCHEMA_ERR" >&2
  exit 2
fi

PHASE_ID="$(jq -r '.identity.phaseId' "$CONTRACT")"
EXPECTED_DIR="$ROOT/.kbd-orchestrator/phases/$PHASE_ID"
[ "$(dirname "$CONTRACT")" = "$EXPECTED_DIR" ] || fail "identity.phaseId=$PHASE_ID but contract is not at $EXPECTED_DIR" 3

# --- 2. Canonical identity cross-check ---
STATUS_JSON="$(prometheus kbd --path "$ROOT" status --json 2>/dev/null)" || fail "prometheus kbd status failed" 1
CANON_PROJECT="$(printf '%s' "$STATUS_JSON" | jq -r '.projectId // empty')"
CANON_RUN="$(printf '%s' "$STATUS_JSON" | jq -r '.runId // empty')"
CANON_PLAN_REV="$(printf '%s' "$STATUS_JSON" | jq -r '.planRevision // empty')"
C_PROJECT="$(jq -r '.identity.projectId' "$CONTRACT")"
C_RUN="$(jq -r '.identity.runId' "$CONTRACT")"
[ "$C_PROJECT" = "$CANON_PROJECT" ] || fail "identity.projectId mismatch: contract=$C_PROJECT canonical=$CANON_PROJECT" 3
[ "$C_RUN" = "$CANON_RUN" ] || fail "identity.runId mismatch: contract=$C_RUN canonical=$CANON_RUN (stale run — amend the contract for the active run)" 3

# --- 3. Digest (frozen normalization; stamping fields excluded) ---
DIGEST="$(jq -S -c 'del(.identity.contractDigest) | del(.identity.planRevision)' "$CONTRACT" | shasum -a 256 | awk '{print $1}')"
[ -n "$DIGEST" ] || fail "digest computation failed" 1

# --- 4. Idempotence + amendment discipline ---
LATEST="$(printf '%s' "$STATUS_JSON" | jq -r --arg path "$REL" '
  [(.decisions // {}) | to_entries[] | select(.value.summary | contains("ADOPTED-CONTRACT path=" + $path + " "))] as $adoptions
  | [$adoptions[] | select(.value as $d | [$adoptions[] | select(.value.supersedes == $d.id)] | length == 0)]
  | if length == 0 then "" elif length == 1 then .[0].value | .id + " " + (try (.summary | capture("sha256=(?<d>[0-9a-f]{64})") | .d) catch "NODIGEST") else "AMBIGUOUS" end
')"
if [ "$LATEST" = "AMBIGUOUS" ]; then
  fail "multiple non-superseded adoption decisions for $REL — canonical state needs operator adjudication (prometheus kbd conflicts)" 4
fi
if [ -n "$LATEST" ]; then
  LATEST_ID="${LATEST%% *}"; LATEST_DIGEST="${LATEST##* }"
  if [ "$LATEST_DIGEST" = "$DIGEST" ]; then
    printf 'ALREADY-ADOPTED path=%s sha256=%s decision=%s (idempotent no-op)\n' "$REL" "$DIGEST" "$LATEST_ID"
    exit 0
  fi
  if [ -z "$SUPERSEDES" ]; then
    printf 'kbd-contract-adopt: contract changed (digest %s -> %s) but no --supersedes given.\n' "$LATEST_DIGEST" "$DIGEST" >&2
    printf 'Amendments require explicit operator version-adoption: re-run with --supersedes %s\n' "$LATEST_ID" >&2
    exit 4
  fi
  [ "$SUPERSEDES" = "$LATEST_ID" ] || fail "--supersedes $SUPERSEDES does not match latest adoption decision $LATEST_ID" 4
fi

# --- 5. Stamp + record (canonical writer only) ---
STAMPED="$(mktemp)"; trap 'rm -f "$STAMPED"' EXIT
jq --arg d "$DIGEST" --argjson rev "$CANON_PLAN_REV" \
  '.identity.planRevision = $rev | .identity.contractDigest = $d' "$CONTRACT" > "$STAMPED"
# Stamping must be digest-neutral:
DIGEST2="$(jq -S -c 'del(.identity.contractDigest) | del(.identity.planRevision)' "$STAMPED" | shasum -a 256 | awk '{print $1}')"
[ "$DIGEST" = "$DIGEST2" ] || fail "internal error: stamping changed the digest" 1
mv "$STAMPED" "$CONTRACT"; trap - EXIT

DECISION_ID="contract-adoption-${PHASE_ID}-${DIGEST:0:8}"
[ -n "$COMMAND_ID" ] || COMMAND_ID="contract-adopt-${PHASE_ID}-${DIGEST:0:8}"
SUMMARY="ADOPTED-CONTRACT path=$REL sha256=$DIGEST planRevision=$CANON_PLAN_REV adoptionSource=$(jq -r '.adoptionSource' "$CONTRACT") owner=$(jq -r '.decisionOwner' "$CONTRACT") scenarios=$(jq -r '[.scenarioMatrix[].id] | join(",")' "$CONTRACT")"
if [ -n "$SUPERSEDES" ]; then
  prometheus kbd --path "$ROOT" decision record --command-id "$COMMAND_ID" --id "$DECISION_ID" --summary "$SUMMARY" --supersedes "$SUPERSEDES" >/dev/null
else
  prometheus kbd --path "$ROOT" decision record --command-id "$COMMAND_ID" --id "$DECISION_ID" --summary "$SUMMARY" >/dev/null
fi
printf 'ADOPTED path=%s sha256=%s planRevision=%s decision=%s%s\n' \
  "$REL" "$DIGEST" "$CANON_PLAN_REV" "$DECISION_ID" "${SUPERSEDES:+ supersedes=$SUPERSEDES}"
