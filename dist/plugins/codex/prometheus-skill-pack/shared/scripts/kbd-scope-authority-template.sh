#!/usr/bin/env bash
# kbd-scope-authority-template.sh — print the canonical manifest an operator
# signs to authorize a material phase-scope change (issue #192). It NEVER
# signs: the signature is the operator's act, made with their own key.
#
# Usage:
#   kbd-scope-authority-template.sh amend-contract --phase <id> --contract <rel-path> \
#       --reason "<why>" [--approver <principal>]
#   kbd-scope-authority-template.sh start-successor --phase <closed-id> --successor <id> \
#       --title "<title>" --reason "<why>" [--approver <principal>]
#
#   kbd-scope-authority-template.sh supersede-exit --phase <id> --details <exit.json> \
#       --reason "<why>" --approver <principal>
#
# Every identity field (project, run, bound digests, closure key, scenario
# diff) is read from canonical state, so the manifest can only authorize the
# exact change it names. Output: canonical JSON plus one newline on stdout
# (save it to a file); the exact signing command on stderr. The approver
# principal must appear in ~/.config/prometheus/kbd-scope-allowed-signers
# with namespaces="prometheus-kbd-scope-amendment".
#
# Exit codes: 0 printed; 1 usage/io; 2 canonical state does not permit it.
set -euo pipefail

NAMESPACE="prometheus-kbd-scope-amendment"
fail() { printf 'kbd-scope-authority-template: %s\n' "$1" >&2; exit "${2:-1}"; }
command -v jq >/dev/null || fail "jq is required"
command -v prometheus >/dev/null || fail "prometheus CLI is required"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ACTION="${1:-}"; [ $# -gt 0 ] && shift
DETAILS=""; OWNER_DECISION=""; PHASE=""; CONTRACT=""; SUCCESSOR=""; TITLE=""; REASON=""; APPROVER=""
while [ $# -gt 0 ]; do
  case "$1" in
    --details) DETAILS="${2:?}"; shift 2 ;;
    --owner-decision) OWNER_DECISION="${2:?}"; shift 2 ;;
    --phase) PHASE="${2:?}"; shift 2 ;;
    --contract) CONTRACT="${2:?}"; shift 2 ;;
    --successor) SUCCESSOR="${2:?}"; shift 2 ;;
    --title) TITLE="${2:?}"; shift 2 ;;
    --reason) REASON="${2:?}"; shift 2 ;;
    --approver) APPROVER="${2:?}"; shift 2 ;;
    -h|--help) sed -n '1,22p' "$0"; exit 0 ;;
    *) fail "unknown argument: $1" ;;
  esac
done
case "$ACTION" in amend-contract|start-successor|supersede-exit) ;; *) fail "action must be amend-contract, start-successor or supersede-exit" ;; esac
[ -n "$PHASE" ] || fail "--phase is required"
[ -n "$REASON" ] || fail "--reason is required (at least 10 characters)"
[ -n "$APPROVER" ] || APPROVER="$(git config user.email 2>/dev/null || true)"
[ -n "$APPROVER" ] || fail "--approver is required"
for value in "$REASON" "$APPROVER" "$TITLE" "$SUCCESSOR" "$CONTRACT"; do
  printf '%s' "$value" | LC_ALL=C grep -q '[^ -~]' && fail "manifest text must be printable ASCII"
done

ROOT="$PWD"
while [ -n "$ROOT" ] && [ "$ROOT" != "/" ] && [ ! -d "$ROOT/.kbd-orchestrator" ]; do ROOT="$(dirname "$ROOT")"; done
[ -d "$ROOT/.kbd-orchestrator" ] || fail "no .kbd-orchestrator above $PWD"
STATUS="$(prometheus kbd --path "$ROOT" status --json)" || fail "prometheus kbd status failed"
DISP="$(prometheus kbd --path "$ROOT" phase disposition --id "$PHASE" --json)" || fail "phase $PHASE has no canonical disposition"
NONCE="$(od -An -N16 -tx1 /dev/urandom | tr -d ' \n')"
NOW="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
BASE="$(jq -n --arg ns "$NAMESPACE" --arg action "$ACTION" --arg reason "$REASON" --arg approver "$APPROVER" \
  --arg now "$NOW" --arg nonce "$NONCE" --arg phase "$PHASE" \
  --arg project "$(printf '%s' "$STATUS" | jq -r '.projectId')" --arg run "$(printf '%s' "$STATUS" | jq -r '.runId')" \
  '{schemaVersion:"1", kind:"kbd.scope-authority", namespace:$ns, action:$action, projectId:$project,
    runId:$run, phaseId:$phase, reason:$reason, approver:$approver, timestamp:$now, nonce:$nonce}')"

if [ "$ACTION" = "amend-contract" ]; then
  [ -n "$CONTRACT" ] || fail "--contract is required for amend-contract"
  [ -f "$ROOT/$CONTRACT" ] || fail "contract not found: $ROOT/$CONTRACT"
  BOUND="$(printf '%s' "$DISP" | jq -r '.binding.contractDigest // empty')"
  [ -n "$BOUND" ] || fail "phase $PHASE is not bound; adopt it first (no amendment needed)" 2
  lint_args=(--lint-only --require-status)
  [ -z "$OWNER_DECISION" ] || lint_args+=(--owner-decision "$OWNER_DECISION")
  (cd "$ROOT" && bash "$SCRIPT_DIR/kbd-contract-adopt.sh" "$CONTRACT" "${lint_args[@]}") >&2 || exit $?
  jq -e --arg phase "$PHASE" --arg project "$(jq -r '.projectId' <<<"$STATUS")" --arg run "$(jq -r '.runId' <<<"$STATUS")" '
    .identity.phaseId == $phase and .identity.projectId == $project and .identity.runId == $run
  ' "$ROOT/$CONTRACT" >/dev/null || fail "contract identity does not match target phase, project and run" 2
  NEW="$(jq -S -c 'del(.identity.contractDigest) | del(.identity.planRevision)' "$ROOT/$CONTRACT" | shasum -a 256 | awk '{print $1}')"
  [ "$NEW" != "$BOUND" ] || fail "contract is unchanged from the bound digest; nothing to amend" 2
  NEW_SCEN="$(jq -c '[.scenarioMatrix[].id]' "$ROOT/$CONTRACT")"
  NEW_MAP="{}"
  for sid in $(printf '%s' "$NEW_SCEN" | jq -r '.[]'); do
    sdig="$(jq -S -c --arg i "$sid" '.scenarioMatrix[] | select(.id == $i)' "$ROOT/$CONTRACT" | shasum -a 256 | awk '{print $1}')"
    NEW_MAP="$(printf '%s' "$NEW_MAP" | jq -c --arg k "$sid" --arg v "$sdig" '. + {($k): $v}')"
  done
  printf '%s' "$BASE" | jq -S -c --arg path "$CONTRACT" --arg prev "$BOUND" --arg new "$NEW" \
    --argjson old "$(printf '%s' "$DISP" | jq -c '.binding.scenarioDigests')" --argjson next "$NEW_MAP" '
    . + {contractPath:$path, previousContractDigest:$prev, newContractDigest:$new,
         scenarioChanges: {
           added: ([$next | keys[] | . as $k | select(($old | has($k)) | not)] | sort),
           modified: ([$next | keys[] | . as $k | select(($old | has($k)) and $old[$k] != $next[$k])] | sort),
           removed: ([$old | keys[] | . as $k | select(($next | has($k)) | not)] | sort) }}'
elif [ "$ACTION" = "supersede-exit" ]; then
  [ -f "$DETAILS" ] || fail "--details requires the operator exit details JSON file"
  candidate="$(mktemp)"
  trap 'rm -f "$candidate"' EXIT
  jq -S -c --argjson base "$BASE" --arg digest "$(jq -er '.binding.contractDigest' <<<"$DISP")" '
    if (keys | sort) != (["operatorDecision","exercised","withdrawn","budgetAccounting","limits"] | sort)
    then error("expected operatorDecision, exercised, withdrawn, budgetAccounting and limits")
    else . + $base + {boundContractDigest:$digest} end
  ' "$DETAILS" > "$candidate"
  prometheus kbd --path "$ROOT" phase validate-exit --id "$PHASE" --manifest "$candidate" >&2 || exit $?
  cat "$candidate"
else
  [ -n "$SUCCESSOR" ] && [ -n "$TITLE" ] || fail "--successor and --title are required for start-successor"
  jq -e '.disposition | IN("CLOSED","CLOSED_SUPERSEDED","CLOSED_UNMET")' <<<"$DISP" >/dev/null || fail "phase $PHASE is not CLOSED; a successor starts only from a closed phase" 2
  [ -z "$(printf '%s' "$DISP" | jq -r '.successor.phaseId // empty')" ] || fail "phase $PHASE already has a successor" 2
  printf '%s' "$BASE" | jq -S -c --arg key "$(printf '%s' "$DISP" | jq -r '.closure.closureKey')" \
    --arg successor "$SUCCESSOR" --arg title "$TITLE" \
    '. + {predecessorClosureKey:$key, successorPhaseId:$successor, successorTitle:$title}'
fi
printf 'Sign with: ssh-keygen -Y sign -f <operator-private-key> -n %s <manifest-file>\n' "$NAMESPACE" >&2
