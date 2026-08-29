#!/usr/bin/env bash
set -euo pipefail

command -v prometheus >/dev/null 2>&1 || {
  printf 'kbd-bottleneck-detector: prometheus CLI is required\n' >&2
  exit 1
}
command -v jq >/dev/null 2>&1 || {
  printf 'kbd-bottleneck-detector: jq is required\n' >&2
  exit 1
}

mode="${1:-status}"
shift || true

case "$mode" in
  status)
    prometheus kbd --path . status --json | jq '{
      revision,
      lifecycle,
      position: .activePath,
      exactNextWork,
      outstandingObligations: (.boundaryObligations // {}),
      latestBoundaryReceipts: (.latestBoundaryReceipts // {}),
      activeGates: (.activeGates // {}),
      latestGateReceipts: (.latestGateReceipts // {}),
      unresolvedBlockers: [.blockers[]? | select(.resolved == false)]
    }'
    ;;
  evaluate|repair)
    boundary="${1:?usage: $mode <task|phase|zeespec> <before|after> <subject>}"
    edge="${2:?usage: $mode <task|phase|zeespec> <before|after> <subject>}"
    subject="${3:?usage: $mode <task|phase|zeespec> <before|after> <subject>}"
    args=(kbd --path . guard evaluate
      --boundary "$boundary" --edge "$edge" --subject "$subject" --json)
    [ "$mode" = "repair" ] && args+=(--repair-projections)
    prometheus "${args[@]}"
    ;;
  *)
    printf 'kbd-bottleneck-detector: unknown mode %s (use status, evaluate, or repair)\n' "$mode" >&2
    exit 64
    ;;
esac
