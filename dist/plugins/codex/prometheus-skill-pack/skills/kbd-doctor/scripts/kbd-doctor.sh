#!/usr/bin/env bash
set -euo pipefail

# `kbd-doctor.sh profile [harness]` prints the harness capability profile
# (VLBV CR-23); everything else is the canonical `prometheus doctor`.
if [ "${1:-}" = "profile" ]; then
  shift
  exec node "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/harness-profile.mjs" "$@"
fi
exec prometheus doctor "$@"
